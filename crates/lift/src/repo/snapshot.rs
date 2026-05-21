use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use gix::bstr::ByteSlice;
use gix::object::tree::EntryKind;
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum SnapshotSource {
    Worktree,
    GitRev { rev: String },
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
    let compiled_ignores = CompiledIgnoreSet::compile(&request.options)?;
    let mut assembly = SnapshotAssembly::new(request);
    match &request.source {
        SnapshotSource::Worktree => {
            materialize_worktree_snapshot(request, &compiled_ignores, &mut assembly)?
        }
        SnapshotSource::GitRev { rev } => {
            materialize_gitrev_snapshot(request, rev, &compiled_ignores, &mut assembly)?
        }
    }
    assembly.finalize()
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

fn materialize_worktree_snapshot(
    request: &SnapshotRequest,
    compiled_ignores: &CompiledIgnoreSet,
    assembly: &mut SnapshotAssembly,
) -> RepoResult<()> {
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
                    assembly.record_non_utf8_skip(display_path);
                    if path_kind.is_dir() {
                        walker.skip_current_dir();
                    }
                    continue;
                }
            },
        };

        if compiled_ignores.is_ignored(&repo_path, path_kind.is_dir()) {
            assembly.record_ignore_skip();
            if path_kind.is_dir() {
                walker.skip_current_dir();
            }
            continue;
        }

        if path_kind.is_dir() {
            continue;
        }

        if path_kind.is_symlink() {
            match request.options.symlink_policy {
                SymlinkPolicy::Skip => {
                    assembly.record_symlink_skip(display_path, repo_path);
                    continue;
                }
                SymlinkPolicy::Follow => {
                    let resolved =
                        resolve_worktree_symlink(request.root.as_path(), entry.path(), &repo_path)?;
                    if followed_target_is_ignored(compiled_ignores, &resolved.repo_path) {
                        assembly.record_ignore_skip();
                        continue;
                    }
                    if assembly.enforce_file_size_policy(
                        request,
                        &resolved.display_path,
                        &repo_path,
                        resolved.size_bytes,
                    )? {
                        continue;
                    }
                    let bytes = fs::read(&resolved.fs_path).map_err(|error| RepoError::Io {
                        op: "read_file",
                        path: resolved.fs_path.display().to_string(),
                        reason: error.to_string(),
                    })?;
                    assembly.push_file(repo_path, bytes);
                    continue;
                }
            }
        }

        if !path_kind.is_file() {
            assembly.record_unsupported_kind(display_path, repo_path);
            continue;
        }

        let metadata = entry.metadata().map_err(|error| RepoError::Io {
            op: "read_metadata",
            path: entry.path().display().to_string(),
            reason: error.to_string(),
        })?;
        let size_bytes = metadata.len();
        if assembly.enforce_file_size_policy(request, &display_path, &repo_path, size_bytes)? {
            continue;
        }

        let bytes = fs::read(entry.path()).map_err(|error| RepoError::Io {
            op: "read_file",
            path: entry.path().display().to_string(),
            reason: error.to_string(),
        })?;
        assembly.push_file(repo_path, bytes);
    }

    Ok(())
}

fn materialize_gitrev_snapshot(
    request: &SnapshotRequest,
    rev: &str,
    compiled_ignores: &CompiledIgnoreSet,
    assembly: &mut SnapshotAssembly,
) -> RepoResult<()> {
    let repo = gix::open(request.root.as_path()).map_err(|error| RepoError::GitOpen {
        path: request.root.display(),
        reason: error.to_string(),
    })?;
    let object = repo
        .rev_parse_single(rev.as_bytes().as_bstr())
        .map_err(|error| RepoError::GitRevisionResolve {
            rev: rev.to_owned(),
            repo_root: request.root.display(),
            reason: error.to_string(),
        })?
        .object()
        .map_err(|error| RepoError::GitRevisionResolve {
            rev: rev.to_owned(),
            repo_root: request.root.display(),
            reason: error.to_string(),
        })?;
    let actual_kind = object.kind.to_string();
    let root_tree = object
        .peel_to_tree()
        .map_err(|_| RepoError::GitRevisionObjectKind {
            rev: rev.to_owned(),
            actual_kind,
            expected_kind: "tree",
        })?;

    walk_git_tree(
        request,
        &root_tree,
        &root_tree,
        None,
        rev,
        compiled_ignores,
        assembly,
    )
}

