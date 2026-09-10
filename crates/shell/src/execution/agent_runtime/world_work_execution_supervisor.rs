use std::{collections::BTreeMap, path::Path, sync::LazyLock};

use anyhow::{Context as _, Result};
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use substrate_common::agent_events::{
    RuntimeEventIdentityV1, RuntimeFrameIdentityV1, RuntimeTerminalIdentityV1,
};
use transport_api_types::ExecuteStreamFrame;
use uuid::Uuid;

use super::{
    host_session_authority::{
        schema::CanonicalDirectoryV1, store::WorldWorkExecutionSupervisorStorageV1,
    },
    state_store::{
        AcceptedWorldWorkIdentityV1, PersistedWorldWorkAcceptanceV1, WorldWorkAcceptanceRecordV1,
    },
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorldWorkExecutionClaimV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) authority_revision_observed: u64,
    pub(crate) acceptance_record_id: String,
    pub(crate) acceptance_record_revision: u64,
    pub(crate) orchestration_session_id: String,
    pub(crate) caller_participant_id: String,
    pub(crate) caller_backend_id: String,
    pub(crate) target_backend_id: String,
    pub(crate) work_identity: AcceptedWorldWorkIdentityV1,
    pub(crate) world_id: String,
    pub(crate) world_generation: u64,
    pub(crate) host_transition_correlation:
        Option<substrate_common::HostTransitionWorkCorrelationV1>,
    pub(crate) stream_id: String,
    pub(crate) acceptance_frame_sequence: u64,
    pub(crate) runtime_submission_id: String,
    pub(crate) observer_instance_id: String,
    pub(crate) observer_epoch: u64,
    pub(crate) claim_revision: u64,
    pub(crate) claimed_at: DateTime<Utc>,
}

impl WorldWorkExecutionClaimV1 {
    fn from_acceptance(
        record: &WorldWorkAcceptanceRecordV1,
        observer_instance_id: &str,
    ) -> Result<Self> {
        record.validate()?;
        Ok(Self {
            schema_version: 1,
            authority_store_id: record.authority_store_id.clone(),
            authority_revision_observed: record.authority_revision_observed,
            acceptance_record_id: record.acceptance_record_id.clone(),
            acceptance_record_revision: record.record_revision,
            orchestration_session_id: record.orchestration_session_id.clone(),
            caller_participant_id: record.caller_participant_id.clone(),
            caller_backend_id: record.caller_backend_id.clone(),
            target_backend_id: record.target_backend_id.clone(),
            work_identity: record.work_identity.clone(),
            world_id: record.world_id.clone(),
            world_generation: record.world_generation,
            host_transition_correlation: record.host_transition_correlation.clone(),
            stream_id: record.runtime_acceptance.stream_id.clone(),
            acceptance_frame_sequence: record.runtime_acceptance.frame_sequence,
            runtime_submission_id: record
                .runtime_acceptance
                .runtime_submission_id
                .clone()
                .ok_or_else(|| anyhow::anyhow!("B2.1 acceptance omits runtime_submission_id"))?,
            observer_instance_id: observer_instance_id.to_string(),
            observer_epoch: 1,
            claim_revision: 1,
            claimed_at: Utc::now(),
        })
    }

    fn validate(&self, authority_store_id: &str) -> Result<()> {
        if self.schema_version != 1
            || self.acceptance_record_revision != 1
            || self.claim_revision == 0
            || self.observer_epoch == 0
            || self.acceptance_frame_sequence != 1
            || self.authority_revision_observed == 0
        {
            anyhow::bail!("B2.1 observation claim must use initial V1 revisions and Start cursor");
        }
        for (field, value) in [
            ("authority_store_id", self.authority_store_id.as_str()),
            ("acceptance_record_id", self.acceptance_record_id.as_str()),
            (
                "orchestration_session_id",
                self.orchestration_session_id.as_str(),
            ),
            ("caller_participant_id", self.caller_participant_id.as_str()),
            ("caller_backend_id", self.caller_backend_id.as_str()),
            ("target_backend_id", self.target_backend_id.as_str()),
            ("world_id", self.world_id.as_str()),
            ("stream_id", self.stream_id.as_str()),
            ("runtime_submission_id", self.runtime_submission_id.as_str()),
            ("observer_instance_id", self.observer_instance_id.as_str()),
        ] {
            if value.trim().is_empty() {
                anyhow::bail!("B2.1 observation claim omits {field}");
            }
        }
        if self.authority_store_id != authority_store_id {
            anyhow::bail!("B2.1 observation claim authority-store scope mismatch");
        }
        if let Some(correlation) = self.host_transition_correlation.as_ref() {
            correlation.validate().map_err(anyhow::Error::msg)?;
            if correlation.authority_store_id != self.authority_store_id
                || correlation.orchestration_session_id != self.orchestration_session_id
                || correlation.authoritative_participant_id != self.caller_participant_id
                || correlation.authority_revision_observed != self.authority_revision_observed
            {
                anyhow::bail!("B2.1 transition correlation does not match claim scope");
            }
        }
        match &self.work_identity {
            AcceptedWorldWorkIdentityV1::EphemeralTask { task_run_id } => {
                if task_run_id.trim().is_empty() {
                    anyhow::bail!("B2.1 ephemeral claim omits task_run_id");
                }
            }
            AcceptedWorldWorkIdentityV1::RetainedTurn {
                active_run_id,
                message_id,
                target_participant_id,
            } => {
                if active_run_id.trim().is_empty()
                    || message_id.trim().is_empty()
                    || target_participant_id.trim().is_empty()
                {
                    anyhow::bail!("B2.1 retained claim omits exact accepted identity");
                }
            }
        }
        Ok(())
    }

