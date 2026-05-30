mod audit;
mod exact;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::normalize::{CompactionKind, CompactionRow};

pub use exact::dedupe_rows_exact;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RowRef {
    pub source_file: Utf8PathBuf,
    pub line_number: usize,
    pub event_index: usize,
    pub row_ordinal: usize,
}

impl RowRef {
    pub fn from_row(row: &CompactionRow) -> Self {
        Self {
            source_file: row.source_file.clone(),
            line_number: row.line_number,
            event_index: row.event_index,
            row_ordinal: row.row_ordinal,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DedupeGroup {
    pub kind: CompactionKind,
    pub canonical_text_hash_hex: String,
    pub representative: RowRef,
    pub duplicates: Vec<RowRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DedupeResult {
    pub archival_rows: Vec<CompactionRow>,
    pub compact_rows: Vec<CompactionRow>,
    pub dedupe_groups: Vec<DedupeGroup>,
}