fn walk_git_tree(
    request: &SnapshotRequest,
    root_tree: &gix::Tree<'_>,
    tree: &gix::Tree<'_>,
    parent_repo_path: Option<&RepoPath>,
    rev: &str,
    compiled_ignores: &CompiledIgnoreSet,
    assembly: &mut SnapshotAssembly,
) -> RepoResult<()> {
    for entry in tree.iter() {
        let entry = entry.map_err(|error| RepoError::GitObjectLookup {
            object_id: tree.id.to_string(),
            reason: error.to_string(),
        })?;
        let repo_path = match repo_path_from_git_entry(parent_repo_path, entry.inner.filename) {
            Ok(repo_path) => repo_path,
            Err(error) => match request.options.non_utf8_path_policy {
                NonUtf8PathPolicy::Error => return Err(error),
                NonUtf8PathPolicy::Skip => {
                    assembly.record_non_utf8_skip(format!("{rev}:{}", entry.inner.filename));
                    continue;
                }
            },
        };
        let entry_kind = entry.inner.mode.kind();

        if compiled_ignores.is_ignored(&repo_path, entry_kind == EntryKind::Tree) {
            assembly.record_ignore_skip();
            continue;
        }

        match entry_kind {
            EntryKind::Tree => {
                let object =
                    tree.repo
                        .find_object(entry.inner.oid.to_owned())
                        .map_err(|error| RepoError::GitObjectLookup {
                            object_id: entry.inner.oid.to_string(),
                            reason: error.to_string(),
                        })?;
                let child_tree = object.into_tree();
                walk_git_tree(
                    request,
                    root_tree,
                    &child_tree,
                    Some(&repo_path),
                    rev,
                    compiled_ignores,
                    assembly,
                )?;
            }
            EntryKind::Blob | EntryKind::BlobExecutable => {
                let object =
                    tree.repo
                        .find_object(entry.inner.oid.to_owned())
                        .map_err(|error| RepoError::GitObjectLookup {
                            object_id: entry.inner.oid.to_string(),
                            reason: error.to_string(),
                        })?;
                let bytes = object.data.to_vec();
                let display_path = format!("{rev}:{}", repo_path.as_str());
                if assembly.enforce_file_size_policy(
                    request,
                    &display_path,
                    &repo_path,
                    bytes.len() as u64,
                )? {
                    continue;
                }
                assembly.push_file(repo_path, bytes);
            }
            EntryKind::Link => match request.options.symlink_policy {
                SymlinkPolicy::Skip => {
                    assembly
                        .record_symlink_skip(format!("{rev}:{}", repo_path.as_str()), repo_path);
                }
                SymlinkPolicy::Follow => {
                    let object =
                        tree.repo
                            .find_object(entry.inner.oid.to_owned())
                            .map_err(|error| RepoError::GitObjectLookup {
                                object_id: entry.inner.oid.to_string(),
                                reason: error.to_string(),
                            })?;
                    let target = std::str::from_utf8(&object.data).map_err(|_| {
                        RepoError::GitSymlinkTargetInvalidUtf8 {
                            rev: rev.to_owned(),
                            path: repo_path.clone(),
                        }
                    })?;
                    let resolved = resolve_git_symlink(
                        root_tree,
                        rev,
                        &repo_path,
                        target.trim_end_matches('\0'),
                    )?;
                    if followed_target_is_ignored(compiled_ignores, &resolved.repo_path) {
                        assembly.record_ignore_skip();
                        continue;
                    }
                    let object = tree.repo.find_object(resolved.object_id).map_err(|error| {
                        RepoError::GitObjectLookup {
                            object_id: resolved.object_id.to_string(),
                            reason: error.to_string(),
                        }
                    })?;
                    let bytes = object.data.to_vec();
                    if assembly.enforce_file_size_policy(
                        request,
                        &resolved.display_path,
                        &repo_path,
                        bytes.len() as u64,
                    )? {
                        continue;
                    }
                    assembly.push_file(repo_path, bytes);
                }
            },
            _ => {
                assembly
                    .record_unsupported_kind(format!("{rev}:{}", repo_path.as_str()), repo_path);
            }
        }
    }

    Ok(())
}

