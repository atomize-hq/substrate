use std::fmt;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::fs::File;
use std::io;

use super::schema::CanonicalDirectoryV1;

#[derive(Debug)]
pub(crate) struct TrustedFsError {
    message: String,
    kind: Option<io::ErrorKind>,
}

impl TrustedFsError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: None,
        }
    }

    fn from_io(operation: &str, error: io::Error) -> Self {
        Self {
            message: format!("{operation}: {error}"),
            kind: Some(error.kind()),
        }
    }

    pub(crate) fn is_already_exists(&self) -> bool {
        self.kind == Some(io::ErrorKind::AlreadyExists)
    }
}

impl fmt::Display for TrustedFsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for TrustedFsError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PrivateHomeReason {
    MissingParent,
    WrongType,
    Symlink,
    WrongOwner,
    WrongMode,
    ForeignAcl,
    Replaced,
    ValidationUnavailable,
}

impl PrivateHomeReason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::MissingParent => "missing-parent",
            Self::WrongType => "wrong-type",
            Self::Symlink => "symlink",
            Self::WrongOwner => "wrong-owner",
            Self::WrongMode => "wrong-mode",
            Self::ForeignAcl => "foreign-acl",
            Self::Replaced => "replaced",
            Self::ValidationUnavailable => "validation-unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PrivateHomeObjectRole {
    Ancestor,
    FinalRoot,
}

impl PrivateHomeObjectRole {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Ancestor => "ancestor",
            Self::FinalRoot => "final-root",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PrivateHomeAclKind {
    Access,
    Default,
    Unavailable,
}

impl PrivateHomeAclKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Access => "access",
            Self::Default => "default",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PrivateHomeAclAuthority {
    EffectiveWrite,
    DefaultAclPresent,
    ExtendedAccessAcl,
    Malformed,
    Unavailable,
}

impl PrivateHomeAclAuthority {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::EffectiveWrite => "effective-write",
            Self::DefaultAclPresent => "default-acl-present",
            Self::ExtendedAccessAcl => "extended-access-acl",
            Self::Malformed => "malformed-acl",
            Self::Unavailable => "acl-validation-unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PrivateHomeCandidateProvenance {
    NotCreated,
    Created,
    Unknown,
}

impl PrivateHomeCandidateProvenance {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::NotCreated => "no",
            Self::Created => "yes",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug)]
pub(crate) struct PrivateHomeError {
    reason: PrivateHomeReason,
    requested_home: Option<std::path::PathBuf>,
    offending_path: Option<std::path::PathBuf>,
    object_role: Option<PrivateHomeObjectRole>,
    acl_kind: Option<PrivateHomeAclKind>,
    acl_authority: Option<PrivateHomeAclAuthority>,
    candidate_provenance: PrivateHomeCandidateProvenance,
}

impl PrivateHomeError {
    fn new(reason: PrivateHomeReason) -> Self {
        Self {
            reason,
            requested_home: None,
            offending_path: None,
            object_role: None,
            acl_kind: None,
            acl_authority: None,
            candidate_provenance: PrivateHomeCandidateProvenance::NotCreated,
        }
    }

    fn acl(
        reason: PrivateHomeReason,
        kind: PrivateHomeAclKind,
        authority: PrivateHomeAclAuthority,
    ) -> Self {
        Self {
            acl_kind: Some(kind),
            acl_authority: Some(authority),
            ..Self::new(reason)
        }
    }

    fn at_path(mut self, path: &std::path::Path, role: PrivateHomeObjectRole) -> Self {
        if self.offending_path.is_none() {
            self.offending_path = Some(path.to_path_buf());
        }
        if self.object_role.is_none() {
            self.object_role = Some(role);
        }
        self
    }

    fn for_attempt(mut self, requested_home: &std::path::Path, candidate_created: bool) -> Self {
        self.requested_home = Some(requested_home.to_path_buf());
        if candidate_created {
            self.candidate_provenance = PrivateHomeCandidateProvenance::Created;
        }
        self
    }

    fn with_candidate_provenance(mut self, provenance: PrivateHomeCandidateProvenance) -> Self {
        self.candidate_provenance = provenance;
        self
    }

    pub(crate) fn reason(&self) -> PrivateHomeReason {
        self.reason
    }

    pub(crate) fn requested_home(&self) -> Option<&std::path::Path> {
        self.requested_home.as_deref()
    }

    pub(crate) fn offending_path(&self) -> Option<&std::path::Path> {
        self.offending_path.as_deref()
    }

    pub(crate) fn object_role(&self) -> Option<PrivateHomeObjectRole> {
        self.object_role
    }

    pub(crate) fn acl_kind(&self) -> Option<PrivateHomeAclKind> {
        self.acl_kind
    }

    pub(crate) fn acl_authority(&self) -> Option<PrivateHomeAclAuthority> {
        self.acl_authority
    }

    pub(crate) fn candidate_created(&self) -> bool {
        self.candidate_provenance == PrivateHomeCandidateProvenance::Created
    }

    pub(crate) fn candidate_provenance(&self) -> PrivateHomeCandidateProvenance {
        self.candidate_provenance
    }

    #[cfg(test)]
    pub(crate) fn acl_diagnostic_for_test(
        requested_home: &std::path::Path,
        offending_path: &std::path::Path,
        role: PrivateHomeObjectRole,
        kind: PrivateHomeAclKind,
        authority: PrivateHomeAclAuthority,
        candidate_provenance: PrivateHomeCandidateProvenance,
    ) -> Self {
        let reason = if authority == PrivateHomeAclAuthority::Unavailable {
            PrivateHomeReason::ValidationUnavailable
        } else {
            PrivateHomeReason::ForeignAcl
        };
        Self::acl(reason, kind, authority)
            .at_path(offending_path, role)
            .for_attempt(requested_home, false)
            .with_candidate_provenance(candidate_provenance)
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod platform {
    use std::ffi::{CStr, CString, OsStr};
    use std::fs;
    use std::fs::OpenOptions;
    use std::io;
    use std::mem::MaybeUninit;
    use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::OpenOptionsExt;
    use std::path::{Component, Path, PathBuf};

    use super::{
        CanonicalDirectoryV1, File, PrivateHomeAclAuthority, PrivateHomeAclKind,
        PrivateHomeCandidateProvenance, PrivateHomeError, PrivateHomeObjectRole, PrivateHomeReason,
        TrustedFsError,
    };
    use crate::execution::agent_runtime::host_session_authority::schema::DirectoryPhysicalIdentityV1;

    const DIRECTORY_MODE: libc::mode_t = 0o700;
    const FILE_MODE: libc::mode_t = 0o600;
    const ACL_USER: u16 = 0x0002;
    const ACL_GROUP: u16 = 0x0008;
    #[cfg(target_os = "linux")]
    const ACL_MASK: u16 = 0x0010;
    #[cfg(all(test, target_os = "linux"))]
    enum TestLinuxAclRead {
        Present(PrivateHomeAclKind, Vec<u8>),
        Unavailable(PrivateHomeAclKind),
    }
    #[cfg(all(test, target_os = "linux"))]
    std::thread_local! {
        static TEST_LINUX_ACL_READ: std::cell::RefCell<Option<TestLinuxAclRead>> = const {
            std::cell::RefCell::new(None)
        };
    }
    #[cfg(target_os = "macos")]
    const MACOS_ACL_EXTENDED_ALLOW: libc::c_int = 1;
    #[cfg(target_os = "macos")]
    const MACOS_ACL_EXTENDED_DENY: libc::c_int = 2;

    #[derive(Debug)]
    pub(crate) struct TrustedAuthorityRoot {
        directory: TrustedDirectory,
        identity: CanonicalDirectoryV1,
        owner_uid: libc::uid_t,
        requested_home: PathBuf,
        candidate_created: bool,
    }

    #[derive(Debug)]
    pub(crate) struct TrustedWorkspaceRoot {
        file: File,
        identity: CanonicalDirectoryV1,
    }

    #[derive(Debug)]
    pub(crate) struct TrustedDirectory {
        file: File,
        device_id: u64,
    }

    #[derive(Debug)]
    pub(crate) struct TrustedFile {
        file: File,
        device_id: u64,
        inode: u64,
    }

    pub(crate) struct TrustedScaffoldDirectory {
        file: File,
        device_id: u64,
        owner_uid: libc::uid_t,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum DirectoryCreation {
        Created,
        Existing,
    }

    struct TrustedDirectoryChain {
        directories: Vec<File>,
        components: Vec<CString>,
        identities: Vec<(u64, u64)>,
    }

    impl TrustedDirectoryChain {
        fn final_directory(&self) -> &File {
            self.directories
                .last()
                .expect("trusted directory chain always contains filesystem root")
        }

        fn revalidate(&self, owner_uid: libc::uid_t) -> Result<(), PrivateHomeError> {
            for (index, directory) in self.directories.iter().enumerate() {
                let path = self.path_at(index);
                let stat = validate_private_home_ancestor(directory, owner_uid)
                    .map_err(|error| error.at_path(&path, PrivateHomeObjectRole::Ancestor))?;
                if (stat.st_dev as u64, stat.st_ino as u64) != self.identities[index] {
                    return Err(PrivateHomeError::new(PrivateHomeReason::Replaced)
                        .at_path(&path, PrivateHomeObjectRole::Ancestor));
                }
                if index > 0 {
                    let named = fstatat_nofollow(
                        self.directories[index - 1].as_raw_fd(),
                        &self.components[index - 1],
                    )
                    .map_err(|_| {
                        PrivateHomeError::new(PrivateHomeReason::Replaced)
                            .at_path(&path, PrivateHomeObjectRole::Ancestor)
                    })?;
                    if kind_from_mode(named.st_mode) == EntryKind::Symlink {
                        return Err(PrivateHomeError::new(PrivateHomeReason::Symlink)
                            .at_path(&path, PrivateHomeObjectRole::Ancestor));
                    }
                    if (named.st_dev as u64, named.st_ino as u64) != self.identities[index] {
                        return Err(PrivateHomeError::new(PrivateHomeReason::Replaced)
                            .at_path(&path, PrivateHomeObjectRole::Ancestor));
                    }
                }
            }
            let path = self.path_at(self.directories.len() - 1);
            validate_private_home_parent(self.final_directory(), owner_uid)
                .map_err(|error| error.at_path(&path, PrivateHomeObjectRole::Ancestor))?;
            Ok(())
        }

        fn path_at(&self, directory_index: usize) -> PathBuf {
            let mut path = PathBuf::from("/");
            for component in self.components.iter().take(directory_index) {
                path.push(OsStr::from_bytes(component.as_bytes()));
            }
            path
        }
    }

    struct TrustedDirectoryLock<'a> {
        file: &'a File,
    }

    impl<'a> TrustedDirectoryLock<'a> {
        fn acquire(file: &'a File) -> Result<Self, TrustedFsError> {
            // SAFETY: file owns a live directory descriptor for this lock lifetime.
            if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } != 0 {
                return Err(io_error_value("lock trusted directory"));
            }
            Ok(Self { file })
        }
    }

    impl Drop for TrustedDirectoryLock<'_> {
        fn drop(&mut self) {
            // SAFETY: the directory descriptor outlives this guard.
            unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
        }
    }

