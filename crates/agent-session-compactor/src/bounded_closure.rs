use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::time::UNIX_EPOCH;

use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;
use serde_json::Value;
use time::OffsetDateTime;

use crate::discovery::{select_direct_linked_closure, DirectLinkedClosure, LinkedClosureArtifact};
use crate::ingest::{
    extract_rollout_linkage_metadata, ingest_rollout_file, ChildSessionOrigin, IngestedRolloutFile,
    ParentSpawnResult, RolloutFormat, RolloutLinkageMetadata, RolloutRowProvenance,
};
use crate::{
    compact_ingested_rollouts, discover_session_artifacts, resolve_codex_home, CompactionRunResult,
    CompactorError, DiscoverOptions, DiscoveryError,
};

/// Request for one compactor-owned linked closure over raw Codex traces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedClosureRequest {
    pub codex_home: Option<Utf8PathBuf>,
    pub root_session_id: String,
}

/// Reconstructible identity and append-generation evidence for one raw trace source.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawTraceSourceState {
    pub source_file: Utf8PathBuf,
    pub generation: u128,
    pub high_water_mark: u64,
}

/// Minimal validated source information exposed by the bounded closure contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedClosureSource {
    pub source_file: Utf8PathBuf,
    pub generation: u128,
    pub high_water_mark: u64,
    pub format: RolloutFormat,
    pub session_id: Option<String>,
    pub event_count: usize,
    pub turn_ids: Vec<String>,
}

/// Root-only facts needed by the Sentinel sparse-startup policy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BoundedClosureStartupReadiness {
    pub has_session_activity: bool,
    pub has_literal_directive_text: bool,
    pub has_path_hint: bool,
    pub has_parseable_tool_call_arguments: bool,
}

/// Deterministic normalized description of a selected linked closure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedClosureSnapshot {
    pub codex_home: Utf8PathBuf,
    pub root_session_id: String,
    pub root_source: BoundedClosureSource,
    pub selected_sources: Vec<BoundedClosureSource>,
    pub linkage_source_files: Vec<Utf8PathBuf>,
    pub startup_readiness: BoundedClosureStartupReadiness,
    pub root_identity_validated: bool,
    pub revision: String,
}

/// Observable cache behavior for focused contract tests and live-poller diagnostics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BoundedClosureCacheStats {
    pub discovered_source_count: usize,
    pub reused_source_index_count: usize,
    pub rebuilt_source_index_count: usize,
    pub closure_selection_reused: bool,
}

/// Prepared closure whose raw sources have not yet been fully decoded.
#[derive(Debug, Clone)]
pub struct PreparedBoundedClosure {
    snapshot: BoundedClosureSnapshot,
    cache_stats: BoundedClosureCacheStats,
    state_token: String,
    all_source_states: Vec<RawTraceSourceState>,
    selected_indexes: Vec<SourceIndex>,
    linkage_indexes: Vec<SourceIndex>,
}

impl PreparedBoundedClosure {
    pub fn snapshot(&self) -> &BoundedClosureSnapshot {
        &self.snapshot
    }

    pub fn cache_stats(&self) -> BoundedClosureCacheStats {
        self.cache_stats
    }

    /// Reconstructible token for the requested root and complete discovered-source state.
    pub fn state_token(&self) -> &str {
        &self.state_token
    }
}

/// Result of fully decoding and compacting only the prepared selected sources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedClosureCompactionResult {
    pub run_result: CompactionRunResult,
    pub snapshot: BoundedClosureSnapshot,
    pub decoded_source_files: Vec<Utf8PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum BoundedClosureError {
    #[error(transparent)]
    Discovery(#[from] DiscoveryError),
    #[error("no rollout JSONL files were found under {codex_home}")]
    NoRolloutFiles { codex_home: Utf8PathBuf },
    #[error("failed to inspect raw trace source {path}: {source}")]
    Inspect {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("raw trace source {path} changed while preparing or decoding its closure")]
    SourceChangedDuringClosure { path: Utf8PathBuf },
    #[error(
        "complete raw trace source state changed while preparing or decoding a closure under {codex_home}"
    )]
    DiscoveredSourceStateChanged { codex_home: Utf8PathBuf },
    #[error("raw trace envelope in {path} at line {line_number} is indeterminate: {reason}")]
    IndeterminateEnvelope {
        path: Utf8PathBuf,
        line_number: usize,
        reason: String,
    },
    #[error("raw trace source {path} is non-empty but has no unambiguous session identity")]
    IndeterminateSourceIdentity { path: Utf8PathBuf },
    #[error("selected raw trace source {path} contains malformed payloads: {failures:?}")]
    SelectedPayloadMalformed {
        path: Utf8PathBuf,
        failures: Vec<String>,
    },
    #[error(
        "selected raw trace source {path} no longer matches its validated closure index: {reason}"
    )]
    ClosureIndexMismatch { path: Utf8PathBuf, reason: String },
}

#[derive(Debug, Clone, Default)]
pub struct BoundedClosureCompactor {
    source_indexes: BTreeMap<Utf8PathBuf, CachedSourceIndex>,
    closure_cache: Option<(ClosureCacheKey, CachedClosureSelection)>,
}