    pub(super) fn matches_acceptance(&self, record: &WorldWorkAcceptanceRecordV1) -> bool {
        self.authority_store_id == record.authority_store_id
            && self.authority_revision_observed == record.authority_revision_observed
            && self.acceptance_record_id == record.acceptance_record_id
            && self.acceptance_record_revision == record.record_revision
            && self.orchestration_session_id == record.orchestration_session_id
            && self.caller_participant_id == record.caller_participant_id
            && self.caller_backend_id == record.caller_backend_id
            && self.target_backend_id == record.target_backend_id
            && self.work_identity == record.work_identity
            && self.world_id == record.world_id
            && self.world_generation == record.world_generation
            && self.host_transition_correlation == record.host_transition_correlation
            && self.stream_id == record.runtime_acceptance.stream_id
            && self.acceptance_frame_sequence == record.runtime_acceptance.frame_sequence
            && record.runtime_acceptance.runtime_submission_id.as_deref()
                == Some(self.runtime_submission_id.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldWorkJournalAppendOutcomeV1 {
    Appended,
    ExactReplay,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorldWorkJournalEntryV1 {
    pub(crate) schema_version: u32,
    pub(crate) frame_identity: RuntimeFrameIdentityV1,
    pub(crate) event_identity: Option<RuntimeEventIdentityV1>,
    pub(crate) terminal_identity: Option<RuntimeTerminalIdentityV1>,
    pub(crate) canonical_ndjson_bytes: Vec<u8>,
}

impl WorldWorkJournalEntryV1 {
    fn from_frame(frame: &ExecuteStreamFrame, canonical_ndjson_bytes: Vec<u8>) -> Result<Self> {
        Ok(Self {
            schema_version: 1,
            frame_identity: frame_identity(frame).clone(),
            event_identity: frame_event_identity(frame)?.cloned(),
            terminal_identity: frame.terminal_identity().cloned(),
            canonical_ndjson_bytes,
        })
    }

    fn decode_and_validate(&self, claim: &WorldWorkExecutionClaimV1) -> Result<ExecuteStreamFrame> {
        if self.schema_version != 1 || !self.canonical_ndjson_bytes.ends_with(b"\n") {
            anyhow::bail!("B2.1 journal entry is not canonical V1 NDJSON");
        }
        let payload = &self.canonical_ndjson_bytes[..self.canonical_ndjson_bytes.len() - 1];
        let frame: ExecuteStreamFrame =
            serde_json::from_slice(payload).context("decode durable B2.1 journal frame")?;
        let canonical = frame
            .canonical_ndjson_bytes()
            .map_err(anyhow::Error::msg)
            .context("canonicalize durable B2.1 journal frame")?;
        if canonical != self.canonical_ndjson_bytes
            || frame_identity(&frame) != &self.frame_identity
            || frame_event_identity(&frame)? != self.event_identity.as_ref()
            || frame.terminal_identity() != self.terminal_identity.as_ref()
            || self.frame_identity.stream_id != claim.stream_id
        {
            anyhow::bail!("B2.1 journal entry identity or canonical bytes changed");
        }
        Ok(frame)
    }

    pub(crate) fn decode_for_c1(
        &self,
        claim: &WorldWorkExecutionClaimV1,
    ) -> Result<ExecuteStreamFrame> {
        self.decode_and_validate(claim)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorldWorkTerminalObservationV1 {
    pub(crate) schema_version: u32,
    pub(crate) frame_identity: RuntimeFrameIdentityV1,
    pub(crate) event_identity: RuntimeEventIdentityV1,
    pub(crate) terminal_identity: RuntimeTerminalIdentityV1,
    pub(crate) exit_code: i32,
    pub(crate) span_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct WorldWorkCancellationAcceptanceV1 {
    schema_version: u32,
    cancel_request_id: String,
}

const WORLD_WORK_CANCELLATION_DELIVERY_LEASE_SECONDS: i64 = 30;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case", tag = "state")]
enum WorldWorkCancellationDeliveryStateV1 {
    #[default]
    Available,
    Claimed {
        delivery_claim_id: String,
        claimed_at: DateTime<Utc>,
        lease_expires_at: DateTime<Utc>,
    },
    Confirmed {
        delivery_claim_id: String,
    },
}

impl WorldWorkCancellationDeliveryStateV1 {
    fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }

    fn validate(&self) -> Result<()> {
        match self {
            Self::Available => Ok(()),
            Self::Claimed {
                delivery_claim_id,
                claimed_at,
                lease_expires_at,
            } => {
                validate_delivery_claim_id(delivery_claim_id)?;
                let expected_expiry = claimed_at
                    .checked_add_signed(TimeDelta::seconds(
                        WORLD_WORK_CANCELLATION_DELIVERY_LEASE_SECONDS,
                    ))
                    .ok_or_else(|| {
                        anyhow::anyhow!("B4 cancellation delivery lease timestamp overflow")
                    })?;
                if *lease_expires_at != expected_expiry {
                    anyhow::bail!("B4 cancellation delivery claim has a noncanonical lease");
                }
                Ok(())
            }
            Self::Confirmed { delivery_claim_id } => validate_delivery_claim_id(delivery_claim_id),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldWorkCancellationDeliveryResultV1 {
    ConfirmedDelivered,
    ConfirmedNotDelivered,
    Ambiguous,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldWorkCancellationDeliveryRecordOutcomeV1 {
    Recorded,
    Stale,
}

fn validate_delivery_claim_id(delivery_claim_id: &str) -> Result<()> {
    if delivery_claim_id.trim().is_empty() || delivery_claim_id.trim() != delivery_claim_id {
        anyhow::bail!("B4 cancellation delivery claim identity is not canonical");
    }
    Ok(())
}

impl WorldWorkCancellationAcceptanceV1 {
    fn new(cancel_request_id: &str) -> Result<Self> {
        let acceptance = Self {
            schema_version: 1,
            cancel_request_id: cancel_request_id.to_string(),
        };
        acceptance.validate()?;
        Ok(acceptance)
    }

    fn validate(&self) -> Result<()> {
        if self.schema_version != 1
            || self.cancel_request_id.trim().is_empty()
            || self.cancel_request_id.trim() != self.cancel_request_id
        {
            anyhow::bail!("B4 cancellation acceptance is not canonical V1 material");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldWorkCancellationAcceptanceOutcomeV1 {
    pub(crate) accepted_cancel_request_id: Option<String>,
    pub(crate) should_deliver_transport: bool,
    pub(crate) delivery_claim_id: Option<String>,
    pub(crate) execution_claim: WorldWorkExecutionClaimV1,
    pub(crate) cancellation_pending: bool,
    pub(crate) terminal: Option<WorldWorkTerminalObservationV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorldWorkInterruptionReasonV1 {
    RecoveryPending,
    ReplayUnavailable,
    ReplayEndedWithoutTerminal,
    ObserverFailure,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorldWorkObservationInterruptionV1 {
    pub(crate) schema_version: u32,
    pub(crate) kind: WorldWorkInterruptionReasonV1,
    pub(crate) observer_epoch: u64,
    pub(crate) interrupted_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldWorkExecutionObservationV1 {
    pub(crate) claim: WorldWorkExecutionClaimV1,
    pub(crate) durable_frame_cursor: Option<u64>,
    pub(crate) durable_event_cursor: Option<u64>,
    pub(crate) journal: Vec<WorldWorkJournalEntryV1>,
    pub(crate) terminal: Option<WorldWorkTerminalObservationV1>,
    pub(crate) interruption: Option<WorldWorkObservationInterruptionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldWorkRecoveryAttemptV1 {
    Recovered(Vec<WorldWorkExecutionObservationV1>),
    ReceiptSnapshotStale,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct SupervisedWorldWorkExecutionV1 {
    schema_version: u32,
    claim: WorldWorkExecutionClaimV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cancellation: Option<WorldWorkCancellationAcceptanceV1>,
    #[serde(
        default,
        skip_serializing_if = "WorldWorkCancellationDeliveryStateV1::is_available"
    )]
    cancellation_delivery: WorldWorkCancellationDeliveryStateV1,
    durable_frame_cursor: Option<u64>,
    durable_event_cursor: Option<u64>,
    journal: Vec<WorldWorkJournalEntryV1>,
    terminal: Option<WorldWorkTerminalObservationV1>,
    interruption: Option<WorldWorkObservationInterruptionV1>,
}

impl SupervisedWorldWorkExecutionV1 {
    fn from_claim(claim: WorldWorkExecutionClaimV1) -> Self {
        Self {
            schema_version: 1,
            claim,
            cancellation: None,
            cancellation_delivery: WorldWorkCancellationDeliveryStateV1::Available,
            durable_frame_cursor: None,
            durable_event_cursor: None,
            journal: Vec::new(),
            terminal: None,
            interruption: None,
        }
    }

    fn observation(&self) -> WorldWorkExecutionObservationV1 {
        WorldWorkExecutionObservationV1 {
            claim: self.claim.clone(),
            durable_frame_cursor: self.durable_frame_cursor,
            durable_event_cursor: self.durable_event_cursor,
            journal: self.journal.clone(),
            terminal: self.terminal.clone(),
            interruption: self.interruption.clone(),
        }
    }

    fn validate(&self, authority_store_id: &str) -> Result<()> {
        if self.schema_version != 1 {
            anyhow::bail!("unsupported supervised world work schema version");
        }
        self.claim.validate(authority_store_id)?;
        if let Some(cancellation) = self.cancellation.as_ref() {
            cancellation.validate()?;
        }
        self.cancellation_delivery.validate()?;
        if self.cancellation.is_none() && !self.cancellation_delivery.is_available() {
            anyhow::bail!("B4 cancellation delivery state exists without acceptance truth");
        }
        let mut expected_frame_sequence = self.claim.acceptance_frame_sequence;
        let mut expected_event_sequence = 1_u64;
        let mut observed_event_cursor = None;
        let mut observed_terminal = None;
        for (index, entry) in self.journal.iter().enumerate() {
            let frame_sequence = entry.frame_identity.frame_sequence;
            if frame_sequence != expected_frame_sequence {
                anyhow::bail!("durable B2.1 journal contains a frame gap or reorder");
            }
            let frame = entry.decode_and_validate(&self.claim)?;
            if expected_frame_sequence == self.claim.acceptance_frame_sequence {
                match &frame {
                    ExecuteStreamFrame::Start { span_id, .. }
                        if span_id == &self.claim.runtime_submission_id => {}
                    _ => anyhow::bail!("durable B2.1 journal does not begin with exact Start"),
                }
            } else if matches!(frame, ExecuteStreamFrame::Start { .. }) {
                anyhow::bail!("durable B2.1 journal contains a repeated Start");
            }
            if let Some(event_identity) = entry.event_identity.as_ref() {
                if event_identity.event_sequence != expected_event_sequence {
                    anyhow::bail!("durable B2.1 journal contains an event gap or reorder");
                }
                observed_event_cursor = Some(event_identity.event_sequence);
                expected_event_sequence = expected_event_sequence.saturating_add(1);
            }
            if let ExecuteStreamFrame::Exit {
                frame_identity,
                event_identity,
                terminal_identity,
                exit,
                span_id,
                ..
            } = frame
            {
                if span_id != self.claim.runtime_submission_id
                    || frame_identity != entry.frame_identity
                {
                    anyhow::bail!("durable B2.1 terminal does not match accepted runtime");
                }
                observed_terminal = Some(WorldWorkTerminalObservationV1 {
                    schema_version: 1,
                    frame_identity,
                    event_identity,
                    terminal_identity,
                    exit_code: exit,
                    span_id,
                });
            }
            if observed_terminal.is_some() && index + 1 != self.journal.len() {
                anyhow::bail!("durable B2.1 journal contains a post-terminal frame");
            }
            expected_frame_sequence = expected_frame_sequence.saturating_add(1);
        }
        let observed_frame_cursor = self
            .journal
            .last()
            .map(|entry| entry.frame_identity.frame_sequence);
        if self.durable_frame_cursor != observed_frame_cursor
            || self.durable_event_cursor != observed_event_cursor
            || self.terminal != observed_terminal
        {
            anyhow::bail!("durable B2.1 observation cursors or terminal truth changed");
        }
        if let Some(interruption) = self.interruption.as_ref() {
            if interruption.schema_version != 1
                || interruption.observer_epoch != self.claim.observer_epoch
                || self.terminal.is_some()
            {
                anyhow::bail!("durable B2.1 interruption does not match the live observer lease");
            }
        }
        Ok(())
    }
}

fn frame_identity(frame: &ExecuteStreamFrame) -> &RuntimeFrameIdentityV1 {
    match frame {
        ExecuteStreamFrame::Start { frame_identity, .. }
        | ExecuteStreamFrame::Stdout { frame_identity, .. }
        | ExecuteStreamFrame::Stderr { frame_identity, .. }
        | ExecuteStreamFrame::Event { frame_identity, .. }
        | ExecuteStreamFrame::Exit { frame_identity, .. }
        | ExecuteStreamFrame::Error { frame_identity, .. } => frame_identity,
    }
}

fn frame_event_identity(frame: &ExecuteStreamFrame) -> Result<Option<&RuntimeEventIdentityV1>> {
    match frame {
        ExecuteStreamFrame::Event { event, .. } => event
            .event_identity
            .as_ref()
            .map(Some)
            .ok_or_else(|| anyhow::anyhow!("B2.1 Event frame omits exact event identity")),
        ExecuteStreamFrame::Exit { event_identity, .. } => Ok(Some(event_identity)),
        ExecuteStreamFrame::Start { .. }
        | ExecuteStreamFrame::Stdout { .. }
        | ExecuteStreamFrame::Stderr { .. }
        | ExecuteStreamFrame::Error { .. } => Ok(None),
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct WorldWorkExecutionSupervisorStateV1 {
    schema_version: u32,
    executions_by_acceptance_record_id: BTreeMap<String, SupervisedWorldWorkExecutionV1>,
}

impl Default for WorldWorkExecutionSupervisorStateV1 {
    fn default() -> Self {
        Self {
            schema_version: 1,
            executions_by_acceptance_record_id: BTreeMap::new(),
        }
    }
}

impl WorldWorkExecutionSupervisorStateV1 {
    fn validate(&self, authority_store_id: &str) -> Result<()> {
        if self.schema_version != 1 {
            anyhow::bail!("unsupported world work execution supervisor schema version");
        }
        for (acceptance_record_id, execution) in &self.executions_by_acceptance_record_id {
            execution.validate(authority_store_id)?;
            if acceptance_record_id != &execution.claim.acceptance_record_id {
                anyhow::bail!("B2.1 observation claim key mismatch");
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct WorldWorkExecutionSupervisor {
    storage: WorldWorkExecutionSupervisorStorageV1,
    authority_store_id: String,
    observer_instance_id: String,
}

impl WorldWorkExecutionSupervisor {
    pub(crate) fn bind(
        substrate_home: &Path,
        expected_root: &CanonicalDirectoryV1,
        expected_authority_store_id: &str,
    ) -> Result<Self> {
        Self::bind_for_observer(
            substrate_home,
            expected_root,
            expected_authority_store_id,
            current_observer_instance_id(),
        )
    }

    fn bind_for_observer(
        substrate_home: &Path,
        expected_root: &CanonicalDirectoryV1,
        expected_authority_store_id: &str,
        observer_instance_id: &str,
    ) -> Result<Self> {
        if expected_authority_store_id.trim().is_empty() {
            anyhow::bail!("B2.1 supervisor requires authority_store_id");
        }
        if observer_instance_id.trim().is_empty() {
            anyhow::bail!("B2.1 supervisor requires observer_instance_id");
        }
        let storage =
            super::host_session_authority::store::bind_world_work_execution_supervisor_storage(
                substrate_home,
                expected_root,
                expected_authority_store_id,
            )
            .context("bind activated-authority execution-supervisor storage")?;
        Ok(Self {
            storage,
            authority_store_id: expected_authority_store_id.to_string(),
            observer_instance_id: observer_instance_id.to_string(),
        })
    }

    #[cfg(test)]
    fn bind_for_test_observer(
        substrate_home: &Path,
        expected_root: &CanonicalDirectoryV1,
        expected_authority_store_id: &str,
        observer_instance_id: &str,
    ) -> Result<Self> {
        Self::bind_for_observer(
            substrate_home,
            expected_root,
            expected_authority_store_id,
            observer_instance_id,
        )
    }

    fn with_state<T>(
        &self,
        operation: impl FnOnce(&mut WorldWorkExecutionSupervisorStateV1) -> Result<(T, bool)>,
    ) -> Result<T> {
        let mut transaction = self
            .storage
            .begin_transaction()
            .context("begin activated-authority execution-supervisor transaction")?;
        let outcome = (|| {
            let mut state = match transaction
                .read_supervisor()
                .context("read canonical world work execution supervisor")?
            {
                Some(bytes) => super::host_session_authority::canonical_json::from_slice(&bytes)
                    .context("decode canonical world work execution supervisor")?,
                None => WorldWorkExecutionSupervisorStateV1::default(),
            };
            state.validate(&self.authority_store_id)?;
            let (value, changed) = operation(&mut state)?;
            state.validate(&self.authority_store_id)?;
            if changed {
                let bytes = super::host_session_authority::canonical_json::to_vec(&state)
                    .context("encode canonical world work execution supervisor")?;
                transaction
                    .replace_supervisor(&bytes)
                    .context("publish canonical world work execution supervisor")?;
            }
            Ok(value)
        })();
        let finish = transaction
            .finish()
            .context("finish activated-authority execution-supervisor transaction");
        match outcome {
            Ok(value) => finish.map(|()| value),
            Err(error) => {
                let _ = finish;
                Err(error)
            }
        }
    }

    pub(crate) fn claim_persisted_world_work(
        &self,
        acceptance: &PersistedWorldWorkAcceptanceV1,
    ) -> Result<WorldWorkExecutionClaimV1> {
        self.claim_record(acceptance.record())
    }

    fn claim_record(
        &self,
        record: &WorldWorkAcceptanceRecordV1,
    ) -> Result<WorldWorkExecutionClaimV1> {
        record.validate()?;
        if record.authority_store_id != self.authority_store_id {
            anyhow::bail!("accepted work does not match bound supervisor authority store");
        }
        self.with_state(|state| {
            if let Some(existing) = state
                .executions_by_acceptance_record_id
                .get(&record.acceptance_record_id)
            {
                if existing.claim.matches_acceptance(record) {
                    if existing.claim.observer_instance_id != self.observer_instance_id {
                        anyhow::bail!(
                            "B2.1 observation claim belongs to another runtime instance; explicit recovery is required"
                        );
                    }
                    return Ok((existing.claim.clone(), false));
                }
                anyhow::bail!("conflicting or stale B2.1 observation claim");
            }
            let claim =
                WorldWorkExecutionClaimV1::from_acceptance(record, &self.observer_instance_id)?;
            claim.validate(&self.authority_store_id)?;
            state.executions_by_acceptance_record_id.insert(
                record.acceptance_record_id.clone(),
                SupervisedWorldWorkExecutionV1::from_claim(claim.clone()),
            );
            Ok((claim, true))
        })
    }

    pub(crate) fn recover_persisted_world_work(
        &self,
        acceptances: &[PersistedWorldWorkAcceptanceV1],
    ) -> Result<WorldWorkRecoveryAttemptV1> {
        let records = acceptances
            .iter()
            .map(PersistedWorldWorkAcceptanceV1::record)
            .collect::<Vec<_>>();
        self.recover_record_refs(&records)
    }

    #[cfg(test)]
    fn recover_records(
        &self,
        records: &[WorldWorkAcceptanceRecordV1],
    ) -> Result<Vec<WorldWorkExecutionObservationV1>> {
        match self.recover_record_refs(&records.iter().collect::<Vec<_>>())? {
            WorldWorkRecoveryAttemptV1::Recovered(observations) => Ok(observations),
            WorldWorkRecoveryAttemptV1::ReceiptSnapshotStale => {
                anyhow::bail!("test recovery receipt snapshot is stale")
            }
        }
    }

    fn recover_record_refs(
        &self,
        records: &[&WorldWorkAcceptanceRecordV1],
    ) -> Result<WorldWorkRecoveryAttemptV1> {
        let mut accepted_by_id = BTreeMap::new();
        for record in records {
            record.validate()?;
            if record.authority_store_id != self.authority_store_id {
                anyhow::bail!(
                    "recovery acceptance does not match bound supervisor authority store"
                );
            }
            if accepted_by_id
                .insert(record.acceptance_record_id.as_str(), *record)
                .is_some()
            {
                anyhow::bail!("recovery acceptance identity is duplicated");
            }
        }

        self.with_state(|state| {
            for (acceptance_record_id, execution) in &state.executions_by_acceptance_record_id {
                let Some(record) = accepted_by_id.get(acceptance_record_id.as_str()) else {
                    return Ok((WorldWorkRecoveryAttemptV1::ReceiptSnapshotStale, false));
                };
                if !execution.claim.matches_acceptance(record) {
                    anyhow::bail!("supervised world work conflicts with durable acceptance truth");
                }
            }

            let mut changed = false;
            let mut recovered = Vec::new();
            for record in records {
                if !state
                    .executions_by_acceptance_record_id
                    .contains_key(&record.acceptance_record_id)
                {
                    let claim = WorldWorkExecutionClaimV1::from_acceptance(
                        record,
                        &self.observer_instance_id,
                    )?;
                    state.executions_by_acceptance_record_id.insert(
                        record.acceptance_record_id.clone(),
                        SupervisedWorldWorkExecutionV1::from_claim(claim),
                    );
                    changed = true;
                }
                let execution = state
                    .executions_by_acceptance_record_id
                    .get_mut(&record.acceptance_record_id)
                    .ok_or_else(|| anyhow::anyhow!("recovery claim insertion was not durable"))?;
                if !execution.claim.matches_acceptance(record) {
                    anyhow::bail!("recovery encountered a conflicting observation claim");
                }
                if execution.terminal.is_some() {
                    continue;
                }
                if execution.claim.observer_instance_id != self.observer_instance_id {
                    execution.claim.observer_instance_id = self.observer_instance_id.clone();
                    execution.claim.observer_epoch = execution
                        .claim
                        .observer_epoch
                        .checked_add(1)
                        .ok_or_else(|| anyhow::anyhow!("B2.1 observer epoch exhausted"))?;
                    execution.claim.claim_revision = execution
                        .claim
                        .claim_revision
                        .checked_add(1)
                        .ok_or_else(|| anyhow::anyhow!("B2.1 claim revision exhausted"))?;
                    execution.claim.claimed_at = Utc::now();
                    changed = true;
                }
                let interruption = WorldWorkObservationInterruptionV1 {
                    schema_version: 1,
                    kind: WorldWorkInterruptionReasonV1::RecoveryPending,
                    observer_epoch: execution.claim.observer_epoch,
                    interrupted_at: Utc::now(),
                };
                if execution.interruption.as_ref().map(|value| value.kind)
                    != Some(WorldWorkInterruptionReasonV1::RecoveryPending)
                    || execution
                        .interruption
                        .as_ref()
                        .map(|value| value.observer_epoch)
                        != Some(execution.claim.observer_epoch)
                {
                    execution.interruption = Some(interruption);
                    changed = true;
                }
                recovered.push(execution.observation());
            }
            Ok((WorldWorkRecoveryAttemptV1::Recovered(recovered), changed))
        })
    }

    pub(crate) fn mark_observing(&self, claim: &WorldWorkExecutionClaimV1) -> Result<()> {
        self.update_interruption(claim, None)
    }

    pub(crate) fn mark_interrupted(
        &self,
        claim: &WorldWorkExecutionClaimV1,
        reason: WorldWorkInterruptionReasonV1,
    ) -> Result<()> {
        self.update_interruption(claim, Some(reason))
    }

    fn update_interruption(
        &self,
        claim: &WorldWorkExecutionClaimV1,
        reason: Option<WorldWorkInterruptionReasonV1>,
    ) -> Result<()> {
        if claim.observer_instance_id != self.observer_instance_id {
            anyhow::bail!("stale or foreign B2.1 observer lease");
        }
        self.with_state(|state| {
            let execution = state
                .executions_by_acceptance_record_id
                .get_mut(&claim.acceptance_record_id)
                .ok_or_else(|| anyhow::anyhow!("B2.1 observation claim is absent"))?;
            if execution.claim != *claim {
                anyhow::bail!("stale or conflicting B2.1 observer lease");
            }
            if execution.terminal.is_some() {
                return Ok(((), false));
            }
            if execution.interruption.as_ref().map(|value| value.kind) == reason {
                return Ok(((), false));
            }
            execution.interruption = reason.map(|reason| WorldWorkObservationInterruptionV1 {
                schema_version: 1,
                kind: reason,
                observer_epoch: claim.observer_epoch,
                interrupted_at: Utc::now(),
            });
            Ok(((), true))
        })
    }

    pub(crate) fn journal_frame(
        &self,
        claim: &WorldWorkExecutionClaimV1,
        frame: &ExecuteStreamFrame,
        observed_ndjson_bytes: &[u8],
    ) -> Result<WorldWorkJournalAppendOutcomeV1> {
        if claim.observer_instance_id != self.observer_instance_id {
            anyhow::bail!("stale or foreign B2.1 observer lease");
        }
        let canonical_ndjson_bytes = frame
            .canonical_ndjson_bytes()
            .map_err(anyhow::Error::msg)
            .context("canonicalize observed B2.1 frame")?;
        if canonical_ndjson_bytes != observed_ndjson_bytes {
            anyhow::bail!("observed B2.1 frame bytes are not exact canonical NDJSON");
        }
        let candidate = WorldWorkJournalEntryV1::from_frame(frame, canonical_ndjson_bytes.clone())?;
        let frame_sequence = candidate.frame_identity.frame_sequence;
        self.with_state(|state| {
            let execution = state
                .executions_by_acceptance_record_id
                .get_mut(&claim.acceptance_record_id)
                .ok_or_else(|| anyhow::anyhow!("B2.1 observation claim is absent"))?;
            if execution.claim != *claim {
                anyhow::bail!("stale or conflicting B2.1 observer lease");
            }
            if let Some(existing) = execution
                .journal
                .iter()
                .find(|entry| entry.frame_identity.frame_sequence == frame_sequence)
            {
                if existing == &candidate {
                    return Ok((WorldWorkJournalAppendOutcomeV1::ExactReplay, false));
                }
                anyhow::bail!("conflicting reuse of B2.1 frame identity");
            }
            if execution.terminal.is_some() {
                anyhow::bail!("B2.1 observer rejected a post-terminal frame");
            }
            if candidate.frame_identity.stream_id != execution.claim.stream_id {
                anyhow::bail!("B2.1 frame stream identity changed");
            }
            let expected_frame_sequence = execution
                .durable_frame_cursor
                .map(|cursor| cursor.saturating_add(1))
                .unwrap_or(execution.claim.acceptance_frame_sequence);
            if frame_sequence != expected_frame_sequence {
                if frame_sequence < expected_frame_sequence {
                    anyhow::bail!("B2.1 observer rejected a reordered frame");
                }
                anyhow::bail!("B2.1 observer rejected a frame gap");
            }
            if execution.durable_frame_cursor.is_none() {
                match frame {
                    ExecuteStreamFrame::Start { span_id, .. }
                        if span_id == &execution.claim.runtime_submission_id => {}
                    _ => anyhow::bail!("B2.1 journal must begin with exact accepted Start"),
                }
            } else if matches!(frame, ExecuteStreamFrame::Start { .. }) {
                anyhow::bail!("B2.1 observer rejected a repeated Start");
            }

            let mut next_event_cursor = execution.durable_event_cursor;
            if let Some(event_identity) = candidate.event_identity.as_ref() {
                let expected_event_sequence = execution
                    .durable_event_cursor
                    .map(|cursor| cursor.saturating_add(1))
                    .unwrap_or(1);
                if event_identity.event_sequence != expected_event_sequence {
                    if event_identity.event_sequence < expected_event_sequence {
                        anyhow::bail!("B2.1 observer rejected a reordered event");
                    }
                    anyhow::bail!("B2.1 observer rejected an event gap");
                }
                next_event_cursor = Some(event_identity.event_sequence);
            }

            let terminal = match frame {
                ExecuteStreamFrame::Exit {
                    frame_identity,
                    event_identity,
                    terminal_identity,
                    exit,
                    span_id,
                    ..
                } => {
                    if span_id != &execution.claim.runtime_submission_id {
                        anyhow::bail!("B2.1 terminal span does not match accepted runtime");
                    }
                    Some(WorldWorkTerminalObservationV1 {
                        schema_version: 1,
                        frame_identity: frame_identity.clone(),
                        event_identity: event_identity.clone(),
                        terminal_identity: terminal_identity.clone(),
                        exit_code: *exit,
                        span_id: span_id.clone(),
                    })
                }
                ExecuteStreamFrame::Start { .. }
                | ExecuteStreamFrame::Stdout { .. }
                | ExecuteStreamFrame::Stderr { .. }
                | ExecuteStreamFrame::Event { .. }
                | ExecuteStreamFrame::Error { .. } => None,
            };
            execution.journal.push(candidate);
            execution.durable_frame_cursor = Some(frame_sequence);
            execution.durable_event_cursor = next_event_cursor;
            execution.terminal = terminal;
            if execution.terminal.is_some() {
                execution.interruption = None;
            }
            Ok((WorldWorkJournalAppendOutcomeV1::Appended, true))
        })
    }

    pub(crate) fn accept_or_join_cancellation(
        &self,
        expected_claim: &WorldWorkExecutionClaimV1,
        proposed_cancel_request_id: &str,
    ) -> Result<WorldWorkCancellationAcceptanceOutcomeV1> {
        self.accept_or_join_cancellation_at(
            expected_claim,
            proposed_cancel_request_id,
            Utc::now(),
            &format!("cancel_delivery_{}", Uuid::now_v7()),
        )
    }

    fn accept_or_join_cancellation_at(
        &self,
        expected_claim: &WorldWorkExecutionClaimV1,
        proposed_cancel_request_id: &str,
        now: DateTime<Utc>,
        proposed_delivery_claim_id: &str,
    ) -> Result<WorldWorkCancellationAcceptanceOutcomeV1> {
        expected_claim.validate(&self.authority_store_id)?;
        if expected_claim.observer_instance_id != self.observer_instance_id {
            anyhow::bail!("stale or foreign B4 cancellation observer lease");
        }
        let proposed = WorldWorkCancellationAcceptanceV1::new(proposed_cancel_request_id)?;
        validate_delivery_claim_id(proposed_delivery_claim_id)?;
        self.with_state(|state| {
            let execution = state
                .executions_by_acceptance_record_id
                .get_mut(&expected_claim.acceptance_record_id)
                .ok_or_else(|| anyhow::anyhow!("B4 cancellation execution claim is absent"))?;
            if execution.claim != *expected_claim {
                anyhow::bail!("stale or conflicting B4 cancellation execution claim");
            }

            if let Some(terminal) = execution.terminal.clone() {
                return Ok((
                    WorldWorkCancellationAcceptanceOutcomeV1 {
                        accepted_cancel_request_id: execution
                            .cancellation
                            .as_ref()
                            .map(|accepted| accepted.cancel_request_id.clone()),
                        should_deliver_transport: false,
                        delivery_claim_id: None,
                        execution_claim: execution.claim.clone(),
                        cancellation_pending: false,
                        terminal: Some(terminal),
                    },
                    false,
                ));
            }

            if let Some(accepted_cancel_request_id) = execution
                .cancellation
                .as_ref()
                .map(|accepted| accepted.cancel_request_id.clone())
            {
                let should_acquire = match &execution.cancellation_delivery {
                    WorldWorkCancellationDeliveryStateV1::Available => true,
                    WorldWorkCancellationDeliveryStateV1::Claimed {
                        lease_expires_at, ..
                    } => now >= *lease_expires_at,
                    WorldWorkCancellationDeliveryStateV1::Confirmed { .. } => false,
                };
                let delivery_claim_id = if should_acquire {
                    execution.cancellation_delivery =
                        cancellation_delivery_claim(proposed_delivery_claim_id, now)?;
                    Some(proposed_delivery_claim_id.to_string())
                } else {
                    None
                };
                return Ok((
                    WorldWorkCancellationAcceptanceOutcomeV1 {
                        accepted_cancel_request_id: Some(accepted_cancel_request_id),
                        should_deliver_transport: delivery_claim_id.is_some(),
                        delivery_claim_id,
                        execution_claim: execution.claim.clone(),
                        cancellation_pending: true,
                        terminal: None,
                    },
                    should_acquire,
                ));
            }

            let accepted_cancel_request_id = proposed.cancel_request_id.clone();
            execution.cancellation = Some(proposed);
            execution.cancellation_delivery =
                cancellation_delivery_claim(proposed_delivery_claim_id, now)?;
            Ok((
                WorldWorkCancellationAcceptanceOutcomeV1 {
                    accepted_cancel_request_id: Some(accepted_cancel_request_id),
                    should_deliver_transport: true,
                    delivery_claim_id: Some(proposed_delivery_claim_id.to_string()),
                    execution_claim: execution.claim.clone(),
                    cancellation_pending: true,
                    terminal: None,
                },
                true,
            ))
        })
    }

    pub(crate) fn record_cancellation_delivery_result(
        &self,
        expected_claim: &WorldWorkExecutionClaimV1,
        delivery_claim_id: &str,
        result: WorldWorkCancellationDeliveryResultV1,
    ) -> Result<WorldWorkCancellationDeliveryRecordOutcomeV1> {
        self.record_cancellation_delivery_result_at(
            expected_claim,
            delivery_claim_id,
            result,
            Utc::now(),
        )
    }

    fn record_cancellation_delivery_result_at(
        &self,
        expected_claim: &WorldWorkExecutionClaimV1,
        delivery_claim_id: &str,
        result: WorldWorkCancellationDeliveryResultV1,
        observed_at: DateTime<Utc>,
    ) -> Result<WorldWorkCancellationDeliveryRecordOutcomeV1> {
        expected_claim.validate(&self.authority_store_id)?;
        if expected_claim.observer_instance_id != self.observer_instance_id {
            anyhow::bail!("stale or foreign B4 cancellation observer lease");
        }
        validate_delivery_claim_id(delivery_claim_id)?;
        self.with_state(|state| {
            let execution = state
                .executions_by_acceptance_record_id
                .get_mut(&expected_claim.acceptance_record_id)
                .ok_or_else(|| anyhow::anyhow!("B4 cancellation execution claim is absent"))?;
            if execution.claim != *expected_claim {
                anyhow::bail!("stale or conflicting B4 cancellation execution claim");
            }
            if execution.cancellation.is_none() {
                anyhow::bail!("B4 cancellation delivery completion has no acceptance truth");
            }
            let is_current = matches!(
                &execution.cancellation_delivery,
                WorldWorkCancellationDeliveryStateV1::Claimed {
                    delivery_claim_id: current,
                    lease_expires_at,
                    ..
                } if current == delivery_claim_id && observed_at < *lease_expires_at
            );
            if !is_current {
                return Ok((WorldWorkCancellationDeliveryRecordOutcomeV1::Stale, false));
            }
            match result {
                WorldWorkCancellationDeliveryResultV1::ConfirmedDelivered => {
                    execution.cancellation_delivery =
                        WorldWorkCancellationDeliveryStateV1::Confirmed {
                            delivery_claim_id: delivery_claim_id.to_string(),
                        };
                    Ok((WorldWorkCancellationDeliveryRecordOutcomeV1::Recorded, true))
                }
                WorldWorkCancellationDeliveryResultV1::ConfirmedNotDelivered => {
                    execution.cancellation_delivery =
                        WorldWorkCancellationDeliveryStateV1::Available;
                    Ok((WorldWorkCancellationDeliveryRecordOutcomeV1::Recorded, true))
                }
                WorldWorkCancellationDeliveryResultV1::Ambiguous => Ok((
                    WorldWorkCancellationDeliveryRecordOutcomeV1::Recorded,
                    false,
                )),
            }
        })
    }

    #[allow(
        dead_code,
        reason = "B2.1-2 consumes exact durable claims for compatibility inspection and waiting"
    )]
    pub(crate) fn inspect_claim_by_acceptance_id(
        &self,
        acceptance_record_id: &str,
    ) -> Result<Option<WorldWorkExecutionClaimV1>> {
        if acceptance_record_id.trim().is_empty() {
            anyhow::bail!("B2.1 claim inspection requires acceptance_record_id");
        }
        self.with_state(|state| {
            Ok((
                state
                    .executions_by_acceptance_record_id
                    .get(acceptance_record_id)
                    .map(|execution| execution.claim.clone()),
                false,
            ))
        })
    }

    pub(crate) fn inspect_observation_by_acceptance_id(
        &self,
        acceptance_record_id: &str,
    ) -> Result<Option<WorldWorkExecutionObservationV1>> {
        if acceptance_record_id.trim().is_empty() {
            anyhow::bail!("B2.1 observation inspection requires acceptance_record_id");
        }
        self.with_state(|state| {
            Ok((
                state
                    .executions_by_acceptance_record_id
                    .get(acceptance_record_id)
                    .map(SupervisedWorldWorkExecutionV1::observation),
                false,
            ))
        })
    }
}

fn cancellation_delivery_claim(
    delivery_claim_id: &str,
    claimed_at: DateTime<Utc>,
) -> Result<WorldWorkCancellationDeliveryStateV1> {
    let lease_expires_at = claimed_at
        .checked_add_signed(TimeDelta::seconds(
            WORLD_WORK_CANCELLATION_DELIVERY_LEASE_SECONDS,
        ))
        .ok_or_else(|| anyhow::anyhow!("B4 cancellation delivery lease timestamp overflow"))?;
    Ok(WorldWorkCancellationDeliveryStateV1::Claimed {
        delivery_claim_id: delivery_claim_id.to_string(),
        claimed_at,
        lease_expires_at,
    })
}

fn current_observer_instance_id() -> &'static str {
    static INSTANCE_ID: LazyLock<String> = LazyLock::new(|| format!("observer_{}", Uuid::now_v7()));
    INSTANCE_ID.as_str()
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use chrono::Utc;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt as _;
    use substrate_common::agent_events::{
        AgentEvent, AgentEventKind, RuntimeEventIdentityV1, RuntimeFrameIdentityV1,
        RuntimeTerminalIdentityV1, RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
    };
    use transport_api_types::ExecuteStreamFrame;
    use uuid::Uuid;

    use super::*;
    use crate::execution::agent_runtime::{
        host_session_authority::schema::{
            AuthorityObjectCommitmentV1, AuthorityObjectKindV1, AuthorityObjectRefV1,
        },
        state_store::{
            AcceptedWorldWorkIdentityV1, RuntimeAcceptanceAcknowledgementKindV1,
            RuntimeAcceptanceEvidenceV1, WorldWorkAcceptanceRecordV1,
        },
    };

    fn with_supervisor<T>(f: impl FnOnce(WorldWorkExecutionSupervisor, PathBuf) -> T) -> T {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").expect("tests require HOME")).join(".cache")
            });
        fs::create_dir_all(&safe_parent).expect("create supervisor test parent");
        let temp = tempfile::tempdir_in(safe_parent).expect("create supervisor test root");
        #[cfg(unix)]
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
            .expect("secure supervisor test root");
        let authority = crate::execution::agent_runtime::host_session_authority::facade::HostSessionAuthority::open(temp.path())
            .expect("open supervisor authority facade");
        let root = authority
            .bootstrap()
            .expect("activate supervisor authority store");
        let supervisor = WorldWorkExecutionSupervisor::bind(
            temp.path(),
            &root.bootstrap_home,
            &root.authority_store_id,
        )
        .expect("bind execution supervisor");
        f(supervisor, temp.path().to_path_buf())
    }

    fn policy_ref() -> AuthorityObjectRefV1 {
        AuthorityObjectRefV1 {
            ref_id: "policy-ref-b2-1".to_string(),
            object_kind: AuthorityObjectKindV1::Policy,
            schema_version: 1,
            commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "b".repeat(64),
            },
        }
    }

    fn transition_correlation() -> substrate_common::HostTransitionWorkCorrelationV1 {
        substrate_common::HostTransitionWorkCorrelationV1 {
            schema_version: 1,
            authority_store_id: "authority-store-b2-1".to_string(),
            orchestration_session_id: "session-b2-1".to_string(),
            authoritative_participant_id: "caller-b2-1".to_string(),
            transition_intent_id: "transition-intent-b2-1".to_string(),
            transition_intent_revision_observed: 1,
            transition_run_id: "transition-run-b2-1".to_string(),
            transition_payload_commitment:
                substrate_common::OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                    digest_hex: "d".repeat(64),
                },
            authority_revision_observed: 7,
        }
    }

    fn accepted_record(
        authority_store_id: &str,
        work_identity: AcceptedWorldWorkIdentityV1,
    ) -> WorldWorkAcceptanceRecordV1 {
        let acceptance_record_id = format!("wwa_{}", Uuid::now_v7());
        let (task_run_id, active_run_id, message_id, retained_participant_id) = match &work_identity
        {
            AcceptedWorldWorkIdentityV1::EphemeralTask { task_run_id } => {
                (Some(task_run_id.clone()), None, None, None)
            }
            AcceptedWorldWorkIdentityV1::RetainedTurn {
                active_run_id,
                message_id,
                target_participant_id,
            } => (
                None,
                Some(active_run_id.clone()),
                Some(message_id.clone()),
                Some(target_participant_id.clone()),
            ),
        };
        WorldWorkAcceptanceRecordV1 {
            schema_version: 1,
            acceptance_record_id: acceptance_record_id.clone(),
            request_id: "request-b2-1".to_string(),
            authority_store_id: authority_store_id.to_string(),
            authority_revision_observed: 7,
            orchestration_session_id: "session-b2-1".to_string(),
            caller_participant_id: "caller-b2-1".to_string(),
            caller_backend_id: "cli:caller".to_string(),
            target_backend_id: "cli:target".to_string(),
            world_id: "world-b2-1".to_string(),
            world_generation: 3,
            work_identity,
            host_transition_correlation: None,
            current_policy_snapshot_ref: policy_ref(),
            current_policy_snapshot_hash: "c".repeat(64),
            current_policy_revision: "policy-revision-b2-1".to_string(),
            runtime_acceptance: RuntimeAcceptanceEvidenceV1 {
                acknowledgement_kind: RuntimeAcceptanceAcknowledgementKindV1::StartFrame,
                acceptance_record_id,
                stream_id: "stream-b2-1".to_string(),
                frame_sequence: 1,
                runtime_submission_id: task_run_id.clone().or_else(|| active_run_id.clone()),
                task_run_id,
                active_run_id,
                message_id,
                retained_participant_id,
                observed_at: Utc::now(),
            },
            accepted_at: Utc::now(),
            record_revision: 1,
        }
    }

    fn frame_identity(stream_id: &str, frame_sequence: u64) -> RuntimeFrameIdentityV1 {
        RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: stream_id.to_string(),
            frame_sequence,
        }
    }

    fn event_identity(event_sequence: u64) -> RuntimeEventIdentityV1 {
        RuntimeEventIdentityV1 {
            event_id: format!("event-b2-1-{event_sequence}"),
            event_sequence,
        }
    }

    fn cancellation_test_time() -> DateTime<Utc> {
        "2026-09-08T12:00:00Z"
            .parse()
            .expect("fixed cancellation test time")
    }

    fn event(stream_id: &str, event_sequence: u64) -> ExecuteStreamFrame {
        ExecuteStreamFrame::Event {
            frame_identity: frame_identity(stream_id, event_sequence + 1),
            event: AgentEvent {
                ts: Utc::now(),
                kind: AgentEventKind::TaskProgress,
                data: serde_json::json!({"message": format!("event {event_sequence}")}),
                agent_id: "codex-world".to_string(),
                orchestration_session_id: "session-b2-1".to_string(),
                run_id: "request-b2-1".to_string(),
                parent_run_id: None,
                participant_id: None,
                parent_participant_id: None,
                resumed_from_participant_id: None,
                backend_id: Some("cli:target".to_string()),
                thread_id: None,
                role: Some("member".to_string()),
                world_id: Some("world-b2-1".to_string()),
                world_generation: Some(3),
                cmd_id: None,
                span_id: Some("task-run-b2-1".to_string()),
                event_identity: Some(event_identity(event_sequence)),
                worker_event: None,
                channel: Some("worker.progress".to_string()),
                identity_tuple: None,
                placement_posture: None,
                project: None,
            },
        }
    }

    fn terminal(stream_id: &str, frame_sequence: u64, event_sequence: u64) -> ExecuteStreamFrame {
        let event_identity = event_identity(event_sequence);
        ExecuteStreamFrame::Exit {
            frame_identity: frame_identity(stream_id, frame_sequence),
            terminal_identity: RuntimeTerminalIdentityV1::from(&event_identity),
            event_identity,
            exit: 0,
            span_id: "task-run-b2-1".to_string(),
            scopes_used: Vec::new(),
            fs_diff: None,
            process_telemetry: Default::default(),
        }
    }

    #[test]
    #[serial_test::serial]
    fn exact_ephemeral_claim_joins_and_survives_reopen_while_conflict_fails_closed() {
        with_supervisor(|supervisor, root_path| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-b2-1".to_string(),
                },
            );
            let claim = supervisor
                .claim_record(&record)
                .expect("claim accepted ephemeral work");
            assert_eq!(claim.acceptance_record_id, record.acceptance_record_id);
            assert_eq!(claim.acceptance_record_revision, 1);
            assert_eq!(claim.stream_id, "stream-b2-1");
            assert_eq!(claim.acceptance_frame_sequence, 1);
            assert_eq!(claim.claim_revision, 1);
            assert_eq!(claim.observer_epoch, 1);
            assert_eq!(
                supervisor
                    .claim_record(&record)
                    .expect("exact duplicate claim joins"),
                claim
            );

            let authority = crate::execution::agent_runtime::host_session_authority::facade::HostSessionAuthority::open(&root_path)
                .expect("reopen supervisor authority");
            let root = authority
                .read_root()
                .expect("read supervisor authority root");
            let reopened = WorldWorkExecutionSupervisor::bind(
                &root_path,
                &root.bootstrap_home,
                &root.authority_store_id,
            )
            .expect("rebind execution supervisor");
            assert_eq!(
                reopened
                    .inspect_claim_by_acceptance_id(&record.acceptance_record_id)
                    .expect("inspect reopened claim"),
                Some(claim.clone())
            );
            let foreign_observer = WorldWorkExecutionSupervisor::bind_for_test_observer(
                &root_path,
                &root.bootstrap_home,
                &root.authority_store_id,
                "observer-other-b2-1",
            )
            .expect("bind distinct observer instance");
            let error = foreign_observer
                .claim_record(&record)
                .expect_err("distinct observer instance must not join the live claim");
            assert!(error.to_string().contains("explicit recovery"));
            let start = ExecuteStreamFrame::Start {
                frame_identity: frame_identity("stream-b2-1", 1),
                span_id: "task-run-b2-1".to_string(),
            };
            let start_bytes = start.canonical_ndjson_bytes().expect("canonical Start");
            let journal_error = foreign_observer
                .journal_frame(&claim, &start, &start_bytes)
                .expect_err("distinct observer must not journal with an inspected claim");
            assert!(journal_error.to_string().contains("foreign"));

            let mut conflicting = record;
            conflicting.runtime_acceptance.stream_id = "stream-conflict".to_string();
            assert!(reopened.claim_record(&conflicting).is_err());
            assert_eq!(
                reopened
                    .inspect_claim_by_acceptance_id(&conflicting.acceptance_record_id)
                    .expect("inspect winner after conflict"),
                Some(claim)
            );
            assert!(
                crate::execution::agent_runtime::host_session_authority::store::legacy_writer_guard(
                    &root_path,
                )
                .is_err(),
                "durable supervisor storage must not weaken activated-store legacy rejection"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn claim_preserves_optional_transition_correlation_and_rejects_mismatch() {
        with_supervisor(|supervisor, _| {
            let mut record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-b2-1".to_string(),
                },
            );
            let mut correlation = transition_correlation();
            correlation.authority_store_id = supervisor.authority_store_id.clone();
            record.host_transition_correlation = Some(correlation.clone());

            let claim = supervisor
                .claim_record(&record)
                .expect("claim correlated accepted work");
            assert_eq!(claim.host_transition_correlation, Some(correlation));

            let mut conflicting = record;
            conflicting
                .host_transition_correlation
                .as_mut()
                .expect("correlation")
                .transition_run_id = "transition-run-conflict".to_string();
            assert!(supervisor.claim_record(&conflicting).is_err());
            assert_eq!(
                supervisor
                    .inspect_claim_by_acceptance_id(&claim.acceptance_record_id)
                    .expect("inspect original correlated claim"),
                Some(claim)
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn retained_claim_binds_exact_acceptance_and_rejects_cross_store_scope() {
        with_supervisor(|supervisor, _| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::RetainedTurn {
                    active_run_id: "active-run-b2-1".to_string(),
                    message_id: format!("wwm_{}", Uuid::now_v7()),
                    target_participant_id: "worker-b2-1".to_string(),
                },
            );
            let claim = supervisor
                .claim_record(&record)
                .expect("claim accepted retained turn");
            assert_eq!(claim.work_identity, record.work_identity);
            assert_eq!(
                claim.orchestration_session_id,
                record.orchestration_session_id
            );
            assert_eq!(claim.world_id, record.world_id);
            assert_eq!(claim.world_generation, record.world_generation);

            let mut cross_store = record;
            cross_store.authority_store_id = "other-authority-store".to_string();
            assert!(supervisor.claim_record(&cross_store).is_err());
        });
    }

    #[test]
    #[serial_test::serial]
    fn journal_persists_canonical_bytes_and_exact_replay_survives_reopen() {
        with_supervisor(|supervisor, root_path| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-b2-1".to_string(),
                },
            );
            let claim = supervisor
                .claim_record(&record)
                .expect("claim work before journaling");
            let start = ExecuteStreamFrame::Start {
                frame_identity: frame_identity("stream-b2-1", 1),
                span_id: "task-run-b2-1".to_string(),
            };
            let start_bytes = start.canonical_ndjson_bytes().expect("canonical Start");

            assert_eq!(
                supervisor
                    .journal_frame(&claim, &start, &start_bytes)
                    .expect("journal canonical Start"),
                WorldWorkJournalAppendOutcomeV1::Appended
            );
            assert_eq!(
                supervisor
                    .journal_frame(&claim, &start, &start_bytes)
                    .expect("exact replay is a no-op"),
                WorldWorkJournalAppendOutcomeV1::ExactReplay
            );
            let observation = supervisor
                .inspect_observation_by_acceptance_id(&record.acceptance_record_id)
                .expect("inspect durable observation")
                .expect("observation exists");
            assert_eq!(observation.durable_frame_cursor, Some(1));
            assert_eq!(observation.durable_event_cursor, None);
            assert_eq!(observation.journal.len(), 1);
            assert_eq!(observation.journal[0].canonical_ndjson_bytes, start_bytes);
            assert_eq!(observation.terminal, None);

            let authority = crate::execution::agent_runtime::host_session_authority::facade::HostSessionAuthority::open(&root_path)
                .expect("reopen journal authority");
            let root = authority.read_root().expect("read journal authority root");
            let reopened = WorldWorkExecutionSupervisor::bind(
                &root_path,
                &root.bootstrap_home,
                &root.authority_store_id,
            )
            .expect("rebind journal supervisor");
            assert_eq!(
                reopened
                    .inspect_observation_by_acceptance_id(&record.acceptance_record_id)
                    .expect("inspect reopened observation"),
                Some(observation)
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn journal_rejects_noncanonical_gap_reorder_conflict_stale_and_post_terminal_frames() {
        with_supervisor(|supervisor, _| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-b2-1".to_string(),
                },
            );
            let claim = supervisor
                .claim_record(&record)
                .expect("claim work before invariant matrix");
            let start = ExecuteStreamFrame::Start {
                frame_identity: frame_identity("stream-b2-1", 1),
                span_id: "task-run-b2-1".to_string(),
            };
            let start_bytes = start.canonical_ndjson_bytes().expect("canonical Start");
            assert!(supervisor
                .journal_frame(&claim, &start, &start_bytes[..start_bytes.len() - 1])
                .is_err());
            supervisor
                .journal_frame(&claim, &start, &start_bytes)
                .expect("journal exact Start");

            let gap = ExecuteStreamFrame::Stdout {
                frame_identity: frame_identity("stream-b2-1", 3),
                chunk_b64: "Z2Fw".to_string(),
            };
            assert!(supervisor
                .journal_frame(
                    &claim,
                    &gap,
                    &gap.canonical_ndjson_bytes().expect("canonical gap")
                )
                .is_err());

            let conflicting_start = ExecuteStreamFrame::Start {
                frame_identity: frame_identity("stream-b2-1", 1),
                span_id: "different-task-run".to_string(),
            };
            assert!(supervisor
                .journal_frame(
                    &claim,
                    &conflicting_start,
                    &conflicting_start
                        .canonical_ndjson_bytes()
                        .expect("canonical conflicting Start")
                )
                .is_err());

            let mut stale_claim = claim.clone();
            stale_claim.observer_epoch += 1;
            let first_event = event("stream-b2-1", 1);
            assert!(supervisor
                .journal_frame(
                    &stale_claim,
                    &first_event,
                    &first_event
                        .canonical_ndjson_bytes()
                        .expect("canonical stale event")
                )
                .is_err());

            let event_gap = event("stream-b2-1", 2);
            assert!(supervisor
                .journal_frame(
                    &claim,
                    &event_gap,
                    &event_gap
                        .canonical_ndjson_bytes()
                        .expect("canonical event gap")
                )
                .is_err());
            supervisor
                .journal_frame(
                    &claim,
                    &first_event,
                    &first_event
                        .canonical_ndjson_bytes()
                        .expect("canonical first event"),
                )
                .expect("journal first event");

            let reordered_event = ExecuteStreamFrame::Event {
                frame_identity: frame_identity("stream-b2-1", 3),
                event: match event("stream-b2-1", 1) {
                    ExecuteStreamFrame::Event { event, .. } => event,
                    _ => unreachable!(),
                },
            };
            assert!(supervisor
                .journal_frame(
                    &claim,
                    &reordered_event,
                    &reordered_event
                        .canonical_ndjson_bytes()
                        .expect("canonical reordered event")
                )
                .is_err());

            let terminal = terminal("stream-b2-1", 3, 2);
            let terminal_bytes = terminal
                .canonical_ndjson_bytes()
                .expect("canonical terminal");
            assert_eq!(
                supervisor
                    .journal_frame(&claim, &terminal, &terminal_bytes)
                    .expect("journal exact terminal"),
                WorldWorkJournalAppendOutcomeV1::Appended
            );
            assert_eq!(
                supervisor
                    .journal_frame(&claim, &terminal, &terminal_bytes)
                    .expect("exact terminal replay is idempotent"),
                WorldWorkJournalAppendOutcomeV1::ExactReplay
            );

            let post_terminal = ExecuteStreamFrame::Stderr {
                frame_identity: frame_identity("stream-b2-1", 4),
                chunk_b64: "bGF0ZQ==".to_string(),
            };
            assert!(supervisor
                .journal_frame(
                    &claim,
                    &post_terminal,
                    &post_terminal
                        .canonical_ndjson_bytes()
                        .expect("canonical post-terminal frame")
                )
                .is_err());

            let observation = supervisor
                .inspect_observation_by_acceptance_id(&record.acceptance_record_id)
                .expect("inspect terminal observation")
                .expect("terminal observation exists");
            assert_eq!(observation.durable_frame_cursor, Some(3));
            assert_eq!(observation.durable_event_cursor, Some(2));
            assert_eq!(observation.journal.len(), 3);
            assert_eq!(
                observation
                    .terminal
                    .as_ref()
                    .map(|terminal| terminal.exit_code),
                Some(0)
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn restart_recovery_rotates_observer_at_exact_cursor_and_replay_converges() {
        with_supervisor(|supervisor, root_path| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-b2-1".to_string(),
                },
            );
            let authority = crate::execution::agent_runtime::host_session_authority::facade::HostSessionAuthority::open(&root_path)
                .expect("open authority");
            let root = authority.read_root().expect("read authority root");
            let old = WorldWorkExecutionSupervisor::bind_for_test_observer(
                &root_path,
                &root.bootstrap_home,
                &supervisor.authority_store_id,
                "observer-before-restart",
            )
            .expect("bind old observer");
            let old_claim = old.claim_record(&record).expect("claim before restart");
            let start = ExecuteStreamFrame::Start {
                frame_identity: frame_identity("stream-b2-1", 1),
                span_id: "task-run-b2-1".to_string(),
            };
            old.journal_frame(
                &old_claim,
                &start,
                &start.canonical_ndjson_bytes().expect("canonical Start"),
            )
            .expect("journal Start before restart");
            let first_event = event("stream-b2-1", 1);
            old.journal_frame(
                &old_claim,
                &first_event,
                &first_event
                    .canonical_ndjson_bytes()
                    .expect("canonical event"),
            )
            .expect("journal event before restart");

            let recovered = WorldWorkExecutionSupervisor::bind_for_test_observer(
                &root_path,
                &root.bootstrap_home,
                &root.authority_store_id,
                "observer-after-restart",
            )
            .expect("bind restarted observer");
            let observations = recovered
                .recover_records(std::slice::from_ref(&record))
                .expect("recover nonterminal accepted work");
            assert_eq!(observations.len(), 1);
            let observation = &observations[0];
            assert_eq!(observation.durable_frame_cursor, Some(2));
            assert_eq!(observation.durable_event_cursor, Some(1));
            assert_eq!(observation.claim.observer_epoch, 2);
            assert_eq!(observation.claim.claim_revision, 2);
            assert_eq!(
                observation.interruption.as_ref().map(|value| value.kind),
                Some(WorldWorkInterruptionReasonV1::RecoveryPending)
            );

            let late = terminal("stream-b2-1", 3, 2);
            assert!(old
                .journal_frame(
                    &old_claim,
                    &late,
                    &late
                        .canonical_ndjson_bytes()
                        .expect("canonical stale terminal"),
                )
                .is_err());
            assert_eq!(
                recovered
                    .journal_frame(
                        &observation.claim,
                        &start,
                        &start
                            .canonical_ndjson_bytes()
                            .expect("canonical replay Start"),
                    )
                    .expect("exact Start replay converges"),
                WorldWorkJournalAppendOutcomeV1::ExactReplay
            );
            recovered
                .mark_observing(&observation.claim)
                .expect("mark exact replay observer live");
            recovered
                .journal_frame(
                    &observation.claim,
                    &late,
                    &late.canonical_ndjson_bytes().expect("canonical terminal"),
                )
                .expect("journal exact terminal after recovery");
            let terminal_observation = recovered
                .inspect_observation_by_acceptance_id(&record.acceptance_record_id)
                .expect("inspect recovered terminal")
                .expect("terminal observation exists");
            assert!(terminal_observation.interruption.is_none());
            assert!(terminal_observation.terminal.is_some());
        });
    }

    #[test]
    #[serial_test::serial]
    fn accepted_without_claim_recovers_interrupted_and_never_fabricates_terminal_truth() {
        with_supervisor(|supervisor, _| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-b2-1".to_string(),
                },
            );
            let observations = supervisor
                .recover_records(std::slice::from_ref(&record))
                .expect("recover accepted work with no claim");
            assert_eq!(observations.len(), 1);
            let observation = &observations[0];
            assert_eq!(observation.durable_frame_cursor, None);
            assert_eq!(observation.terminal, None);
            assert_eq!(
                observation.interruption.as_ref().map(|value| value.kind),
                Some(WorldWorkInterruptionReasonV1::RecoveryPending)
            );
            supervisor
                .mark_interrupted(
                    &observation.claim,
                    WorldWorkInterruptionReasonV1::ReplayUnavailable,
                )
                .expect("record unavailable exact producer replay");
            let interrupted = supervisor
                .inspect_observation_by_acceptance_id(&record.acceptance_record_id)
                .expect("inspect interrupted recovery")
                .expect("interrupted observation exists");
            assert_eq!(interrupted.terminal, None);
            assert_eq!(
                interrupted.interruption.map(|value| value.kind),
                Some(WorldWorkInterruptionReasonV1::ReplayUnavailable)
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn concurrent_claim_reports_stale_receipt_snapshot_without_mutating_observers() {
        with_supervisor(|supervisor, _| {
            let first = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-first".to_string(),
                },
            );
            let second = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-concurrent".to_string(),
                },
            );
            let first_claim = supervisor.claim_record(&first).expect("claim first work");
            let second_claim = supervisor
                .claim_record(&second)
                .expect("claim concurrent work");
            assert_eq!(
                supervisor
                    .recover_record_refs(&[&first])
                    .expect("classify stale receipt snapshot"),
                WorldWorkRecoveryAttemptV1::ReceiptSnapshotStale
            );
            assert_eq!(
                supervisor
                    .inspect_claim_by_acceptance_id(&first.acceptance_record_id)
                    .expect("inspect first unchanged claim"),
                Some(first_claim)
            );
            assert_eq!(
                supervisor
                    .inspect_claim_by_acceptance_id(&second.acceptance_record_id)
                    .expect("inspect concurrent unchanged claim"),
                Some(second_claim)
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn b4_cancellation_acceptance_records_first_identity_and_replays_original_after_restart() {
        with_supervisor(|supervisor, root_path| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-b2-1".to_string(),
                },
            );
            let authority = crate::execution::agent_runtime::host_session_authority::facade::HostSessionAuthority::open(&root_path)
                .expect("open cancellation authority");
            let root = authority
                .read_root()
                .expect("read cancellation authority root");
            let before_restart = WorldWorkExecutionSupervisor::bind_for_test_observer(
                &root_path,
                &root.bootstrap_home,
                &root.authority_store_id,
                "observer-before-cancellation-restart",
            )
            .expect("bind pre-restart cancellation supervisor");
            let claim = before_restart
                .claim_record(&record)
                .expect("claim cancellation work");

            let first = before_restart
                .accept_or_join_cancellation(&claim, "cancel-request-a")
                .expect("durably accept first cancellation");
            assert_eq!(
                first.accepted_cancel_request_id.as_deref(),
                Some("cancel-request-a")
            );
            assert!(first.should_deliver_transport);
            assert!(first.cancellation_pending);
            assert!(first.terminal.is_none());

            let retry = before_restart
                .accept_or_join_cancellation(&claim, "cancel-request-b")
                .expect("join accepted cancellation");
            assert_eq!(
                retry.accepted_cancel_request_id.as_deref(),
                Some("cancel-request-a")
            );
            assert!(!retry.should_deliver_transport);
            assert!(retry.cancellation_pending);
            assert!(retry.terminal.is_none());

            let after_restart = WorldWorkExecutionSupervisor::bind_for_test_observer(
                &root_path,
                &root.bootstrap_home,
                &root.authority_store_id,
                "observer-after-cancellation-restart",
            )
            .expect("bind post-restart cancellation supervisor");
            let recovered = after_restart
                .recover_records(std::slice::from_ref(&record))
                .expect("recover accepted cancellation owner");
            assert_eq!(recovered.len(), 1);
            let replayed = after_restart
                .accept_or_join_cancellation(&recovered[0].claim, "cancel-request-c")
                .expect("join replayed cancellation owner");
            assert_eq!(
                replayed.accepted_cancel_request_id.as_deref(),
                Some("cancel-request-a")
            );
            assert!(!replayed.should_deliver_transport);
            assert!(replayed.cancellation_pending);
            assert!(replayed.terminal.is_none());
        });
    }

    #[test]
    #[serial_test::serial]
    fn b4_cancellation_delivery_confirmation_suppresses_retry_after_reopen() {
        with_supervisor(|supervisor, root_path| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-b2-1".to_string(),
                },
            );
            let claim = supervisor
                .claim_record(&record)
                .expect("claim cancellation work");
            let accepted_at = cancellation_test_time();
            let first = supervisor
                .accept_or_join_cancellation_at(
                    &claim,
                    "cancel-request-a",
                    accepted_at,
                    "delivery-claim-a",
                )
                .expect("accept cancellation with delivery claim");
            assert_eq!(first.delivery_claim_id.as_deref(), Some("delivery-claim-a"));
            assert_eq!(
                supervisor
                    .record_cancellation_delivery_result_at(
                        &claim,
                        "delivery-claim-a",
                        WorldWorkCancellationDeliveryResultV1::ConfirmedDelivered,
                        accepted_at + chrono::TimeDelta::seconds(1),
                    )
                    .expect("confirm cancellation delivery"),
                WorldWorkCancellationDeliveryRecordOutcomeV1::Recorded
            );

            let authority = crate::execution::agent_runtime::host_session_authority::facade::HostSessionAuthority::open(&root_path)
                .expect("reopen cancellation authority");
            let root = authority
                .read_root()
                .expect("read cancellation authority root");
            let reopened = WorldWorkExecutionSupervisor::bind_for_test_observer(
                &root_path,
                &root.bootstrap_home,
                &root.authority_store_id,
                "observer-after-delivery-confirmation",
            )
            .expect("bind reopened cancellation supervisor");
            let recovered = reopened
                .recover_records(std::slice::from_ref(&record))
                .expect("recover confirmed cancellation owner");
            let retry = reopened
                .accept_or_join_cancellation_at(
                    &recovered[0].claim,
                    "cancel-request-b",
                    accepted_at + chrono::TimeDelta::minutes(5),
                    "delivery-claim-b",
                )
                .expect("join confirmed cancellation");
            assert_eq!(
                retry.accepted_cancel_request_id.as_deref(),
                Some("cancel-request-a")
            );
            assert!(!retry.should_deliver_transport);
            assert_eq!(retry.delivery_claim_id, None);
        });
    }

    #[test]
    #[serial_test::serial]
    fn b4_cancellation_delivery_false_restores_immediate_retry_eligibility() {
        with_supervisor(|supervisor, _| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::RetainedTurn {
                    active_run_id: "task-run-b2-1".to_string(),
                    message_id: format!("wwm_{}", Uuid::now_v7()),
                    target_participant_id: "participant-original".to_string(),
                },
            );
            let claim = supervisor
                .claim_record(&record)
                .expect("claim retained cancellation work");
            let accepted_at = cancellation_test_time();
            let first = supervisor
                .accept_or_join_cancellation_at(
                    &claim,
                    "cancel-request-a",
                    accepted_at,
                    "delivery-claim-a",
                )
                .expect("accept retained cancellation");
            assert!(first.should_deliver_transport);
            assert_eq!(
                supervisor
                    .record_cancellation_delivery_result_at(
                        &claim,
                        "delivery-claim-a",
                        WorldWorkCancellationDeliveryResultV1::ConfirmedNotDelivered,
                        accepted_at + chrono::TimeDelta::seconds(1),
                    )
                    .expect("release undelivered cancellation claim"),
                WorldWorkCancellationDeliveryRecordOutcomeV1::Recorded
            );

            let retry = supervisor
                .accept_or_join_cancellation_at(
                    &claim,
                    "cancel-request-b",
                    accepted_at + chrono::TimeDelta::seconds(1),
                    "delivery-claim-b",
                )
                .expect("reclaim undelivered cancellation");
            assert_eq!(
                retry.accepted_cancel_request_id.as_deref(),
                Some("cancel-request-a")
            );
            assert_eq!(retry.delivery_claim_id.as_deref(), Some("delivery-claim-b"));
            assert_eq!(retry.execution_claim.work_identity, claim.work_identity);
            assert_eq!(
                retry.execution_claim.runtime_submission_id,
                claim.runtime_submission_id
            );
            assert_eq!(retry.execution_claim.stream_id, claim.stream_id);
        });
    }

    #[test]
    #[serial_test::serial]
    fn b4_cancellation_pre_send_failure_restores_immediate_retry_eligibility() {
        with_supervisor(|supervisor, _| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-b2-1".to_string(),
                },
            );
            let claim = supervisor
                .claim_record(&record)
                .expect("claim cancellation work");
            let accepted_at = cancellation_test_time();
            supervisor
                .accept_or_join_cancellation_at(
                    &claim,
                    "cancel-request-a",
                    accepted_at,
                    "delivery-claim-a",
                )
                .expect("accept cancellation before pre-send failure");
            supervisor
                .record_cancellation_delivery_result_at(
                    &claim,
                    "delivery-claim-a",
                    WorldWorkCancellationDeliveryResultV1::ConfirmedNotDelivered,
                    accepted_at,
                )
                .expect("release pre-send failure");
            let retry = supervisor
                .accept_or_join_cancellation_at(
                    &claim,
                    "cancel-request-b",
                    accepted_at,
                    "delivery-claim-b",
                )
                .expect("reclaim after pre-send failure");
            assert_eq!(
                retry.accepted_cancel_request_id.as_deref(),
                Some("cancel-request-a")
            );
            assert_eq!(retry.delivery_claim_id.as_deref(), Some("delivery-claim-b"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn b4_cancellation_abandoned_or_ambiguous_claim_recovers_only_after_lease_expiry() {
        with_supervisor(|supervisor, root_path| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-b2-1".to_string(),
                },
            );
            let claim = supervisor
                .claim_record(&record)
                .expect("claim cancellation work");
            let accepted_at = cancellation_test_time();
            supervisor
                .accept_or_join_cancellation_at(
                    &claim,
                    "cancel-request-a",
                    accepted_at,
                    "delivery-claim-a",
                )
                .expect("accept first delivery claim");
            let authority = crate::execution::agent_runtime::host_session_authority::facade::HostSessionAuthority::open(&root_path)
                .expect("reopen cancellation authority");
            let root = authority
                .read_root()
                .expect("read cancellation authority root");
            let reopened = WorldWorkExecutionSupervisor::bind_for_test_observer(
                &root_path,
                &root.bootstrap_home,
                &root.authority_store_id,
                "observer-after-ambiguous-delivery",
            )
            .expect("bind reopened cancellation supervisor");
            let recovered = reopened
                .recover_records(std::slice::from_ref(&record))
                .expect("recover ambiguous cancellation owner");
            let recovered_claim = &recovered[0].claim;
            let before_expiry = reopened
                .accept_or_join_cancellation_at(
                    recovered_claim,
                    "cancel-request-b",
                    accepted_at + chrono::TimeDelta::seconds(29),
                    "delivery-claim-b",
                )
                .expect("join unexpired delivery claim");
            assert!(!before_expiry.should_deliver_transport);
            assert_eq!(before_expiry.delivery_claim_id, None);

            let at_expiry = reopened
                .accept_or_join_cancellation_at(
                    recovered_claim,
                    "cancel-request-c",
                    accepted_at + chrono::TimeDelta::seconds(30),
                    "delivery-claim-c",
                )
                .expect("recover expired delivery claim");
            assert_eq!(
                at_expiry.accepted_cancel_request_id.as_deref(),
                Some("cancel-request-a")
            );
            assert_eq!(
                at_expiry.delivery_claim_id.as_deref(),
                Some("delivery-claim-c")
            );
            assert_eq!(at_expiry.execution_claim.work_identity, claim.work_identity);
            assert_eq!(
                at_expiry.execution_claim.runtime_submission_id,
                claim.runtime_submission_id
            );
            assert_eq!(at_expiry.execution_claim.stream_id, claim.stream_id);
            reopened
                .record_cancellation_delivery_result_at(
                    recovered_claim,
                    "delivery-claim-c",
                    WorldWorkCancellationDeliveryResultV1::Ambiguous,
                    accepted_at + chrono::TimeDelta::seconds(31),
                )
                .expect("retain ambiguous recovery claim");
            let ambiguous_before_expiry = reopened
                .accept_or_join_cancellation_at(
                    recovered_claim,
                    "cancel-request-d",
                    accepted_at + chrono::TimeDelta::seconds(59),
                    "delivery-claim-d",
                )
                .expect("join unexpired ambiguous claim");
            assert_eq!(ambiguous_before_expiry.delivery_claim_id, None);
            let ambiguous_at_expiry = reopened
                .accept_or_join_cancellation_at(
                    recovered_claim,
                    "cancel-request-e",
                    accepted_at + chrono::TimeDelta::seconds(60),
                    "delivery-claim-e",
                )
                .expect("recover ambiguous claim at lease expiry");
            assert_eq!(
                ambiguous_at_expiry.delivery_claim_id.as_deref(),
                Some("delivery-claim-e")
            );
            assert_eq!(
                ambiguous_at_expiry.accepted_cancel_request_id.as_deref(),
                Some("cancel-request-a")
            );
            assert_eq!(
                ambiguous_at_expiry.execution_claim.runtime_submission_id,
                claim.runtime_submission_id
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn b4_retained_cancellation_ambiguous_claim_blocks_retry_until_expiry() {
        with_supervisor(|supervisor, _| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::RetainedTurn {
                    active_run_id: "active-run-original".to_string(),
                    message_id: format!("wwm_{}", Uuid::now_v7()),
                    target_participant_id: "participant-original".to_string(),
                },
            );
            let claim = supervisor
                .claim_record(&record)
                .expect("claim retained cancellation work");
            let accepted_at = cancellation_test_time();
            let first = supervisor
                .accept_or_join_cancellation_at(
                    &claim,
                    "cancel-request-original",
                    accepted_at,
                    "delivery-claim-original",
                )
                .expect("accept retained cancellation");
            assert!(first.should_deliver_transport);
            supervisor
                .record_cancellation_delivery_result_at(
                    &claim,
                    "delivery-claim-original",
                    WorldWorkCancellationDeliveryResultV1::Ambiguous,
                    accepted_at + chrono::TimeDelta::seconds(1),
                )
                .expect("retain ambiguous retained delivery claim");

            let immediate = supervisor
                .accept_or_join_cancellation_at(
                    &claim,
                    "cancel-request-retry",
                    accepted_at + chrono::TimeDelta::seconds(1),
                    "delivery-claim-retry",
                )
                .expect("join ambiguous retained cancellation");
            assert_eq!(
                immediate.accepted_cancel_request_id.as_deref(),
                Some("cancel-request-original")
            );
            assert!(!immediate.should_deliver_transport);
            assert_eq!(immediate.delivery_claim_id, None);

            let recovered = supervisor
                .accept_or_join_cancellation_at(
                    &claim,
                    "cancel-request-after-expiry",
                    accepted_at + chrono::TimeDelta::seconds(30),
                    "delivery-claim-recovered",
                )
                .expect("recover ambiguous retained cancellation");
            assert_eq!(
                recovered.accepted_cancel_request_id.as_deref(),
                Some("cancel-request-original")
            );
            assert_eq!(
                recovered.delivery_claim_id.as_deref(),
                Some("delivery-claim-recovered")
            );
            assert_eq!(recovered.execution_claim.work_identity, claim.work_identity);
            assert_eq!(
                recovered.execution_claim.runtime_submission_id,
                claim.runtime_submission_id
            );
            assert_eq!(recovered.execution_claim.stream_id, claim.stream_id);
        });
    }

    #[test]
    #[serial_test::serial]
    fn b4_cancellation_concurrent_callers_obtain_at_most_one_unexpired_delivery_claim() {
        with_supervisor(|supervisor, _| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-b2-1".to_string(),
                },
            );
            let claim = supervisor
                .claim_record(&record)
                .expect("claim cancellation work");
            let accepted_at = cancellation_test_time();
            let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
            let mut callers = Vec::new();
            for index in 0..8 {
                let supervisor = supervisor.clone();
                let claim = claim.clone();
                let barrier = barrier.clone();
                callers.push(std::thread::spawn(move || {
                    barrier.wait();
                    supervisor
                        .accept_or_join_cancellation_at(
                            &claim,
                            &format!("cancel-request-{index}"),
                            accepted_at,
                            &format!("delivery-claim-{index}"),
                        )
                        .expect("concurrent cancellation joins durable owner")
                }));
            }
            let outcomes = callers
                .into_iter()
                .map(|caller| caller.join().expect("concurrent caller joins"))
                .collect::<Vec<_>>();
            assert_eq!(
                outcomes
                    .iter()
                    .filter(|outcome| outcome.delivery_claim_id.is_some())
                    .count(),
                1
            );
            let accepted_ids = outcomes
                .iter()
                .map(|outcome| outcome.accepted_cancel_request_id.as_deref())
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(accepted_ids.len(), 1);
        });
    }

    #[test]
    #[serial_test::serial]
    fn b4_cancellation_stale_completion_cannot_overwrite_newer_delivery_claim() {
        with_supervisor(|supervisor, _| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-b2-1".to_string(),
                },
            );
            let claim = supervisor
                .claim_record(&record)
                .expect("claim cancellation work");
            let accepted_at = cancellation_test_time();
            supervisor
                .accept_or_join_cancellation_at(
                    &claim,
                    "cancel-request-a",
                    accepted_at,
                    "delivery-claim-a",
                )
                .expect("accept first delivery claim");
            assert_eq!(
                supervisor
                    .record_cancellation_delivery_result_at(
                        &claim,
                        "delivery-claim-a",
                        WorldWorkCancellationDeliveryResultV1::ConfirmedDelivered,
                        accepted_at + chrono::TimeDelta::seconds(30),
                    )
                    .expect("ignore completion at expired delivery lease"),
                WorldWorkCancellationDeliveryRecordOutcomeV1::Stale
            );
            let recovered = supervisor
                .accept_or_join_cancellation_at(
                    &claim,
                    "cancel-request-b",
                    accepted_at + chrono::TimeDelta::seconds(30),
                    "delivery-claim-b",
                )
                .expect("recover expired delivery claim");
            assert_eq!(
                recovered.delivery_claim_id.as_deref(),
                Some("delivery-claim-b")
            );
            assert_eq!(
                supervisor
                    .record_cancellation_delivery_result_at(
                        &claim,
                        "delivery-claim-a",
                        WorldWorkCancellationDeliveryResultV1::ConfirmedDelivered,
                        accepted_at + chrono::TimeDelta::seconds(31),
                    )
                    .expect("ignore stale delivery completion"),
                WorldWorkCancellationDeliveryRecordOutcomeV1::Stale
            );
            assert_eq!(
                supervisor
                    .record_cancellation_delivery_result_at(
                        &claim,
                        "delivery-claim-b",
                        WorldWorkCancellationDeliveryResultV1::ConfirmedNotDelivered,
                        accepted_at + chrono::TimeDelta::seconds(31),
                    )
                    .expect("release current delivery claim"),
                WorldWorkCancellationDeliveryRecordOutcomeV1::Recorded
            );
            let third = supervisor
                .accept_or_join_cancellation_at(
                    &claim,
                    "cancel-request-c",
                    accepted_at + chrono::TimeDelta::seconds(31),
                    "delivery-claim-c",
                )
                .expect("claim after current release");
            assert_eq!(third.delivery_claim_id.as_deref(), Some("delivery-claim-c"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn b4_legacy_cancellation_without_delivery_state_remains_recoverable() {
        with_supervisor(|supervisor, _| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-b2-1".to_string(),
                },
            );
            let claim = supervisor
                .claim_record(&record)
                .expect("claim cancellation work");
            let accepted_at = cancellation_test_time();
            supervisor
                .accept_or_join_cancellation_at(
                    &claim,
                    "cancel-request-a",
                    accepted_at,
                    "delivery-claim-a",
                )
                .expect("persist modern cancellation fixture");

            let mut transaction = supervisor
                .storage
                .begin_transaction()
                .expect("begin legacy cancellation fixture read");
            let bytes = transaction
                .read_supervisor()
                .expect("read modern cancellation fixture")
                .expect("modern cancellation fixture exists");
            transaction
                .finish()
                .expect("finish legacy cancellation fixture read");
            let mut value: serde_json::Value =
                serde_json::from_slice(&bytes).expect("decode modern cancellation fixture");
            let executions = value
                .get_mut("executions_by_acceptance_record_id")
                .and_then(serde_json::Value::as_object_mut)
                .expect("execution map");
            let execution = executions
                .get_mut(&record.acceptance_record_id)
                .and_then(serde_json::Value::as_object_mut)
                .expect("exact execution");
            execution.remove("cancellation_delivery");
            let legacy_bytes = super::super::host_session_authority::canonical_json::to_vec(&value)
                .expect("encode canonical legacy cancellation fixture");
            let mut transaction = supervisor
                .storage
                .begin_transaction()
                .expect("begin legacy cancellation fixture write");
            transaction
                .replace_supervisor(&legacy_bytes)
                .expect("publish legacy cancellation fixture");
            transaction
                .finish()
                .expect("finish legacy cancellation fixture write");

            let recovered = supervisor
                .accept_or_join_cancellation_at(
                    &claim,
                    "cancel-request-b",
                    accepted_at + chrono::TimeDelta::seconds(1),
                    "delivery-claim-b",
                )
                .expect("recover legacy cancellation delivery");
            assert_eq!(
                recovered.accepted_cancel_request_id.as_deref(),
                Some("cancel-request-a")
            );
            assert_eq!(
                recovered.delivery_claim_id.as_deref(),
                Some("delivery-claim-b")
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn b4_terminal_truth_suppresses_transport_independently_of_delivery_state() {
        with_supervisor(|supervisor, _| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-b2-1".to_string(),
                },
            );
            let claim = supervisor
                .claim_record(&record)
                .expect("claim cancellation work");
            let accepted_at = cancellation_test_time();
            supervisor
                .accept_or_join_cancellation_at(
                    &claim,
                    "cancel-request-a",
                    accepted_at,
                    "delivery-claim-a",
                )
                .expect("accept cancellation before terminal truth");
            let start = ExecuteStreamFrame::Start {
                frame_identity: frame_identity("stream-b2-1", 1),
                span_id: "task-run-b2-1".to_string(),
            };
            supervisor
                .journal_frame(
                    &claim,
                    &start,
                    &start.canonical_ndjson_bytes().expect("canonical Start"),
                )
                .expect("journal Start");
            let terminal = terminal("stream-b2-1", 2, 1);
            supervisor
                .journal_frame(
                    &claim,
                    &terminal,
                    &terminal
                        .canonical_ndjson_bytes()
                        .expect("canonical terminal"),
                )
                .expect("journal exact terminal truth");

            let retry = supervisor
                .accept_or_join_cancellation_at(
                    &claim,
                    "cancel-request-b",
                    accepted_at + chrono::TimeDelta::minutes(5),
                    "delivery-claim-b",
                )
                .expect("terminal truth wins over expired delivery claim");
            assert_eq!(
                retry.accepted_cancel_request_id.as_deref(),
                Some("cancel-request-a")
            );
            assert!(!retry.should_deliver_transport);
            assert_eq!(retry.delivery_claim_id, None);
            assert_eq!(retry.terminal.map(|terminal| terminal.exit_code), Some(0));
        });
    }

    #[test]
    #[serial_test::serial]
    fn b4_cancellation_acceptance_rejects_claim_drift_and_terminal_before_cancel() {
        with_supervisor(|supervisor, _| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-b2-1".to_string(),
                },
            );
            let claim = supervisor
                .claim_record(&record)
                .expect("claim cancellation work");
            let mut drifted = claim.clone();
            drifted.world_generation += 1;
            assert!(supervisor
                .accept_or_join_cancellation(&drifted, "cancel-request-drift")
                .is_err());
            let first = supervisor
                .accept_or_join_cancellation(&claim, "cancel-request-a")
                .expect("claim drift must not mutate accepted cancellation");
            assert_eq!(
                first.accepted_cancel_request_id.as_deref(),
                Some("cancel-request-a")
            );
            assert!(first.should_deliver_transport);

            let terminal_record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-b2-1".to_string(),
                },
            );
            let terminal_claim = supervisor
                .claim_record(&terminal_record)
                .expect("claim terminal-before-cancel work");
            let start = ExecuteStreamFrame::Start {
                frame_identity: frame_identity("stream-b2-1", 1),
                span_id: "task-run-b2-1".to_string(),
            };
            supervisor
                .journal_frame(
                    &terminal_claim,
                    &start,
                    &start.canonical_ndjson_bytes().expect("canonical Start"),
                )
                .expect("journal terminal-before-cancel Start");
            let terminal = terminal("stream-b2-1", 2, 1);
            supervisor
                .journal_frame(
                    &terminal_claim,
                    &terminal,
                    &terminal
                        .canonical_ndjson_bytes()
                        .expect("canonical terminal"),
                )
                .expect("journal terminal-before-cancel Exit");

            let outcome = supervisor
                .accept_or_join_cancellation(&terminal_claim, "must-not-be-accepted")
                .expect("terminal truth wins before cancellation acceptance");
            assert_eq!(outcome.accepted_cancel_request_id, None);
            assert!(!outcome.should_deliver_transport);
            assert!(!outcome.cancellation_pending);
            assert_eq!(outcome.terminal.map(|terminal| terminal.exit_code), Some(0));
        });
    }

