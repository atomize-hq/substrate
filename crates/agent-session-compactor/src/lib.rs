#![forbid(unsafe_code)]

use blake3 as _;
use camino as _;
use camino::Utf8PathBuf;
use codex as _;
use serde as _;
use serde_json as _;
#[cfg(test)]
use tempfile as _;
use time as _;
use time::OffsetDateTime;

pub mod canonicalize;
pub mod cli;
pub mod dedupe;
pub mod discovery;
pub mod export;
pub mod ingest;
pub mod normalize;

pub use dedupe::{dedupe_rows_exact, DedupeResult};
pub use dedupe::{DedupeGroup, RowRef};
pub use discovery::{
    discover_session_artifacts, resolve_codex_home, DiscoverOptions, DiscoveredSessionArtifact,
    DiscoveryError,
};
pub use export::{
    export_bundle, BundleFileV0_2, BundleManifest, DedupeGroupV0_2, ExportBundleRequest,
    ExportError, ExportRowV0_2, RowRefV0_2,
};
pub use ingest::{
    ingest_rollout_artifacts, ingest_rollout_file, IngestError, IngestedRolloutFile,
    IngestedRolloutRecord, IngestedRolloutUnknown, RolloutParseFailure,
};
pub use normalize::{
    normalize_rollout_file, CompactionKind, CompactionRow, SourceKind, UserMessageRole,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunConfig {
    pub codex_home: Option<Utf8PathBuf>,
    pub session_id: Option<String>,
    pub output_dir: Utf8PathBuf,
    pub generated_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionRunResult {
    pub manifest: BundleManifest,
}

#[derive(Debug, thiserror::Error)]
pub enum CompactorError {
    #[error(transparent)]
    Discovery(#[from] DiscoveryError),
    #[error(transparent)]
    Ingest(#[from] IngestError),
    #[error(transparent)]
    Export(#[from] ExportError),
    #[error("no rollout JSONL files were found under {codex_home}")]
    NoRolloutFiles { codex_home: Utf8PathBuf },
}

pub fn run() -> anyhow::Result<()> {
    cli::run()
}

pub fn compact_codex_sessions(config: &RunConfig) -> Result<CompactionRunResult, CompactorError> {
    let codex_home = resolve_codex_home(config.codex_home.clone())?;
    let discovery_options = DiscoverOptions {
        codex_home: Some(codex_home.clone()),
        session_id: config.session_id.clone(),
    };
    let artifacts = discover_session_artifacts(&discovery_options)?;
    let ingested_rollouts = ingest_rollout_artifacts(&artifacts)?;

    if ingested_rollouts.is_empty() {
        return Err(CompactorError::NoRolloutFiles { codex_home });
    }

    let archival_rows = ingested_rollouts
        .iter()
        .flat_map(normalize_rollout_file)
        .collect::<Vec<_>>();
    let dedupe_result = dedupe_rows_exact(&archival_rows);
    let session_ids = unique_session_ids(&ingested_rollouts);
    let source_files = ingested_rollouts
        .iter()
        .map(|rollout| rollout.source_file.clone())
        .collect::<Vec<_>>();
    let manifest = export_bundle(&ExportBundleRequest {
        codex_home: &codex_home,
        output_dir: &config.output_dir,
        generated_at: config.generated_at.unwrap_or_else(OffsetDateTime::now_utc),
        session_ids,
        source_files,
        archival_rows: &dedupe_result.archival_rows,
        compact_rows: &dedupe_result.compact_rows,
        dedupe_groups: &dedupe_result.dedupe_groups,
    })?;

    Ok(CompactionRunResult { manifest })
}

fn unique_session_ids(rollouts: &[IngestedRolloutFile]) -> Vec<String> {
    let mut session_ids = rollouts
        .iter()
        .filter_map(|rollout| rollout.session_id.clone())
        .collect::<Vec<_>>();
    session_ids.sort();
    session_ids.dedup();
    session_ids
}

#[cfg(test)]
mod core_types_tests {
    use camino::Utf8PathBuf;
    use time::macros::datetime;

    use crate::{
        BundleFileV0_2, BundleManifest, CompactionKind, CompactionRow, DedupeGroup, RowRef,
        SourceKind,
    };

    #[test]
    fn core_types_preserve_manifest_and_row_contracts() {
        let row = CompactionRow {
            source_file: Utf8PathBuf::from("/tmp/session/rollout.jsonl"),
            source_kind: SourceKind::CodexRolloutJsonl,
            session_id: Some("session-123".to_string()),
            turn_id: Some("turn-456".to_string()),
            event_index: 7,
            line_number: 11,
            row_ordinal: 0,
            timestamp: Some(datetime!(2026-05-29 12:00:00 UTC)),
            kind: CompactionKind::AssistantMessage,
            user_message_role: None,
            dedupe_identity: None,
            text: "raw".to_string(),
            canonical_text: "raw".to_string(),
            text_hash_hex: "abc123".to_string(),
        };
        let row_ref = RowRef::from_row(&row);
        let dedupe_group = DedupeGroup {
            kind: row.kind,
            canonical_text_hash_hex: row.text_hash_hex.clone(),
            representative: row_ref.clone(),
            duplicates: vec![RowRef {
                source_file: Utf8PathBuf::from("/tmp/session/rollout-2.jsonl"),
                line_number: 12,
                event_index: 8,
                row_ordinal: 0,
            }],
        };
        let manifest = BundleManifest {
            schema_version: "v0.2".to_string(),
            generated_at: datetime!(2026-05-29 12:00:00 UTC),
            codex_home: Utf8PathBuf::from("/tmp/.codex"),
            output_dir: Utf8PathBuf::from("/tmp/output"),
            discovered_file_count: 2,
            archival_row_count: 10,
            compact_row_count: 8,
            dedupe_group_count: 1,
            session_ids: vec!["session-123".to_string()],
            files: vec![
                BundleFileV0_2 {
                    id: 0,
                    path: Utf8PathBuf::from("/tmp/session/rollout-2.jsonl"),
                },
                BundleFileV0_2 {
                    id: 1,
                    path: Utf8PathBuf::from("/tmp/session/rollout.jsonl"),
                },
            ],
        };

        assert_eq!(row_ref.line_number, row.line_number);
        assert_eq!(dedupe_group.duplicates.len(), 1);
        assert_eq!(manifest.schema_version, "v0.2");
    }
}
