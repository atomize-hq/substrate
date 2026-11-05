//! Strict Linux Workspace discovery, native owner load, approved absent initialization,
//! and exact no-send observations. Internal settings observation alone is not eligibility.
//! The preflight composes genuine per-World association, current HSA and backend custody
//! with bounded owner revision revalidation. No returned observation grants later use.

#![allow(
    dead_code,
    reason = "P1-A discovery precedes native owner-backed load adoption"
)]

use super::agent_runtime::host_session_authority::{
    schema::DirectoryPhysicalIdentityV1, trusted_fs::TrustedAuthorityRoot,
};
use super::agent_runtime::workspace_admission_authority::{
    self as owner, DirectoryBindingV1, HeldWorkspaceOwnerBoundaryV1, NativeOwnerBindingV1,
    NativeOwnerV1, PublicationEffectsV1, WorkspaceIdentityObservationV1, WorkspaceOwnerBlockerV1,
    WorkspaceOwnerPublicationV1,
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
    peer_authority: Option<(
        u32,
        std::sync::Arc<config_projection::ConfiguredAcceptedHomeAuthorityV1>,
    )>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkspaceSettingsContextFailureV1 {
    InvalidNativeContext,
    PrincipalDenied,
    HomeUnsupportedOrUnavailable,
}

impl WorkspaceNativeSettingsContextV1 {
    pub(crate) fn owner_binding(
        &self,
    ) -> Result<NativeOwnerBindingV1, WorkspaceSettingsContextFailureV1> {
        self.revalidate()?;
        let PlatformPrincipalV1::Unix { uid, account } =
            &self.carrier.context.intended_host_principal
        else {
            return Err(WorkspaceSettingsContextFailureV1::PrincipalDenied);
        };
        let bootstrap = self.authority.bootstrap_home();
        let home = bootstrap
            .identity()
            .map_err(|_| WorkspaceSettingsContextFailureV1::HomeUnsupportedOrUnavailable)?;
        let DirectoryPhysicalIdentityV1::Linux { device_id, inode } = home.physical_identity else {
            return Err(WorkspaceSettingsContextFailureV1::HomeUnsupportedOrUnavailable);
        };
        Ok(NativeOwnerBindingV1 {
            owner: NativeOwnerV1 {
                uid: *uid,
                account: account.clone(),
            },
            home: DirectoryBindingV1 {
                physical_path: home.physical_path.clone(),
                device_id,
                inode,
            },
        })
    }

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
            peer_authority: None,
        })
    }

    fn revalidate(&self) -> Result<(), WorkspaceSettingsContextFailureV1> {
        if let Some((uid, configured)) = &self.peer_authority {
            self.revalidate_peer_authority(*uid, configured)?;
        } else {
            super::install_bootstrap::bind_unix_install_bootstrap_context(&self.carrier)
                .map_err(|_| WorkspaceSettingsContextFailureV1::PrincipalDenied)?;
        }
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
    pub(crate) workspace_authority: substrate_broker::WorkspaceAuthorityEffectiveV1,
    pub(crate) global_config_source: Option<PathBuf>,
    pub(crate) global_policy_source: Option<PathBuf>,
    pub(crate) workspace_config_source: PathBuf,
    pub(crate) workspace_policy_source: Option<PathBuf>,
    pub(crate) captured_overrides: CapturedConfigOverridesV1,
    pub(crate) identity: WorkspaceIdentityObservationV1,
    pub(crate) lanes: WorkspaceOwnerComponentV1,
    pub(crate) destinations: WorkspaceOwnerComponentV1,
    pub(crate) consumer_read_authorization: WorkspaceOwnerComponentV1,
}

