use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader};

use agent_session_compactor::{
    BundleManifest, CompactionKind, CompactionRow, DedupeGroup, DedupeGroupV0_2, DelegationLink,
    DelegationLinkState, ExportRowV0_2, RowRef, RowRefV0_2,
};
use camino::{Utf8Path, Utf8PathBuf};
use serde::de::DeserializeOwned;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzerSurface {
    pub literal_objective_rows: bool,
    pub truth_artifact_hints: bool,
    pub working_set_hints: bool,
    pub repetition_preserved: bool,
    pub stable_row_refs: bool,
    pub tool_argument_json: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleSession {
    pub session_id: String,
    pub archival_rows: Vec<CompactionRow>,
    pub compact_rows: Vec<CompactionRow>,
}

/// Analyzer-owned adjacency index containing only verified direct delegation links.
///
/// Non-verified observations remain available through [`InputBundle::manifest`]
/// but never enter this semantic graph.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DelegationLinkGraph {
    /// Verified direct-link adjacency keyed deterministically by included session id.
    pub by_session_id: BTreeMap<String, SessionDelegationLinks>,
}

/// Verified direct links entering and leaving one included bundle session.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionDelegationLinks {
    /// Links for which this session is the child.
    pub parent_links: Vec<DelegationLink>,
    /// Links for which this session is the parent.
    pub child_links: Vec<DelegationLink>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputBundle {
    pub manifest: BundleManifest,
    pub archival_rows: Vec<CompactionRow>,
    pub compact_rows: Vec<CompactionRow>,
    pub dedupe_groups: Vec<DedupeGroup>,
    pub sessions: Vec<BundleSession>,
    pub delegation_graph: DelegationLinkGraph,
    pub unscoped_archival_rows: Vec<CompactionRow>,
    pub unscoped_compact_rows: Vec<CompactionRow>,
    pub surface: AnalyzerSurface,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileEntry {
    path: Utf8PathBuf,
    session_id: Option<String>,
    turns: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("required compactor artifact is missing: {path}")]
    MissingArtifact { path: Utf8PathBuf },
    #[error("failed to read artifact {path}: {source}")]
    ReadArtifact {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse JSON artifact {path}: {source}")]
    ParseJson {
        path: Utf8PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to parse JSONL artifact {path} at line {line_number}: {source}")]
    ParseJsonl {
        path: Utf8PathBuf,
        line_number: usize,
        #[source]
        source: serde_json::Error,
    },
    #[error("unsupported compactor bundle schema in {path}: expected v0.2, found {found}")]
    UnsupportedSchemaVersion { path: Utf8PathBuf, found: String },
    #[error("manifest file registry reuses source_file_id {id}")]
    DuplicateSourceFileId { id: u32 },
    #[error("manifest file registry repeats path {path}")]
    DuplicateSourceFilePath { path: Utf8PathBuf },
    #[error("manifest file registry repeats turn_id {turn_id} for source file {path}")]
    DuplicateSourceFileTurnId { path: Utf8PathBuf, turn_id: String },
    #[error("artifact {path} references unknown source_file_id {source_file_id}")]
    UnknownSourceFileId {
        path: Utf8PathBuf,
        source_file_id: u32,
    },
    #[error("artifact {path} references unknown turn_id_ref {turn_id_ref} for source_file_id {source_file_id}")]
    UnknownTurnIdRef {
        path: Utf8PathBuf,
        source_file_id: u32,
        turn_id_ref: u16,
    },
    #[error("bundle {input_dir} does not contain any session-scoped rows")]
    NoSessions { input_dir: Utf8PathBuf },
    #[error("bundle contract is not sufficient for analyzer working-set inference: {reason}")]
    InsufficientContract { reason: String },
    #[error("compactor row ordering is unstable for session {session_id}")]
    UnstableOrdering { session_id: String },
    #[error(
        "verified delegation link {parent_session_id} -> {child_session_id} references session {missing_session_id} that is not included in the bundle"
    )]
    VerifiedDelegationSessionMissing {
        parent_session_id: String,
        child_session_id: String,
        missing_session_id: String,
    },
    #[error("dedupe audit references an archival row that is not present: {row:?}")]
    MissingDedupeRepresentative { row: RowRef },
}

pub fn load_bundle(input_dir: &Utf8Path) -> Result<InputBundle, InputError> {
    let manifest_path = input_dir.join("manifest.json");
    let archival_path = input_dir.join("rows.archival.jsonl");
    let compact_path = input_dir.join("rows.compact.jsonl");
    let audit_path = input_dir.join("dedupe-audit.jsonl");

    let manifest: BundleManifest = read_json_file(&manifest_path)?;
    validate_manifest_schema(&manifest, &manifest_path)?;
    let file_registry = build_file_registry(&manifest)?;
    let archival_rows = resolve_rows(
        &archival_path,
        &file_registry,
        &read_jsonl_file::<ExportRowV0_2>(&archival_path)?,
    )?;
    let compact_rows = resolve_rows(
        &compact_path,
        &file_registry,
        &read_jsonl_file::<ExportRowV0_2>(&compact_path)?,
    )?;
    let dedupe_groups = resolve_dedupe_groups(
        &audit_path,
        &file_registry,
        &read_jsonl_file::<DedupeGroupV0_2>(&audit_path)?,
    )?;

    validate_dedupe_refs(&archival_rows, &dedupe_groups)?;

    let sessions = build_sessions(input_dir, &file_registry, &archival_rows, &compact_rows)?;
    let delegation_graph = build_delegation_graph(&manifest, &sessions)?;
    let unscoped_archival_rows = archival_rows
        .iter()
        .filter(|row| row.session_id.is_none())
        .cloned()
        .collect::<Vec<_>>();
    let unscoped_compact_rows = compact_rows
        .iter()
        .filter(|row| row.session_id.is_none())
        .cloned()
        .collect::<Vec<_>>();
    let surface = validate_surface(&archival_rows, &compact_rows, &dedupe_groups)?;

    Ok(InputBundle {
        manifest,
        archival_rows,
        compact_rows,
        dedupe_groups,
        sessions,
        delegation_graph,
        unscoped_archival_rows,
        unscoped_compact_rows,
        surface,
    })
}

