use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use agent_drift_analyzer::{
    analyze_bundle, AnalyzeRequest, AnalyzerError, Checkpoint, InputError as AnalyzerInputError,
};
use agent_session_compactor::{
    resolve_codex_home, BoundedClosureCompactor, BoundedClosureError, BoundedClosureRequest,
    BoundedClosureStartupReadiness, CompactorError, DiscoveryError, PreparedBoundedClosure,
};
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::input::{load_replay_bundle, CheckpointCursor, InputError};
use crate::live_input::LiveCheckpointEvent;
use crate::live_runtime::{LiveObservation, LiveRuntime, LiveRuntimeError, LiveRuntimeSnapshot};
use crate::operator_surface::WarningPolicy;
use crate::scheduler::SchedulerPolicy;

const LEGACY_LIVE_SESSION_STATE_SCHEMA_VERSION: u32 = 1;
const LEGACY_PROGRESS_LIVE_SESSION_STATE_SCHEMA_VERSION: u32 = 2;
const LIVE_SESSION_STATE_SCHEMA_VERSION: u32 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveSessionRequest {
    pub codex_home: Option<Utf8PathBuf>,
    pub session_id: String,
    pub state_dir: Utf8PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveSessionPollResult {
    pub rollout_path: Utf8PathBuf,
    pub observed_size_bytes: u64,
    pub reran_pipeline: bool,
    pub emitted_checkpoints: usize,
    pub latest_cursor: Option<CheckpointCursor>,
    pub observations: Vec<LiveObservation>,
}

impl LiveSessionPollResult {
    fn idle(
        rollout_path: Utf8PathBuf,
        observed_size_bytes: u64,
        latest_cursor: Option<CheckpointCursor>,
    ) -> Self {
        Self {
            rollout_path,
            observed_size_bytes,
            reran_pipeline: false,
            emitted_checkpoints: 0,
            latest_cursor,
            observations: Vec::new(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LiveSessionError {
    #[error(transparent)]
    BoundedClosure(#[from] BoundedClosureError),
    #[error(transparent)]
    Discovery(#[from] DiscoveryError),
    #[error(transparent)]
    Compactor(#[from] CompactorError),
    #[error(transparent)]
    Analyzer(#[from] AnalyzerError),
    #[error(transparent)]
    Input(#[from] InputError),
    #[error(transparent)]
    Runtime(#[from] LiveRuntimeError),
    #[error("failed to inspect live rollout artifact {path}: {source}")]
    InspectRollout {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("no rollout-*.jsonl artifact was found for session {session_id} under {codex_home}")]
    MissingRolloutArtifact {
        session_id: String,
        codex_home: Utf8PathBuf,
    },
    #[error(
        "multiple rollout-*.jsonl artifacts matched session {session_id} under {codex_home}: {paths:?}"
    )]
    AmbiguousRolloutArtifacts {
        session_id: String,
        codex_home: Utf8PathBuf,
        paths: Vec<Utf8PathBuf>,
    },
    #[error(
        "live rollout artifact {path} shrank from {previous_size_bytes} bytes to {current_size_bytes} bytes"
    )]
    RolloutShrank {
        path: Utf8PathBuf,
        previous_size_bytes: u64,
        current_size_bytes: u64,
    },
    #[error(
        "live analyzer bundle for session {expected_session_id} included unexpected session ids {found_session_ids:?}"
    )]
    UnexpectedCheckpointSessions {
        expected_session_id: String,
        found_session_ids: Vec<String>,
    },
    #[error("failed to read persisted live session state {path}: {source}")]
    ReadPersistedState {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse persisted live session state {path}: {source}")]
    ParsePersistedState {
        path: Utf8PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error(
        "persisted live session state {path} uses unsupported schema version {actual}; expected {expected}"
    )]
    UnsupportedPersistedStateSchema {
        path: Utf8PathBuf,
        actual: u32,
        expected: u32,
    },
    #[error(
        "persisted live session state {path} belongs to session {actual_session_id}, expected {expected_session_id}"
    )]
    PersistedStateSessionMismatch {
        path: Utf8PathBuf,
        expected_session_id: String,
        actual_session_id: String,
    },
    #[error(
        "persisted live session state {path} stored cursor {actual_session_id}:{actual_ordinal}, expected session {expected_session_id}"
    )]
    PersistedCursorSessionMismatch {
        path: Utf8PathBuf,
        expected_session_id: String,
        actual_session_id: String,
        actual_ordinal: usize,
    },
    #[error(
        "persisted cursor {session_id}:{persisted_ordinal} is ahead of analyzer-owned closure maximum {session_id}:{current_max_ordinal}"
    )]
    PersistedCursorAheadOfAnalyzerClosure {
        session_id: String,
        persisted_ordinal: usize,
        current_max_ordinal: usize,
    },
}

