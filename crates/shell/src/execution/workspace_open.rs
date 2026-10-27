//! Strict Linux discovery and internal native settings observation.
//!
//! A candidate proves only that the nearest control boundary has a compatible marker.
//! Discovery alone leaves policy unchecked. Settings observation composes config/policy,
//! while Workspace identity, Lanes, destinations and consumer authorization stay unsupported.
//! This module has no public transport or legacy discovery callers.

#![allow(
    dead_code,
    reason = "P1-A discovery precedes native owner-backed load adoption"
)]

use super::agent_runtime::host_session_authority::{
    schema::DirectoryPhysicalIdentityV1, trusted_fs::TrustedAuthorityRoot,
};
use super::agent_runtime::{HostSessionAuthority, OpenedBootstrapHomeV1};
use super::{
    config_model::{
        self, parse_config_patch_yaml, CapturedConfigOverridesV1, CliConfigOverrides,
        ConfigExplainV1, SubstrateConfig,
    },
    policy_model, workspace,
};
use std::ffi::{CString, OsStr, OsString};
use std::fs::File;
use std::io::{self, Read};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{FileExt, OpenOptionsExt};
use std::path::Component;
use std::path::{Path, PathBuf};
use transport_api_types::{InstallBootstrapContextCarrierV1, PlatformPrincipalV1};

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
    marker_bytes: Option<Vec<u8>>,
    control_directory: Option<usize>,
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
            marker_bytes: None,
            control_directory: None,
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
            self.marker_bytes = Some(bytes.clone());
            self.control_directory = Some(control_directory);
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
        if let (Some(held), Some(bytes), Some(directory)) = (
            &self.held_marker,
            &self.marker_bytes,
            self.control_directory,
        ) {
            let expected = self
                .entries
                .iter()
                .find(|entry| {
                    entry.directory == directory
                        && entry.name == workspace::WORKSPACE_MARKER_FILENAME
                })
                .and_then(|entry| entry.identity);
            if Some(EntryIdentity::held(held)?) != expected || read_held_bytes(held)? != *bytes {
                return Err(WorkspaceInspectionBlocker::ChangedDuringRead);
            }
        }
        Ok(())
    }
}

/// Sealed native invocation binding, not a Workspace consumer read grant.
pub(crate) struct WorkspaceNativeSettingsContextV1 {
    carrier: InstallBootstrapContextCarrierV1,
    authority: HostSessionAuthority,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkspaceSettingsContextFailureV1 {
    InvalidNativeContext,
    PrincipalDenied,
    HomeUnsupportedOrUnavailable,
}

impl WorkspaceNativeSettingsContextV1 {
    pub(crate) fn from_native_invocation(
        carrier: &InstallBootstrapContextCarrierV1,
    ) -> Result<Self, WorkspaceSettingsContextFailureV1> {
        use WorkspaceSettingsContextFailureV1::*;
        carrier.validate().map_err(|_| InvalidNativeContext)?;
        super::install_bootstrap::bind_unix_install_bootstrap_context(carrier)
            .map_err(|_| PrincipalDenied)?;
        let PlatformPrincipalV1::Unix { uid, .. } = carrier.context.intended_host_principal else {
            return Err(PrincipalDenied);
        };
        let uid = libc::uid_t::try_from(uid).map_err(|_| InvalidNativeContext)?;
        let root = TrustedAuthorityRoot::open_for_owner(
            Path::new(&carrier.context.host_substrate_home),
            uid,
        )
        .map_err(|_| HomeUnsupportedOrUnavailable)?;
        let authority = HostSessionAuthority::from_trusted_root(root)
            .map_err(|_| HomeUnsupportedOrUnavailable)?;
        Ok(Self {
            carrier: carrier.clone(),
            authority,
        })
    }