    #[test]
    #[serial_test::serial]
    fn b4_cancellation_acceptance_rejects_malformed_conflicting_and_noncanonical_material() {
        with_supervisor(|supervisor, _| {
            let record = accepted_record(
                &supervisor.authority_store_id,
                AcceptedWorldWorkIdentityV1::EphemeralTask {
                    task_run_id: "task-run-b2-1".to_string(),
                },
            );
            let claim = supervisor
                .claim_record(&record)
                .expect("claim cancellation corruption fixture");
            supervisor
                .accept_or_join_cancellation(&claim, "cancel-request-a")
                .expect("persist valid cancellation fixture");

            let mut transaction = supervisor
                .storage
                .begin_transaction()
                .expect("begin cancellation fixture read");
            let valid_bytes = transaction
                .read_supervisor()
                .expect("read valid cancellation fixture")
                .expect("valid cancellation fixture exists");
            transaction
                .finish()
                .expect("finish cancellation fixture read");
            let valid = String::from_utf8(valid_bytes.clone()).expect("canonical UTF-8 fixture");
            let canonical =
                r#""cancellation":{"cancel_request_id":"cancel-request-a","schema_version":1}"#;
            assert!(valid.contains(canonical));
            let corruptions = [
                valid.replace(canonical, r#""cancellation":{"schema_version":1}"#),
                valid.replace(
                    canonical,
                    r#""cancellation":{"cancel_request_id":"cancel-request-a","cancel_request_id":"cancel-request-b","schema_version":1}"#,
                ),
                valid.replace(
                    canonical,
                    r#""cancellation":{"schema_version":1,"cancel_request_id":"cancel-request-a"}"#,
                ),
            ];

            for corrupted in corruptions {
                let mut transaction = supervisor
                    .storage
                    .begin_transaction()
                    .expect("begin cancellation corruption write");
                transaction
                    .replace_supervisor(corrupted.as_bytes())
                    .expect("publish cancellation corruption fixture");
                transaction
                    .finish()
                    .expect("finish cancellation corruption write");
                assert!(supervisor
                    .accept_or_join_cancellation(&claim, "cancel-request-b")
                    .is_err());

                let mut transaction = supervisor
                    .storage
                    .begin_transaction()
                    .expect("begin cancellation fixture restore");
                transaction
                    .replace_supervisor(&valid_bytes)
                    .expect("restore valid cancellation fixture");
                transaction
                    .finish()
                    .expect("finish cancellation fixture restore");
            }

            let retry = supervisor
                .accept_or_join_cancellation(&claim, "cancel-request-b")
                .expect("valid cancellation owner survives rejected corruption");
            assert_eq!(
                retry.accepted_cancel_request_id.as_deref(),
                Some("cancel-request-a")
            );
            assert!(!retry.should_deliver_transport);
        });
    }
}