#[allow(
    clippy::large_enum_variant,
    reason = "retain the complete held workspace settings observation and established absent or blocked result API"
)]
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
        Self::observe_existing(selected, context, cli, false, None).0
    }

    pub(crate) fn publish_existing_owner(
        selected: &Path,
        context: &WorkspaceNativeSettingsContextV1,
        cli: &CliConfigOverrides,
    ) -> (WorkspaceSettingsLoadV1, WorkspaceOwnerPublicationV1) {
        let (settings, publication) = Self::observe_existing(selected, context, cli, true, None);
        let mut publication = publication.unwrap_or(WorkspaceOwnerPublicationV1::Failed(
            WorkspaceOwnerBlockerV1::UnsafeOwnerState,
            PublicationEffectsV1::default(),
        ));
        if !matches!(settings, WorkspaceSettingsLoadV1::SettingsObserved(_)) {
            let effects = match &publication {
                WorkspaceOwnerPublicationV1::Published(_, effects)
                | WorkspaceOwnerPublicationV1::Failed(_, effects) => *effects,
                WorkspaceOwnerPublicationV1::AlreadyPublished(_) => PublicationEffectsV1::default(),
            };
            publication = WorkspaceOwnerPublicationV1::Failed(
                WorkspaceOwnerBlockerV1::ChangedDuringRead,
                effects,
            );
        }
        (settings, publication)
    }

    fn observe_existing(
        selected: &Path,
        context: &WorkspaceNativeSettingsContextV1,
        cli: &CliConfigOverrides,
        publish: bool,
        mut operation: Option<&mut NativeWorkspaceOperationV1<'_>>,
    ) -> (WorkspaceSettingsLoadV1, Option<WorkspaceOwnerPublicationV1>) {
        let mut publication = None;
        let settings = (|| {
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
                let overrides = CapturedConfigOverridesV1::capture(cli)
                    .map_err(|_| Blocker::InvalidOverrides)?;
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
                let revalidate = || -> Result<(), WorkspaceOwnerBlockerV1> {
                    let verify = || -> SettingsResult<()> {
                        guard.revalidate().map_err(Blocker::Inspection)?;
                        context.revalidate().map_err(Blocker::NativeContext)?;
                        workspace_policy.revalidate(&control.held)?;
                        home.revalidate(&bootstrap)?;
                        global_config_guard.revalidate(&home.held)?;
                        global_config
                            .revalidate()
                            .map_err(|_| Blocker::ChangedDuringRead)?;
                        global_policy.revalidate(&home.held)
                    };
                    verify().map_err(|_| WorkspaceOwnerBlockerV1::ChangedDuringRead)
                };
                let boundary = HeldWorkspaceOwnerBoundaryV1::capture(
                    context,
                    &root,
                    &control.held,
                    guard
                        .directories
                        .iter()
                        .map(|d| (d.path.as_path(), &d.held)),
                );
                let identity = if publish {
                    let result = match boundary {
                        Ok(boundary) => {
                            owner::publish_workspace_identity(boundary, context, revalidate)
                        }
                        Err(reason) => WorkspaceOwnerPublicationV1::Failed(
                            reason,
                            PublicationEffectsV1::default(),
                        ),
                    };
                    let identity = match &result {
                        WorkspaceOwnerPublicationV1::Published(identity, _)
                        | WorkspaceOwnerPublicationV1::AlreadyPublished(identity) => {
                            WorkspaceIdentityObservationV1::OwnerStateObserved(identity.clone())
                        }
                        WorkspaceOwnerPublicationV1::Failed(reason, _) => {
                            WorkspaceIdentityObservationV1::OwnerStateBlocked(*reason)
                        }
                    };
                    publication = Some(result);
                    identity
                } else {
                    match boundary
                        .and_then(|boundary| owner::read_workspace_identity(boundary, context))
                    {
                        Ok(read) => match revalidate().and_then(|_| read.revalidate(context)) {
                            Ok(()) => read.observation.clone(),
                            Err(reason) => {
                                WorkspaceIdentityObservationV1::OwnerStateBlocked(reason)
                            }
                        },
                        Err(reason) => WorkspaceIdentityObservationV1::OwnerStateBlocked(reason),
                    }
                };
                let unsupported = WorkspaceOwnerComponentV1::Unsupported(
                    UnsupportedWorkspaceOwnerReasonV1::OwnerReaderNotImplemented,
                );
                let workspace_authority =
                    substrate_broker::resolve_workspace_authority_from_sources_v1(
                        &global_policy_path,
                        policy_bytes.as_deref(),
                        workspace_policy
                            .bytes
                            .as_deref()
                            .map(|bytes| (workspace_policy_path.as_path(), bytes)),
                    )
                    .map_err(|_| Blocker::IncompatibleConfigPolicy)?;
                let observation = WorkspaceSettingsObservationV1 {
                    selected_location: selected.to_path_buf(),
                    root,
                    config,
                    config_provenance,
                    policy,
                    policy_provenance,
                    workspace_authority,
                    global_config_source: global_config.source_bytes().map(|_| global_config_path),
                    global_policy_source: policy_bytes.as_ref().map(|_| global_policy_path),
                    workspace_config_source: marker_path,
                    workspace_policy_source: workspace_policy
                        .bytes
                        .as_ref()
                        .map(|_| workspace_policy_path),
                    captured_overrides: overrides,
                    identity,
                    lanes: unsupported,
                    destinations: unsupported,
                    consumer_read_authorization: unsupported,
                };
                if let Some(operation) = operation.as_mut() {
                    let boundary = HeldWorkspaceOwnerBoundaryV1::capture(
                        context,
                        &observation.root,
                        &control.held,
                        guard
                            .directories
                            .iter()
                            .map(|d| (d.path.as_path(), &d.held)),
                    )
                    .map_err(|_| Blocker::ChangedDuringRead)?;
                    operation(&observation, boundary, &revalidate)
                        .map_err(|_| Blocker::ChangedDuringRead)?;
                }
                Ok(Some(observation))
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
        })();
        (settings, publication)
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
        fs::set_permissions(&policy, fs::Permissions::from_mode(0o0)).unwrap();
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
            fs::set_permissions(&target, fs::Permissions::from_mode(0o0)).unwrap();
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
        fs::set_permissions(&nested, fs::Permissions::from_mode(0o0)).unwrap();
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
        for directory in [&root, &nested, &root.join(".substrate")] {
            fs::set_permissions(directory, fs::Permissions::from_mode(0o700)).unwrap();
        }
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

    #[test]
    fn test_workspace_current_native_policy_change_refuses_before_disclosure_or_transport() {
        let fixture = settings_fixture();
        private_file(
            &fixture.home.join("policy.yaml"),
            b"workspace_authority: {current_actor_read: allow}\n",
        );
        let (_, published) = WorkspaceOpenAdapter::publish_existing_owner(
            &fixture.root,
            &fixture.context,
            &CliConfigOverrides::default(),
        );
        let WorkspaceOwnerPublicationV1::Published(identity, _) = published else {
            panic!("genuine owner publication");
        };
        let carrier = settings_carrier(&fixture.home);
        let home = fixture.context.owner_binding().unwrap().home;
        let request = transport_api_types::WorkspaceCurrentRequestV1 {
            schema_version: 1,
            native_context: carrier,
            resource: transport_api_types::WorkspaceCurrentResourceV1 {
                workspace_path: fixture.root.to_str().unwrap().into(),
                workspace_ref: identity.workspace_ref,
                workspace_incarnation: identity.store_incarnation,
                orchestration_session_id: "negative-selector".into(),
                world_id: "negative-selector".into(),
                world_generation: 0,
                backend_id: "cli:codex".into(),
            },
            observation: transport_api_types::WorkspaceHeldAuthorityObservationV1 {
                authority_store_id: "negative-selector".into(),
                home_path: home.physical_path,
                home_device: home.device_id,
                home_inode: home.inode,
                root_revision: 1,
                authority_revision: 1,
                authority_record_sha256: "a".repeat(64),
                authoritative_lineage_sha256: "b".repeat(64),
            },
            operation: transport_api_types::WorkspaceCurrentOperationV1::CurrentActorRead {
                lane_ref: "negative-selector".into(),
                association_incarnation: "negative-selector".into(),
                association_revision: 1,
                backend_custody_ref: "c".repeat(64),
                expected_actor: "negative-selector".into(),
            },
        };
        let path = fixture.home.join("policy.yaml");
        let changed = path.clone();
        AFTER_SETTINGS_COLLECT.with(|hook| {
            *hook.borrow_mut() = Some(Box::new(move || {
                fs::write(
                    changed,
                    b"workspace_authority: {current_actor_read: deny}\n",
                )
                .unwrap();
            }))
        });
        let reply = run_native_workspace_current_request_v1(&request).unwrap();
        assert!(matches!(
            reply,
            transport_api_types::WorkspaceCurrentResponseV1::Refused { .. }
        ));
        assert_eq!(
            fs::read(path).unwrap(),
            b"workspace_authority: {current_actor_read: deny}\n",
            "read repaired policy"
        );
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
            // Preserve every semantic field while separately proving all three
            // skipped physical-capture fields and their exact provenance.
            assert_eq!(
                serde_json::to_value(&observation.config).unwrap(),
                serde_json::to_value(&legacy_config.0).unwrap()
            );
            assert_eq!(
                observation.config.repl.max_pty_buffered_lines_clamp,
                legacy_config.0.repl.max_pty_buffered_lines_clamp
            );
            assert!(observation.config.selection_sources.is_empty());
            assert!(observation.config.selection_preference_source.is_none());
            assert!(observation.config.selection_profiles.is_empty());
            use sha2::{Digest, Sha256};
            use std::os::unix::fs::MetadataExt;
            use transport_api_types::{
                AgentConnectionAuthV1, AgentConnectionBillingV1, AgentConnectionProfileV1,
                AgentSelectionSourceScopeV1, AgentSelectionSourceV1,
            };
            let builtin = AgentSelectionSourceV1 {
                scope: AgentSelectionSourceScopeV1::BuiltIn,
                root_identity: "substrate-agent-routes.v1".into(),
                relative_path: "config/agent-routes".into(),
                revision: "1".into(),
                sha256: "af4b5e5a2c07723d4d0b8fb592e90487d2c38e71b5501d14ff63cef4cf247874".into(),
            };
            let mut sources = vec![builtin.clone()];
            for (scope, path) in [
                (
                    AgentSelectionSourceScopeV1::Global,
                    f.home.join("config.yaml"),
                ),
                (
                    AgentSelectionSourceScopeV1::Workspace,
                    f.root.join(".substrate/workspace.yaml"),
                ),
            ] {
                let parent = path.parent().unwrap().canonicalize().unwrap();
                let metadata = fs::metadata(&parent).unwrap();
                let hash = format!("{:x}", Sha256::digest(fs::read(&path).unwrap()));
                sources.push(AgentSelectionSourceV1 {
                    scope,
                    root_identity: format!(
                        "{}:{}:{}",
                        metadata.dev(),
                        metadata.ino(),
                        parent.display()
                    ),
                    relative_path: path.file_name().unwrap().to_str().unwrap().into(),
                    revision: hash.clone(),
                    sha256: hash,
                });
            }
            assert_eq!(legacy_config.0.selection_sources.len(), 3);
            assert_eq!(legacy_config.0.selection_sources, sources);
            assert_eq!(
                legacy_config.0.selection_preference_source.as_ref(),
                Some(&builtin)
            );
            let profiles = vec![
                AgentConnectionProfileV1 {
                    id: "built_in:codex-subscription".into(),
                    revision: "1".into(),
                    provider: "openai".into(),
                    billing: AgentConnectionBillingV1::Subscription,
                    endpoint_identity: "openai-codex-subscription".into(),
                    upstream_base_url: "https://chatgpt.com/backend-api".into(),
                    auth: AgentConnectionAuthV1::CodexAccount {
                        reference: "configured-account".into(),
                    },
                    source: builtin.clone(),
                },
                AgentConnectionProfileV1 {
                    id: "built_in:claude-existing".into(),
                    revision: "1".into(),
                    provider: "anthropic".into(),
                    billing: AgentConnectionBillingV1::Api,
                    endpoint_identity: "anthropic-api".into(),
                    upstream_base_url: "https://api.anthropic.com".into(),
                    auth: AgentConnectionAuthV1::ApiKeyEnvironment {
                        name: "ANTHROPIC_API_KEY".into(),
                    },
                    source: builtin,
                },
            ];
            assert_eq!(legacy_config.0.selection_profiles.len(), 2);
            assert_eq!(legacy_config.0.selection_profiles, profiles);
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
                    fs::set_permissions(&path, fs::Permissions::from_mode(0o0)).unwrap();
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
        assert_eq!(
            observation.identity,
            WorkspaceIdentityObservationV1::OwnerStateBlocked(
                WorkspaceOwnerBlockerV1::CorruptOwnerState
            )
        );
        for component in [
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

    mod owner_tests {
        use super::*;
        use owner::{
            DurabilityEffectV1 as Durability, PublicationEffectV1 as Visibility,
            StagingEffectV1 as Staging,
        };

        fn publish(f: &SettingsFixture) -> WorkspaceOwnerPublicationV1 {
            WorkspaceOpenAdapter::publish_existing_owner(
                &f.nested,
                &f.context,
                &CliConfigOverrides::default(),
            )
            .1
        }

        fn issued(f: &SettingsFixture) -> owner::WorkspaceOwnerIdentityV1 {
            match publish(f) {
                WorkspaceOwnerPublicationV1::Published(identity, effects) => {
                    assert_eq!(effects.durability, Durability::Confirmed);
                    identity
                }
                other => panic!("publisher: {other:?}"),
            }
        }

        fn identity(f: &SettingsFixture) -> WorkspaceIdentityObservationV1 {
            settings_observed(settings_preserved(f, &f.nested)).identity
        }

        #[test]
        #[serial_test::serial]
        fn workspace_owner_settings_production_identity_and_independent_allocations() {
            let f = settings_fixture();
            assert_eq!(
                identity(&f),
                WorkspaceIdentityObservationV1::OwnerStateMissing
            );
            let a = issued(&f);
            assert_eq!(
                identity(&f),
                WorkspaceIdentityObservationV1::OwnerStateObserved(a.clone())
            );
            assert_eq!(a.state_revision, 1);
            assert_eq!(a.workspace_ref.len(), 67);
            assert_eq!(a.store_incarnation.len(), 68);
            assert_ne!(&a.workspace_ref[3..], &a.store_incarnation[4..]);
            let before = snapshot(f.temp.path());
            assert!(
                matches!(publish(&f), WorkspaceOwnerPublicationV1::AlreadyPublished(b) if b == a)
            );
            assert_eq!(before, snapshot(f.temp.path()));
            let g = settings_fixture();
            let b = issued(&g);
            assert_ne!(a.workspace_ref, b.workspace_ref);
            assert_ne!(a.store_incarnation, b.store_incarnation);
        }

        #[test]
        #[serial_test::serial]
        fn workspace_owner_interruption_effects_and_nonreconciling_reopen() {
            for phase in [
                "BeforeCreate",
                "StagingCreated",
                "PartialWrite",
                "FileSync",
                "StagingSynced",
                "BeforeRename",
                "PublishedVisible",
                "BeforeDirectorySync",
                "DirectorySynced",
            ] {
                let f = settings_fixture();
                let before = snapshot(f.temp.path());
                owner::set_publication_hook(Some(Box::new(move |current| {
                    if current == phase {
                        Err(WorkspaceOwnerBlockerV1::Unavailable)
                    } else {
                        Ok(())
                    }
                })));
                let result = publish(&f);
                owner::set_publication_hook(None);
                let WorkspaceOwnerPublicationV1::Failed(
                    WorkspaceOwnerBlockerV1::Unavailable,
                    effects,
                ) = result
                else {
                    panic!("{phase}: {result:?}");
                };
                let after = snapshot(f.temp.path());
                let visible = matches!(
                    phase,
                    "PublishedVisible" | "BeforeDirectorySync" | "DirectorySynced"
                );
                if phase == "BeforeCreate" {
                    assert_eq!(effects.staging, Staging::NotCreated);
                    assert_eq!(before, after);
                    assert_eq!(
                        identity(&f),
                        WorkspaceIdentityObservationV1::OwnerStateMissing
                    );
                } else if visible {
                    assert_eq!(effects.publication, Visibility::Visible);
                    assert_eq!(
                        effects.durability,
                        if phase == "DirectorySynced" {
                            Durability::Confirmed
                        } else {
                            Durability::Unconfirmed
                        }
                    );
                    assert!(matches!(
                        identity(&f),
                        WorkspaceIdentityObservationV1::OwnerStateObserved(_)
                    ));
                    assert!(!f.root.join(".substrate").join(owner::PENDING_FILE).exists());
                } else {
                    assert_eq!(effects.staging, Staging::Created);
                    assert_eq!(effects.publication, Visibility::NotPublished);
                    assert_eq!(
                        identity(&f),
                        WorkspaceIdentityObservationV1::OwnerStateBlocked(
                            WorkspaceOwnerBlockerV1::PartialPublication
                        )
                    );
                    assert!(!f.root.join(".substrate").join(owner::OWNER_FILE).exists());
                    let before_retry = snapshot(f.temp.path());
                    assert!(matches!(
                        publish(&f),
                        WorkspaceOwnerPublicationV1::Failed(
                            WorkspaceOwnerBlockerV1::PartialPublication,
                            _
                        )
                    ));
                    assert_eq!(before_retry, snapshot(f.temp.path()));
                }
                let mut permitted = snapshot(f.temp.path());
                permitted.remove(&PathBuf::from("workspace/.substrate").join(owner::OWNER_FILE));
                permitted.remove(&PathBuf::from("workspace/.substrate").join(owner::PENDING_FILE));
                assert_eq!(
                    permitted, before,
                    "only declared publication entry changed at {phase}"
                );
                assert_eq!(
                    after,
                    snapshot(f.temp.path()),
                    "read/retry altered interrupted state: {phase}"
                );
                println!("workspace_owner_ interruption {phase}: {effects:?}; reopen checked, no cleanup");
            }
        }

        #[test]
        #[serial_test::serial]
        fn workspace_owner_idempotent_and_read_substitution_refuse_final_disclosure() {
            for phase in ["AlreadyCollected", "ReadCollected"] {
                for object in ["record", "settings", "control", "home"] {
                    let f = settings_fixture();
                    issued(&f);
                    let path = match object {
                        "record" => f.root.join(".substrate").join(owner::OWNER_FILE),
                        "settings" => f.root.join(".substrate/policy.yaml"),
                        "control" => f.root.join(".substrate"),
                        _ => f.home.clone(),
                    };
                    owner::set_publication_hook(Some(Box::new(move |current| {
                        if current == phase {
                            if object == "settings" {
                                fs::write(&path, b"name: changed\n").unwrap();
                            } else {
                                let moved = path.with_extension("moved");
                                fs::rename(&path, &moved).unwrap();
                                if object == "record" {
                                    fs::copy(&moved, &path).unwrap();
                                } else {
                                    fs::create_dir(&path).unwrap();
                                }
                            }
                        }
                        Ok(())
                    })));
                    if phase == "AlreadyCollected" {
                        assert!(
                            matches!(publish(&f),WorkspaceOwnerPublicationV1::Failed(_,effects) if effects.staging==Staging::NotCreated)
                        );
                    } else {
                        let result = WorkspaceOpenAdapter::load_existing(
                            &f.nested,
                            &f.context,
                            &CliConfigOverrides::default(),
                        );
                        assert!(!matches!(
                            result,
                            WorkspaceSettingsLoadV1::SettingsObserved(
                                WorkspaceSettingsObservationV1 {
                                    identity: WorkspaceIdentityObservationV1::OwnerStateObserved(_),
                                    ..
                                }
                            )
                        ));
                    }
                    owner::set_publication_hook(None);
                    println!(
                        "workspace_owner_ {phase}/{object} substitution: final identity refused"
                    );
                }
            }
        }

        #[test]
        #[serial_test::serial]
        fn workspace_owner_no_replace_preserves_conflicting_writer_bytes() {
            let f = settings_fixture();
            let target = f.root.join(".substrate").join(owner::OWNER_FILE);
            owner::set_publication_hook(Some(Box::new(move |phase| {
                if phase == "BeforeRename" {
                    private_file(&target, b"foreign writer bytes");
                }
                Ok(())
            })));
            let result = publish(&f);
            owner::set_publication_hook(None);
            assert!(
                matches!(result,WorkspaceOwnerPublicationV1::Failed(_,effects) if effects.publication==Visibility::Uncertain)
            );
            assert_eq!(
                fs::read(f.root.join(".substrate").join(owner::OWNER_FILE)).unwrap(),
                b"foreign writer bytes"
            );
            assert!(f
                .root
                .join(".substrate")
                .join(owner::PENDING_FILE)
                .is_file());
            assert_eq!(
                identity(&f),
                WorkspaceIdentityObservationV1::OwnerStateBlocked(
                    WorkspaceOwnerBlockerV1::PartialPublication
                )
            );
        }

        #[test]
        #[serial_test::serial]
        fn workspace_owner_unavailable_acl_checks_refuse_without_effects() {
            let f = settings_fixture();
            let before = snapshot(f.temp.path());
            owner::set_publication_hook(Some(Box::new(|phase| {
                if phase == "AclValidation" {
                    Err(WorkspaceOwnerBlockerV1::Unavailable)
                } else {
                    Ok(())
                }
            })));
            assert_eq!(
                identity(&f),
                WorkspaceIdentityObservationV1::OwnerStateBlocked(
                    WorkspaceOwnerBlockerV1::Unavailable
                )
            );
            assert!(
                matches!(publish(&f),WorkspaceOwnerPublicationV1::Failed(WorkspaceOwnerBlockerV1::Unavailable,effects) if effects.staging==Staging::NotCreated)
            );
            owner::set_publication_hook(None);
            assert_eq!(before, snapshot(f.temp.path()));
        }

        #[test]
        #[serial_test::serial]
        fn workspace_owner_postpublish_settings_failure_retains_visible_effects() {
            for phase in ["PublishedVisible", "DirectorySynced"] {
                let f = settings_fixture();
                let policy = f.root.join(".substrate/policy.yaml");
                owner::set_publication_hook(Some(Box::new(move |current| {
                    if current == phase {
                        fs::write(&policy, b"[invalid\n").unwrap();
                    }
                    Ok(())
                })));
                let (settings, result) = WorkspaceOpenAdapter::publish_existing_owner(
                    &f.root,
                    &f.context,
                    &CliConfigOverrides::default(),
                );
                owner::set_publication_hook(None);
                let WorkspaceOwnerPublicationV1::Failed(_, effects) = result else {
                    panic!("{result:?}");
                };
                assert_eq!(effects.publication, Visibility::Visible);
                assert_eq!(effects.durability, Durability::Confirmed);
                assert!(matches!(
                    settings,
                    WorkspaceSettingsLoadV1::SettingsObserved(WorkspaceSettingsObservationV1 {
                        identity: WorkspaceIdentityObservationV1::OwnerStateBlocked(_),
                        ..
                    }) | WorkspaceSettingsLoadV1::Blocked { .. }
                ));
                assert!(f.root.join(".substrate").join(owner::OWNER_FILE).is_file());
                assert!(!f.root.join(".substrate").join(owner::PENDING_FILE).exists());
                // This test's explicitly changed policy is invalid, so no read may disclose
                // the identity until separately repaired. The read never repairs that input.
                assert!(matches!(
                    settings_preserved(&f, &f.root),
                    WorkspaceSettingsLoadV1::Blocked { .. }
                ));
            }
        }
    }
}

type NativeWorkspaceOperationV1<'a> = dyn FnMut(
        &WorkspaceSettingsObservationV1,
        HeldWorkspaceOwnerBoundaryV1,
        &dyn Fn() -> Result<(), WorkspaceOwnerBlockerV1>,
    ) -> Result<(), WorkspaceOwnerBlockerV1>
    + 'a;

/// Kernel identity from the actual accepted connection. No public constructor from a UID.
#[derive(Clone)]
pub struct WorkspaceNativePeerV1 {
    uid: u32,
}
impl WorkspaceNativePeerV1 {
    pub fn capture_linux_uds_v1(stream: &tokio::net::UnixStream) -> Result<Self, &'static str> {
        let peer = stream.peer_cred().map_err(|_| "Denied")?;
        if peer.pid().is_none() {
            return Err("Denied");
        }
        Ok(Self { uid: peer.uid() })
    }
}