#[derive(Debug, Clone)]
pub struct LiveSessionCoordinator {
    request: LiveSessionRequest,
    rollout_path: Utf8PathBuf,
    closure_compactor: BoundedClosureCompactor,
    runtime: LiveRuntime,
    progress: LiveSessionProgress,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LiveSessionProgress {
    // This is the last rollout size that the coordinator fully drained and can safely treat as
    // idle on restart.
    last_observed_size_bytes: Option<u64>,
    // This token binds idle authority to the complete prepared source state that was last fully
    // processed, not to byte length or an in-memory compactor cache hit.
    #[serde(default)]
    completed_prepared_state_token: Option<String>,
    // This records the rollout size currently being drained so an interrupted poll will rerun
    // against that unchanged size instead of skipping still-undelivered checkpoints.
    #[serde(default)]
    pending_observed_size_bytes: Option<u64>,
    #[serde(default)]
    last_delivered_cursors: BTreeMap<String, CheckpointCursor>,
    #[serde(default)]
    monitor_linked_closure: bool,
    #[serde(default = "default_next_emission_ordinal")]
    next_emission_ordinal: usize,
}

impl Default for LiveSessionProgress {
    fn default() -> Self {
        Self {
            last_observed_size_bytes: None,
            completed_prepared_state_token: None,
            pending_observed_size_bytes: None,
            last_delivered_cursors: BTreeMap::new(),
            monitor_linked_closure: false,
            next_emission_ordinal: default_next_emission_ordinal(),
        }
    }
}

impl LiveSessionProgress {
    fn latest_cursor(&self, session_id: &str) -> Option<&CheckpointCursor> {
        self.last_delivered_cursors.get(session_id)
    }

    fn has_delivered_checkpoint(&self) -> bool {
        !self.last_delivered_cursors.is_empty()
    }

    fn tracks_linked_session(&self, root_session_id: &str) -> bool {
        self.last_delivered_cursors
            .keys()
            .any(|session_id| session_id != root_session_id)
    }

    fn last_observed_size_bytes(&self) -> Option<u64> {
        self.last_observed_size_bytes
    }

    fn completed_state_matches(&self, prepared_state_token: &str) -> bool {
        self.completed_prepared_state_token.as_deref() == Some(prepared_state_token)
    }

    fn largest_observed_size_bytes(&self) -> Option<u64> {
        match (
            self.last_observed_size_bytes,
            self.pending_observed_size_bytes,
        ) {
            (Some(last_completed), Some(pending)) => Some(last_completed.max(pending)),
            (Some(last_completed), None) => Some(last_completed),
            (None, Some(pending)) => Some(pending),
            (None, None) => None,
        }
    }

    fn checkpoint_is_fresh(&self, checkpoint: &Checkpoint) -> bool {
        self.last_delivered_cursors
            .get(&checkpoint.session_id)
            .is_none_or(|cursor| checkpoint_after_cursor(checkpoint, cursor))
    }

    fn checkpoint_ready_event(
        &mut self,
        checkpoint: Checkpoint,
        rollout_path: &Utf8Path,
    ) -> LiveCheckpointEvent {
        let event = LiveCheckpointEvent::checkpoint_ready(
            self.next_emission_ordinal,
            checkpoint,
            Some(rollout_path.as_str().to_string()),
        );
        self.next_emission_ordinal += 1;
        event
    }

