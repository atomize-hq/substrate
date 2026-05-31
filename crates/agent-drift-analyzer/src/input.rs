use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader};

use agent_session_compactor::{
    BundleManifest, CompactionKind, CompactionRow, DedupeGroup, DedupeGroupV0_2, ExportRowV0_2,
    RowRef, RowRefV0_2,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputBundle {
    pub manifest: BundleManifest,
    pub archival_rows: Vec<CompactionRow>,
    pub compact_rows: Vec<CompactionRow>,
    pub dedupe_groups: Vec<DedupeGroup>,
    pub sessions: Vec<BundleSession>,
    pub unscoped_archival_rows: Vec<CompactionRow>,
    pub unscoped_compact_rows: Vec<CompactionRow>,
    pub surface: AnalyzerSurface,
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
    #[error("artifact {path} references unknown source_file_id {source_file_id}")]
    UnknownSourceFileId {
        path: Utf8PathBuf,
        source_file_id: u32,
    },
    #[error("bundle {input_dir} does not contain any session-scoped rows")]
    NoSessions { input_dir: Utf8PathBuf },
    #[error("bundle contract is not sufficient for analyzer working-set inference: {reason}")]
    InsufficientContract { reason: String },
    #[error("compactor row ordering is unstable for session {session_id}")]
    UnstableOrdering { session_id: String },
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

    let sessions = build_sessions(input_dir, &archival_rows, &compact_rows)?;
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

fn build_file_registry(
    manifest: &BundleManifest,
) -> Result<BTreeMap<u32, Utf8PathBuf>, InputError> {
    let mut files_by_id = BTreeMap::new();
    let mut seen_paths = BTreeSet::new();
    for file in &manifest.files {
        if files_by_id.insert(file.id, file.path.clone()).is_some() {
            return Err(InputError::DuplicateSourceFileId { id: file.id });
        }
        if !seen_paths.insert(file.path.clone()) {
            return Err(InputError::DuplicateSourceFilePath {
                path: file.path.clone(),
            });
        }
    }
    Ok(files_by_id)
}

fn resolve_rows(
    artifact_path: &Utf8Path,
    file_registry: &BTreeMap<u32, Utf8PathBuf>,
    rows: &[ExportRowV0_2],
) -> Result<Vec<CompactionRow>, InputError> {
    rows.iter()
        .map(|row| {
            let source_file =
                resolve_source_file(artifact_path, file_registry, row.source_file_id)?;
            Ok(CompactionRow {
                source_file,
                source_kind: row.source_kind,
                session_id: row.session_id.clone(),
                turn_id: row.turn_id.clone(),
                event_index: row.event_index,
                line_number: row.line_number,
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
    file_registry: &BTreeMap<u32, Utf8PathBuf>,
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
    file_registry: &BTreeMap<u32, Utf8PathBuf>,
    row_ref: &RowRefV0_2,
) -> Result<RowRef, InputError> {
    Ok(RowRef {
        source_file: resolve_source_file(artifact_path, file_registry, row_ref.source_file_id)?,
        line_number: row_ref.line_number,
        event_index: row_ref.event_index,
        row_ordinal: row_ref.row_ordinal,
    })
}

fn resolve_source_file(
    artifact_path: &Utf8Path,
    file_registry: &BTreeMap<u32, Utf8PathBuf>,
    source_file_id: u32,
) -> Result<Utf8PathBuf, InputError> {
    file_registry
        .get(&source_file_id)
        .cloned()
        .ok_or_else(|| InputError::UnknownSourceFileId {
            path: artifact_path.to_owned(),
            source_file_id,
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

    let session_ids = archival_by_session
        .keys()
        .chain(compact_by_session.keys())
        .cloned()
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
    dedupe_groups: &[DedupeGroup],
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
    let repetition_preserved =
        archival_rows.len() >= compact_rows.len() && !dedupe_groups.is_empty();
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
    if !truth_artifact_hints {
        return Err(InputError::InsufficientContract {
            reason: "no path-like hints survived in directive text".to_string(),
        });
    }
    if !working_set_hints || !tool_argument_json {
        return Err(InputError::InsufficientContract {
            reason: "tool-call argument payloads are not parseable enough to infer command families and working-set paths".to_string(),
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
        (
            &left.source_file,
            left.event_index,
            left.line_number,
            left.row_ordinal,
        )
            .cmp(&(
                &right.source_file,
                right.event_index,
                right.line_number,
                right.row_ordinal,
            ))
    });
}

fn rows_are_stable(rows: &[CompactionRow]) -> bool {
    rows.windows(2).all(|pair| {
        let left = &pair[0];
        let right = &pair[1];
        (
            &left.source_file,
            left.event_index,
            left.line_number,
            left.row_ordinal,
        ) <= (
            &right.source_file,
            right.event_index,
            right.line_number,
            right.row_ordinal,
        )
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
        let token = raw_token
            .trim_matches(|ch: char| {
                matches!(
                    ch,
                    ',' | ':' | ';' | '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | '`'
                )
            })
            .trim_end_matches('.');
        if looks_like_path(token) {
            paths.insert(token.to_string());
        }
    }
    paths.into_iter().collect()
}

fn looks_like_path(token: &str) -> bool {
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
}

fn row_ref_key(row: RowRef) -> (Utf8PathBuf, usize, usize, usize) {
    (
        row.source_file,
        row.line_number,
        row.event_index,
        row.row_ordinal,
    )
}