impl WorkspaceNativeSettingsContextV1 {
    fn from_peer_invocation(
        carrier: &InstallBootstrapContextCarrierV1,
        peer: &WorkspaceNativePeerV1,
        configured: std::sync::Arc<config_projection::ConfiguredAcceptedHomeAuthorityV1>,
    ) -> Result<Self, WorkspaceSettingsContextFailureV1> {
        use WorkspaceSettingsContextFailureV1::*;
        carrier.validate().map_err(|_| InvalidNativeContext)?;
        configured
            .revalidate()
            .map_err(|_| HomeUnsupportedOrUnavailable)?;
        let accepted = configured.accepted_home();
        if configured.intended_uid() != u64::from(peer.uid)
            || carrier.context.host_substrate_home != accepted.physical_path
        {
            return Err(PrincipalDenied);
        }
        let root =
            TrustedAuthorityRoot::open_for_owner(Path::new(&accepted.physical_path), peer.uid)
                .map_err(|_| HomeUnsupportedOrUnavailable)?;
        let authority = HostSessionAuthority::from_trusted_root(root)
            .map_err(|_| HomeUnsupportedOrUnavailable)?;
        let context = Self {
            carrier: carrier.clone(),
            authority,
            peer_authority: Some((peer.uid, configured)),
        };
        context.revalidate()?;
        Ok(context)
    }

    fn revalidate_peer_authority(
        &self,
        peer_uid: u32,
        configured: &config_projection::ConfiguredAcceptedHomeAuthorityV1,
    ) -> Result<(), WorkspaceSettingsContextFailureV1> {
        use WorkspaceSettingsContextFailureV1::*;
        configured
            .revalidate()
            .map_err(|_| HomeUnsupportedOrUnavailable)?;
        // Observe today's installed binding afresh, independently of E3 registry/preparations.
        let fresh = config_projection::ConfiguredAcceptedHomeAuthorityV1::from_installed_bootstrap_authority()
            .map_err(|_| HomeUnsupportedOrUnavailable)?;
        if fresh.intended_uid() != u64::from(peer_uid)
            || fresh.accepted_home() != configured.accepted_home()
            || self.carrier.context.host_substrate_home != fresh.accepted_home().physical_path
        {
            return Err(PrincipalDenied);
        }
        let PlatformPrincipalV1::Unix { uid, account } =
            &self.carrier.context.intended_host_principal
        else {
            return Err(PrincipalDenied);
        };
        if *uid != peer_uid {
            return Err(PrincipalDenied);
        }
        let account = CString::new(account.as_bytes()).map_err(|_| InvalidNativeContext)?;
        let mut pwd = std::mem::MaybeUninit::<libc::passwd>::uninit();
        let mut result = std::ptr::null_mut();
        let mut buffer = vec![0u8; 16384];
        if unsafe {
            libc::getpwnam_r(
                account.as_ptr(),
                pwd.as_mut_ptr(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                &mut result,
            )
        } != 0
            || result.is_null()
            || unsafe { pwd.assume_init() }.pw_uid != peer_uid
        {
            return Err(PrincipalDenied);
        }
        let accepted = configured.accepted_home();
        let bootstrap_home = self.authority.bootstrap_home();
        let physical = &bootstrap_home
            .identity()
            .map_err(|_| HomeUnsupportedOrUnavailable)?
            .physical_identity;
        match (physical, &accepted.physical_identity) {
            (
                DirectoryPhysicalIdentityV1::Linux {
                    device_id: a,
                    inode: b,
                },
                config_projection::DirectoryPhysicalIdentityV1::Linux {
                    device_id: c,
                    inode: d,
                },
            ) if a == c && b == d => Ok(()),
            _ => Err(HomeUnsupportedOrUnavailable),
        }
    }
}

impl WorkspaceOpenAdapter {
    fn with_native_operation_v1(
        selected: &Path,
        context: &WorkspaceNativeSettingsContextV1,
        operation: &mut NativeWorkspaceOperationV1<'_>,
    ) -> WorkspaceSettingsLoadV1 {
        Self::observe_existing(
            selected,
            context,
            &CliConfigOverrides::default(),
            false,
            Some(operation),
        )
        .0
    }
}

fn current_request_matches_owner_v1(
    observation: &WorkspaceSettingsObservationV1,
    request: &transport_api_types::WorkspaceCurrentRequestV1,
) -> bool {
    matches!(&observation.identity, WorkspaceIdentityObservationV1::OwnerStateObserved(identity)
        if identity.workspace_ref == request.resource.workspace_ref
            && identity.store_incarnation == request.resource.workspace_incarnation)
        && observation.root.to_str() == Some(request.resource.workspace_path.as_str())
}

fn current_operation_allowed_v1(
    observation: &WorkspaceSettingsObservationV1,
    operation: &transport_api_types::WorkspaceCurrentOperationV1,
) -> bool {
    use substrate_broker::WorkspaceAuthorityDecisionV1::Allow;
    use transport_api_types::WorkspaceCurrentOperationV1::*;
    match operation {
        CurrentActorRead { .. } => {
            observation.workspace_authority.decisions.current_actor_read == Some(Allow)
                && observation.workspace_authority.read_source.is_some()
        }
        PublishAssociation => {
            observation
                .workspace_authority
                .decisions
                .association_publish
                == Some(Allow)
                && observation.workspace_authority.publish_source.is_some()
        }
    }
}

fn refused_current_v1(
    reason: transport_api_types::WorkspaceCurrentRefusalV1,
) -> transport_api_types::WorkspaceCurrentResponseV1 {
    transport_api_types::WorkspaceCurrentResponseV1::Refused { reason }
}

/// Source-owner service entry. The peer comes from the accepted connection; accepted home
/// comes from installed authority, not a request pathname. This function never ensures a World.
///
/// ```no_run
/// # fn read(stream: &tokio::net::UnixStream, request: &transport_api_types::WorkspaceCurrentRequestV1, actual_backend: &world::LinuxLocalBackend) {
/// if let Ok(peer) = substrate_shell::WorkspaceNativePeerV1::capture_linux_uds_v1(stream) {
///     let response = substrate_shell::workspace_current_for_peer_v1(&peer, request, actual_backend);
///     # let _ = response;
/// }
/// # }
/// ```
pub fn workspace_current_for_peer_v1(
    peer: &WorkspaceNativePeerV1,
    request: &transport_api_types::WorkspaceCurrentRequestV1,
    backend: &world::LinuxLocalBackend,
) -> transport_api_types::WorkspaceCurrentResponseV1 {
    use transport_api_types::{
        WorkspaceCurrentOperationV1 as O, WorkspaceCurrentRefusalV1 as F,
        WorkspaceCurrentResponseV1 as R,
    };
    if request.validate().is_err() {
        return refused_current_v1(F::Denied);
    }
    let configured = match config_projection::ConfiguredAcceptedHomeAuthorityV1::from_installed_bootstrap_authority() {
        Ok(value) => std::sync::Arc::new(value), Err(_) => return refused_current_v1(F::Denied),
    };
    let context = match WorkspaceNativeSettingsContextV1::from_peer_invocation(
        &request.native_context,
        peer,
        configured,
    ) {
        Ok(context) => context,
        Err(_) => return refused_current_v1(F::Denied),
    };
    let effects =
        std::cell::Cell::new(transport_api_types::WorkspaceAssociationEffectsV1::default());
    let mut output = refused_current_v1(F::Denied);
    let settings = WorkspaceOpenAdapter::with_native_operation_v1(
        Path::new(&request.resource.workspace_path),
        &context,
        &mut |observation, boundary, sources_current| {
            // Authorize before target lookup or any diagnostic revealing resource existence.
            if !current_operation_allowed_v1(observation, &request.operation) {
                return Ok(());
            }
            if !current_request_matches_owner_v1(observation, request) {
                return Ok(());
            }
            let publish = matches!(request.operation, O::PublishAssociation);
            let owner = match owner::WorkspaceAssociationOperationV1::open(
                boundary, &context, request, publish,
            ) {
                Ok(value) => value,
                Err(_) => {
                    return Ok(());
                }
            };
            if let O::CurrentActorRead {
                backend_custody_ref,
                ..
            } = &request.operation
            {
                if let Err(reason) =
                    owner.validate_read_selectors_v1(&context, request, backend_custody_ref)
                {
                    output = refused_current_v1(workspace_owner_refusal_v1(reason));
                    owner.revalidate(&context)?;
                    return Ok(());
                }
            }
            let resource_authorized = std::cell::Cell::new(!publish);
            let refusal = std::cell::Cell::new(F::Integrity);
            let result = backend.with_existing_custody_nonreconciling_v1(
                &request.resource.orchestration_session_id, &request.resource.world_id, request.resource.world_generation,
                &observation.root, |custody| {
                    let held = context.held_current_observation_v1(request);
                    context.authority.with_workspace_current_actor_nonreconciling_v1(
                        &request.resource.orchestration_session_id, &held, |resolved| {
                            context.validate_current_join_v1(request, resolved)?;
                            resource_authorized.set(true);
                            let result = if publish {
                                owner.publish(&context, custody, &effects, sources_current)
                                    .map_err(|_| super::agent_runtime::host_session_authority::facade::AuthorityFacadeError::workspace_current_refusal_v1())?
                            } else {
                                owner.read_joined(&context, request, custody, resolved)
                                    .map_err(|reason| if reason == WorkspaceOwnerBlockerV1::UnsupportedOwnerState {
                                        super::agent_runtime::host_session_authority::facade::AuthorityFacadeError::workspace_current_unsupported_v1()
                                    } else { super::agent_runtime::host_session_authority::facade::AuthorityFacadeError::workspace_current_refusal_v1() })?
                            };
                            sources_current().map_err(|_| super::agent_runtime::host_session_authority::facade::AuthorityFacadeError::workspace_current_refusal_v1())?;
                            owner.revalidate(&context).map_err(|_| super::agent_runtime::host_session_authority::facade::AuthorityFacadeError::workspace_current_refusal_v1())?;
                            custody.revalidate().map_err(|_| super::agent_runtime::host_session_authority::facade::AuthorityFacadeError::workspace_current_refusal_v1())?;
                            Ok(result)
                        }).map_err(|error| { refusal.set(error.workspace_current_category_v1()); anyhow::anyhow!("current authority refused") })
                });
            output = match result {
                Ok(value) => value,
                Err(_) if effects.get().staging_created == Some(true) => {
                    R::AssociationPublicationFailed {
                        reason: F::Integrity,
                        effects: effects.get(),
                    }
                }
                Err(error) => refused_current_v1(if !resource_authorized.get() {
                    F::Denied
                } else if error.to_string() == "custody producer unsupported" {
                    F::Unsupported
                } else {
                    refusal.get()
                }),
            };
            Ok(())
        },
    );
    if !matches!(settings, WorkspaceSettingsLoadV1::SettingsObserved(_)) {
        return if effects.get().staging_created == Some(true) {
            R::AssociationPublicationFailed {
                reason: F::Changed,
                effects: effects.get(),
            }
        } else {
            refused_current_v1(F::Denied)
        };
    }
    output
}

impl WorkspaceNativeSettingsContextV1 {
    fn held_current_observation_v1(
        &self,
        request: &transport_api_types::WorkspaceCurrentRequestV1,
    ) -> super::agent_runtime::host_session_authority::facade::AuthorityObservationV1 {
        use super::agent_runtime::host_session_authority::{
            facade::AuthorityObservationV1,
            schema::{AuthorityObjectCommitmentV1, CanonicalDirectoryV1},
        };
        let o = &request.observation;
        AuthorityObservationV1 {
            authority_store_id: o.authority_store_id.clone(),
            orchestration_session_id: request.resource.orchestration_session_id.clone(),
            bootstrap_home: CanonicalDirectoryV1 {
                physical_path: o.home_path.clone(),
                physical_identity: DirectoryPhysicalIdentityV1::Linux {
                    device_id: o.home_device,
                    inode: o.home_inode,
                },
            },
            root_revision: o.root_revision,
            authority_revision: o.authority_revision,
            authority_record_commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: o.authority_record_sha256.clone(),
            },
            authoritative_lineage_commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: o.authoritative_lineage_sha256.clone(),
            },
        }
    }

    fn validate_current_join_v1(
        &self,
        request: &transport_api_types::WorkspaceCurrentRequestV1,
        resolved: &super::agent_runtime::host_session_authority::facade::ResolvedCurrentAuthorityV1,
    ) -> Result<(), super::agent_runtime::host_session_authority::facade::AuthorityFacadeError>
    {
        use super::agent_runtime::host_session_authority::facade::AuthorityFacadeError;
        let r = &request.resource;
        let workspace = &resolved.authority.workspace_binding;
        let matches = workspace.workspace_root.physical_path == r.workspace_path
            && workspace.authority_store_id == request.observation.authority_store_id
            && workspace.authority_store_root == resolved.observation.bootstrap_home
            && resolved.caller.descriptor.backend_id == r.backend_id
            && resolved.host_attach_contract.backend_id == r.backend_id
            && resolved
                .authority
                .world_binding
                .as_ref()
                .is_some_and(|world| {
                    world.world_id == r.world_id && world.world_generation == r.world_generation
                })
            && match &request.operation {
                transport_api_types::WorkspaceCurrentOperationV1::CurrentActorRead {
                    expected_actor,
                    ..
                } => &resolved.caller.participant_id == expected_actor,
                transport_api_types::WorkspaceCurrentOperationV1::PublishAssociation => true,
            };
        if resolved.authority.world_binding.is_none() {
            return Err(AuthorityFacadeError::workspace_current_ineligible_v1());
        }
        if !matches {
            return Err(AuthorityFacadeError::workspace_current_refusal_v1());
        }
        let inspection = InspectionGuard::capture(Path::new(&r.workspace_path))
            .map_err(|_| AuthorityFacadeError::workspace_current_refusal_v1())?;
        let selected = &inspection.directories[inspection.selected];
        let expected = &workspace.workspace_root.physical_identity;
        if !matches!(expected, DirectoryPhysicalIdentityV1::Linux { device_id, inode } if *device_id == selected.identity.device && *inode == selected.identity.inode)
        {
            return Err(AuthorityFacadeError::workspace_current_refusal_v1());
        }
        self.revalidate()
            .map_err(|_| AuthorityFacadeError::workspace_current_refusal_v1())
    }
}