fn validate_manifest_schema(manifest: &BundleManifest, path: &Utf8Path) -> Result<(), InputError> {
    if manifest.schema_version != "v0.2" {
        return Err(InputError::UnsupportedSchemaVersion {
            path: path.to_owned(),
            found: manifest.schema_version.clone(),
        });
    }
    Ok(())
}

fn build_file_registry(manifest: &BundleManifest) -> Result<BTreeMap<u32, FileEntry>, InputError> {
    let mut files_by_id = BTreeMap::new();
    let mut seen_paths = BTreeSet::new();
    for file in &manifest.files {
        if files_by_id
            .insert(
                file.id,
                FileEntry {
                    path: file.path.clone(),
                    session_id: file.session_id.clone(),
                    turns: file.turns.clone(),
                },
            )
            .is_some()
        {
            return Err(InputError::DuplicateSourceFileId { id: file.id });
        }
        if !seen_paths.insert(file.path.clone()) {
            return Err(InputError::DuplicateSourceFilePath {
                path: file.path.clone(),
            });
        }
        let mut seen_turns = BTreeSet::new();
        for turn_id in &file.turns {
            if !seen_turns.insert(turn_id.clone()) {
                return Err(InputError::DuplicateSourceFileTurnId {
                    path: file.path.clone(),
                    turn_id: turn_id.clone(),
                });
            }
        }
    }
    Ok(files_by_id)
}

fn resolve_rows(
    artifact_path: &Utf8Path,
    file_registry: &BTreeMap<u32, FileEntry>,
    rows: &[ExportRowV0_2],
) -> Result<Vec<CompactionRow>, InputError> {
    rows.iter()
        .map(|row| {
            let file_entry = resolve_file_entry(artifact_path, file_registry, row.source_file_id)?;
            Ok(CompactionRow {
                source_file: file_entry.path.clone(),
                source_kind: row.source_kind,
                session_id: file_entry.session_id.clone(),
                turn_id: resolve_turn_id(
                    artifact_path,
                    file_entry,
                    row.source_file_id,
                    row.turn_id_ref,
                )?,
                event_index: row.event_index,
                line_number: row.event_index + 1,
                row_ordinal: row.row_ordinal,
                timestamp: row.timestamp,
                kind: row.kind,
                user_message_role: row.user_message_role,
                dedupe_identity: row.dedupe_identity.clone(),
                text: row.text.clone(),
                canonical_text: String::new(),
                text_hash_hex: row.text_hash_hex.clone(),
            })
        })
        .collect()
}

fn resolve_dedupe_groups(
    artifact_path: &Utf8Path,
    file_registry: &BTreeMap<u32, FileEntry>,
    groups: &[DedupeGroupV0_2],
) -> Result<Vec<DedupeGroup>, InputError> {
    groups
        .iter()
        .map(|group| {
            Ok(DedupeGroup {
                kind: group.kind,
                canonical_text_hash_hex: group.canonical_text_hash_hex.clone(),
                representative: resolve_row_ref(
                    artifact_path,
                    file_registry,
                    &group.representative,
                )?,
                duplicates: group
                    .duplicates
                    .iter()
                    .map(|row_ref| resolve_row_ref(artifact_path, file_registry, row_ref))
                    .collect::<Result<Vec<_>, _>>()?,
            })
        })
        .collect()
}

fn resolve_row_ref(
    artifact_path: &Utf8Path,
    file_registry: &BTreeMap<u32, FileEntry>,
    row_ref: &RowRefV0_2,
) -> Result<RowRef, InputError> {
    Ok(RowRef {
        source_file: resolve_source_file(artifact_path, file_registry, row_ref.source_file_id)?,
        event_index: row_ref.event_index,
        row_ordinal: row_ref.row_ordinal,
    })
}

fn resolve_file_entry<'a>(
    artifact_path: &Utf8Path,
    file_registry: &'a BTreeMap<u32, FileEntry>,
    source_file_id: u32,
) -> Result<&'a FileEntry, InputError> {
    file_registry
        .get(&source_file_id)
        .ok_or_else(|| InputError::UnknownSourceFileId {
            path: artifact_path.to_owned(),
            source_file_id,
        })
}

fn resolve_source_file(
    artifact_path: &Utf8Path,
    file_registry: &BTreeMap<u32, FileEntry>,
    source_file_id: u32,
) -> Result<Utf8PathBuf, InputError> {
    Ok(
        resolve_file_entry(artifact_path, file_registry, source_file_id)?
            .path
            .clone(),
    )
}

