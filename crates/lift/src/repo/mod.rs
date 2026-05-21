//! Internal repo substrate seam.

#![allow(dead_code)]

pub(crate) mod blob;
pub(crate) mod diagnostics;
pub(crate) mod diff;
pub(crate) mod error;
pub(crate) mod ignore;
pub(crate) mod inventory;
pub(crate) mod root;
pub(crate) mod schema;
pub(crate) mod snapshot;

#[allow(unused_imports)]
pub(crate) use blob::{BlobRecord, BlobStore};
#[allow(unused_imports)]
pub(crate) use diagnostics::{RepoDiagnostic, RepoLocation, RepoRelatedLocation};
#[allow(unused_imports)]
pub(crate) use diff::{build_diff, DiffEntry, DiffKind, RepoDiff};
#[allow(unused_imports)]
pub(crate) use error::{RepoError, RepoResult};
#[allow(unused_imports)]
pub(crate) use ignore::{
    CompiledIgnoreSet, LargeFilePolicy, NonUtf8PathPolicy, SnapshotOptions, SymlinkPolicy,
    WellKnownExclude,
};
#[allow(unused_imports)]
pub(crate) use inventory::{Inventory, InventoryEntry};
#[allow(unused_imports)]
pub(crate) use root::{RepoRoot, RepoRootDetectionOptions, RootMarker};
#[allow(unused_imports)]
pub(crate) use snapshot::{
    materialize_snapshot, RepoSnapshot, SnapshotRequest, SnapshotSource, SnapshotStats,
};