pub(crate) fn run_native_workspace_current_request_v1(
    request: &transport_api_types::WorkspaceCurrentRequestV1,
) -> anyhow::Result<transport_api_types::WorkspaceCurrentResponseV1> {
    use transport_api_types::{
        WorkspaceCurrentOperationV1 as O, WorkspaceCurrentRefusalV1 as F,
        WorkspaceCurrentResponseV1 as R,
    };
    request.validate().map_err(anyhow::Error::msg)?;
    let context = WorkspaceNativeSettingsContextV1::from_native_invocation(&request.native_context)
        .map_err(|_| anyhow::anyhow!("Denied"))?;
    let mut output = refused_current_v1(F::Denied);
    let settings = WorkspaceOpenAdapter::with_native_operation_v1(
        Path::new(&request.resource.workspace_path),
        &context,
        &mut |observation, boundary, sources_current| {
            if !current_operation_allowed_v1(observation, &request.operation) {
                return Ok(());
            }
            if !current_request_matches_owner_v1(observation, request) {
                return Ok(());
            }
            let after_boundary = boundary.duplicate_unlocked_v1(&context)?;
            let owner = if matches!(request.operation, O::CurrentActorRead { .. }) {
                match owner::WorkspaceAssociationOperationV1::open(
                    boundary, &context, request, false,
                ) {
                    Ok(owner) => Some(owner),
                    Err(reason) => {
                        output = refused_current_v1(workspace_owner_refusal_v1(reason));
                        return Ok(());
                    }
                }
            } else {
                None
            };
            if let Some(owner) = &owner {
                let O::CurrentActorRead {
                    backend_custody_ref,
                    ..
                } = &request.operation
                else {
                    unreachable!()
                };
                if let Err(reason) =
                    owner.validate_read_selectors_v1(&context, request, backend_custody_ref)
                {
                    output = refused_current_v1(workspace_owner_refusal_v1(reason));
                    return Ok(());
                }
            }
            drop(owner);
            let socket = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/run/substrate.sock"));
            let result = (|| -> anyhow::Result<R> {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;
                let client = transport_api_client::AgentClient::unix_socket(socket)?;
                runtime.block_on(client.workspace_current_v1(request))
            })();
            output = match result {
                Ok(response) => response,
                Err(_) if matches!(request.operation, O::PublishAssociation) => {
                    R::AssociationPublicationFailed {
                        reason: F::Changed,
                        effects: transport_api_types::WorkspaceAssociationEffectsV1::unknown(),
                    }
                }
                Err(_) => refused_current_v1(F::Unsupported),
            };
            let after = owner::WorkspaceAssociationOperationV1::open(
                after_boundary,
                &context,
                request,
                false,
            );
            let owner_current = match (&output, &after) {
                (R::CurrentActor(..), Ok(owner)) => match &request.operation {
                    O::CurrentActorRead {
                        backend_custody_ref,
                        ..
                    } => owner
                        .validate_read_selectors_v1(&context, request, backend_custody_ref)
                        .is_ok(),
                    _ => false,
                },
                (R::AssociationPublished { .. }, Ok(owner)) => {
                    owner.publication_matches_v1(&output)
                }
                (R::Refused { .. } | R::AssociationPublicationFailed { .. }, _) => true,
                _ => false,
            };
            if sources_current().is_err() || context.revalidate().is_err() || !owner_current {
                output = match &output {
                    R::AssociationPublished { effects, .. }
                    | R::AssociationPublicationFailed { effects, .. } => {
                        R::AssociationPublicationFailed {
                            reason: F::Changed,
                            effects: *effects,
                        }
                    }
                    _ => refused_current_v1(F::Changed),
                };
            }
            Ok(())
        },
    );
    if !matches!(settings, WorkspaceSettingsLoadV1::SettingsObserved(_)) {
        output = match &output {
            R::AssociationPublished { effects, .. }
            | R::AssociationPublicationFailed { effects, .. } => R::AssociationPublicationFailed {
                reason: F::Changed,
                effects: *effects,
            },
            _ => refused_current_v1(F::Denied),
        };
    }
    Ok(output)
}

fn workspace_owner_refusal_v1(
    reason: WorkspaceOwnerBlockerV1,
) -> transport_api_types::WorkspaceCurrentRefusalV1 {
    use transport_api_types::WorkspaceCurrentRefusalV1 as F;
    match reason {
        WorkspaceOwnerBlockerV1::UnsupportedOwnerState => F::Unsupported,
        WorkspaceOwnerBlockerV1::ChangedDuringRead => F::Changed,
        WorkspaceOwnerBlockerV1::BindingMismatch => F::Ineligible,
        _ => F::Integrity,
    }
}

#[derive(Clone, PartialEq, Eq)]
struct WorkspaceLocalCutV1 {
    settings: Vec<Option<super::workspace_read_cut::SettingsRevisionV1>>,
    effective: Vec<u8>,
    identity: owner::WorkspaceOwnerIdentityV1,
    association: owner::SelectedWorldAssociationV1,
}

fn local_cut_v1(
    context: &WorkspaceNativeSettingsContextV1,
    observation: &WorkspaceSettingsObservationV1,
    boundary: HeldWorkspaceOwnerBoundaryV1,
    request: &transport_api_types::WorkspaceCurrentRequestV1,
) -> anyhow::Result<WorkspaceLocalCutV1> {
    use super::workspace_read_cut as cut;
    anyhow::ensure!(
        current_operation_allowed_v1(observation, &request.operation)
            && current_request_matches_owner_v1(observation, request),
        "Denied"
    );
    let native = context
        .owner_binding()
        .map_err(|_| anyhow::anyhow!("Denied"))?;
    let home = cut::directory(Path::new(&native.home.physical_path), native.owner.uid)?;
    let control = cut::directory(&observation.root.join(".substrate"), native.owner.uid)?;
    anyhow::ensure!(
        cut::source_identity(&control, ".workspace-init.v1.pending", native.owner.uid)?.is_none(),
        "Incomplete"
    );
    anyhow::ensure!(
        cut::source_identity(
            &control,
            ".workspace-association.v1.pending",
            native.owner.uid
        )?
        .is_none(),
        "Incomplete association publication"
    );
    let settings = [
        (&home, "config.yaml"),
        (&home, "policy.yaml"),
        (&control, "workspace.yaml"),
        (&control, "policy.yaml"),
        (&control, "workspace.disabled"),
    ]
    .into_iter()
    .map(|(parent, name)| cut::optional_revision(parent, name, native.owner.uid))
    .collect::<anyhow::Result<Vec<_>>>()?;
    let WorkspaceIdentityObservationV1::OwnerStateObserved(identity) = &observation.identity else {
        anyhow::bail!("Incomplete");
    };
    let association = owner::read_selected_world_association_v1(boundary, context, request)
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let transport_api_types::WorkspaceCurrentOperationV1::CurrentActorRead {
        lane_ref,
        association_incarnation,
        association_revision,
        backend_custody_ref,
        ..
    } = &request.operation
    else {
        anyhow::bail!("Denied");
    };
    anyhow::ensure!(
        association.active
            && &association.lane_ref == lane_ref
            && &association.incarnation == association_incarnation
            && &association.revision == association_revision
            && &association.custody_ref == backend_custody_ref,
        "Ineligible"
    );
    let effective = serde_json::to_vec(&(
        &observation.config,
        &observation.policy,
        &observation.workspace_authority.decisions,
    ))?;
    Ok(WorkspaceLocalCutV1 {
        settings,
        effective,
        identity: identity.clone(),
        association,
    })
}

fn current_disclosure_allowed_v1(
    context: &WorkspaceNativeSettingsContextV1,
    request: &transport_api_types::WorkspaceCurrentRequestV1,
) -> bool {
    matches!(WorkspaceOpenAdapter::load_existing(Path::new(&request.resource.workspace_path),context,&CliConfigOverrides::default()),
        WorkspaceSettingsLoadV1::SettingsObserved(observation) if current_operation_allowed_v1(&observation,&request.operation)
            && current_request_matches_owner_v1(&observation,request))
}

// The installed registry is a consumed service input. Observe its namespace before
// opening the configured capability; do not hold its flock across any collection.
fn installed_authority_watch_v1() -> anyhow::Result<super::workspace_read_cut::ReadWatchV1> {
    let mut watch = super::workspace_read_cut::ReadWatchV1::new()?;
    let root = Path::new("/var/lib/substrate/install-bootstrap-authority-v1");
    for directory in root.ancestors().skip(1) {
        let name = root
            .strip_prefix(directory)?
            .components()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Unsupported installed authority ancestor"))?;
        watch.add(
            directory,
            &[name
                .as_os_str()
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Unsupported installed authority spelling"))?],
        )?;
    }
    watch.add(root, &["active.json", "lock", "records"])?;
    let records = root.join("records");
    // Only the selected immutable record is consumed, but conservatively observe
    // every extant record here rather than exposing its hash through a new API.
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&records)? {
        anyhow::ensure!(
            names.len() < 1024,
            "Unsupported installed authority population"
        );
        names.push(
            entry?
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("Unsupported installed record spelling"))?,
        );
    }
    watch.add(
        &records,
        &names.iter().map(String::as_str).collect::<Vec<_>>(),
    )?;
    watch.verify()?;
    Ok(watch)
}

fn cut_watch_v1(
    context: &WorkspaceNativeSettingsContextV1,
    request: &transport_api_types::WorkspaceCurrentRequestV1,
) -> anyhow::Result<super::workspace_read_cut::ReadWatchV1> {
    let mut watch = super::workspace_read_cut::ReadWatchV1::new()?;
    let native = context
        .owner_binding()
        .map_err(|_| anyhow::anyhow!("Denied"))?;
    let home = Path::new(&native.home.physical_path);
    let source_names = [
        "config.yaml",
        "policy.yaml",
        "workspace.yaml",
        "workspace.disabled",
        ".config.yaml.owner-revision.v1.json",
        ".policy.yaml.owner-revision.v1.json",
        ".workspace.yaml.owner-revision.v1.json",
        ".workspace.disabled.owner-revision.v1.json",
        ".config.yaml.owner-revision.v1.pending",
        ".policy.yaml.owner-revision.v1.pending",
        ".workspace.yaml.owner-revision.v1.pending",
        ".workspace.disabled.owner-revision.v1.pending",
        "workspace-state.v1.json",
        ".workspace-state.v1.pending",
        ".workspace-init.v1.pending",
        ".workspace-association.v1.pending",
        "settings.yaml",
    ];
    watch.add(home, &source_names)?;
    let selected = Path::new(&request.resource.workspace_path);
    for watched_root in [selected, home] {
        for directory in watched_root.ancestors() {
            let mut names = vec![".substrate"];
            if let Some(child) = watched_root
                .strip_prefix(directory)
                .ok()
                .and_then(|rest| rest.components().next())
            {
                if let Some(name) = child.as_os_str().to_str() {
                    names.push(name);
                }
            }
            watch.add(directory, &names)?;
        }
    }
    watch.add(&selected.join(".substrate"), &source_names)?;
    let association =
        owner::selected_association_name_v1(&request.resource.orchestration_session_id);
    watch.add(
        &selected.join(".substrate"),
        &[
            &association,
            &format!(".{association}.pending"),
            &format!(".{association}.source-cut.v1.json"),
            &format!(".{association}.source-cut.v1.pending"),
        ],
    )?;
    watch.add(&home.join("authority-v1"), &["state-root-v1.json"])?;
    Ok(watch)
}

fn cut_refusal_v1(error: &anyhow::Error) -> transport_api_types::WorkspaceCurrentRefusalV1 {
    use transport_api_types::WorkspaceCurrentRefusalV1 as F;
    let message = error.to_string();
    if message.contains("Denied") {
        F::Denied
    } else if message.contains("Unsupported") {
        F::Unsupported
    } else if message.contains("Ineligible") || message.contains("BindingMismatch") {
        F::Ineligible
    } else if message.contains("Changed") {
        F::Changed
    } else {
        F::Integrity
    }
}

fn peer_context_v1(
    peer: &WorkspaceNativePeerV1,
    request: &transport_api_types::WorkspaceCurrentRequestV1,
) -> anyhow::Result<WorkspaceNativeSettingsContextV1> {
    request.validate().map_err(anyhow::Error::msg)?;
    let configured = std::sync::Arc::new(
        config_projection::ConfiguredAcceptedHomeAuthorityV1::from_installed_bootstrap_authority()
            .map_err(|_| anyhow::anyhow!("Denied"))?,
    );
    WorkspaceNativeSettingsContextV1::from_peer_invocation(
        &request.native_context,
        peer,
        configured,
    )
    .map_err(|_| anyhow::anyhow!("Denied"))
}