fn resolve_turn_id(
    artifact_path: &Utf8Path,
    file_entry: &FileEntry,
    source_file_id: u32,
    turn_id_ref: Option<u16>,
) -> Result<Option<String>, InputError> {
    let Some(turn_id_ref) = turn_id_ref else {
        return Ok(None);
    };
    file_entry
        .turns
        .get(usize::from(turn_id_ref))
        .cloned()
        .map(Some)
        .ok_or_else(|| InputError::UnknownTurnIdRef {
            path: artifact_path.to_owned(),
            source_file_id,
            turn_id_ref,
        })
}

fn read_json_file<T>(path: &Utf8Path) -> Result<T, InputError>
where
    T: DeserializeOwned,
{
    if !path.exists() {
        return Err(InputError::MissingArtifact {
            path: path.to_owned(),
        });
    }
    let text = fs::read_to_string(path).map_err(|source| InputError::ReadArtifact {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| InputError::ParseJson {
        path: path.to_owned(),
        source,
    })
}

fn read_jsonl_file<T>(path: &Utf8Path) -> Result<Vec<T>, InputError>
where
    T: DeserializeOwned,
{
    if !path.exists() {
        return Err(InputError::MissingArtifact {
            path: path.to_owned(),
        });
    }
    let file = fs::File::open(path).map_err(|source| InputError::ReadArtifact {
        path: path.to_owned(),
        source,
    })?;
    let reader = BufReader::new(file);
    let mut items = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = line.map_err(|source| InputError::ReadArtifact {
            path: path.to_owned(),
            source,
        })?;
        if line.trim().is_empty() {
            continue;
        }
        let item = serde_json::from_str(&line).map_err(|source| InputError::ParseJsonl {
            path: path.to_owned(),
            line_number,
            source,
        })?;
        items.push(item);
    }
    Ok(items)
}

fn build_sessions(
    input_dir: &Utf8Path,
    file_registry: &BTreeMap<u32, FileEntry>,
    archival_rows: &[CompactionRow],
    compact_rows: &[CompactionRow],
) -> Result<Vec<BundleSession>, InputError> {
    let mut archival_by_session: BTreeMap<String, Vec<CompactionRow>> = BTreeMap::new();
    let mut compact_by_session: BTreeMap<String, Vec<CompactionRow>> = BTreeMap::new();

    for row in archival_rows.iter().filter_map(row_with_session) {
        archival_by_session
            .entry(row.session_id.clone().expect("session"))
            .or_default()
            .push(row.clone());
    }
    for row in compact_rows.iter().filter_map(row_with_session) {
        compact_by_session
            .entry(row.session_id.clone().expect("session"))
            .or_default()
            .push(row.clone());
    }

    let session_ids = file_registry
        .values()
        .filter_map(|file| file.session_id.clone())
        .chain(
            archival_by_session
                .keys()
                .chain(compact_by_session.keys())
                .cloned(),
        )
        .collect::<BTreeSet<_>>();
    if session_ids.is_empty() {
        return Err(InputError::NoSessions {
            input_dir: input_dir.to_owned(),
        });
    }

    let mut sessions = Vec::new();
    for session_id in session_ids {
        let mut archival = archival_by_session.remove(&session_id).unwrap_or_default();
        let mut compact = compact_by_session.remove(&session_id).unwrap_or_default();
        sort_rows(&mut archival);
        sort_rows(&mut compact);
        if !rows_are_stable(&archival) || !rows_are_stable(&compact) {
            return Err(InputError::UnstableOrdering { session_id });
        }
        sessions.push(BundleSession {
            session_id,
            archival_rows: archival,
            compact_rows: compact,
        });
    }
    Ok(sessions)
}

