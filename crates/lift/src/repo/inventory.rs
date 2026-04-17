use std::collections::BTreeMap;

use serde::Serialize;

use crate::kernel::{sha256_canonical_json, FileId, Fingerprint, RepoPath};
use crate::repo::{RepoError, RepoResult};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, serde::Deserialize)]
pub(crate) struct InventoryEntry {
    pub file_id: FileId,
    pub path: RepoPath,
    pub blob_fingerprint: Fingerprint,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Inventory {
    pub entries: BTreeMap<RepoPath, InventoryEntry>,
    pub fingerprint: Fingerprint,
}

impl Inventory {
    pub(crate) fn from_entries(entries: Vec<InventoryEntry>) -> RepoResult<Self> {
        let entries = entries
            .into_iter()
            .map(|entry| (entry.path.clone(), entry))
            .collect::<BTreeMap<_, _>>();
        let fingerprint = fingerprint_entries(entries.values())?;
        Ok(Self {
            entries,
            fingerprint,
        })
    }

    pub(crate) fn get(&self, path: &RepoPath) -> Option<&InventoryEntry> {
        self.entries.get(path)
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &InventoryEntry> {
        self.entries.values()
    }
}

pub(crate) fn fingerprint_entries<'a>(
    entries: impl IntoIterator<Item = &'a InventoryEntry>,
) -> RepoResult<Fingerprint> {
    #[derive(Serialize)]
    struct FileRecord<'a> {
        path: &'a RepoPath,
        blob_fingerprint: &'a Fingerprint,
        size_bytes: u64,
    }

    #[derive(Serialize)]
    struct FingerprintDocument<'a> {
        version: u32,
        files: Vec<FileRecord<'a>>,
    }

    let files = entries
        .into_iter()
        .map(|entry| FileRecord {
            path: &entry.path,
            blob_fingerprint: &entry.blob_fingerprint,
            size_bytes: entry.size_bytes,
        })
        .collect::<Vec<_>>();

    sha256_canonical_json(&FingerprintDocument { version: 1, files }).map_err(|error| {
        RepoError::Io {
            op: "fingerprint_inventory",
            path: "<inventory>".to_owned(),
            reason: error.to_string(),
        }
    })
}