    pub(crate) struct TrustedFileLock<'a> {
        file: Option<&'a TrustedFile>,
    }

    pub(crate) struct TrustedOwnedFileLock {
        file: Option<File>,
    }

    impl TrustedFileLock<'_> {
        pub(crate) fn release(mut self) -> Result<(), TrustedFsError> {
            let file = self
                .file
                .take()
                .ok_or_else(|| TrustedFsError::new("trusted lock already released"))?;
            // SAFETY: descriptor is live and owns this flock.
            if unsafe { libc::flock(file.file.as_raw_fd(), libc::LOCK_UN) } != 0 {
                return Err(io_error_value("release trusted file lock"));
            }
            Ok(())
        }
    }

    impl Drop for TrustedFileLock<'_> {
        fn drop(&mut self) {
            if let Some(file) = self.file.take() {
                // SAFETY: descriptor remains live; fd close also releases after any failure.
                unsafe { libc::flock(file.file.as_raw_fd(), libc::LOCK_UN) };
            }
        }
    }

    impl Drop for TrustedOwnedFileLock {
        fn drop(&mut self) {
            if let Some(file) = self.file.take() {
                // SAFETY: descriptor is live and owns this flock through dup(2).
                unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_UN) };
            }
        }
    }

    struct DirectoryStream(*mut libc::DIR);

    impl DirectoryStream {
        fn close(mut self) -> Result<(), TrustedFsError> {
            // SAFETY: the stream is live and this consumes its sole owner.
            let result = unsafe { libc::closedir(self.0) };
            self.0 = std::ptr::null_mut();
            if result == 0 {
                Ok(())
            } else {
                Err(io_error_value("close trusted directory enumeration"))
            }
        }
    }

    impl Drop for DirectoryStream {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: a non-null stream is owned by this guard and not yet closed.
                unsafe { libc::closedir(self.0) };
            }
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum EntryKind {
        Directory,
        RegularFile,
        Symlink,
        Other,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub(crate) struct DirectoryEntry {
        pub(crate) name: String,
        pub(crate) kind: EntryKind,
        device_id: u64,
        inode: u64,
    }

    impl TrustedAuthorityRoot {
        pub(crate) fn open(raw_path: &Path) -> Result<Self, TrustedFsError> {
            Self::open_for_owner(raw_path, effective_uid())
        }

        pub(crate) fn open_for_owner(
            raw_path: &Path,
            owner_uid: libc::uid_t,
        ) -> Result<Self, TrustedFsError> {
            validate_bootstrap_input(raw_path)?;
            let (parent_chain, name) = open_private_home_parent(raw_path, owner_uid)
                .map_err(private_home_trusted_error)?;
            let parent = parent_chain.final_directory();
            let _lock = TrustedDirectoryLock::acquire(parent)?;
            let file = openat_file(
                parent.as_raw_fd(),
                &name,
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                0,
            )?;
            let stat = fstat(file.as_raw_fd())?;
            validate_private_home_stat(&stat, owner_uid).map_err(private_home_trusted_error)?;
            validate_private_home_acl(file.as_raw_fd()).map_err(private_home_trusted_error)?;
            let root = Self::from_opened(file, owner_uid, raw_path, false)
                .map_err(private_home_trusted_error)?;
            parent_chain
                .revalidate(owner_uid)
                .map_err(private_home_trusted_error)?;
            validate_named_private_home(parent.as_raw_fd(), &name, &stat, owner_uid)
                .map_err(private_home_trusted_error)?;
            Ok(root)
        }

        fn from_opened(
            file: File,
            owner_uid: libc::uid_t,
            requested_home: &Path,
            candidate_created: bool,
        ) -> Result<Self, PrivateHomeError> {
            let error_at_root = |error: PrivateHomeError| {
                error
                    .at_path(requested_home, PrivateHomeObjectRole::FinalRoot)
                    .for_attempt(requested_home, candidate_created)
            };
            let stat = fstat(file.as_raw_fd()).map_err(|_| {
                error_at_root(PrivateHomeError::new(
                    PrivateHomeReason::ValidationUnavailable,
                ))
            })?;
            validate_private_home_stat(&stat, owner_uid).map_err(error_at_root)?;
            validate_private_home_acl(file.as_raw_fd()).map_err(error_at_root)?;
            let opened_physical_path = physical_path_from_handle(&file).map_err(|_| {
                error_at_root(PrivateHomeError::new(
                    PrivateHomeReason::ValidationUnavailable,
                ))
            })?;
            let physical_utf8 = opened_physical_path
                .to_str()
                .filter(|path| path.starts_with('/') && !path.ends_with(" (deleted)"))
                .ok_or_else(|| {
                    error_at_root(PrivateHomeError::new(
                        PrivateHomeReason::ValidationUnavailable,
                    ))
                })?;
            let identity = directory_identity(&file, physical_utf8, &stat).map_err(|_| {
                error_at_root(PrivateHomeError::new(
                    PrivateHomeReason::ValidationUnavailable,
                ))
            })?;
            let root = Self {
                directory: TrustedDirectory {
                    file,
                    device_id: stat.st_dev as u64,
                },
                identity,
                owner_uid,
                requested_home: requested_home.to_path_buf(),
                candidate_created,
            };
            root.revalidate_private_home()?;
            Ok(root)
        }

        pub(crate) fn identity(&self) -> &CanonicalDirectoryV1 {
            &self.identity
        }

        pub(crate) fn revalidate(&self) -> Result<(), TrustedFsError> {
            let stat = fstat(self.directory.file.as_raw_fd())?;
            validate_private_home_stat(&stat, self.owner_uid)
                .map_err(private_home_trusted_error)?;
            validate_private_home_acl(self.directory.file.as_raw_fd())
                .map_err(private_home_trusted_error)?;
            if stat.st_dev as u64 != self.directory.device_id {
                return Err(TrustedFsError::new(
                    "trusted root physical identity changed",
                ));
            }
            let current_path = physical_path_from_handle(&self.directory.file)?;
            if current_path.to_str() != Some(self.identity.physical_path.as_str()) {
                return Err(TrustedFsError::new("trusted root physical path changed"));
            }
            match &self.identity.physical_identity {
                DirectoryPhysicalIdentityV1::Linux { device_id, inode }
                    if *device_id == stat.st_dev as u64 && *inode == stat.st_ino as u64 =>
                {
                    Ok(())
                }
                #[cfg(target_os = "macos")]
                DirectoryPhysicalIdentityV1::MacOs {
                    volume_uuid,
                    file_id,
                    case_sensitive,
                } => {
                    let (current_uuid, current_case_sensitive) =
                        macos_volume_identity(self.directory.file.as_raw_fd())?;
                    if *volume_uuid == current_uuid
                        && *file_id == stat.st_ino as u64
                        && *case_sensitive == current_case_sensitive
                    {
                        Ok(())
                    } else {
                        Err(TrustedFsError::new(
                            "trusted root physical identity changed",
                        ))
                    }
                }
                _ => Err(TrustedFsError::new(
                    "trusted root physical identity changed",
                )),
            }
        }

        pub(crate) fn revalidate_private_home(&self) -> Result<(), PrivateHomeError> {
            let error_at_root = |error: PrivateHomeError| {
                error
                    .at_path(&self.requested_home, PrivateHomeObjectRole::FinalRoot)
                    .for_attempt(&self.requested_home, self.candidate_created)
            };
            let stat = fstat(self.directory.file.as_raw_fd()).map_err(|_| {
                error_at_root(PrivateHomeError::new(
                    PrivateHomeReason::ValidationUnavailable,
                ))
            })?;
            validate_private_home_stat(&stat, self.owner_uid).map_err(error_at_root)?;
            validate_private_home_acl(self.directory.file.as_raw_fd()).map_err(error_at_root)?;
            if stat.st_dev as u64 != self.directory.device_id {
                return Err(error_at_root(PrivateHomeError::new(
                    PrivateHomeReason::Replaced,
                )));
            }
            let current_path = physical_path_from_handle(&self.directory.file).map_err(|_| {
                error_at_root(PrivateHomeError::new(
                    PrivateHomeReason::ValidationUnavailable,
                ))
            })?;
            if current_path.to_str() != Some(self.identity.physical_path.as_str()) {
                return Err(error_at_root(PrivateHomeError::new(
                    PrivateHomeReason::Replaced,
                )));
            }
            let identity_matches = match &self.identity.physical_identity {
                DirectoryPhysicalIdentityV1::Linux { device_id, inode } => {
                    *device_id == stat.st_dev as u64 && *inode == stat.st_ino as u64
                }
                #[cfg(target_os = "macos")]
                DirectoryPhysicalIdentityV1::MacOs {
                    volume_uuid,
                    file_id,
                    case_sensitive,
                } => {
                    let (current_uuid, current_case_sensitive) =
                        macos_volume_identity(self.directory.file.as_raw_fd()).map_err(|_| {
                            error_at_root(PrivateHomeError::new(
                                PrivateHomeReason::ValidationUnavailable,
                            ))
                        })?;
                    *volume_uuid == current_uuid
                        && *file_id == stat.st_ino as u64
                        && *case_sensitive == current_case_sensitive
                }
                _ => false,
            };
            if !identity_matches {
                return Err(error_at_root(PrivateHomeError::new(
                    PrivateHomeReason::Replaced,
                )));
            }
            Ok(())
        }

        pub(crate) fn directory(&self) -> &TrustedDirectory {
            &self.directory
        }

        pub(crate) fn ensure_scaffold_directory(
            &self,
            name: &str,
        ) -> Result<TrustedScaffoldDirectory, TrustedFsError> {
            let root = TrustedScaffoldDirectory {
                file: self
                    .directory
                    .file
                    .try_clone()
                    .map_err(io_error("duplicate private home handle"))?,
                device_id: self.directory.device_id,
                owner_uid: self.owner_uid,
            };
            root.ensure_directory(name)
        }
    }

    impl TrustedWorkspaceRoot {
        pub(crate) fn open_exact(expected: &CanonicalDirectoryV1) -> Result<Self, TrustedFsError> {
            let path = Path::new(&expected.physical_path);
            if !path.is_absolute()
                || path.components().any(|component| {
                    !matches!(component, Component::RootDir | Component::Normal(_))
                })
            {
                return Err(TrustedFsError::new(
                    "workspace root requires an absolute normalized path",
                ));
            }
            let file = OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
                .open(path)
                .map_err(|error| TrustedFsError::from_io("open exact workspace root", error))?;
            let stat = fstat(file.as_raw_fd())?;
            let physical_path = physical_path_from_handle(&file)?;
            let physical_utf8 = physical_path
                .to_str()
                .filter(|path| path.starts_with('/') && !path.ends_with(" (deleted)"))
                .ok_or_else(|| {
                    TrustedFsError::new("opened workspace has no stable UTF-8 physical path")
                })?;
            let identity = directory_identity(&file, physical_utf8, &stat)?;
            if &identity != expected {
                return Err(TrustedFsError::new(
                    "workspace root physical identity does not match request",
                ));
            }
            let opened = Self { file, identity };
            opened.revalidate()?;
            Ok(opened)
        }

        pub(crate) fn revalidate(&self) -> Result<(), TrustedFsError> {
            let stat = fstat(self.file.as_raw_fd())?;
            let physical_path = physical_path_from_handle(&self.file)?;
            if physical_path.to_str() != Some(self.identity.physical_path.as_str())
                || directory_identity(&self.file, &self.identity.physical_path, &stat)?
                    != self.identity
            {
                return Err(TrustedFsError::new(
                    "workspace root physical identity changed",
                ));
            }
            Ok(())
        }
    }

    impl TrustedScaffoldDirectory {
        pub(crate) fn ensure_directory(&self, name: &str) -> Result<Self, TrustedFsError> {
            let name = component(name)?;
            let _lock = TrustedDirectoryLock::acquire(&self.file)?;
            let creation = mkdirat_exact_mode(self.file.as_raw_fd(), &name)
                .map_err(private_home_trusted_error)?;
            let file = openat_file(
                self.file.as_raw_fd(),
                &name,
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                0,
            )?;
            if creation == DirectoryCreation::Created {
                converge_created_owner(file.as_raw_fd(), self.owner_uid)?;
                // SAFETY: legitimate creators serialize on the parent lock, and file is the
                // first no-follow-opened candidate descriptor for this creation attempt.
                if unsafe { libc::fchmod(file.as_raw_fd(), DIRECTORY_MODE) } != 0 {
                    return Err(io_error_value("set private scaffold directory mode"));
                }
            }
            validate_scaffold_directory(&file, self.device_id, self.owner_uid, creation)?;
            let accepted = fstat(file.as_raw_fd())?;
            validate_named_identity(self.file.as_raw_fd(), &name, &accepted)?;
            file.sync_all()
                .map_err(io_error("sync private scaffold directory"))?;
            self.file
                .sync_all()
                .map_err(io_error("sync private scaffold parent"))?;
            Ok(Self {
                file,
                device_id: self.device_id,
                owner_uid: self.owner_uid,
            })
        }

        pub(crate) fn ensure_file_if_missing(
            &self,
            name: &str,
            contents: &[u8],
        ) -> Result<(), TrustedFsError> {
            use std::io::Write as _;

            let name = component(name)?;
            let mut created = match openat_file(
                self.file.as_raw_fd(),
                &name,
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                FILE_MODE,
            ) {
                Ok(file) => file,
                Err(error) if error.is_already_exists() => {
                    let existing = openat_file(
                        self.file.as_raw_fd(),
                        &name,
                        libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                        0,
                    )?;
                    validate_scaffold_file(&existing, self.device_id, self.owner_uid, false)?;
                    return Ok(());
                }
                Err(error) => return Err(error),
            };
            converge_created_owner(created.as_raw_fd(), self.owner_uid)?;
            // SAFETY: created is the O_EXCL file returned by this call.
            if unsafe { libc::fchmod(created.as_raw_fd(), FILE_MODE) } != 0 {
                return Err(io_error_value("set private scaffold file mode"));
            }
            validate_scaffold_file(&created, self.device_id, self.owner_uid, true)?;
            created
                .write_all(contents)
                .map_err(io_error("write private scaffold file"))?;
            created
                .sync_all()
                .map_err(io_error("sync private scaffold file"))?;
            self.file
                .sync_all()
                .map_err(io_error("sync private scaffold parent"))
        }
    }

    pub(crate) fn ensure_private_substrate_home(
        raw_path: &Path,
        owner_uid: libc::uid_t,
    ) -> Result<TrustedAuthorityRoot, PrivateHomeError> {
        ensure_private_substrate_home_with(raw_path, owner_uid, || {}, || {}, || {})
    }

    #[cfg(test)]
    pub(crate) fn ensure_private_substrate_home_with_after_create_for_test(
        raw_path: &Path,
        owner_uid: libc::uid_t,
        after_create: impl FnOnce(),
    ) -> Result<TrustedAuthorityRoot, PrivateHomeError> {
        ensure_private_substrate_home_with(raw_path, owner_uid, after_create, || {}, || {})
    }

    #[cfg(test)]
    pub(crate) fn ensure_private_substrate_home_with_before_from_opened_for_test(
        raw_path: &Path,
        owner_uid: libc::uid_t,
        before_from_opened: impl FnOnce(),
    ) -> Result<TrustedAuthorityRoot, PrivateHomeError> {
        ensure_private_substrate_home_with(raw_path, owner_uid, || {}, || {}, before_from_opened)
    }

    #[cfg_attr(
        target_os = "linux",
        allow(
            clippy::unnecessary_cast,
            reason = "Darwin dev_t and ino_t require normalization to u64"
        )
    )]
    fn ensure_private_substrate_home_with(
        raw_path: &Path,
        owner_uid: libc::uid_t,
        after_create: impl FnOnce(),
        after_first_open: impl FnOnce(),
        before_from_opened: impl FnOnce(),
    ) -> Result<TrustedAuthorityRoot, PrivateHomeError> {
        validate_bootstrap_input(raw_path)
            .map_err(|_| PrivateHomeError::new(PrivateHomeReason::ValidationUnavailable))
            .map_err(|error| {
                error
                    .at_path(raw_path, PrivateHomeObjectRole::FinalRoot)
                    .for_attempt(raw_path, false)
            })?;
        let parent_path = raw_path
            .parent()
            .filter(|parent| *parent != raw_path)
            .ok_or_else(|| PrivateHomeError::new(PrivateHomeReason::MissingParent))
            .map_err(|error| {
                error
                    .at_path(raw_path, PrivateHomeObjectRole::FinalRoot)
                    .for_attempt(raw_path, false)
            })?;
        let name = raw_path
            .file_name()
            .ok_or_else(|| PrivateHomeError::new(PrivateHomeReason::WrongType))
            .map_err(|error| {
                error
                    .at_path(raw_path, PrivateHomeObjectRole::FinalRoot)
                    .for_attempt(raw_path, false)
            })?;
        let name = c_string(name)
            .map_err(|_| PrivateHomeError::new(PrivateHomeReason::ValidationUnavailable))
            .map_err(|error| {
                error
                    .at_path(raw_path, PrivateHomeObjectRole::FinalRoot)
                    .for_attempt(raw_path, false)
            })?;
        let parent_chain = open_private_home_parent_chain(parent_path, owner_uid)
            .map_err(|error| error.for_attempt(raw_path, false))?;
        let parent = parent_chain.final_directory();
        let _lock = TrustedDirectoryLock::acquire(parent)
            .map_err(|_| PrivateHomeError::new(PrivateHomeReason::ValidationUnavailable))
            .map_err(|error| {
                error
                    .at_path(parent_path, PrivateHomeObjectRole::Ancestor)
                    .for_attempt(raw_path, false)
            })?;
        parent_chain
            .revalidate(owner_uid)
            .map_err(|error| error.for_attempt(raw_path, false))?;

        let creation = mkdirat_exact_mode(parent.as_raw_fd(), &name).map_err(|error| {
            error
                .at_path(raw_path, PrivateHomeObjectRole::FinalRoot)
                .for_attempt(raw_path, false)
        })?;
        if creation == DirectoryCreation::Created {
            after_create();
        }
        let candidate_created = creation == DirectoryCreation::Created;

        let opened = openat_file(
            parent.as_raw_fd(),
            &name,
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            0,
        )
        .map_err(|_| {
            private_home_candidate_open_error(parent.as_raw_fd(), &name, owner_uid, creation)
                .at_path(raw_path, PrivateHomeObjectRole::FinalRoot)
                .for_attempt(raw_path, candidate_created)
        })?;

        let opened_stat = fstat(opened.as_raw_fd())
            .map_err(|_| PrivateHomeError::new(PrivateHomeReason::ValidationUnavailable))
            .map_err(|error| {
                error
                    .at_path(raw_path, PrivateHomeObjectRole::FinalRoot)
                    .for_attempt(raw_path, candidate_created)
            })?;
        after_first_open();

        if creation == DirectoryCreation::Created {
            // A1 V1 accepts identity only from this first no-follow-opened descriptor. Legitimate
            // creators serialize on the parent lock; malicious same-UID/root pre-open
            // substitution is outside the threat model and cannot be distinguished portably.
            let effective = effective_uid();
            if effective != owner_uid {
                if effective != 0 {
                    return Err(PrivateHomeError::new(PrivateHomeReason::WrongOwner)
                        .at_path(raw_path, PrivateHomeObjectRole::FinalRoot)
                        .for_attempt(raw_path, candidate_created));
                }
                // SAFETY: opened is the newly created directory and gid -1 preserves its group.
                if unsafe { libc::fchown(opened.as_raw_fd(), owner_uid, !0 as libc::gid_t) } != 0 {
                    return Err(
                        PrivateHomeError::new(PrivateHomeReason::ValidationUnavailable)
                            .at_path(raw_path, PrivateHomeObjectRole::FinalRoot)
                            .for_attempt(raw_path, candidate_created),
                    );
                }
            }
            // SAFETY: opened is the newly created directory; fchmod defeats ambient umask.
            if unsafe { libc::fchmod(opened.as_raw_fd(), DIRECTORY_MODE) } != 0 {
                return Err(
                    PrivateHomeError::new(PrivateHomeReason::ValidationUnavailable)
                        .at_path(raw_path, PrivateHomeObjectRole::FinalRoot)
                        .for_attempt(raw_path, candidate_created),
                );
            }
            opened
                .sync_all()
                .map_err(|_| PrivateHomeError::new(PrivateHomeReason::ValidationUnavailable))
                .map_err(|error| {
                    error
                        .at_path(raw_path, PrivateHomeObjectRole::FinalRoot)
                        .for_attempt(raw_path, candidate_created)
                })?;
            parent
                .sync_all()
                .map_err(|_| PrivateHomeError::new(PrivateHomeReason::ValidationUnavailable))
                .map_err(|error| {
                    error
                        .at_path(parent_path, PrivateHomeObjectRole::Ancestor)
                        .for_attempt(raw_path, candidate_created)
                })?;
        }

        let accepted_stat = fstat(opened.as_raw_fd())
            .map_err(|_| PrivateHomeError::new(PrivateHomeReason::ValidationUnavailable))
            .map_err(|error| {
                error
                    .at_path(raw_path, PrivateHomeObjectRole::FinalRoot)
                    .for_attempt(raw_path, candidate_created)
            })?;
        if accepted_stat.st_dev != opened_stat.st_dev || accepted_stat.st_ino != opened_stat.st_ino
        {
            return Err(PrivateHomeError::new(PrivateHomeReason::Replaced)
                .at_path(raw_path, PrivateHomeObjectRole::FinalRoot)
                .for_attempt(raw_path, candidate_created));
        }
        validate_private_home_stat(&accepted_stat, owner_uid).map_err(|error| {
            error
                .at_path(raw_path, PrivateHomeObjectRole::FinalRoot)
                .for_attempt(raw_path, candidate_created)
        })?;
        validate_private_home_acl(opened.as_raw_fd()).map_err(|error| {
            error
                .at_path(raw_path, PrivateHomeObjectRole::FinalRoot)
                .for_attempt(raw_path, candidate_created)
        })?;
        before_from_opened();
        let root =
            TrustedAuthorityRoot::from_opened(opened, owner_uid, raw_path, candidate_created)?;
        parent_chain
            .revalidate(owner_uid)
            .map_err(|error| error.for_attempt(raw_path, candidate_created))?;
        validate_named_private_home(parent.as_raw_fd(), &name, &accepted_stat, owner_uid).map_err(
            |error| {
                error
                    .at_path(raw_path, PrivateHomeObjectRole::FinalRoot)
                    .for_attempt(raw_path, candidate_created)
            },
        )?;
        Ok(root)
    }

    #[cfg_attr(
        target_os = "linux",
        allow(
            clippy::unnecessary_cast,
            reason = "Darwin dev_t and ino_t require normalization to u64"
        )
    )]
    fn mkdirat_exact_mode(
        parent: RawFd,
        name: &CStr,
    ) -> Result<DirectoryCreation, PrivateHomeError> {
        mkdirat_exact_mode_inner(parent, name, false)
    }

    #[cfg(test)]
    fn mkdirat_exact_mode_with_signaled_child_for_test(
        parent: RawFd,
        name: &CStr,
    ) -> Result<DirectoryCreation, PrivateHomeError> {
        mkdirat_exact_mode_inner(parent, name, true)
    }

    fn mkdirat_exact_mode_inner(
        parent: RawFd,
        name: &CStr,
        _signal_after_create: bool,
    ) -> Result<DirectoryCreation, PrivateHomeError> {
        match fstatat_nofollow(parent, name) {
            Ok(_) => return Ok(DirectoryCreation::Existing),
            Err(error) if error.kind == Some(io::ErrorKind::NotFound) => {}
            Err(_) => {
                return Err(PrivateHomeError::new(
                    PrivateHomeReason::ValidationUnavailable,
                ));
            }
        }
        // `umask` is process-global. Perform the umask-independent creation in a short-lived
        // child so concurrent threads in the parent can never observe a broadened mask. The
        // child calls only async-signal-safe functions between `fork` and `_exit`.
        // SAFETY: fork has no pointer preconditions; the child immediately performs syscalls.
        let child = unsafe { libc::fork() };
        if child < 0 {
            return Err(PrivateHomeError::new(
                PrivateHomeReason::ValidationUnavailable,
            ));
        }
        if child == 0 {
            // SAFETY: umask/mkdirat/_exit are async-signal-safe and pointers reference inherited
            // immutable input. mkdirat establishes only a candidate name, never child identity.
            unsafe {
                libc::umask(0);
                let created = libc::mkdirat(parent, name.as_ptr(), DIRECTORY_MODE) == 0;
                #[cfg(test)]
                if created && _signal_after_create {
                    libc::kill(libc::getpid(), libc::SIGKILL);
                    libc::_exit(127);
                }
                libc::_exit(if created { 0 } else { 1 });
            }
        }

        let mut status = 0;
        loop {
            // SAFETY: child is the positive pid returned by fork and status is writable.
            if unsafe { libc::waitpid(child, &mut status, 0) } == child {
                break;
            }
            if io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
                return Err(private_home_creation_wait_error(parent, name));
            }
        }
        if !libc::WIFEXITED(status) {
            return Err(private_home_creation_wait_error(parent, name));
        }
        match libc::WEXITSTATUS(status) {
            0 => Ok(DirectoryCreation::Created),
            1 if fstatat_nofollow(parent, name).is_ok() => Ok(DirectoryCreation::Existing),
            _ => Err(private_home_creation_wait_error(parent, name)),
        }
    }

    fn private_home_creation_wait_error(parent: RawFd, name: &CStr) -> PrivateHomeError {
        let provenance = match fstatat_nofollow(parent, name) {
            Ok(_) => PrivateHomeCandidateProvenance::Created,
            Err(error) if error.kind == Some(io::ErrorKind::NotFound) => {
                PrivateHomeCandidateProvenance::NotCreated
            }
            Err(_) => PrivateHomeCandidateProvenance::Unknown,
        };
        PrivateHomeError::new(PrivateHomeReason::ValidationUnavailable)
            .with_candidate_provenance(provenance)
    }

    impl TrustedDirectory {
        pub(crate) fn create_directory(&self, name: &str) -> Result<Self, TrustedFsError> {
            let name = component(name)?;
            // SAFETY: parent fd is open and name is a single validated component.
            let result =
                unsafe { libc::mkdirat(self.file.as_raw_fd(), name.as_ptr(), DIRECTORY_MODE) };
            if result != 0 && io::Error::last_os_error().kind() != io::ErrorKind::AlreadyExists {
                return Err(io_error_value("create trusted directory"));
            }
            let child = self.open_directory_cstr(&name)?;
            self.sync()?;
            child.sync()?;
            Ok(child)
        }

        pub(crate) fn create_directory_entry_exclusive(
            &self,
            name: &str,
        ) -> Result<(DirectoryEntry, Self), TrustedFsError> {
            self.create_directory_entry_exclusive_with(name, || {})
        }

        fn create_directory_entry_exclusive_with(
            &self,
            name: &str,
            after_open: impl FnOnce(),
        ) -> Result<(DirectoryEntry, Self), TrustedFsError> {
            let entry_name = name.to_owned();
            let name = component(name)?;
            // SAFETY: parent fd is open and name is a single validated component.
            if unsafe { libc::mkdirat(self.file.as_raw_fd(), name.as_ptr(), DIRECTORY_MODE) } != 0 {
                return Err(io_error_value("exclusively create trusted directory"));
            }
            let child = self.open_directory_cstr(&name)?;
            after_open();
            let child_stat = fstat(child.file.as_raw_fd())?;
            let entry = DirectoryEntry {
                name: entry_name,
                kind: EntryKind::Directory,
                device_id: child_stat.st_dev as u64,
                inode: child_stat.st_ino as u64,
            };
            self.revalidate_entry(&entry)?;
            self.sync()?;
            child.sync()?;
            Ok((entry, child))
        }

        pub(crate) fn open_directory(&self, name: &str) -> Result<Self, TrustedFsError> {
            self.open_directory_cstr(&component(name)?)
        }

        pub(crate) fn open_controlled_directory(&self, name: &str) -> Result<Self, TrustedFsError> {
            let name = component(name)?;
            let file = openat_file(
                self.file.as_raw_fd(),
                &name,
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                0,
            )?;
            validate_open_directory(&file, self.device_id, false)?;
            Ok(Self {
                file,
                device_id: self.device_id,
            })
        }

        #[cfg_attr(
            target_os = "linux",
            allow(
                clippy::unnecessary_cast,
                reason = "Darwin dev_t and ino_t require normalization to u64"
            )
        )]
        pub(crate) fn open_controlled_directory_entry(
            &self,
            expected: &DirectoryEntry,
        ) -> Result<Self, TrustedFsError> {
            if expected.kind != EntryKind::Directory {
                return Err(TrustedFsError::new(
                    "controlled authority entry is not a directory",
                ));
            }
            let current = self
                .stat_entry(&expected.name)?
                .ok_or_else(|| TrustedFsError::new("controlled authority entry disappeared"))?;
            if current.st_dev as u64 != expected.device_id
                || current.st_ino as u64 != expected.inode
                || kind_from_mode(current.st_mode) != EntryKind::Directory
            {
                return Err(TrustedFsError::new(
                    "controlled authority directory changed after enumeration",
                ));
            }
            let opened = self.open_controlled_directory(&expected.name)?;
            let opened_stat = fstat(opened.file.as_raw_fd())?;
            if opened_stat.st_dev as u64 != expected.device_id
                || opened_stat.st_ino as u64 != expected.inode
            {
                return Err(TrustedFsError::new(
                    "opened controlled authority directory changed identity",
                ));
            }
            Ok(opened)
        }

        fn open_directory_cstr(&self, name: &CStr) -> Result<Self, TrustedFsError> {
            let file = openat_file(
                self.file.as_raw_fd(),
                name,
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                0,
            )?;
            validate_open_directory(&file, self.device_id, true)?;
            Ok(Self {
                file,
                device_id: self.device_id,
            })
        }

        pub(crate) fn create_file(&self, name: &str) -> Result<TrustedFile, TrustedFsError> {
            let file = openat_file(
                self.file.as_raw_fd(),
                &component(name)?,
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                FILE_MODE,
            )?;
            validate_open_file(&file, self.device_id)?;
            let stat = fstat(file.as_raw_fd())?;
            Ok(TrustedFile {
                file,
                device_id: stat.st_dev as u64,
                inode: stat.st_ino as u64,
            })
        }

        pub(crate) fn open_file(&self, name: &str) -> Result<TrustedFile, TrustedFsError> {
            let file = openat_file(
                self.file.as_raw_fd(),
                &component(name)?,
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                0,
            )?;
            validate_open_file(&file, self.device_id)?;
            let stat = fstat(file.as_raw_fd())?;
            Ok(TrustedFile {
                file,
                device_id: stat.st_dev as u64,
                inode: stat.st_ino as u64,
            })
        }

        pub(crate) fn open_file_entry(
            &self,
            expected: &DirectoryEntry,
        ) -> Result<TrustedFile, TrustedFsError> {
            if expected.kind != EntryKind::RegularFile {
                return Err(TrustedFsError::new(
                    "controlled authority entry is not a regular file",
                ));
            }
            let opened = self.open_file(&expected.name)?;
            if opened.device_id != expected.device_id || opened.inode != expected.inode {
                return Err(TrustedFsError::new(
                    "opened controlled authority file changed identity",
                ));
            }
            Ok(opened)
        }

        pub(crate) fn entry_kind(&self, name: &str) -> Result<Option<EntryKind>, TrustedFsError> {
            Ok(self
                .stat_entry(name)?
                .map(|stat| kind_from_mode(stat.st_mode)))
        }

        fn stat_entry(&self, name: &str) -> Result<Option<libc::stat>, TrustedFsError> {
            let mut stat = MaybeUninit::<libc::stat>::uninit();
            // SAFETY: output points to valid storage; name is a validated component.
            let result = unsafe {
                libc::fstatat(
                    self.file.as_raw_fd(),
                    component(name)?.as_ptr(),
                    stat.as_mut_ptr(),
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            };
            if result != 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::NotFound {
                    return Ok(None);
                }
                return Err(TrustedFsError::new(format!("stat trusted entry: {error}")));
            }
            // SAFETY: fstatat initialized output on success.
            // SAFETY: fstatat initialized output on success.
            Ok(Some(unsafe { stat.assume_init() }))
        }

        #[cfg_attr(
            target_os = "linux",
            allow(
                clippy::unnecessary_cast,
                reason = "Darwin dev_t and ino_t require normalization to u64"
            )
        )]
        pub(crate) fn revalidate_entry(
            &self,
            expected: &DirectoryEntry,
        ) -> Result<(), TrustedFsError> {
            let current = self
                .stat_entry(&expected.name)?
                .ok_or_else(|| TrustedFsError::new("scanned authority entry disappeared"))?;
            if kind_from_mode(current.st_mode) != expected.kind
                || current.st_dev as u64 != expected.device_id
                || current.st_ino as u64 != expected.inode
            {
                return Err(TrustedFsError::new(
                    "authority entry changed after safe enumeration",
                ));
            }
            let opened = match expected.kind {
                EntryKind::Directory => {
                    let opened = self.open_directory(&expected.name)?;
                    fstat(opened.file.as_raw_fd())?
                }
                EntryKind::RegularFile => {
                    let opened = self.open_file(&expected.name)?;
                    fstat(opened.file.as_raw_fd())?
                }
                EntryKind::Symlink | EntryKind::Other => {
                    return Err(TrustedFsError::new(
                        "unsafe authority entry cannot be revalidated",
                    ));
                }
            };
            if opened.st_dev as u64 != expected.device_id || opened.st_ino as u64 != expected.inode
            {
                return Err(TrustedFsError::new(
                    "opened authority entry differs from scanned identity",
                ));
            }
            Ok(())
        }

        pub(crate) fn entries(&self) -> Result<Vec<DirectoryEntry>, TrustedFsError> {
            let fresh = openat_file(
                self.file.as_raw_fd(),
                c".",
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                0,
            )?;
            // SAFETY: the raw descriptor ownership transfers to fdopendir on success.
            let fresh_fd = fresh.into_raw_fd();
            // SAFETY: raw descriptor ownership transfers to fdopendir on success.
            let stream = unsafe { libc::fdopendir(fresh_fd) };
            if stream.is_null() {
                // SAFETY: fdopendir did not consume the descriptor on failure.
                unsafe { libc::close(fresh_fd) };
                return Err(io_error_value("enumerate trusted directory"));
            }
            let stream = DirectoryStream(stream);
            let mut entries = Vec::new();
            loop {
                clear_errno();
                // SAFETY: stream is live until closed below.
                let entry = unsafe { libc::readdir(stream.0) };
                if entry.is_null() {
                    let error = io::Error::last_os_error();
                    if error.raw_os_error() != Some(0) {
                        return Err(TrustedFsError::new(format!(
                            "enumerate trusted directory: {error}"
                        )));
                    }
                    break;
                }
                // SAFETY: d_name is NUL-terminated within dirent.
                let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) };
                if name.to_bytes() == b"." || name.to_bytes() == b".." {
                    continue;
                }
                let name = std::str::from_utf8(name.to_bytes())
                    .map_err(|_| TrustedFsError::new("trusted directory entry is not UTF-8"))?
                    .to_string();
                let stat = self
                    .stat_entry(&name)?
                    .ok_or_else(|| TrustedFsError::new("trusted directory entry disappeared"))?;
                if stat.st_dev as u64 != self.device_id {
                    return Err(TrustedFsError::new(
                        "trusted directory contains cross-filesystem entry",
                    ));
                }
                entries.push(DirectoryEntry {
                    name,
                    kind: kind_from_mode(stat.st_mode),
                    device_id: stat.st_dev as u64,
                    inode: stat.st_ino as u64,
                });
            }
            entries.sort_by(|left, right| left.name.as_bytes().cmp(right.name.as_bytes()));
            stream.close()?;
            Ok(entries)
        }

        pub(crate) fn unlink_file(&self, name: &str) -> Result<(), TrustedFsError> {
            // SAFETY: descriptor and component are valid; flags select non-directory removal.
            if unsafe { libc::unlinkat(self.file.as_raw_fd(), component(name)?.as_ptr(), 0) } != 0 {
                return Err(io_error_value("unlink trusted file"));
            }
            self.sync()
        }

        pub(crate) fn rename_no_replace(
            &self,
            source: &str,
            expected_source: TrustedFile,
            destination: &TrustedDirectory,
            target: &str,
        ) -> Result<(), TrustedFsError> {
            if self.device_id != destination.device_id {
                return Err(TrustedFsError::new("cross-filesystem authority rename"));
            }
            expected_source.sync()?;
            self.verify_named_file_identity(source, &expected_source)?;
            // SAFETY: descriptors and validated component pointers remain live for the syscall.
            let result = rename_no_replace_at(
                self.file.as_raw_fd(),
                &component(source)?,
                destination.file.as_raw_fd(),
                &component(target)?,
            );
            if result != 0 {
                return Err(io_error_value("publish trusted file without replacement"));
            }
            destination.verify_named_file_identity(target, &expected_source)?;
            destination.sync()
        }

        pub(crate) fn rename_replace(
            &self,
            source: &str,
            expected_source: TrustedFile,
            destination: &TrustedDirectory,
            target: &str,
        ) -> Result<(), TrustedFsError> {
            if self.device_id != destination.device_id {
                return Err(TrustedFsError::new("cross-filesystem authority rename"));
            }
            expected_source.sync()?;
            self.verify_named_file_identity(source, &expected_source)?;
            // SAFETY: descriptors and component pointers are valid for renameat.
            if unsafe {
                libc::renameat(
                    self.file.as_raw_fd(),
                    component(source)?.as_ptr(),
                    destination.file.as_raw_fd(),
                    component(target)?.as_ptr(),
                )
            } != 0
            {
                return Err(io_error_value("replace trusted file"));
            }
            destination.verify_named_file_identity(target, &expected_source)?;
            destination.sync()
        }

        fn verify_named_file_identity(
            &self,
            name: &str,
            expected: &TrustedFile,
        ) -> Result<(), TrustedFsError> {
            let opened = self.open_file(name)?;
            if opened.device_id != expected.device_id || opened.inode != expected.inode {
                return Err(TrustedFsError::new(
                    "authority publication source identity changed",
                ));
            }
            Ok(())
        }

        pub(crate) fn sync(&self) -> Result<(), TrustedFsError> {
            self.file
                .sync_all()
                .map_err(io_error("sync trusted directory"))
        }
    }

    impl TrustedFile {
        pub(crate) fn write_all(&mut self, bytes: &[u8]) -> Result<(), TrustedFsError> {
            use std::io::Write as _;

            self.file
                .write_all(bytes)
                .map_err(io_error("write trusted file"))
        }

        pub(crate) fn read_all(&self) -> Result<Vec<u8>, TrustedFsError> {
            use std::io::Read as _;

            let mut bytes = Vec::new();
            (&self.file)
                .read_to_end(&mut bytes)
                .map_err(io_error("read trusted file"))?;
            Ok(bytes)
        }

        pub(crate) fn sync(&self) -> Result<(), TrustedFsError> {
            self.file.sync_all().map_err(io_error("sync trusted file"))
        }

        pub(crate) fn lock_exclusive(&self) -> Result<TrustedFileLock<'_>, TrustedFsError> {
            self.lock_with_flags(libc::LOCK_EX)?.ok_or_else(|| {
                TrustedFsError::new("blocking trusted lock unexpectedly unavailable")
            })
        }

        pub(crate) fn lock_exclusive_owned(&self) -> Result<TrustedOwnedFileLock, TrustedFsError> {
            let owned = self
                .file
                .try_clone()
                .map_err(io_error("duplicate trusted lock descriptor"))?;
            // SAFETY: both descriptors reference the same live open-file description.
            if unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_EX) } != 0 {
                return Err(io_error_value("acquire owned trusted file lock"));
            }
            Ok(TrustedOwnedFileLock { file: Some(owned) })
        }

        pub(crate) fn try_lock_exclusive(
            &self,
        ) -> Result<Option<TrustedFileLock<'_>>, TrustedFsError> {
            self.lock_with_flags(libc::LOCK_EX | libc::LOCK_NB)
        }

        fn lock_with_flags(
            &self,
            flags: libc::c_int,
        ) -> Result<Option<TrustedFileLock<'_>>, TrustedFsError> {
            // SAFETY: descriptor is live and flock accepts these flags.
            if unsafe { libc::flock(self.file.as_raw_fd(), flags) } == 0 {
                return Ok(Some(TrustedFileLock { file: Some(self) }));
            }
            let error = io::Error::last_os_error();
            if flags & libc::LOCK_NB != 0 && error.raw_os_error() == Some(libc::EWOULDBLOCK) {
                return Ok(None);
            }
            Err(TrustedFsError::new(format!(
                "acquire trusted file lock: {error}"
            )))
        }
    }

    fn validate_bootstrap_input(path: &Path) -> Result<(), TrustedFsError> {
        if !path.is_absolute() || path.to_str().is_none() {
            return Err(TrustedFsError::new(
                "trusted root input must be absolute UTF-8",
            ));
        }
        if path
            .as_os_str()
            .as_bytes()
            .split(|byte| *byte == b'/')
            .any(|component| component == b"." || component == b"..")
        {
            return Err(TrustedFsError::new(
                "trusted root input contains dot component",
            ));
        }
        Ok(())
    }

    fn open_physical_directory(path: &Path) -> Result<File, TrustedFsError> {
        // SAFETY: the static slash path is valid and flags require a directory.
        let mut current = owned_file(
            unsafe {
                libc::open(
                    c"/".as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                )
            },
            "open physical root",
        )?;
        for component_value in path.components() {
            let Component::Normal(component_value) = component_value else {
                continue;
            };
            let component = c_string(component_value)?;
            let observed = fstatat_nofollow(current.as_raw_fd(), &component)?;
            if kind_from_mode(observed.st_mode) == EntryKind::Symlink {
                return Err(TrustedFsError::new("trusted root path contains a symlink"));
            }
            let next = openat_file(
                current.as_raw_fd(),
                &component,
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                0,
            )?;
            let stat = fstat(next.as_raw_fd())?;
            if kind_from_mode(stat.st_mode) != EntryKind::Directory || stat.st_mode & 0o022 != 0 {
                return Err(TrustedFsError::new(
                    "physical root ancestor is writable or not a directory",
                ));
            }
            validate_acl(next.as_raw_fd())?;
            current = next;
        }
        Ok(current)
    }

    fn fstatat_nofollow(parent: RawFd, name: &CStr) -> Result<libc::stat, TrustedFsError> {
        let mut stat = MaybeUninit::<libc::stat>::uninit();
        // SAFETY: parent/name are live and stat points to writable storage.
        if unsafe {
            libc::fstatat(
                parent,
                name.as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } != 0
        {
            return Err(io_error_value("inspect trusted path component"));
        }
        // SAFETY: successful fstatat initialized stat.
        Ok(unsafe { stat.assume_init() })
    }

    fn component(value: &str) -> Result<CString, TrustedFsError> {
        if value.is_empty() || value == "." || value == ".." || value.as_bytes().contains(&b'/') {
            return Err(TrustedFsError::new(
                "authority name is not one path component",
            ));
        }
        CString::new(value).map_err(|_| TrustedFsError::new("authority name contains NUL"))
    }

    fn c_string(value: &OsStr) -> Result<CString, TrustedFsError> {
        CString::new(value.as_bytes()).map_err(|_| TrustedFsError::new("path contains NUL"))
    }

    fn openat_file(
        parent: RawFd,
        name: &CStr,
        flags: libc::c_int,
        mode: libc::mode_t,
    ) -> Result<File, TrustedFsError> {
        // SAFETY: parent is live and name is NUL-terminated; mode is used only with O_CREAT.
        owned_file(
            unsafe { libc::openat(parent, name.as_ptr(), flags, mode as libc::c_uint) },
            "open trusted entry",
        )
    }

    fn owned_file(fd: RawFd, operation: &str) -> Result<File, TrustedFsError> {
        if fd < 0 {
            return Err(io_error_value(operation));
        }
        // SAFETY: fd is newly returned and ownership transfers to File exactly once.
        Ok(unsafe { File::from_raw_fd(fd) })
    }

    fn fstat(fd: RawFd) -> Result<libc::stat, TrustedFsError> {
        let mut stat = MaybeUninit::<libc::stat>::uninit();
        // SAFETY: stat points to valid output storage.
        if unsafe { libc::fstat(fd, stat.as_mut_ptr()) } != 0 {
            return Err(io_error_value("fstat trusted entry"));
        }
        // SAFETY: fstat initialized output on success.
        Ok(unsafe { stat.assume_init() })
    }

    #[cfg(target_os = "linux")]
    fn clear_errno() {
        // SAFETY: this thread owns its errno location.
        unsafe { *libc::__errno_location() = 0 };
    }

    #[cfg(target_os = "macos")]
    fn clear_errno() {
        // SAFETY: this thread owns its errno location.
        unsafe { *libc::__error() = 0 };
    }

    #[cfg(target_os = "linux")]
    fn physical_path_from_handle(file: &File) -> Result<std::path::PathBuf, TrustedFsError> {
        fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd()))
            .map_err(io_error("derive trusted root physical path from handle"))
    }

    #[cfg(target_os = "macos")]
    fn physical_path_from_handle(file: &File) -> Result<std::path::PathBuf, TrustedFsError> {
        let mut bytes = [0_u8; libc::PATH_MAX as usize];
        // SAFETY: buffer is writable PATH_MAX storage and fd is live.
        if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETPATH, bytes.as_mut_ptr()) } != 0 {
            return Err(io_error_value(
                "derive trusted root physical path from handle",
            ));
        }
        let length = bytes.iter().position(|byte| *byte == 0).ok_or_else(|| {
            TrustedFsError::new("opened trusted root physical path is unterminated")
        })?;
        Ok(std::path::PathBuf::from(OsStr::from_bytes(
            &bytes[..length],
        )))
    }

    #[cfg(target_os = "linux")]
    fn directory_identity(
        _file: &File,
        physical_path: &str,
        stat: &libc::stat,
    ) -> Result<CanonicalDirectoryV1, TrustedFsError> {
        Ok(CanonicalDirectoryV1 {
            physical_path: physical_path.to_string(),
            physical_identity: DirectoryPhysicalIdentityV1::Linux {
                device_id: stat.st_dev,
                inode: stat.st_ino,
            },
        })
    }

    #[cfg(target_os = "macos")]
    fn directory_identity(
        file: &File,
        physical_path: &str,
        stat: &libc::stat,
    ) -> Result<CanonicalDirectoryV1, TrustedFsError> {
        let (volume_uuid, case_sensitive) = macos_volume_identity(file.as_raw_fd())?;
        Ok(CanonicalDirectoryV1 {
            physical_path: physical_path.to_string(),
            physical_identity: DirectoryPhysicalIdentityV1::MacOs {
                volume_uuid,
                file_id: stat.st_ino as u64,
                case_sensitive,
            },
        })
    }

    #[cfg(target_os = "linux")]
    fn rename_no_replace_at(
        source_dir: RawFd,
        source: &CStr,
        destination_dir: RawFd,
        target: &CStr,
    ) -> libc::c_long {
        // SAFETY: descriptors and component pointers remain live for the syscall.
        unsafe {
            libc::syscall(
                libc::SYS_renameat2,
                source_dir,
                source.as_ptr(),
                destination_dir,
                target.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        }
    }

    #[cfg(target_os = "macos")]
    fn rename_no_replace_at(
        source_dir: RawFd,
        source: &CStr,
        destination_dir: RawFd,
        target: &CStr,
    ) -> libc::c_long {
        // SAFETY: descriptors and component pointers remain live for the call.
        unsafe {
            libc::renameatx_np(
                source_dir,
                source.as_ptr(),
                destination_dir,
                target.as_ptr(),
                libc::RENAME_EXCL,
            ) as libc::c_long
        }
    }

    #[cfg(target_os = "macos")]
    fn macos_volume_identity(fd: RawFd) -> Result<(String, bool), TrustedFsError> {
        #[repr(C)]
        struct VolumeResult {
            length: u32,
            capabilities: libc::vol_capabilities_attr_t,
            uuid: libc::uuid_t,
        }
        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: 0,
            volattr: libc::ATTR_VOL_INFO | libc::ATTR_VOL_CAPABILITIES | libc::ATTR_VOL_UUID,
            dirattr: 0,
            fileattr: 0,
            forkattr: 0,
        };
        let mut result = MaybeUninit::<VolumeResult>::zeroed();
        // SAFETY: structures match Darwin getattrlist ABI and buffer has exact capacity.
        if unsafe {
            libc::fgetattrlist(
                fd,
                (&mut attributes as *mut libc::attrlist).cast(),
                result.as_mut_ptr().cast(),
                std::mem::size_of::<VolumeResult>(),
                0,
            )
        } != 0
        {
            return Err(io_error_value("derive trusted volume identity"));
        }
        // SAFETY: fgetattrlist initialized the full fixed result on success.
        let result = unsafe { result.assume_init() };
        if result.length as usize != std::mem::size_of::<VolumeResult>()
            || result.capabilities.valid[0] & libc::VOL_CAP_FMT_CASE_SENSITIVE == 0
        {
            return Err(TrustedFsError::new(
                "trusted volume did not report required capabilities",
            ));
        }
        let uuid = result
            .uuid
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        Ok((
            uuid,
            result.capabilities.capabilities[0] & libc::VOL_CAP_FMT_CASE_SENSITIVE != 0,
        ))
    }

    fn validate_open_directory(
        file: &File,
        expected_device: u64,
        exact_mode: bool,
    ) -> Result<libc::stat, TrustedFsError> {
        let stat = fstat(file.as_raw_fd())?;
        if kind_from_mode(stat.st_mode) != EntryKind::Directory
            || stat.st_uid != effective_uid()
            || stat.st_dev as u64 != expected_device
            || (exact_mode && stat.st_mode & 0o7777 != DIRECTORY_MODE)
            || (!exact_mode && stat.st_mode & 0o022 != 0)
        {
            return Err(TrustedFsError::new(
                "trusted directory type/owner/mode/device mismatch",
            ));
        }
        validate_acl(file.as_raw_fd())?;
        Ok(stat)
    }

    fn open_private_home_parent(
        raw_path: &Path,
        owner_uid: libc::uid_t,
    ) -> Result<(TrustedDirectoryChain, CString), PrivateHomeError> {
        validate_bootstrap_input(raw_path)
            .map_err(|_| PrivateHomeError::new(PrivateHomeReason::ValidationUnavailable))?;
        let parent_path = raw_path
            .parent()
            .filter(|parent| *parent != raw_path)
            .ok_or_else(|| PrivateHomeError::new(PrivateHomeReason::MissingParent))?;
        let name = raw_path
            .file_name()
            .ok_or_else(|| PrivateHomeError::new(PrivateHomeReason::WrongType))?;
        let name = c_string(name)
            .map_err(|_| PrivateHomeError::new(PrivateHomeReason::ValidationUnavailable))?;
        let parent = open_private_home_parent_chain(parent_path, owner_uid)?;
        Ok((parent, name))
    }

    fn open_private_home_parent_chain(
        parent_path: &Path,
        owner_uid: libc::uid_t,
    ) -> Result<TrustedDirectoryChain, PrivateHomeError> {
        // SAFETY: the static slash path is valid and flags require a directory.
        let root = owned_file(
            unsafe {
                libc::open(
                    c"/".as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                )
            },
            "open private home path root",
        )
        .map_err(private_home_parent_open_error)
        .map_err(|error| error.at_path(Path::new("/"), PrivateHomeObjectRole::Ancestor))?;
        let root_stat = validate_private_home_ancestor(&root, owner_uid)
            .map_err(|error| error.at_path(Path::new("/"), PrivateHomeObjectRole::Ancestor))?;
        let mut chain = TrustedDirectoryChain {
            directories: vec![root],
            components: Vec::new(),
            identities: vec![(root_stat.st_dev as u64, root_stat.st_ino as u64)],
        };

        for component_value in parent_path.components() {
            let Component::Normal(component_value) = component_value else {
                continue;
            };
            let path = chain
                .path_at(chain.directories.len() - 1)
                .join(component_value);
            let component = c_string(component_value).map_err(|_| {
                PrivateHomeError::new(PrivateHomeReason::ValidationUnavailable)
                    .at_path(&path, PrivateHomeObjectRole::Ancestor)
            })?;
            let observed = fstatat_nofollow(chain.final_directory().as_raw_fd(), &component)
                .map_err(private_home_parent_open_error)
                .map_err(|error| error.at_path(&path, PrivateHomeObjectRole::Ancestor))?;
            match kind_from_mode(observed.st_mode) {
                EntryKind::Directory => {}
                EntryKind::Symlink => {
                    return Err(PrivateHomeError::new(PrivateHomeReason::Symlink)
                        .at_path(&path, PrivateHomeObjectRole::Ancestor));
                }
                EntryKind::RegularFile | EntryKind::Other => {
                    return Err(PrivateHomeError::new(PrivateHomeReason::WrongType)
                        .at_path(&path, PrivateHomeObjectRole::Ancestor));
                }
            }
            let next = openat_file(
                chain.final_directory().as_raw_fd(),
                &component,
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                0,
            )
            .map_err(private_home_parent_open_error)
            .map_err(|error| error.at_path(&path, PrivateHomeObjectRole::Ancestor))?;
            let opened = validate_private_home_ancestor(&next, owner_uid)
                .map_err(|error| error.at_path(&path, PrivateHomeObjectRole::Ancestor))?;
            if opened.st_dev != observed.st_dev || opened.st_ino != observed.st_ino {
                return Err(PrivateHomeError::new(PrivateHomeReason::Replaced)
                    .at_path(&path, PrivateHomeObjectRole::Ancestor));
            }
            chain.components.push(component);
            chain
                .identities
                .push((opened.st_dev as u64, opened.st_ino as u64));
            chain.directories.push(next);
        }
        chain.revalidate(owner_uid)?;
        Ok(chain)
    }

    fn private_home_parent_open_error(error: TrustedFsError) -> PrivateHomeError {
        let reason = if error.kind == Some(io::ErrorKind::NotFound) {
            PrivateHomeReason::MissingParent
        } else if error.message.contains("symlink") {
            PrivateHomeReason::Symlink
        } else if error.message.contains("writable") {
            PrivateHomeReason::WrongMode
        } else if error.kind == Some(io::ErrorKind::NotADirectory)
            || error.message.contains("not a directory")
        {
            PrivateHomeReason::WrongType
        } else if error.message.contains("ACL")
            || error.message.contains("another principal")
            || error.message.contains("unexpected")
        {
            PrivateHomeReason::ForeignAcl
        } else {
            PrivateHomeReason::ValidationUnavailable
        };
        PrivateHomeError::new(reason)
    }

    fn validate_private_home_parent(
        parent: &File,
        owner_uid: libc::uid_t,
    ) -> Result<(u64, u64), PrivateHomeError> {
        let stat = validate_private_home_ancestor(parent, owner_uid)?;
        validate_private_home_parent_acl(parent.as_raw_fd())?;
        Ok((stat.st_dev as u64, stat.st_ino as u64))
    }

    fn validate_private_home_ancestor(
        directory: &File,
        owner_uid: libc::uid_t,
    ) -> Result<libc::stat, PrivateHomeError> {
        let stat = fstat(directory.as_raw_fd())
            .map_err(|_| PrivateHomeError::new(PrivateHomeReason::ValidationUnavailable))?;
        if kind_from_mode(stat.st_mode) != EntryKind::Directory {
            return Err(PrivateHomeError::new(PrivateHomeReason::WrongType));
        }
        // Root and the intended per-user owner are the only expected parent owners. Root is
        // outside the A1 V1 adversary model; any other owner could replace the child.
        if stat.st_uid != 0 && stat.st_uid != owner_uid {
            return Err(PrivateHomeError::new(PrivateHomeReason::WrongOwner));
        }
        validate_private_home_parent_acl(directory.as_raw_fd())?;
        if stat.st_mode & 0o022 != 0 {
            return Err(PrivateHomeError::new(PrivateHomeReason::WrongMode));
        }
        Ok(stat)
    }

    fn validate_named_identity(
        parent: RawFd,
        name: &CStr,
        accepted: &libc::stat,
    ) -> Result<(), TrustedFsError> {
        let named = fstatat_nofollow(parent, name)?;
        if named.st_dev != accepted.st_dev || named.st_ino != accepted.st_ino {
            return Err(TrustedFsError::new(
                "trusted directory name no longer joins accepted descriptor identity",
            ));
        }
        Ok(())
    }

    fn validate_named_private_home(
        parent: RawFd,
        name: &CStr,
        accepted: &libc::stat,
        owner_uid: libc::uid_t,
    ) -> Result<(), PrivateHomeError> {
        let named = fstatat_nofollow(parent, name)
            .map_err(|_| PrivateHomeError::new(PrivateHomeReason::Replaced))?;
        if kind_from_mode(named.st_mode) == EntryKind::Symlink {
            return Err(PrivateHomeError::new(PrivateHomeReason::Symlink));
        }
        validate_private_home_stat(&named, owner_uid)?;
        if named.st_dev != accepted.st_dev || named.st_ino != accepted.st_ino {
            return Err(PrivateHomeError::new(PrivateHomeReason::Replaced));
        }
        Ok(())
    }

    fn private_home_candidate_open_error(
        parent: RawFd,
        name: &CStr,
        owner_uid: libc::uid_t,
        creation: DirectoryCreation,
    ) -> PrivateHomeError {
        let reason = match fstatat_nofollow(parent, name) {
            Ok(stat) => private_home_reason_from_stat(&stat, owner_uid),
            Err(_) if creation == DirectoryCreation::Created => PrivateHomeReason::Replaced,
            Err(_) => PrivateHomeReason::ValidationUnavailable,
        };
        PrivateHomeError::new(reason)
    }

    fn validate_private_home_stat(
        stat: &libc::stat,
        owner_uid: libc::uid_t,
    ) -> Result<(), PrivateHomeError> {
        if kind_from_mode(stat.st_mode) != EntryKind::Directory {
            return Err(PrivateHomeError::new(PrivateHomeReason::WrongType));
        }
        if stat.st_uid != owner_uid {
            return Err(PrivateHomeError::new(PrivateHomeReason::WrongOwner));
        }
        if stat.st_mode & 0o7777 != DIRECTORY_MODE {
            return Err(PrivateHomeError::new(PrivateHomeReason::WrongMode));
        }
        Ok(())
    }

    fn converge_created_owner(fd: RawFd, owner_uid: libc::uid_t) -> Result<(), TrustedFsError> {
        let effective = effective_uid();
        if effective == owner_uid {
            return Ok(());
        }
        if effective != 0 {
            return Err(TrustedFsError::new(
                "created private entry has the wrong owner",
            ));
        }
        // SAFETY: fd is an identity-bound entry created by this call; gid -1 is preserved.
        if unsafe { libc::fchown(fd, owner_uid, !0 as libc::gid_t) } != 0 {
            return Err(io_error_value("set created private entry owner"));
        }
        Ok(())
    }

    fn validate_scaffold_directory(
        file: &File,
        expected_device: u64,
        owner_uid: libc::uid_t,
        creation: DirectoryCreation,
    ) -> Result<(), TrustedFsError> {
        let stat = fstat(file.as_raw_fd())?;
        let mode_valid = match creation {
            DirectoryCreation::Created => stat.st_mode & 0o7777 == DIRECTORY_MODE,
            DirectoryCreation::Existing => stat.st_mode & 0o7022 == 0,
        };
        if kind_from_mode(stat.st_mode) != EntryKind::Directory
            || stat.st_uid != owner_uid
            || stat.st_dev as u64 != expected_device
            || !mode_valid
        {
            return Err(TrustedFsError::new(
                "private scaffold directory type/owner/mode/device mismatch",
            ));
        }
        validate_private_home_acl(file.as_raw_fd()).map_err(private_home_trusted_error)
    }

    fn validate_scaffold_file(
        file: &File,
        expected_device: u64,
        owner_uid: libc::uid_t,
        exact_mode: bool,
    ) -> Result<(), TrustedFsError> {
        let stat = fstat(file.as_raw_fd())?;
        let mode_valid = if exact_mode {
            stat.st_mode & 0o7777 == FILE_MODE
        } else {
            stat.st_mode & 0o7022 == 0
        };
        if kind_from_mode(stat.st_mode) != EntryKind::RegularFile
            || stat.st_uid != owner_uid
            || stat.st_dev as u64 != expected_device
            || !mode_valid
        {
            return Err(TrustedFsError::new(
                "private scaffold file type/owner/mode/device mismatch",
            ));
        }
        validate_acl(file.as_raw_fd())
    }

    fn private_home_reason_from_stat(
        stat: &libc::stat,
        owner_uid: libc::uid_t,
    ) -> PrivateHomeReason {
        match kind_from_mode(stat.st_mode) {
            EntryKind::Symlink => PrivateHomeReason::Symlink,
            EntryKind::Directory if stat.st_uid != owner_uid => PrivateHomeReason::WrongOwner,
            EntryKind::Directory if stat.st_mode & 0o7777 != DIRECTORY_MODE => {
                PrivateHomeReason::WrongMode
            }
            EntryKind::Directory => PrivateHomeReason::ValidationUnavailable,
            EntryKind::RegularFile | EntryKind::Other => PrivateHomeReason::WrongType,
        }
    }

    fn validate_private_home_acl(fd: RawFd) -> Result<(), PrivateHomeError> {
        #[cfg(target_os = "linux")]
        {
            for (name, kind, authority) in [
                (
                    c"system.posix_acl_access",
                    PrivateHomeAclKind::Access,
                    PrivateHomeAclAuthority::ExtendedAccessAcl,
                ),
                (
                    c"system.posix_acl_default",
                    PrivateHomeAclKind::Default,
                    PrivateHomeAclAuthority::DefaultAclPresent,
                ),
            ] {
                let state = read_linux_acl_xattr(fd, name).map_err(|_| {
                    PrivateHomeError::acl(
                        PrivateHomeReason::ValidationUnavailable,
                        PrivateHomeAclKind::Unavailable,
                        PrivateHomeAclAuthority::Unavailable,
                    )
                })?;
                let LinuxAclXattr::Present(bytes) = state else {
                    continue;
                };
                if parse_parent_acl(&bytes).is_err() {
                    return Err(PrivateHomeError::acl(
                        PrivateHomeReason::ForeignAcl,
                        kind,
                        PrivateHomeAclAuthority::Malformed,
                    ));
                }
                return Err(PrivateHomeError::acl(
                    PrivateHomeReason::ForeignAcl,
                    kind,
                    authority,
                ));
            }
        }
        #[cfg(target_os = "macos")]
        validate_macos_acl(fd, true)
            .map_err(|_| PrivateHomeError::new(PrivateHomeReason::ForeignAcl))?;
        Ok(())
    }

    fn validate_private_home_parent_acl(fd: RawFd) -> Result<(), PrivateHomeError> {
        #[cfg(target_os = "linux")]
        {
            let access = read_linux_acl_xattr(fd, c"system.posix_acl_access").map_err(|_| {
                PrivateHomeError::acl(
                    PrivateHomeReason::ValidationUnavailable,
                    PrivateHomeAclKind::Unavailable,
                    PrivateHomeAclAuthority::Unavailable,
                )
            })?;
            if let LinuxAclXattr::Present(bytes) = access {
                match parse_parent_acl(&bytes) {
                    Ok(false) => {}
                    Ok(true) => {
                        return Err(PrivateHomeError::acl(
                            PrivateHomeReason::ForeignAcl,
                            PrivateHomeAclKind::Access,
                            PrivateHomeAclAuthority::EffectiveWrite,
                        ));
                    }
                    Err(()) => {
                        return Err(PrivateHomeError::acl(
                            PrivateHomeReason::ForeignAcl,
                            PrivateHomeAclKind::Access,
                            PrivateHomeAclAuthority::Malformed,
                        ));
                    }
                }
            }

            let default = read_linux_acl_xattr(fd, c"system.posix_acl_default").map_err(|_| {
                PrivateHomeError::acl(
                    PrivateHomeReason::ValidationUnavailable,
                    PrivateHomeAclKind::Unavailable,
                    PrivateHomeAclAuthority::Unavailable,
                )
            })?;
            if let LinuxAclXattr::Present(bytes) = default {
                let authority = if parse_parent_acl(&bytes).is_ok() {
                    PrivateHomeAclAuthority::DefaultAclPresent
                } else {
                    PrivateHomeAclAuthority::Malformed
                };
                return Err(PrivateHomeError::acl(
                    PrivateHomeReason::ForeignAcl,
                    PrivateHomeAclKind::Default,
                    authority,
                ));
            }
        }
        #[cfg(target_os = "macos")]
        validate_macos_acl(fd, false)
            .map_err(|_| PrivateHomeError::new(PrivateHomeReason::ForeignAcl))?;
        Ok(())
    }

    fn private_home_trusted_error(error: PrivateHomeError) -> TrustedFsError {
        TrustedFsError::new(format!(
            "private SUBSTRATE_HOME validation failed: {}",
            error.reason().as_str()
        ))
    }

    fn validate_open_file(file: &File, expected_device: u64) -> Result<(), TrustedFsError> {
        let stat = fstat(file.as_raw_fd())?;
        if kind_from_mode(stat.st_mode) != EntryKind::RegularFile
            || stat.st_uid != effective_uid()
            || stat.st_dev as u64 != expected_device
            || stat.st_mode & 0o7777 != FILE_MODE
        {
            return Err(TrustedFsError::new(
                "trusted file type/owner/mode/device mismatch",
            ));
        }
        validate_acl(file.as_raw_fd())
    }

    #[cfg(target_os = "linux")]
    fn validate_acl(fd: RawFd) -> Result<(), TrustedFsError> {
        let name = c"system.posix_acl_access";
        // SAFETY: fd is live; null buffer with zero length queries the xattr size.
        let size = unsafe { libc::fgetxattr(fd, name.as_ptr(), std::ptr::null_mut(), 0) };
        if size < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ENODATA) {
                return Ok(());
            }
            return Err(TrustedFsError::new(format!("read trusted ACL: {error}")));
        }
        let mut bytes = vec![0_u8; size as usize];
        // SAFETY: buffer is allocated to the queried size.
        if unsafe { libc::fgetxattr(fd, name.as_ptr(), bytes.as_mut_ptr().cast(), bytes.len()) } < 0
        {
            return Err(io_error_value("read trusted ACL"));
        }
        if acl_grants_named_principal(&bytes) {
            return Err(TrustedFsError::new(
                "trusted entry ACL grants another principal",
            ));
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn validate_acl(fd: RawFd) -> Result<(), TrustedFsError> {
        validate_macos_acl(fd, false)
    }

    #[cfg(target_os = "macos")]
    fn validate_macos_acl(fd: RawFd, reject_all_entries: bool) -> Result<(), TrustedFsError> {
        type Acl = *mut libc::c_void;
        type AclEntry = *mut libc::c_void;
        const ACL_TYPE_EXTENDED: libc::c_int = 0x0000_0100;
        const ACL_FIRST_ENTRY: libc::c_int = 0;
        const ACL_NEXT_ENTRY: libc::c_int = -1;

        unsafe extern "C" {
            fn acl_get_fd_np(fd: libc::c_int, acl_type: libc::c_int) -> Acl;
            fn acl_get_entry(acl: Acl, entry_id: libc::c_int, entry: *mut AclEntry) -> libc::c_int;
            fn acl_get_tag_type(entry: AclEntry, tag: *mut libc::c_int) -> libc::c_int;
            fn acl_free(acl: Acl) -> libc::c_int;
        }

        clear_errno();
        // SAFETY: fd is live and ACL_TYPE_EXTENDED requests a detached ACL copy.
        let acl = unsafe { acl_get_fd_np(fd, ACL_TYPE_EXTENDED) };
        if acl.is_null() {
            let error = io::Error::last_os_error();
            return if error.kind() == io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(TrustedFsError::new(format!("inspect trusted ACL: {error}")))
            };
        }

        let validation = (|| {
            let mut entry_id = ACL_FIRST_ENTRY;
            loop {
                let mut entry = std::ptr::null_mut();
                clear_errno();
                // SAFETY: acl is live and entry points to initialized pointer storage.
                if unsafe { acl_get_entry(acl, entry_id, &mut entry) } != 0 {
                    let error = io::Error::last_os_error();
                    if error.raw_os_error() == Some(libc::EINVAL) {
                        return Ok(());
                    }
                    return Err(TrustedFsError::new(format!(
                        "enumerate trusted ACL: {error}"
                    )));
                }
                if reject_all_entries {
                    return Err(TrustedFsError::new("trusted entry has an extended ACL"));
                }
                let mut tag = 0;
                // SAFETY: acl_get_entry returned a live entry owned by acl.
                if unsafe { acl_get_tag_type(entry, &mut tag) } != 0 {
                    return Err(io_error_value("inspect trusted ACL entry"));
                }
                validate_macos_acl_tag(tag)?;
                entry_id = ACL_NEXT_ENTRY;
            }
        })();

        // SAFETY: acl_get_fd_np returned a uniquely allocated ACL object.
        if unsafe { acl_free(acl) } != 0 {
            return Err(io_error_value("free trusted ACL"));
        }
        validation
    }

    #[cfg(target_os = "macos")]
    fn validate_macos_acl_tag(tag: libc::c_int) -> Result<(), TrustedFsError> {
        match tag {
            MACOS_ACL_EXTENDED_ALLOW => Err(TrustedFsError::new(
                "trusted entry ACL grants another principal",
            )),
            MACOS_ACL_EXTENDED_DENY => Ok(()),
            _ => Err(TrustedFsError::new(
                "trusted entry ACL has an unsupported tag",
            )),
        }
    }

    fn acl_grants_named_principal(bytes: &[u8]) -> bool {
        if bytes.get(..4) != Some(2_u32.to_le_bytes().as_slice()) {
            return true;
        }
        bytes.get(4..).is_none_or(|entries| {
            entries.len() % 8 != 0
                || entries.chunks_exact(8).any(|entry| {
                    let tag = u16::from_le_bytes([entry[0], entry[1]]);
                    let permissions = u16::from_le_bytes([entry[2], entry[3]]);
                    let _ = permissions;
                    matches!(tag, ACL_USER | ACL_GROUP)
                })
        })
    }

    #[cfg(target_os = "linux")]
    enum LinuxAclXattr {
        Absent,
        Present(Vec<u8>),
    }

    #[cfg(target_os = "linux")]
    fn read_linux_acl_xattr(fd: RawFd, name: &CStr) -> Result<LinuxAclXattr, ()> {
        const MAX_ACL_XATTR_BYTES: usize = 64 * 1024;

        #[cfg(test)]
        {
            let kind = match name.to_bytes() {
                b"system.posix_acl_access" => Some(PrivateHomeAclKind::Access),
                b"system.posix_acl_default" => Some(PrivateHomeAclKind::Default),
                _ => None,
            };
            if let Some(result) = TEST_LINUX_ACL_READ.with(|state| match &*state.borrow() {
                Some(TestLinuxAclRead::Present(expected, bytes)) if Some(*expected) == kind => {
                    Some(Ok(LinuxAclXattr::Present(bytes.clone())))
                }
                Some(TestLinuxAclRead::Unavailable(expected)) if Some(*expected) == kind => {
                    Some(Err(()))
                }
                _ => None,
            }) {
                return result;
            }
        }

        // SAFETY: fd is live; a null buffer with zero length queries the xattr size.
        let size = unsafe { libc::fgetxattr(fd, name.as_ptr(), std::ptr::null_mut(), 0) };
        if size < 0 {
            if io::Error::last_os_error().raw_os_error() != Some(libc::ENODATA) {
                return Err(());
            }
            // Linux routes POSIX ACL names through do_get_acl/vfs_get_acl. LSM denial and
            // filesystem errors are returned before ENODATA, which vfs_get_acl emits only when
            // the descriptor-bound ACL lookup returns no ACL.
            return Ok(LinuxAclXattr::Absent);
        }
        let size = usize::try_from(size).map_err(|_| ())?;
        if size > MAX_ACL_XATTR_BYTES {
            return Err(());
        }
        let mut bytes = vec![0_u8; size];
        // SAFETY: bytes owns exactly the queried writable capacity.
        let read =
            unsafe { libc::fgetxattr(fd, name.as_ptr(), bytes.as_mut_ptr().cast(), bytes.len()) };
        if read < 0 || usize::try_from(read).map_err(|_| ())? != bytes.len() {
            return Err(());
        }
        Ok(LinuxAclXattr::Present(bytes))
    }

    #[cfg(target_os = "linux")]
    fn parse_parent_acl(bytes: &[u8]) -> Result<bool, ()> {
        #[derive(Clone, Copy, Eq, PartialEq)]
        enum AclEntryState {
            UserObject,
            Users,
            Groups,
            Other,
            Complete,
        }

        let Some(entries) = bytes.get(4..) else {
            return Err(());
        };
        if bytes.get(..4) != Some(2_u32.to_le_bytes().as_slice()) || entries.len() % 8 != 0 {
            return Err(());
        }
        let mut mask = None;
        let mut named_entries = Vec::new();
        let mut state = AclEntryState::UserObject;
        for entry in entries.chunks_exact(8) {
            let tag = u16::from_le_bytes([entry[0], entry[1]]);
            let permissions = u16::from_le_bytes([entry[2], entry[3]]);
            let identifier = u32::from_le_bytes([entry[4], entry[5], entry[6], entry[7]]);
            if permissions & !0o7 != 0 {
                return Err(());
            }
            match tag {
                0x0001 if state == AclEntryState::UserObject => {
                    if identifier != u32::MAX {
                        return Err(());
                    }
                    state = AclEntryState::Users;
                }
                ACL_USER if state == AclEntryState::Users => {
                    if identifier == u32::MAX
                        || named_entries.iter().any(|(seen_tag, seen_id, _)| {
                            *seen_tag == tag && *seen_id == identifier
                        })
                    {
                        return Err(());
                    }
                    named_entries.push((tag, identifier, permissions));
                }
                0x0004 if state == AclEntryState::Users => {
                    if identifier != u32::MAX {
                        return Err(());
                    }
                    state = AclEntryState::Groups;
                }
                ACL_GROUP if state == AclEntryState::Groups => {
                    if identifier == u32::MAX
                        || named_entries.iter().any(|(seen_tag, seen_id, _)| {
                            *seen_tag == tag && *seen_id == identifier
                        })
                    {
                        return Err(());
                    }
                    named_entries.push((tag, identifier, permissions));
                }
                ACL_MASK if state == AclEntryState::Groups => {
                    if identifier != u32::MAX || mask.replace(permissions).is_some() {
                        return Err(());
                    }
                    state = AclEntryState::Other;
                }
                0x0020
                    if state == AclEntryState::Other
                        || (state == AclEntryState::Groups && named_entries.is_empty()) =>
                {
                    if identifier != u32::MAX {
                        return Err(());
                    }
                    state = AclEntryState::Complete;
                }
                _ => return Err(()),
            }
        }
        if state != AclEntryState::Complete {
            return Err(());
        }
        if named_entries.is_empty() {
            return Ok(false);
        }
        let Some(mask) = mask else {
            return Err(());
        };
        Ok(named_entries
            .into_iter()
            .any(|(_, _, permissions)| permissions & mask & 0o2 != 0))
    }

    #[cfg(target_os = "linux")]
    fn parent_acl_grants_named_principal(bytes: &[u8]) -> bool {
        parse_parent_acl(bytes).unwrap_or(true)
    }

    fn kind_from_mode(mode: libc::mode_t) -> EntryKind {
        match mode & libc::S_IFMT {
            libc::S_IFDIR => EntryKind::Directory,
            libc::S_IFREG => EntryKind::RegularFile,
            libc::S_IFLNK => EntryKind::Symlink,
            _ => EntryKind::Other,
        }
    }

    fn effective_uid() -> libc::uid_t {
        // SAFETY: geteuid has no preconditions.
        unsafe { libc::geteuid() }
    }

    fn io_error(operation: &'static str) -> impl FnOnce(io::Error) -> TrustedFsError {
        move |error| TrustedFsError::from_io(operation, error)
    }

    fn io_error_value(operation: &str) -> TrustedFsError {
        TrustedFsError::from_io(operation, io::Error::last_os_error())
    }

    #[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
    mod tests {
        use std::os::unix::fs::{symlink, MetadataExt, PermissionsExt};
        use std::sync::{Arc, Barrier};

        use super::*;

        #[test]
        fn trusted_fs_errors_distinguish_create_races_from_other_failures() {
            assert!(TrustedFsError::from_io(
                "create",
                io::Error::from(io::ErrorKind::AlreadyExists),
            )
            .is_already_exists());
            assert!(!TrustedFsError::from_io(
                "create",
                io::Error::from(io::ErrorKind::PermissionDenied),
            )
            .is_already_exists());
        }

        fn safe_test_parent() -> std::path::PathBuf {
            if let Some(explicit) = std::env::var_os("SUBSTRATE_A1_TEST_PARENT") {
                return explicit.into();
            }
            #[cfg(target_os = "macos")]
            {
                return std::path::PathBuf::from(
                    std::env::var_os("HOME").expect("macOS tests require HOME"),
                )
                .join("Library/Caches");
            }
            #[cfg(target_os = "linux")]
            std::env::var_os("XDG_RUNTIME_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| {
                    std::path::PathBuf::from(format!("/run/user/{}", effective_uid()))
                })
        }

        fn root() -> (tempfile::TempDir, TrustedAuthorityRoot) {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-trusted-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let root = TrustedAuthorityRoot::open(temp.path()).unwrap();
            (temp, root)
        }

        #[cfg(target_os = "linux")]
        fn linux_acl(entries: &[(u16, u16, u32)]) -> Vec<u8> {
            let mut acl = 2_u32.to_le_bytes().to_vec();
            for (tag, permissions, identifier) in entries {
                acl.extend_from_slice(&tag.to_le_bytes());
                acl.extend_from_slice(&permissions.to_le_bytes());
                acl.extend_from_slice(&identifier.to_le_bytes());
            }
            acl
        }

        #[cfg(target_os = "linux")]
        fn linux_named_user_acl(permissions: u16, mask: u16) -> Vec<u8> {
            linux_acl(&[
                (0x0001, 0o7, u32::MAX),
                (ACL_USER, permissions, effective_uid().saturating_add(1)),
                (0x0004, 0, u32::MAX),
                (ACL_MASK, mask, u32::MAX),
                (0x0020, 0, u32::MAX),
            ])
        }

        #[cfg(target_os = "linux")]
        fn set_linux_acl(directory: &File, name: &CStr, acl: &[u8]) {
            // SAFETY: directory is live and acl points to an initialized buffer.
            assert_eq!(
                unsafe {
                    libc::fsetxattr(
                        directory.as_raw_fd(),
                        name.as_ptr(),
                        acl.as_ptr().cast(),
                        acl.len(),
                        0,
                    )
                },
                0,
                "{}",
                io::Error::last_os_error()
            );
        }

        #[cfg(target_os = "linux")]
        fn with_test_linux_acl_read<T>(
            override_value: TestLinuxAclRead,
            run: impl FnOnce() -> T,
        ) -> T {
            struct Reset;
            impl Drop for Reset {
                fn drop(&mut self) {
                    TEST_LINUX_ACL_READ.with(|state| *state.borrow_mut() = None);
                }
            }

            TEST_LINUX_ACL_READ.with(|state| *state.borrow_mut() = Some(override_value));
            let _reset = Reset;
            run()
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn ancestor_access_acl_matrix_rejects_only_effective_named_write_or_invalid_state() {
            for (permissions, mask) in [(0o1, 0o1), (0o5, 0o5), (0o7, 0o5)] {
                let acl = linux_named_user_acl(permissions, mask);
                assert!(
                    !parent_acl_grants_named_principal(&acl),
                    "raw permissions {permissions:o} under mask {mask:o} confer no write"
                );
            }

            let multiple_non_writers = linux_acl(&[
                (0x0001, 0o7, u32::MAX),
                (ACL_USER, 0o1, effective_uid().saturating_add(1)),
                (0x0004, 0, u32::MAX),
                (ACL_GROUP, 0o5, effective_uid().saturating_add(1)),
                (ACL_MASK, 0o5, u32::MAX),
                (0x0020, 0, u32::MAX),
            ]);
            assert!(!parent_acl_grants_named_principal(&multiple_non_writers));

            for (permissions, mask) in [(0o2, 0o2), (0o3, 0o3), (0o6, 0o6), (0o7, 0o7)] {
                let acl = linux_named_user_acl(permissions, mask);
                assert!(
                    parent_acl_grants_named_principal(&acl),
                    "raw permissions {permissions:o} under mask {mask:o} confer write"
                );
            }

            let duplicate_named = linux_acl(&[
                (0x0001, 0o7, u32::MAX),
                (ACL_USER, 0o1, 1),
                (ACL_USER, 0o1, 1),
                (0x0004, 0, u32::MAX),
                (ACL_MASK, 0o1, u32::MAX),
                (0x0020, 0, u32::MAX),
            ]);
            let missing_base = linux_acl(&[
                (0x0001, 0o7, u32::MAX),
                (ACL_USER, 0o1, 1),
                (ACL_MASK, 0o1, u32::MAX),
                (0x0020, 0, u32::MAX),
            ]);
            let missing_mask = linux_acl(&[
                (0x0001, 0o7, u32::MAX),
                (ACL_USER, 0o1, 1),
                (0x0004, 0, u32::MAX),
                (0x0020, 0, u32::MAX),
            ]);
            let duplicate_mask = linux_acl(&[
                (0x0001, 0o7, u32::MAX),
                (ACL_USER, 0o1, 1),
                (0x0004, 0, u32::MAX),
                (ACL_MASK, 0o1, u32::MAX),
                (ACL_MASK, 0o1, u32::MAX),
                (0x0020, 0, u32::MAX),
            ]);
            let out_of_order_named_group = linux_acl(&[
                (0x0001, 0o7, u32::MAX),
                (ACL_USER, 0o1, 1),
                (ACL_GROUP, 0o1, 2),
                (0x0004, 0, u32::MAX),
                (ACL_MASK, 0o1, u32::MAX),
                (0x0020, 0, u32::MAX),
            ]);
            let unsupported_tag = linux_acl(&[
                (0x0001, 0o7, u32::MAX),
                (0x0040, 0, u32::MAX),
                (0x0004, 0, u32::MAX),
                (0x0020, 0, u32::MAX),
            ]);
            let mut malformed_version = linux_named_user_acl(0o1, 0o1);
            malformed_version[..4].copy_from_slice(&3_u32.to_le_bytes());
            let mut invalid_length = linux_named_user_acl(0o1, 0o1);
            invalid_length.push(0);

            for malformed in [
                duplicate_named,
                missing_base,
                missing_mask,
                duplicate_mask,
                out_of_order_named_group,
                unsupported_tag,
                malformed_version,
                invalid_length,
            ] {
                assert!(parent_acl_grants_named_principal(&malformed));
            }
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn every_present_ancestor_default_acl_is_rejected_before_candidate_creation() {
            for (label, permissions, mask) in [
                ("granting", 0o7, 0o7),
                ("zero-effective", 0, 0),
                ("masked", 0o7, 0o1),
            ] {
                let temp = tempfile::Builder::new()
                    .prefix(&format!("substrate-a1-default-acl-{label}-"))
                    .tempdir_in(safe_test_parent())
                    .unwrap();
                fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
                let directory = File::open(temp.path()).unwrap();
                let acl = linux_named_user_acl(permissions, mask);
                set_linux_acl(&directory, c"system.posix_acl_default", &acl);
                let target = temp.path().join("home");

                let error = ensure_private_substrate_home(&target, effective_uid()).unwrap_err();
                assert_eq!(error.reason(), PrivateHomeReason::ForeignAcl, "{label}");
                assert!(!target.exists(), "{label} default ACL created a candidate");
            }
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn malformed_and_unavailable_ancestor_acl_state_fails_closed_before_creation() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-invalid-acl-state-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();

            let mut malformed_default = linux_named_user_acl(0, 0);
            malformed_default[..4].copy_from_slice(&3_u32.to_le_bytes());
            let malformed_access = malformed_default.clone();
            for (label, override_value, reason, kind, authority) in [
                (
                    "malformed-access",
                    TestLinuxAclRead::Present(PrivateHomeAclKind::Access, malformed_access),
                    PrivateHomeReason::ForeignAcl,
                    PrivateHomeAclKind::Access,
                    PrivateHomeAclAuthority::Malformed,
                ),
                (
                    "malformed-default",
                    TestLinuxAclRead::Present(PrivateHomeAclKind::Default, malformed_default),
                    PrivateHomeReason::ForeignAcl,
                    PrivateHomeAclKind::Default,
                    PrivateHomeAclAuthority::Malformed,
                ),
                (
                    "unavailable-default",
                    TestLinuxAclRead::Unavailable(PrivateHomeAclKind::Default),
                    PrivateHomeReason::ValidationUnavailable,
                    PrivateHomeAclKind::Unavailable,
                    PrivateHomeAclAuthority::Unavailable,
                ),
                (
                    "unavailable-access",
                    TestLinuxAclRead::Unavailable(PrivateHomeAclKind::Access),
                    PrivateHomeReason::ValidationUnavailable,
                    PrivateHomeAclKind::Unavailable,
                    PrivateHomeAclAuthority::Unavailable,
                ),
            ] {
                let target = temp.path().join(label);
                let error = with_test_linux_acl_read(override_value, || {
                    ensure_private_substrate_home(&target, effective_uid()).unwrap_err()
                });
                assert_eq!(error.reason(), reason, "{label}");
                assert_eq!(error.acl_kind(), Some(kind), "{label}");
                assert_eq!(error.acl_authority(), Some(authority), "{label}");
                assert_eq!(error.object_role(), Some(PrivateHomeObjectRole::Ancestor));
                assert!(!error.candidate_created(), "{label}");
                assert!(!target.exists(), "{label} created a candidate");
            }
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn ancestor_acl_failure_records_exact_non_sensitive_provenance() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-acl-provenance-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let directory = File::open(temp.path()).unwrap();
            set_linux_acl(
                &directory,
                c"system.posix_acl_access",
                &linux_named_user_acl(0o2, 0o2),
            );
            let target = temp.path().join("home");

            let error = ensure_private_substrate_home(&target, effective_uid()).unwrap_err();
            assert_eq!(error.requested_home(), Some(target.as_path()));
            assert_eq!(error.offending_path(), Some(temp.path()));
            assert_eq!(error.object_role(), Some(PrivateHomeObjectRole::Ancestor));
            assert_eq!(error.acl_kind(), Some(PrivateHomeAclKind::Access));
            assert_eq!(
                error.acl_authority(),
                Some(PrivateHomeAclAuthority::EffectiveWrite)
            );
            assert!(!error.candidate_created());
            assert!(!target.exists());
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn rejected_current_attempt_candidate_records_final_root_provenance_without_cleanup() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-created-acl-provenance-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let target = temp.path().join("home");
            let error = ensure_private_substrate_home_with(
                &target,
                effective_uid(),
                || {
                    let candidate = File::open(&target).unwrap();
                    set_linux_acl(
                        &candidate,
                        c"system.posix_acl_default",
                        &linux_named_user_acl(0, 0),
                    );
                },
                || {},
                || {},
            )
            .unwrap_err();

            assert_eq!(error.requested_home(), Some(target.as_path()));
            assert_eq!(error.offending_path(), Some(target.as_path()));
            assert_eq!(error.object_role(), Some(PrivateHomeObjectRole::FinalRoot));
            assert_eq!(error.acl_kind(), Some(PrivateHomeAclKind::Default));
            assert_eq!(
                error.acl_authority(),
                Some(PrivateHomeAclAuthority::DefaultAclPresent)
            );
            assert!(error.candidate_created());
            assert!(
                target.is_dir(),
                "R1 must not clean up the rejected candidate"
            );
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn existing_final_root_matrix_preserves_acl_mode_and_identity_on_rejection() {
            for (label, xattr_name, expected_kind, expected_authority) in [
                (
                    "access",
                    c"system.posix_acl_access",
                    PrivateHomeAclKind::Access,
                    PrivateHomeAclAuthority::ExtendedAccessAcl,
                ),
                (
                    "default",
                    c"system.posix_acl_default",
                    PrivateHomeAclKind::Default,
                    PrivateHomeAclAuthority::DefaultAclPresent,
                ),
            ] {
                let temp = tempfile::Builder::new()
                    .prefix(&format!("substrate-a1-final-{label}-acl-"))
                    .tempdir_in(safe_test_parent())
                    .unwrap();
                fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
                let target = temp.path().join("home");
                fs::create_dir(&target).unwrap();
                fs::set_permissions(&target, fs::Permissions::from_mode(0o700)).unwrap();
                let directory = File::open(&target).unwrap();
                let acl = linux_named_user_acl(0o7, 0);
                set_linux_acl(&directory, xattr_name, &acl);
                let before = fs::symlink_metadata(&target).unwrap();

                let error = ensure_private_substrate_home(&target, effective_uid()).unwrap_err();
                let after = fs::symlink_metadata(&target).unwrap();
                assert_eq!(error.requested_home(), Some(target.as_path()));
                assert_eq!(error.offending_path(), Some(target.as_path()));
                assert_eq!(error.object_role(), Some(PrivateHomeObjectRole::FinalRoot));
                assert_eq!(error.acl_kind(), Some(expected_kind));
                assert_eq!(error.acl_authority(), Some(expected_authority));
                assert!(!error.candidate_created());
                assert_eq!(before.ino(), after.ino());
                assert_eq!(before.mode(), after.mode());
                assert_eq!(before.uid(), after.uid());
                let LinuxAclXattr::Present(after_acl) =
                    read_linux_acl_xattr(directory.as_raw_fd(), xattr_name).unwrap()
                else {
                    panic!("{label} ACL disappeared after rejection");
                };
                assert_eq!(after_acl, acl);
            }
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn valid_created_and_existing_final_root_is_exact_owner_0700_and_acl_free() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-valid-final-root-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let target = temp.path().join("home");

            let created = ensure_private_substrate_home(&target, effective_uid()).unwrap();
            let first = fs::symlink_metadata(&target).unwrap();
            assert_eq!(first.uid(), effective_uid());
            assert_eq!(first.mode() & 0o7777, 0o700);
            let directory = File::open(&target).unwrap();
            assert!(matches!(
                read_linux_acl_xattr(directory.as_raw_fd(), c"system.posix_acl_access").unwrap(),
                LinuxAclXattr::Absent
            ));
            assert!(matches!(
                read_linux_acl_xattr(directory.as_raw_fd(), c"system.posix_acl_default").unwrap(),
                LinuxAclXattr::Absent
            ));
            created.revalidate().unwrap();

            let existing = ensure_private_substrate_home(&target, effective_uid()).unwrap();
            let second = fs::symlink_metadata(&target).unwrap();
            assert_eq!(first.ino(), second.ino());
            existing.revalidate().unwrap();
        }

        #[test]
        fn final_root_type_mode_and_special_bit_rejections_are_non_mutating() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-final-state-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();

            for mode in [0o755, 0o750, 0o1700, 0o2700, 0o4700] {
                let target = temp.path().join(format!("mode-{mode:o}"));
                fs::create_dir(&target).unwrap();
                fs::set_permissions(&target, fs::Permissions::from_mode(mode)).unwrap();
                let before = fs::symlink_metadata(&target).unwrap();
                let error = ensure_private_substrate_home(&target, effective_uid()).unwrap_err();
                let after = fs::symlink_metadata(&target).unwrap();
                assert_eq!(error.reason(), PrivateHomeReason::WrongMode);
                assert_eq!(error.object_role(), Some(PrivateHomeObjectRole::FinalRoot));
                assert!(!error.candidate_created());
                assert_eq!(before.ino(), after.ino());
                assert_eq!(before.mode(), after.mode());
            }

            let file = temp.path().join("file");
            fs::write(&file, b"unchanged").unwrap();
            let file_before = fs::read(&file).unwrap();
            let error = ensure_private_substrate_home(&file, effective_uid()).unwrap_err();
            assert_eq!(error.reason(), PrivateHomeReason::WrongType);
            assert_eq!(error.object_role(), Some(PrivateHomeObjectRole::FinalRoot));
            assert_eq!(fs::read(&file).unwrap(), file_before);

            let destination = temp.path().join("destination");
            fs::create_dir(&destination).unwrap();
            fs::set_permissions(&destination, fs::Permissions::from_mode(0o700)).unwrap();
            let link = temp.path().join("link");
            symlink(&destination, &link).unwrap();
            let error = ensure_private_substrate_home(&link, effective_uid()).unwrap_err();
            assert_eq!(error.reason(), PrivateHomeReason::Symlink);
            assert_eq!(error.object_role(), Some(PrivateHomeObjectRole::FinalRoot));
            assert!(fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink());
        }

        #[test]
        fn trusted_root_rejects_relative_dot_file_and_unsafe_mode_inputs() {
            assert!(TrustedAuthorityRoot::open(Path::new("relative")).is_err());
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-trusted-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            assert!(TrustedAuthorityRoot::open(&temp.path().join(".")).is_err());
            let file = temp.path().join("file");
            fs::write(&file, b"x").unwrap();
            assert!(TrustedAuthorityRoot::open(&file).is_err());
            for mode in [0o755, 0o750, 0o1700, 0o777] {
                fs::set_permissions(temp.path(), fs::Permissions::from_mode(mode)).unwrap();
                assert!(
                    TrustedAuthorityRoot::open(temp.path()).is_err(),
                    "mode {mode:o} must not satisfy the exact private-root contract"
                );
            }
        }

        #[test]
        fn trusted_root_rejects_an_owner_other_than_the_intended_user() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-trusted-owner-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let foreign_uid = effective_uid().saturating_add(1);
            let error = TrustedAuthorityRoot::open_for_owner(temp.path(), foreign_uid).unwrap_err();
            assert!(error.to_string().contains("wrong-owner"));
        }

        #[test]
        fn first_open_accepts_valid_prebind_same_uid_substitution_as_candidate_identity() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-create-replacement-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let target = temp.path().join("home");
            let retained = temp.path().join("retained");
            let outcome = ensure_private_substrate_home_with(
                &target,
                effective_uid(),
                || {
                    fs::rename(&target, &retained).unwrap();
                    fs::create_dir(&target).unwrap();
                    fs::set_permissions(&target, fs::Permissions::from_mode(0o700)).unwrap();
                },
                || {},
                || {},
            );
            let root = outcome.expect("the first opened valid candidate defines accepted identity");
            let retained_inode = fs::symlink_metadata(&retained).unwrap().ino();
            let accepted_inode = fs::symlink_metadata(&target).unwrap().ino();
            assert_ne!(retained_inode, accepted_inode);
            match root.identity().physical_identity {
                DirectoryPhysicalIdentityV1::Linux { inode, .. } => {
                    assert_eq!(inode, accepted_inode)
                }
                DirectoryPhysicalIdentityV1::MacOs { file_id, .. } => {
                    assert_eq!(file_id, accepted_inode)
                }
            }
            root.revalidate().unwrap();
        }

        #[test]
        fn parent_name_replacement_after_first_open_fails_closed() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-parent-replacement-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let parent = temp.path().join("parent");
            fs::create_dir(&parent).unwrap();
            fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
            let target = parent.join("home");
            let retained_parent = temp.path().join("retained-parent");
            let outcome = ensure_private_substrate_home_with(
                &target,
                effective_uid(),
                || {},
                || {
                    fs::rename(&parent, &retained_parent).unwrap();
                    fs::create_dir(&parent).unwrap();
                    fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
                    fs::create_dir(parent.join("home")).unwrap();
                    fs::set_permissions(parent.join("home"), fs::Permissions::from_mode(0o700))
                        .unwrap();
                },
                || {},
            );

            assert_eq!(outcome.unwrap_err().reason(), PrivateHomeReason::Replaced);
            assert!(retained_parent.join("home").is_dir());
            assert!(parent.join("home").is_dir());
            assert!(!parent.join("home/deps").exists());
        }

        #[test]
        fn accepted_root_rejects_name_replacement_at_later_boundary() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-post-open-replacement-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let target = temp.path().join("home");
            let root = ensure_private_substrate_home(&target, effective_uid()).unwrap();
            let retained = temp.path().join("retained");
            fs::rename(&target, &retained).unwrap();
            fs::create_dir(&target).unwrap();
            fs::set_permissions(&target, fs::Permissions::from_mode(0o700)).unwrap();

            assert!(root.revalidate().is_err());
            assert!(retained.is_dir());
            assert!(target.is_dir());
        }

        #[test]
        fn private_home_rejects_parent_writable_by_another_principal() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-unsafe-parent-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o770)).unwrap();
            let target = temp.path().join("home");

            let error = ensure_private_substrate_home(&target, effective_uid()).unwrap_err();
            assert_eq!(error.reason(), PrivateHomeReason::WrongMode);
            assert!(!target.exists());
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn private_home_accepts_non_grant_parent_acl() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-parent-acl-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let directory = File::open(temp.path()).unwrap();
            let mut acl = 2_u32.to_le_bytes().to_vec();
            for (tag, permissions, id) in [
                (0x0001_u16, 7_u16, u32::MAX),
                (ACL_USER, 0_u16, effective_uid().saturating_add(1)),
                (0x0004_u16, 0_u16, u32::MAX),
                (0x0010_u16, 0_u16, u32::MAX),
                (0x0020_u16, 0_u16, u32::MAX),
            ] {
                acl.extend_from_slice(&tag.to_le_bytes());
                acl.extend_from_slice(&permissions.to_le_bytes());
                acl.extend_from_slice(&id.to_le_bytes());
            }
            // SAFETY: directory is live and acl points to an initialized buffer.
            assert_eq!(
                unsafe {
                    libc::fsetxattr(
                        directory.as_raw_fd(),
                        c"system.posix_acl_access".as_ptr(),
                        acl.as_ptr().cast(),
                        acl.len(),
                        0,
                    )
                },
                0,
                "{}",
                io::Error::last_os_error()
            );
            assert!(!parent_acl_grants_named_principal(&acl));
            validate_private_home_parent_acl(directory.as_raw_fd()).unwrap();
            let target = temp.path().join("home");

            let root = ensure_private_substrate_home(&target, effective_uid()).unwrap();
            root.revalidate().unwrap();
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn private_home_rejects_foreign_parent_acl_grant() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-parent-acl-grant-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let directory = File::open(temp.path()).unwrap();
            let mut acl = 2_u32.to_le_bytes().to_vec();
            for (tag, permissions, id) in [
                (0x0001_u16, 7_u16, u32::MAX),
                (ACL_USER, 2_u16, effective_uid().saturating_add(1)),
                (0x0004_u16, 0_u16, u32::MAX),
                (0x0010_u16, 2_u16, u32::MAX),
                (0x0020_u16, 0_u16, u32::MAX),
            ] {
                acl.extend_from_slice(&tag.to_le_bytes());
                acl.extend_from_slice(&permissions.to_le_bytes());
                acl.extend_from_slice(&id.to_le_bytes());
            }
            // SAFETY: directory is live and acl points to an initialized buffer.
            assert_eq!(
                unsafe {
                    libc::fsetxattr(
                        directory.as_raw_fd(),
                        c"system.posix_acl_access".as_ptr(),
                        acl.as_ptr().cast(),
                        acl.len(),
                        0,
                    )
                },
                0,
                "{}",
                io::Error::last_os_error()
            );
            let target = temp.path().join("home");

            let error = ensure_private_substrate_home(&target, effective_uid()).unwrap_err();
            assert_eq!(error.reason(), PrivateHomeReason::ForeignAcl);
            assert!(!target.exists());
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn private_home_accepts_execute_only_named_parent_acl() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-parent-acl-search-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let directory = File::open(temp.path()).unwrap();
            let acl = linux_named_user_acl(0o1, 0o1);
            // SAFETY: directory is live and acl points to an initialized buffer.
            assert_eq!(
                unsafe {
                    libc::fsetxattr(
                        directory.as_raw_fd(),
                        c"system.posix_acl_access".as_ptr(),
                        acl.as_ptr().cast(),
                        acl.len(),
                        0,
                    )
                },
                0,
                "{}",
                io::Error::last_os_error()
            );
            let target = temp.path().join("home");

            let root = ensure_private_substrate_home(&target, effective_uid()).unwrap();
            root.revalidate().unwrap();
        }

        #[cfg(target_os = "macos")]
        #[test]
        fn private_home_accepts_deny_only_parent_acl() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-parent-deny-acl-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let status = std::process::Command::new("/bin/chmod")
                .args(["+a", "everyone deny delete"])
                .arg(temp.path())
                .status()
                .unwrap();
            assert!(status.success());
            let target = temp.path().join("home");

            let result = ensure_private_substrate_home(&target, effective_uid());
            let cleanup = std::process::Command::new("/bin/chmod")
                .arg("-N")
                .arg(temp.path())
                .status()
                .unwrap();
            assert!(cleanup.success());
            result.unwrap().revalidate().unwrap();
        }

        #[test]
        fn signaled_creation_child_preserves_created_candidate_provenance() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-signaled-create-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let parent = File::open(temp.path()).unwrap();
            let name = component("candidate").unwrap();
            let target = temp.path().join("candidate");

            let error = mkdirat_exact_mode_with_signaled_child_for_test(parent.as_raw_fd(), &name)
                .unwrap_err()
                .at_path(&target, PrivateHomeObjectRole::FinalRoot)
                .for_attempt(&target, false);

            assert_eq!(error.reason(), PrivateHomeReason::ValidationUnavailable);
            assert_eq!(
                error.candidate_provenance(),
                PrivateHomeCandidateProvenance::Created
            );
            assert!(error.candidate_created());
            assert!(target.is_dir());
        }

        #[test]
        fn legitimate_concurrent_private_home_creators_converge() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-concurrent-home-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let target = Arc::new(temp.path().join("home"));
            let barrier = Arc::new(Barrier::new(8));
            let mut creators = Vec::new();
            for _ in 0..8 {
                let target = Arc::clone(&target);
                let barrier = Arc::clone(&barrier);
                creators.push(std::thread::spawn(move || {
                    barrier.wait();
                    let root = ensure_private_substrate_home(&target, effective_uid()).unwrap();
                    root.identity().physical_identity.clone()
                }));
            }
            let identities = creators
                .into_iter()
                .map(|creator| creator.join().unwrap())
                .collect::<Vec<_>>();

            assert!(identities.windows(2).all(|pair| pair[0] == pair[1]));
            let metadata = fs::symlink_metadata(target.as_path()).unwrap();
            assert_eq!(metadata.mode() & 0o7777, DIRECTORY_MODE);
            assert_eq!(metadata.uid(), effective_uid());
        }

        #[test]
        fn failed_open_metadata_classifies_owner_and_mode_exactly() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-reason-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            let file = File::open(temp.path()).unwrap();
            let mut stat = fstat(file.as_raw_fd()).unwrap();
            stat.st_uid = effective_uid().saturating_add(1);
            assert_eq!(
                private_home_reason_from_stat(&stat, effective_uid()),
                PrivateHomeReason::WrongOwner
            );
            stat.st_uid = effective_uid();
            stat.st_mode &= libc::S_IFMT;
            assert_eq!(
                private_home_reason_from_stat(&stat, effective_uid()),
                PrivateHomeReason::WrongMode
            );
        }

        #[test]
        fn scaffold_writes_remain_bound_to_the_accepted_root_handle() {
            let (temp, root) = root();
            let original = temp.path().to_path_buf();
            let moved = original.with_extension("scaffold-moved");
            let _ = fs::remove_dir_all(&moved);
            fs::rename(&original, &moved).unwrap();
            fs::create_dir(&original).unwrap();
            fs::set_permissions(&original, fs::Permissions::from_mode(0o700)).unwrap();

            let deps = root.ensure_scaffold_directory("deps").unwrap();
            deps.ensure_file_if_missing("README.md", b"bound\n")
                .unwrap();
            assert_eq!(fs::read(moved.join("deps/README.md")).unwrap(), b"bound\n");
            assert!(!original.join("deps").exists());
            assert!(root.revalidate().is_err());

            fs::remove_dir(&original).unwrap();
            fs::rename(&moved, &original).unwrap();
            root.revalidate().unwrap();
        }

        #[test]
        fn descendant_access_is_nofollow_and_single_component_only() {
            let (temp, root) = root();
            let authority = root.directory().create_directory("authority-v1").unwrap();
            symlink(temp.path(), temp.path().join("authority-v1/link")).unwrap();
            assert_eq!(
                authority.entry_kind("link").unwrap(),
                Some(EntryKind::Symlink)
            );
            assert!(authority.open_directory("link").is_err());
            assert!(authority.open_file("link").is_err());
            assert!(authority.open_directory("../authority-v1").is_err());
            assert!(authority.create_file("nested/file").is_err());
        }

        #[test]
        fn opened_handle_survives_path_replacement_without_following_replacement() {
            let (temp, root) = root();
            let authority = root.directory().create_directory("authority-v1").unwrap();
            fs::rename(temp.path().join("authority-v1"), temp.path().join("moved")).unwrap();
            fs::create_dir(temp.path().join("authority-v1")).unwrap();
            fs::set_permissions(
                temp.path().join("authority-v1"),
                fs::Permissions::from_mode(0o700),
            )
            .unwrap();
            let mut file = authority.create_file("bound").unwrap();
            file.write_all(b"opened-handle").unwrap();
            file.sync().unwrap();
            assert!(temp.path().join("moved/bound").is_file());
            assert!(!temp.path().join("authority-v1/bound").exists());
        }

        #[test]
        fn exclusive_directory_creation_rejects_name_replacement_before_identity_binding() {
            let (temp, root) = root();
            let original = temp.path().join("child");
            let retained = temp.path().join("child-retained");
            let outcome = root
                .directory()
                .create_directory_entry_exclusive_with("child", || {
                    fs::rename(&original, &retained).unwrap();
                    fs::create_dir(&original).unwrap();
                    fs::set_permissions(&original, fs::Permissions::from_mode(0o700)).unwrap();
                });

            assert!(outcome.is_err());
            assert!(fs::read_dir(original).unwrap().next().is_none());
            assert!(fs::read_dir(retained).unwrap().next().is_none());
        }

        #[test]
        fn no_replace_replace_enumerate_and_unlink_are_directory_relative() {
            let (_temp, root) = root();
            let authority = root.directory().create_directory("authority-v1").unwrap();
            let temp = authority.create_directory("tmp").unwrap();
            let objects = authority.create_directory("objects").unwrap();
            let mut source = temp.create_file("object.tmp").unwrap();
            source.write_all(b"one").unwrap();
            source.sync().unwrap();
            temp.rename_no_replace("object.tmp", source, &objects, "object.obj")
                .unwrap();

            let mut replacement = temp.create_file("root.tmp").unwrap();
            replacement.write_all(b"two").unwrap();
            replacement.sync().unwrap();
            temp.rename_replace("root.tmp", replacement, &objects, "object.obj")
                .unwrap();
            let bytes = objects.open_file("object.obj").unwrap().read_all().unwrap();
            assert_eq!(bytes, b"two");
            let first_scan = objects.entries().unwrap();
            let second_scan = objects.entries().unwrap();
            assert_eq!(first_scan, second_scan);
            assert_eq!(first_scan[0].name, "object.obj");
            objects.unlink_file("object.obj").unwrap();
            assert!(objects.entries().unwrap().is_empty());
        }

        #[test]
        fn no_replace_refuses_existing_target_without_partial_publication() {
            let (_temp, root) = root();
            let authority = root.directory().create_directory("authority-v1").unwrap();
            let temp = authority.create_directory("tmp").unwrap();
            let objects = authority.create_directory("objects").unwrap();
            let mut published = temp.create_file("first.tmp").unwrap();
            published.write_all(b"published").unwrap();
            published.sync().unwrap();
            temp.rename_no_replace("first.tmp", published, &objects, "object.obj")
                .unwrap();

            let mut conflicting = temp.create_file("second.tmp").unwrap();
            conflicting.write_all(b"conflicting").unwrap();
            conflicting.sync().unwrap();
            assert!(temp
                .rename_no_replace("second.tmp", conflicting, &objects, "object.obj")
                .is_err());
            assert_eq!(
                objects.open_file("object.obj").unwrap().read_all().unwrap(),
                b"published"
            );
            assert_eq!(
                temp.open_file("second.tmp").unwrap().read_all().unwrap(),
                b"conflicting"
            );
        }

        #[test]
        fn child_file_and_directory_permissions_are_enforced() {
            let (temp, root) = root();
            let authority = root.directory().create_directory("authority-v1").unwrap();
            let child = authority.create_directory("child").unwrap();
            let mut file = authority.create_file("record").unwrap();
            file.write_all(b"record").unwrap();
            file.sync().unwrap();

            fs::set_permissions(
                temp.path().join("authority-v1/child"),
                fs::Permissions::from_mode(0o755),
            )
            .unwrap();
            assert!(authority.open_directory("child").is_err());
            fs::set_permissions(
                temp.path().join("authority-v1/child"),
                fs::Permissions::from_mode(0o700),
            )
            .unwrap();
            child.sync().unwrap();

            fs::set_permissions(
                temp.path().join("authority-v1/record"),
                fs::Permissions::from_mode(0o640),
            )
            .unwrap();
            assert!(authority.open_file("record").is_err());
        }

        #[test]
        fn child_file_and_directory_special_permission_bits_are_rejected() {
            let (temp, root) = root();
            let authority = root.directory().create_directory("authority-v1").unwrap();
            authority.create_directory("child").unwrap();
            authority.create_file("record").unwrap();

            let child_path = temp.path().join("authority-v1/child");
            for mode in [0o4700, 0o2700, 0o1700] {
                fs::set_permissions(&child_path, fs::Permissions::from_mode(mode)).unwrap();
                assert_eq!(
                    fs::metadata(&child_path).unwrap().permissions().mode() & 0o7777,
                    mode
                );
                assert!(authority.open_directory("child").is_err());
                fs::set_permissions(&child_path, fs::Permissions::from_mode(0o700)).unwrap();
            }

            let record_path = temp.path().join("authority-v1/record");
            for mode in [0o4600, 0o2600, 0o1600] {
                fs::set_permissions(&record_path, fs::Permissions::from_mode(mode)).unwrap();
                assert_eq!(
                    fs::metadata(&record_path).unwrap().permissions().mode() & 0o7777,
                    mode
                );
                assert!(authority.open_file("record").is_err());
                fs::set_permissions(&record_path, fs::Permissions::from_mode(0o600)).unwrap();
            }
        }

        #[test]
        fn scanned_entry_replacement_fails_revalidation() {
            let (_temp, root) = root();
            let authority = root.directory().create_directory("authority-v1").unwrap();
            let mut file = authority.create_file("root.json").unwrap();
            file.write_all(b"one").unwrap();
            file.sync().unwrap();
            let scanned = authority.entries().unwrap().remove(0);
            authority.unlink_file("root.json").unwrap();
            let mut replacement = authority.create_file("root.json").unwrap();
            replacement.write_all(b"two").unwrap();
            replacement.sync().unwrap();
            assert!(authority.revalidate_entry(&scanned).is_err());
        }

        #[test]
        fn publication_rejects_source_name_substitution() {
            let (_temp, root) = root();
            let authority = root.directory().create_directory("authority-v1").unwrap();
            let temp = authority.create_directory("tmp").unwrap();
            let objects = authority.create_directory("objects").unwrap();
            let mut expected = temp.create_file("object.tmp").unwrap();
            expected.write_all(b"expected").unwrap();
            expected.sync().unwrap();
            temp.unlink_file("object.tmp").unwrap();
            let mut substitute = temp.create_file("object.tmp").unwrap();
            substitute.write_all(b"substitute").unwrap();
            substitute.sync().unwrap();
            assert!(temp
                .rename_no_replace("object.tmp", expected, &objects, "object.obj")
                .is_err());
            assert_eq!(objects.entry_kind("object.obj").unwrap(), None);
        }

        #[test]
        fn root_symlink_is_rejected_without_following() {
            let (target, _direct) = root();
            let parent = safe_test_parent();
            let link = parent.join(format!("substrate-a1-link-{}", std::process::id()));
            let _ = fs::remove_file(&link);
            symlink(target.path(), &link).unwrap();
            assert!(TrustedAuthorityRoot::open(&link).is_err());
            fs::remove_file(link).unwrap();
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn masked_named_acl_entry_is_rejected_even_without_effective_permissions() {
            let mut acl = 2_u32.to_le_bytes().to_vec();
            for (tag, permissions, id) in [
                (0x0001_u16, 7_u16, u32::MAX),
                (ACL_USER, 0_u16, effective_uid().saturating_add(1)),
                (0x0004_u16, 0_u16, u32::MAX),
                (0x0010_u16, 0_u16, u32::MAX),
                (0x0020_u16, 0_u16, u32::MAX),
            ] {
                acl.extend_from_slice(&tag.to_le_bytes());
                acl.extend_from_slice(&permissions.to_le_bytes());
                acl.extend_from_slice(&id.to_le_bytes());
            }
            assert!(acl_grants_named_principal(&acl));
        }

        #[test]
        fn trusted_root_rename_fails_byte_exact_path_revalidation() {
            let (temp, root) = root();
            let original = temp.path().to_path_buf();
            let moved = original.with_extension("moved");
            let _ = fs::remove_dir_all(&moved);
            fs::rename(&original, &moved).unwrap();
            assert!(root.revalidate().is_err());
            fs::rename(&moved, &original).unwrap();
            root.revalidate().unwrap();
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn actual_named_acl_grant_is_rejected() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-acl-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let directory = File::open(temp.path()).unwrap();
            let mut acl = 2_u32.to_le_bytes().to_vec();
            for (tag, permissions, id) in [
                (0x0001_u16, 7_u16, u32::MAX),
                (ACL_USER, 1_u16, effective_uid().saturating_add(1)),
                (0x0004_u16, 0_u16, u32::MAX),
                (0x0010_u16, 1_u16, u32::MAX),
                (0x0020_u16, 0_u16, u32::MAX),
            ] {
                acl.extend_from_slice(&tag.to_le_bytes());
                acl.extend_from_slice(&permissions.to_le_bytes());
                acl.extend_from_slice(&id.to_le_bytes());
            }
            // SAFETY: directory is live and acl points to an initialized buffer.
            let result = unsafe {
                libc::fsetxattr(
                    directory.as_raw_fd(),
                    c"system.posix_acl_access".as_ptr(),
                    acl.as_ptr().cast(),
                    acl.len(),
                    0,
                )
            };
            assert_eq!(result, 0, "{}", io::Error::last_os_error());
            assert!(TrustedAuthorityRoot::open(temp.path()).is_err());
        }

        #[cfg(target_os = "macos")]
        #[test]
        fn actual_extended_acl_is_rejected() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-acl-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let status = std::process::Command::new("/bin/chmod")
                .args(["+a", "everyone allow write"])
                .arg(temp.path())
                .status()
                .unwrap();
            assert!(status.success());
            assert!(TrustedAuthorityRoot::open(temp.path()).is_err());
        }

        #[cfg(target_os = "macos")]
        #[test]
        fn protective_extended_acl_deny_is_rejected() {
            let temp = tempfile::Builder::new()
                .prefix("substrate-a1-acl-")
                .tempdir_in(safe_test_parent())
                .unwrap();
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let status = std::process::Command::new("/bin/chmod")
                .args(["+a", "everyone deny delete"])
                .arg(temp.path())
                .status()
                .unwrap();
            assert!(status.success());
            assert!(TrustedAuthorityRoot::open(temp.path()).is_err());
        }

        #[cfg(target_os = "macos")]
        #[test]
        fn macos_acl_tag_validation_fails_closed_except_for_deny() {
            assert!(validate_macos_acl_tag(MACOS_ACL_EXTENDED_ALLOW).is_err());
            validate_macos_acl_tag(MACOS_ACL_EXTENDED_DENY).unwrap();
            assert!(validate_macos_acl_tag(libc::c_int::MAX).is_err());
        }

        #[test]
        fn cross_process_flock_excludes_and_crash_releases() {
            const CHILD_TEST: &str = "execution::agent_runtime::host_session_authority::trusted_fs::platform::tests::cross_process_flock_excludes_and_crash_releases";
            if let Some(root_path) = std::env::var_os("SUBSTRATE_A1_LOCK_CHILD_ROOT") {
                let root = TrustedAuthorityRoot::open(Path::new(&root_path)).unwrap();
                let authority = root.directory().open_directory("authority-v1").unwrap();
                let lock_dir = authority.open_directory("lock").unwrap();
                let lock_file = lock_dir.open_file("root.lock").unwrap();
                match std::env::var("SUBSTRATE_A1_LOCK_CHILD_MODE")
                    .unwrap()
                    .as_str()
                {
                    "probe" => assert!(lock_file.try_lock_exclusive().unwrap().is_none()),
                    "lock-exit" => {
                        let _lock = lock_file.lock_exclusive().unwrap();
                        std::process::exit(0);
                    }
                    mode => panic!("unknown child mode {mode}"),
                }
                return;
            }

            let (temp, root) = root();
            let authority = root.directory().create_directory("authority-v1").unwrap();
            let lock_dir = authority.create_directory("lock").unwrap();
            let lock_file = lock_dir.create_file("root.lock").unwrap();
            lock_file.sync().unwrap();
            let lock = lock_file.lock_exclusive().unwrap();
            let status = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", CHILD_TEST, "--nocapture"])
                .env("SUBSTRATE_A1_LOCK_CHILD_ROOT", temp.path())
                .env("SUBSTRATE_A1_LOCK_CHILD_MODE", "probe")
                .status()
                .unwrap();
            assert!(status.success());
            lock.release().unwrap();

            let status = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", CHILD_TEST, "--nocapture"])
                .env("SUBSTRATE_A1_LOCK_CHILD_ROOT", temp.path())
                .env("SUBSTRATE_A1_LOCK_CHILD_MODE", "lock-exit")
                .status()
                .unwrap();
            assert!(status.success());
            assert!(lock_file.try_lock_exclusive().unwrap().is_some());
        }

        #[test]
        fn interrupted_primitive_temp_is_non_authoritative_and_cleanup_is_durable() {
            const CHILD_TEST: &str = "execution::agent_runtime::host_session_authority::trusted_fs::platform::tests::interrupted_primitive_temp_is_non_authoritative_and_cleanup_is_durable";
            const TEMP_NAME: &str =
                "object--ao_11111111111111111111111111111111--22222222222222222222222222222222.tmp";
            if let Some(root_path) = std::env::var_os("SUBSTRATE_A1_TEMP_CHILD_ROOT") {
                let root = TrustedAuthorityRoot::open(Path::new(&root_path)).unwrap();
                let authority = root.directory().open_directory("authority-v1").unwrap();
                let temp = authority.open_directory("tmp").unwrap();
                let mut partial = temp.create_file(TEMP_NAME).unwrap();
                partial.write_all(b"partial").unwrap();
                std::process::exit(73);
            }

            let (root_temp, root) = root();
            let authority = root.directory().create_directory("authority-v1").unwrap();
            let temp = authority.create_directory("tmp").unwrap();
            let objects = authority.create_directory("objects").unwrap();
            let status = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", CHILD_TEST, "--nocapture"])
                .env("SUBSTRATE_A1_TEMP_CHILD_ROOT", root_temp.path())
                .status()
                .unwrap();
            assert_eq!(status.code(), Some(73));
            assert_eq!(objects.entry_kind("partial.obj").unwrap(), None);
            assert_eq!(temp.entries().unwrap().len(), 1);
            temp.unlink_file(TEMP_NAME).unwrap();
            temp.sync().unwrap();
            assert!(temp.entries().unwrap().is_empty());
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn acl_parser_rejects_named_principal_grants_and_malformed_state() {
            let mut acl = 2_u32.to_le_bytes().to_vec();
            acl.extend_from_slice(&ACL_USER.to_le_bytes());
            acl.extend_from_slice(&4_u16.to_le_bytes());
            acl.extend_from_slice(&1001_u32.to_le_bytes());
            assert!(acl_grants_named_principal(&acl));
            assert!(acl_grants_named_principal(&[1, 2, 3]));

            let mut harmless = 2_u32.to_le_bytes().to_vec();
            harmless.extend_from_slice(&1_u16.to_le_bytes());
            harmless.extend_from_slice(&7_u16.to_le_bytes());
            harmless.extend_from_slice(&u32::MAX.to_le_bytes());
            assert!(!acl_grants_named_principal(&harmless));
        }

        #[cfg(target_os = "macos")]
        #[test]
        fn macos_case_aliases_follow_reported_volume_semantics() {
            let (temp, direct) = root();
            let name = temp.path().file_name().unwrap().to_str().unwrap();
            let alias_name = name
                .chars()
                .map(|character| {
                    if character.is_ascii_lowercase() {
                        character.to_ascii_uppercase()
                    } else {
                        character.to_ascii_lowercase()
                    }
                })
                .collect::<String>();
            assert_ne!(name, alias_name);
            let alias_path = temp.path().with_file_name(alias_name);
            let DirectoryPhysicalIdentityV1::MacOs { case_sensitive, .. } =
                &direct.identity().physical_identity
            else {
                panic!("macOS root must carry macOS physical identity");
            };
            if *case_sensitive {
                assert!(TrustedAuthorityRoot::open(&alias_path).is_err());
            } else {
                let through_alias = TrustedAuthorityRoot::open(&alias_path).unwrap();
                assert_eq!(through_alias.identity(), direct.identity());
            }
        }

        #[cfg(target_os = "macos")]
        #[test]
        fn macos_case_folded_publication_collisions_follow_volume_semantics() {
            let (_temp, root) = root();
            let DirectoryPhysicalIdentityV1::MacOs { case_sensitive, .. } =
                &root.identity().physical_identity
            else {
                panic!("macOS root must carry macOS physical identity");
            };
            let authority = root.directory().create_directory("authority-v1").unwrap();
            let mut first = authority.create_file("Object.obj").unwrap();
            first.write_all(b"first").unwrap();
            first.sync().unwrap();
            let second = authority.create_file("object.obj");
            if *case_sensitive {
                let mut second = second.unwrap();
                second.write_all(b"second").unwrap();
                second.sync().unwrap();
                assert_eq!(authority.entries().unwrap().len(), 2);
            } else {
                assert!(second.is_err());
                assert_eq!(authority.entries().unwrap().len(), 1);
                assert_eq!(
                    authority
                        .open_file("object.obj")
                        .unwrap()
                        .read_all()
                        .unwrap(),
                    b"first"
                );
            }
        }

        #[cfg(target_os = "macos")]
        #[test]
        fn macos_case_folded_rename_collisions_follow_volume_semantics() {
            let (_temp, root) = root();
            let DirectoryPhysicalIdentityV1::MacOs { case_sensitive, .. } =
                &root.identity().physical_identity
            else {
                panic!("macOS root must carry macOS physical identity");
            };
            let authority = root.directory().create_directory("authority-v1").unwrap();
            let temp = authority.create_directory("tmp").unwrap();
            let objects = authority.create_directory("objects").unwrap();
            let mut first = temp.create_file("first.tmp").unwrap();
            first.write_all(b"first").unwrap();
            first.sync().unwrap();
            temp.rename_no_replace("first.tmp", first, &objects, "Object.obj")
                .unwrap();

            let mut second = temp.create_file("second.tmp").unwrap();
            second.write_all(b"second").unwrap();
            second.sync().unwrap();
            let publication = temp.rename_no_replace("second.tmp", second, &objects, "object.obj");
            if *case_sensitive {
                publication.unwrap();
                assert_eq!(objects.entries().unwrap().len(), 2);
            } else {
                assert!(publication.is_err());
                assert_eq!(objects.entries().unwrap().len(), 1);
                assert_eq!(
                    objects.open_file("object.obj").unwrap().read_all().unwrap(),
                    b"first"
                );
                assert_eq!(
                    temp.open_file("second.tmp").unwrap().read_all().unwrap(),
                    b"second"
                );
            }
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[allow(unused_imports)]
pub(crate) use platform::*;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[derive(Debug)]
pub(crate) struct TrustedAuthorityRoot;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[derive(Debug)]
pub(crate) struct TrustedWorkspaceRoot;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
impl TrustedAuthorityRoot {
    pub(crate) fn open(_path: &std::path::Path) -> Result<Self, TrustedFsError> {
        Err(TrustedFsError::new(
            "A1 trusted authority storage is unsupported on this platform",
        ))
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
impl TrustedWorkspaceRoot {
    pub(crate) fn open_exact(_expected: &CanonicalDirectoryV1) -> Result<Self, TrustedFsError> {
        Err(TrustedFsError::new(
            "A1 trusted workspace binding is unsupported on this platform",
        ))
    }

    pub(crate) fn revalidate(&self) -> Result<(), TrustedFsError> {
        Err(TrustedFsError::new(
            "A1 trusted workspace binding is unsupported on this platform",
        ))
    }
}