fn build_delegation_graph(
    manifest: &BundleManifest,
    sessions: &[BundleSession],
) -> Result<DelegationLinkGraph, InputError> {
    let session_ids = sessions
        .iter()
        .map(|session| session.session_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut verified_links = manifest
        .delegation_links
        .iter()
        .filter(|link| link.state == DelegationLinkState::Verified)
        .cloned()
        .collect::<Vec<_>>();
    verified_links.sort_by(|left, right| {
        (
            &left.parent_session_id,
            &left.child_session_id,
            &left.child_origin_parent_session_id,
            left.depth,
            left.state,
            &left.parent_evidence,
            &left.child_evidence,
        )
            .cmp(&(
                &right.parent_session_id,
                &right.child_session_id,
                &right.child_origin_parent_session_id,
                right.depth,
                right.state,
                &right.parent_evidence,
                &right.child_evidence,
            ))
    });
    verified_links.dedup();

    let mut graph = DelegationLinkGraph::default();
    for link in verified_links {
        let missing_session_id = [&link.parent_session_id, &link.child_session_id]
            .into_iter()
            .find(|session_id| !session_ids.contains(session_id.as_str()))
            .cloned();
        if let Some(missing_session_id) = missing_session_id {
            return Err(InputError::VerifiedDelegationSessionMissing {
                parent_session_id: link.parent_session_id,
                child_session_id: link.child_session_id,
                missing_session_id,
            });
        }

        graph
            .by_session_id
            .entry(link.parent_session_id.clone())
            .or_default()
            .child_links
            .push(link.clone());
        graph
            .by_session_id
            .entry(link.child_session_id.clone())
            .or_default()
            .parent_links
            .push(link);
    }

    Ok(graph)
}

fn validate_dedupe_refs(
    archival_rows: &[CompactionRow],
    dedupe_groups: &[DedupeGroup],
) -> Result<(), InputError> {
    let rows = archival_rows
        .iter()
        .map(RowRef::from_row)
        .map(row_ref_key)
        .collect::<BTreeSet<_>>();
    for group in dedupe_groups {
        if !rows.contains(&row_ref_key(group.representative.clone())) {
            return Err(InputError::MissingDedupeRepresentative {
                row: group.representative.clone(),
            });
        }
    }
    Ok(())
}

fn validate_surface(
    archival_rows: &[CompactionRow],
    compact_rows: &[CompactionRow],
    _dedupe_groups: &[DedupeGroup],
) -> Result<AnalyzerSurface, InputError> {
    let literal_objective_rows = compact_rows.iter().any(|row| {
        matches!(
            row.kind,
            CompactionKind::UserMessage
                | CompactionKind::DeveloperMessage
                | CompactionKind::SystemMessage
        ) && !row.text.trim().is_empty()
    });
    let truth_artifact_hints = compact_rows
        .iter()
        .any(|row| !extract_path_hints(&row.text).is_empty());
    let working_set_hints = compact_rows
        .iter()
        .filter(|row| row.kind == CompactionKind::ToolCall)
        .any(|row| {
            parse_tool_payload(&row.text).is_some() || !extract_path_hints(&row.text).is_empty()
        });
    let repetition_preserved = archival_rows.len() >= compact_rows.len();
    let stable_row_refs = archival_rows
        .iter()
        .map(RowRef::from_row)
        .map(row_ref_key)
        .collect::<BTreeSet<_>>()
        .len()
        == archival_rows.len();
    let tool_argument_json = compact_rows
        .iter()
        .filter(|row| row.kind == CompactionKind::ToolCall)
        .any(|row| parse_tool_payload(&row.text).is_some());

    if !literal_objective_rows {
        return Err(InputError::InsufficientContract {
            reason: "no literal user/developer/system rows survived normalization".to_string(),
        });
    }
    if !repetition_preserved {
        return Err(InputError::InsufficientContract {
            reason: "archival rows do not preserve repetition beyond the compacted view"
                .to_string(),
        });
    }
    if !stable_row_refs {
        return Err(InputError::InsufficientContract {
            reason: "row references are not unique and stable".to_string(),
        });
    }

    Ok(AnalyzerSurface {
        literal_objective_rows,
        truth_artifact_hints,
        working_set_hints,
        repetition_preserved,
        stable_row_refs,
        tool_argument_json,
    })
}

fn sort_rows(rows: &mut [CompactionRow]) {
    rows.sort_by(|left, right| {
        (&left.source_file, left.event_index, left.row_ordinal).cmp(&(
            &right.source_file,
            right.event_index,
            right.row_ordinal,
        ))
    });
}

fn rows_are_stable(rows: &[CompactionRow]) -> bool {
    rows.windows(2).all(|pair| {
        let left = &pair[0];
        let right = &pair[1];
        (&left.source_file, left.event_index, left.row_ordinal)
            <= (&right.source_file, right.event_index, right.row_ordinal)
    })
}

fn row_with_session(row: &CompactionRow) -> Option<&CompactionRow> {
    row.session_id.as_ref()?;
    Some(row)
}

pub(crate) fn parse_tool_payload(text: &str) -> Option<Value> {
    serde_json::from_str::<Value>(text)
        .ok()
        .filter(|value| value.is_object())
}

pub(crate) fn extract_path_hints(text: &str) -> Vec<String> {
    let mut paths = BTreeSet::new();
    for raw_token in text.split_whitespace() {
        let token = trim_path_token(raw_token);
        if looks_like_path(token, false) {
            paths.insert(token.to_string());
        }
    }
    paths.into_iter().collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DirectivePathHint {
    pub path: String,
    pub control_only: bool,
    pub trusted_root: bool,
}

/// Extracts path-shaped directive evidence with invocation-aware control classification.
///
/// Only a known control token at the start of a logical line opens a control invocation.
/// Option-only continuation lines remain part of that invocation; ordinary prose containing a
/// slash-prefixed absolute path does not. Authority candidates reject raw parent components before
/// punctuation cleanup or lexical normalization.
pub(crate) fn extract_directive_path_hints(text: &str) -> Vec<String> {
    classify_directive_path_hints(text)
        .into_iter()
        .map(|hint| hint.path)
        .collect()
}

pub(crate) fn classify_directive_path_hints(text: &str) -> Vec<DirectivePathHint> {
    let mut paths = BTreeMap::<String, (bool, bool)>::new();
    let mut control_continuation = false;

    for line in text.lines() {
        let tokens = line.split_whitespace().collect::<Vec<_>>();
        let Some(first_index) = first_invocation_token_index(&tokens) else {
            control_continuation = false;
            continue;
        };
        let first = trim_control_token(tokens[first_index]);
        let starts_control = is_known_control_invocation(first);
        let continues_control = control_continuation && first.starts_with('-');
        let control_only = starts_control || continues_control;
        let phrase_trusted_root = trusted_repository_root_hint(line);

        let mut index = first_index;
        while index < tokens.len() {
            let raw_token = tokens[index];
            let option_token = trim_path_delimiters(raw_token);
            if is_path_option(option_token) {
                if let Some(raw_value) = tokens.get(index + 1) {
                    insert_directive_candidate(
                        &mut paths,
                        raw_value,
                        true,
                        control_only,
                        option_token == "--root",
                    );
                    index += 2;
                    continue;
                }
            } else if let Some((option, raw_value)) = option_token.split_once('=') {
                if is_path_option(option) {
                    insert_directive_candidate(
                        &mut paths,
                        raw_value,
                        true,
                        control_only,
                        option == "--root",
                    );
                    index += 1;
                    continue;
                }
            }

            if !(control_only && is_known_control_invocation(trim_control_token(raw_token))) {
                insert_directive_candidate(&mut paths, raw_token, false, control_only, false);
            }
            index += 1;
        }

        if !control_only {
            if let Some(root) = phrase_trusted_root {
                paths
                    .entry(root)
                    .and_modify(|existing| {
                        existing.0 = false;
                        existing.1 = true;
                    })
                    .or_insert((false, true));
            }
        }

        control_continuation = starts_control || continues_control;
    }

    paths
        .into_iter()
        .map(|(path, (control_only, trusted_root))| DirectivePathHint {
            path,
            control_only,
            trusted_root,
        })
        .collect()
}

fn first_invocation_token_index(tokens: &[&str]) -> Option<usize> {
    tokens
        .iter()
        .position(|token| !is_line_prefix_token(trim_path_delimiters(token)))
}

fn is_line_prefix_token(token: &str) -> bool {
    matches!(token, "-" | "*" | "+" | ">")
        || token
            .strip_suffix('.')
            .or_else(|| token.strip_suffix(')'))
            .is_some_and(|prefix| {
                !prefix.is_empty() && prefix.chars().all(|ch| ch.is_ascii_digit())
            })
}

fn is_known_control_invocation(token: &str) -> bool {
    let Some(name) = token.strip_prefix('/') else {
        return false;
    };
    matches!(
        name,
        "goal"
            | "review"
            | "spec"
            | "plan"
            | "compact"
            | "clear"
            | "status"
            | "help"
            | "init"
            | "new"
            | "resume"
            | "fork"
            | "permissions"
            | "model"
            | "feedback"
            | "logout"
            | "mention"
            | "apps"
            | "skills"
            | "plugins"
    )
}

fn is_path_option(token: &str) -> bool {
    matches!(
        token,
        "--path" | "--file" | "--dir" | "--directory" | "--scope" | "--root"
    )
}

fn trusted_repository_root_hint(line: &str) -> Option<String> {
    const MARKER: &str = "trusted repository root";

    let lower = line.to_ascii_lowercase();
    let suffix = &line[lower.find(MARKER)? + MARKER.len()..];
    for raw_token in suffix.split_whitespace() {
        if raw_path_has_parent_component(raw_token) {
            return None;
        }
        let token = trim_path_token(raw_token);
        if token.is_empty() || !looks_like_path(token, true) {
            continue;
        }
        return normalize_directive_path(token).and_then(|path| trusted_repository_root(&path));
    }
    None
}

fn insert_directive_candidate(
    paths: &mut BTreeMap<String, (bool, bool)>,
    raw_token: &str,
    explicit_option: bool,
    control_only: bool,
    trusted_root: bool,
) {
    if raw_path_has_parent_component(raw_token) {
        return;
    }
    let token = if explicit_option {
        trim_option_path_token(raw_token)
    } else {
        trim_path_token(raw_token)
    };
    if token.is_empty() || (!explicit_option && !looks_like_path(token, true)) {
        return;
    }
    let Some(path) = normalize_directive_path(token) else {
        return;
    };
    paths
        .entry(path)
        .and_modify(|existing| {
            existing.0 &= control_only;
            existing.1 |= trusted_root && !control_only;
        })
        .or_insert((control_only, trusted_root && !control_only));
}

fn raw_path_has_parent_component(raw_token: &str) -> bool {
    let raw_value = raw_token
        .split_once('=')
        .map_or(raw_token, |(_, value)| value);
    raw_value.split(['/', '\\']).any(|component| {
        component.trim_matches(|ch: char| {
            matches!(
                ch,
                ',' | ':' | ';' | '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | '`'
            )
        }) == ".."
    })
}

fn trim_path_delimiters(token: &str) -> &str {
    token.trim_matches(|ch: char| {
        matches!(
            ch,
            ',' | ':' | ';' | '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | '`'
        )
    })
}

fn trim_path_token(token: &str) -> &str {
    trim_path_delimiters(token).trim_end_matches('.')
}

fn trim_option_path_token(token: &str) -> &str {
    let token = trim_path_delimiters(token);
    if token == "." {
        token
    } else {
        token.trim_end_matches('.')
    }
}

fn trim_control_token(token: &str) -> &str {
    let token = token.trim_matches(|ch: char| {
        matches!(ch, '"' | '\'' | '`' | '(' | ')' | '[' | ']' | '{' | '}')
    });
    token
        .split_once('=')
        .map_or(token, |(directive, _)| directive)
        .trim_end_matches([':', ',', ';', '.'])
}

fn looks_like_path(token: &str, portable_separators: bool) -> bool {
    if token.is_empty() || token.starts_with("http://") || token.starts_with("https://") {
        return false;
    }
    let has_separator = token.contains('/')
        || token.starts_with('.')
        || (portable_separators && token.contains('\\'));
    let has_extension = [
        ".md", ".rs", ".toml", ".json", ".jsonl", ".yaml", ".yml", ".sh", ".txt",
    ]
    .iter()
    .any(|suffix| token.ends_with(suffix));
    has_separator || has_extension
}

/// Lexically normalizes a validated repository-relative filesystem path.
///
/// Absolute truth identity and trusted typed-path resolution use separate helpers below; a caller-
/// supplied workdir is never treated as repository-root authority.
pub(crate) fn normalize_repo_path(path: &str, _workdir: Option<&str>) -> Option<String> {
    let path = strip_path_location_suffix(path);
    let parsed = parse_lexical_path(path)?;
    parsed
        .root
        .is_none()
        .then(|| render_repo_path(&parsed.components))
}

pub(crate) fn normalize_directive_path(path: &str) -> Option<String> {
    if raw_path_has_parent_component(path) {
        return None;
    }
    let parsed = parse_lexical_path(strip_path_location_suffix(path))?;
    Some(render_truth_path(&parsed))
}

/// Returns a canonical absolute directory scope that may anchor typed cwd resolution.
pub(crate) fn trusted_repository_root(path: &str) -> Option<String> {
    let parsed = parse_truth_path(path)?;
    parsed.root?;
    Some(render_truth_path(&parsed))
}

/// Resolves one typed filesystem path without allowing the tool-provided cwd to define authority.
///
/// Absolute paths and explicit cwd values require a separately observed trusted repository root.
/// A relative typed path without cwd remains a repository-relative identity. A relative path with
/// cwd is resolved through that cwd and retains root-aware absolute identity for truth grounding.
pub(crate) fn normalize_typed_path(
    path: &str,
    cwd: Option<&str>,
    trusted_roots: &[String],
) -> Option<String> {
    let parsed_roots = trusted_roots
        .iter()
        .filter_map(|root| parse_truth_path(root))
        .filter(|root| root.root.is_some())
        .collect::<Vec<_>>();
    let raw = strip_path_location_suffix(path).trim();

    if path_is_lexically_relative(raw) {
        let Some(cwd) = cwd else {
            let parsed = parse_lexical_path(raw)?;
            return Some(render_repo_path(&parsed.components));
        };
        let (cwd, trusted_root) = resolve_absolute_under_unique_root(cwd, &parsed_roots)?;
        let resolved = resolve_relative_path(raw, &cwd, trusted_root)?;
        return Some(render_truth_path(&resolved));
    }

    let (parsed, trusted_root) = resolve_absolute_under_unique_root(raw, &parsed_roots)?;
    if let Some(cwd) = cwd {
        let (_, cwd_root) = resolve_absolute_under_unique_root(cwd, &parsed_roots)?;
        if cwd_root != trusted_root {
            return None;
        }
    }
    Some(render_truth_path(&parsed))
}

fn path_is_lexically_relative(raw: &str) -> bool {
    if raw.starts_with('/') || raw.starts_with('\\') {
        return false;
    }
    raw.as_bytes().get(1) != Some(&b':')
}

/// Projects a root-aware command path into the repository-relative identity used by
/// WrongPlanBranch. Absolute paths project only through separately observed trusted roots.
pub(crate) fn repo_relative_path_identity(path: &str, trusted_roots: &[String]) -> Option<String> {
    let raw = strip_path_location_suffix(path).trim();
    if path_is_lexically_relative(raw) {
        let parsed = parse_lexical_path(raw)?;
        return Some(render_repo_path(&parsed.components));
    }
    let parsed_roots = trusted_roots
        .iter()
        .filter_map(|root| parse_truth_path(root))
        .filter(|root| root.root.is_some())
        .collect::<Vec<_>>();
    let (parsed, trusted_root) = resolve_absolute_under_unique_root(raw, &parsed_roots)?;
    Some(render_repo_path(
        &parsed.components[trusted_root.components.len()..],
    ))
}

pub(crate) fn truth_paths_equal(left: &str, right: &str) -> bool {
    matches!(
        (parse_truth_path(left), parse_truth_path(right)),
        (Some(left), Some(right)) if left == right
    )
}

/// Returns whether `path` is exactly `scope` or is beneath it on a component boundary.
pub(crate) fn path_is_equal_or_descendant(path: &str, scope: &str) -> bool {
    let Some(path) = normalize_repo_path(path, None) else {
        return false;
    };
    let Some(scope) = normalize_repo_path(scope, None) else {
        return false;
    };
    components_start_with(&path_components(&path), &path_components(&scope))
}

pub(crate) fn truth_paths_overlap(left: &str, right: &str) -> bool {
    let Some(left) = parse_truth_path(left) else {
        return false;
    };
    let Some(right) = parse_truth_path(right) else {
        return false;
    };
    left.root == right.root
        && (components_start_with(&left.components, &right.components)
            || components_start_with(&right.components, &left.components))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedPath {
    root: Option<PathRoot>,
    components: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathRoot {
    Posix,
    Drive(char),
}

fn parse_lexical_path(raw: &str) -> Option<ParsedPath> {
    let raw = raw.trim();
    if raw.is_empty()
        || raw.contains(['\0', '\n', '\r'])
        || raw.contains("://")
        || raw.starts_with('\\')
        || raw.starts_with('~')
        || raw.starts_with('$')
        || raw.starts_with('%')
    {
        return None;
    }

    let normalized = raw.replace('\\', "/");
    if normalized.starts_with("//") {
        return None;
    }
    let (root, remainder) = if normalized.as_bytes().get(1) == Some(&b':') {
        let drive = normalized.chars().next()?.to_ascii_lowercase();
        if !drive.is_ascii_alphabetic() || normalized.as_bytes().get(2) != Some(&b'/') {
            return None;
        }
        (Some(PathRoot::Drive(drive)), &normalized[3..])
    } else if let Some(remainder) = normalized.strip_prefix('/') {
        (Some(PathRoot::Posix), remainder)
    } else {
        (None, normalized.as_str())
    };

    let mut components = Vec::new();
    for component in remainder.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop()?;
            }
            _ if component.chars().any(|ch| {
                ch.is_control() || matches!(ch, ':' | '*' | '?' | '|' | '<' | '>' | '"')
            }) =>
            {
                return None
            }
            _ => components.push(component.to_string()),
        }
    }
    Some(ParsedPath { root, components })
}

fn resolve_relative_path(
    path: &str,
    cwd: &ParsedPath,
    trusted_root: &ParsedPath,
) -> Option<ParsedPath> {
    if cwd.root.is_none()
        || cwd.root != trusted_root.root
        || !components_start_with(&cwd.components, &trusted_root.components)
    {
        return None;
    }
    let raw = strip_path_location_suffix(path).trim();
    if raw.is_empty()
        || raw.contains(['\0', '\n', '\r'])
        || raw.contains("://")
        || raw.starts_with('~')
        || raw.starts_with('$')
        || raw.starts_with('%')
    {
        return None;
    }
    let normalized = raw.replace('\\', "/");
    if normalized.starts_with('/')
        || normalized.starts_with("//")
        || normalized.as_bytes().get(1) == Some(&b':')
    {
        return None;
    }

    let mut components = cwd.components.clone();
    for component in normalized.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if components.len() <= trusted_root.components.len() {
                    return None;
                }
                components.pop()?;
            }
            _ if component.chars().any(|ch| {
                ch.is_control() || matches!(ch, ':' | '*' | '?' | '|' | '<' | '>' | '"')
            }) =>
            {
                return None
            }
            _ => components.push(component.to_string()),
        }
    }
    Some(ParsedPath {
        root: cwd.root,
        components,
    })
}

