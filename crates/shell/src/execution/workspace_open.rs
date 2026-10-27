//! Strict Linux discovery for a future native Workspace owner adapter.
//!
//! A candidate proves only that the nearest control boundary has a compatible marker.
//! Policy, Workspace identity, Lanes, destinations and read authorization are UNCHECKED.
//! This module has no public transport or legacy discovery callers.

#![allow(
    dead_code,
    reason = "P1-A discovery precedes native owner-backed load adoption"
)]

use super::{config_model::parse_config_patch_yaml, workspace};
use std::ffi::{CString, OsStr, OsString};
use std::fs::File;
use std::io::{self, Read};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::ffi::OsStrExt;
use std::path::Component;
use std::path::{Path, PathBuf};

pub(crate) struct WorkspaceOpenAdapter;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DiscoveredWorkspace {
    pub(crate) selected_location: PathBuf,
    pub(crate) root: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum WorkspaceInspection {
    Candidate(DiscoveredWorkspace),
    Absent {
        selected_location: PathBuf,
    },
    Blocked {
        boundary: Option<PathBuf>,
        reason: WorkspaceInspectionBlocker,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkspaceInspectionBlocker {
    Disabled,
    MissingMarker,
    LegacySettings,
    MalformedMarker,
    IncompatibleConfiguration,
    UnsafePath,
    ChangedDuringRead,
    Unavailable(io::ErrorKind),
}

impl WorkspaceOpenAdapter {
    /// Inspect marker compatibility only; callers must separately authenticate and load owners.
    ///
    /// Absolute normalized paths are required. Symlinks (including directory aliases) and
    /// control-subtree selections are refused instead of canonicalized. Held descriptors and
    /// final named-to-held checks detect observed replacement during this call. They do not
    /// authenticate an expected owner, provide an atomic snapshot, detect every ABA, or keep
    /// the observation current after return. No descriptor or discovery result is a capability.
    pub(crate) fn inspect_existing(selected_location: &Path) -> WorkspaceInspection {
        let mut guard = match InspectionGuard::capture(selected_location) {
            Ok(guard) => guard,
            Err(reason) => {
                return WorkspaceInspection::Blocked {
                    boundary: None,
                    reason,
                }
            }
        };
        let result = guard.classify();
        if let Err(reason) = guard.revalidate() {
            return WorkspaceInspection::Blocked {
                boundary: guard.boundary,
                reason,
            };
        }
        match result {
            Ok(Some(root)) => WorkspaceInspection::Candidate(DiscoveredWorkspace {
                selected_location: selected_location.to_path_buf(),
                root,
            }),
            Ok(None) => WorkspaceInspection::Absent {
                selected_location: selected_location.to_path_buf(),
            },
            Err(reason) => WorkspaceInspection::Blocked {
                boundary: guard.boundary,
                reason,
            },
        }
    }
}

type InspectionResult<T> = Result<T, WorkspaceInspectionBlocker>;

fn io_blocker(error: io::Error) -> WorkspaceInspectionBlocker {
    match error.raw_os_error() {
        Some(libc::ELOOP | libc::ENOTDIR) => WorkspaceInspectionBlocker::UnsafePath,
        _ => WorkspaceInspectionBlocker::Unavailable(error.kind()),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EntryIdentity {
    device: u64,
    inode: u64,
    mode: u32,
    owner: u32,
    group: u32,
    // Ignore directory content metadata: unrelated entries may change. Control-entry
    // presence and named physical identity are separately observed and revalidated.
    content: Option<(u64, i64, i64, i64, i64, i64)>,
}

impl EntryIdentity {
    fn from_stat(stat: libc::stat) -> Self {
        Self {
            device: stat.st_dev,
            inode: stat.st_ino,
            mode: stat.st_mode,
            owner: stat.st_uid,
            group: stat.st_gid,
            content: (stat.st_mode & libc::S_IFMT != libc::S_IFDIR).then_some((
                stat.st_nlink,
                stat.st_size,
                stat.st_mtime,
                stat.st_mtime_nsec,
                stat.st_ctime,
                stat.st_ctime_nsec,
            )),
        }
    }

    fn held(file: &File) -> InspectionResult<Self> {
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: the borrowed live file descriptor and output pointer are valid.
        if unsafe { libc::fstat(file.as_raw_fd(), stat.as_mut_ptr()) } != 0 {
            return Err(io_blocker(io::Error::last_os_error()));
        }
        // SAFETY: successful fstat initialized the output.
        Ok(Self::from_stat(unsafe { stat.assume_init() }))
    }

    fn is_directory(self) -> bool {
        self.mode & libc::S_IFMT == libc::S_IFDIR
    }
    fn is_marker(self, device: u64) -> bool {
        self.mode & libc::S_IFMT == libc::S_IFREG
            && self.device == device
            && matches!(self.content, Some((1, size, ..)) if size >= 0)
    }
}

fn observe(directory: &File, name: &OsStr) -> InspectionResult<Option<EntryIdentity>> {
    let name = CString::new(name.as_bytes()).map_err(|_| WorkspaceInspectionBlocker::UnsafePath)?;
    let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: borrowed descriptor, terminated component, and output pointer are valid.
    if unsafe {
        libc::fstatat(
            directory.as_raw_fd(),
            name.as_ptr(),
            stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } != 0
    {
        let error = io::Error::last_os_error();
        return if error.raw_os_error() == Some(libc::ENOENT) {
            Ok(None)
        } else {
            Err(io_blocker(error))
        };
    }
    // SAFETY: successful fstatat initialized the output.
    let identity = EntryIdentity::from_stat(unsafe { stat.assume_init() });
    if identity.mode & libc::S_IFMT == libc::S_IFLNK {
        return Err(WorkspaceInspectionBlocker::UnsafePath);
    }
    Ok(Some(identity))
}

fn open_child(directory: &File, name: &OsStr, is_directory: bool) -> InspectionResult<File> {
    let name = CString::new(name.as_bytes()).map_err(|_| WorkspaceInspectionBlocker::UnsafePath)?;
    let flags = libc::O_RDONLY
        | libc::O_CLOEXEC
        | libc::O_NOFOLLOW
        | libc::O_NONBLOCK
        | if is_directory { libc::O_DIRECTORY } else { 0 };
    // SAFETY: the borrowed directory and terminated component are valid. No creation flags.
    let fd = unsafe { libc::openat(directory.as_raw_fd(), name.as_ptr(), flags) };
    if fd < 0 {
        return Err(io_blocker(io::Error::last_os_error()));
    }
    // SAFETY: successful openat returned a new owned descriptor.
    Ok(unsafe { File::from_raw_fd(fd) })
}

struct HeldDirectory {
    path: PathBuf,
    held: File,
    identity: EntryIdentity,
    parent: Option<(usize, OsString)>,
}

struct EntryObservation {
    directory: usize,
    name: &'static str,
    identity: Option<EntryIdentity>,
}

struct InspectionGuard {
    directories: Vec<HeldDirectory>,
    selected: usize,
    entries: Vec<EntryObservation>,
    held_marker: Option<File>,
    boundary: Option<PathBuf>,
}

impl InspectionGuard {
    fn capture(selected: &Path) -> InspectionResult<Self> {
        let normalized: PathBuf = selected.components().collect();
        if !selected.is_absolute()
            || selected.as_os_str() != normalized.as_os_str()
            || selected.components().any(|component| match component {
                Component::Normal(name) => name == workspace::SUBSTRATE_DIR_NAME,
                Component::RootDir => false,
                _ => true,
            })
        {
            return Err(WorkspaceInspectionBlocker::UnsafePath);
        }
        let held = File::open("/").map_err(io_blocker)?;
        let identity = EntryIdentity::held(&held)?;
        let mut guard = Self {
            directories: vec![HeldDirectory {
                path: PathBuf::from("/"),
                held,
                identity,
                parent: None,
            }],
            selected: 0,
            entries: Vec::new(),
            held_marker: None,
            boundary: None,
        };
        for component in selected.components() {
            if let Component::Normal(name) = component {
                let parent = guard.directories.len() - 1;
                guard.push_directory(parent, name, None)?;
            }
        }
        guard.selected = guard.directories.len() - 1;
        Ok(guard)
    }

    fn push_directory(
        &mut self,
        parent: usize,
        name: &OsStr,
        expected: Option<EntryIdentity>,
    ) -> InspectionResult<usize> {
        let held = open_child(&self.directories[parent].held, name, true)?;
        let identity = EntryIdentity::held(&held)?;
        if !identity.is_directory() {
            return Err(WorkspaceInspectionBlocker::UnsafePath);
        }
        if expected.is_some_and(|expected| expected != identity) {
            return Err(WorkspaceInspectionBlocker::ChangedDuringRead);
        }
        let index = self.directories.len();
        self.directories.push(HeldDirectory {
            path: self.directories[parent].path.join(name),
            held,
            identity,
            parent: Some((parent, name.to_os_string())),
        });
        Ok(index)
    }

    fn observe_entry(
        &mut self,
        directory: usize,
        name: &'static str,
    ) -> InspectionResult<Option<EntryIdentity>> {
        let identity = observe(&self.directories[directory].held, OsStr::new(name))?;
        self.entries.push(EntryObservation {
            directory,
            name,
            identity,
        });
        Ok(identity)
    }

    fn classify(&mut self) -> InspectionResult<Option<PathBuf>> {
        for directory in (0..=self.selected).rev() {
            self.boundary = Some(self.directories[directory].path.clone());
            let control = self.observe_entry(directory, workspace::SUBSTRATE_DIR_NAME)?;
            let Some(control) = control else {
                self.boundary = None;
                continue;
            };
            if !control.is_directory()
                || control.device != self.directories[directory].identity.device
            {
                return Err(WorkspaceInspectionBlocker::UnsafePath);
            }
            let control_directory = self.push_directory(
                directory,
                OsStr::new(workspace::SUBSTRATE_DIR_NAME),
                Some(control),
            )?;
            let marker =
                self.observe_entry(control_directory, workspace::WORKSPACE_MARKER_FILENAME)?;
            let disabled =
                self.observe_entry(control_directory, workspace::WORKSPACE_DISABLED_FILENAME)?;
            let legacy =
                self.observe_entry(control_directory, workspace::LEGACY_SETTINGS_FILENAME)?;
            if disabled.is_some() {
                return Err(WorkspaceInspectionBlocker::Disabled);
            }
            if legacy.is_some() {
                return Err(WorkspaceInspectionBlocker::LegacySettings);
            }
            let marker = marker.ok_or(WorkspaceInspectionBlocker::MissingMarker)?;
            if !marker.is_marker(control.device) {
                return Err(WorkspaceInspectionBlocker::UnsafePath);
            }
            let mut held = open_child(
                &self.directories[control_directory].held,
                OsStr::new(workspace::WORKSPACE_MARKER_FILENAME),
                false,
            )?;
            if EntryIdentity::held(&held)? != marker {
                return Err(WorkspaceInspectionBlocker::ChangedDuringRead);
            }
            let mut bytes = Vec::new();
            held.read_to_end(&mut bytes).map_err(io_blocker)?;
            if EntryIdentity::held(&held)? != marker
                || marker.content.map(|(_, len, ..)| len as u64) != Some(bytes.len() as u64)
            {
                return Err(WorkspaceInspectionBlocker::ChangedDuringRead);
            }
            self.held_marker = Some(held);
            let raw = std::str::from_utf8(&bytes)
                .map_err(|_| WorkspaceInspectionBlocker::MalformedMarker)?;
            // Preserve the existing parser's aliases/anchors and sparse-patch semantics. This
            // first parse only separates malformed YAML from incompatible config/schema.
            serde_yaml::from_str::<serde_yaml::Value>(raw)
                .map_err(|_| WorkspaceInspectionBlocker::MalformedMarker)?;
            parse_config_patch_yaml(
                &self.directories[control_directory]
                    .path
                    .join(workspace::WORKSPACE_MARKER_FILENAME),
                raw,
            )
            .map_err(|_| WorkspaceInspectionBlocker::IncompatibleConfiguration)?;
            return Ok(self.boundary.clone());
        }
        Ok(None)
    }

    fn revalidate(&self) -> InspectionResult<()> {
        for directory in &self.directories {
            if EntryIdentity::held(&directory.held)? != directory.identity {
                return Err(WorkspaceInspectionBlocker::ChangedDuringRead);
            }
            if let Some((parent, name)) = &directory.parent {
                if observe(&self.directories[*parent].held, name)? != Some(directory.identity) {
                    return Err(WorkspaceInspectionBlocker::ChangedDuringRead);
                }
            }
        }
        for entry in &self.entries {
            if observe(
                &self.directories[entry.directory].held,
                OsStr::new(entry.name),
            )? != entry.identity
            {
                return Err(WorkspaceInspectionBlocker::ChangedDuringRead);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::fs::{self, File};
    use std::os::unix::fs::{symlink, FileExt, MetadataExt, PermissionsExt};
    use tempfile::TempDir;

    #[derive(Debug, PartialEq, Eq)]
    struct EntrySnapshot {
        mode: u32,
        bytes: Option<Vec<u8>>,
        link: Option<PathBuf>,
        read_error: Option<io::ErrorKind>,
    }

    fn snapshot(root: &Path) -> BTreeMap<PathBuf, EntrySnapshot> {
        fn collect(root: &Path, path: &Path, out: &mut BTreeMap<PathBuf, EntrySnapshot>) {
            let meta = fs::symlink_metadata(path).unwrap();
            let mut item = EntrySnapshot {
                mode: meta.mode(),
                bytes: None,
                link: None,
                read_error: None,
            };
            if meta.is_symlink() {
                item.link = Some(fs::read_link(path).unwrap());
            } else if meta.is_file() {
                match fs::read(path) {
                    Ok(bytes) => item.bytes = Some(bytes),
                    Err(e) => item.read_error = Some(e.kind()),
                }
            } else if meta.is_dir() {
                match fs::read_dir(path) {
                    Ok(entries) => {
                        for entry in entries {
                            collect(root, &entry.unwrap().path(), out);
                        }
                    }
                    Err(e) => item.read_error = Some(e.kind()),
                }
            }
            out.insert(path.strip_prefix(root).unwrap().to_path_buf(), item);
        }
        let mut out = BTreeMap::new();
        collect(root, root, &mut out);
        out
    }

    fn preserved(root: &Path, selected: &Path) -> WorkspaceInspection {
        let before = snapshot(root);
        let result = WorkspaceOpenAdapter::inspect_existing(selected);
        assert_eq!(
            before,
            snapshot(root),
            "inspection changed fixture entries/content/modes"
        );
        println!("workspace_open_ snapshot unchanged: entries={}, contents={}, modes={}, unreadable={}; result={result:?}",
            before.len(), before.values().filter(|entry| entry.bytes.is_some()).count(),
            before.len(), before.values().filter(|entry| entry.read_error.is_some()).count());
        result
    }

    fn marker(root: &Path, bytes: &[u8]) {
        fs::create_dir_all(root.join(".substrate")).unwrap();
        fs::write(root.join(".substrate/workspace.yaml"), bytes).unwrap();
    }

    fn fixture() -> (TempDir, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let ancestor = temp.path().join("ancestor");
        marker(&ancestor, b"{}\n");
        let boundary = ancestor.join("child");
        let selected = boundary.join("nested");
        fs::create_dir_all(&selected).unwrap();
        (temp, boundary, selected)
    }

    fn blocked(boundary: &Path, reason: WorkspaceInspectionBlocker) -> WorkspaceInspection {
        WorkspaceInspection::Blocked {
            boundary: Some(boundary.to_path_buf()),
            reason,
        }
    }

    #[test]
    fn workspace_open_positive_direct_and_nested_marker_only() {
        let (temp, root, nested) = fixture();
        marker(&root, b"world:\n  enabled: false\n");
        for selected in [&root, &nested] {
            assert_eq!(
                preserved(temp.path(), selected),
                WorkspaceInspection::Candidate(DiscoveredWorkspace {
                    selected_location: selected.to_path_buf(),
                    root: root.clone(),
                })
            );
        }
    }

    #[test]
    fn workspace_open_absence_is_distinct_from_missing_or_non_directory_selection() {
        let temp = tempfile::tempdir().unwrap();
        assert_eq!(
            preserved(temp.path(), temp.path()),
            WorkspaceInspection::Absent {
                selected_location: temp.path().to_path_buf()
            }
        );
        assert!(matches!(
            preserved(temp.path(), &temp.path().join("absent")),
            WorkspaceInspection::Blocked {
                reason: WorkspaceInspectionBlocker::Unavailable(io::ErrorKind::NotFound),
                ..
            }
        ));
        let file = temp.path().join("file");
        fs::write(&file, "").unwrap();
        assert!(matches!(
            preserved(temp.path(), &file),
            WorkspaceInspection::Blocked { .. }
        ));
    }

    #[test]
    fn workspace_open_partial_and_disabled_boundaries_block_valid_ancestor() {
        for case in ["empty", "policy-only", "disabled", "disabled-no-marker"] {
            let (temp, root, nested) = fixture();
            fs::create_dir(root.join(".substrate")).unwrap();
            let reason = match case {
                "policy-only" => {
                    fs::write(root.join(".substrate/policy.yaml"), "{}").unwrap();
                    WorkspaceInspectionBlocker::MissingMarker
                }
                "disabled" | "disabled-no-marker" => {
                    if case == "disabled" {
                        marker(&root, b"{}");
                    }
                    fs::write(root.join(".substrate/workspace.disabled"), "").unwrap();
                    WorkspaceInspectionBlocker::Disabled
                }
                _ => WorkspaceInspectionBlocker::MissingMarker,
            };
            assert_eq!(
                preserved(temp.path(), &nested),
                blocked(&root, reason),
                "{case}"
            );
        }
    }

    #[test]
    fn workspace_open_malformed_legacy_incompatible_boundaries_block_valid_ancestor() {
        for (bytes, reason) in [
            (b"[".as_slice(), WorkspaceInspectionBlocker::MalformedMarker),
            (
                b"world: {}\nworld: {}",
                WorkspaceInspectionBlocker::MalformedMarker,
            ),
            (b"\xff", WorkspaceInspectionBlocker::MalformedMarker),
            (
                b"version: 99",
                WorkspaceInspectionBlocker::IncompatibleConfiguration,
            ),
            (
                b"unknown_field: true",
                WorkspaceInspectionBlocker::IncompatibleConfiguration,
            ),
            (
                b"world:\n  anchor_mode: impossible",
                WorkspaceInspectionBlocker::IncompatibleConfiguration,
            ),
        ] {
            let (temp, root, nested) = fixture();
            marker(&root, bytes);
            assert_eq!(
                preserved(temp.path(), &nested),
                blocked(&root, reason),
                "{bytes:?}"
            );
        }
        let (temp, root, nested) = fixture();
        marker(&root, b"{}");
        fs::write(root.join(".substrate/settings.yaml"), "{}").unwrap();
        assert_eq!(
            preserved(temp.path(), &nested),
            blocked(&root, WorkspaceInspectionBlocker::LegacySettings)
        );
    }

    #[test]
    fn workspace_open_policy_and_owner_state_are_unchecked_even_when_bad_or_missing() {
        let (temp, root, nested) = fixture();
        marker(&root, b"{}");
        let policy = root.join(".substrate/policy.yaml");
        for content in [None, Some("[invalid"), Some("{}")] {
            if let Some(content) = content {
                fs::write(&policy, content).unwrap();
            }
            assert!(
                matches!(preserved(temp.path(), &nested), WorkspaceInspection::Candidate(DiscoveredWorkspace { root: candidate, .. }) if candidate == root)
            );
        }
        fs::set_permissions(&policy, fs::Permissions::from_mode(0)).unwrap();
        assert!(matches!(
            preserved(temp.path(), &nested),
            WorkspaceInspection::Candidate(_)
        ));
        fs::set_permissions(&policy, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(
            root.join(".substrate/workspace-state.v1.json"),
            "invalid owner state",
        )
        .unwrap();
        assert!(matches!(
            preserved(temp.path(), &nested),
            WorkspaceInspection::Candidate(_)
        ));
    }

    #[test]
    fn workspace_open_unreadable_boundary_and_marker_refuse_without_fallback() {
        assert_ne!(
            unsafe { libc::geteuid() },
            0,
            "permission proof requires a non-root test identity"
        );
        for name in [".substrate", ".substrate/workspace.yaml"] {
            let (temp, root, nested) = fixture();
            marker(&root, b"{}\n");
            let marker_path = root.join(".substrate/workspace.yaml");
            let held = File::open(&marker_path).unwrap();
            let before = snapshot(temp.path());
            let target = root.join(name);
            let original = fs::metadata(&target).unwrap().permissions();
            fs::set_permissions(&target, fs::Permissions::from_mode(0)).unwrap();
            let permission_error = File::open(&target).unwrap_err();
            assert_eq!(permission_error.raw_os_error(), Some(libc::EACCES));
            println!(
                "workspace_open_ unreadable {name}: actual EACCES errno={:?}",
                permission_error.raw_os_error()
            );
            let result = preserved(temp.path(), &nested);
            let mut bytes = [0; 3];
            assert_eq!(held.read_at(&mut bytes, 0).unwrap(), 3);
            assert_eq!(&bytes, b"{}\n");
            assert_eq!(
                result,
                blocked(
                    &root,
                    WorkspaceInspectionBlocker::Unavailable(io::ErrorKind::PermissionDenied)
                )
            );
            fs::set_permissions(&target, original).unwrap();
            assert_eq!(before, snapshot(temp.path()));
        }
    }

    #[test]
    fn workspace_open_symlink_control_and_marker_boundaries_block_valid_ancestor() {
        for dangling in [false, true] {
            for name in [
                ".substrate",
                ".substrate/workspace.yaml",
                ".substrate/workspace.disabled",
                ".substrate/settings.yaml",
            ] {
                let (temp, root, nested) = fixture();
                if name != ".substrate" {
                    marker(&root, b"{}");
                }
                let target = root.join(name);
                if target.exists() {
                    fs::remove_file(&target).unwrap();
                }
                let link_target = temp.path().join("link-target");
                if !dangling {
                    if name == ".substrate" {
                        fs::create_dir(&link_target).unwrap();
                        fs::write(link_target.join("workspace.yaml"), "{}").unwrap();
                    } else {
                        fs::write(&link_target, "{}").unwrap();
                    }
                }
                symlink(&link_target, &target).unwrap();
                for selected in [&root, &nested] {
                    assert_eq!(
                        preserved(temp.path(), selected),
                        blocked(&root, WorkspaceInspectionBlocker::UnsafePath),
                        "{name}, dangling={dangling}"
                    );
                }
            }
        }
    }

    #[test]
    fn workspace_open_selected_aliases_and_control_subtrees_are_unsafe() {
        let (temp, root, nested) = fixture();
        marker(&root, b"{}");
        let alias = temp.path().join("alias");
        symlink(&root, &alias).unwrap();
        for selected in [
            alias.clone(),
            alias.join("nested"),
            nested.join(".."),
            PathBuf::from(format!("{}/./nested", root.display())),
            PathBuf::from(format!("{}//nested", root.display())),
            PathBuf::from(format!("{}/", nested.display())),
            root.join(".substrate"),
            root.join(".substrate/subdir"),
            PathBuf::from("relative"),
        ] {
            assert!(
                matches!(
                    preserved(temp.path(), &selected),
                    WorkspaceInspection::Blocked {
                        reason: WorkspaceInspectionBlocker::UnsafePath,
                        ..
                    }
                ),
                "{selected:?}"
            );
        }
    }

    #[test]
    fn workspace_open_unsafe_entry_kinds_and_hardlinks_block_valid_ancestor() {
        for case in [
            "control-file",
            "marker-directory",
            "marker-fifo",
            "marker-hardlink",
        ] {
            let (temp, root, nested) = fixture();
            let control = root.join(".substrate");
            let target = control.join("workspace.yaml");
            if case == "control-file" {
                fs::write(&control, "{}").unwrap();
            } else {
                fs::create_dir(&control).unwrap();
                match case {
                    "marker-directory" => fs::create_dir(&target).unwrap(),
                    "marker-fifo" => {
                        use std::os::unix::ffi::OsStrExt;
                        let name = std::ffi::CString::new(target.as_os_str().as_bytes()).unwrap();
                        assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
                    }
                    _ => {
                        fs::write(&target, "{}").unwrap();
                        fs::hard_link(&target, temp.path().join("hardlink")).unwrap();
                    }
                }
            }
            assert_eq!(
                preserved(temp.path(), &nested),
                blocked(&root, WorkspaceInspectionBlocker::UnsafePath),
                "{case}"
            );
        }
    }
    #[test]
    fn workspace_open_revalidation_detects_root_control_marker_and_boundary_substitution() {
        for case in ["selected-root", "control", "marker", "new-boundary"] {
            let (temp, root, nested) = fixture();
            if case != "new-boundary" {
                marker(&root, b"{}");
            }
            let mut guard = InspectionGuard::capture(&nested).unwrap();
            let expected_root = if case == "new-boundary" {
                root.parent().unwrap()
            } else {
                &root
            };
            assert_eq!(guard.classify().unwrap(), Some(expected_root.to_path_buf()));
            match case {
                "selected-root" => {
                    fs::rename(&nested, root.join("held-nested")).unwrap();
                    fs::create_dir(&nested).unwrap();
                }
                "control" => {
                    fs::rename(root.join(".substrate"), root.join("held-control")).unwrap();
                    marker(&root, b"{}");
                }
                "marker" => {
                    fs::rename(
                        root.join(".substrate/workspace.yaml"),
                        root.join("held-marker"),
                    )
                    .unwrap();
                    fs::write(root.join(".substrate/workspace.yaml"), "{}").unwrap();
                }
                _ => fs::create_dir(root.join(".substrate")).unwrap(),
            }
            let before = snapshot(temp.path());
            assert_eq!(
                guard.revalidate(),
                Err(WorkspaceInspectionBlocker::ChangedDuringRead),
                "{case}"
            );
            assert_eq!(before, snapshot(temp.path()));
            println!("workspace_open_ guard {case}: ChangedDuringRead, entries/content/modes unchanged ({} entries)", before.len());
        }
    }

    #[test]
    fn workspace_open_yaml_anchors_preserve_existing_config_parser_semantics() {
        let (temp, root, nested) = fixture();
        marker(&root, b"world:\n  enabled: &flag false\n  caged: *flag\n");
        assert!(matches!(
            preserved(temp.path(), &nested),
            WorkspaceInspection::Candidate(_)
        ));
    }

    #[test]
    fn workspace_open_unreadable_selected_directory_is_not_absent() {
        assert_ne!(
            unsafe { libc::geteuid() },
            0,
            "permission proof requires a non-root test identity"
        );
        let (temp, root, nested) = fixture();
        let before = snapshot(temp.path());
        let original = fs::metadata(&nested).unwrap().permissions();
        fs::set_permissions(&nested, fs::Permissions::from_mode(0)).unwrap();
        let permission_error = File::open(&nested).unwrap_err();
        assert_eq!(permission_error.raw_os_error(), Some(libc::EACCES));
        println!(
            "workspace_open_ unreadable selected directory: actual EACCES errno={:?}",
            permission_error.raw_os_error()
        );
        assert!(matches!(
            preserved(temp.path(), &nested),
            WorkspaceInspection::Blocked {
                reason: WorkspaceInspectionBlocker::Unavailable(io::ErrorKind::PermissionDenied),
                ..
            }
        ));
        fs::set_permissions(&nested, original).unwrap();
        assert_eq!(before, snapshot(temp.path()));
        assert!(root.is_dir());
    }
    #[test]
    fn workspace_open_missing_and_file_selected_paths_never_find_valid_ancestor() {
        let (temp, root, nested) = fixture();
        let file = root.join("selected-file");
        fs::write(&file, "{}").unwrap();
        for selected in [&file, &nested.join("missing")] {
            assert!(matches!(
                preserved(temp.path(), selected),
                WorkspaceInspection::Blocked { .. }
            ));
        }
    }

    #[test]
    fn workspace_open_revalidation_detects_marker_mutation_disabled_and_legacy_changes() {
        for case in ["marker-mutation", "disabled", "legacy"] {
            let (temp, root, nested) = fixture();
            marker(&root, b"{}");
            let mut guard = InspectionGuard::capture(&nested).unwrap();
            assert_eq!(guard.classify().unwrap(), Some(root.clone()));
            let name = match case {
                "disabled" => "workspace.disabled",
                "legacy" => "settings.yaml",
                _ => "workspace.yaml",
            };
            fs::write(
                root.join(".substrate").join(name),
                "world: {enabled: false}",
            )
            .unwrap();
            let before = snapshot(temp.path());
            assert_eq!(
                guard.revalidate(),
                Err(WorkspaceInspectionBlocker::ChangedDuringRead)
            );
            assert_eq!(before, snapshot(temp.path()));
            println!("workspace_open_ guard {case}: ChangedDuringRead, entries/content/modes unchanged ({} entries)", before.len());
        }
    }
}
