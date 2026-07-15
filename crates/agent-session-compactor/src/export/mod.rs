mod files;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::dedupe::DedupeGroup;
use crate::ingest::RolloutLinkageMetadata;
use crate::normalize::{CompactionKind, CompactionRow, SourceKind, UserMessageRole};

pub use files::export_bundle;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BundleFileV0_2 {
    pub id: u32,
    pub path: Utf8PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub turns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BundleManifest {
    pub schema_version: String,
    #[serde(with = "time::serde::rfc3339")]
    pub generated_at: OffsetDateTime,
    pub codex_home: Utf8PathBuf,
    pub output_dir: Utf8PathBuf,
    pub discovered_file_count: usize,
    pub archival_row_count: usize,
    pub compact_row_count: usize,
    pub dedupe_group_count: usize,
    pub session_ids: Vec<String>,
    pub files: Vec<BundleFileV0_2>,
    /// Direct delegation observations validated from both parent and child rollout metadata.
    ///
    /// This field is additive to bundle schema v0.2. Manifests written before
    /// delegation linkage was added deserialize with an empty collection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delegation_links: Vec<DelegationLink>,
}

/// Validation outcome for one direct parent/child delegation observation.
///
/// Only [`Verified`](Self::Verified) is semantic linkage. Every other variant
/// preserves incomplete, contradictory, invalid, or unsupported metadata as a
/// typed non-semantic observation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum DelegationLinkState {
    /// Parent and child metadata agree exactly and the child declares depth one.
    Verified,
    /// A parent spawn result has no child-origin artifact.
    ParentOnly,
    /// A child origin has no matching parent spawn result.
    ChildOnly,
    /// Parent-side and child-side metadata disagree about the parent session.
    ConflictingParent,
    /// A session attempts to link to itself.
    SelfLink,
    /// More than one observation exists for the same side of the link.
    Duplicate,
    /// A parent or child session identifier is empty or contains whitespace/control characters.
    MalformedSessionId,
    /// The child declares a non-direct depth that is not a deeper descendant.
    DepthMismatch,
    /// The child declares a depth greater than one and remains unsupported residue.
    DeeperResidue,
}

/// Stable source-row reference proving one side of a delegation observation.
///
/// Evidence references intentionally contain only source location data. They
/// never copy ordinary message text or raw tool output into the manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct DelegationEvidenceRef {
    /// Rollout JSONL file containing the evidence record.
    pub source_file: Utf8PathBuf,
    /// One-based physical line number within `source_file`.
    pub line_number: usize,
    /// Zero-based ingested event ordinal for the record.
    pub event_index: usize,
}