fn resolve_absolute_under_unique_root<'a>(
    path: &str,
    roots: &'a [ParsedPath],
) -> Option<(ParsedPath, &'a ParsedPath)> {
    let mut matches = roots
        .iter()
        .filter_map(|root| resolve_absolute_path(path, root).map(|resolved| (resolved, root)));
    let resolved = matches.next()?;
    matches.next().is_none().then_some(resolved)
}

fn resolve_absolute_path(path: &str, trusted_root: &ParsedPath) -> Option<ParsedPath> {
    let raw = strip_path_location_suffix(path).trim();
    if trusted_root.root.is_none()
        || raw.is_empty()
        || raw.contains(['\0', '\n', '\r'])
        || raw.contains("://")
        || raw.starts_with('\\')
        || raw.starts_with('~')
        || raw.starts_with('$')
        || raw.starts_with('%')
    {
        return None;
    }

    let normalized = raw.replace('\\', "/");
    if normalized.starts_with("//") {
        return None;
    }
    let (root, remainder) = if normalized.as_bytes().get(1) == Some(&b':') {
        let drive = normalized.chars().next()?.to_ascii_lowercase();
        if !drive.is_ascii_alphabetic() || normalized.as_bytes().get(2) != Some(&b'/') {
            return None;
        }
        (PathRoot::Drive(drive), &normalized[3..])
    } else {
        (PathRoot::Posix, normalized.strip_prefix('/')?)
    };
    if trusted_root.root != Some(root) {
        return None;
    }

    let mut components = Vec::new();
    for component in remainder.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if components.len() <= trusted_root.components.len() {
                    return None;
                }
                components.pop()?;
            }
            _ if component.chars().any(|ch| {
                ch.is_control() || matches!(ch, ':' | '*' | '?' | '|' | '<' | '>' | '"')
            }) =>
            {
                return None
            }
            _ => {
                if components.len() < trusted_root.components.len()
                    && component != trusted_root.components[components.len()].as_str()
                {
                    return None;
                }
                components.push(component.to_string());
            }
        }
    }
    components_start_with(&components, &trusted_root.components).then_some(ParsedPath {
        root: Some(root),
        components,
    })
}

