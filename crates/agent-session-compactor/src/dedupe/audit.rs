use crate::dedupe::{DedupeGroup, RowRef};
use crate::normalize::{CompactionKind, CompactionRow};

pub(super) fn build_dedupe_group(
    kind: CompactionKind,
    canonical_text_hash_hex: String,
    representative: &CompactionRow,
    duplicates: &[CompactionRow],
) -> DedupeGroup {
    DedupeGroup {
        kind,
        canonical_text_hash_hex,
        representative: RowRef::from_row(representative),
        duplicates: duplicates.iter().map(RowRef::from_row).collect(),
    }
}
