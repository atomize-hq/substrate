use std::fs;
use std::io::{BufRead, BufReader};

use agent_drift_analyzer::{
    analyze_bundle, AnalyzeRequest, AnalyzerError, Checkpoint, InputError as AnalyzerInputError,
};
use agent_session_compactor::{
    compact_codex_sessions, discover_session_artifacts, CompactorError, DiscoverOptions,
    DiscoveryError, RunConfig,
};
use camino::{Utf8Path, Utf8PathBuf};
use serde_json::Value;

use crate::input::{load_replay_bundle, CheckpointCursor, InputError};
use crate::live_input::LiveCheckpointEvent;
use crate::live_runtime::{LiveObservation, LiveRuntime, LiveRuntimeError, LiveRuntimeSnapshot};
use crate::operator_surface::WarningPolicy;
use crate::scheduler::SchedulerPolicy;

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
}

#[derive(Debug, Clone)]
pub struct LiveSessionCoordinator {
    request: LiveSessionRequest,
    rollout_path: Utf8PathBuf,
    runtime: LiveRuntime,
    last_observed_size_bytes: Option<u64>,
    last_delivered_cursor: Option<CheckpointCursor>,
    next_emission_ordinal: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct RolloutStartupReadiness {
    has_session_activity: bool,
    has_literal_directive_text: bool,
    has_path_hint: bool,
    has_parseable_tool_call_arguments: bool,
}

impl LiveSessionCoordinator {
    pub fn new(
        request: LiveSessionRequest,
        scheduler_policy: SchedulerPolicy,
        warning_policy: WarningPolicy,
    ) -> Result<Self, LiveSessionError> {
        let rollout_path = resolve_rollout_path(&request)?;
        Ok(Self {
            request,
            rollout_path,
            runtime: LiveRuntime::new(scheduler_policy, warning_policy),
            last_observed_size_bytes: None,
            last_delivered_cursor: None,
            next_emission_ordinal: 1,
        })
    }

    pub fn rollout_path(&self) -> &Utf8Path {
        &self.rollout_path
    }

    pub fn latest_cursor(&self) -> Option<&CheckpointCursor> {
        self.last_delivered_cursor.as_ref()
    }

    pub fn runtime_snapshot(&self) -> LiveRuntimeSnapshot {
        self.runtime.snapshot()
    }

    pub fn poll_once(&mut self) -> Result<LiveSessionPollResult, LiveSessionError> {
        let observed_size_bytes = file_size_bytes(&self.rollout_path)?;
        if let Some(previous_size_bytes) = self.last_observed_size_bytes {
            if observed_size_bytes < previous_size_bytes {
                return Err(LiveSessionError::RolloutShrank {
                    path: self.rollout_path.clone(),
                    previous_size_bytes,
                    current_size_bytes: observed_size_bytes,
                });
            }
            if observed_size_bytes == previous_size_bytes {
                return Ok(LiveSessionPollResult::idle(
                    self.rollout_path.clone(),
                    observed_size_bytes,
                    self.last_delivered_cursor.clone(),
                ));
            }
        }

        let checkpoints = match self.run_pipeline() {
            Ok(checkpoints) => checkpoints,
            Err(error)
                if self.last_delivered_cursor.is_none()
                    && sparse_startup_retry_allowed(&self.rollout_path, &error) =>
            {
                self.last_observed_size_bytes = Some(observed_size_bytes);
                return Ok(LiveSessionPollResult {
                    rollout_path: self.rollout_path.clone(),
                    observed_size_bytes,
                    reran_pipeline: true,
                    emitted_checkpoints: 0,
                    latest_cursor: None,
                    observations: Vec::new(),
                });
            }
            Err(error) => return Err(error),
        };
        let fresh_checkpoints = checkpoints
            .into_iter()
            .filter(|checkpoint| {
                self.last_delivered_cursor
                    .as_ref()
                    .is_none_or(|cursor| checkpoint_after_cursor(checkpoint, cursor))
            })
            .collect::<Vec<_>>();

        let mut observations = Vec::with_capacity(fresh_checkpoints.len());
        for checkpoint in fresh_checkpoints {
            let event = LiveCheckpointEvent::checkpoint_ready(
                self.next_emission_ordinal,
                checkpoint,
                Some(self.rollout_path.as_str().to_string()),
            );
            self.next_emission_ordinal += 1;
            let observation = self.runtime.observe(event)?;
            self.last_delivered_cursor = Some(observation.event.cursor.clone());
            observations.push(observation);
        }

        self.last_observed_size_bytes = Some(observed_size_bytes);

        Ok(LiveSessionPollResult {
            rollout_path: self.rollout_path.clone(),
            observed_size_bytes,
            reran_pipeline: true,
            emitted_checkpoints: observations.len(),
            latest_cursor: self.last_delivered_cursor.clone(),
            observations,
        })
    }

