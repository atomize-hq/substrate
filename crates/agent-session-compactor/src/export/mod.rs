mod files;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::dedupe::DedupeGroup;
use crate::normalize::CompactionRow;

pub use files::export_bundle;

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
    pub source_files: Vec<Utf8PathBuf>,
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
    #[error("export interrupted at {point}")]
    InjectedFailure { point: String },
}