/// Exact current no-send observation. Each of the two local owner collects releases
/// every lock before the next collect. No runtime is ensured or reconciled.
///
/// ```no_run
/// # fn inspect(peer: &substrate_shell::WorkspaceNativePeerV1, request: &transport_api_types::WorkspaceCurrentRequestV1, backend: &world::LinuxLocalBackend) {
/// let reply = substrate_shell::workspace_preflight_for_peer_v1(peer, request, backend);
/// # let _ = reply;
/// # }
/// ```
pub fn workspace_preflight_for_peer_v1(
    peer: &WorkspaceNativePeerV1,
    request: &transport_api_types::WorkspaceCurrentRequestV1,
    backend: &world::LinuxLocalBackend,
) -> transport_api_types::WorkspacePreflightResponseV1 {
    use transport_api_types::{
        WorkspaceCurrentOperationV1 as O, WorkspaceCurrentRefusalV1 as F,
        WorkspacePreflightResponseV1 as R,
    };
    let result = (|| -> anyhow::Result<_> {
        anyhow::ensure!(
            matches!(request.operation, O::CurrentActorRead { .. }),
            "Denied"
        );
        let mut installed_watch =
            installed_authority_watch_v1().map_err(|_| anyhow::anyhow!("Denied"))?;
        let context = peer_context_v1(peer, request)?;
        anyhow::ensure!(current_disclosure_allowed_v1(&context, request), "Denied");
        let mut watch = cut_watch_v1(&context, request).map_err(|_| anyhow::anyhow!("Denied"))?;
        let mut previous = None;
        let mut eligible = None;
        for _ in 0..2 {
            let mut collected = None;
            let mut failure = F::Denied;
            let settings = WorkspaceOpenAdapter::with_native_operation_v1(
                Path::new(&request.resource.workspace_path),
                &context,
                &mut |observation, boundary, sources_current| {
                    if !current_operation_allowed_v1(observation, &request.operation)
                        || !current_request_matches_owner_v1(observation, request)
                    {
                        return Ok(());
                    }
                    let local = match local_cut_v1(&context, observation, boundary, request) {
                        Ok(value) => value,
                        Err(error) => {
                            failure = cut_refusal_v1(&error);
                            return Ok(());
                        }
                    };
                    let joined = backend.with_existing_custody_nonreconciling_v1(
                        &request.resource.orchestration_session_id, &request.resource.world_id, request.resource.world_generation, &observation.root,
                        |custody| {
                            anyhow::ensure!(custody.binding_ref() == local.association.custody_ref, "Changed custody");
                            let held = context.held_current_observation_v1(request);
                            context.authority.with_workspace_current_cut_v1(&request.resource.orchestration_session_id, &held,
                                |resolved, hsa_physical| {
                                    context.validate_current_join_v1(request, resolved)?;
                                    sources_current().map_err(|_| super::agent_runtime::host_session_authority::facade::AuthorityFacadeError::workspace_current_refusal_v1())?;
                                    custody.revalidate().map_err(|_| super::agent_runtime::host_session_authority::facade::AuthorityFacadeError::workspace_current_refusal_v1())?;
                                    let (incarnation, revision) = custody.revision_observation_v1();
                                    Ok((hsa_physical, incarnation.to_owned(), revision))
                                }).map_err(|error| anyhow::anyhow!("{error:?}"))
                        });
                    match joined {
                        Ok((hsa_physical, custody_incarnation, custody_revision)) => {
                            let vector = transport_api_types::WorkspaceReadRevisionVectorV1 {
                                settings: local
                                    .settings
                                    .iter()
                                    .map(|value| value.as_ref().map(|v| v.token()))
                                    .collect(),
                                workspace_revision: local.identity.state_revision,
                                association_incarnation: local.association.incarnation.clone(),
                                association_revision: local.association.revision,
                                authority_store_id: request.observation.authority_store_id.clone(),
                                root_revision: request.observation.root_revision,
                                authority_revision: request.observation.authority_revision,
                                custody_incarnation,
                                custody_revision,
                            };
                            let O::CurrentActorRead { expected_actor, .. } = &request.operation
                            else {
                                unreachable!()
                            };
                            eligible = Some(transport_api_types::WorkspaceEligibleObservationV1 {
                                workspace_ref: local.identity.workspace_ref.clone(),
                                workspace_incarnation: local.identity.store_incarnation.clone(),
                                lane_ref: local.association.lane_ref.clone(),
                                world_id: request.resource.world_id.clone(),
                                world_generation: request.resource.world_generation,
                                backend_id: request.resource.backend_id.clone(),
                                actor_id: expected_actor.clone(),
                                revisions: vector.clone(),
                            });
                            collected = Some((local, hsa_physical, vector));
                        }
                        Err(_) => failure = F::Integrity,
                    }
                    sources_current()?;
                    Ok(())
                },
            );
            anyhow::ensure!(
                matches!(settings, WorkspaceSettingsLoadV1::SettingsObserved(_)),
                "Denied"
            );
            let current = collected.ok_or_else(|| anyhow::anyhow!("{failure:?}"))?;
            if let Some(previous) = &previous {
                anyhow::ensure!(previous == &current, "Changed owner revision vector");
            }
            previous = Some(current);
        }
        watch.verify()?;
        context
            .revalidate()
            .map_err(|_| anyhow::anyhow!("Denied"))?;
        installed_watch.verify()?;
        eligible.ok_or_else(|| anyhow::anyhow!("Integrity"))
    })();
    match result {
        Ok(value) => R::EligibleObservation(Box::new(value)),
        Err(error) => R::Refused {
            reason: cut_refusal_v1(&error),
        },
    }
}

/// Explicit authorized source-owner mutation; never invoked by open/preflight.
///
/// ```no_run
/// # fn update(peer: &substrate_shell::WorkspaceNativePeerV1, request: &transport_api_types::WorkspaceAssociationUpdateV1, backend: &world::LinuxLocalBackend) {
/// let reply = substrate_shell::workspace_association_update_for_peer_v1(peer, request, backend);
/// # let _ = reply;
/// # }
/// ```
pub fn workspace_association_update_for_peer_v1(
    peer: &WorkspaceNativePeerV1,
    update: &transport_api_types::WorkspaceAssociationUpdateV1,
    backend: &world::LinuxLocalBackend,
) -> transport_api_types::WorkspaceAssociationUpdateResponseV1 {
    use transport_api_types::{
        WorkspaceAssociationUpdateResponseV1 as R, WorkspaceCurrentOperationV1 as O,
        WorkspaceCurrentRefusalV1 as F,
    };
    let request = &update.current;
    let Ok(context) = peer_context_v1(peer, request) else {
        return R::Refused { reason: F::Denied };
    };
    if !matches!(request.operation, O::PublishAssociation) {
        return R::Refused { reason: F::Denied };
    }
    let effects =
        std::cell::Cell::new(transport_api_types::WorkspaceAssociationUpdateEffectsV1::default());
    let mut output = R::Refused { reason: F::Denied };
    let settings = WorkspaceOpenAdapter::with_native_operation_v1(
        Path::new(&request.resource.workspace_path),
        &context,
        &mut |observation, boundary, sources_current| {
            if !current_operation_allowed_v1(observation, &request.operation)
                || !current_request_matches_owner_v1(observation, request)
            {
                return Ok(());
            }
            let mut boundary = Some(boundary);
            let result = backend.with_existing_custody_nonreconciling_v1(&request.resource.orchestration_session_id,
                &request.resource.world_id, request.resource.world_generation, &observation.root, |custody| {
                    context.authority.with_workspace_current_actor_nonreconciling_v1(&request.resource.orchestration_session_id,
                        &context.held_current_observation_v1(request), |resolved| {
                            context.validate_current_join_v1(request, resolved)?;
                            owner::publish_selected_world_association_v1(boundary.take().expect("one joined operation"), &context,
                                request, update.active, custody.binding_ref(), &effects, sources_current)
                                .map_err(|_| super::agent_runtime::host_session_authority::facade::AuthorityFacadeError::workspace_current_refusal_v1())
                        }).map_err(|error| anyhow::anyhow!("{error:?}"))
                });
            output = result.unwrap_or_else(|_| R::AssociationPublicationFailed {
                reason: F::Integrity,
                effects: effects.get(),
            });
            Ok(())
        },
    );
    if !matches!(settings, WorkspaceSettingsLoadV1::SettingsObserved(_)) {
        return R::AssociationPublicationFailed {
            reason: F::Changed,
            effects: effects.get(),
        };
    }
    output
}

pub(crate) fn run_native_workspace_preflight_request_v1(
    request: &transport_api_types::WorkspaceCurrentRequestV1,
) -> transport_api_types::WorkspacePreflightResponseV1 {
    use transport_api_types::{WorkspaceCurrentRefusalV1 as F, WorkspacePreflightResponseV1 as R};
    let result = (|| -> anyhow::Result<R> {
        request.validate().map_err(anyhow::Error::msg)?;
        let context =
            WorkspaceNativeSettingsContextV1::from_native_invocation(&request.native_context)
                .map_err(|_| anyhow::anyhow!("Denied"))?;
        anyhow::ensure!(current_disclosure_allowed_v1(&context, request), "Denied");
        let mut watch = cut_watch_v1(&context, request).map_err(|_| anyhow::anyhow!("Denied"))?;
        let mut before = None;
        let mut output = None;
        let settings = WorkspaceOpenAdapter::with_native_operation_v1(
            Path::new(&request.resource.workspace_path),
            &context,
            &mut |observation, boundary, sources_current| {
                let local = match local_cut_v1(&context, observation, boundary, request) {
                    Ok(local) => local,
                    Err(error) => {
                        output = Some(R::Refused {
                            reason: cut_refusal_v1(&error),
                        });
                        return Ok(());
                    }
                };
                // local_cut returns owned observations and drops every Workspace lock.
                before = Some(local);
                let result = (|| -> anyhow::Result<R> {
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()?;
                    let socket = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("/run/substrate.sock"));
                    let client = transport_api_client::AgentClient::unix_socket(socket)?;
                    runtime.block_on(client.workspace_preflight_v1(request))
                })();
                output = Some(result.unwrap_or(R::Refused {
                    reason: F::Unsupported,
                }));
                sources_current()?;
                Ok(())
            },
        );
        anyhow::ensure!(
            matches!(settings, WorkspaceSettingsLoadV1::SettingsObserved(_)),
            "Denied"
        );
        if before.is_none() {
            return Ok(output.unwrap_or(R::Refused { reason: F::Denied }));
        }
        let mut after = None;
        let settings = WorkspaceOpenAdapter::with_native_operation_v1(
            Path::new(&request.resource.workspace_path),
            &context,
            &mut |observation, boundary, sources_current| {
                after = local_cut_v1(&context, observation, boundary, request).ok();
                sources_current()?;
                Ok(())
            },
        );
        anyhow::ensure!(
            matches!(settings, WorkspaceSettingsLoadV1::SettingsObserved(_))
                && after.is_some()
                && before == after,
            "Changed native owner cut"
        );
        watch.verify()?;
        let result = output.ok_or_else(|| anyhow::anyhow!("Integrity"))?;
        if let R::EligibleObservation(value) = &result {
            let local = after.as_ref().ok_or_else(|| anyhow::anyhow!("Integrity"))?;
            anyhow::ensure!(
                value.revisions.settings
                    == local
                        .settings
                        .iter()
                        .map(|v| v.as_ref().map(|v| v.token()))
                        .collect::<Vec<_>>()
                    && value.revisions.workspace_revision == local.identity.state_revision
                    && value.revisions.association_revision == local.association.revision
                    && value.revisions.association_incarnation == local.association.incarnation,
                "Changed cross-owner cut"
            );
        }
        Ok(result)
    })();
    result.unwrap_or_else(|error| R::Refused {
        reason: cut_refusal_v1(&error),
    })
}

fn global_open_authority_v1(
    context: &WorkspaceNativeSettingsContextV1,
) -> anyhow::Result<(Vec<u8>, substrate_broker::WorkspaceAuthorityEffectiveV1)> {
    context
        .revalidate()
        .map_err(|_| anyhow::anyhow!("Denied"))?;
    let native = context
        .owner_binding()
        .map_err(|_| anyhow::anyhow!("Denied"))?;
    let bytes = context
        .authority
        .bootstrap_home()
        .read_policy_yaml()
        .map_err(|_| anyhow::anyhow!("Denied"))?
        .unwrap_or_default();
    let decisions = substrate_broker::resolve_workspace_authority_from_sources_v1(
        &Path::new(&native.home.physical_path).join("policy.yaml"),
        (!bytes.is_empty()).then_some(bytes.as_slice()),
        None,
    )?;
    Ok((bytes, decisions))
}

fn initial_effects_v1(settings: &transport_api_types::WorkspaceInitialSettingsV1) -> Vec<String> {
    let mut effects = vec![
        ".substrate/".into(),
        ".substrate/.workspace-init.v1.pending".into(),
        ".substrate/workspace.yaml".into(),
        ".substrate/.workspace.yaml.owner-revision.v1.json".into(),
        ".substrate/.policy.yaml.owner-revision.v1.json".into(),
        ".substrate/.workspace.disabled.owner-revision.v1.json".into(),
        ".substrate/workspace-state.v1.json".into(),
        ".substrate/workspace-collections.v1.json".into(),
    ];
    if settings.policy_yaml.is_some() {
        effects.extend([".substrate/policy.yaml".into()]);
    }
    effects.push(".substrate/.workspace-state.v1.pending (transient)".into());
    effects.push(".substrate/.workspace-init.v1.pending removal on success".into());
    for source in ["workspace.yaml", "policy.yaml", "workspace.disabled"] {
        for suffix in ["owner-revision.v1.pending", "owner-revision.v1.written"] {
            effects.push(format!(".substrate/.{source}.{suffix} (transient)"));
        }
        if source == "workspace.yaml" || source == "policy.yaml" && settings.policy_yaml.is_some() {
            effects.push(format!(
                ".substrate/.{source}.owner-write.v1.pending (transient)"
            ));
        }
    }
    effects
}

fn absence_offer_v1(
    context: &WorkspaceNativeSettingsContextV1,
    selected: &Path,
    settings: transport_api_types::WorkspaceInitialSettingsV1,
) -> anyhow::Result<transport_api_types::WorkspaceInitializationOfferV1> {
    anyhow::ensure!(
        settings.config_yaml.len() <= 16384
            && settings
                .policy_yaml
                .as_ref()
                .is_none_or(|p| p.len() <= 16384),
        "Invalid settings"
    );
    parse_config_patch_yaml(
        &selected.join(".substrate/workspace.yaml"),
        &settings.config_yaml,
    )?;
    if let Some(policy) = &settings.policy_yaml {
        policy_model::parse_policy_patch_yaml(&selected.join(".substrate/policy.yaml"), policy)?;
    }
    let mut guard =
        InspectionGuard::capture(selected).map_err(|_| anyhow::anyhow!("Unsafe boundary"))?;
    anyhow::ensure!(
        guard
            .classify()
            .map_err(|_| anyhow::anyhow!("Blocked boundary"))?
            .is_none(),
        "Existing boundary"
    );
    guard
        .revalidate()
        .map_err(|_| anyhow::anyhow!("Changed boundary"))?;
    let root = guard.directories[guard.selected].identity;
    anyhow::ensure!(
        root.owner == unsafe { libc::geteuid() } && root.mode & 0o7022 == 0,
        "Denied"
    );
    let native = context
        .owner_binding()
        .map_err(|_| anyhow::anyhow!("Denied"))?;
    Ok(transport_api_types::WorkspaceInitializationOfferV1 {
        schema_version: 1,
        native_context: context.carrier.clone(),
        home_device: native.home.device_id,
        home_inode: native.home.inode,
        selected_location: selected.to_string_lossy().into_owned(),
        root_device: root.device,
        root_inode: root.inode,
        effects: initial_effects_v1(&settings),
        settings,
    })
}