struct SnapshotAssembly {
    request: SnapshotRequest,
    inventory_entries: Vec<InventoryEntry>,
    blob_records: Vec<BlobRecord>,
    diagnostics: Vec<RepoDiagnostic>,
    stats: SnapshotStats,
}

impl SnapshotAssembly {
    fn new(request: &SnapshotRequest) -> Self {
        Self {
            request: request.clone(),
            inventory_entries: Vec::new(),
            blob_records: Vec::new(),
            diagnostics: Vec::new(),
            stats: SnapshotStats::default(),
        }
    }

    fn record_ignore_skip(&mut self) {
        self.stats.skipped_by_ignore += 1;
    }

    fn record_symlink_skip(&mut self, display_path: String, repo_path: RepoPath) {
        self.stats.skipped_symlinks += 1;
        self.diagnostics.push(repo_diagnostic(
            "repo.snapshot.symlink_skipped",
            Severity::Warning,
            "skipped symlink during repo snapshot materialization".to_owned(),
            Some(RepoLocation {
                display_path,
                repo_path: Some(repo_path),
            }),
        ));
    }

    fn record_non_utf8_skip(&mut self, display_path: String) {
        self.stats.skipped_non_utf8_paths += 1;
        self.diagnostics.push(repo_diagnostic(
            "repo.snapshot.non_utf8_path",
            Severity::Warning,
            "skipped non-utf8 filesystem path".to_owned(),
            Some(RepoLocation {
                display_path,
                repo_path: None,
            }),
        ));
    }

    fn record_unsupported_kind(&mut self, display_path: String, repo_path: RepoPath) {
        self.stats.skipped_unsupported_file_kinds += 1;
        self.diagnostics.push(repo_diagnostic(
            "repo.snapshot.unsupported_file_kind",
            Severity::Warning,
            "skipped unsupported filesystem entry kind".to_owned(),
            Some(RepoLocation {
                display_path,
                repo_path: Some(repo_path),
            }),
        ));
    }

    fn enforce_file_size_policy(
        &mut self,
        request: &SnapshotRequest,
        display_path: &str,
        repo_path: &RepoPath,
        size_bytes: u64,
    ) -> RepoResult<bool> {
        let Some(max_file_bytes) = request.options.max_file_bytes else {
            return Ok(false);
        };
        if size_bytes <= max_file_bytes {
            return Ok(false);
        }

        match request.options.large_file_policy {
            LargeFilePolicy::Error => Err(RepoError::FileTooLarge {
                display_path: display_path.to_owned(),
                size_bytes,
                max_file_bytes,
            }),
            LargeFilePolicy::Skip => {
                self.stats.skipped_large_files += 1;
                self.diagnostics.push(repo_diagnostic(
                    "repo.snapshot.file_too_large",
                    Severity::Warning,
                    format!(
                        "skipped file larger than configured max_file_bytes ({max_file_bytes})"
                    ),
                    Some(RepoLocation {
                        display_path: display_path.to_owned(),
                        repo_path: Some(repo_path.clone()),
                    }),
                ));
                Ok(true)
            }
        }
    }

    fn push_file(&mut self, repo_path: RepoPath, bytes: Vec<u8>) {
        let size_bytes = bytes.len() as u64;
        let blob_fingerprint = sha256_bytes(&bytes);
        let file_id = FileId::from_identity(&format!("repo\0file\0v1\0{}", repo_path.as_str()));

        self.inventory_entries.push(InventoryEntry {
            file_id: file_id.clone(),
            path: repo_path.clone(),
            blob_fingerprint: blob_fingerprint.clone(),
            size_bytes,
        });
        self.blob_records.push(BlobRecord::from_bytes(
            file_id,
            repo_path,
            blob_fingerprint,
            size_bytes,
            bytes,
        ));
        self.stats.file_count += 1;
        self.stats.total_bytes += size_bytes;
    }

