use std::fs;
use std::path::Path;

use serde::Serialize;
use walkdir::{DirEntry, WalkDir};

use crate::kernel::{sha256_bytes, DiagnosticCode, FileId, KernelError, RepoPath, Severity};
use crate::repo::blob::{BlobRecord, BlobStore};
use crate::repo::diagnostics::{RepoDiagnostic, RepoLocation};
use crate::repo::ignore::{
    CompiledIgnoreSet, LargeFilePolicy, NonUtf8PathPolicy, SnapshotOptions, SymlinkPolicy,
};
use crate::repo::inventory::{Inventory, InventoryEntry};
use crate::repo::root::{detect_repo_root, RepoRoot};
use crate::repo::{RepoError, RepoResult};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SnapshotSource {
    Worktree,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SnapshotRequest {
    pub root: RepoRoot,
    pub source: SnapshotSource,
    pub options: SnapshotOptions,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, serde::Deserialize)]
pub(crate) struct SnapshotStats {
    pub file_count: u64,
    pub total_bytes: u64,
    pub skipped_by_ignore: u64,
    pub skipped_symlinks: u64,
    pub skipped_non_utf8_paths: u64,
    pub skipped_large_files: u64,
    pub skipped_unsupported_file_kinds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepoSnapshot {
    pub source: SnapshotSource,
    pub inventory: Inventory,
    pub blob_store: BlobStore,
    pub fingerprint: crate::kernel::Fingerprint,
    pub diagnostics: Vec<RepoDiagnostic>,
    pub stats: SnapshotStats,

    root: RepoRoot,
}

impl RepoSnapshot {
    pub(crate) fn root(&self) -> &RepoRoot {
        &self.root
    }

    pub(crate) fn entry(&self, path: &RepoPath) -> Option<&InventoryEntry> {
        self.inventory.get(path)
    }

    pub(crate) fn read_bytes(&self, path: &RepoPath) -> RepoResult<&[u8]> {
        self.blob_store.read_bytes(path)
    }
}

pub(crate) fn materialize_snapshot(request: &SnapshotRequest) -> RepoResult<RepoSnapshot> {
    match request.source {
        SnapshotSource::Worktree => materialize_worktree_snapshot(request),
    }
}

pub(crate) fn detect_root_and_materialize(
    start_path: &Path,
    options: &crate::repo::RepoRootDetectionOptions,
    snapshot_options: SnapshotOptions,
) -> RepoResult<RepoSnapshot> {
    let root = detect_repo_root(start_path, options)?;
    materialize_snapshot(&SnapshotRequest {
        root,
        source: SnapshotSource::Worktree,
        options: snapshot_options,
    })
}

fn materialize_worktree_snapshot(request: &SnapshotRequest) -> RepoResult<RepoSnapshot> {
    let compiled_ignores = CompiledIgnoreSet::compile(&request.options.exclude_globs)?;
    let mut inventory_entries = Vec::new();
    let mut blob_records = Vec::new();
    let mut diagnostics = Vec::new();
    let mut stats = SnapshotStats::default();

    let mut walker = WalkDir::new(request.root.as_path())
        .follow_links(false)
        .sort_by_file_name()
        .into_iter();

    while let Some(entry) = walker.next() {
        let entry = entry.map_err(|error| RepoError::Io {
            op: "walk_repo",
            path: error
                .path()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| request.root.display()),
            reason: error.to_string(),
        })?;

        if entry.path() == request.root.as_path() {
            continue;
        }

        let path_kind = EntryPathKind::from_entry(&entry);
        let display_path = entry.path().display().to_string();
        let repo_path = match repo_path_from_entry(request.root.as_path(), entry.path()) {
            Ok(repo_path) => repo_path,
            Err(error) => match request.options.non_utf8_path_policy {
                NonUtf8PathPolicy::Error => return Err(error),
                NonUtf8PathPolicy::Skip => {
                    stats.skipped_non_utf8_paths += 1;
                    diagnostics.push(repo_diagnostic(
                        "repo.snapshot.non_utf8_path",
                        Severity::Warning,
                        "skipped non-utf8 filesystem path".to_owned(),
                        Some(RepoLocation {
                            display_path,
                            repo_path: None,
                        }),
                    ));

                    if path_kind.is_dir() {
                        walker.skip_current_dir();
                        continue;
                    }
                    continue;
                }
            },
        };

        if compiled_ignores.is_ignored(&repo_path, path_kind.is_dir()) {
            stats.skipped_by_ignore += 1;
            if path_kind.is_dir() {
                walker.skip_current_dir();
                continue;
            }
            continue;
        }

        if path_kind.is_dir() {
            continue;
        }

        if path_kind.is_symlink() {
            match request.options.symlink_policy {
                SymlinkPolicy::Skip => {
                    stats.skipped_symlinks += 1;
                    diagnostics.push(repo_diagnostic(
                        "repo.snapshot.symlink_skipped",
                        Severity::Warning,
                        "skipped symlink during phase-a snapshot materialization".to_owned(),
                        Some(RepoLocation {
                            display_path,
                            repo_path: Some(repo_path),
                        }),
                    ));
                    continue;
                }
            }
        }

        if !path_kind.is_file() {
            stats.skipped_unsupported_file_kinds += 1;
            diagnostics.push(repo_diagnostic(
                "repo.snapshot.unsupported_file_kind",
                Severity::Warning,
                "skipped unsupported filesystem entry kind".to_owned(),
                Some(RepoLocation {
                    display_path,
                    repo_path: Some(repo_path),
                }),
            ));
            continue;
        }

        let metadata = entry.metadata().map_err(|error| RepoError::Io {
            op: "read_metadata",
            path: entry.path().display().to_string(),
            reason: error.to_string(),
        })?;
        let size_bytes = metadata.len();
        if let Some(max_file_bytes) = request.options.max_file_bytes {
            if size_bytes > max_file_bytes {
                match request.options.large_file_policy {
                    LargeFilePolicy::Error => {
                        return Err(RepoError::FileTooLarge {
                            display_path: entry.path().display().to_string(),
                            size_bytes,
                            max_file_bytes,
                        });
                    }
                    LargeFilePolicy::Skip => {
                        stats.skipped_large_files += 1;
                        diagnostics.push(repo_diagnostic(
                            "repo.snapshot.file_too_large",
                            Severity::Warning,
                            format!(
                                "skipped file larger than configured max_file_bytes ({max_file_bytes})"
                            ),
                            Some(RepoLocation {
                                display_path: entry.path().display().to_string(),
                                repo_path: Some(repo_path),
                            }),
                        ));
                        continue;
                    }
                }
            }
        }

        let bytes = fs::read(entry.path()).map_err(|error| RepoError::Io {
            op: "read_file",
            path: entry.path().display().to_string(),
            reason: error.to_string(),
        })?;
        let blob_fingerprint = sha256_bytes(&bytes);
        let file_id = FileId::from_identity(&format!("repo\0file\0v1\0{}", repo_path.as_str()));

        inventory_entries.push(InventoryEntry {
            file_id: file_id.clone(),
            path: repo_path.clone(),
            blob_fingerprint: blob_fingerprint.clone(),
            size_bytes,
        });
        blob_records.push(BlobRecord::from_bytes(
            file_id,
            repo_path,
            blob_fingerprint,
            size_bytes,
            bytes,
        ));
        stats.file_count += 1;
        stats.total_bytes += size_bytes;
    }

