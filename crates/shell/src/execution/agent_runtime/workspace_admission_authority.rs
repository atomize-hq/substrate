//! Initial Workspace owner publication and nonreconciling native observation (Linux).
//!
//! The cooperative lock and physical bindings do not protect against malicious same-UID
//! writers, root, same-inode restores, or universal ABA. No consumer capability is issued.

use crate::execution::workspace_open::WorkspaceNativeSettingsContextV1;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::fs::File;
use std::io::{self, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};

pub(crate) const OWNER_FILE: &str = "workspace-state.v1.json";
pub(crate) const PENDING_FILE: &str = ".workspace-state.v1.pending";
const MAX_RECORD_BYTES: u64 = 16 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) enum WorkspaceOwnerBlockerV1 {
    CorruptOwnerState,
    UnsupportedOwnerState,
    BindingMismatch,
    UnsafeOwnerState,
    PartialPublication,
    PublicationInProgress,
    ChangedDuringRead,
    Unavailable,
}
type OwnerResult<T> = Result<T, WorkspaceOwnerBlockerV1>;
use WorkspaceOwnerBlockerV1 as Blocker;

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::enum_variant_names,
    reason = "retain the established explicit OwnerState observation variants used by workspace settings admission"
)]
pub(crate) enum WorkspaceIdentityObservationV1 {
    OwnerStateMissing,
    OwnerStateObserved(WorkspaceOwnerIdentityV1),
    OwnerStateBlocked(WorkspaceOwnerBlockerV1),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorkspaceOwnerIdentityV1 {
    pub(crate) workspace_ref: String,
    pub(crate) store_incarnation: String,
    pub(crate) state_revision: u64,
    pub(crate) revision_coverage: RevisionCoverageV1,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeOwnerV1 {
    pub(crate) uid: u32,
    pub(crate) account: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DirectoryBindingV1 {
    pub(crate) physical_path: String,
    pub(crate) device_id: u64,
    pub(crate) inode: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeOwnerBindingV1 {
    pub(crate) owner: NativeOwnerV1,
    pub(crate) home: DirectoryBindingV1,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FileBindingV1 {
    device_id: u64,
    inode: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RevisionCoverageV1 {
    pub(crate) config: String,
    pub(crate) policy: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WorkspaceStateV1 {
    version: u32,
    workspace_ref: String,
    store_incarnation: String,
    native_owner: NativeOwnerV1,
    product_home: DirectoryBindingV1,
    workspace_root: DirectoryBindingV1,
    control_root: DirectoryBindingV1,
    record_identity: FileBindingV1,
    state_revision: u64,
    revision_coverage: RevisionCoverageV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) enum StagingEffectV1 {
    NotCreated,
    Created,
    Uncertain,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) enum PublicationEffectV1 {
    NotPublished,
    Visible,
    Uncertain,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) enum DurabilityEffectV1 {
    NotApplicable,
    Confirmed,
    Unconfirmed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct PublicationEffectsV1 {
    pub(crate) phase: &'static str,
    pub(crate) staging: StagingEffectV1,
    pub(crate) publication: PublicationEffectV1,
    pub(crate) durability: DurabilityEffectV1,
}
impl Default for PublicationEffectsV1 {
    fn default() -> Self {
        Self {
            phase: "BeforeCreate",
            staging: StagingEffectV1::NotCreated,
            publication: PublicationEffectV1::NotPublished,
            durability: DurabilityEffectV1::NotApplicable,
        }
    }
}

#[derive(Debug)]
pub(crate) enum WorkspaceOwnerPublicationV1 {
    Published(WorkspaceOwnerIdentityV1, PublicationEffectsV1),
    AlreadyPublished(WorkspaceOwnerIdentityV1),
    Failed(WorkspaceOwnerBlockerV1, PublicationEffectsV1),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StatV1 {
    device: u64,
    inode: u64,
    mode: u32,
    uid: u32,
    gid: u32,
    links: u64,
    size: i64,
    modified: (i64, i64),
    changed: (i64, i64),
}
impl StatV1 {
    fn from_raw(s: libc::stat) -> Self {
        Self {
            device: s.st_dev,
            inode: s.st_ino,
            mode: s.st_mode,
            uid: s.st_uid,
            gid: s.st_gid,
            links: s.st_nlink,
            size: s.st_size,
            modified: (s.st_mtime, s.st_mtime_nsec),
            changed: (s.st_ctime, s.st_ctime_nsec),
        }
    }
    fn file_binding(self) -> FileBindingV1 {
        FileBindingV1 {
            device_id: self.device,
            inode: self.inode,
        }
    }
    fn same_directory(self, other: Self) -> bool {
        (self.device, self.inode, self.mode, self.uid, self.gid)
            == (other.device, other.inode, other.mode, other.uid, other.gid)
            && other.links != 0
    }
}

fn unavailable(_: impl std::fmt::Display) -> Blocker {
    Blocker::Unavailable
}

fn stat(file: &File) -> OwnerResult<StatV1> {
    let mut s = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: live descriptor and valid output pointer.
    if unsafe { libc::fstat(file.as_raw_fd(), s.as_mut_ptr()) } != 0 {
        return Err(Blocker::Unavailable);
    }
    // SAFETY: successful fstat initialized output.
    Ok(StatV1::from_raw(unsafe { s.assume_init() }))
}

fn named(directory: &File, name: &str) -> OwnerResult<Option<StatV1>> {
    let name = CString::new(name).map_err(unavailable)?;
    let mut s = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: live directory, validated single component and valid output pointer.
    if unsafe {
        libc::fstatat(
            directory.as_raw_fd(),
            name.as_ptr(),
            s.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } != 0
    {
        return if io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
            Ok(None)
        } else {
            Err(Blocker::Unavailable)
        };
    }
    // SAFETY: successful fstatat initialized output.
    Ok(Some(StatV1::from_raw(unsafe { s.assume_init() })))
}

fn open_at(directory: &File, name: &str, flags: i32) -> OwnerResult<File> {
    let name = CString::new(name).map_err(unavailable)?;
    // SAFETY: live directory and terminated component. Mode is used only with O_CREAT.
    let fd = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            flags | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK,
            0o600,
        )
    };
    if fd < 0 {
        return Err(Blocker::Unavailable);
    }
    // SAFETY: successful openat returned a new owned descriptor.
    Ok(unsafe { File::from_raw_fd(fd) })
}

fn acl_free(file: &File) -> OwnerResult<()> {
    checkpoint("AclValidation")?;
    for name in [c"system.posix_acl_access", c"system.posix_acl_default"] {
        // SAFETY: live fd, constant terminated xattr name, null size-query buffer.
        let result =
            unsafe { libc::fgetxattr(file.as_raw_fd(), name.as_ptr(), std::ptr::null_mut(), 0) };
        if result >= 0 {
            return Err(Blocker::UnsafeOwnerState);
        }
        if io::Error::last_os_error().raw_os_error() != Some(libc::ENODATA) {
            return Err(Blocker::Unavailable);
        }
    }
    Ok(())
}

fn physical_path(file: &File) -> OwnerResult<PathBuf> {
    std::fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd())).map_err(unavailable)
}

struct HeldDirectoryV1 {
    path: PathBuf,
    file: File,
    initial: StatV1,
}

pub(crate) struct HeldWorkspaceOwnerBoundaryV1 {
    directories: Vec<HeldDirectoryV1>,
    control: File,
    native: NativeOwnerBindingV1,
    root: DirectoryBindingV1,
    control_binding: DirectoryBindingV1,
}

impl HeldWorkspaceOwnerBoundaryV1 {
    pub(crate) fn capture<'a>(
        context: &WorkspaceNativeSettingsContextV1,
        root: &Path,
        control: &File,
        walk: impl Iterator<Item = (&'a Path, &'a File)>,
    ) -> OwnerResult<Self> {
        let native = context
            .owner_binding()
            .map_err(|_| Blocker::BindingMismatch)?;
        let mut directories = Vec::new();
        for (path, file) in walk {
            let s = stat(file)?;
            let exact = path == root || path == root.join(".substrate");
            if s.mode & libc::S_IFMT != libc::S_IFDIR
                || s.links == 0
                || s.mode & 0o7022 != 0
                || (s.uid != native.owner.uid && (exact || s.uid != 0))
                || (exact && s.mode & 0o700 != 0o700)
            {
                return Err(Blocker::UnsafeOwnerState);
            }
            acl_free(file)?;
            if physical_path(file)? != path {
                return Err(Blocker::BindingMismatch);
            }
            directories.push(HeldDirectoryV1 {
                path: path.to_path_buf(),
                file: file.try_clone().map_err(unavailable)?,
                initial: s,
            });
        }
        let find = |path: &Path| -> OwnerResult<DirectoryBindingV1> {
            let entry = directories
                .iter()
                .find(|d| d.path == path)
                .ok_or(Blocker::BindingMismatch)?;
            Ok(DirectoryBindingV1 {
                physical_path: path
                    .to_str()
                    .ok_or(Blocker::UnsupportedOwnerState)?
                    .to_owned(),
                device_id: entry.initial.device,
                inode: entry.initial.inode,
            })
        };
        let root = find(root)?;
        let control_binding = find(&Path::new(&root.physical_path).join(".substrate"))?;
        if root.device_id != control_binding.device_id {
            return Err(Blocker::UnsafeOwnerState);
        }
        // A new open-file description is essential: cloned descriptors share flock ownership.
        let control = open_at(control, ".", libc::O_RDONLY | libc::O_DIRECTORY)?;
        let boundary = Self {
            directories,
            control,
            native,
            root,
            control_binding,
        };
        boundary.revalidate(context)?;
        Ok(boundary)
    }

    fn revalidate(&self, context: &WorkspaceNativeSettingsContextV1) -> OwnerResult<()> {
        if context
            .owner_binding()
            .map_err(|_| Blocker::BindingMismatch)?
            != self.native
        {
            return Err(Blocker::BindingMismatch);
        }
        for d in &self.directories {
            if !d.initial.same_directory(stat(&d.file)?) || physical_path(&d.file)? != d.path {
                return Err(Blocker::ChangedDuringRead);
            }
            acl_free(&d.file)?;
            if !d.initial.same_directory(stat(&d.file)?) {
                return Err(Blocker::ChangedDuringRead);
            }
        }
        let s = stat(&self.control)?;
        if s.device != self.control_binding.device_id
            || s.inode != self.control_binding.inode
            || physical_path(&self.control)?.to_str()
                != Some(self.control_binding.physical_path.as_str())
        {
            return Err(Blocker::ChangedDuringRead);
        }
        Ok(())
    }

    fn lock(&self, exclusive: bool) -> OwnerResult<()> {
        let flags = if exclusive {
            libc::LOCK_EX
        } else {
            libc::LOCK_SH | libc::LOCK_NB
        };
        loop {
            // SAFETY: live directory descriptor; flock has no other pointer arguments.
            if unsafe { libc::flock(self.control.as_raw_fd(), flags) } == 0 {
                return Ok(());
            }
            match io::Error::last_os_error().raw_os_error() {
                Some(libc::EINTR) => continue,
                Some(libc::EWOULDBLOCK) if !exclusive => {
                    return Err(Blocker::PublicationInProgress)
                }
                _ => return Err(Blocker::Unavailable),
            }
        }
    }

    fn pending_absent(&self) -> OwnerResult<()> {
        if named(&self.control, PENDING_FILE)?.is_some() {
            Err(Blocker::PartialPublication)
        } else {
            Ok(())
        }
    }

    fn validate_file(&self, file: &File, s: StatV1) -> OwnerResult<()> {
        if s.mode & libc::S_IFMT != libc::S_IFREG
            || s.uid != self.native.owner.uid
            || s.mode & 0o7777 != 0o600
            || s.links != 1
            || s.device != self.control_binding.device_id
            || s.size < 0
            || s.size as u64 > MAX_RECORD_BYTES
        {
            return Err(Blocker::UnsafeOwnerState);
        }
        acl_free(file)?;
        if stat(file)? != s {
            return Err(Blocker::ChangedDuringRead);
        }
        Ok(())
    }

    #[allow(
        clippy::type_complexity,
        reason = "return the held file, initial stat, bytes and owner observation together at the existing workspace owner read boundary"
    )]
    fn collect(
        &self,
    ) -> OwnerResult<(
        Option<File>,
        Option<StatV1>,
        Vec<u8>,
        WorkspaceIdentityObservationV1,
    )> {
        self.pending_absent()?;
        let Some(initial) = named(&self.control, OWNER_FILE)? else {
            return Ok((
                None,
                None,
                Vec::new(),
                WorkspaceIdentityObservationV1::OwnerStateMissing,
            ));
        };
        // Reject special files and links before open; nonblocking open also closes replacement races.
        if initial.mode & libc::S_IFMT != libc::S_IFREG || initial.links != 1 {
            return Err(Blocker::UnsafeOwnerState);
        }
        let file = open_at(&self.control, OWNER_FILE, libc::O_RDONLY)?;
        self.validate_file(&file, initial)?;
        let bytes = read_bytes(&file)?;
        if stat(&file)? != initial || named(&self.control, OWNER_FILE)? != Some(initial) {
            return Err(Blocker::ChangedDuringRead);
        }
        let record: WorkspaceStateV1 =
            serde_json::from_slice(&bytes).map_err(|_| Blocker::CorruptOwnerState)?;
        if record.version != 1
            || record.state_revision != 1
            || record.revision_coverage.config != "Unsupported"
            || record.revision_coverage.policy != "Unsupported"
        {
            return Err(Blocker::UnsupportedOwnerState);
        }
        if !valid_id(&record.workspace_ref, "ws_")
            || !valid_id(&record.store_incarnation, "wsi_")
            || record.native_owner.account.is_empty()
        {
            return Err(Blocker::CorruptOwnerState);
        }
        if record.native_owner != self.native.owner
            || record.product_home != self.native.home
            || record.workspace_root != self.root
            || record.control_root != self.control_binding
            || record.record_identity != initial.file_binding()
        {
            return Err(Blocker::BindingMismatch);
        }
        let identity = WorkspaceOwnerIdentityV1 {
            workspace_ref: record.workspace_ref,
            store_incarnation: record.store_incarnation,
            state_revision: record.state_revision,
            revision_coverage: record.revision_coverage,
        };
        Ok((
            Some(file),
            Some(initial),
            bytes,
            WorkspaceIdentityObservationV1::OwnerStateObserved(identity),
        ))
    }

    fn revalidate_record(
        &self,
        file: &Option<File>,
        initial: Option<StatV1>,
        bytes: &[u8],
    ) -> OwnerResult<()> {
        self.pending_absent()?;
        if named(&self.control, OWNER_FILE)? != initial {
            return Err(Blocker::ChangedDuringRead);
        }
        if let (Some(file), Some(s)) = (file, initial) {
            self.validate_file(file, s)?;
            if read_bytes(file)? != bytes {
                return Err(Blocker::ChangedDuringRead);
            }
        }
        Ok(())
    }
}

fn valid_id(value: &str, prefix: &str) -> bool {
    value.strip_prefix(prefix).is_some_and(|v| {
        v.len() == 64
            && v.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}
fn random_id(prefix: &str) -> OwnerResult<String> {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng
        .try_fill_bytes(&mut bytes)
        .map_err(unavailable)?;
    let mut result = String::from(prefix);
    use std::fmt::Write as _;
    for b in bytes {
        write!(&mut result, "{b:02x}").map_err(unavailable)?;
    }
    Ok(result)
}
fn read_bytes(file: &File) -> OwnerResult<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut chunk = [0; 4096];
    loop {
        let count = file
            .read_at(&mut chunk, bytes.len() as u64)
            .map_err(unavailable)?;
        if count == 0 {
            return Ok(bytes);
        }
        bytes.extend_from_slice(&chunk[..count]);
        if bytes.len() as u64 > MAX_RECORD_BYTES {
            return Err(Blocker::UnsafeOwnerState);
        }
    }
}

pub(crate) struct WorkspaceOwnerReadV1 {
    boundary: HeldWorkspaceOwnerBoundaryV1,
    file: Option<File>,
    initial: Option<StatV1>,
    bytes: Vec<u8>,
    pub(crate) observation: WorkspaceIdentityObservationV1,
}
impl WorkspaceOwnerReadV1 {
    pub(crate) fn revalidate(&self, context: &WorkspaceNativeSettingsContextV1) -> OwnerResult<()> {
        self.boundary.revalidate(context)?;
        self.boundary
            .revalidate_record(&self.file, self.initial, &self.bytes)
    }
}

pub(crate) fn read_workspace_identity(
    boundary: HeldWorkspaceOwnerBoundaryV1,
    context: &WorkspaceNativeSettingsContextV1,
) -> OwnerResult<WorkspaceOwnerReadV1> {
    boundary.lock(false)?;
    boundary.revalidate(context)?;
    let (file, initial, bytes, observation) = boundary.collect()?;
    checkpoint("ReadCollected")?;
    let read = WorkspaceOwnerReadV1 {
        boundary,
        file,
        initial,
        bytes,
        observation,
    };
    read.revalidate(context)?;
    Ok(read)
}

pub(crate) fn publish_workspace_identity(
    boundary: HeldWorkspaceOwnerBoundaryV1,
    context: &WorkspaceNativeSettingsContextV1,
    mut revalidate: impl FnMut() -> OwnerResult<()>,
) -> WorkspaceOwnerPublicationV1 {
    let mut effects = PublicationEffectsV1::default();
    let result = (|| -> OwnerResult<WorkspaceOwnerPublicationV1> {
        boundary.lock(true)?;
        boundary.revalidate(context)?;
        revalidate()?;
        let (file, initial, bytes, observation) = boundary.collect()?;
        if let WorkspaceIdentityObservationV1::OwnerStateObserved(identity) = observation {
            checkpoint("AlreadyCollected")?;
            boundary.revalidate(context)?;
            revalidate()?;
            boundary.revalidate_record(&file, initial, &bytes)?;
            return Ok(WorkspaceOwnerPublicationV1::AlreadyPublished(identity));
        }
        checkpoint("BeforeCreate")?;
        boundary.revalidate(context)?;
        revalidate()?;
        effects.phase = "CreatingStaging";
        effects.staging = StagingEffectV1::Uncertain;
        let mut staging = open_at(
            &boundary.control,
            PENDING_FILE,
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
        )?;
        effects.phase = "StagingCreated";
        effects.staging = StagingEffectV1::Created;
        checkpoint("StagingCreated")?;
        let initial = stat(&staging)?;
        boundary.validate_file(&staging, initial)?;
        if named(&boundary.control, PENDING_FILE)? != Some(initial) {
            return Err(Blocker::ChangedDuringRead);
        }
        let record = WorkspaceStateV1 {
            version: 1,
            workspace_ref: random_id("ws_")?,
            store_incarnation: random_id("wsi_")?,
            native_owner: boundary.native.owner.clone(),
            product_home: boundary.native.home.clone(),
            workspace_root: boundary.root.clone(),
            control_root: boundary.control_binding.clone(),
            record_identity: initial.file_binding(),
            state_revision: 1,
            revision_coverage: RevisionCoverageV1 {
                config: "Unsupported".into(),
                policy: "Unsupported".into(),
            },
        };
        let bytes = serde_json::to_vec(&record).map_err(unavailable)?;
        let half = bytes.len() / 2;
        effects.phase = "WritingStaging";
        staging.write_all(&bytes[..half]).map_err(unavailable)?;
        checkpoint("PartialWrite")?;
        staging.write_all(&bytes[half..]).map_err(unavailable)?;
        checkpoint("FileSync")?;
        staging.sync_all().map_err(unavailable)?;
        effects.phase = "StagingSynced";
        checkpoint("StagingSynced")?;
        boundary.revalidate(context)?;
        revalidate()?;
        let written = stat(&staging)?;
        boundary.validate_file(&staging, written)?;
        if written.file_binding() != initial.file_binding()
            || named(&boundary.control, PENDING_FILE)? != Some(written)
            || named(&boundary.control, OWNER_FILE)?.is_some()
        {
            return Err(Blocker::ChangedDuringRead);
        }
        checkpoint("BeforeRename")?;
        let old = CString::new(PENDING_FILE).map_err(unavailable)?;
        let new = CString::new(OWNER_FILE).map_err(unavailable)?;
        effects.phase = "Publishing";
        // SAFETY: live source/destination directory and fixed terminated components.
        let result = unsafe {
            libc::syscall(
                libc::SYS_renameat2,
                boundary.control.as_raw_fd(),
                old.as_ptr(),
                boundary.control.as_raw_fd(),
                new.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        };
        if result != 0 {
            // Linux local syscall failures do not authorize cleanup. Re-observe only to report
            // known visibility; an unavailable observation remains explicitly uncertain.
            effects.publication = match named(&boundary.control, OWNER_FILE) {
                Ok(None) => PublicationEffectV1::NotPublished,
                _ => PublicationEffectV1::Uncertain,
            };
            return Err(Blocker::Unavailable);
        }
        effects.phase = "PublishedVisible";
        effects.publication = PublicationEffectV1::Visible;
        effects.durability = DurabilityEffectV1::Unconfirmed;
        checkpoint("PublishedVisible")?;
        let published = stat(&staging)?;
        boundary.validate_file(&staging, published)?;
        if published.file_binding() != written.file_binding()
            || named(&boundary.control, OWNER_FILE)? != Some(published)
        {
            effects.publication = PublicationEffectV1::Uncertain;
            return Err(Blocker::ChangedDuringRead);
        }
        checkpoint("BeforeDirectorySync")?;
        boundary.control.sync_all().map_err(unavailable)?;
        effects.phase = "DirectorySynced";
        effects.durability = DurabilityEffectV1::Confirmed;
        checkpoint("DirectorySynced")?;
        boundary.revalidate(context)?;
        revalidate()?;
        let (file, initial, bytes, observation) = boundary.collect()?;
        boundary.revalidate(context)?;
        revalidate()?;
        boundary.revalidate_record(&file, initial, &bytes)?;
        let WorkspaceIdentityObservationV1::OwnerStateObserved(identity) = observation else {
            return Err(Blocker::ChangedDuringRead);
        };
        Ok(WorkspaceOwnerPublicationV1::Published(identity, effects))
    })();
    result.unwrap_or_else(|reason| WorkspaceOwnerPublicationV1::Failed(reason, effects))
}

#[cfg(test)]
type PublicationHookV1 = Option<Box<dyn FnMut(&str) -> OwnerResult<()>>>;

#[cfg(test)]
thread_local! {
    static PUBLICATION_HOOK:std::cell::RefCell<PublicationHookV1>=const {std::cell::RefCell::new(None)};
}
fn checkpoint(phase: &str) -> OwnerResult<()> {
    #[cfg(test)]
    return PUBLICATION_HOOK.with(|hook| match hook.borrow_mut().as_mut() {
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
pub(crate) fn set_publication_hook(hook: PublicationHookV1) {
    PUBLICATION_HOOK.with(|slot| *slot.borrow_mut() = hook);
}

const ASSOCIATION_FILE_V1: &str = "workspace-association.v1.json";
const ASSOCIATION_PENDING_V1: &str = ".workspace-association.v1.pending";

/// Per-session selected-World owner state. The immutable R08 first-association
/// record remains unchanged; unrelated sessions have independent revisions.
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SelectedWorldAssociationV1 {
    schema_version: u32,
    native_owner: NativeOwnerV1,
    product_home: DirectoryBindingV1,
    workspace_root: DirectoryBindingV1,
    control_root: DirectoryBindingV1,
    record_identity: FileBindingV1,
    pub(crate) resource: transport_api_types::WorkspaceCurrentResourceV1,
    pub(crate) lane_ref: String,
    pub(crate) incarnation: String,
    pub(crate) revision: u64,
    pub(crate) active: bool,
    pub(crate) custody_ref: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SelectedWorldSourceReceiptV1 {
    schema_version: u32,
    record_identity: FileBindingV1,
    association_incarnation: String,
    association_revision: u64,
    association_source: crate::execution::workspace_read_cut::SourceIdentityV1,
    workspace_owner_source: crate::execution::workspace_read_cut::SourceIdentityV1,
}

fn selected_sources_v1(
    boundary: &HeldWorkspaceOwnerBoundaryV1,
    name: &str,
) -> OwnerResult<(
    crate::execution::workspace_read_cut::SourceIdentityV1,
    crate::execution::workspace_read_cut::SourceIdentityV1,
)> {
    let capture = |source| {
        crate::execution::workspace_read_cut::source_identity(
            &boundary.control,
            source,
            boundary.native.owner.uid,
        )
        .map_err(unavailable)?
        .ok_or(Blocker::ChangedDuringRead)
    };
    Ok((capture(name)?, capture(OWNER_FILE)?))
}

fn selected_receipt_names_v1(name: &str) -> (String, String) {
    (
        format!(".{name}.source-cut.v1.json"),
        format!(".{name}.source-cut.v1.pending"),
    )
}

pub(crate) fn selected_association_name_v1(session: &str) -> String {
    use sha2::{Digest, Sha256};
    format!(
        "workspace-world-{:x}.v1.json",
        Sha256::digest(session.as_bytes())
    )
}

fn selected_association_read_v1(
    boundary: &HeldWorkspaceOwnerBoundaryV1,
    session: &str,
) -> OwnerResult<Option<SelectedWorldAssociationV1>> {
    let name = selected_association_name_v1(session);
    let (receipt_name, receipt_pending) = selected_receipt_names_v1(&name);
    if named(&boundary.control, &format!(".{name}.pending"))?.is_some()
        || named(&boundary.control, &receipt_pending)?.is_some()
    {
        return Err(Blocker::PartialPublication);
    }
    let Some(initial) = named(&boundary.control, &name)? else {
        if named(&boundary.control, &receipt_name)?.is_some() {
            return Err(Blocker::PartialPublication);
        }
        return Ok(None);
    };
    let file = open_at(&boundary.control, &name, libc::O_RDONLY)?;
    boundary.validate_file(&file, initial)?;
    let bytes = read_bytes(&file)?;
    let value: SelectedWorldAssociationV1 =
        serde_json::from_slice(&bytes).map_err(|_| Blocker::CorruptOwnerState)?;
    if value.schema_version != 1
        || value.revision == 0
        || !valid_id(&value.lane_ref, "lane_")
        || !valid_id(&value.incarnation, "asi_")
        || value.native_owner != boundary.native.owner
        || value.product_home != boundary.native.home
        || value.workspace_root != boundary.root
        || value.control_root != boundary.control_binding
        || value.record_identity != initial.file_binding()
        || value.resource.orchestration_session_id != session
        || stat(&file)? != initial
        || named(&boundary.control, &name)? != Some(initial)
    {
        return Err(Blocker::BindingMismatch);
    }
    let receipt_initial =
        named(&boundary.control, &receipt_name)?.ok_or(Blocker::UnsupportedOwnerState)?;
    let receipt_file = open_at(&boundary.control, &receipt_name, libc::O_RDONLY)?;
    boundary.validate_file(&receipt_file, receipt_initial)?;
    let receipt: SelectedWorldSourceReceiptV1 = serde_json::from_slice(&read_bytes(&receipt_file)?)
        .map_err(|_| Blocker::CorruptOwnerState)?;
    let (association_source, workspace_owner_source) = selected_sources_v1(boundary, &name)?;
    if receipt.schema_version != 1
        || receipt.record_identity != receipt_initial.file_binding()
        || receipt.association_incarnation != value.incarnation
        || receipt.association_revision != value.revision
        || receipt.association_source != association_source
        || receipt.workspace_owner_source != workspace_owner_source
        || stat(&receipt_file)? != receipt_initial
        || named(&boundary.control, &receipt_name)? != Some(receipt_initial)
    {
        return Err(Blocker::ChangedDuringRead);
    }
    Ok(Some(value))
}

pub(crate) fn read_selected_world_association_v1(
    boundary: HeldWorkspaceOwnerBoundaryV1,
    context: &WorkspaceNativeSettingsContextV1,
    request: &transport_api_types::WorkspaceCurrentRequestV1,
) -> OwnerResult<SelectedWorldAssociationV1> {
    let owner = read_workspace_identity(boundary, context)?;
    let value =
        selected_association_read_v1(&owner.boundary, &request.resource.orchestration_session_id)?
            .ok_or(Blocker::UnsupportedOwnerState)?;
    if value.resource != request.resource {
        return Err(Blocker::BindingMismatch);
    }
    owner.revalidate(context)?;
    Ok(value)
}

fn publish_selected_receipt_v1(
    boundary: &HeldWorkspaceOwnerBoundaryV1,
    name: &str,
    value: &SelectedWorldAssociationV1,
    effects: &std::cell::Cell<transport_api_types::WorkspaceAssociationUpdateEffectsV1>,
) -> OwnerResult<()> {
    let (receipt_name, pending) = selected_receipt_names_v1(name);
    let mut effect = effects.get();
    effect.receipt.staging_created = None;
    effects.set(effect);
    let mut file = open_at(
        &boundary.control,
        &pending,
        libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
    )?;
    let mut effect = effects.get();
    effect.receipt.staging_created = Some(true);
    effects.set(effect);
    if stat(&file)?.uid != boundary.native.owner.uid
        && unsafe { libc::fchown(file.as_raw_fd(), boundary.native.owner.uid, u32::MAX) } != 0
    {
        return Err(Blocker::Unavailable);
    }
    let initial = stat(&file)?;
    boundary.validate_file(&file, initial)?;
    let (association_source, workspace_owner_source) = selected_sources_v1(boundary, name)?;
    let receipt = SelectedWorldSourceReceiptV1 {
        schema_version: 1,
        record_identity: initial.file_binding(),
        association_incarnation: value.incarnation.clone(),
        association_revision: value.revision,
        association_source,
        workspace_owner_source,
    };
    file.write_all(&serde_json::to_vec(&receipt).map_err(unavailable)?)
        .map_err(unavailable)?;
    file.sync_all().map_err(unavailable)?;
    checkpoint("SelectedReceiptStaged")?;
    let mut effect = effects.get();
    effect.receipt.published_visible = None;
    effect.receipt.directory_sync_confirmed = None;
    effects.set(effect);
    crate::execution::workspace_read_cut::rename(&boundary.control, &pending, &receipt_name, false)
        .map_err(unavailable)?;
    let mut effect = effects.get();
    effect.receipt.published_visible = Some(true);
    effect.receipt.directory_sync_confirmed = Some(true);
    effects.set(effect);
    Ok(())
}

pub(crate) fn publish_selected_world_association_v1(
    boundary: HeldWorkspaceOwnerBoundaryV1,
    context: &WorkspaceNativeSettingsContextV1,
    request: &transport_api_types::WorkspaceCurrentRequestV1,
    active: bool,
    custody: &str,
    effects: &std::cell::Cell<transport_api_types::WorkspaceAssociationUpdateEffectsV1>,
    sources_current: &dyn Fn() -> OwnerResult<()>,
) -> OwnerResult<transport_api_types::WorkspaceAssociationUpdateResponseV1> {
    boundary.lock(true)?;
    boundary.revalidate(context)?;
    sources_current()?;
    let (_, _, _, identity) = boundary.collect()?;
    if !matches!(identity, WorkspaceIdentityObservationV1::OwnerStateObserved(ref i)
        if i.workspace_ref == request.resource.workspace_ref && i.store_incarnation == request.resource.workspace_incarnation)
    {
        return Err(Blocker::BindingMismatch);
    }
    if named(&boundary.control, ASSOCIATION_PENDING_V1)?.is_some() {
        return Err(Blocker::PartialPublication);
    }
    let previous =
        selected_association_read_v1(&boundary, &request.resource.orchestration_session_id)?;
    let first = if previous.is_none() {
        if let Some(initial) = named(&boundary.control, ASSOCIATION_FILE_V1)? {
            let file = open_at(&boundary.control, ASSOCIATION_FILE_V1, libc::O_RDONLY)?;
            boundary.validate_file(&file, initial)?;
            let first: WorkspaceAssociationRecordV1 = serde_json::from_slice(&read_bytes(&file)?)
                .map_err(|_| Blocker::CorruptOwnerState)?;
            if first.schema_version != 1
                || first.association_revision != 1
                || first.record_identity != initial.file_binding()
                || first.native_owner != boundary.native.owner
                || first.product_home != boundary.native.home
                || first.workspace_root != boundary.root
                || first.control_root != boundary.control_binding
                || first.workspace_ref != request.resource.workspace_ref
                || first.workspace_incarnation != request.resource.workspace_incarnation
                || !valid_id(&first.lane_ref, "lane_")
                || !valid_id(&first.association_incarnation, "asi_")
            {
                return Err(Blocker::BindingMismatch);
            }
            if first.orchestration_session_id == request.resource.orchestration_session_id
                && first.world_id == request.resource.world_id
                && first.world_generation == request.resource.world_generation
                && first.backend_id == request.resource.backend_id
            {
                Some((first.lane_ref, first.association_incarnation))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    let name = selected_association_name_v1(&request.resource.orchestration_session_id);
    let pending = format!(".{name}.pending");
    let mut uncertain_create = effects.get();
    uncertain_create.association.staging_created = None;
    effects.set(uncertain_create);
    let mut file = open_at(
        &boundary.control,
        &pending,
        libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
    )?;
    let mut effect = effects.get();
    effect.association.staging_created = Some(true);
    effects.set(effect);
    if stat(&file)?.uid != boundary.native.owner.uid
        && unsafe { libc::fchown(file.as_raw_fd(), boundary.native.owner.uid, u32::MAX) } != 0
    {
        return Err(Blocker::Unavailable);
    }
    let initial = stat(&file)?;
    boundary.validate_file(&file, initial)?;
    let value = SelectedWorldAssociationV1 {
        schema_version: 1,
        native_owner: boundary.native.owner.clone(),
        product_home: boundary.native.home.clone(),
        workspace_root: boundary.root.clone(),
        control_root: boundary.control_binding.clone(),
        record_identity: initial.file_binding(),
        resource: request.resource.clone(),
        lane_ref: previous
            .as_ref()
            .map(|v| v.lane_ref.clone())
            .or_else(|| first.as_ref().map(|v| v.0.clone()))
            .map(Ok)
            .unwrap_or_else(|| random_id("lane_"))?,
        incarnation: previous
            .as_ref()
            .map(|v| v.incarnation.clone())
            .or_else(|| first.as_ref().map(|v| v.1.clone()))
            .map(Ok)
            .unwrap_or_else(|| random_id("asi_"))?,
        revision: previous
            .as_ref()
            .map_or(Some(1), |v| v.revision.checked_add(1))
            .ok_or(Blocker::UnsupportedOwnerState)?,
        active,
        custody_ref: custody.to_owned(),
    };
    file.write_all(&serde_json::to_vec(&value).map_err(unavailable)?)
        .map_err(unavailable)?;
    file.sync_all().map_err(unavailable)?;
    boundary.revalidate(context)?;
    sources_current()?;
    let mut uncertain = effects.get();
    uncertain.association.published_visible = None;
    uncertain.association.directory_sync_confirmed = None;
    effects.set(uncertain);
    crate::execution::workspace_read_cut::rename(&boundary.control, &pending, &name, false)
        .map_err(unavailable)?;
    let mut effect = effects.get();
    effect.association.published_visible = Some(true);
    effect.association.directory_sync_confirmed = Some(true);
    effects.set(effect);
    checkpoint("SelectedAssociationPublished")?;
    publish_selected_receipt_v1(&boundary, &name, &value, effects)?;
    if selected_association_read_v1(&boundary, &request.resource.orchestration_session_id)?
        != Some(value.clone())
    {
        return Err(Blocker::ChangedDuringRead);
    }
    boundary.revalidate(context)?;
    sources_current()?;
    Ok(
        transport_api_types::WorkspaceAssociationUpdateResponseV1::AssociationPublished {
            lane_ref: value.lane_ref,
            association_incarnation: value.incarnation,
            association_revision: value.revision,
            backend_custody_ref: value.custody_ref,
            effects: effects.get(),
        },
    )
}

/// A separately versioned, immutable first association. Owner-v1 remains identity-only.
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WorkspaceAssociationRecordV1 {
    schema_version: u32,
    native_owner: NativeOwnerV1,
    product_home: DirectoryBindingV1,
    workspace_root: DirectoryBindingV1,
    control_root: DirectoryBindingV1,
    record_identity: FileBindingV1,
    workspace_ref: String,
    workspace_incarnation: String,
    lane_ref: String,
    association_incarnation: String,
    association_revision: u64,
    orchestration_session_id: String,
    world_id: String,
    world_generation: u64,
    backend_id: String,
    backend_custody_ref: String,
}

struct HeldAssociationRecordV1 {
    file: File,
    initial: StatV1,
    bytes: Vec<u8>,
    record: WorkspaceAssociationRecordV1,
}

pub(crate) struct WorkspaceAssociationOperationV1 {
    owner: WorkspaceOwnerReadV1,
    request: transport_api_types::WorkspaceCurrentRequestV1,
    association: std::cell::RefCell<Option<HeldAssociationRecordV1>>,
    publish: bool,
}

impl WorkspaceAssociationOperationV1 {
    pub(crate) fn open(
        boundary: HeldWorkspaceOwnerBoundaryV1,
        context: &WorkspaceNativeSettingsContextV1,
        request: &transport_api_types::WorkspaceCurrentRequestV1,
        publish: bool,
    ) -> OwnerResult<Self> {
        boundary.lock(publish)?;
        boundary.revalidate(context)?;
        let (file, initial, bytes, observation) = boundary.collect()?;
        if !matches!(&observation, WorkspaceIdentityObservationV1::OwnerStateObserved(identity)
            if identity.workspace_ref == request.resource.workspace_ref
                && identity.store_incarnation == request.resource.workspace_incarnation)
        {
            return Err(Blocker::BindingMismatch);
        }
        let owner = WorkspaceOwnerReadV1 {
            boundary,
            file,
            initial,
            bytes,
            observation,
        };
        let value = Self {
            owner,
            request: request.clone(),
            association: std::cell::RefCell::new(None),
            publish,
        };
        *value.association.borrow_mut() = value.collect_association()?;
        value.revalidate(context)?;
        Ok(value)
    }

    fn collect_association(&self) -> OwnerResult<Option<HeldAssociationRecordV1>> {
        let boundary = &self.owner.boundary;
        if named(&boundary.control, ASSOCIATION_PENDING_V1)?.is_some() {
            return Err(Blocker::PartialPublication);
        }
        let Some(initial) = named(&boundary.control, ASSOCIATION_FILE_V1)? else {
            return Ok(None);
        };
        let file = open_at(&boundary.control, ASSOCIATION_FILE_V1, libc::O_RDONLY)?;
        boundary.validate_file(&file, initial)?;
        let bytes = read_bytes(&file)?;
        let record: WorkspaceAssociationRecordV1 =
            serde_json::from_slice(&bytes).map_err(|_| Blocker::CorruptOwnerState)?;
        let r = &self.request.resource;
        if record.schema_version != 1
            || record.association_revision != 1
            || !valid_id(&record.lane_ref, "lane_")
            || !valid_id(&record.association_incarnation, "asi_")
            || record.backend_custody_ref.len() != 64
            || !record
                .backend_custody_ref
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(Blocker::UnsupportedOwnerState);
        }
        if record.record_identity != initial.file_binding()
            || record.native_owner != boundary.native.owner
            || record.product_home != boundary.native.home
            || record.workspace_root != boundary.root
            || record.control_root != boundary.control_binding
            || record.workspace_ref != r.workspace_ref
            || record.workspace_incarnation != r.workspace_incarnation
            || record.orchestration_session_id != r.orchestration_session_id
            || record.world_id != r.world_id
            || record.world_generation != r.world_generation
            || record.backend_id != r.backend_id
        {
            return Err(Blocker::BindingMismatch);
        }
        if stat(&file)? != initial
            || named(&boundary.control, ASSOCIATION_FILE_V1)? != Some(initial)
        {
            return Err(Blocker::ChangedDuringRead);
        }
        Ok(Some(HeldAssociationRecordV1 {
            file,
            initial,
            bytes,
            record,
        }))
    }

    pub(crate) fn revalidate(&self, context: &WorkspaceNativeSettingsContextV1) -> OwnerResult<()> {
        self.owner.revalidate(context)?;
        let expected = self.association.borrow();
        if named(&self.owner.boundary.control, ASSOCIATION_PENDING_V1)?.is_some() {
            return Err(Blocker::PartialPublication);
        }
        match expected.as_ref() {
            None => {
                if named(&self.owner.boundary.control, ASSOCIATION_FILE_V1)?.is_some() {
                    return Err(Blocker::ChangedDuringRead);
                }
            }
            Some(record) => {
                self.owner
                    .boundary
                    .validate_file(&record.file, record.initial)?;
                if read_bytes(&record.file)? != record.bytes
                    || named(&self.owner.boundary.control, ASSOCIATION_FILE_V1)?
                        != Some(record.initial)
                {
                    return Err(Blocker::ChangedDuringRead);
                }
                let fresh = self
                    .collect_association()?
                    .ok_or(Blocker::ChangedDuringRead)?;
                if fresh.initial != record.initial || fresh.bytes != record.bytes {
                    return Err(Blocker::ChangedDuringRead);
                }
            }
        }
        Ok(())
    }

    pub(crate) fn read_joined(
        &self,
        context: &WorkspaceNativeSettingsContextV1,
        request: &transport_api_types::WorkspaceCurrentRequestV1,
        held_custody: &world::session::ExistingWorldCustodyV1<'_>,
        actor: &crate::execution::agent_runtime::host_session_authority::facade::ResolvedCurrentAuthorityV1,
    ) -> OwnerResult<transport_api_types::WorkspaceCurrentResponseV1> {
        held_custody.revalidate().map_err(unavailable)?;
        let custody = held_custody.binding_ref();
        self.validate_read_selectors_v1(context, request, custody)?;
        if !matches!(&request.operation, transport_api_types::WorkspaceCurrentOperationV1::CurrentActorRead { expected_actor, .. }
            if &actor.caller.participant_id == expected_actor)
        {
            return Err(Blocker::BindingMismatch);
        }
        use transport_api_types::{
            WorkspaceCurrentOperationV1 as O, WorkspaceCurrentResponseV1 as R,
        };
        self.revalidate(context)?;
        let O::CurrentActorRead {
            lane_ref,
            association_incarnation,
            association_revision,
            backend_custody_ref,
            expected_actor,
        } = &request.operation
        else {
            return Err(Blocker::BindingMismatch);
        };
        let association = self.association.borrow();
        let record = &association
            .as_ref()
            .ok_or(Blocker::UnsupportedOwnerState)?
            .record;
        if record.lane_ref != *lane_ref
            || record.association_incarnation != *association_incarnation
            || record.association_revision != *association_revision
            || record.backend_custody_ref != *backend_custody_ref
            || record.backend_custody_ref != custody
        {
            return Err(Blocker::BindingMismatch);
        }
        Ok(R::CurrentActor(Box::new(
            transport_api_types::WorkspaceCurrentActorV1 {
                workspace_ref: request.resource.workspace_ref.clone(),
                workspace_incarnation: request.resource.workspace_incarnation.clone(),
                orchestration_session_id: request.resource.orchestration_session_id.clone(),
                world_id: request.resource.world_id.clone(),
                world_generation: request.resource.world_generation,
                backend_id: request.resource.backend_id.clone(),
                authority_store_id: request.observation.authority_store_id.clone(),
                root_revision: request.observation.root_revision,
                authority_revision: request.observation.authority_revision,
                authority_record_sha256: request.observation.authority_record_sha256.clone(),
                authoritative_lineage_sha256: request
                    .observation
                    .authoritative_lineage_sha256
                    .clone(),
                lane_ref: record.lane_ref.clone(),
                association_incarnation: record.association_incarnation.clone(),
                association_revision: record.association_revision,
                backend_custody_ref: record.backend_custody_ref.clone(),
                actor_id: expected_actor.clone(),
            },
        )))
    }

    /// Called only by the joined owner operation after independent mutation permission,
    /// exact current HSA validation and an actual backend-owned held custody observation.
    pub(crate) fn publish(
        &self,
        context: &WorkspaceNativeSettingsContextV1,
        held_custody: &world::session::ExistingWorldCustodyV1<'_>,
        effects: &std::cell::Cell<transport_api_types::WorkspaceAssociationEffectsV1>,
        sources_current: &dyn Fn() -> OwnerResult<()>,
    ) -> OwnerResult<transport_api_types::WorkspaceCurrentResponseV1> {
        use transport_api_types::WorkspaceCurrentResponseV1 as R;
        if !self.publish {
            return Err(Blocker::BindingMismatch);
        }
        held_custody.revalidate().map_err(unavailable)?;
        let custody = held_custody.binding_ref();
        self.revalidate(context)?;
        sources_current()?;
        if let Some(existing) = self.association.borrow().as_ref() {
            if existing.record.backend_custody_ref != custody {
                return Err(Blocker::BindingMismatch);
            }
            return Ok(R::AssociationPublished {
                lane_ref: existing.record.lane_ref.clone(),
                association_incarnation: existing.record.association_incarnation.clone(),
                association_revision: existing.record.association_revision,
                backend_custody_ref: existing.record.backend_custody_ref.clone(),
                effects: effects.get(),
            });
        }
        let boundary = &self.owner.boundary;
        let mut pending = open_at(
            &boundary.control,
            ASSOCIATION_PENDING_V1,
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
        )?;
        let mut observed = effects.get();
        observed.staging_created = Some(true);
        effects.set(observed);
        checkpoint("AssociationStagingCreated")?;
        // Only the newly created staging file receives its genuine native owner's UID.
        // No existing file, Workspace, World or account ownership changes.
        if stat(&pending)?.uid != boundary.native.owner.uid
            && unsafe { libc::fchown(pending.as_raw_fd(), boundary.native.owner.uid, u32::MAX) }
                != 0
        {
            return Err(Blocker::Unavailable);
        }
        let initial = stat(&pending)?;
        boundary.validate_file(&pending, initial)?;
        let r = &self.request.resource;
        let record = WorkspaceAssociationRecordV1 {
            schema_version: 1,
            native_owner: boundary.native.owner.clone(),
            product_home: boundary.native.home.clone(),
            workspace_root: boundary.root.clone(),
            control_root: boundary.control_binding.clone(),
            record_identity: initial.file_binding(),
            workspace_ref: r.workspace_ref.clone(),
            workspace_incarnation: r.workspace_incarnation.clone(),
            lane_ref: random_id("lane_")?,
            association_incarnation: random_id("asi_")?,
            association_revision: 1,
            orchestration_session_id: r.orchestration_session_id.clone(),
            world_id: r.world_id.clone(),
            world_generation: r.world_generation,
            backend_id: r.backend_id.clone(),
            backend_custody_ref: custody.to_owned(),
        };
        let bytes = serde_json::to_vec(&record).map_err(unavailable)?;
        pending.write_all(&bytes).map_err(unavailable)?;
        pending.sync_all().map_err(unavailable)?;
        checkpoint("AssociationStagingSynced")?;
        held_custody.revalidate().map_err(unavailable)?;
        sources_current()?;
        self.owner.revalidate(context)?;
        let written = stat(&pending)?;
        boundary.validate_file(&pending, written)?;
        if named(&boundary.control, ASSOCIATION_PENDING_V1)? != Some(written)
            || named(&boundary.control, ASSOCIATION_FILE_V1)?.is_some()
        {
            return Err(Blocker::ChangedDuringRead);
        }
        let source = CString::new(ASSOCIATION_PENDING_V1).map_err(unavailable)?;
        let target = CString::new(ASSOCIATION_FILE_V1).map_err(unavailable)?;
        if unsafe {
            libc::syscall(
                libc::SYS_renameat2,
                boundary.control.as_raw_fd(),
                source.as_ptr(),
                boundary.control.as_raw_fd(),
                target.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        } != 0
        {
            return Err(Blocker::Unavailable);
        }
        let mut observed = effects.get();
        observed.published_visible = Some(true);
        effects.set(observed);
        checkpoint("AssociationPublishedVisible")?;
        boundary.control.sync_all().map_err(unavailable)?;
        let mut observed = effects.get();
        observed.directory_sync_confirmed = Some(true);
        effects.set(observed);
        checkpoint("AssociationDirectorySynced")?;
        *self.association.borrow_mut() = self.collect_association()?;
        self.revalidate(context)?;
        sources_current()?;
        Ok(R::AssociationPublished {
            lane_ref: record.lane_ref,
            association_incarnation: record.association_incarnation,
            association_revision: 1,
            backend_custody_ref: record.backend_custody_ref,
            effects: effects.get(),
        })
    }
}

impl HeldWorkspaceOwnerBoundaryV1 {
    pub(crate) fn duplicate_unlocked_v1(
        &self,
        context: &WorkspaceNativeSettingsContextV1,
    ) -> OwnerResult<Self> {
        Self::capture(
            context,
            Path::new(&self.root.physical_path),
            &self.control,
            self.directories
                .iter()
                .map(|directory| (directory.path.as_path(), &directory.file)),
        )
    }
}

impl WorkspaceAssociationOperationV1 {
    pub(crate) fn publication_matches_v1(
        &self,
        response: &transport_api_types::WorkspaceCurrentResponseV1,
    ) -> bool {
        let association = self.association.borrow();
        let Some(association) = association.as_ref() else {
            return false;
        };
        matches!(response, transport_api_types::WorkspaceCurrentResponseV1::AssociationPublished {
            lane_ref, association_incarnation, association_revision, backend_custody_ref, ..
        } if lane_ref == &association.record.lane_ref && association_incarnation == &association.record.association_incarnation
            && association_revision == &association.record.association_revision && backend_custody_ref == &association.record.backend_custody_ref)
    }
}

impl WorkspaceAssociationOperationV1 {
    pub(crate) fn validate_read_selectors_v1(
        &self,
        context: &WorkspaceNativeSettingsContextV1,
        request: &transport_api_types::WorkspaceCurrentRequestV1,
        custody: &str,
    ) -> OwnerResult<()> {
        self.revalidate(context)?;
        let transport_api_types::WorkspaceCurrentOperationV1::CurrentActorRead {
            lane_ref,
            association_incarnation,
            association_revision,
            backend_custody_ref,
            ..
        } = &request.operation
        else {
            return Err(Blocker::BindingMismatch);
        };
        let association = self.association.borrow();
        let record = &association
            .as_ref()
            .ok_or(Blocker::UnsupportedOwnerState)?
            .record;
        if record.lane_ref != *lane_ref
            || record.association_incarnation != *association_incarnation
            || record.association_revision != *association_revision
            || record.backend_custody_ref != *backend_custody_ref
            || record.backend_custody_ref != custody
        {
            return Err(Blocker::BindingMismatch);
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyWorkspaceCollectionsV1 {
    schema_version: u32,
    workspace_ref: String,
    workspace_incarnation: String,
    lanes: Vec<String>,
    associations: Vec<String>,
}

pub(crate) fn read_current_workspace_components_v1(
    boundary: HeldWorkspaceOwnerBoundaryV1,
    context: &WorkspaceNativeSettingsContextV1,
) -> OwnerResult<serde_json::Value> {
    let owner = read_workspace_identity(boundary, context)?;
    let WorkspaceIdentityObservationV1::OwnerStateObserved(identity) = &owner.observation else {
        return Err(Blocker::BindingMismatch);
    };
    let boundary = &owner.boundary;
    let mut entries = Vec::new();
    let mut supported = false;
    if let Some(initial) = named(&boundary.control, "workspace-collections.v1.json")? {
        let file = open_at(
            &boundary.control,
            "workspace-collections.v1.json",
            libc::O_RDONLY,
        )?;
        boundary.validate_file(&file, initial)?;
        let value: EmptyWorkspaceCollectionsV1 =
            serde_json::from_slice(&read_bytes(&file)?).map_err(|_| Blocker::CorruptOwnerState)?;
        if value.schema_version != 1
            || value.workspace_ref != identity.workspace_ref
            || value.workspace_incarnation != identity.store_incarnation
            || !value.lanes.is_empty()
            || !value.associations.is_empty()
        {
            return Err(Blocker::UnsupportedOwnerState);
        }
        supported = true;
    }
    for entry in std::fs::read_dir(format!("/proc/self/fd/{}", boundary.control.as_raw_fd()))
        .map_err(unavailable)?
    {
        let name = entry.map_err(unavailable)?.file_name();
        let Some(name) = name.to_str() else {
            return Err(Blocker::UnsafeOwnerState);
        };
        if name.starts_with("workspace-world-") && name.ends_with(".v1.json") {
            let file = open_at(&boundary.control, name, libc::O_RDONLY)?;
            let initial = stat(&file)?;
            boundary.validate_file(&file, initial)?;
            let raw: SelectedWorldAssociationV1 = serde_json::from_slice(&read_bytes(&file)?)
                .map_err(|_| Blocker::CorruptOwnerState)?;
            if selected_association_name_v1(&raw.resource.orchestration_session_id) != name {
                return Err(Blocker::BindingMismatch);
            }
            let value =
                selected_association_read_v1(boundary, &raw.resource.orchestration_session_id)?
                    .ok_or(Blocker::ChangedDuringRead)?;
            if value.resource.workspace_ref != identity.workspace_ref
                || value.resource.workspace_incarnation != identity.store_incarnation
            {
                return Err(Blocker::BindingMismatch);
            }
            entries.push(serde_json::json!({"lane_ref":value.lane_ref,"world_id":value.resource.world_id,"world_generation":value.resource.world_generation,
                "association_incarnation":value.incarnation,"association_revision":value.revision,"active":value.active}));
            supported = true;
        }
        if name.starts_with(".workspace-world-") && name.ends_with(".pending") {
            return Err(Blocker::PartialPublication);
        }
    }
    if let Some(initial) = named(&boundary.control, ASSOCIATION_FILE_V1)? {
        let file = open_at(&boundary.control, ASSOCIATION_FILE_V1, libc::O_RDONLY)?;
        boundary.validate_file(&file, initial)?;
        let value: WorkspaceAssociationRecordV1 =
            serde_json::from_slice(&read_bytes(&file)?).map_err(|_| Blocker::CorruptOwnerState)?;
        if value.schema_version != 1
            || value.association_revision != 1
            || value.record_identity != initial.file_binding()
            || value.native_owner != boundary.native.owner
            || value.product_home != boundary.native.home
            || value.workspace_root != boundary.root
            || value.control_root != boundary.control_binding
            || value.workspace_ref != identity.workspace_ref
            || value.workspace_incarnation != identity.store_incarnation
            || !valid_id(&value.lane_ref, "lane_")
            || !valid_id(&value.association_incarnation, "asi_")
        {
            return Err(Blocker::BindingMismatch);
        }
        if !entries
            .iter()
            .any(|entry| entry["lane_ref"] == value.lane_ref)
        {
            entries.push(serde_json::json!({"lane_ref":value.lane_ref,"world_id":value.world_id,"world_generation":value.world_generation,
                "association_incarnation":value.association_incarnation,"association_revision":value.association_revision,"coverage":"FirstAssociationOnly"}));
        }
        supported = true;
    }
    owner.revalidate(context)?;
    Ok(
        serde_json::json!({"availability":if supported{"OwnerObserved"}else{"Unsupported"},"associations":entries,"runtime_liveness":"NotObserved"}),
    )
}