    fn finalize(mut self) -> RepoResult<RepoSnapshot> {
        self.diagnostics.sort();
        let inventory = Inventory::from_entries(self.inventory_entries)?;
        let blob_store = BlobStore::from_records(self.blob_records);
        if blob_store.len() != inventory.len() {
            return Err(RepoError::Io {
                op: "assemble_snapshot",
                path: self.request.root.display(),
                reason: "inventory/blob store length mismatch".to_owned(),
            });
        }

        Ok(RepoSnapshot {
            source: self.request.source.clone(),
            fingerprint: inventory.fingerprint.clone(),
            inventory,
            blob_store,
            diagnostics: self.diagnostics,
            stats: self.stats,
            root: self.request.root,
        })
    }
}

struct ResolvedWorktreeTarget {
    fs_path: PathBuf,
    repo_path: RepoPath,
    display_path: String,
    size_bytes: u64,
}

struct ResolvedGitTarget {
    object_id: gix::ObjectId,
    repo_path: RepoPath,
    display_path: String,
}

fn resolve_worktree_symlink(
    root: &Path,
    observed_path: &Path,
    observed_repo_path: &RepoPath,
) -> RepoResult<ResolvedWorktreeTarget> {
    let mut current = observed_path.to_path_buf();
    let mut visited = BTreeSet::from([observed_repo_path.as_str().to_owned()]);

    loop {
        let metadata = fs::symlink_metadata(&current).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                RepoError::SymlinkTargetDangling {
                    path: observed_repo_path.clone(),
                    target: current.display().to_string(),
                }
            } else {
                RepoError::Io {
                    op: "symlink_metadata",
                    path: current.display().to_string(),
                    reason: error.to_string(),
                }
            }
        })?;

        if metadata.file_type().is_symlink() {
            let raw_target = fs::read_link(&current).map_err(|error| RepoError::Io {
                op: "read_link",
                path: current.display().to_string(),
                reason: error.to_string(),
            })?;
            let next = normalize_follow_target(root, current.parent().unwrap_or(root), &raw_target)
                .ok_or_else(|| RepoError::SymlinkTargetEscape {
                    path: observed_repo_path.clone(),
                    target: raw_target.display().to_string(),
                })?;
            let next_repo_path = repo_path_from_entry(root, &next)?;
            if !visited.insert(next_repo_path.as_str().to_owned()) {
                return Err(RepoError::SymlinkTargetLoop {
                    path: observed_repo_path.clone(),
                    target: raw_target.display().to_string(),
                });
            }
            current = next;
            continue;
        }

        if metadata.is_dir() {
            return Err(RepoError::SymlinkTargetDirectory {
                path: observed_repo_path.clone(),
                target: current.display().to_string(),
            });
        }
        if !metadata.is_file() {
            return Err(RepoError::SymlinkTargetDangling {
                path: observed_repo_path.clone(),
                target: current.display().to_string(),
            });
        }

        let repo_path = repo_path_from_entry(root, &current)?;
        return Ok(ResolvedWorktreeTarget {
            fs_path: current.clone(),
            repo_path,
            display_path: current.display().to_string(),
            size_bytes: metadata.len(),
        });
    }
}