fn render_truth_path(path: &ParsedPath) -> String {
    let body = path.components.join("/");
    match &path.root {
        None => render_repo_path(&path.components),
        Some(PathRoot::Posix) if body.is_empty() => "/".to_string(),
        Some(PathRoot::Posix) => format!("/{body}"),
        Some(PathRoot::Drive(drive)) if body.is_empty() => format!("{drive}:/"),
        Some(PathRoot::Drive(drive)) => format!("{drive}:/{body}"),
    }
}

fn parse_truth_path(path: &str) -> Option<ParsedPath> {
    parse_lexical_path(strip_path_location_suffix(path))
}

fn strip_path_location_suffix(path: &str) -> &str {
    let mut end = path.len();
    for _ in 0..2 {
        let candidate = &path[..end];
        let Some((base, suffix)) = candidate.rsplit_once(':') else {
            break;
        };
        if base.is_empty() || !is_location_component(suffix) {
            break;
        }
        end = base.len();
    }
    &path[..end]
}

fn is_location_component(component: &str) -> bool {
    !component.is_empty()
        && (component.chars().all(|ch| ch.is_ascii_digit())
            || matches!(component, "<line>" | "<col>"))
}

fn components_start_with(path: &[String], scope: &[String]) -> bool {
    scope.len() <= path.len() && scope.iter().zip(path).all(|(scope, path)| scope == path)
}