/// Native owner open. No setup, history, runtime or destination effect follows this read.
///
/// ```no_run
/// # fn open(carrier: &transport_api_types::InstallBootstrapContextCarrierV1) {
/// let result = substrate_shell::workspace_open_native_v1(carrier, std::path::Path::new("/workspace"));
/// # let _ = result;
/// # }
/// ```
pub fn workspace_open_native_v1(
    carrier: &InstallBootstrapContextCarrierV1,
    selected: &Path,
) -> serde_json::Value {
    use serde_json::json;
    use substrate_broker::WorkspaceAuthorityDecisionV1::Allow;
    let result = (|| -> anyhow::Result<serde_json::Value> {
        let context = WorkspaceNativeSettingsContextV1::from_native_invocation(carrier)
            .map_err(|_| anyhow::anyhow!("Denied"))?;
        let (global_bytes, global) = global_open_authority_v1(&context)?;
        let globally_authorized =
            global.decisions.workspace_load == Some(Allow) && global.load_source.is_some();
        let mut components = json!({"availability":"Unsupported"});
        let loaded = WorkspaceOpenAdapter::with_native_operation_v1(
            selected,
            &context,
            &mut |observation, boundary, sources_current| {
                if observation.workspace_authority.decisions.workspace_load == Some(Allow) {
                    components =
                        match owner::read_current_workspace_components_v1(boundary, &context) {
                            Ok(value) => value,
                            Err(reason) => {
                                json!({"availability":"Blocked","reason":format!("{reason:?}")})
                            }
                        };
                    sources_current()?;
                }
                Ok(())
            },
        );
        let reply = match loaded {
            WorkspaceSettingsLoadV1::Absent { .. } => {
                anyhow::ensure!(globally_authorized, "Denied");
                json!({"state":"InitializationOffered", "offer": absence_offer_v1(&context, selected,
                    transport_api_types::WorkspaceInitialSettingsV1 { config_yaml: "{}\n".into(), policy_yaml: None })?})
            }
            WorkspaceSettingsLoadV1::Blocked { boundary, reason } => {
                anyhow::ensure!(globally_authorized, "Denied");
                let incomplete = boundary
                    .as_ref()
                    .and_then(|root| {
                        super::workspace_read_cut::directory(&root.join(".substrate"), unsafe {
                            libc::geteuid()
                        })
                        .ok()
                    })
                    .is_some_and(|control| {
                        super::workspace_read_cut::source_identity(
                            &control,
                            ".workspace-init.v1.pending",
                            unsafe { libc::geteuid() },
                        )
                        .ok()
                        .flatten()
                        .is_some()
                    });
                if incomplete
                    || reason
                        == WorkspaceSettingsBlockerV1::Inspection(
                            WorkspaceInspectionBlocker::MissingMarker,
                        )
                {
                    json!({"state":"Incomplete","reason":"InterruptedInitialization"})
                } else {
                    json!({"state":"Invalid", "reason":format!("{reason:?}")})
                }
            }
            WorkspaceSettingsLoadV1::SettingsObserved(observation) => {
                anyhow::ensure!(
                    observation.workspace_authority.decisions.workspace_load == Some(Allow),
                    "Denied"
                );
                let control = super::workspace_read_cut::directory(
                    &observation.root.join(".substrate"),
                    unsafe { libc::geteuid() },
                )?;
                if super::workspace_read_cut::source_identity(
                    &control,
                    ".workspace-init.v1.pending",
                    unsafe { libc::geteuid() },
                )?
                .is_some()
                {
                    json!({"state":"Incomplete", "reason":"InterruptedInitialization"})
                } else if let WorkspaceIdentityObservationV1::OwnerStateObserved(identity) =
                    observation.identity
                {
                    let native = context
                        .owner_binding()
                        .map_err(|_| anyhow::anyhow!("Denied"))?;
                    let home = super::workspace_read_cut::directory(
                        Path::new(&native.home.physical_path),
                        native.owner.uid,
                    )?;
                    let coverage = [
                        (&home, "config.yaml"),
                        (&home, "policy.yaml"),
                        (&control, "workspace.yaml"),
                        (&control, "policy.yaml"),
                        (&control, "workspace.disabled"),
                    ]
                    .into_iter()
                    .all(|(directory, name)| {
                        super::workspace_read_cut::optional_revision(
                            directory,
                            name,
                            native.owner.uid,
                        )
                        .is_ok()
                    });
                    json!({"state":"Loaded", "workspace_ref":identity.workspace_ref,"workspace_incarnation":identity.store_incarnation,
                        "config":observation.config,"config_provenance":observation.config_provenance,"policy_provenance":observation.policy_provenance,
                        "components":components,"destinations":"ExactPreflightRequired", "read_cut":if coverage { "ExactPreflightRequired" } else { "UnsupportedRevisionCoverage" }})
                } else {
                    json!({"state":"Incomplete", "reason":format!("{:?}", observation.identity)})
                }
            }
        };
        anyhow::ensure!(
            global_open_authority_v1(&context)?.0 == global_bytes,
            "Changed permission"
        );
        Ok(reply)
    })();
    result.unwrap_or_else(|error| json!({"state":"Refused", "reason":if error.to_string().contains("Denied") {"Denied"} else {"Integrity"}}))
}

/// Apply one exact absent-only approval, separately authorized from current read.
/// A failed apply retains its incomplete boundary; reopening never repairs it.
///
/// ```no_run
/// # fn init(carrier: &transport_api_types::InstallBootstrapContextCarrierV1, decision: &transport_api_types::WorkspaceInitializationDecisionV1) {
/// let result = substrate_shell::workspace_initialize_native_v1(carrier, decision);
/// # let _ = result;
/// # }
/// ```
pub fn workspace_initialize_native_v1(
    carrier: &InstallBootstrapContextCarrierV1,
    decision: &transport_api_types::WorkspaceInitializationDecisionV1,
) -> serde_json::Value {
    use super::workspace_read_cut as cut;
    use serde_json::json;
    use substrate_broker::WorkspaceAuthorityDecisionV1::Allow;
    let transport_api_types::WorkspaceInitializationDecisionV1::Approved { offer } = decision
    else {
        return json!({"state":"NotInitialized","effects":[],"decision":match decision { transport_api_types::WorkspaceInitializationDecisionV1::Denied => "Denied", _ => "Cancelled" }});
    };
    let mut effects = Vec::<String>::new();
    let mut unknown = false;
    let mut phase = "BeforeApply";
    let mut owner_effects = None;
    let result = (|| -> anyhow::Result<_> {
        let context = WorkspaceNativeSettingsContextV1::from_native_invocation(carrier)
            .map_err(|_| anyhow::anyhow!("Denied"))?;
        let (global_before, authority) = global_open_authority_v1(&context)?;
        anyhow::ensure!(
            authority.decisions.initialize == Some(Allow) && authority.initialize_source.is_some(),
            "Denied"
        );
        let selected = Path::new(&offer.selected_location);
        anyhow::ensure!(
            &absence_offer_v1(&context, selected, offer.settings.clone())? == offer.as_ref(),
            "Changed approval"
        );
        let native = context
            .owner_binding()
            .map_err(|_| anyhow::anyhow!("Denied"))?;
        let root = cut::directory(selected, native.owner.uid)?;
        let meta = root.metadata()?;
        use std::os::unix::fs::MetadataExt;
        anyhow::ensure!(
            (meta.dev(), meta.ino()) == (offer.root_device, offer.root_inode),
            "Changed root"
        );
        let mut absent_guard =
            InspectionGuard::capture(selected).map_err(|_| anyhow::anyhow!("Unsafe boundary"))?;
        let mut initialization_watch = cut::ReadWatchV1::new()?;
        let mut selected_watch = None;
        for (index, directory) in absent_guard.directories.iter().enumerate() {
            let wd = initialization_watch.add_held(&directory.held, &[".substrate"])?;
            if index == absent_guard.selected {
                anyhow::ensure!(
                    EntryIdentity::held(&root).map_err(|_| anyhow::anyhow!("Changed root"))?
                        == directory.identity,
                    "Changed selected root"
                );
                selected_watch = Some(wd);
            }
        }
        let selected_watch =
            selected_watch.ok_or_else(|| anyhow::anyhow!("Missing selected watch"))?;
        anyhow::ensure!(
            absent_guard
                .classify()
                .map_err(|_| anyhow::anyhow!("Changed boundary"))?
                .is_none(),
            "Changed boundary"
        );
        absent_guard
            .revalidate()
            .map_err(|_| anyhow::anyhow!("Changed boundary"))?;
        let global_directory =
            cut::directory(Path::new(&native.home.physical_path), native.owner.uid)?;
        let global_config_identity =
            cut::source_identity(&global_directory, "config.yaml", native.owner.uid)?;
        let global_policy_identity =
            cut::source_identity(&global_directory, "policy.yaml", native.owner.uid)?;
        let global_path = Path::new(&native.home.physical_path).join("policy.yaml");
        let local_policy_path = selected.join(".substrate/policy.yaml");
        let (policy, _) = policy_model::resolve_effective_policy_from_sources_v1(
            &global_path,
            (!global_before.is_empty()).then_some(global_before.as_slice()),
            offer
                .settings
                .policy_yaml
                .as_ref()
                .map(|p| (local_policy_path.as_path(), p.as_bytes())),
        )?;
        let bootstrap = context.authority.bootstrap_home();
        let config_bytes = bootstrap
            .read_config_yaml()
            .map_err(|_| anyhow::anyhow!("Unavailable"))?;
        config_model::resolve_effective_config_from_sources_v1(
            &Path::new(&native.home.physical_path).join("config.yaml"),
            config_bytes.as_deref(),
            &selected.join(".substrate/workspace.yaml"),
            offer.settings.config_yaml.as_bytes(),
            &CapturedConfigOverridesV1::capture(&CliConfigOverrides::default())?,
            &policy,
        )?;
        anyhow::ensure!(
            global_open_authority_v1(&context)?.0 == global_before,
            "Changed permission"
        );
        anyhow::ensure!(
            cut::source_identity(&global_directory, "config.yaml", native.owner.uid)?
                == global_config_identity
                && cut::source_identity(&global_directory, "policy.yaml", native.owner.uid)?
                    == global_policy_identity,
            "Changed initial global sources"
        );
        absent_guard
            .revalidate()
            .map_err(|_| anyhow::anyhow!("Changed boundary"))?;
        initialization_watch.verify()?;
        // Exact exclusive creation is the first mutation. No existing boundary is opened for repair.
        let name = CString::new(".substrate")?;
        if unsafe { libc::mkdirat(root.as_raw_fd(), name.as_ptr(), 0o700) } != 0 {
            let error = io::Error::last_os_error();
            unknown = matches!(error.raw_os_error(), Some(libc::EIO));
            return Err(error.into());
        }
        unknown = false;
        effects.push(".substrate/".into());
        root.sync_all()?;
        initialization_checkpoint_v1("ControlCreated")?;
        let control_path = selected.join(".substrate");
        let control = cut::child(&root, b".substrate", libc::O_RDONLY | libc::O_DIRECTORY)?;
        let control_meta = control.metadata()?;
        anyhow::ensure!(
            control_meta.dev() == meta.dev()
                && control_meta.uid() == native.owner.uid
                && control_meta.mode() & 0o7777 == 0o700,
            "Changed initialized control directory"
        );
        let control_identity = EntryIdentity::held(&control)
            .map_err(|_| anyhow::anyhow!("Changed initialized control directory"))?;
        initialization_watch.verify_created_directory_once(selected_watch, b".substrate")?;
        let initialization_watch = std::cell::RefCell::new(initialization_watch);
        let revalidate_approved = || -> Result<(), WorkspaceOwnerBlockerV1> {
            let check = || -> anyhow::Result<()> {
                initialization_watch.borrow_mut().verify()?;
                anyhow::ensure!(
                    observe(&root, OsStr::new(".substrate"))
                        .map_err(|_| anyhow::anyhow!("Changed control directory"))?
                        == Some(control_identity)
                        && EntryIdentity::held(&control)
                            .map_err(|_| anyhow::anyhow!("Changed control directory"))?
                            == control_identity,
                    "Changed control directory"
                );
                anyhow::ensure!(
                    std::fs::read_link(format!("/proc/self/fd/{}", root.as_raw_fd()))? == selected
                        && std::fs::read_link(format!("/proc/self/fd/{}", control.as_raw_fd()))?
                            == control_path,
                    "Changed initialized root"
                );
                for directory in &absent_guard.directories {
                    anyhow::ensure!(
                        EntryIdentity::held(&directory.held)
                            .map_err(|_| anyhow::anyhow!("Changed root"))?
                            == directory.identity,
                        "Changed root identity"
                    );
                }
                for entry in &absent_guard.entries {
                    if entry.directory != absent_guard.selected {
                        anyhow::ensure!(
                            observe(
                                &absent_guard.directories[entry.directory].held,
                                OsStr::new(entry.name)
                            )
                            .map_err(|_| anyhow::anyhow!("Changed ancestor"))?
                                == entry.identity,
                            "Changed ancestor"
                        );
                    }
                }
                anyhow::ensure!(
                    cut::source_identity(&global_directory, "config.yaml", native.owner.uid)?
                        == global_config_identity
                        && cut::source_identity(
                            &global_directory,
                            "policy.yaml",
                            native.owner.uid
                        )? == global_policy_identity,
                    "Changed initial global sources"
                );
                anyhow::ensure!(
                    global_open_authority_v1(&context)?.0 == global_before,
                    "Changed initialization permission"
                );
                Ok(())
            };
            check().map_err(|_| WorkspaceOwnerBlockerV1::ChangedDuringRead)
        };
        revalidate_approved().map_err(|_| anyhow::anyhow!("Changed initialized root"))?;
        phase = "PublishingInitializationMarker";
        unknown = true;
        cut::write_new(
            &control,
            ".workspace-init.v1.pending",
            b"incomplete approved initialization\n",
        )?;
        unknown = false;
        effects.push(".substrate/.workspace-init.v1.pending".into());
        control.sync_all()?;
        initialization_checkpoint_v1("PendingCreated")?;
        unknown = true;
        phase = "PublishingConfig";
        revalidate_approved().map_err(|_| anyhow::anyhow!("Changed initialized root"))?;
        cut::write_initial_settings_at(
            &control,
            &control_path,
            "workspace.yaml",
            Some(offer.settings.config_yaml.as_bytes()),
        )?;
        unknown = false;
        effects.extend([
            ".substrate/workspace.yaml".into(),
            ".substrate/.workspace.yaml.owner-revision.v1.json".into(),
        ]);
        initialization_checkpoint_v1("ConfigPublished")?;
        if let Some(policy) = &offer.settings.policy_yaml {
            unknown = true;
            phase = "PublishingPolicy";
            revalidate_approved().map_err(|_| anyhow::anyhow!("Changed initialized root"))?;
            cut::write_initial_settings_at(
                &control,
                &control_path,
                "policy.yaml",
                Some(policy.as_bytes()),
            )?;
            effects.extend([
                ".substrate/policy.yaml".into(),
                ".substrate/.policy.yaml.owner-revision.v1.json".into(),
            ]);
        }
        if offer.settings.policy_yaml.is_none() {
            phase = "PublishingPolicyAbsence";
            unknown = true;
            revalidate_approved().map_err(|_| anyhow::anyhow!("Changed initialized root"))?;
            cut::write_initial_settings_at(&control, &control_path, "policy.yaml", None)?;
            effects.push(".substrate/.policy.yaml.owner-revision.v1.json".into());
        }
        phase = "PublishingEnabledAbsence";
        unknown = true;
        revalidate_approved().map_err(|_| anyhow::anyhow!("Changed initialized root"))?;
        cut::write_initial_settings_at(&control, &control_path, "workspace.disabled", None)?;
        unknown = false;
        effects.push(".substrate/.workspace.disabled.owner-revision.v1.json".into());
        initialization_checkpoint_v1("SettingsPublished")?;
        unknown = true;
        phase = "PublishingOwner";
        revalidate_approved().map_err(|_| anyhow::anyhow!("Changed initialized root"))?;
        let boundary = HeldWorkspaceOwnerBoundaryV1::capture(
            &context,
            selected,
            &control,
            absent_guard
                .directories
                .iter()
                .map(|d| (d.path.as_path(), &d.held))
                .chain(std::iter::once((control_path.as_path(), &control))),
        )
        .map_err(|_| anyhow::anyhow!("Changed initialized boundary"))?;
        let publication =
            owner::publish_workspace_identity(boundary, &context, &revalidate_approved);
        let identity = match publication {
            WorkspaceOwnerPublicationV1::Published(identity, e) => {
                owner_effects = Some(e);
                identity
            }
            WorkspaceOwnerPublicationV1::Failed(_, e) => {
                owner_effects = Some(e);
                if e.staging == owner::StagingEffectV1::Created {
                    effects
                        .push(".substrate/.workspace-state.v1.pending (created transient)".into());
                }
                if e.publication == owner::PublicationEffectV1::Visible {
                    effects.push(".substrate/workspace-state.v1.json".into());
                }
                unknown = e.staging == owner::StagingEffectV1::Uncertain
                    || e.publication == owner::PublicationEffectV1::Uncertain;
                anyhow::bail!("Incomplete owner publication");
            }
            WorkspaceOwnerPublicationV1::AlreadyPublished(_) => {
                anyhow::bail!("Unexpected existing identity")
            }
        };
        unknown = false;
        effects.push(".substrate/workspace-state.v1.json".into());
        initialization_checkpoint_v1("OwnerPublished")?;
        unknown = true;
        phase = "PublishingEmptyCollections";
        revalidate_approved().map_err(|_| anyhow::anyhow!("Changed initialized root"))?;
        cut::write_new(
            &control,
            "workspace-collections.v1.json",
            &serde_json::to_vec(&json!({"schema_version":1,
            "workspace_ref":identity.workspace_ref,"workspace_incarnation":identity.store_incarnation,"lanes":[],"associations":[]}))?,
        )?;
        unknown = false;
        effects.push(".substrate/workspace-collections.v1.json".into());
        control.sync_all()?;
        initialization_checkpoint_v1("CollectionsPublished")?;
        for entry in &absent_guard.entries {
            if entry.directory != absent_guard.selected
                && observe(
                    &absent_guard.directories[entry.directory].held,
                    OsStr::new(entry.name),
                )
                .map_err(|_| anyhow::anyhow!("Changed ancestor"))?
                    != entry.identity
            {
                anyhow::bail!("Changed ancestor");
            }
        }
        anyhow::ensure!(
            std::fs::read_link(format!("/proc/self/fd/{}", root.as_raw_fd()))? == selected,
            "Changed root"
        );
        context
            .revalidate()
            .map_err(|_| anyhow::anyhow!("Changed native binding"))?;
        anyhow::ensure!(
            global_open_authority_v1(&context)?.0 == global_before,
            "Changed permission"
        );
        phase = "CommittingInitialization";
        revalidate_approved().map_err(|_| anyhow::anyhow!("Changed initialized root"))?;
        unknown = true;
        cut::unlink(&control, ".workspace-init.v1.pending")?;
        unknown = false;
        Ok(
            json!({"state":"Initialized","workspace_ref":identity.workspace_ref,"workspace_incarnation":identity.store_incarnation,"effects":effects,"owner_publication":owner_effects,"durability":"Confirmed"}),
        )
    })();
    result.unwrap_or_else(|error| json!({"state":"InitializationFailed","reason":if error.to_string().contains("Denied") {"Denied"} else {"Incomplete"},
        "known_effects":effects,"phase":phase,"owner_publication":owner_effects,"additional_effects_unknown":unknown,"durability":if effects.is_empty()&&!unknown{"NotApplicable"}else{"Unconfirmed"}}))
}