fn resolve_git_symlink(
    root_tree: &gix::Tree<'_>,
    rev: &str,
    observed_repo_path: &RepoPath,
    initial_target: &str,
) -> RepoResult<ResolvedGitTarget> {
    let mut current_target = initial_target.to_owned();
    let mut current_repo_path =
        normalize_repo_symlink_target(observed_repo_path.parent(), &current_target).map_err(
            |reason| RepoError::SymlinkTargetEscape {
                path: observed_repo_path.clone(),
                target: reason,
            },
        )?;
    let mut visited = BTreeSet::from([observed_repo_path.as_str().to_owned()]);

    loop {
        if !visited.insert(current_repo_path.as_str().to_owned()) {
            return Err(RepoError::SymlinkTargetLoop {
                path: observed_repo_path.clone(),
                target: current_target.clone(),
            });
        }

        let entry = lookup_git_entry(root_tree, &current_repo_path)?.ok_or_else(|| {
            RepoError::SymlinkTargetDangling {
                path: observed_repo_path.clone(),
                target: current_target.clone(),
            }
        })?;

        match entry.mode().kind() {
            EntryKind::Tree => {
                return Err(RepoError::SymlinkTargetDirectory {
                    path: observed_repo_path.clone(),
                    target: current_target.clone(),
                });
            }
            EntryKind::Blob | EntryKind::BlobExecutable => {
                return Ok(ResolvedGitTarget {
                    object_id: entry.object_id().to_owned(),
                    repo_path: current_repo_path.clone(),
                    display_path: format!("{rev}:{}", current_repo_path.as_str()),
                });
            }
            EntryKind::Link => {
                let object = entry.object().map_err(|error| RepoError::GitObjectLookup {
                    object_id: entry.object_id().to_string(),
                    reason: error.to_string(),
                })?;
                current_target = std::str::from_utf8(&object.data)
                    .map_err(|_| RepoError::GitSymlinkTargetInvalidUtf8 {
                        rev: rev.to_owned(),
                        path: current_repo_path.clone(),
                    })?
                    .trim_end_matches('\0')
                    .to_owned();
                current_repo_path =
                    normalize_repo_symlink_target(current_repo_path.parent(), &current_target)
                        .map_err(|reason| RepoError::SymlinkTargetEscape {
                            path: observed_repo_path.clone(),
                            target: reason,
                        })?;
            }
            _ => {
                return Err(RepoError::SymlinkTargetDangling {
                    path: observed_repo_path.clone(),
                    target: current_target,
                });
            }
        }
    }
}

fn lookup_git_entry<'repo>(
    root_tree: &gix::Tree<'repo>,
    repo_path: &RepoPath,
) -> RepoResult<Option<gix::object::tree::Entry<'repo>>> {
    root_tree
        .lookup_entry(repo_path.as_str().split('/'))
        .map_err(|error| RepoError::GitObjectLookup {
            object_id: root_tree.id.to_string(),
            reason: error.to_string(),
        })
}

fn followed_target_is_ignored(compiled_ignores: &CompiledIgnoreSet, repo_path: &RepoPath) -> bool {
    let mut ancestor = repo_path.parent();
    while let Some(directory) = ancestor {
        if compiled_ignores.is_ignored(&directory, true) {
            return true;
        }
        ancestor = directory.parent();
    }

    compiled_ignores.is_ignored(repo_path, false)
}

fn normalize_repo_symlink_target(
    base_parent: Option<RepoPath>,
    target: &str,
) -> Result<RepoPath, String> {
    if target.starts_with('/') {
        return Err(target.to_owned());
    }

    let mut segments = base_parent
        .as_ref()
        .map(|path| {
            path.as_str()
                .split('/')
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    for component in target.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if segments.pop().is_none() {
                    return Err(target.to_owned());
                }
            }
            other => segments.push(other.to_owned()),
        }
    }

    let joined = segments.join("/");
    RepoPath::parse(&joined).map_err(|_| target.to_owned())
}

fn normalize_follow_target(root: &Path, current_parent: &Path, target: &Path) -> Option<PathBuf> {
    if target.is_absolute() {
        return None;
    }

    normalize_absolute_path(&current_parent.join(target))
        .filter(|candidate| candidate.starts_with(root))
}

fn normalize_absolute_path(path: &Path) -> Option<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    Some(normalized)
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

fn repo_path_from_git_entry(
    parent: Option<&RepoPath>,
    filename: &gix::bstr::BStr,
) -> RepoResult<RepoPath> {
    let segment = filename.to_str().map_err(|_| RepoError::NonUtf8Path {
        display_path: filename.to_string(),
    })?;
    let path = match parent {
        Some(parent) => format!("{}/{}", parent.as_str(), segment),
        None => segment.to_owned(),
    };
    RepoPath::parse(&path).map_err(|error| map_kernel_repo_path_error(Path::new(&path), error))
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