fn render_repo_path(components: &[String]) -> String {
    if components.is_empty() {
        ".".to_string()
    } else {
        components.join("/")
    }
}

fn path_components(path: &str) -> Vec<String> {
    if path == "." {
        Vec::new()
    } else {
        path.split('/').map(ToOwned::to_owned).collect()
    }
}

fn row_ref_key(row: RowRef) -> (Utf8PathBuf, usize, usize) {
    (row.source_file, row.event_index, row.row_ordinal)
}

#[cfg(test)]
mod path_identity_tests {
    use super::{
        classify_directive_path_hints, normalize_typed_path, repo_relative_path_identity,
        truth_paths_equal, truth_paths_overlap,
    };

    #[test]
    fn absolute_truth_path_identity_preserves_roots_and_component_boundaries() {
        assert!(truth_paths_equal(
            "/repo/docs/./truth.md",
            "/repo/docs/truth.md"
        ));
        assert!(truth_paths_overlap("/repo/docs", "/repo/docs/truth.md"));
        assert!(truth_paths_overlap("/repo/docs/truth.md", "/repo/docs"));
        assert!(truth_paths_overlap("/", "/repo/docs/truth.md"));
        assert!(!truth_paths_overlap(
            "/repo/docs",
            "/repo/docsmith/truth.md"
        ));
        assert!(!truth_paths_overlap("/repo/docs", "/other/docs/truth.md"));

        assert!(truth_paths_equal(
            r"C:\repo\docs\.\truth.md",
            "c:/repo/docs/truth.md"
        ));
        assert!(truth_paths_overlap(
            r"C:\repo\docs",
            "c:/repo/docs/truth.md"
        ));
        assert!(truth_paths_overlap(
            "c:/repo/docs/truth.md",
            r"C:\repo\docs"
        ));
        assert!(truth_paths_overlap("c:/", r"C:\repo\docs\truth.md"));
        assert!(!truth_paths_overlap(
            r"C:\repo\docs",
            r"D:\repo\docs\truth.md"
        ));
        assert!(!truth_paths_overlap(
            "/repo/docs/truth.md",
            r"C:\repo\docs\truth.md"
        ));
        assert!(!truth_paths_overlap(
            "repo/docs/truth.md",
            "/repo/docs/truth.md"
        ));
    }

