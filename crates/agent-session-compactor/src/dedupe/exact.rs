use std::collections::HashMap;

use camino::Utf8PathBuf;

use crate::dedupe::audit::build_dedupe_group;
use crate::dedupe::DedupeResult;
use crate::normalize::{CompactionRow, SourceKind};

pub fn dedupe_rows_exact(rows: &[CompactionRow]) -> DedupeResult {
    let archival_rows = rows.to_vec();
    let mut compact_rows = Vec::new();
    let mut first_seen: HashMap<
        (
            EvidenceScope,
            crate::normalize::CompactionKind,
            String,
            Option<String>,
            Option<crate::normalize::UserMessageRole>,
        ),
        usize,
    > = HashMap::new();
    let mut duplicates_by_representative: HashMap<usize, Vec<CompactionRow>> = HashMap::new();

    for row in rows {
        let key = (
            EvidenceScope::from_row(row),
            row.kind,
            row.text_hash_hex.clone(),
            row.dedupe_identity.clone(),
            row.user_message_role,
        );
        if let Some(&representative_index) = first_seen.get(&key) {
            duplicates_by_representative
                .entry(representative_index)
                .or_default()
                .push(row.clone());
        } else {
            let representative_index = compact_rows.len();
            first_seen.insert(key, representative_index);
            compact_rows.push(row.clone());
        }
    }

    let mut dedupe_groups = Vec::new();
    for (representative_index, duplicates) in duplicates_by_representative {
        let representative = &compact_rows[representative_index];
        dedupe_groups.push(build_dedupe_group(
            representative.kind,
            representative.text_hash_hex.clone(),
            representative,
            &duplicates,
        ));
    }
    dedupe_groups.sort_by(|left, right| {
        left.representative
            .source_file
            .cmp(&right.representative.source_file)
            .then(
                left.representative
                    .event_index
                    .cmp(&right.representative.event_index),
            )
            .then(
                left.representative
                    .row_ordinal
                    .cmp(&right.representative.row_ordinal),
            )
    });

    DedupeResult {
        archival_rows,
        compact_rows,
        dedupe_groups,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum EvidenceScope {
    SessionTurn {
        source_kind: SourceKind,
        session_id: String,
        turn_id: String,
    },
    Session {
        source_kind: SourceKind,
        session_id: String,
    },
    SourceTurn {
        source_kind: SourceKind,
        source_file: Utf8PathBuf,
        turn_id: String,
    },
    SourceStream {
        source_kind: SourceKind,
        source_file: Utf8PathBuf,
    },
}

impl EvidenceScope {
    fn from_row(row: &CompactionRow) -> Self {
        match (row.session_id.as_deref(), row.turn_id.as_deref()) {
            (Some(session_id), Some(turn_id)) => Self::SessionTurn {
                source_kind: row.source_kind,
                session_id: session_id.to_string(),
                turn_id: turn_id.to_string(),
            },
            (Some(session_id), None) => Self::Session {
                source_kind: row.source_kind,
                session_id: session_id.to_string(),
            },
            (None, Some(turn_id)) => Self::SourceTurn {
                source_kind: row.source_kind,
                source_file: row.source_file.clone(),
                turn_id: turn_id.to_string(),
            },
            (None, None) => Self::SourceStream {
                source_kind: row.source_kind,
                source_file: row.source_file.clone(),
            },
        }
    }
}
