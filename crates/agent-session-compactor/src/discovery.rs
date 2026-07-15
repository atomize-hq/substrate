use std::collections::{BTreeMap, BTreeSet};
use std::{env, path::PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use walkdir::WalkDir;

use crate::ingest::{extract_rollout_linkage_metadata, IngestedRolloutFile};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoverOptions {
    pub codex_home: Option<Utf8PathBuf>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredSessionArtifact {
    pub path: Utf8PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error(
        "failed to resolve Codex home: neither --codex-home, CODEX_HOME, nor HOME were available"
    )]
    MissingCodexHome,
    #[error("resolved Codex home from {origin} was not valid UTF-8: {path}")]
    NonUtf8Path { origin: &'static str, path: PathBuf },
    #[error("Codex sessions directory does not exist: {0}")]
    MissingSessionsDirectory(Utf8PathBuf),
    #[error("failed to walk Codex sessions directory {root}: {source}")]
    Walk {
        root: Utf8PathBuf,
        #[source]
        source: walkdir::Error,
    },
    #[error("--include-linked-children requires a non-empty session id")]
    LinkedChildrenRequireSessionId,
    #[error(
        "requested linked-closure root session was not found in parsed metadata: {session_id}"
    )]
    LinkedSessionNotFound { session_id: String },
    #[error(
        "requested linked-closure root session {session_id} has ambiguous rollout artifacts: {source_files:?}"
    )]
    AmbiguousLinkedSession {
        session_id: String,
        source_files: Vec<Utf8PathBuf>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DirectLinkedClosure {
    pub included_source_files: BTreeSet<Utf8PathBuf>,
    pub linkage_source_files: BTreeSet<Utf8PathBuf>,
}

pub(crate) fn select_direct_linked_closure(
    requested_session_id: &str,
    rollouts: &[IngestedRolloutFile],
) -> Result<DirectLinkedClosure, DiscoveryError> {
    let requested_session_id = requested_session_id.trim();
    let requested_session_id = requested_session_id
        .strip_prefix("urn:uuid:")
        .unwrap_or(requested_session_id);
    if requested_session_id.is_empty() {
        return Err(DiscoveryError::LinkedChildrenRequireSessionId);
    }

    let metadata = rollouts
        .iter()
        .map(extract_rollout_linkage_metadata)
        .collect::<Vec<_>>();
    let root_indexes = rollouts
        .iter()
        .enumerate()
        .filter_map(|(index, rollout)| {
            (rollout.session_id.as_deref() == Some(requested_session_id)).then_some(index)
        })
        .collect::<Vec<_>>();
    let root_index = match root_indexes.as_slice() {
        [] => {
            return Err(DiscoveryError::LinkedSessionNotFound {
                session_id: requested_session_id.to_string(),
            })
        }
        [root_index] => *root_index,
        _ => {
            return Err(DiscoveryError::AmbiguousLinkedSession {
                session_id: requested_session_id.to_string(),
                source_files: root_indexes
                    .iter()
                    .map(|index| rollouts[*index].source_file.clone())
                    .collect(),
            })
        }
    };

    let mut included_source_files = BTreeSet::from([rollouts[root_index].source_file.clone()]);
    let mut linkage_source_files = included_source_files.clone();
    let mut root_claim_counts = BTreeMap::<String, usize>::new();
    for claim in &metadata[root_index].parent_spawn_results {
        *root_claim_counts
            .entry(claim.child_session_id.clone())
            .or_default() += 1;
    }
    let root_child_ids = root_claim_counts.keys().cloned().collect::<BTreeSet<_>>();

    for child_session_id in &root_child_ids {
        let candidate_indexes = rollouts
            .iter()
            .enumerate()
            .filter_map(|(index, rollout)| {
                (rollout.session_id.as_deref() == Some(child_session_id.as_str())).then_some(index)
            })
            .collect::<Vec<_>>();
        let parent_claim_indexes = metadata
            .iter()
            .enumerate()
            .filter_map(|(index, linkage)| {
                linkage
                    .parent_spawn_results
                    .iter()
                    .any(|claim| claim.child_session_id == *child_session_id)
                    .then_some(index)
            })
            .collect::<Vec<_>>();
        for index in &parent_claim_indexes {
            linkage_source_files.insert(rollouts[*index].source_file.clone());
        }

        let child_origin_count = candidate_indexes
            .iter()
            .filter(|index| metadata[**index].child_origin.is_some())
            .count();
        if candidate_indexes.len() == 1 || child_origin_count > 1 {
            for index in &candidate_indexes {
                linkage_source_files.insert(rollouts[*index].source_file.clone());
            }
        }

        let is_valid_id = |value: &str| {
            !value.is_empty()
                && !value
                    .chars()
                    .any(|character| character.is_whitespace() || character.is_control())
        };
        let is_unique_parent_claim = parent_claim_indexes
            .iter()
            .flat_map(|index| &metadata[*index].parent_spawn_results)
            .filter(|claim| claim.child_session_id == *child_session_id)
            .count()
            == 1;
        let verified_candidate = candidate_indexes.as_slice().first().and_then(|index| {
            (candidate_indexes.len() == 1)
                .then_some(*index)
                .and_then(|index| {
                    metadata[index]
                        .child_origin
                        .as_ref()
                        .map(|origin| (index, origin))
                })
        });
        let is_verified = root_claim_counts.get(child_session_id) == Some(&1)
            && is_unique_parent_claim
            && is_valid_id(requested_session_id)
            && is_valid_id(child_session_id)
            && requested_session_id != child_session_id
            && verified_candidate.is_some_and(|(_, origin)| {
                origin.child_session_id == *child_session_id
                    && origin.parent_session_id == requested_session_id
                    && origin.depth == 1
            });
        if let (true, Some((index, _))) = (is_verified, verified_candidate) {
            included_source_files.insert(rollouts[index].source_file.clone());
        }
    }

    for (index, linkage) in metadata.iter().enumerate() {
        let Some(origin) = linkage.child_origin.as_ref() else {
            continue;
        };
        if origin.depth > 1
            && (root_child_ids.contains(&origin.child_session_id)
                || root_child_ids.contains(&origin.parent_session_id))
        {
            linkage_source_files.insert(rollouts[index].source_file.clone());
        }
    }

    Ok(DirectLinkedClosure {
        included_source_files,
        linkage_source_files,
    })
}

pub fn resolve_codex_home(explicit: Option<Utf8PathBuf>) -> Result<Utf8PathBuf, DiscoveryError> {
    resolve_codex_home_from(
        explicit,
        env::var_os("CODEX_HOME").map(PathBuf::from),
        env::var_os("HOME").map(PathBuf::from),
    )
}

pub fn resolve_codex_home_from(
    explicit: Option<Utf8PathBuf>,
    env_codex_home: Option<PathBuf>,
    home_dir: Option<PathBuf>,
) -> Result<Utf8PathBuf, DiscoveryError> {
    if let Some(codex_home) = explicit {
        return Ok(codex_home);
    }

    if let Some(codex_home) = env_codex_home {
        return utf8_path("CODEX_HOME", codex_home);
    }

    let home_dir = home_dir.ok_or(DiscoveryError::MissingCodexHome)?;
    let home_dir = utf8_path("HOME", home_dir)?;
    Ok(home_dir.join(".codex"))
}

pub fn discover_session_artifacts(
    options: &DiscoverOptions,
) -> Result<Vec<DiscoveredSessionArtifact>, DiscoveryError> {
    let codex_home = resolve_codex_home(options.codex_home.clone())?;
    discover_session_artifacts_in_home(&codex_home, options.session_id.as_deref())
}

pub fn discover_session_artifacts_in_home(
    codex_home: &Utf8Path,
    session_id: Option<&str>,
) -> Result<Vec<DiscoveredSessionArtifact>, DiscoveryError> {
    let sessions_root = codex_home.join("sessions");
    if !sessions_root.is_dir() {
        return Err(DiscoveryError::MissingSessionsDirectory(sessions_root));
    }

    let mut artifacts = Vec::new();
    let normalized_filter = normalize_session_filter(session_id);

    for entry in WalkDir::new(&sessions_root) {
        let entry = entry.map_err(|source| DiscoveryError::Walk {
            root: sessions_root.clone(),
            source,
        })?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = utf8_path("sessions entry", entry.path().to_path_buf())?;
        if matches_session_filter(&path, normalized_filter.as_deref()) {
            artifacts.push(DiscoveredSessionArtifact { path });
        }
    }

    artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(artifacts)
}

fn normalize_session_filter(session_id: Option<&str>) -> Option<String> {
    session_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.strip_prefix("urn:uuid:").unwrap_or(value).to_string())
}

fn matches_session_filter(path: &Utf8Path, session_id: Option<&str>) -> bool {
    match session_id {
        Some(session_id) => path.as_str().contains(session_id),
        None => true,
    }
}

fn utf8_path(source: &'static str, path: PathBuf) -> Result<Utf8PathBuf, DiscoveryError> {
    Utf8PathBuf::from_path_buf(path.clone()).map_err(|path| DiscoveryError::NonUtf8Path {
        origin: source,
        path,
    })
}