impl BoundedClosureCompactor {
    /// Discovers and minimally indexes every rollout source, then selects the requested closure.
    ///
    /// Full event decoding is deliberately deferred to [`Self::compact`].
    pub fn prepare(
        &mut self,
        request: &BoundedClosureRequest,
    ) -> Result<PreparedBoundedClosure, BoundedClosureError> {
        let root_session_id = normalize_session_id(&request.root_session_id)?;
        let codex_home = resolve_codex_home(request.codex_home.clone())?;
        let states = match discover_source_states(&codex_home) {
            Ok(states) if !states.is_empty() => states,
            Ok(_) => {
                self.invalidate_all();
                return Err(BoundedClosureError::NoRolloutFiles { codex_home });
            }
            Err(error) => {
                self.invalidate_all();
                return Err(error);
            }
        };

        let closure_key = ClosureCacheKey {
            root_session_id: root_session_id.clone(),
            source_states: states.clone(),
        };
        if self
            .closure_cache
            .as_ref()
            .is_some_and(|(key, _)| key != &closure_key)
        {
            self.closure_cache = None;
        }

        let live_paths = states
            .iter()
            .map(|state| state.source_file.clone())
            .collect::<BTreeSet<_>>();
        self.source_indexes
            .retain(|path, _| live_paths.contains(path));

        let mut cache_stats = BoundedClosureCacheStats {
            discovered_source_count: states.len(),
            ..BoundedClosureCacheStats::default()
        };
        for state in &states {
            let reused = self
                .source_indexes
                .get(&state.source_file)
                .is_some_and(|cached| cached.index.state == *state);
            if reused {
                cache_stats.reused_source_index_count += 1;
                continue;
            }

            self.source_indexes.remove(&state.source_file);
            self.closure_cache = None;
            let index = index_source(state)?;
            self.source_indexes.insert(
                state.source_file.clone(),
                CachedSourceIndex {
                    index,
                    startup_readiness: None,
                },
            );
            cache_stats.rebuilt_source_index_count += 1;
        }

        let indexes = states
            .iter()
            .map(|state| {
                self.source_indexes
                    .get(&state.source_file)
                    .map(|cached| cached.index.clone())
                    .ok_or_else(|| BoundedClosureError::ClosureIndexMismatch {
                        path: state.source_file.clone(),
                        reason: "source index was not retained in the compactor cache".to_string(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        for index in &indexes {
            if index.state.high_water_mark > 0 && index.session_id.is_none() {
                return Err(BoundedClosureError::IndeterminateSourceIdentity {
                    path: index.state.source_file.clone(),
                });
            }
        }

        let selection = if let Some((_, selection)) = &self.closure_cache {
            cache_stats.closure_selection_reused = true;
            selection.clone()
        } else {
            let selection = select_closure(&root_session_id, &indexes)?;
            self.closure_cache = Some((closure_key, selection.clone()));
            selection
        };

        let root_state = states
            .iter()
            .find(|state| state.source_file == selection.root_source_file)
            .cloned()
            .ok_or_else(|| BoundedClosureError::ClosureIndexMismatch {
                path: selection.root_source_file.clone(),
                reason: "selected root source is absent from the discovered source state"
                    .to_string(),
            })?;
        let cached_readiness = self
            .source_indexes
            .get(&selection.root_source_file)
            .ok_or_else(|| BoundedClosureError::ClosureIndexMismatch {
                path: selection.root_source_file.clone(),
                reason: "selected root source has no source index".to_string(),
            })?
            .startup_readiness
            .clone();
        let startup_readiness = if let Some(readiness) = cached_readiness {
            readiness
        } else {
            let readiness = match inspect_startup_readiness(&root_state) {
                Ok(readiness) => readiness,
                Err(error) => {
                    self.invalidate_source(&root_state.source_file);
                    return Err(error);
                }
            };
            let cached = self
                .source_indexes
                .get_mut(&selection.root_source_file)
                .ok_or_else(|| BoundedClosureError::ClosureIndexMismatch {
                    path: selection.root_source_file.clone(),
                    reason: "selected root source cache disappeared".to_string(),
                })?;
            cached.startup_readiness = Some(readiness.clone());
            readiness
        };

        let selected_indexes = indexes
            .iter()
            .filter(|index| {
                selection
                    .closure
                    .included_source_files
                    .contains(&index.state.source_file)
            })
            .cloned()
            .collect::<Vec<_>>();
        let linkage_indexes = indexes
            .iter()
            .filter(|index| {
                selection
                    .closure
                    .linkage_source_files
                    .contains(&index.state.source_file)
            })
            .cloned()
            .collect::<Vec<_>>();
        let root_index = selected_indexes
            .iter()
            .find(|index| index.state.source_file == selection.root_source_file)
            .ok_or_else(|| BoundedClosureError::ClosureIndexMismatch {
                path: selection.root_source_file.clone(),
                reason: "selected closure did not retain its root source".to_string(),
            })?;
        let selected_sources = selected_indexes
            .iter()
            .map(SourceIndex::public_source)
            .collect::<Vec<_>>();
        let linkage_source_files = linkage_indexes
            .iter()
            .map(|index| index.state.source_file.clone())
            .collect::<Vec<_>>();
        let revision = closure_revision(
            &root_session_id,
            selection.root_identity_validated,
            &selected_indexes,
            &linkage_indexes,
        );
        let state_token = complete_source_state_token(&root_session_id, &states);
        let snapshot = BoundedClosureSnapshot {
            codex_home,
            root_session_id,
            root_source: root_index.public_source(),
            selected_sources,
            linkage_source_files,
            startup_readiness,
            root_identity_validated: selection.root_identity_validated,
            revision,
        };

        self.verify_complete_source_state_or_invalidate(&snapshot.codex_home, &states)?;

        Ok(PreparedBoundedClosure {
            snapshot,
            cache_stats,
            state_token,
            all_source_states: states,
            selected_indexes,
            linkage_indexes,
        })
    }

    fn invalidate_all(&mut self) {
        self.source_indexes.clear();
        self.closure_cache = None;
    }

    fn invalidate_source(&mut self, path: &Utf8Path) {
        self.source_indexes.remove(path);
        self.closure_cache = None;
    }

    fn verify_complete_source_state_or_invalidate(
        &mut self,
        codex_home: &Utf8Path,
        expected: &[RawTraceSourceState],
    ) -> Result<(), BoundedClosureError> {
        let actual = match discover_source_states(codex_home) {
            Ok(actual) => actual,
            Err(error) => {
                self.invalidate_all();
                return Err(error);
            }
        };
        if actual.as_slice() != expected {
            self.invalidate_all();
            return Err(BoundedClosureError::DiscoveredSourceStateChanged {
                codex_home: codex_home.to_owned(),
            });
        }
        Ok(())
    }

    fn verify_source_state_or_invalidate(
        &mut self,
        state: &RawTraceSourceState,
    ) -> Result<(), BoundedClosureError> {
        if let Err(error) = verify_source_state(state) {
            self.invalidate_source(&state.source_file);
            return Err(error);
        }
        Ok(())
    }

    /// Fully decodes only the sources already selected into a prepared closure.
    pub fn compact(
        &mut self,
        prepared: PreparedBoundedClosure,
        output_dir: &Utf8Path,
        generated_at: Option<OffsetDateTime>,
    ) -> Result<BoundedClosureCompactionResult, CompactorError> {
        self.verify_complete_source_state_or_invalidate(
            &prepared.snapshot.codex_home,
            &prepared.all_source_states,
        )
        .map_err(CompactorError::from)?;

        let mut decoded_rollouts = Vec::with_capacity(prepared.selected_indexes.len());
        let mut decoded_source_files = Vec::with_capacity(prepared.selected_indexes.len());
        for index in &prepared.selected_indexes {
            let rollout = match ingest_rollout_file(&index.state.source_file) {
                Ok(rollout) => rollout,
                Err(error) => {
                    self.invalidate_source(&index.state.source_file);
                    return Err(CompactorError::from(
                        BoundedClosureError::SelectedPayloadMalformed {
                            path: index.state.source_file.clone(),
                            failures: vec![error.to_string()],
                        },
                    ));
                }
            };
            if !rollout.parse_failures.is_empty() {
                self.invalidate_source(&index.state.source_file);
                return Err(CompactorError::from(
                    BoundedClosureError::SelectedPayloadMalformed {
                        path: index.state.source_file.clone(),
                        failures: rollout
                            .parse_failures
                            .iter()
                            .map(|failure| {
                                format!("line {}: {}", failure.line_number, failure.error)
                            })
                            .collect(),
                    },
                ));
            }
            if let Err(error) = verify_decoded_rollout(index, &rollout) {
                self.invalidate_source(&index.state.source_file);
                return Err(CompactorError::from(error));
            }
            self.verify_source_state_or_invalidate(&index.state)
                .map_err(CompactorError::from)?;
            decoded_source_files.push(index.state.source_file.clone());
            decoded_rollouts.push(rollout);
        }
        self.verify_complete_source_state_or_invalidate(
            &prepared.snapshot.codex_home,
            &prepared.all_source_states,
        )
        .map_err(CompactorError::from)?;

        let linkage_metadata = prepared
            .linkage_indexes
            .iter()
            .map(|index| index.linkage_metadata.clone())
            .collect::<Vec<_>>();
        let run_result = compact_ingested_rollouts(
            &prepared.snapshot.codex_home,
            output_dir,
            generated_at,
            decoded_rollouts,
            linkage_metadata,
        )?;
        self.verify_complete_source_state_or_invalidate(
            &prepared.snapshot.codex_home,
            &prepared.all_source_states,
        )
        .map_err(CompactorError::from)?;

        Ok(BoundedClosureCompactionResult {
            run_result,
            snapshot: prepared.snapshot,
            decoded_source_files,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceIndex {
    state: RawTraceSourceState,
    format: RolloutFormat,
    session_id: Option<String>,
    event_count: usize,
    turn_ids: Vec<String>,
    linkage_metadata: RolloutLinkageMetadata,
}

impl SourceIndex {
    fn public_source(&self) -> BoundedClosureSource {
        BoundedClosureSource {
            source_file: self.state.source_file.clone(),
            generation: self.state.generation,
            high_water_mark: self.state.high_water_mark,
            format: self.format,
            session_id: self.session_id.clone(),
            event_count: self.event_count,
            turn_ids: self.turn_ids.clone(),
        }
    }

    fn linked_artifact(&self) -> LinkedClosureArtifact {
        LinkedClosureArtifact {
            source_file: self.state.source_file.clone(),
            session_id: self.session_id.clone(),
            linkage_metadata: self.linkage_metadata.clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct CachedSourceIndex {
    index: SourceIndex,
    startup_readiness: Option<BoundedClosureStartupReadiness>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClosureCacheKey {
    root_session_id: String,
    source_states: Vec<RawTraceSourceState>,
}

#[derive(Debug, Clone)]
struct CachedClosureSelection {
    closure: DirectLinkedClosure,
    root_source_file: Utf8PathBuf,
    root_identity_validated: bool,
}

#[derive(Debug, Deserialize)]
struct RawRecordEnvelope {
    #[serde(rename = "type")]
    record_type: String,
    payload: RawPayloadEnvelope,
}

#[derive(Debug, Default, Deserialize)]
struct RawPayloadEnvelope {
    #[serde(rename = "type")]
    item_type: Option<Value>,
    id: Option<Value>,
    multi_agent_version: Option<Value>,
    source: Option<Value>,
    parent_thread_id: Option<Value>,
    turn_id: Option<Value>,
    internal_chat_message_metadata_passthrough: Option<Value>,
    name: Option<Value>,
    call_id: Option<Value>,
    output: Option<Value>,
    sender_thread_id: Option<Value>,
    new_thread_id: Option<Value>,
    prompt: Option<Value>,
}

#[derive(Debug)]
struct IndexedEnvelopeRecord {
    line_number: usize,
    event_index: usize,
    record_type: String,
    payload: RawPayloadEnvelope,
}

fn normalize_session_id(value: &str) -> Result<String, DiscoveryError> {
    let value = value.trim();
    let value = value.strip_prefix("urn:uuid:").unwrap_or(value);
    if value.is_empty() {
        return Err(DiscoveryError::LinkedChildrenRequireSessionId);
    }
    Ok(value.to_string())
}

fn is_rollout_jsonl(path: &Utf8Path) -> bool {
    matches!(
        path.file_name(),
        Some(file_name) if file_name.starts_with("rollout-") && file_name.ends_with(".jsonl")
    )
}

fn discover_source_states(
    codex_home: &Utf8Path,
) -> Result<Vec<RawTraceSourceState>, BoundedClosureError> {
    let artifacts = discover_session_artifacts(&DiscoverOptions {
        codex_home: Some(codex_home.to_owned()),
        session_id: None,
    })?;
    let mut states = Vec::new();
    for path in artifacts
        .into_iter()
        .map(|artifact| artifact.path)
        .filter(|path| is_rollout_jsonl(path))
    {
        states.push(inspect_source_state(&path)?);
    }
    states.sort();
    Ok(states)
}

fn inspect_source_state(path: &Utf8Path) -> Result<RawTraceSourceState, BoundedClosureError> {
    let metadata = fs::metadata(path).map_err(|source| BoundedClosureError::Inspect {
        path: path.to_owned(),
        source,
    })?;
    let modified = metadata
        .modified()
        .map_err(|source| BoundedClosureError::Inspect {
            path: path.to_owned(),
            source,
        })?;
    let generation = modified
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_else(|error| u128::MAX - error.duration().as_nanos());
    Ok(RawTraceSourceState {
        source_file: path.to_owned(),
        generation,
        high_water_mark: metadata.len(),
    })
}

fn verify_source_state(state: &RawTraceSourceState) -> Result<(), BoundedClosureError> {
    let actual = inspect_source_state(&state.source_file)?;
    if actual != *state {
        return Err(BoundedClosureError::SourceChangedDuringClosure {
            path: state.source_file.clone(),
        });
    }
    Ok(())
}

fn index_source(state: &RawTraceSourceState) -> Result<SourceIndex, BoundedClosureError> {
    let file =
        fs::File::open(&state.source_file).map_err(|source| BoundedClosureError::Inspect {
            path: state.source_file.clone(),
            source,
        })?;
    let mut records = Vec::new();
    for (line_index, line) in BufReader::new(file).lines().enumerate() {
        let line_number = line_index + 1;
        let line = line.map_err(|source| BoundedClosureError::Inspect {
            path: state.source_file.clone(),
            source,
        })?;
        if line.trim().is_empty() {
            continue;
        }
        let envelope = serde_json::from_str::<RawRecordEnvelope>(&line).map_err(|error| {
            BoundedClosureError::IndeterminateEnvelope {
                path: state.source_file.clone(),
                line_number,
                reason: error.to_string(),
            }
        })?;
        if envelope.record_type.trim().is_empty() {
            return Err(BoundedClosureError::IndeterminateEnvelope {
                path: state.source_file.clone(),
                line_number,
                reason: "record.type must not be empty".to_string(),
            });
        }
        records.push(IndexedEnvelopeRecord {
            line_number,
            event_index: records.len(),
            record_type: envelope.record_type,
            payload: envelope.payload,
        });
    }
    verify_source_state(state)?;

    let format = select_minimal_format(&state.source_file, &records)?;
    let session_id = index_session_identity(&state.source_file, &records)?;
    let turn_ids = index_turn_ids(&records);
    let linkage_metadata =
        index_linkage(&state.source_file, format, session_id.as_deref(), &records)?;
    Ok(SourceIndex {
        state: state.clone(),
        format,
        session_id,
        event_count: records.len(),
        turn_ids,
        linkage_metadata,
    })
}

fn select_minimal_format(
    path: &Utf8Path,
    records: &[IndexedEnvelopeRecord],
) -> Result<RolloutFormat, BoundedClosureError> {
    let mut selected = None;
    for record in records
        .iter()
        .filter(|record| record.record_type == "session_meta")
    {
        let candidate = match record.payload.multi_agent_version.as_ref() {
            None => RolloutFormat::Legacy,
            Some(Value::String(version)) if version == "v2" => RolloutFormat::CurrentNativeV2,
            Some(Value::String(version)) if matches!(version.as_str(), "disabled" | "v1") => {
                RolloutFormat::Legacy
            }
            Some(Value::String(version)) => {
                return Err(envelope_error(
                    path,
                    record,
                    format!("unsupported session_meta.multi_agent_version {version:?}"),
                ))
            }
            Some(_) => {
                return Err(envelope_error(
                    path,
                    record,
                    "session_meta.multi_agent_version must be a string".to_string(),
                ))
            }
        };
        if let Some(existing) = selected {
            if existing != candidate {
                return Err(envelope_error(
                    path,
                    record,
                    format!("conflicting rollout format markers: {existing:?} then {candidate:?}"),
                ));
            }
        } else {
            selected = Some(candidate);
        }
    }
    Ok(selected.unwrap_or(RolloutFormat::Legacy))
}

fn index_session_identity(
    path: &Utf8Path,
    records: &[IndexedEnvelopeRecord],
) -> Result<Option<String>, BoundedClosureError> {
    let mut session_id = None;
    for record in records
        .iter()
        .filter(|record| record.record_type == "session_meta")
    {
        let candidate = required_non_empty_string(record.payload.id.as_ref(), "session_meta.id")
            .map_err(|reason| envelope_error(path, record, reason))?;
        match session_id.as_deref() {
            Some(existing) if existing != candidate => {
                return Err(envelope_error(
                    path,
                    record,
                    format!(
                    "conflicting session identities {existing:?} and {candidate:?} in one source"
                ),
                ))
            }
            Some(_) => {}
            None => session_id = Some(candidate.to_string()),
        }
    }
    Ok(session_id)
}

fn index_turn_ids(records: &[IndexedEnvelopeRecord]) -> Vec<String> {
    let mut turn_ids = BTreeSet::new();
    for record in records {
        if let Some(turn_id) = optional_non_empty_string_lossy(record.payload.turn_id.as_ref()) {
            turn_ids.insert(turn_id.to_string());
        }
        if record.record_type == "response_item" {
            let metadata_turn = record
                .payload
                .internal_chat_message_metadata_passthrough
                .as_ref()
                .and_then(Value::as_object)
                .and_then(|metadata| metadata.get("turn_id"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|turn_id| !turn_id.is_empty());
            if let Some(turn_id) = metadata_turn {
                turn_ids.insert(turn_id.to_string());
            }
        }
    }
    turn_ids.into_iter().collect()
}

fn index_linkage(
    path: &Utf8Path,
    format: RolloutFormat,
    session_id: Option<&str>,
    records: &[IndexedEnvelopeRecord],
) -> Result<RolloutLinkageMetadata, BoundedClosureError> {
    let parent_spawn_results = match format {
        RolloutFormat::Legacy => index_legacy_parent_spawns(path, session_id, records)?,
        RolloutFormat::CurrentNativeV2 => index_current_parent_spawns(path, session_id, records)?,
    };
    let mut child_origin = None;
    if let Some(session_id) = session_id {
        for record in records
            .iter()
            .filter(|record| record.record_type == "session_meta")
        {
            let candidate = index_child_origin(path, format, session_id, record)?;
            if candidate.is_some() {
                child_origin = candidate;
                break;
            }
        }
    }
    Ok(RolloutLinkageMetadata {
        parent_spawn_results,
        child_origin,
    })
}

fn index_legacy_parent_spawns(
    path: &Utf8Path,
    session_id: Option<&str>,
    records: &[IndexedEnvelopeRecord],
) -> Result<Vec<ParentSpawnResult>, BoundedClosureError> {
    let Some(parent_session_id) = session_id else {
        return Ok(Vec::new());
    };
    let mut calls = BTreeMap::<String, Vec<&IndexedEnvelopeRecord>>::new();
    for record in records
        .iter()
        .filter(|record| record.record_type == "response_item")
    {
        let item_type = required_non_empty_string(
            record.payload.item_type.as_ref(),
            "response_item.payload.type",
        )
        .map_err(|reason| envelope_error(path, record, reason))?;
        if item_type != "function_call" {
            continue;
        }
        let name = required_non_empty_string(record.payload.name.as_ref(), "function_call.name")
            .map_err(|reason| envelope_error(path, record, reason))?;
        if name != "spawn_agent" {
            continue;
        }
        let call_id =
            required_non_empty_string(record.payload.call_id.as_ref(), "function_call.call_id")
                .map_err(|reason| envelope_error(path, record, reason))?;
        calls.entry(call_id.to_string()).or_default().push(record);
    }

    let mut results = Vec::new();
    for record in records
        .iter()
        .filter(|record| record.record_type == "response_item")
    {
        let item_type = required_non_empty_string(
            record.payload.item_type.as_ref(),
            "response_item.payload.type",
        )
        .map_err(|reason| envelope_error(path, record, reason))?;
        if item_type != "function_call_output" {
            continue;
        }
        let call_id = required_non_empty_string(
            record.payload.call_id.as_ref(),
            "function_call_output.call_id",
        )
        .map_err(|reason| envelope_error(path, record, reason))?;
        let Some(spawn_calls) = calls.get(call_id) else {
            continue;
        };
        let output = required_string(
            record.payload.output.as_ref(),
            "function_call_output.output",
            true,
        )
        .map_err(|reason| envelope_error(path, record, reason))?;
        let Some(child_session_id) = extract_spawn_result_child_id(output) else {
            continue;
        };
        for spawn_call in spawn_calls {
            results.push(ParentSpawnResult {
                parent_session_id: parent_session_id.to_string(),
                child_session_id: child_session_id.clone(),
                call_id: call_id.to_string(),
                spawn_call_provenance: provenance(path, spawn_call),
                spawn_result_provenance: provenance(path, record),
            });
        }
    }
    Ok(results)
}

fn index_current_parent_spawns(
    path: &Utf8Path,
    session_id: Option<&str>,
    records: &[IndexedEnvelopeRecord],
) -> Result<Vec<ParentSpawnResult>, BoundedClosureError> {
    let Some(parent_session_id) = session_id else {
        return Ok(Vec::new());
    };
    let mut calls = BTreeMap::<String, Vec<&IndexedEnvelopeRecord>>::new();
    for record in records
        .iter()
        .filter(|record| record.record_type == "event_msg")
    {
        let item_type =
            required_non_empty_string(record.payload.item_type.as_ref(), "event_msg.payload.type")
                .map_err(|reason| envelope_error(path, record, reason))?;
        if item_type != "collab_agent_spawn_begin" {
            continue;
        }
        let call_id = required_non_empty_string(
            record.payload.call_id.as_ref(),
            "collab_agent_spawn_begin.call_id",
        )
        .map_err(|reason| envelope_error(path, record, reason))?;
        let sender = required_non_empty_string(
            record.payload.sender_thread_id.as_ref(),
            "collab_agent_spawn_begin.sender_thread_id",
        )
        .map_err(|reason| envelope_error(path, record, reason))?;
        required_string(
            record.payload.prompt.as_ref(),
            "collab_agent_spawn_begin.prompt",
            true,
        )
        .map_err(|reason| envelope_error(path, record, reason))?;
        if sender == parent_session_id {
            calls.entry(call_id.to_string()).or_default().push(record);
        }
    }

    let mut results = Vec::new();
    for record in records
        .iter()
        .filter(|record| record.record_type == "event_msg")
    {
        let item_type =
            required_non_empty_string(record.payload.item_type.as_ref(), "event_msg.payload.type")
                .map_err(|reason| envelope_error(path, record, reason))?;
        if item_type != "collab_agent_spawn_end" {
            continue;
        }
        let call_id = required_non_empty_string(
            record.payload.call_id.as_ref(),
            "collab_agent_spawn_end.call_id",
        )
        .map_err(|reason| envelope_error(path, record, reason))?;
        let sender = required_non_empty_string(
            record.payload.sender_thread_id.as_ref(),
            "collab_agent_spawn_end.sender_thread_id",
        )
        .map_err(|reason| envelope_error(path, record, reason))?;
        if sender != parent_session_id {
            continue;
        }
        let Some(child_session_id) = optional_non_empty_string(
            record.payload.new_thread_id.as_ref(),
            "collab_agent_spawn_end.new_thread_id",
        )
        .map_err(|reason| envelope_error(path, record, reason))?
        else {
            continue;
        };
        let Some(spawn_call) = calls.get(call_id).and_then(|candidates| {
            candidates
                .iter()
                .rev()
                .copied()
                .find(|candidate| candidate.event_index < record.event_index)
        }) else {
            continue;
        };
        results.push(ParentSpawnResult {
            parent_session_id: parent_session_id.to_string(),
            child_session_id: child_session_id.to_string(),
            call_id: call_id.to_string(),
            spawn_call_provenance: provenance(path, spawn_call),
            spawn_result_provenance: provenance(path, record),
        });
    }
    Ok(results)
}

fn index_child_origin(
    path: &Utf8Path,
    format: RolloutFormat,
    child_session_id: &str,
    record: &IndexedEnvelopeRecord,
) -> Result<Option<ChildSessionOrigin>, BoundedClosureError> {
    let Some(source) = record.payload.source.as_ref() else {
        return Ok(None);
    };
    if source.is_null() {
        return Ok(None);
    }
    if matches!(format, RolloutFormat::CurrentNativeV2) && source.is_string() {
        return Ok(None);
    }
    let source = if let Some(encoded) = source.as_str() {
        match serde_json::from_str::<Value>(encoded) {
            Ok(source) => source,
            Err(error) if matches!(encoded.trim_start().chars().next(), Some('{' | '[')) => {
                return Err(envelope_error(
                    path,
                    record,
                    format!("session_meta.source has ambiguous encoded linkage: {error}"),
                ))
            }
            Err(_) => return Ok(None),
        }
    } else {
        source.clone()
    };
    let Some(source_object) = source.as_object() else {
        if matches!(format, RolloutFormat::CurrentNativeV2) {
            return Err(envelope_error(
                path,
                record,
                "current-native session_meta.source must be a string, null, or object".to_string(),
            ));
        }
        return Ok(None);
    };
    let subagent = match format {
        RolloutFormat::Legacy => source_object.get("subagent"),
        RolloutFormat::CurrentNativeV2 => source_object
            .get("subagent")
            .or_else(|| source_object.get("sub_agent")),
    };
    let Some(subagent) = subagent else {
        return Ok(None);
    };
    if subagent.is_null() || subagent.is_string() {
        return Ok(None);
    }
    let subagent = subagent.as_object().ok_or_else(|| {
        envelope_error(
            path,
            record,
            "session_meta.source.subagent must be a string, null, or object".to_string(),
        )
    })?;
    let Some(thread_spawn) = subagent.get("thread_spawn") else {
        return Ok(None);
    };
    let thread_spawn = thread_spawn.as_object().ok_or_else(|| {
        envelope_error(
            path,
            record,
            "session_meta.source.subagent.thread_spawn must be an object".to_string(),
        )
    })?;
    let parent_session_id = required_non_empty_string(
        thread_spawn.get("parent_thread_id"),
        "thread_spawn.parent_thread_id",
    )
    .map_err(|reason| envelope_error(path, record, reason))?;
    if matches!(format, RolloutFormat::CurrentNativeV2) {
        if let Some(top_level_parent) = optional_non_empty_string(
            record.payload.parent_thread_id.as_ref(),
            "session_meta.parent_thread_id",
        )
        .map_err(|reason| envelope_error(path, record, reason))?
        {
            if top_level_parent != parent_session_id {
                return Err(envelope_error(
                    path,
                    record,
                    "session_meta parent_thread_id conflicts with child-origin metadata"
                        .to_string(),
                ));
            }
        }
    }
    let depth = thread_spawn
        .get("depth")
        .and_then(Value::as_u64)
        .and_then(|depth| u32::try_from(depth).ok())
        .ok_or_else(|| {
            envelope_error(path, record, "thread_spawn.depth must be a u32".to_string())
        })?;
    let agent_nickname = match format {
        RolloutFormat::Legacy => optional_string_preserve_empty(
            thread_spawn.get("agent_nickname"),
            "thread_spawn.agent_nickname",
        ),
        RolloutFormat::CurrentNativeV2 => optional_non_empty_string(
            thread_spawn.get("agent_nickname"),
            "thread_spawn.agent_nickname",
        ),
    }
    .map_err(|reason| envelope_error(path, record, reason))?
    .map(str::to_string);
    let agent_role = match format {
        RolloutFormat::Legacy => optional_string_preserve_empty(
            thread_spawn.get("agent_role"),
            "thread_spawn.agent_role",
        ),
        RolloutFormat::CurrentNativeV2 => {
            optional_non_empty_string(thread_spawn.get("agent_role"), "thread_spawn.agent_role")
        }
    }
    .map_err(|reason| envelope_error(path, record, reason))?
    .map(str::to_string);

    Ok(Some(ChildSessionOrigin {
        child_session_id: child_session_id.to_string(),
        parent_session_id: parent_session_id.to_string(),
        depth,
        agent_nickname,
        agent_role,
        provenance: provenance(path, record),
    }))
}

fn select_closure(
    root_session_id: &str,
    indexes: &[SourceIndex],
) -> Result<CachedClosureSelection, BoundedClosureError> {
    let artifacts = indexes
        .iter()
        .map(SourceIndex::linked_artifact)
        .collect::<Vec<_>>();
    match select_direct_linked_closure(root_session_id, &artifacts) {
        Ok(closure) => {
            let root_source_file = artifacts
                .iter()
                .find(|artifact| artifact.session_id.as_deref() == Some(root_session_id))
                .map(|artifact| artifact.source_file.clone())
                .ok_or_else(|| DiscoveryError::LinkedSessionNotFound {
                    session_id: root_session_id.to_string(),
                })?;
            Ok(CachedClosureSelection {
                closure,
                root_source_file,
                root_identity_validated: true,
            })
        }
        Err(DiscoveryError::LinkedSessionNotFound { .. }) => {
            let candidates = indexes
                .iter()
                .filter(|index| {
                    index.state.high_water_mark == 0
                        && index
                            .state
                            .source_file
                            .file_name()
                            .is_some_and(|file_name| file_name.contains(root_session_id))
                })
                .collect::<Vec<_>>();
            match candidates.as_slice() {
                [candidate] => {
                    let source_file = candidate.state.source_file.clone();
                    Ok(CachedClosureSelection {
                        closure: DirectLinkedClosure {
                            included_source_files: BTreeSet::from([source_file.clone()]),
                            linkage_source_files: BTreeSet::from([source_file.clone()]),
                        },
                        root_source_file: source_file,
                        root_identity_validated: false,
                    })
                }
                [] => Err(BoundedClosureError::Discovery(
                    DiscoveryError::LinkedSessionNotFound {
                        session_id: root_session_id.to_string(),
                    },
                )),
                _ => Err(BoundedClosureError::Discovery(
                    DiscoveryError::AmbiguousLinkedSession {
                        session_id: root_session_id.to_string(),
                        source_files: candidates
                            .iter()
                            .map(|candidate| candidate.state.source_file.clone())
                            .collect(),
                    },
                )),
            }
        }
        Err(error) => Err(BoundedClosureError::Discovery(error)),
    }
}

fn inspect_startup_readiness(
    expected_state: &RawTraceSourceState,
) -> Result<BoundedClosureStartupReadiness, BoundedClosureError> {
    let file = fs::File::open(&expected_state.source_file).map_err(|source| {
        BoundedClosureError::Inspect {
            path: expected_state.source_file.clone(),
            source,
        }
    })?;
    let mut readiness = BoundedClosureStartupReadiness::default();
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|source| BoundedClosureError::Inspect {
            path: expected_state.source_file.clone(),
            source,
        })?;
        if line.trim().is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        update_startup_readiness(&mut readiness, &value);
    }
    verify_source_state(expected_state)?;
    Ok(readiness)
}

fn update_startup_readiness(readiness: &mut BoundedClosureStartupReadiness, value: &Value) {
    let event_type = value.get("type").and_then(Value::as_str);
    if event_type.is_some_and(|event_type| event_type != "session_meta") {
        readiness.has_session_activity = true;
    }

    for text in startup_text_fragments(value) {
        if !text.trim().is_empty() {
            readiness.has_literal_directive_text = true;
            if text_has_path_hint(text) {
                readiness.has_path_hint = true;
            }
        }
    }

    if let Some(arguments) = startup_tool_call_arguments(value) {
        if serde_json::from_str::<Value>(arguments)
            .ok()
            .is_some_and(|arguments| arguments.is_object())
        {
            readiness.has_parseable_tool_call_arguments = true;
        }
    }
}

fn startup_text_fragments(value: &Value) -> Vec<&str> {
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

fn startup_tool_call_arguments(value: &Value) -> Option<&str> {
    let payload = value.get("payload")?;
    (payload.get("type").and_then(Value::as_str) == Some("function_call"))
        .then(|| payload.get("arguments").and_then(Value::as_str))
        .flatten()
}

fn text_has_path_hint(text: &str) -> bool {
    text.split_whitespace().any(|raw_token| {
        let token = raw_token
            .trim_matches(|character: char| {
                matches!(
                    character,
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

fn verify_decoded_rollout(
    index: &SourceIndex,
    rollout: &IngestedRolloutFile,
) -> Result<(), BoundedClosureError> {
    if rollout.format != index.format {
        return Err(BoundedClosureError::ClosureIndexMismatch {
            path: index.state.source_file.clone(),
            reason: format!(
                "format changed from {:?} to {:?}",
                index.format, rollout.format
            ),
        });
    }
    if rollout.session_id != index.session_id {
        return Err(BoundedClosureError::ClosureIndexMismatch {
            path: index.state.source_file.clone(),
            reason: format!(
                "session identity changed from {:?} to {:?}",
                index.session_id, rollout.session_id
            ),
        });
    }
    if rollout.records.len() != index.event_count {
        return Err(BoundedClosureError::ClosureIndexMismatch {
            path: index.state.source_file.clone(),
            reason: format!(
                "event count changed from {} to {}",
                index.event_count,
                rollout.records.len()
            ),
        });
    }
    let decoded_linkage = extract_rollout_linkage_metadata(rollout);
    if decoded_linkage != index.linkage_metadata {
        return Err(BoundedClosureError::ClosureIndexMismatch {
            path: index.state.source_file.clone(),
            reason: "fully decoded linkage differs from the validated minimal index".to_string(),
        });
    }
    Ok(())
}

fn closure_revision(
    root_session_id: &str,
    root_identity_validated: bool,
    selected_indexes: &[SourceIndex],
    linkage_indexes: &[SourceIndex],
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(root_session_id.as_bytes());
    hasher.update(&[u8::from(root_identity_validated)]);
    for (kind, indexes) in [(b's', selected_indexes), (b'l', linkage_indexes)] {
        for index in indexes {
            hasher.update(&[kind]);
            hasher.update(index.state.source_file.as_str().as_bytes());
            hasher.update(&index.state.generation.to_le_bytes());
            hasher.update(&index.state.high_water_mark.to_le_bytes());
        }
    }
    hasher.finalize().to_hex().to_string()
}

fn complete_source_state_token(root_session_id: &str, states: &[RawTraceSourceState]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"agent-session-compactor:bounded-closure-state:v1\0");
    update_length_prefixed(&mut hasher, root_session_id.as_bytes());
    hasher.update(&(states.len() as u64).to_le_bytes());
    for state in states {
        update_length_prefixed(&mut hasher, state.source_file.as_str().as_bytes());
        hasher.update(&state.generation.to_le_bytes());
        hasher.update(&state.high_water_mark.to_le_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn update_length_prefixed(hasher: &mut blake3::Hasher, value: &[u8]) {
    hasher.update(&(value.len() as u64).to_le_bytes());
    hasher.update(value);
}

fn envelope_error(
    path: &Utf8Path,
    record: &IndexedEnvelopeRecord,
    reason: String,
) -> BoundedClosureError {
    BoundedClosureError::IndeterminateEnvelope {
        path: path.to_owned(),
        line_number: record.line_number,
        reason,
    }
}

fn provenance(path: &Utf8Path, record: &IndexedEnvelopeRecord) -> RolloutRowProvenance {
    RolloutRowProvenance {
        source_file: path.to_owned(),
        line_number: record.line_number,
        event_index: record.event_index,
    }
}

fn required_non_empty_string<'a>(value: Option<&'a Value>, field: &str) -> Result<&'a str, String> {
    let value = value.ok_or_else(|| format!("{field} is required"))?;
    let value = value
        .as_str()
        .ok_or_else(|| format!("{field} must be a string"))?;
    if value.trim().is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(value)
}

fn required_string<'a>(
    value: Option<&'a Value>,
    field: &str,
    allow_empty: bool,
) -> Result<&'a str, String> {
    let value = value.ok_or_else(|| format!("{field} is required"))?;
    let value = value
        .as_str()
        .ok_or_else(|| format!("{field} must be a string"))?;
    if !allow_empty && value.trim().is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(value)
}

fn optional_string_preserve_empty<'a>(
    value: Option<&'a Value>,
    field: &str,
) -> Result<Option<&'a str>, String> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        Some(_) => Err(format!("{field} must be a string or null")),
    }
}

fn optional_non_empty_string<'a>(
    value: Option<&'a Value>,
    field: &str,
) -> Result<Option<&'a str>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let value = value
        .as_str()
        .ok_or_else(|| format!("{field} must be a string or null"))?;
    Ok((!value.trim().is_empty()).then_some(value))
}

fn optional_non_empty_string_lossy(value: Option<&Value>) -> Option<&str> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn extract_spawn_result_child_id(output: &str) -> Option<String> {
    serde_json::from_str::<Value>(output)
        .ok()?
        .get("agent_id")?
        .as_str()
        .map(str::to_string)
}
