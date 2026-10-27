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
thread_local! {
    static PUBLICATION_HOOK:std::cell::RefCell<Option<Box<dyn FnMut(&str)->OwnerResult<()>>>>=const {std::cell::RefCell::new(None)};
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
pub(crate) fn set_publication_hook(hook: Option<Box<dyn FnMut(&str) -> OwnerResult<()>>>) {
    PUBLICATION_HOOK.with(|slot| *slot.borrow_mut() = hook);
}
