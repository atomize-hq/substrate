use std::collections::BTreeMap;
use std::sync::Arc;

use crate::kernel::{FileId, Fingerprint, RepoPath};
use crate::repo::{RepoError, RepoResult};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BlobRecord {
    pub file_id: FileId,
    pub path: RepoPath,
    pub blob_fingerprint: Fingerprint,
    pub size_bytes: u64,
    bytes: Arc<[u8]>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct BlobStore {
    by_path: BTreeMap<RepoPath, BlobRecord>,
}

impl BlobRecord {
    pub(crate) fn from_bytes(
        file_id: FileId,
        path: RepoPath,
        blob_fingerprint: Fingerprint,
        size_bytes: u64,
        bytes: Vec<u8>,
    ) -> Self {
        Self {
            file_id,
            path,
            blob_fingerprint,
            size_bytes,
            bytes: Arc::from(bytes),
        }
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

impl BlobStore {
    pub(crate) fn from_records(records: Vec<BlobRecord>) -> Self {
        let by_path = records
            .into_iter()
            .map(|record| (record.path.clone(), record))
            .collect();
        Self { by_path }
    }

    pub(crate) fn contains(&self, path: &RepoPath) -> bool {
        self.by_path.contains_key(path)
    }

    pub(crate) fn get(&self, path: &RepoPath) -> Option<&BlobRecord> {
        self.by_path.get(path)
    }

    pub(crate) fn read_bytes(&self, path: &RepoPath) -> RepoResult<&[u8]> {
        self.by_path
            .get(path)
            .map(BlobRecord::bytes)
            .ok_or_else(|| RepoError::MissingBlob { path: path.clone() })
    }

    pub(crate) fn len(&self) -> usize {
        self.by_path.len()
    }
}