    diagnostics.sort();
    let inventory = Inventory::from_entries(inventory_entries)?;
    let blob_store = BlobStore::from_records(blob_records);
    if blob_store.len() != inventory.len() {
        return Err(RepoError::Io {
            op: "assemble_snapshot",
            path: request.root.display(),
            reason: "inventory/blob store length mismatch".to_owned(),
        });
    }

    Ok(RepoSnapshot {
        source: request.source.clone(),
        fingerprint: inventory.fingerprint.clone(),
        inventory,
        blob_store,
        diagnostics,
        stats,
        root: request.root.clone(),
    })
}

#[derive(Clone, Copy)]
enum EntryPathKind {
    Dir,
    File,
    Symlink,
    Other,
}

impl EntryPathKind {
    fn from_entry(entry: &DirEntry) -> Self {
        let file_type = entry.file_type();
        if file_type.is_dir() {
            Self::Dir
        } else if file_type.is_file() {
            Self::File
        } else if file_type.is_symlink() {
            Self::Symlink
        } else {
            Self::Other
        }
    }

    fn is_dir(self) -> bool {
        matches!(self, Self::Dir)
    }

    fn is_file(self) -> bool {
        matches!(self, Self::File)
    }

    fn is_symlink(self) -> bool {
        matches!(self, Self::Symlink)
    }
}

fn repo_path_from_entry(root: &Path, path: &Path) -> RepoResult<RepoPath> {
    let relative = path
        .strip_prefix(root)
        .map_err(|error| RepoError::InvalidRepoPath {
            display_path: path.display().to_string(),
            reason: error.to_string(),
        })?;

    let mut segments = Vec::new();
    for component in relative.components() {
        let std::path::Component::Normal(part) = component else {
            return Err(RepoError::InvalidRepoPath {
                display_path: path.display().to_string(),
                reason: "path contained a non-normal component".to_owned(),
            });
        };
        let Some(segment) = part.to_str() else {
            return Err(RepoError::NonUtf8Path {
                display_path: path.display().to_string(),
            });
        };
        segments.push(segment);
    }

    RepoPath::parse(&segments.join("/")).map_err(|error| map_kernel_repo_path_error(path, error))
}

fn map_kernel_repo_path_error(path: &Path, error: KernelError) -> RepoError {
    match error {
        KernelError::InvalidRepoPath { reason, .. } => RepoError::InvalidRepoPath {
            display_path: path.display().to_string(),
            reason,
        },
        other => RepoError::InvalidRepoPath {
            display_path: path.display().to_string(),
            reason: other.to_string(),
        },
    }
}

fn repo_diagnostic(
    code: &'static str,
    severity: Severity,
    message: String,
    subject: Option<RepoLocation>,
) -> RepoDiagnostic {
    RepoDiagnostic {
        code: DiagnosticCode::parse(code).expect("repo diagnostic code should be valid"),
        severity,
        message,
        subject,
        related: Vec::new(),
        help: None,
    }
}