#[cfg(test)]
type InitializationHookV1 = Box<dyn FnMut(&str) -> anyhow::Result<()>>;

#[cfg(test)]
thread_local! {
    static INITIALIZATION_HOOK_V1: std::cell::RefCell<Option<InitializationHookV1>> = const { std::cell::RefCell::new(None) };
}
fn initialization_checkpoint_v1(phase: &str) -> anyhow::Result<()> {
    #[cfg(test)]
    return INITIALIZATION_HOOK_V1.with(|hook| match hook.borrow_mut().as_mut() {
        Some(hook) => hook(phase),
        None => Ok(()),
    });
    #[cfg(not(test))]
    {
        let _ = phase;
        Ok(())
    }
}

#[cfg(test)]
mod r09_r10_tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::fs;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use transport_api_types::{
        WorkspaceInitializationDecisionV1 as Decision, WorkspaceInitializationOfferV1 as Offer,
    };
    fn fixture(initialize: bool) -> (tempfile::TempDir, PathBuf, InstallBootstrapContextCarrierV1) {
        let temp = tempfile::tempdir_in(
            std::env::var_os("XDG_RUNTIME_DIR").expect("private native fixture parent"),
        )
        .unwrap();
        let home = temp.path().join("home");
        let selected = temp.path().join("workspace");
        for path in [&home, &selected] {
            fs::create_dir(path).unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
        }
        super::super::workspace_read_cut::write_settings(&home.join("config.yaml"), Some(b"{}\n"))
            .unwrap();
        super::super::workspace_read_cut::write_settings(
            &home.join("policy.yaml"),
            Some(if initialize {
                b"workspace_authority: {workspace_load: allow, current_actor_read: allow, initialize: allow}\n"
            } else {
                b"workspace_authority: {workspace_load: allow, current_actor_read: allow}\n"
            }),
        )
        .unwrap();
        let carrier = super::super::install_bootstrap::construct_unix_install_bootstrap_context(
            Some(&home),
            OsStr::new("substrate"),
            Path::new("/unused/substrate"),
            None,
            Path::new("/"),
        )
        .unwrap();
        (temp, selected, carrier)
    }
    type InitializationTreeSnapshotV1 = BTreeMap<PathBuf, (u32, u32, u64, u64, i64, i64, Vec<u8>)>;

    fn snapshot(root: &Path) -> InitializationTreeSnapshotV1 {
        fn walk(root: &Path, path: &Path, state: &mut InitializationTreeSnapshotV1) {
            let m = fs::symlink_metadata(path).unwrap();
            state.insert(
                path.strip_prefix(root).unwrap().to_path_buf(),
                (
                    m.mode(),
                    m.uid(),
                    m.nlink(),
                    m.len(),
                    m.mtime_nsec(),
                    m.ctime_nsec(),
                    if m.is_file() {
                        fs::read(path).unwrap()
                    } else {
                        Vec::new()
                    },
                ),
            );
            if m.is_dir() {
                for entry in fs::read_dir(path).unwrap() {
                    walk(root, &entry.unwrap().path(), state);
                }
            }
        }
        let mut state = BTreeMap::new();
        walk(root, root, &mut state);
        state
    }
    fn offer(carrier: &InstallBootstrapContextCarrierV1, selected: &Path) -> Offer {
        let value = workspace_open_native_v1(carrier, selected);
        assert_eq!(value["state"], "InitializationOffered", "{value}");
        serde_json::from_value(value["offer"].clone()).unwrap()
    }
    #[test]
    #[serial_test::serial]
    fn workspace_open_approved_absent_init_load_reopen_and_zero_read_effects() {
        let (temp, selected, carrier) = fixture(true);
        let before = snapshot(temp.path());
        let proposed = offer(&carrier, &selected);
        assert_eq!(
            snapshot(temp.path()),
            before,
            "PF-OPEN absence offer is read-only"
        );
        let initialized = workspace_initialize_native_v1(
            &carrier,
            &Decision::Approved {
                offer: Box::new(proposed),
            },
        );
        assert_eq!(initialized["state"], "Initialized", "{initialized}");
        assert!(!selected.join(".gitignore").exists());
        assert!(!selected.join(".substrate/git").exists());
        assert!(!selected
            .join(".substrate/.workspace-init.v1.pending")
            .exists());
        let after = snapshot(temp.path());
        let first = workspace_open_native_v1(&carrier, &selected);
        let second = workspace_open_native_v1(&carrier, &selected);
        assert_eq!(first["state"], "Loaded", "{first}");
        assert_eq!(first, second);
        assert_eq!(snapshot(temp.path()), after, "PF-ZERO configured reopen");
        println!(
            "PF-INIT approved + PF-OPEN configured/reopen + PF-ZERO: {} declared created entries",
            initialized["effects"].as_array().unwrap().len()
        );
    }
    #[test]
    #[serial_test::serial]
    fn workspace_open_denied_cancelled_init_and_independent_mutation_authority() {
        let (temp, selected, carrier) = fixture(false);
        let proposed = offer(&carrier, &selected);
        let before = snapshot(temp.path());
        for decision in [
            Decision::Denied,
            Decision::Cancelled,
            Decision::Approved {
                offer: Box::new(proposed),
            },
        ] {
            let reply = workspace_initialize_native_v1(&carrier, &decision);
            assert_ne!(reply["state"], "Initialized");
            assert_eq!(
                snapshot(temp.path()),
                before,
                "PF-INIT denied/cancel/read permission cannot initialize"
            );
        }
    }
    #[test]
    #[serial_test::serial]
    fn workspace_open_interrupted_initializer_never_repairs_or_reoffers() {
        for phase in [
            "ControlCreated",
            "PendingCreated",
            "ConfigPublished",
            "SettingsPublished",
            "OwnerPublished",
            "CollectionsPublished",
        ] {
            let (temp, selected, carrier) = fixture(true);
            let proposed = offer(&carrier, &selected);
            INITIALIZATION_HOOK_V1.with(|hook| {
                *hook.borrow_mut() = Some(Box::new(move |current| {
                    if current == phase {
                        anyhow::bail!("injected interruption");
                    }
                    Ok(())
                }))
            });
            let reply = workspace_initialize_native_v1(
                &carrier,
                &Decision::Approved {
                    offer: Box::new(proposed.clone()),
                },
            );
            INITIALIZATION_HOOK_V1.with(|hook| *hook.borrow_mut() = None);
            assert_eq!(reply["state"], "InitializationFailed", "{phase}: {reply}");
            let before = snapshot(temp.path());
            let reopen = workspace_open_native_v1(&carrier, &selected);
            assert!(
                matches!(reopen["state"].as_str(), Some("Incomplete" | "Invalid")),
                "{phase}: {reopen}"
            );
            assert_eq!(
                snapshot(temp.path()),
                before,
                "PF-ZERO failed init reopen {phase}"
            );
            let retry = workspace_initialize_native_v1(
                &carrier,
                &Decision::Approved {
                    offer: Box::new(proposed),
                },
            );
            assert_eq!(retry["state"], "InitializationFailed");
            assert_eq!(
                snapshot(temp.path()),
                before,
                "no reapproval/repair {phase}"
            );
            println!(
                "PF-INIT partial {phase}: known={} unknown={}",
                reply["known_effects"], reply["additional_effects_unknown"]
            );
        }
    }
    #[test]
    #[serial_test::serial]
    fn workspace_open_intervening_boundary_changed_settings_and_spoofed_context_zero() {
        for kind in [
            "boundary",
            "settings",
            "effects",
            "home-identity",
            "native-context",
            "principal",
        ] {
            let (temp, selected, mut carrier) = fixture(true);
            let mut proposed = offer(&carrier, &selected);
            match kind {
                "boundary" => fs::create_dir(selected.join(".substrate")).unwrap(),
                "settings" => proposed.settings.config_yaml = "[invalid".into(),
                "effects" => proposed.effects.clear(),
                "home-identity" => proposed.home_inode += 1,
                "native-context" => {
                    let alternate_home = temp.path().join("alternate");
                    fs::create_dir(&alternate_home).unwrap();
                    fs::set_permissions(&alternate_home, fs::Permissions::from_mode(0o700))
                        .unwrap();
                    fs::write(
                        alternate_home.join("policy.yaml"),
                        b"workspace_authority: {workspace_load: allow, current_actor_read: allow, initialize: allow}\n",
                    )
                    .unwrap();
                    carrier =
                        super::super::install_bootstrap::construct_unix_install_bootstrap_context(
                            Some(&alternate_home),
                            OsStr::new("substrate"),
                            Path::new("/unused/substrate"),
                            None,
                            Path::new("/"),
                        )
                        .unwrap();
                }
                _ => {
                    carrier.context.intended_host_principal = PlatformPrincipalV1::Unix {
                        uid: unsafe { libc::geteuid() } + 1,
                        account: "spoof".into(),
                    };
                }
            }
            let before = snapshot(temp.path());
            let reply = workspace_initialize_native_v1(
                &carrier,
                &Decision::Approved {
                    offer: Box::new(proposed),
                },
            );
            assert_eq!(reply["state"], "InitializationFailed");
            assert_eq!(snapshot(temp.path()), before, "PF-AUTH/PF-INIT {kind}");
        }
    }
    #[test]
    #[serial_test::serial]
    fn workspace_open_initializer_never_writes_substituted_selected_root() {
        let (_temp, selected, carrier) = fixture(true);
        let proposed = offer(&carrier, &selected);
        let original = selected.clone();
        let retained = selected.with_file_name("retained-approved-root");
        INITIALIZATION_HOOK_V1.with(|hook| {
            *hook.borrow_mut() = Some(Box::new(move |phase| {
                if phase == "ControlCreated" {
                    fs::rename(&original, &retained)?;
                    fs::create_dir(&original)?;
                    fs::set_permissions(&original, fs::Permissions::from_mode(0o700))?;
                }
                Ok(())
            }))
        });
        let reply = workspace_initialize_native_v1(
            &carrier,
            &Decision::Approved {
                offer: Box::new(proposed),
            },
        );
        INITIALIZATION_HOOK_V1.with(|hook| *hook.borrow_mut() = None);
        assert_eq!(reply["state"], "InitializationFailed", "{reply}");
        assert_eq!(
            fs::read_dir(&selected).unwrap().count(),
            0,
            "no write entered replacement root"
        );
        assert_eq!(
            fs::read_dir(
                selected
                    .with_file_name("retained-approved-root")
                    .join(".substrate")
            )
            .unwrap()
            .count(),
            0
        );
    }
    #[test]
    #[serial_test::serial]
    fn workspace_open_initializer_rejects_replaced_control_directory() {
        for kind in ["nonempty", "empty", "restored"] {
            let (temp, selected, carrier) = fixture(true);
            let proposed = offer(&carrier, &selected);
            let control = selected.join(".substrate");
            let retained = temp.path().join("retained-control");
            let observed = std::rc::Rc::new(std::cell::RefCell::new(None));
            let captured = observed.clone();
            let replacement = control.clone();
            INITIALIZATION_HOOK_V1.with(|hook| {
                *hook.borrow_mut() = Some(Box::new(move |phase| {
                    if phase == "ControlCreated" {
                        fs::rename(&replacement, &retained)?;
                        fs::create_dir(&replacement)?;
                        fs::set_permissions(&replacement, fs::Permissions::from_mode(0o700))?;
                        if kind == "nonempty" {
                            let path = replacement.join("workspace.yaml");
                            fs::write(&path, b"foreign configuration\n")?;
                            fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
                        } else if kind == "restored" {
                            fs::remove_dir(&replacement)?;
                            fs::rename(&retained, &replacement)?;
                        }
                        *captured.borrow_mut() = Some(snapshot(&replacement));
                    }
                    Ok(())
                }));
            });
            let reply = workspace_initialize_native_v1(
                &carrier,
                &Decision::Approved {
                    offer: Box::new(proposed),
                },
            );
            INITIALIZATION_HOOK_V1.with(|hook| *hook.borrow_mut() = None);
            assert_eq!(reply["state"], "InitializationFailed", "{kind}: {reply}");
            assert_eq!(
                snapshot(&control),
                *observed.borrow().as_ref().unwrap(),
                "{kind}"
            );
        }
    }

    #[test]
    #[serial_test::serial]
    fn workspace_open_initializer_preserves_intervening_settings() {
        for (phase, name) in [
            ("PendingCreated", "workspace.yaml"),
            ("ConfigPublished", "policy.yaml"),
            ("ConfigPublished", "workspace.disabled"),
        ] {
            let (_temp, selected, carrier) = fixture(true);
            let proposed = offer(&carrier, &selected);
            let path = selected.join(".substrate").join(name);
            let injected = path.clone();
            let observed = std::rc::Rc::new(std::cell::RefCell::new(None));
            let captured = observed.clone();
            INITIALIZATION_HOOK_V1.with(|hook| {
                *hook.borrow_mut() = Some(Box::new(move |checkpoint| {
                    if checkpoint == phase {
                        fs::write(&injected, b"intervening owner bytes\n")?;
                        fs::set_permissions(&injected, fs::Permissions::from_mode(0o600))?;
                        *captured.borrow_mut() = Some(snapshot(&injected));
                    }
                    Ok(())
                }));
            });
            let reply = workspace_initialize_native_v1(
                &carrier,
                &Decision::Approved {
                    offer: Box::new(proposed),
                },
            );
            INITIALIZATION_HOOK_V1.with(|hook| *hook.borrow_mut() = None);
            assert_eq!(reply["state"], "InitializationFailed", "{name}: {reply}");
            assert_eq!(
                snapshot(&path),
                *observed.borrow().as_ref().unwrap(),
                "{name}"
            );
        }
    }

    #[test]
    #[serial_test::serial]
    fn workspace_open_initialize_permission_does_not_regrant_read() {
        let (temp, selected, carrier) = fixture(true);
        let proposed = offer(&carrier, &selected);
        super::super::workspace_read_cut::write_settings(
            &Path::new(&carrier.context.host_substrate_home).join("policy.yaml"),
            Some(b"workspace_authority: {current_actor_read: deny, initialize: allow}\n"),
        )
        .unwrap();
        let reply = workspace_initialize_native_v1(
            &carrier,
            &Decision::Approved {
                offer: Box::new(proposed),
            },
        );
        assert_eq!(reply["state"], "Initialized", "{reply}");
        let before = snapshot(temp.path());
        let denied = workspace_open_native_v1(&carrier, &selected);
        assert_eq!(
            denied,
            serde_json::json!({"state":"Refused","reason":"Denied"})
        );
        assert_eq!(snapshot(temp.path()), before);
    }
    #[test]
    #[serial_test::serial]
    fn workspace_open_legacy_coverage_and_invalid_owner_never_enroll_or_reoffer() {
        for kind in [
            "coverage",
            "unknown-owner",
            "missing-owner",
            "disabled",
            "legacy",
        ] {
            let (temp, selected, carrier) = fixture(true);
            let proposed = offer(&carrier, &selected);
            assert_eq!(
                workspace_initialize_native_v1(
                    &carrier,
                    &Decision::Approved {
                        offer: Box::new(proposed)
                    }
                )["state"],
                "Initialized"
            );
            match kind {
                "coverage" => fs::remove_file(
                    selected.join(".substrate/.workspace.yaml.owner-revision.v1.json"),
                )
                .unwrap(),
                "unknown-owner" => {
                    let path = selected.join(".substrate/workspace-state.v1.json");
                    let mut value: serde_json::Value =
                        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                    value["version"] = 99.into();
                    fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
                }
                "missing-owner" => {
                    fs::remove_file(selected.join(".substrate/workspace-state.v1.json")).unwrap()
                }
                "disabled" => super::super::workspace_read_cut::write_settings(
                    &selected.join(".substrate/workspace.disabled"),
                    Some(b""),
                )
                .unwrap(),
                _ => fs::write(selected.join(".substrate/settings.yaml"), b"legacy").unwrap(),
            }
            let before = snapshot(temp.path());
            let reopened = workspace_open_native_v1(&carrier, &selected);
            if kind == "coverage" {
                assert_eq!(reopened["state"], "Loaded");
                assert_eq!(reopened["read_cut"], "UnsupportedRevisionCoverage");
            } else {
                assert!(
                    !matches!(
                        reopened["state"].as_str(),
                        Some("Loaded" | "InitializationOffered")
                    ),
                    "{kind}: {reopened}"
                );
            }
            assert_eq!(
                snapshot(temp.path()),
                before,
                "no metadata enrollment/repair for {kind}"
            );
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod installed_authority_cut_private_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    #[ignore = "requires exact released private Root namespace and genuine installed-home publisher"]
    fn workspace_open_installed_authority_watch_genuine_publisher_and_restoration() {
        assert_eq!(unsafe { libc::geteuid() }, 0);
        assert_eq!(
            std::env::var("TASK22_PRIVATE_NAMESPACE").unwrap(),
            "joined-v1"
        );
        let private = std::path::PathBuf::from(std::env::var_os("TASK22_PRIVATE_ROOT").unwrap());
        let product = std::path::PathBuf::from(std::env::var_os("TASK22_PRODUCT").unwrap());
        let private_parent = product.join(".codex/task20-workspace-proof/joined");
        assert_eq!(
            private.parent(),
            Some(private_parent.as_path()),
            "exact owned private source-proof case"
        );
        let first = transport_api_types::InstallBootstrapContextCarrierV1::decode(
            std::fs::read_to_string(private.join("control/carrier.txt"))
                .unwrap()
                .trim(),
        )
        .unwrap();
        assert!(
            WorkspaceNativeSettingsContextV1::from_native_invocation(&first).is_err(),
            "Root may not borrow the unprivileged native constructor"
        );
        let alternate_path = Path::new("/run/task22-owner/registry-alternate");
        std::fs::create_dir(alternate_path).unwrap();
        std::fs::set_permissions(alternate_path, std::fs::Permissions::from_mode(0o700)).unwrap();
        let name = std::ffi::CString::new(alternate_path.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::chown(name.as_ptr(), 1000, 1000) }, 0);
        let second = transport_api_types::InstallBootstrapContextCarrierV1::from_context(
            transport_api_types::InstallBootstrapContextV1::new_unix(
                alternate_path.to_str().unwrap(),
                "task22-owner",
                1000,
            )
            .unwrap(),
        )
        .unwrap();
        let publish = |carrier: &transport_api_types::InstallBootstrapContextCarrierV1| {
            let output = std::process::Command::new("/usr/bin/bash")
                .args([
                    "--noprofile",
                    "--norc",
                    "-c",
                    r#"
set -euo pipefail
source "$TASK22_PRODUCT/scripts/linux/world-lifecycle.sh"
sudo_cmd() { "$@"; }
INSTALL_BOOTSTRAP_CONTEXT_V1="$TASK22_PUBLISH_CARRIER"
INSTALL_BOOTSTRAP_COMMITMENT="$TASK22_PUBLISH_COMMITMENT"
INSTALL_BOOTSTRAP_ACCOUNT=task22-owner
INSTALL_BOOTSTRAP_UID=1000
INSTALL_BOOTSTRAP_PRIMARY_GID=1000
INSTALL_PREFIX="$TASK22_PUBLISH_PREFIX"
unset FAKE_ROOT
publish_installed_home_bootstrap_v1
"#,
                ])
                .env("TASK22_PUBLISH_CARRIER", carrier.encode().unwrap())
                .env(
                    "TASK22_PUBLISH_COMMITMENT",
                    &carrier.host_context_commitment,
                )
                .env(
                    "TASK22_PUBLISH_PREFIX",
                    &carrier.context.selected_host_prefix,
                )
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        let mut watch = installed_authority_watch_v1().unwrap();
        publish(&second);
        publish(&first);
        assert!(
            watch.verify().is_err(),
            "genuine installed-home A→B→A must invalidate cut"
        );
        let registry = Path::new("/var/lib/substrate/install-bootstrap-authority-v1");
        for mode in [
            "head-content-restore",
            "head-remove-restore",
            "directory-attributes-restore",
        ] {
            let mut watch = installed_authority_watch_v1().unwrap();
            let head = registry.join("active.json");
            let bytes = std::fs::read(&head).unwrap();
            match mode {
                "head-content-restore" => {
                    std::fs::write(&head, b"unmediated").unwrap();
                    std::fs::write(&head, &bytes).unwrap();
                }
                "head-remove-restore" => {
                    let moved = registry.join("private-proof-moved-head");
                    std::fs::rename(&head, &moved).unwrap();
                    std::fs::rename(&moved, &head).unwrap();
                }
                _ => {
                    std::fs::set_permissions(registry, std::fs::Permissions::from_mode(0o700))
                        .unwrap();
                    std::fs::set_permissions(registry, std::fs::Permissions::from_mode(0o750))
                        .unwrap();
                }
            }
            assert!(watch.verify().is_err(), "{mode}");
            assert_eq!(std::fs::read(head).unwrap(), bytes);
        }
        installed_authority_watch_v1().unwrap().verify().unwrap();
        config_projection::ConfiguredAcceptedHomeAuthorityV1::from_installed_bootstrap_authority()
            .unwrap();
        println!("PF-REV installed-home exact watch: genuine publisher A→B→A and three restoration controls; fresh accepted binding restored");
    }
}

#[cfg(all(test, target_os = "linux"))]
pub(crate) fn selected_receipt_failure_for_test_v1(
    carrier: &InstallBootstrapContextCarrierV1,
    current_request: &transport_api_types::WorkspaceCurrentRequestV1,
) -> transport_api_types::WorkspaceAssociationUpdateEffectsV1 {
    let mut request = current_request.clone();
    let transport_api_types::WorkspaceCurrentOperationV1::CurrentActorRead {
        backend_custody_ref,
        ..
    } = &request.operation
    else {
        panic!("actual genuine private read request");
    };
    let custody = backend_custody_ref.clone();
    request.operation = transport_api_types::WorkspaceCurrentOperationV1::PublishAssociation;
    let context = WorkspaceNativeSettingsContextV1::from_native_invocation(carrier).unwrap();
    let effects =
        std::cell::Cell::new(transport_api_types::WorkspaceAssociationUpdateEffectsV1::default());
    owner::set_publication_hook(Some(Box::new(|phase| {
        if phase == "SelectedAssociationPublished" {
            Err(owner::WorkspaceOwnerBlockerV1::Unavailable)
        } else {
            Ok(())
        }
    })));
    let mut attempted = false;
    let settings = WorkspaceOpenAdapter::with_native_operation_v1(
        Path::new(&request.resource.workspace_path),
        &context,
        &mut |observation, boundary, current| {
            assert_eq!(
                observation
                    .workspace_authority
                    .decisions
                    .association_publish,
                Some(substrate_broker::WorkspaceAuthorityDecisionV1::Allow)
            );
            attempted = true;
            assert!(matches!(
                owner::publish_selected_world_association_v1(
                    boundary, &context, &request, true, &custody, &effects, current
                ),
                Err(owner::WorkspaceOwnerBlockerV1::Unavailable)
            ));
            Ok(())
        },
    );
    owner::set_publication_hook(None);
    assert!(attempted && matches!(settings, WorkspaceSettingsLoadV1::SettingsObserved(_)));
    let effect = effects.get();
    assert_eq!(effect.association.published_visible, Some(true));
    assert_eq!(effect.association.directory_sync_confirmed, Some(true));
    assert_eq!(
        effect.receipt,
        transport_api_types::WorkspaceAssociationEffectsV1::default()
    );
    effect
}