    fn run_pipeline(&self) -> Result<Vec<Checkpoint>, LiveSessionError> {
        fs::create_dir_all(&self.request.state_dir).map_err(|source| {
            LiveSessionError::InspectRollout {
                path: self.request.state_dir.clone(),
                source,
            }
        })?;

        compact_codex_sessions(&RunConfig {
            codex_home: self.request.codex_home.clone(),
            session_id: Some(self.request.session_id.clone()),
            output_dir: self.compactor_output_dir(),
            generated_at: None,
        })?;

        analyze_bundle(&AnalyzeRequest {
            input_dir: self.compactor_output_dir(),
            output_dir: self.analyzer_output_dir(),
        })?;

        let bundle = load_replay_bundle(&self.analyzer_output_dir())?;
        let mut found_session_ids = bundle
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.session_id.clone())
            .collect::<Vec<_>>();
        found_session_ids.sort();
        found_session_ids.dedup();
        if found_session_ids != [self.request.session_id.clone()] {
            return Err(LiveSessionError::UnexpectedCheckpointSessions {
                expected_session_id: self.request.session_id.clone(),
                found_session_ids,
            });
        }

        Ok(bundle.checkpoints)
    }

    fn compactor_output_dir(&self) -> Utf8PathBuf {
        self.request.state_dir.join("compactor")
    }

    fn analyzer_output_dir(&self) -> Utf8PathBuf {
        self.request.state_dir.join("analyzer")
    }
}

fn resolve_rollout_path(request: &LiveSessionRequest) -> Result<Utf8PathBuf, LiveSessionError> {
    let codex_home = agent_session_compactor::resolve_codex_home(request.codex_home.clone())?;
    let artifacts = discover_session_artifacts(&DiscoverOptions {
        codex_home: Some(codex_home.clone()),
        session_id: Some(request.session_id.clone()),
    })?;
    let rollout_paths = artifacts
        .into_iter()
        .map(|artifact| artifact.path)
        .filter(|path| is_rollout_artifact(path))
        .collect::<Vec<_>>();

    match rollout_paths.as_slice() {
        [] => Err(LiveSessionError::MissingRolloutArtifact {
            session_id: request.session_id.clone(),
            codex_home,
        }),
        [path] => Ok(path.clone()),
        _ => Err(LiveSessionError::AmbiguousRolloutArtifacts {
            session_id: request.session_id.clone(),
            codex_home,
            paths: rollout_paths,
        }),
    }
}

fn is_rollout_artifact(path: &Utf8Path) -> bool {
    matches!(
        path.file_name(),
        Some(file_name) if file_name.starts_with("rollout-") && file_name.ends_with(".jsonl")
    )
}

fn file_size_bytes(path: &Utf8Path) -> Result<u64, LiveSessionError> {
    let metadata = fs::metadata(path).map_err(|source| LiveSessionError::InspectRollout {
        path: path.to_owned(),
        source,
    })?;
    Ok(metadata.len())
}

fn checkpoint_after_cursor(checkpoint: &Checkpoint, cursor: &CheckpointCursor) -> bool {
    checkpoint.session_id > cursor.session_id
        || (checkpoint.session_id == cursor.session_id && checkpoint.ordinal > cursor.ordinal)
}

fn sparse_startup_retry_allowed(rollout_path: &Utf8Path, error: &LiveSessionError) -> bool {
    let Ok(readiness) = inspect_rollout_startup_readiness(rollout_path) else {
        return false;
    };

    match error {
        LiveSessionError::Analyzer(AnalyzerError::Input(AnalyzerInputError::NoSessions {
            ..
        })) => !readiness.has_session_activity,
        LiveSessionError::Analyzer(AnalyzerError::Input(
            AnalyzerInputError::InsufficientContract { reason },
        )) => sparse_startup_contract_gap(&readiness, reason),
        _ => false,
    }
}

