use std::fmt;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::fs::File;

use super::schema::CanonicalDirectoryV1;

#[derive(Debug)]
pub(crate) struct TrustedFsError(String);

impl TrustedFsError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for TrustedFsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for TrustedFsError {}

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod platform {
    use std::ffi::{CStr, CString, OsStr};
    use std::fs;
    use std::io;
    use std::mem::MaybeUninit;
    use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::path::{Component, Path};

    use super::{CanonicalDirectoryV1, File, TrustedFsError};
    use crate::execution::agent_runtime::host_session_authority::schema::DirectoryPhysicalIdentityV1;

    const DIRECTORY_MODE: libc::mode_t = 0o700;
    const FILE_MODE: libc::mode_t = 0o600;
    const ACL_USER: u16 = 0x0002;
    const ACL_GROUP: u16 = 0x0008;

    #[derive(Debug)]
    pub(crate) struct TrustedAuthorityRoot {
        directory: TrustedDirectory,
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

    pub(crate) struct TrustedFileLock<'a> {
        file: Option<&'a TrustedFile>,
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
            validate_bootstrap_input(raw_path)?;
            let physical_path =
                fs::canonicalize(raw_path).map_err(io_error("resolve trusted root"))?;
            let metadata = fs::metadata(&physical_path).map_err(io_error("stat trusted root"))?;
            if !metadata.is_dir() {
                return Err(TrustedFsError::new("trusted root is not a directory"));
            }
            if metadata.uid() != effective_uid() || metadata.permissions().mode() & 0o022 != 0 {
                return Err(TrustedFsError::new("trusted root owner or mode is unsafe"));
            }

            let file = open_physical_directory(&physical_path)?;
            validate_acl(file.as_raw_fd())?;
            let stat = fstat(file.as_raw_fd())?;
            if stat.st_dev as u64 != metadata.dev() || stat.st_ino as u64 != metadata.ino() {
                return Err(TrustedFsError::new("trusted root changed while opening"));
            }
            let opened_physical_path = physical_path_from_handle(&file)?;
            let physical_utf8 = opened_physical_path
                .to_str()
                .filter(|path| path.starts_with('/') && !path.ends_with(" (deleted)"))
                .ok_or_else(|| {
                    TrustedFsError::new("opened trusted root has no stable UTF-8 physical path")
                })?;
            if opened_physical_path != physical_path {
                return Err(TrustedFsError::new(
                    "trusted root physical path changed while opening",
                ));
            }
            let identity = directory_identity(&file, physical_utf8, &stat)?;
            Ok(Self {
                directory: TrustedDirectory {
                    file,
                    device_id: stat.st_dev as u64,
                },
                identity,
            })
        }

        pub(crate) fn identity(&self) -> &CanonicalDirectoryV1 {
            &self.identity
        }

        pub(crate) fn revalidate(&self) -> Result<(), TrustedFsError> {
            let stat =
                validate_open_directory(&self.directory.file, self.directory.device_id, false)?;
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

        pub(crate) fn directory(&self) -> &TrustedDirectory {
            &self.directory
        }
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

        pub(crate) fn open_directory(&self, name: &str) -> Result<Self, TrustedFsError> {
            self.open_directory_cstr(&component(name)?)
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
                device_id: stat.st_dev,
                inode: stat.st_ino,
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
                device_id: stat.st_dev,
                inode: stat.st_ino,
            })
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

        pub(crate) fn revalidate_entry(
            &self,
            expected: &DirectoryEntry,
        ) -> Result<(), TrustedFsError> {
            let current = self
                .stat_entry(&expected.name)?
                .ok_or_else(|| TrustedFsError::new("scanned authority entry disappeared"))?;
            if kind_from_mode(current.st_mode) != expected.kind
                || current.st_dev != expected.device_id
                || current.st_ino != expected.inode
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
            if opened.st_dev != expected.device_id || opened.st_ino != expected.inode {
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
            let next = openat_file(
                current.as_raw_fd(),
                &c_string(component_value)?,
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
            unsafe { libc::openat(parent, name.as_ptr(), flags, mode) },
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
            || (exact_mode && stat.st_mode & 0o777 != DIRECTORY_MODE)
            || (!exact_mode && stat.st_mode & 0o022 != 0)
        {
            return Err(TrustedFsError::new(
                "trusted directory type/owner/mode/device mismatch",
            ));
        }
        validate_acl(file.as_raw_fd())?;
        Ok(stat)
    }

    fn validate_open_file(file: &File, expected_device: u64) -> Result<(), TrustedFsError> {
        let stat = fstat(file.as_raw_fd())?;
        if kind_from_mode(stat.st_mode) != EntryKind::RegularFile
            || stat.st_uid != effective_uid()
            || stat.st_dev as u64 != expected_device
            || stat.st_mode & 0o777 != FILE_MODE
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
        unsafe extern "C" {
            fn acl_extended_fd_np(fd: libc::c_int) -> libc::c_int;
        }
        // SAFETY: fd is live and acl_extended_fd_np only inspects its ACL.
        match unsafe { acl_extended_fd_np(fd) } {
            0 => Ok(()),
            1 => Err(TrustedFsError::new("trusted entry has an extended ACL")),
            _ => Err(io_error_value("inspect trusted ACL")),
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
                    matches!(tag, ACL_USER | ACL_GROUP) && permissions != 0
                })
        })
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
        move |error| TrustedFsError::new(format!("{operation}: {error}"))
    }

    fn io_error_value(operation: &str) -> TrustedFsError {
        TrustedFsError::new(format!("{operation}: {}", io::Error::last_os_error()))
    }

    #[cfg(all(test, target_os = "linux"))]
    mod tests {
        use std::os::unix::fs::{symlink, PermissionsExt};

        use super::*;

        fn safe_test_parent() -> std::path::PathBuf {
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
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o777)).unwrap();
            assert!(TrustedAuthorityRoot::open(temp.path()).is_err());
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
        fn root_symlink_resolves_to_opened_physical_identity() {
            let (target, direct) = root();
            let parent = safe_test_parent();
            let link = parent.join(format!("substrate-a1-link-{}", std::process::id()));
            let _ = fs::remove_file(&link);
            symlink(target.path(), &link).unwrap();
            let through_link = TrustedAuthorityRoot::open(&link).unwrap();
            assert_eq!(through_link.identity(), direct.identity());
            fs::remove_file(link).unwrap();
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
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[allow(unused_imports)]
pub(crate) use platform::*;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[derive(Debug)]
pub(crate) struct TrustedAuthorityRoot;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
impl TrustedAuthorityRoot {
    pub(crate) fn open(_path: &std::path::Path) -> Result<Self, TrustedFsError> {
        Err(TrustedFsError::new(
            "A1 trusted authority storage is unsupported on this platform",
        ))
    }
}