    #[test]
    fn directive_authority_is_parent_safe_and_invocation_aware() {
        let hints = |text: &str| {
            classify_directive_path_hints(text)
                .into_iter()
                .map(|hint| (hint.path, hint.control_only))
                .collect::<Vec<_>>()
        };

        assert_eq!(
            hints("/goal update src/foo and then /spec"),
            vec![("src/foo".to_string(), true)]
        );
        assert_eq!(
            hints("Use /repo as the trusted root"),
            vec![("/repo".to_string(), false)]
        );
        assert_eq!(hints("--path /repo"), vec![("/repo".to_string(), false)]);
        assert_eq!(
            hints("/review\n--path /repo"),
            vec![("/repo".to_string(), true)]
        );
        let root_hints = classify_directive_path_hints(
            "Trusted repository root: /repo. Authorized scope: /repo/src/foo.",
        );
        assert_eq!(
            root_hints
                .iter()
                .filter(|hint| hint.trusted_root)
                .map(|hint| hint.path.as_str())
                .collect::<Vec<_>>(),
            vec!["/repo"]
        );
        assert!(classify_directive_path_hints("/review\n--root /repo")
            .iter()
            .all(|hint| hint.control_only && !hint.trusted_root));
        assert!(classify_directive_path_hints("--root /repo")
            .iter()
            .any(|hint| !hint.control_only && hint.trusted_root));

        for candidate in [
            "scope/../sibling",
            "scope/../..",
            "scope/..",
            "scope/../../.",
            r"scope\..\sibling",
        ] {
            assert!(hints(candidate).is_empty(), "{candidate}");
            assert!(
                hints(&format!("--path={candidate}")).is_empty(),
                "{candidate}"
            );
            assert!(
                hints(&format!("/review\n--path {candidate}")).is_empty(),
                "{candidate}"
            );
        }
    }

    #[test]
    fn typed_cwd_resolution_requires_a_separate_trusted_root() {
        let roots = vec!["/repo".to_string()];
        assert_eq!(
            normalize_typed_path("bar.rs", Some("/repo/src/foo"), &roots),
            Some("/repo/src/foo/bar.rs".to_string())
        );
        assert_eq!(
            normalize_typed_path("../bar.rs", Some("/repo/src/foo"), &roots),
            Some("/repo/src/bar.rs".to_string())
        );
        assert_eq!(
            repo_relative_path_identity("/repo/src/foo/bar.rs", &roots),
            Some("src/foo/bar.rs".to_string())
        );
        assert_eq!(
            normalize_typed_path("src/foo.rs", Some("/tmp"), &roots),
            None
        );
        assert_eq!(
            normalize_typed_path("/tmp/src/foo.rs", Some("/tmp"), &roots),
            None
        );
        assert_eq!(normalize_typed_path("src/foo.rs", Some("/repo"), &[]), None);
        assert_eq!(
            normalize_typed_path("../../../repo/secret.rs", Some("/repo/src/foo"), &roots,),
            None
        );
        assert_eq!(
            normalize_typed_path("/repo/../repo/src/foo.rs", Some("/repo"), &roots,),
            None
        );
        assert_eq!(
            normalize_typed_path("foo.rs", Some("/repo/src/../../repo/src"), &roots,),
            None
        );
        assert_eq!(
            normalize_typed_path(r"\repo\src\foo.rs", Some("/repo"), &roots),
            None
        );
    }
}