fn sparse_startup_contract_gap(readiness: &RolloutStartupReadiness, reason: &str) -> bool {
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

fn inspect_rollout_startup_readiness(
    rollout_path: &Utf8Path,
) -> Result<RolloutStartupReadiness, LiveSessionError> {
    let file = fs::File::open(rollout_path).map_err(|source| LiveSessionError::InspectRollout {
        path: rollout_path.to_owned(),
        source,
    })?;
    let reader = BufReader::new(file);
    let mut readiness = RolloutStartupReadiness::default();

    for line in reader.lines() {
        let line = line.map_err(|source| LiveSessionError::InspectRollout {
            path: rollout_path.to_owned(),
            source,
        })?;
        if line.trim().is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        update_rollout_startup_readiness(&mut readiness, &value);
    }

    Ok(readiness)
}

fn update_rollout_startup_readiness(readiness: &mut RolloutStartupReadiness, value: &Value) {
    let event_type = value.get("type").and_then(Value::as_str);
    if event_type.is_some_and(|event_type| event_type != "session_meta") {
        readiness.has_session_activity = true;
    }

    for text in rollout_text_fragments(value) {
        if !text.trim().is_empty() {
            readiness.has_literal_directive_text = true;
            if rollout_text_has_path_hint(text) {
                readiness.has_path_hint = true;
            }
        }
    }

    if let Some(arguments) = rollout_tool_call_arguments(value) {
        if parse_tool_arguments(arguments).is_some() {
            readiness.has_parseable_tool_call_arguments = true;
        }
    }
}

fn rollout_text_fragments(value: &Value) -> Vec<&str> {
    let mut texts = Vec::new();
    let Some(payload) = value.get("payload") else {
        return texts;
    };

    if let Some(message) = payload.get("message").and_then(Value::as_str) {
        texts.push(message);
    }
    if let Some(user_instructions) = payload.get("user_instructions").and_then(Value::as_str) {
        texts.push(user_instructions);
    }
    if let Some(base_instruction_text) = payload
        .get("base_instructions")
        .and_then(|base| base.get("text"))
        .and_then(Value::as_str)
    {
        texts.push(base_instruction_text);
    }
    if payload.get("type").and_then(Value::as_str) == Some("message")
        && matches!(
            payload.get("role").and_then(Value::as_str),
            Some("user" | "developer" | "system")
        )
    {
        if let Some(content) = payload.get("content").and_then(Value::as_array) {
            for item in content {
                if let Some(text) = item.get("text").and_then(Value::as_str) {
                    texts.push(text);
                }
            }
        }
    }

    texts
}

fn rollout_tool_call_arguments(value: &Value) -> Option<&str> {
    let payload = value.get("payload")?;
    (payload.get("type").and_then(Value::as_str) == Some("function_call"))
        .then(|| payload.get("arguments").and_then(Value::as_str))
        .flatten()
}

fn parse_tool_arguments(text: &str) -> Option<Value> {
    serde_json::from_str::<Value>(text)
        .ok()
        .filter(Value::is_object)
}

fn rollout_text_has_path_hint(text: &str) -> bool {
    text.split_whitespace().any(|raw_token| {
        let token = raw_token
            .trim_matches(|ch: char| {
                matches!(
                    ch,
                    ',' | ':' | ';' | '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | '`'
                )
            })
            .trim_end_matches('.');
        if token.is_empty() || token.starts_with("http://") || token.starts_with("https://") {
            return false;
        }
        let has_separator = token.contains('/') || token.starts_with('.');
        let has_extension = [
            ".md", ".rs", ".toml", ".json", ".jsonl", ".yaml", ".yml", ".sh", ".txt",
        ]
        .iter()
        .any(|suffix| token.ends_with(suffix));
        has_separator || has_extension
    })
}

#[cfg(test)]
mod tests {
    use super::{
        inspect_rollout_startup_readiness, sparse_startup_retry_allowed, LiveSessionError,
        RolloutStartupReadiness,
    };
    use agent_drift_analyzer::{AnalyzerError, InputError as AnalyzerInputError};
    use camino::Utf8Path;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn sparse_startup_retry_allows_no_sessions_only_before_session_activity() {
        let temp_dir = TempDir::new().expect("temp dir");
        let rollout_path = Utf8Path::from_path(temp_dir.path())
            .expect("utf8 temp dir")
            .join("rollout.jsonl");
        fs::write(
            &rollout_path,
            "{\"timestamp\":\"2026-06-01T12:00:00Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"session-live\"}}\n",
        )
        .expect("write sparse rollout");

        let error =
            LiveSessionError::Analyzer(AnalyzerError::Input(AnalyzerInputError::NoSessions {
                input_dir: rollout_path.parent().expect("parent").join("bundle"),
            }));
        assert!(sparse_startup_retry_allowed(&rollout_path, &error));

        fs::write(
            &rollout_path,
            concat!(
                "{\"timestamp\":\"2026-06-01T12:00:00Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"session-live\"}}\n",
                "{\"timestamp\":\"2026-06-01T12:00:01Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_started\",\"turn_id\":\"turn-1\"}}\n"
            ),
        )
        .expect("write active rollout");
        assert!(!sparse_startup_retry_allowed(&rollout_path, &error));
    }

    #[test]
    fn sparse_startup_retry_rejects_non_sparse_contract_breakage() {
        let temp_dir = TempDir::new().expect("temp dir");
        let rollout_path = Utf8Path::from_path(temp_dir.path())
            .expect("utf8 temp dir")
            .join("rollout.jsonl");
        fs::write(
            &rollout_path,
            concat!(
                "{\"timestamp\":\"2026-06-01T12:00:00Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"session-live\",\"base_instructions\":{\"text\":\"Base instructions\"}}}\n",
                "{\"timestamp\":\"2026-06-01T12:00:01Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_started\",\"turn_id\":\"turn-1\",\"message\":\"docs/specs/agent-drift-sentinel-real-session-live-v0.5-spec.md\"}}\n",
                "{\"timestamp\":\"2026-06-01T12:00:02Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"inspect crates/agent-drift-sentinel/src/real_session_live.rs\"}]}}\n",
                "{\"timestamp\":\"2026-06-01T12:00:03Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call\",\"name\":\"functions.shell_command\",\"arguments\":\"{\\\"command\\\":\\\"cargo test -p agent-drift-sentinel\\\",\\\"workdir\\\":\\\"/repo\\\"}\"}}\n"
            ),
        )
        .expect("write rollout");

        let stable_rows_error = LiveSessionError::Analyzer(AnalyzerError::Input(
            AnalyzerInputError::InsufficientContract {
                reason: "row references are not unique and stable".to_string(),
            },
        ));
        assert!(!sparse_startup_retry_allowed(
            &rollout_path,
            &stable_rows_error
        ));

        let missing_tool_error = LiveSessionError::Analyzer(AnalyzerError::Input(
            AnalyzerInputError::InsufficientContract {
                reason: "tool-call argument payloads are not parseable enough to infer command families and working-set paths".to_string(),
            },
        ));
        assert!(!sparse_startup_retry_allowed(
            &rollout_path,
            &missing_tool_error
        ));
    }

    #[test]
    fn inspect_rollout_startup_readiness_tracks_sparse_requirements() {
        let temp_dir = TempDir::new().expect("temp dir");
        let rollout_path = Utf8Path::from_path(temp_dir.path())
            .expect("utf8 temp dir")
            .join("rollout.jsonl");
        fs::write(
            &rollout_path,
            concat!(
                "{\"timestamp\":\"2026-06-01T12:00:00Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"session-live\",\"base_instructions\":{\"text\":\"Base instructions\"}}}\n",
                "{\"timestamp\":\"2026-06-01T12:00:01Z\",\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-1\",\"user_instructions\":\"keep crates/agent-drift-sentinel/src/real_session_live.rs in scope\"}}\n",
                "{\"timestamp\":\"2026-06-01T12:00:02Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call\",\"name\":\"functions.shell_command\",\"arguments\":\"{\\\"command\\\":\\\"cargo test -p agent-drift-sentinel\\\",\\\"workdir\\\":\\\"/repo\\\"}\"}}\n"
            ),
        )
        .expect("write rollout");

        let readiness = inspect_rollout_startup_readiness(&rollout_path).expect("readiness");
        assert_eq!(
            readiness,
            RolloutStartupReadiness {
                has_session_activity: true,
                has_literal_directive_text: true,
                has_path_hint: true,
                has_parseable_tool_call_arguments: true,
            }
        );
    }
}