    fn revalidate(&self) -> Result<(), WorkspaceSettingsContextFailureV1> {
        super::install_bootstrap::bind_unix_install_bootstrap_context(&self.carrier)
            .map_err(|_| WorkspaceSettingsContextFailureV1::PrincipalDenied)?;
        self.authority
            .bootstrap_home()
            .revalidate()
            .map_err(|_| WorkspaceSettingsContextFailureV1::HomeUnsupportedOrUnavailable)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingsSourceV1 {
    GlobalConfig,
    WorkspaceConfig,
    GlobalPolicy,
    WorkspacePolicy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkspaceSettingsBlockerV1 {
    Inspection(WorkspaceInspectionBlocker),
    NativeContext(WorkspaceSettingsContextFailureV1),
    InvalidOverrides,
    InvalidSource(SettingsSourceV1),
    IncompatibleConfigPolicy,
    UnsafeSource(SettingsSourceV1),
    ChangedDuringRead,
    Unavailable(SettingsSourceV1, io::ErrorKind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UnsupportedWorkspaceOwnerReasonV1 {
    OwnerReaderNotImplemented,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkspaceOwnerComponentV1 {
    Unsupported(UnsupportedWorkspaceOwnerReasonV1),
}

/// Paths and values are internal observations, with no consumer disclosure authorization.
/// Deliberately lacks Debug and serialization, including for its contained source material.
pub(crate) struct WorkspaceSettingsObservationV1 {
    pub(crate) selected_location: PathBuf,
    pub(crate) root: PathBuf,
    pub(crate) config: SubstrateConfig,
    pub(crate) config_provenance: ConfigExplainV1,
    pub(crate) policy: substrate_broker::Policy,
    pub(crate) policy_provenance: substrate_broker::PolicyExplainV1,
    pub(crate) global_config_source: Option<PathBuf>,
    pub(crate) global_policy_source: Option<PathBuf>,
    pub(crate) workspace_config_source: PathBuf,
    pub(crate) workspace_policy_source: Option<PathBuf>,
    pub(crate) captured_overrides: CapturedConfigOverridesV1,
    pub(crate) identity: WorkspaceOwnerComponentV1,
    pub(crate) lanes: WorkspaceOwnerComponentV1,
    pub(crate) destinations: WorkspaceOwnerComponentV1,
    pub(crate) consumer_read_authorization: WorkspaceOwnerComponentV1,
}

pub(crate) enum WorkspaceSettingsLoadV1 {
    SettingsObserved(WorkspaceSettingsObservationV1),
    Absent {
        selected_location: PathBuf,
    },
    Blocked {
        boundary: Option<PathBuf>,
        reason: WorkspaceSettingsBlockerV1,
    },
}

type SettingsResult<T> = Result<T, WorkspaceSettingsBlockerV1>;

fn source_blocker(
    source: SettingsSourceV1,
    reason: WorkspaceInspectionBlocker,
) -> WorkspaceSettingsBlockerV1 {
    match reason {
        WorkspaceInspectionBlocker::UnsafePath => WorkspaceSettingsBlockerV1::UnsafeSource(source),
        WorkspaceInspectionBlocker::ChangedDuringRead => {
            WorkspaceSettingsBlockerV1::ChangedDuringRead
        }
        WorkspaceInspectionBlocker::Unavailable(kind) => {
            WorkspaceSettingsBlockerV1::Unavailable(source, kind)
        }
        other => WorkspaceSettingsBlockerV1::Inspection(other),
    }
}

// Positional reads leave held-descriptor offsets untouched, including at revalidation.
fn read_held_bytes(held: &File) -> InspectionResult<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = held
            .read_at(&mut buffer, bytes.len() as u64)
            .map_err(io_blocker)?;
        if count == 0 {
            return Ok(bytes);
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
}

struct HeldSettingsSource {
    name: &'static str,
    source: SettingsSourceV1,
    identity: Option<EntryIdentity>,
    held: Option<File>,
    bytes: Option<Vec<u8>>,
}

impl HeldSettingsSource {
    fn capture(
        directory: &File,
        name: &'static str,
        source: SettingsSourceV1,
    ) -> SettingsResult<Self> {
        let capture = || -> InspectionResult<Self> {
            let identity = observe(directory, OsStr::new(name))?;
            let Some(expected) = identity else {
                return Ok(Self {
                    name,
                    source,
                    identity,
                    held: None,
                    bytes: None,
                });
            };
            if !expected.is_marker(EntryIdentity::held(directory)?.device) {
                return Err(WorkspaceInspectionBlocker::UnsafePath);
            }
            let held = open_child(directory, OsStr::new(name), false)?;
            if EntryIdentity::held(&held)? != expected {
                return Err(WorkspaceInspectionBlocker::ChangedDuringRead);
            }
            let bytes = read_held_bytes(&held)?;
            if EntryIdentity::held(&held)? != expected
                || expected.content.map(|(_, len, ..)| len as u64) != Some(bytes.len() as u64)
            {
                return Err(WorkspaceInspectionBlocker::ChangedDuringRead);
            }
            Ok(Self {
                name,
                source,
                identity,
                held: Some(held),
                bytes: Some(bytes),
            })
        };
        capture().map_err(|reason| source_blocker(source, reason))
    }

    fn revalidate(&self, directory: &File) -> SettingsResult<()> {
        let check = || -> InspectionResult<()> {
            if observe(directory, OsStr::new(self.name))? != self.identity {
                return Err(WorkspaceInspectionBlocker::ChangedDuringRead);
            }
            if let (Some(expected), Some(held), Some(bytes)) =
                (self.identity, &self.held, &self.bytes)
            {
                if EntryIdentity::held(held)? != expected || read_held_bytes(held)? != *bytes {
                    return Err(WorkspaceInspectionBlocker::ChangedDuringRead);
                }
                let named = open_child(directory, OsStr::new(self.name), false)?;
                if EntryIdentity::held(&named)? != expected
                    || read_held_bytes(&named)? != *bytes
                    || EntryIdentity::held(&named)? != expected
                    || observe(directory, OsStr::new(self.name))? != Some(expected)
                {
                    return Err(WorkspaceInspectionBlocker::ChangedDuringRead);
                }
            }
            Ok(())
        };
        check().map_err(|reason| source_blocker(self.source, reason))
    }
}

struct HeldSettingsHome {
    held: File,
    identity: EntryIdentity,
    path: PathBuf,
}

impl HeldSettingsHome {
    fn capture(bootstrap: &OpenedBootstrapHomeV1<'_>) -> SettingsResult<Self> {
        let identity = bootstrap.identity().map_err(|_| {
            WorkspaceSettingsBlockerV1::NativeContext(
                WorkspaceSettingsContextFailureV1::HomeUnsupportedOrUnavailable,
            )
        })?;
        let path = PathBuf::from(&identity.physical_path);
        let held = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(&path)
            .map_err(|error| source_blocker(SettingsSourceV1::GlobalPolicy, io_blocker(error)))?;
        let observed = EntryIdentity::held(&held)
            .map_err(|reason| source_blocker(SettingsSourceV1::GlobalPolicy, reason))?;
        let matches = match identity.physical_identity {
            DirectoryPhysicalIdentityV1::Linux { device_id, inode } => {
                observed.device == device_id && observed.inode == inode
            }
            _ => false,
        };
        if !observed.is_directory() || !matches {
            return Err(WorkspaceSettingsBlockerV1::ChangedDuringRead);
        }
        Ok(Self {
            held,
            identity: observed,
            path,
        })
    }

    fn revalidate(&self, bootstrap: &OpenedBootstrapHomeV1<'_>) -> SettingsResult<()> {
        let reopened = Self::capture(bootstrap)?;
        if reopened.path != self.path
            || reopened.identity != self.identity
            || EntryIdentity::held(&self.held)
                .map_err(|reason| source_blocker(SettingsSourceV1::GlobalPolicy, reason))?
                != self.identity
        {
            return Err(WorkspaceSettingsBlockerV1::ChangedDuringRead);
        }
        Ok(())
    }
}

#[cfg(test)]
thread_local! {
    static AFTER_SETTINGS_COLLECT: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = const { std::cell::RefCell::new(None) };
}

impl WorkspaceOpenAdapter {
    /// Observe fresh settings at the strict held boundary. This neither authorizes consumer
    /// disclosure nor establishes an atomic snapshot, ABA/revision cut, or later-use authority.
    pub(crate) fn load_existing(
        selected: &Path,
        context: &WorkspaceNativeSettingsContextV1,
        cli: &CliConfigOverrides,
    ) -> WorkspaceSettingsLoadV1 {
        use WorkspaceSettingsBlockerV1 as Blocker;
        if let Err(reason) = context.revalidate() {
            return WorkspaceSettingsLoadV1::Blocked {
                boundary: None,
                reason: Blocker::NativeContext(reason),
            };
        }
        let mut guard = match InspectionGuard::capture(selected) {
            Ok(guard) => guard,
            Err(reason) => {
                return WorkspaceSettingsLoadV1::Blocked {
                    boundary: None,
                    reason: Blocker::Inspection(reason),
                }
            }
        };
        let collected = (|| -> SettingsResult<Option<WorkspaceSettingsObservationV1>> {
            let Some(root) = guard.classify().map_err(Blocker::Inspection)? else {
                return Ok(None);
            };
            let overrides =
                CapturedConfigOverridesV1::capture(cli).map_err(|_| Blocker::InvalidOverrides)?;
            let control_index = guard.control_directory.ok_or(Blocker::ChangedDuringRead)?;
            let control = &guard.directories[control_index];
            let marker_path = control.path.join(workspace::WORKSPACE_MARKER_FILENAME);
            let marker_bytes = guard
                .marker_bytes
                .as_deref()
                .ok_or(Blocker::ChangedDuringRead)?;
            let workspace_policy = HeldSettingsSource::capture(
                &control.held,
                workspace::WORKSPACE_POLICY_FILENAME,
                SettingsSourceV1::WorkspacePolicy,
            )?;
            let workspace_policy_path = control.path.join(workspace::WORKSPACE_POLICY_FILENAME);
            let bootstrap = context.authority.bootstrap_home();
            let home = HeldSettingsHome::capture(&bootstrap)?;
            let global_config_guard = HeldSettingsSource::capture(
                &home.held,
                "config.yaml",
                SettingsSourceV1::GlobalConfig,
            )?;
            let global_config = bootstrap.open_e3_config_source().map_err(|_| {
                Blocker::Unavailable(SettingsSourceV1::GlobalConfig, io::ErrorKind::Other)
            })?;
            if global_config.source_bytes() != global_config_guard.bytes.as_deref() {
                return Err(Blocker::ChangedDuringRead);
            }
            let global_policy = HeldSettingsSource::capture(
                &home.held,
                "policy.yaml",
                SettingsSourceV1::GlobalPolicy,
            )?;
            let policy_bytes = bootstrap.read_policy_yaml().map_err(|_| {
                Blocker::Unavailable(SettingsSourceV1::GlobalPolicy, io::ErrorKind::Other)
            })?;
            if policy_bytes != global_policy.bytes {
                return Err(Blocker::ChangedDuringRead);
            }
            let global_config_path = home.path.join("config.yaml");
            let global_policy_path = home.path.join("policy.yaml");
            // Parsing and canonical finalization happen for every supported gateway/policy mode.
            let (policy, policy_provenance) =
                policy_model::resolve_effective_policy_from_sources_v1(
                    &global_policy_path,
                    policy_bytes.as_deref(),
                    workspace_policy
                        .bytes
                        .as_deref()
                        .map(|bytes| (workspace_policy_path.as_path(), bytes)),
                )
                .map_err(|_| Blocker::IncompatibleConfigPolicy)?;
            if let Some(bytes) = global_config.source_bytes() {
                let raw = std::str::from_utf8(bytes)
                    .map_err(|_| Blocker::InvalidSource(SettingsSourceV1::GlobalConfig))?;
                parse_config_patch_yaml(&global_config_path, raw)
                    .map_err(|_| Blocker::InvalidSource(SettingsSourceV1::GlobalConfig))?;
            }
            let (config, config_provenance) =
                config_model::resolve_effective_config_from_sources_v1(
                    &global_config_path,
                    global_config.source_bytes(),
                    &marker_path,
                    marker_bytes,
                    &overrides,
                    &policy,
                )
                .map_err(|_| Blocker::IncompatibleConfigPolicy)?;
            #[cfg(test)]
            AFTER_SETTINGS_COLLECT.with(|hook| {
                if let Some(hook) = hook.borrow_mut().take() {
                    hook();
                }
            });
            // These are bounded observable substitution checks, not owner monotonic revisions.
            workspace_policy.revalidate(&control.held)?;
            home.revalidate(&bootstrap)?;
            global_config_guard.revalidate(&home.held)?;
            global_config
                .revalidate()
                .map_err(|_| Blocker::ChangedDuringRead)?;
            global_policy.revalidate(&home.held)?;
            let unsupported = WorkspaceOwnerComponentV1::Unsupported(
                UnsupportedWorkspaceOwnerReasonV1::OwnerReaderNotImplemented,
            );
            Ok(Some(WorkspaceSettingsObservationV1 {
                selected_location: selected.to_path_buf(),
                root,
                config,
                config_provenance,
                policy,
                policy_provenance,
                global_config_source: global_config.source_bytes().map(|_| global_config_path),
                global_policy_source: policy_bytes.as_ref().map(|_| global_policy_path),
                workspace_config_source: marker_path,
                workspace_policy_source: workspace_policy
                    .bytes
                    .as_ref()
                    .map(|_| workspace_policy_path),
                captured_overrides: overrides,
                identity: unsupported,
                lanes: unsupported,
                destinations: unsupported,
                consumer_read_authorization: unsupported,
            }))
        })();
        let result = guard
            .revalidate()
            .map_err(Blocker::Inspection)
            .and_then(|_| context.revalidate().map_err(Blocker::NativeContext))
            .and(collected);
        match result {
            Ok(Some(observation)) => WorkspaceSettingsLoadV1::SettingsObserved(observation),
            Ok(None) => WorkspaceSettingsLoadV1::Absent {
                selected_location: selected.to_path_buf(),
            },
            Err(reason) => WorkspaceSettingsLoadV1::Blocked {
                boundary: guard.boundary,
                reason,
            },
        }
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
    struct SettingsFixture {
        temp: TempDir,
        home: PathBuf,
        root: PathBuf,
        nested: PathBuf,
        context: WorkspaceNativeSettingsContextV1,
    }

    fn private_file(path: &Path, bytes: &[u8]) {
        fs::write(path, bytes).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    }

    fn settings_carrier(home: &Path) -> InstallBootstrapContextCarrierV1 {
        super::super::install_bootstrap::construct_unix_install_bootstrap_context(
            Some(home),
            OsStr::new("substrate"),
            Path::new("/unused/substrate"),
            None,
            Path::new("/"),
        )
        .unwrap()
    }

    fn settings_fixture() -> SettingsFixture {
        let temp = tempfile::tempdir_in(
            std::env::var_os("XDG_RUNTIME_DIR").expect("supported private fixture parent"),
        )
        .unwrap();
        let home = temp.path().join("product");
        fs::create_dir(&home).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
        private_file(
            &home.join("config.yaml"),
            b"world: {enabled: false, caged: false}\nsync: {exclude: [global]}\n",
        );
        private_file(
            &home.join("policy.yaml"),
            b"id: global-policy\ncmd_allowed: [global]\nllm: {fail_closed: {routing: false}}\n",
        );
        let root = temp.path().join("workspace");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).unwrap();
        marker(&root, b"world: {enabled: true}\nsync: {exclude: [local]}\n");
        private_file(
            &root.join(".substrate/policy.yaml"),
            b"name: local-policy\ncmd_allowed: [local]\n",
        );
        let before = snapshot(temp.path());
        let context =
            WorkspaceNativeSettingsContextV1::from_native_invocation(&settings_carrier(&home))
                .expect("native fixture context");
        assert_eq!(
            before,
            snapshot(temp.path()),
            "context construction wrote fixture"
        );
        SettingsFixture {
            temp,
            home,
            root,
            nested,
            context,
        }
    }

    fn settings_preserved(fixture: &SettingsFixture, selected: &Path) -> WorkspaceSettingsLoadV1 {
        let before = snapshot(fixture.temp.path());
        let result = WorkspaceOpenAdapter::load_existing(
            selected,
            &fixture.context,
            &CliConfigOverrides::default(),
        );
        assert_eq!(
            before,
            snapshot(fixture.temp.path()),
            "settings read changed entries/content/modes"
        );
        println!(
            "workspace_settings_ unchanged entries/content/modes: {} entries",
            before.len()
        );
        result
    }

    fn settings_observed(result: WorkspaceSettingsLoadV1) -> WorkspaceSettingsObservationV1 {
        match result {
            WorkspaceSettingsLoadV1::SettingsObserved(observation) => observation,
            WorkspaceSettingsLoadV1::Blocked { reason, .. } => {
                panic!("expected observation; safe blocker: {reason:?}")
            }
            WorkspaceSettingsLoadV1::Absent { .. } => panic!("expected observation; got absence"),
        }
    }

    fn settings_reason(result: WorkspaceSettingsLoadV1) -> WorkspaceSettingsBlockerV1 {
        match result {
            WorkspaceSettingsLoadV1::Blocked { reason, .. } => reason,
            _ => panic!("expected blocked settings load"),
        }
    }

    #[test]
    #[serial_test::serial]
    fn workspace_settings_direct_nested_values_provenance_and_legacy_parity() {
        let f = settings_fixture();
        for selected in [&f.root, &f.nested] {
            let observation = settings_observed(settings_preserved(&f, selected));
            assert_eq!(observation.selected_location, *selected);
            assert_eq!(observation.root, f.root);
            assert!(observation.config.world.enabled);
            assert!(!observation.config.world.caged);
            assert_eq!(
                observation.config.sync.exclude,
                vec![".git/**", ".substrate/**", "local"]
            );
            assert_eq!(observation.policy.id, "global-policy");
            assert_eq!(observation.policy.name, "local-policy");
            assert_eq!(observation.policy.cmd_allowed, vec!["local"]);
            assert_eq!(
                observation.global_config_source,
                Some(f.home.join("config.yaml"))
            );
            assert_eq!(
                observation.global_policy_source,
                Some(f.home.join("policy.yaml"))
            );
            assert_eq!(
                observation.workspace_config_source,
                f.root.join(".substrate/workspace.yaml")
            );
            assert_eq!(
                observation.workspace_policy_source,
                Some(f.root.join(".substrate/policy.yaml"))
            );
            let explain = serde_json::to_value(&observation.config_provenance).unwrap();
            assert_eq!(
                explain["keys"]["world.enabled"]["sources"][0]["layer"],
                "workspace_patch"
            );
            assert_eq!(
                explain["keys"]["world.caged"]["sources"][0]["layer"],
                "global_patch"
            );
            let explain = serde_json::to_value(&observation.policy_provenance).unwrap();
            assert_eq!(
                explain["keys"]["id"]["sources"][0]["path"],
                f.home.join("policy.yaml").to_str().unwrap()
            );
            assert_eq!(
                explain["keys"]["name"]["sources"][0]["layer"],
                "workspace_patch"
            );
            let before = snapshot(f.temp.path());
            let bootstrap = f.context.authority.bootstrap_home();
            let legacy_config =
                config_model::resolve_effective_config_with_explain_for_bootstrap_home(
                    selected,
                    &CliConfigOverrides::default(),
                    &bootstrap,
                    true,
                )
                .unwrap();
            let legacy_policy =
                policy_model::resolve_effective_policy_with_explain_for_bootstrap_home(
                    selected, &bootstrap, true,
                )
                .unwrap();
            assert_eq!(observation.config, legacy_config.0);
            assert_eq!(Some(observation.config_provenance), legacy_config.1);
            assert_eq!(
                serde_json::to_value(observation.policy).unwrap(),
                serde_json::to_value(legacy_policy.0).unwrap()
            );
            assert_eq!(Some(observation.policy_provenance), legacy_policy.1);
            assert_eq!(before, snapshot(f.temp.path()));
        }
    }

    #[test]
    #[serial_test::serial]
    fn workspace_settings_fresh_reopen_uses_current_global_and_workspace_bytes() {
        let f = settings_fixture();
        let before = settings_observed(settings_preserved(&f, &f.nested));
        private_file(&f.home.join("config.yaml"), b"world: {caged: true}\n");
        private_file(
            &f.root.join(".substrate/workspace.yaml"),
            b"world: {enabled: false}\n",
        );
        private_file(&f.root.join(".substrate/policy.yaml"), b"id: reopened\n");
        let after = settings_observed(settings_preserved(&f, &f.nested));
        assert!(before.config.world.enabled);
        assert!(!before.config.world.caged);
        assert!(!after.config.world.enabled);
        assert!(after.config.world.caged);
        assert_eq!(after.policy.id, "reopened");
        assert_ne!(before.config, after.config);
    }

    #[test]
    #[serial_test::serial]
    fn workspace_settings_optional_policy_inherits_same_boundary_and_global_absence() {
        let f = settings_fixture();
        marker(f.temp.path(), b"{}\n");
        private_file(
            &f.temp.path().join(".substrate/policy.yaml"),
            b"id: wrong-ancestor\n",
        );
        fs::remove_file(f.root.join(".substrate/policy.yaml")).unwrap();
        let observation = settings_observed(settings_preserved(&f, &f.nested));
        assert_eq!(observation.policy.id, "global-policy");
        assert_eq!(observation.workspace_policy_source, None);
        for name in ["config.yaml", "policy.yaml"] {
            fs::remove_file(f.home.join(name)).unwrap();
        }
        let observation = settings_observed(settings_preserved(&f, &f.nested));
        assert_eq!(observation.global_config_source, None);
        assert_eq!(observation.global_policy_source, None);
        assert_eq!(
            observation.policy.id,
            substrate_broker::Policy::default().id
        );
        assert!(observation.config.world.enabled);
    }

    #[test]
    #[serial_test::serial]
    fn workspace_settings_nearer_boundaries_block_valid_ancestor() {
        for case in [
            "disabled",
            "malformed",
            "incomplete",
            "legacy",
            "incompatible",
        ] {
            let f = settings_fixture();
            let boundary = f.root.join("nearer");
            let selected = boundary.join("nested");
            fs::create_dir_all(&selected).unwrap();
            marker(
                &boundary,
                match case {
                    "malformed" => b"[broken",
                    "incompatible" => b"unknown: true",
                    _ => b"{}",
                },
            );
            match case {
                "disabled" => private_file(&boundary.join(".substrate/workspace.disabled"), b""),
                "legacy" => private_file(&boundary.join(".substrate/settings.yaml"), b"{}"),
                "incomplete" => {
                    fs::remove_file(boundary.join(".substrate/workspace.yaml")).unwrap()
                }
                _ => (),
            }
            match settings_preserved(&f, &selected) {
                WorkspaceSettingsLoadV1::Blocked {
                    boundary: observed,
                    reason: WorkspaceSettingsBlockerV1::Inspection(_),
                } => assert_eq!(observed, Some(boundary)),
                _ => panic!("nearer boundary fell through: {case}"),
            }
        }
    }

    #[test]
    #[serial_test::serial]
    fn workspace_settings_invalid_policy_blocks_both_gateway_and_all_policy_modes() {
        for gateway in ["in_world", "host_only"] {
            for policy_mode in ["disabled", "observe", "enforce"] {
                for global in [false, true] {
                    for invalid in [
                        b"[broken".as_slice(),
                        b"secret_unknown: DO_NOT_DISCLOSE\n",
                        b"world_fs: {write: {enabled: false}, fail_closed: {routing: false}}\n",
                    ] {
                        let f = settings_fixture();
                        private_file(&f.root.join(".substrate/workspace.yaml"), format!("llm: {{gateway: {{mode: {gateway}}}}}\npolicy: {{mode: {policy_mode}}}\n").as_bytes());
                        let path = if global {
                            f.home.join("policy.yaml")
                        } else {
                            f.root.join(".substrate/policy.yaml")
                        };
                        private_file(&path, invalid);
                        assert_eq!(
                            settings_reason(settings_preserved(&f, &f.nested)),
                            WorkspaceSettingsBlockerV1::IncompatibleConfigPolicy
                        );
                    }
                }
                println!("workspace_settings_ policy refusal gateway={gateway} policy={policy_mode}: global/workspace parse+validation blocked");
            }
        }
    }

    #[test]
    #[serial_test::serial]
    fn workspace_settings_unreadable_policy_has_real_permission_denial() {
        assert_ne!(
            unsafe { libc::geteuid() },
            0,
            "permission proof requires non-root"
        );
        for gateway in ["in_world", "host_only"] {
            for policy_mode in ["disabled", "observe", "enforce"] {
                for (global, source) in [
                    (true, SettingsSourceV1::GlobalPolicy),
                    (false, SettingsSourceV1::WorkspacePolicy),
                ] {
                    let f = settings_fixture();
                    private_file(&f.root.join(".substrate/workspace.yaml"), format!("llm: {{gateway: {{mode: {gateway}}}}}\npolicy: {{mode: {policy_mode}}}\n").as_bytes());
                    let path = if global {
                        f.home.join("policy.yaml")
                    } else {
                        f.root.join(".substrate/policy.yaml")
                    };
                    fs::set_permissions(&path, fs::Permissions::from_mode(0)).unwrap();
                    assert_eq!(
                        fs::read(&path).unwrap_err().raw_os_error(),
                        Some(libc::EACCES)
                    );
                    assert_eq!(
                        settings_reason(settings_preserved(&f, &f.nested)),
                        WorkspaceSettingsBlockerV1::Unavailable(
                            source,
                            io::ErrorKind::PermissionDenied
                        )
                    );
                    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
                }
                println!("workspace_settings_ actual EACCES blocks global/workspace policy: gateway={gateway}, policy={policy_mode}");
            }
        }
    }

    #[test]
    #[serial_test::serial]
    fn workspace_settings_alias_hardlink_fifo_and_directory_sources_refuse() {
        for source in [
            SettingsSourceV1::GlobalConfig,
            SettingsSourceV1::WorkspaceConfig,
            SettingsSourceV1::GlobalPolicy,
            SettingsSourceV1::WorkspacePolicy,
        ] {
            for kind in ["symlink", "hardlink", "fifo", "directory"] {
                let f = settings_fixture();
                let path = match source {
                    SettingsSourceV1::GlobalConfig => f.home.join("config.yaml"),
                    SettingsSourceV1::WorkspaceConfig => f.root.join(".substrate/workspace.yaml"),
                    SettingsSourceV1::GlobalPolicy => f.home.join("policy.yaml"),
                    SettingsSourceV1::WorkspacePolicy => f.root.join(".substrate/policy.yaml"),
                };
                fs::remove_file(&path).unwrap();
                let target = f.temp.path().join("other-file");
                private_file(&target, b"{}\n");
                match kind {
                    "symlink" => symlink(&target, &path).unwrap(),
                    "hardlink" => fs::hard_link(&target, &path).unwrap(),
                    "fifo" => {
                        let name = CString::new(path.as_os_str().as_bytes()).unwrap();
                        assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
                    }
                    _ => fs::create_dir(&path).unwrap(),
                }
                assert!(
                    matches!(
                        settings_preserved(&f, &f.nested),
                        WorkspaceSettingsLoadV1::Blocked { .. }
                    ),
                    "unsafe source accepted: {source:?}/{kind}"
                );
            }
        }
    }

    #[test]
    #[serial_test::serial]
    fn workspace_settings_substitutions_and_new_absences_block_after_collect() {
        use std::{cell::RefCell, rc::Rc};
        for case in [
            "global-config",
            "global-policy",
            "workspace-policy",
            "marker",
            "control",
            "root",
            "home",
            "nearer",
            "global-config-absence",
            "global-policy-absence",
            "workspace-policy-absence",
        ] {
            let f = settings_fixture();
            let target = match case {
                "global-config" | "global-config-absence" => f.home.join("config.yaml"),
                "global-policy" | "global-policy-absence" => f.home.join("policy.yaml"),
                "workspace-policy" | "workspace-policy-absence" => {
                    f.root.join(".substrate/policy.yaml")
                }
                "marker" => f.root.join(".substrate/workspace.yaml"),
                "control" => f.root.join(".substrate"),
                "root" => f.root.clone(),
                "home" => f.home.clone(),
                _ => f.nested.join(".substrate"),
            };
            if case.ends_with("absence") {
                fs::remove_file(&target).unwrap();
            }
            let captured = Rc::new(RefCell::new(None));
            let output = Rc::clone(&captured);
            let fixture_root = f.temp.path().to_path_buf();
            AFTER_SETTINGS_COLLECT.with(|hook| {
                *hook.borrow_mut() = Some(Box::new(move || {
                    if case == "nearer" {
                        fs::create_dir(&target).unwrap();
                    } else if case.ends_with("absence") {
                        private_file(&target, b"{}");
                    } else {
                        let metadata = fs::metadata(&target).unwrap();
                        let bytes = metadata.is_file().then(|| fs::read(&target).unwrap());
                        let held = target.with_extension("held-original");
                        fs::rename(&target, &held).unwrap();
                        if let Some(bytes) = bytes {
                            private_file(&target, &bytes);
                        } else {
                            fs::create_dir(&target).unwrap();
                            fs::set_permissions(&target, metadata.permissions()).unwrap();
                        }
                    }
                    *output.borrow_mut() = Some(snapshot(&fixture_root));
                }));
            });
            let result = WorkspaceOpenAdapter::load_existing(
                &f.nested,
                &f.context,
                &CliConfigOverrides::default(),
            );
            assert!(
                matches!(result, WorkspaceSettingsLoadV1::Blocked { .. }),
                "replacement accepted: {case}"
            );
            assert_eq!(
                captured.borrow().as_ref().expect("collection hook reached"),
                &snapshot(f.temp.path()),
                "reader modified fixture after test substitution: {case}"
            );
            println!("workspace_settings_ substitution {case}: blocked; post-substitution entries/content/modes unchanged");
        }
    }

    #[test]
    #[serial_test::serial]
    fn workspace_settings_unsupported_owners_ignore_plausible_fixture_identities() {
        let f = settings_fixture();
        private_file(&f.root.join(".substrate/workspace-state.v1.json"), br#"{"version":1,"workspace_ref":"workspace-real-looking","lanes":["lane-plausible"],"destinations":["world-plausible"]}"#);
        let observation = settings_observed(settings_preserved(&f, &f.nested));
        let expected = WorkspaceOwnerComponentV1::Unsupported(
            UnsupportedWorkspaceOwnerReasonV1::OwnerReaderNotImplemented,
        );
        for component in [
            observation.identity,
            observation.lanes,
            observation.destinations,
            observation.consumer_read_authorization,
        ] {
            assert_eq!(component, expected);
        }
    }

    #[test]
    #[serial_test::serial]
    fn workspace_settings_context_wrong_principal_missing_invalid_and_aliased_home_no_writes() {
        let mut f = settings_fixture();
        let current = super::super::install_bootstrap::current_unix_principal_and_home()
            .unwrap()
            .0;
        let PlatformPrincipalV1::Unix { uid, .. } = current else {
            panic!("Unix principal required");
        };
        let wrong = InstallBootstrapContextCarrierV1::from_context(
            transport_api_types::InstallBootstrapContextV1::new_unix(
                f.temp.path().join("missing-private-home").to_str().unwrap(),
                "different-user",
                uid + 1,
            )
            .unwrap(),
        )
        .unwrap();
        let before = snapshot(f.temp.path());
        assert!(matches!(
            WorkspaceNativeSettingsContextV1::from_native_invocation(&wrong),
            Err(WorkspaceSettingsContextFailureV1::PrincipalDenied)
        ));
        assert_eq!(before, snapshot(f.temp.path()));
        let missing = f.temp.path().join("missing-home");
        let invalid = f.temp.path().join("invalid-home");
        fs::create_dir(&invalid).unwrap();
        fs::set_permissions(&invalid, fs::Permissions::from_mode(0o755)).unwrap();
        let file = f.temp.path().join("file-home");
        private_file(&file, b"{}");
        let alias = f.temp.path().join("aliased-home");
        symlink(&f.home, &alias).unwrap();
        for home in [&missing, &invalid, &file, &alias] {
            let before = snapshot(f.temp.path());
            assert!(matches!(
                WorkspaceNativeSettingsContextV1::from_native_invocation(&settings_carrier(home)),
                Err(WorkspaceSettingsContextFailureV1::HomeUnsupportedOrUnavailable)
            ));
            assert_eq!(before, snapshot(f.temp.path()));
        }
        assert!(!missing.exists());
        f.context.carrier = wrong;
        let before = snapshot(f.temp.path());
        match WorkspaceOpenAdapter::load_existing(
            &f.nested.join("not-opened"),
            &f.context,
            &CliConfigOverrides::default(),
        ) {
            WorkspaceSettingsLoadV1::Blocked {
                boundary: None,
                reason:
                    WorkspaceSettingsBlockerV1::NativeContext(
                        WorkspaceSettingsContextFailureV1::PrincipalDenied,
                    ),
            } => (),
            _ => panic!("load must rebind native principal before selected-path access"),
        }
        assert_eq!(before, snapshot(f.temp.path()));
        let mut malformed = settings_carrier(&f.home);
        malformed.host_context_commitment = "invalid".into();
        let before = snapshot(f.temp.path());
        assert!(matches!(
            WorkspaceNativeSettingsContextV1::from_native_invocation(&malformed),
            Err(WorkspaceSettingsContextFailureV1::InvalidNativeContext)
        ));
        assert_eq!(before, snapshot(f.temp.path()));
    }

    #[test]
    #[serial_test::serial]
    fn workspace_settings_invalid_config_and_host_only_policy_conflict_refuse() {
        let f = settings_fixture();
        private_file(&f.home.join("config.yaml"), b"unknown: private-value\n");
        assert_eq!(
            settings_reason(settings_preserved(&f, &f.nested)),
            WorkspaceSettingsBlockerV1::InvalidSource(SettingsSourceV1::GlobalConfig)
        );
        private_file(
            &f.home.join("config.yaml"),
            b"llm: {gateway: {mode: host_only}}\n",
        );
        private_file(
            &f.home.join("policy.yaml"),
            b"llm: {fail_closed: {routing: true}}\n",
        );
        assert_eq!(
            settings_reason(settings_preserved(&f, &f.nested)),
            WorkspaceSettingsBlockerV1::IncompatibleConfigPolicy
        );
    }

    #[test]
    #[serial_test::serial]
    fn workspace_settings_absence_and_selected_alias_remain_distinct() {
        let f = settings_fixture();
        let empty = f.temp.path().join("empty");
        fs::create_dir(&empty).unwrap();
        match settings_preserved(&f, &empty) {
            WorkspaceSettingsLoadV1::Absent { selected_location } => {
                assert_eq!(selected_location, empty)
            }
            _ => panic!("empty location must be absent"),
        }
        let alias = f.temp.path().join("alias");
        symlink(&f.root, &alias).unwrap();
        assert_eq!(
            settings_reason(settings_preserved(&f, &alias)),
            WorkspaceSettingsBlockerV1::Inspection(WorkspaceInspectionBlocker::UnsafePath)
        );
    }

    struct SettingsEnvGuard {
        key: &'static str,
        previous: Option<OsString>,
    }
    impl SettingsEnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }
    impl Drop for SettingsEnvGuard {
        fn drop(&mut self) {
            if let Some(value) = &self.previous {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    #[test]
    #[serial_test::serial]
    fn workspace_settings_native_load_captures_env_and_cli_once() {
        let f = settings_fixture();
        let _environment = SettingsEnvGuard::set("SUBSTRATE_OVERRIDE_WORLD", "disabled");
        let before = snapshot(f.temp.path());
        AFTER_SETTINGS_COLLECT.with(|hook| {
            *hook.borrow_mut() = Some(Box::new(|| {
                std::env::set_var("SUBSTRATE_OVERRIDE_WORLD", "invalid-after-capture");
            }))
        });
        let cli = CliConfigOverrides {
            world_enabled: Some(true),
            ..Default::default()
        };
        let observed = settings_observed(WorkspaceOpenAdapter::load_existing(
            &f.nested, &f.context, &cli,
        ));
        assert!(observed.config.world.enabled);
        let provenance = serde_json::to_value(&observed.config_provenance).unwrap();
        assert_eq!(
            provenance["keys"]["world.enabled"]["sources"][0]["layer"],
            "cli_flag"
        );
        let replay = config_model::resolve_effective_config_from_sources_v1(
            &f.home.join("config.yaml"),
            Some(&fs::read(f.home.join("config.yaml")).unwrap()),
            &observed.workspace_config_source,
            &fs::read(&observed.workspace_config_source).unwrap(),
            &observed.captured_overrides,
            &observed.policy,
        )
        .unwrap();
        assert_eq!(observed.config, replay.0);
        assert_eq!(observed.config_provenance, replay.1);
        assert_eq!(before, snapshot(f.temp.path()));
    }

    #[test]
    #[serial_test::serial]
    fn workspace_settings_invalid_override_is_typed_and_effect_free() {
        let f = settings_fixture();
        let _environment =
            SettingsEnvGuard::set("SUBSTRATE_OVERRIDE_WORLD", "private-invalid-value");
        assert_eq!(
            settings_reason(settings_preserved(&f, &f.nested)),
            WorkspaceSettingsBlockerV1::InvalidOverrides
        );
    }
}