/// Analyzer-facing direct delegation link carried by a v0.2 bundle manifest.
///
/// `parent_session_id` is the parent observed in a `spawn_agent` result, or the
/// child-declared parent for a child-only observation.
/// `child_origin_parent_session_id` and `depth` preserve the child-side claim
/// when a child artifact exists. Consumers must treat a link as semantic only
/// when `state` is [`DelegationLinkState::Verified`].
///
/// # Examples
///
/// ```
/// use agent_session_compactor::{
///     DelegationEvidenceRef, DelegationLink, DelegationLinkState,
/// };
/// use camino::Utf8PathBuf;
///
/// let link = DelegationLink {
///     parent_session_id: "session-parent".to_string(),
///     child_session_id: "session-child".to_string(),
///     child_origin_parent_session_id: Some("session-parent".to_string()),
///     depth: Some(1),
///     state: DelegationLinkState::Verified,
///     parent_evidence: vec![DelegationEvidenceRef {
///         source_file: Utf8PathBuf::from("rollout-parent.jsonl"),
///         line_number: 2,
///         event_index: 1,
///     }],
///     child_evidence: vec![DelegationEvidenceRef {
///         source_file: Utf8PathBuf::from("rollout-child.jsonl"),
///         line_number: 1,
///         event_index: 0,
///     }],
/// };
///
/// assert_eq!(link.state, DelegationLinkState::Verified);
/// assert_eq!(link.parent_evidence[0].line_number, 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DelegationLink {
    /// Parent session observed by the parent side, or declared by a child-only artifact.
    pub parent_session_id: String,
    /// Child session observed by either side of the delegation.
    pub child_session_id: String,
    /// Parent session declared by child-origin metadata, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_origin_parent_session_id: Option<String>,
    /// Delegation depth declared by child-origin metadata, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,
    /// Reciprocal validation outcome. Only `verified` is semantic.
    pub state: DelegationLinkState,
    /// Deterministically ordered, deduplicated parent call/result evidence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parent_evidence: Vec<DelegationEvidenceRef>,
    /// Deterministically ordered, deduplicated child-origin evidence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub child_evidence: Vec<DelegationEvidenceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportRowV0_2 {
    pub source_file_id: u32,
    pub source_kind: SourceKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_id_ref: Option<u16>,
    pub event_index: usize,
    pub row_ordinal: usize,
    #[serde(with = "time::serde::rfc3339::option")]
    pub timestamp: Option<OffsetDateTime>,
    pub kind: CompactionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_message_role: Option<UserMessageRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedupe_identity: Option<String>,
    pub text: String,
    pub text_hash_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RowRefV0_2 {
    pub source_file_id: u32,
    pub event_index: usize,
    pub row_ordinal: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DedupeGroupV0_2 {
    pub kind: CompactionKind,
    pub canonical_text_hash_hex: String,
    pub representative: RowRefV0_2,
    pub duplicates: Vec<RowRefV0_2>,
}

/// Bundle publication contract for the five-file analyzer-facing export:
///
/// - row files, dedupe audit, and summary are written into a hidden sibling
///   staging directory under the requested output parent
/// - `manifest.json` is written after the other four contract files
/// - the final output directory is published only after the staging bundle is
///   complete
/// - failed or interrupted runs may leave only clearly marked staging
///   directories; the final output path is either absent or still points to the
///   last complete bundle
#[derive(Debug, Clone)]
pub struct ExportBundleRequest<'a> {
    pub codex_home: &'a camino::Utf8Path,
    pub output_dir: &'a camino::Utf8Path,
    pub generated_at: OffsetDateTime,
    pub session_ids: Vec<String>,
    pub source_files: Vec<Utf8PathBuf>,
    /// Structured parent- and child-side linkage metadata for included rollouts.
    pub linkage_metadata: &'a [RolloutLinkageMetadata],
    pub archival_rows: &'a [CompactionRow],
    pub compact_rows: &'a [CompactionRow],
    pub dedupe_groups: &'a [DedupeGroup],
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("failed to create output directory {path}: {source}")]
    CreateOutputDirectory {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("output directory {path} must include a final path segment")]
    InvalidOutputDirectory { path: Utf8PathBuf },
    #[error("failed to write bundle file {path}: {source}")]
    WriteFile {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to serialize bundle file {path}: {source}")]
    Serialize {
        path: Utf8PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to publish completed bundle to {path}: {source}")]
    PublishOutputDirectory {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("file registry overflowed the v0.2 u32 source_file_id space")]
    SourceFileIdOverflow,
    #[error("row provenance path was not registered in the manifest file table: {path}")]
    UnregisteredSourceFile { path: Utf8PathBuf },
    #[error("row turn_id was not registered in the manifest turn table for {path}: {turn_id}")]
    UnregisteredTurnId { path: Utf8PathBuf, turn_id: String },
    #[error("source file {path} carried conflicting session ids: {left:?} vs {right:?}")]
    ConflictingSourceFileSessionIds {
        path: Utf8PathBuf,
        left: Option<String>,
        right: Option<String>,
    },
    #[error("source file {path} overflowed the v0.2 u16 turn_id_ref space")]
    SourceFileTurnIdOverflow { path: Utf8PathBuf },
    #[error("export interrupted at {point}")]
    InjectedFailure { point: String },
}
