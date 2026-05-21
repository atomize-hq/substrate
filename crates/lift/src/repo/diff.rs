use std::cmp::Ordering;

use serde::Serialize;

use crate::kernel::{sha256_canonical_json, Fingerprint, RepoPath};
use crate::repo::{InventoryEntry, RepoSnapshot};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DiffKind {
    Added,
    Modified,
    Removed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DiffEntry {
    pub path: RepoPath,
    pub kind: DiffKind,
    pub before: Option<InventoryEntry>,
    pub after: Option<InventoryEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepoDiff {
    pub base_fingerprint: Fingerprint,
    pub head_fingerprint: Fingerprint,
    pub entries: Vec<DiffEntry>,
    pub fingerprint: Fingerprint,
}

pub(crate) fn build_diff(base: &RepoSnapshot, head: &RepoSnapshot) -> RepoDiff {
    let mut base_entries = base.inventory.iter().peekable();
    let mut head_entries = head.inventory.iter().peekable();
    let mut entries = Vec::new();

    loop {
        match (base_entries.peek(), head_entries.peek()) {
            (Some(base_entry), Some(head_entry)) => match base_entry.path.cmp(&head_entry.path) {
                Ordering::Less => {
                    let removed = base_entries
                        .next()
                        .expect("peeked base inventory entry should exist");
                    entries.push(DiffEntry {
                        path: removed.path.clone(),
                        kind: DiffKind::Removed,
                        before: Some(removed.clone()),
                        after: None,
                    });
                }
                Ordering::Equal => {
                    let before = base_entries
                        .next()
                        .expect("peeked base inventory entry should exist");
                    let after = head_entries
                        .next()
                        .expect("peeked head inventory entry should exist");

                    if before.blob_fingerprint != after.blob_fingerprint {
                        entries.push(DiffEntry {
                            path: before.path.clone(),
                            kind: DiffKind::Modified,
                            before: Some(before.clone()),
                            after: Some(after.clone()),
                        });
                    }
                }
                Ordering::Greater => {
                    let added = head_entries
                        .next()
                        .expect("peeked head inventory entry should exist");
                    entries.push(DiffEntry {
                        path: added.path.clone(),
                        kind: DiffKind::Added,
                        before: None,
                        after: Some(added.clone()),
                    });
                }
            },
            (Some(_), None) => {
                let removed = base_entries
                    .next()
                    .expect("peeked base inventory entry should exist");
                entries.push(DiffEntry {
                    path: removed.path.clone(),
                    kind: DiffKind::Removed,
                    before: Some(removed.clone()),
                    after: None,
                });
            }
            (None, Some(_)) => {
                let added = head_entries
                    .next()
                    .expect("peeked head inventory entry should exist");
                entries.push(DiffEntry {
                    path: added.path.clone(),
                    kind: DiffKind::Added,
                    before: None,
                    after: Some(added.clone()),
                });
            }
            (None, None) => break,
        }
    }

    let base_fingerprint = base.fingerprint.clone();
    let head_fingerprint = head.fingerprint.clone();
    let fingerprint = fingerprint_diff(&base_fingerprint, &head_fingerprint, &entries);

    RepoDiff {
        base_fingerprint,
        head_fingerprint,
        entries,
        fingerprint,
    }
}

fn fingerprint_diff(
    base_fingerprint: &Fingerprint,
    head_fingerprint: &Fingerprint,
    entries: &[DiffEntry],
) -> Fingerprint {
    #[derive(Serialize)]
    struct DiffEntryRecord<'a> {
        path: &'a RepoPath,
        kind: DiffKind,
        before_blob_fingerprint: Option<&'a Fingerprint>,
        after_blob_fingerprint: Option<&'a Fingerprint>,
    }

    #[derive(Serialize)]
    struct DiffFingerprintDocument<'a> {
        version: u32,
        base_fingerprint: &'a Fingerprint,
        head_fingerprint: &'a Fingerprint,
        entries: Vec<DiffEntryRecord<'a>>,
    }

    let entries = entries
        .iter()
        .map(|entry| DiffEntryRecord {
            path: &entry.path,
            kind: entry.kind,
            before_blob_fingerprint: entry.before.as_ref().map(|before| &before.blob_fingerprint),
            after_blob_fingerprint: entry.after.as_ref().map(|after| &after.blob_fingerprint),
        })
        .collect::<Vec<_>>();

    sha256_canonical_json(&DiffFingerprintDocument {
        version: 1,
        base_fingerprint,
        head_fingerprint,
        entries,
    })
    .expect("repo diff fingerprint serialization should succeed")
}