    fn record_delivery(&mut self, cursor: CheckpointCursor) {
        self.last_delivered_cursors
            .insert(cursor.session_id.clone(), cursor);
    }

    fn begin_poll(&mut self, observed_size_bytes: u64) {
        self.completed_prepared_state_token = None;
        if self.last_observed_size_bytes != Some(observed_size_bytes) {
            self.pending_observed_size_bytes = Some(observed_size_bytes);
        }
    }

    fn complete_poll(&mut self, observed_size_bytes: u64, prepared_state_token: String) {
        self.last_observed_size_bytes = Some(observed_size_bytes);
        self.completed_prepared_state_token = Some(prepared_state_token);
        self.pending_observed_size_bytes = None;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PersistedLiveSessionState {
    schema_version: u32,
    root_session_id: String,
    progress: LiveSessionProgress,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct LegacyPersistedLiveSessionState {
    schema_version: u32,
    session_id: String,
    last_delivered_cursor: Option<CheckpointCursor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct LegacyProgressLiveSessionState {
    schema_version: u32,
    session_id: String,
    progress: LegacyLiveSessionProgress,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct LegacyLiveSessionProgress {
    last_observed_size_bytes: Option<u64>,
    #[serde(default)]
    pending_observed_size_bytes: Option<u64>,
    last_delivered_cursor: Option<CheckpointCursor>,
    #[serde(default = "default_next_emission_ordinal")]
    next_emission_ordinal: usize,
}

fn default_next_emission_ordinal() -> usize {
    1
}

impl LiveSessionCoordinator {
    pub fn new(
        mut request: LiveSessionRequest,
        scheduler_policy: SchedulerPolicy,
        warning_policy: WarningPolicy,
    ) -> Result<Self, LiveSessionError> {
        let codex_home = resolve_codex_home(request.codex_home.clone())?;
        request.codex_home = Some(codex_home.clone());
        let mut closure_compactor = BoundedClosureCompactor::default();
        let prepared = closure_compactor
            .prepare(&BoundedClosureRequest {
                codex_home: Some(codex_home.clone()),
                root_session_id: request.session_id.clone(),
            })
            .map_err(|error| map_initial_closure_error(error, &request, &codex_home))?;
        request.session_id = prepared.snapshot().root_session_id.clone();
        let rollout_path = prepared.snapshot().root_source.source_file.clone();
        let progress = load_persisted_progress(&request)?;
        Ok(Self {
            request,
            rollout_path,
            closure_compactor,
            runtime: LiveRuntime::new(scheduler_policy, warning_policy),
            progress,
        })
    }

    pub fn rollout_path(&self) -> &Utf8Path {
        &self.rollout_path
    }

    pub fn latest_cursor(&self) -> Option<&CheckpointCursor> {
        self.progress.latest_cursor(&self.request.session_id)
    }

    pub fn runtime_snapshot(&self) -> LiveRuntimeSnapshot {
        self.runtime.snapshot()
    }

    pub fn poll_once(&mut self) -> Result<LiveSessionPollResult, LiveSessionError> {
        let closure_request = BoundedClosureRequest {
            codex_home: self.request.codex_home.clone(),
            root_session_id: self.request.session_id.clone(),
        };
        let prepared = self.closure_compactor.prepare(&closure_request)?;
        let prepared_state_token = prepared.state_token().to_string();
        let snapshot = prepared.snapshot();
        let observed_size_bytes = snapshot.root_source.high_water_mark;
        let startup_readiness = snapshot.startup_readiness.clone();
        self.rollout_path = snapshot.root_source.source_file.clone();

        if let Some(previous_size_bytes) = self.progress.largest_observed_size_bytes() {
            if observed_size_bytes < previous_size_bytes {
                return Err(LiveSessionError::RolloutShrank {
                    path: self.rollout_path.clone(),
                    previous_size_bytes,
                    current_size_bytes: observed_size_bytes,
                });
            }
        }
        if let Some(previous_size_bytes) = self.progress.last_observed_size_bytes() {
            if observed_size_bytes == previous_size_bytes
                && self.progress.completed_state_matches(&prepared_state_token)
                && !self.progress.monitor_linked_closure
                && !self
                    .progress
                    .tracks_linked_session(&self.request.session_id)
            {
                return Ok(LiveSessionPollResult::idle(
                    self.rollout_path.clone(),
                    observed_size_bytes,
                    self.progress
                        .latest_cursor(&self.request.session_id)
                        .cloned(),
                ));
            }
        }

        let checkpoints = match self.run_pipeline(prepared) {
            Ok(checkpoints) => checkpoints,
            Err(error)
                if !self.progress.has_delivered_checkpoint()
                    && sparse_startup_retry_allowed(&startup_readiness, &error) =>
            {
                self.progress
                    .complete_poll(observed_size_bytes, prepared_state_token);
                self.persist_state()?;
                return Ok(LiveSessionPollResult {
                    rollout_path: self.rollout_path.clone(),
                    observed_size_bytes,
                    reran_pipeline: true,
                    emitted_checkpoints: 0,
                    latest_cursor: self
                        .progress
                        .latest_cursor(&self.request.session_id)
                        .cloned(),
                    observations: Vec::new(),
                });
            }
            Err(error) => return Err(error),
        };
        self.progress.monitor_linked_closure = checkpoints.iter().any(|checkpoint| {
            checkpoint.session_id == self.request.session_id
                && matches!(
                    checkpoint.delegation.topology,
                    agent_drift_analyzer::DelegationTopology::DelegatingParent
                )
        });
        if sparse_startup_checkpoint_emission_deferred(&self.progress, &startup_readiness) {
            self.progress
                .complete_poll(observed_size_bytes, prepared_state_token);
            self.persist_state()?;
            return Ok(LiveSessionPollResult {
                rollout_path: self.rollout_path.clone(),
                observed_size_bytes,
                reran_pipeline: true,
                emitted_checkpoints: 0,
                latest_cursor: self
                    .progress
                    .latest_cursor(&self.request.session_id)
                    .cloned(),
                observations: Vec::new(),
            });
        }
        self.progress.begin_poll(observed_size_bytes);
        let fresh_checkpoints = checkpoints
            .into_iter()
            .filter(|checkpoint| self.progress.checkpoint_is_fresh(checkpoint))
            .collect::<Vec<_>>();

        let mut observations = Vec::with_capacity(fresh_checkpoints.len());
        for checkpoint in fresh_checkpoints {
            let event = self
                .progress
                .checkpoint_ready_event(checkpoint, &self.rollout_path);
            let observation = self.runtime.observe(event)?;
            self.progress
                .record_delivery(observation.event.cursor.clone());
            self.persist_state()?;
            observations.push(observation);
        }

        self.progress
            .complete_poll(observed_size_bytes, prepared_state_token);
        self.persist_state()?;

        Ok(LiveSessionPollResult {
            rollout_path: self.rollout_path.clone(),
            observed_size_bytes,
            reran_pipeline: true,
            emitted_checkpoints: observations.len(),
            latest_cursor: self
                .progress
                .latest_cursor(&self.request.session_id)
                .cloned(),
            observations,
        })
    }

    fn run_pipeline(
        &mut self,
        prepared: PreparedBoundedClosure,
    ) -> Result<Vec<Checkpoint>, LiveSessionError> {
        fs::create_dir_all(&self.request.state_dir).map_err(|source| {
            LiveSessionError::InspectRollout {
                path: self.request.state_dir.clone(),
                source,
            }
        })?;

        let compactor_output_dir = self.compactor_output_dir();
        self.closure_compactor
            .compact(prepared, &compactor_output_dir, None)?;

        analyze_bundle(&AnalyzeRequest {
            input_dir: compactor_output_dir,
            output_dir: self.analyzer_output_dir(),
        })?;

        let bundle = load_replay_bundle(&self.analyzer_output_dir())?;
        validate_analyzer_verified_direct_closure(
            &self.request.session_id,
            &bundle.checkpoints,
            &self.progress.last_delivered_cursors,
        )?;

        Ok(bundle.checkpoints)
    }

    fn compactor_output_dir(&self) -> Utf8PathBuf {
        self.request.state_dir.join("compactor")
    }

    fn analyzer_output_dir(&self) -> Utf8PathBuf {
        self.request.state_dir.join("analyzer")
    }

    fn persist_state(&self) -> Result<(), LiveSessionError> {
        fs::create_dir_all(&self.request.state_dir).map_err(|source| {
            LiveSessionError::ReadPersistedState {
                path: self.request.state_dir.clone(),
                source,
            }
        })?;

        let state_path = persisted_state_path(&self.request);
        let temp_path = self.request.state_dir.join("live-session-state.json.tmp");
        let state = PersistedLiveSessionState {
            schema_version: LIVE_SESSION_STATE_SCHEMA_VERSION,
            root_session_id: self.request.session_id.clone(),
            progress: self.progress.clone(),
        };
        let encoded = serde_json::to_vec_pretty(&state).map_err(|source| {
            LiveSessionError::ParsePersistedState {
                path: state_path.clone(),
                source,
            }
        })?;
        fs::write(&temp_path, encoded).map_err(|source| LiveSessionError::ReadPersistedState {
            path: temp_path.clone(),
            source,
        })?;
        fs::rename(&temp_path, &state_path).map_err(|source| {
            LiveSessionError::ReadPersistedState {
                path: state_path,
                source,
            }
        })?;
        Ok(())
    }
}

fn persisted_state_path(request: &LiveSessionRequest) -> Utf8PathBuf {
    request.state_dir.join("live-session-state.json")
}

fn load_persisted_progress(
    request: &LiveSessionRequest,
) -> Result<LiveSessionProgress, LiveSessionError> {
    let state_path = persisted_state_path(request);
    if !state_path.exists() {
        return Ok(LiveSessionProgress::default());
    }

    let raw =
        fs::read_to_string(&state_path).map_err(|source| LiveSessionError::ReadPersistedState {
            path: state_path.clone(),
            source,
        })?;
    let schema_version = parse_schema_version(&raw, &state_path)?;

    match schema_version {
        LEGACY_LIVE_SESSION_STATE_SCHEMA_VERSION => {
            let state: LegacyPersistedLiveSessionState =
                serde_json::from_str(&raw).map_err(|source| {
                    LiveSessionError::ParsePersistedState {
                        path: state_path.clone(),
                        source,
                    }
                })?;
            validate_persisted_state_session(&state_path, request, &state.session_id)?;
            validate_cursor_session(
                &state_path,
                &request.session_id,
                state.last_delivered_cursor.as_ref(),
            )?;
            Ok(progress_from_legacy_cursor(
                state.last_delivered_cursor,
                default_next_emission_ordinal(),
            ))
        }
        LEGACY_PROGRESS_LIVE_SESSION_STATE_SCHEMA_VERSION => {
            let state: LegacyProgressLiveSessionState =
                serde_json::from_str(&raw).map_err(|source| {
                    LiveSessionError::ParsePersistedState {
                        path: state_path.clone(),
                        source,
                    }
                })?;
            validate_persisted_state_session(&state_path, request, &state.session_id)?;
            validate_cursor_session(
                &state_path,
                &request.session_id,
                state.progress.last_delivered_cursor.as_ref(),
            )?;
            let mut progress = progress_from_legacy_cursor(
                state.progress.last_delivered_cursor,
                state.progress.next_emission_ordinal,
            );
            progress.pending_observed_size_bytes = state.progress.pending_observed_size_bytes;
            if progress.pending_observed_size_bytes.is_some() {
                progress.last_observed_size_bytes = state.progress.last_observed_size_bytes;
            }
            Ok(progress)
        }
        LIVE_SESSION_STATE_SCHEMA_VERSION => {
            let state: PersistedLiveSessionState =
                serde_json::from_str(&raw).map_err(|source| {
                    LiveSessionError::ParsePersistedState {
                        path: state_path.clone(),
                        source,
                    }
                })?;
            validate_persisted_state_session(&state_path, request, &state.root_session_id)?;
            validate_cursor_map(&state_path, &state.progress.last_delivered_cursors)?;
            Ok(state.progress)
        }
        actual => Err(LiveSessionError::UnsupportedPersistedStateSchema {
            path: state_path,
            actual,
            expected: LIVE_SESSION_STATE_SCHEMA_VERSION,
        }),
    }
}

fn progress_from_legacy_cursor(
    cursor: Option<CheckpointCursor>,
    next_emission_ordinal: usize,
) -> LiveSessionProgress {
    let last_delivered_cursors = cursor
        .into_iter()
        .map(|cursor| (cursor.session_id.clone(), cursor))
        .collect();
    LiveSessionProgress {
        last_observed_size_bytes: None,
        completed_prepared_state_token: None,
        pending_observed_size_bytes: None,
        last_delivered_cursors,
        monitor_linked_closure: false,
        next_emission_ordinal,
    }
}

fn parse_schema_version(raw: &str, path: &Utf8Path) -> Result<u32, LiveSessionError> {
    #[derive(Deserialize)]
    struct SchemaVersionProbe {
        schema_version: u32,
    }

    let probe: SchemaVersionProbe =
        serde_json::from_str(raw).map_err(|source| LiveSessionError::ParsePersistedState {
            path: path.to_owned(),
            source,
        })?;
    Ok(probe.schema_version)
}

fn validate_persisted_state_session(
    path: &Utf8Path,
    request: &LiveSessionRequest,
    actual_session_id: &str,
) -> Result<(), LiveSessionError> {
    if actual_session_id != request.session_id {
        return Err(LiveSessionError::PersistedStateSessionMismatch {
            path: path.to_owned(),
            expected_session_id: request.session_id.clone(),
            actual_session_id: actual_session_id.to_string(),
        });
    }
    Ok(())
}

fn validate_cursor_session(
    path: &Utf8Path,
    expected_session_id: &str,
    cursor: Option<&CheckpointCursor>,
) -> Result<(), LiveSessionError> {
    if let Some(cursor) = cursor {
        if cursor.session_id != expected_session_id {
            return Err(LiveSessionError::PersistedCursorSessionMismatch {
                path: path.to_owned(),
                expected_session_id: expected_session_id.to_string(),
                actual_session_id: cursor.session_id.clone(),
                actual_ordinal: cursor.ordinal,
            });
        }
    }
    Ok(())
}

fn validate_cursor_map(
    path: &Utf8Path,
    cursors: &BTreeMap<String, CheckpointCursor>,
) -> Result<(), LiveSessionError> {
    for (session_id, cursor) in cursors {
        validate_cursor_session(path, session_id, Some(cursor))?;
    }
    Ok(())
}

fn validate_analyzer_verified_direct_closure(
    root_session_id: &str,
    checkpoints: &[Checkpoint],
    persisted_cursors: &BTreeMap<String, CheckpointCursor>,
) -> Result<(), LiveSessionError> {
    let mut expected_session_ids = BTreeSet::from([root_session_id.to_string()]);
    for checkpoint in checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.session_id == root_session_id)
    {
        if matches!(
            checkpoint.delegation.topology,
            agent_drift_analyzer::DelegationTopology::DelegatingParent
        ) {
            expected_session_ids.extend(checkpoint.delegation.child_session_ids.iter().cloned());
        }
    }

    let found_session_ids = checkpoints
        .iter()
        .map(|checkpoint| checkpoint.session_id.clone())
        .collect::<BTreeSet<_>>();
    let child_contract_is_valid = checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.session_id != root_session_id)
        .all(|checkpoint| {
            matches!(
                checkpoint.delegation.topology,
                agent_drift_analyzer::DelegationTopology::DelegatedChild
            ) && checkpoint.delegation.parent_session_id.as_deref() == Some(root_session_id)
        });
    let persisted_sessions_are_expected = persisted_cursors
        .keys()
        .all(|session_id| expected_session_ids.contains(session_id));
    if found_session_ids != expected_session_ids
        || !child_contract_is_valid
        || !persisted_sessions_are_expected
    {
        let found_session_ids = found_session_ids
            .into_iter()
            .chain(persisted_cursors.keys().cloned())
            .collect::<BTreeSet<_>>();
        return Err(LiveSessionError::UnexpectedCheckpointSessions {
            expected_session_id: root_session_id.to_string(),
            found_session_ids: found_session_ids.into_iter().collect(),
        });
    }

    for (session_id, cursor) in persisted_cursors {
        let Some(current_max_ordinal) = checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.session_id == *session_id)
            .map(|checkpoint| checkpoint.ordinal)
            .max()
        else {
            return Err(LiveSessionError::UnexpectedCheckpointSessions {
                expected_session_id: root_session_id.to_string(),
                found_session_ids: found_session_ids.iter().cloned().collect(),
            });
        };
        if current_max_ordinal < cursor.ordinal {
            return Err(LiveSessionError::PersistedCursorAheadOfAnalyzerClosure {
                session_id: session_id.clone(),
                persisted_ordinal: cursor.ordinal,
                current_max_ordinal,
            });
        }
    }

    Ok(())
}

fn checkpoint_after_cursor(checkpoint: &Checkpoint, cursor: &CheckpointCursor) -> bool {
    checkpoint.session_id > cursor.session_id
        || (checkpoint.session_id == cursor.session_id && checkpoint.ordinal > cursor.ordinal)
}

fn sparse_startup_retry_allowed(
    readiness: &BoundedClosureStartupReadiness,
    error: &LiveSessionError,
) -> bool {
    match error {
        LiveSessionError::BoundedClosure(BoundedClosureError::Discovery(
            DiscoveryError::LinkedSessionNotFound { .. },
        )) => !readiness.has_session_activity,
        LiveSessionError::Analyzer(AnalyzerError::Input(AnalyzerInputError::NoSessions {
            ..
        })) => !readiness.has_session_activity,
        LiveSessionError::Analyzer(AnalyzerError::Input(
            AnalyzerInputError::InsufficientContract { reason },
        )) => sparse_startup_contract_gap(readiness, reason),
        _ => false,
    }
}

fn sparse_startup_checkpoint_emission_deferred(
    progress: &LiveSessionProgress,
    readiness: &BoundedClosureStartupReadiness,
) -> bool {
    !progress.has_delivered_checkpoint() && !readiness.has_session_activity
}

fn sparse_startup_contract_gap(readiness: &BoundedClosureStartupReadiness, reason: &str) -> bool {
    match reason {
        "no literal user/developer/system rows survived normalization" => {
            !readiness.has_literal_directive_text
        }
        "no path-like hints survived in directive text" => !readiness.has_path_hint,
        "tool-call argument payloads are not parseable enough to infer command families and working-set paths" => {
            !readiness.has_parseable_tool_call_arguments
        }
        _ => false,
    }
}

fn map_initial_closure_error(
    error: BoundedClosureError,
    request: &LiveSessionRequest,
    codex_home: &Utf8Path,
) -> LiveSessionError {
    match error {
        BoundedClosureError::NoRolloutFiles { .. }
        | BoundedClosureError::Discovery(DiscoveryError::LinkedSessionNotFound { .. }) => {
            LiveSessionError::MissingRolloutArtifact {
                session_id: request.session_id.clone(),
                codex_home: codex_home.to_owned(),
            }
        }
        BoundedClosureError::Discovery(DiscoveryError::AmbiguousLinkedSession {
            source_files,
            ..
        }) => LiveSessionError::AmbiguousRolloutArtifacts {
            session_id: request.session_id.clone(),
            codex_home: codex_home.to_owned(),
            paths: source_files,
        },
        BoundedClosureError::Discovery(error) => LiveSessionError::Discovery(error),
        error => LiveSessionError::BoundedClosure(error),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        sparse_startup_retry_allowed, LiveSessionCoordinator, LiveSessionError,
        LiveSessionProgress, LiveSessionRequest, PersistedLiveSessionState,
    };
    use crate::input::CheckpointCursor;
    use crate::live_runtime::LiveRuntime;
    use crate::operator_surface::WarningPolicy;
    use crate::scheduler::SchedulerPolicy;
    use agent_drift_analyzer::{AnalyzerError, InputError as AnalyzerInputError};
    use agent_session_compactor::{BoundedClosureCompactor, BoundedClosureStartupReadiness};
    use camino::{Utf8Path, Utf8PathBuf};
    use tempfile::TempDir;

    #[test]
    fn same_length_begin_poll_invalidates_completed_certificate_before_delivery() {
        let temp_dir = TempDir::new().expect("temp dir");
        let state_dir = Utf8Path::from_path(temp_dir.path())
            .expect("utf8 temp dir")
            .join("state");
        let mut coordinator = LiveSessionCoordinator {
            request: LiveSessionRequest {
                codex_home: None,
                session_id: "session-live".to_string(),
                state_dir: state_dir.clone(),
            },
            rollout_path: Utf8PathBuf::from("/tmp/rollout-session-live.jsonl"),
            closure_compactor: BoundedClosureCompactor::default(),
            runtime: LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default()),
            progress: LiveSessionProgress {
                last_observed_size_bytes: Some(512),
                completed_prepared_state_token: Some("generation-a".to_string()),
                ..LiveSessionProgress::default()
            },
        };
        coordinator.progress.record_delivery(CheckpointCursor {
            session_id: "session-live".to_string(),
            ordinal: 1,
        });

        coordinator.progress.begin_poll(512);
        coordinator.progress.record_delivery(CheckpointCursor {
            session_id: "session-live".to_string(),
            ordinal: 2,
        });
        coordinator
            .persist_state()
            .expect("persist one same-length generation B delivery");

        let persisted: PersistedLiveSessionState = serde_json::from_str(
            &fs::read_to_string(state_dir.join("live-session-state.json"))
                .expect("read interrupted progress"),
        )
        .expect("parse interrupted progress");
        assert!(persisted.progress.completed_prepared_state_token.is_none());
        assert!(persisted.progress.pending_observed_size_bytes.is_none());
        assert_eq!(
            persisted
                .progress
                .latest_cursor("session-live")
                .map(|cursor| cursor.ordinal),
            Some(2)
        );
    }

    #[test]
    fn sparse_startup_retry_allows_no_sessions_only_before_session_activity() {
        let error =
            LiveSessionError::Analyzer(AnalyzerError::Input(AnalyzerInputError::NoSessions {
                input_dir: Utf8PathBuf::from("/tmp/bundle"),
            }));
        assert!(sparse_startup_retry_allowed(
            &BoundedClosureStartupReadiness::default(),
            &error
        ));

        let active = BoundedClosureStartupReadiness {
            has_session_activity: true,
            ..BoundedClosureStartupReadiness::default()
        };
        assert!(!sparse_startup_retry_allowed(&active, &error));
    }

    #[test]
    fn sparse_startup_retry_rejects_non_sparse_contract_breakage() {
        let readiness = BoundedClosureStartupReadiness {
            has_session_activity: true,
            has_literal_directive_text: true,
            has_path_hint: true,
            has_parseable_tool_call_arguments: true,
        };
        let stable_rows_error = LiveSessionError::Analyzer(AnalyzerError::Input(
            AnalyzerInputError::InsufficientContract {
                reason: "row references are not unique and stable".to_string(),
            },
        ));
        assert!(!sparse_startup_retry_allowed(
            &readiness,
            &stable_rows_error
        ));

        let missing_tool_error = LiveSessionError::Analyzer(AnalyzerError::Input(
            AnalyzerInputError::InsufficientContract {
                reason: "tool-call argument payloads are not parseable enough to infer command families and working-set paths".to_string(),
            },
        ));
        assert!(!sparse_startup_retry_allowed(
            &readiness,
            &missing_tool_error
        ));
    }

    #[test]
    fn sparse_startup_retry_uses_compactor_owned_readiness_facts() {
        let readiness = BoundedClosureStartupReadiness {
            has_session_activity: true,
            has_literal_directive_text: true,
            has_path_hint: true,
            has_parseable_tool_call_arguments: false,
        };
        let error = LiveSessionError::Analyzer(AnalyzerError::Input(
            AnalyzerInputError::InsufficientContract {
                reason: "tool-call argument payloads are not parseable enough to infer command families and working-set paths".to_string(),
            },
        ));
        assert!(sparse_startup_retry_allowed(&readiness, &error));
    }
}
