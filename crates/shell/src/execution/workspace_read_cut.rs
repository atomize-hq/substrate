//! Owner revision publication and bounded Linux observations. No read enrolls a source.
use anyhow::{anyhow, bail, ensure, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ffi::CString;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Component, Path};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceIdentityV1 {
    device: u64,
    inode: u64,
    mode: u32,
    uid: u32,
    length: u64,
    modified: (i64, i64),
    changed: (i64, i64),
    digest: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SettingsRevisionV1 {
    schema_version: u32,
    incarnation: String,
    pub(crate) revision: u64,
    owner: u32,
    directory: (u64, u64),
    source: String,
    identity: Option<SourceIdentityV1>,
}

/// A trust rejection before any protected publication is attempted.
#[derive(Debug)]
struct UnsupportedSettingsCustodyV1;
impl std::fmt::Display for UnsupportedSettingsCustodyV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Unsupported settings custody")
    }
}
impl std::error::Error for UnsupportedSettingsCustodyV1 {}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum LegacySettingsWriteV1 {
    Revisioned,
    UnsupportedCustody,
}

pub(crate) fn directory(path: &Path, uid: u32) -> Result<File> {
    if !path.is_absolute()
        || !path
            .components()
            .all(|c| matches!(c, Component::RootDir | Component::Normal(_)))
    {
        return Err(UnsupportedSettingsCustodyV1.into());
    }
    let mut held = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open("/")?;
    for part in path.components() {
        if let Component::Normal(name) = part {
            held = child(&held, name.as_bytes(), libc::O_RDONLY | libc::O_DIRECTORY).map_err(
                |error| {
                    if error.downcast_ref::<std::io::Error>().is_some_and(|e| {
                        matches!(e.raw_os_error(), Some(libc::ELOOP | libc::ENOTDIR))
                    }) {
                        anyhow::Error::from(UnsupportedSettingsCustodyV1)
                    } else {
                        error
                    }
                },
            )?;
            let m = held.metadata()?;
            if m.mode() & 0o7022 != 0 || (m.uid() != uid && m.uid() != 0) {
                return Err(UnsupportedSettingsCustodyV1.into());
            }
        }
    }
    let m = held.metadata()?;
    if m.uid() != uid || m.mode() & 0o700 != 0o700 {
        return Err(UnsupportedSettingsCustodyV1.into());
    }
    no_acl(&held)?;
    ensure!(
        std::fs::read_link(format!("/proc/self/fd/{}", held.as_raw_fd()))? == path,
        "Changed"
    );
    Ok(held)
}

fn no_acl(file: &File) -> Result<()> {
    for name in [
        b"system.posix_acl_access\0".as_slice(),
        b"system.posix_acl_default\0".as_slice(),
    ] {
        let result = unsafe {
            libc::fgetxattr(
                file.as_raw_fd(),
                name.as_ptr().cast(),
                std::ptr::null_mut(),
                0,
            )
        };
        if result >= 0 {
            return Err(UnsupportedSettingsCustodyV1.into());
        }
        let error = std::io::Error::last_os_error();
        ensure!(
            matches!(error.raw_os_error(), Some(libc::ENODATA | libc::ENOTSUP)),
            "Unavailable ACL"
        );
    }
    Ok(())
}

pub(crate) fn child(parent: &File, name: &[u8], flags: i32) -> Result<File> {
    ensure!(!name.contains(&b'/'), "Unsafe component");
    let name = CString::new(name)?;
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            flags | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            0o600,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(unsafe { File::from_raw_fd(fd) })
}

fn read(parent: &File, name: &str, uid: u32) -> Result<Option<(File, Vec<u8>)>> {
    let mut file = match child(parent, name.as_bytes(), libc::O_RDONLY) {
        Ok(file) => file,
        Err(error)
            if error
                .downcast_ref::<std::io::Error>()
                .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound) =>
        {
            return Ok(None)
        }
        Err(error) => return Err(error),
    };
    let m = file.metadata()?;
    ensure!(
        m.is_file()
            && m.uid() == uid
            && m.nlink() == 1
            && m.mode() & 0o7022 == 0
            && m.len() <= 65536,
        "Unsafe source"
    );
    no_acl(&file)?;
    let mut bytes = Vec::new();
    (&mut file).take(65537).read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= 65536, "Oversize");
    Ok(Some((file, bytes)))
}

fn identity(file: &File, bytes: &[u8]) -> Result<SourceIdentityV1> {
    let m = file.metadata()?;
    Ok(SourceIdentityV1 {
        device: m.dev(),
        inode: m.ino(),
        mode: m.mode(),
        uid: m.uid(),
        length: m.len(),
        modified: (m.mtime(), m.mtime_nsec()),
        changed: (m.ctime(), m.ctime_nsec()),
        digest: format!("{:x}", Sha256::digest(bytes)),
    })
}

pub(crate) fn source_identity(
    parent: &File,
    source: &str,
    uid: u32,
) -> Result<Option<SourceIdentityV1>> {
    read(parent, source, uid)?
        .map(|(file, bytes)| identity(&file, &bytes))
        .transpose()
}

fn names(source: &str) -> Result<(String, String)> {
    ensure!(
        matches!(
            source,
            "config.yaml" | "policy.yaml" | "workspace.yaml" | "workspace.disabled"
        ),
        "Unsupported source"
    );
    Ok((
        format!(".{source}.owner-revision.v1.json"),
        format!(".{source}.owner-revision.v1.pending"),
    ))
}

pub(crate) fn read_revision(parent: &File, source: &str, uid: u32) -> Result<SettingsRevisionV1> {
    let (record, pending) = names(source)?;
    ensure!(
        read(parent, &pending, uid)?.is_none(),
        "Incomplete revision"
    );
    let (_, bytes) =
        read(parent, &record, uid)?.ok_or_else(|| anyhow!("Unsupported revision coverage"))?;
    let value: SettingsRevisionV1 = serde_json::from_slice(&bytes)?;
    let m = parent.metadata()?;
    ensure!(
        value.schema_version == 1
            && value.revision > 0
            && value.incarnation.len() == 36
            && value.owner == uid
            && value.directory == (m.dev(), m.ino())
            && value.source == source,
        "Integrity revision"
    );
    ensure!(
        value.identity == source_identity(parent, source, uid)?,
        "Changed unmediated source"
    );
    Ok(value)
}

pub(crate) fn optional_revision(
    parent: &File,
    source: &str,
    uid: u32,
) -> Result<Option<SettingsRevisionV1>> {
    // An absent source still requires its owner's explicit monotonic tombstone.
    // No read creates coverage or treats a missing ledger as a stable absence.
    read_revision(parent, source, uid).map(Some)
}

impl SettingsRevisionV1 {
    pub(crate) fn token(&self) -> transport_api_types::WorkspaceSettingsRevisionV1 {
        transport_api_types::WorkspaceSettingsRevisionV1 {
            incarnation: self.incarnation.clone(),
            revision: self.revision,
        }
    }
}

pub(crate) fn write_new(parent: &File, name: &str, bytes: &[u8]) -> Result<()> {
    let mut file = child(
        parent,
        name.as_bytes(),
        libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
    )?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

pub(crate) fn rename(parent: &File, from: &str, to: &str, no_replace: bool) -> Result<()> {
    let from = CString::new(from)?;
    let to = CString::new(to)?;
    let result = unsafe {
        libc::syscall(
            libc::SYS_renameat2,
            parent.as_raw_fd(),
            from.as_ptr(),
            parent.as_raw_fd(),
            to.as_ptr(),
            if no_replace {
                libc::RENAME_NOREPLACE
            } else {
                0
            },
        )
    };
    if result != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    parent.sync_all()?;
    Ok(())
}

pub(crate) fn unlink(parent: &File, name: &str) -> Result<()> {
    let name = CString::new(name)?;
    if unsafe { libc::unlinkat(parent.as_raw_fd(), name.as_ptr(), 0) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    parent.sync_all()?;
    Ok(())
}

/// Explicit source-owner write. Reads cannot call this to enroll or repair a source.
/// Legacy commands may retain their ordinary writer only when custody was
/// rejected before publication. Transaction and I/O errors are never fallback.
pub(crate) fn try_write_legacy_settings(
    path: &Path,
    bytes: Option<&[u8]>,
) -> Result<LegacySettingsWriteV1> {
    let parent_path = path.parent().ok_or_else(|| anyhow!("parent absent"))?;
    let source = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Unsafe source"))?;
    let (_, pending) = names(source)?;
    // A legacy operation must not replay a failed protected publication, even
    // when that directory has since become unsupported or moved below /tmp.
    for marker in [
        pending,
        format!(".{source}.owner-write.v1.pending"),
        format!(".{source}.owner-revision.v1.written"),
        ".workspace-init.v1.pending".into(),
        ".workspace-state.v1.pending".into(),
    ] {
        match std::fs::symlink_metadata(parent_path.join(marker)) {
            Ok(_) => bail!("Incomplete protected publication"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    let parent = match directory(parent_path, unsafe { libc::geteuid() }) {
        Ok(parent) => parent,
        Err(error) if error.is::<UnsupportedSettingsCustodyV1>() => {
            return Ok(LegacySettingsWriteV1::UnsupportedCustody)
        }
        Err(error) => return Err(error),
    };
    write_settings_at(&parent, parent_path, source, bytes)?;
    Ok(LegacySettingsWriteV1::Revisioned)
}

#[cfg(test)]
pub(crate) fn write_settings(path: &Path, bytes: Option<&[u8]>) -> Result<()> {
    let uid = unsafe { libc::geteuid() };
    let parent_path = path.parent().ok_or_else(|| anyhow!("parent absent"))?;
    let parent = directory(parent_path, uid)?;
    let source = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Unsafe"))?;
    write_settings_at(&parent, parent_path, source, bytes)
}

pub(crate) fn write_settings_at(
    parent: &File,
    expected_path: &Path,
    source: &str,
    bytes: Option<&[u8]>,
) -> Result<()> {
    write_settings_at_mode(parent, expected_path, source, bytes, false)
}

/// Initial publication preserves intervening sources and revision records.
pub(crate) fn write_initial_settings_at(
    parent: &File,
    expected_path: &Path,
    source: &str,
    bytes: Option<&[u8]>,
) -> Result<()> {
    write_settings_at_mode(parent, expected_path, source, bytes, true)
}

fn write_settings_at_mode(
    parent: &File,
    expected_path: &Path,
    source: &str,
    bytes: Option<&[u8]>,
    initial: bool,
) -> Result<()> {
    let uid = unsafe { libc::geteuid() };
    let held = child(parent, b".", libc::O_RDONLY | libc::O_DIRECTORY)?;
    ensure!(
        std::fs::read_link(format!("/proc/self/fd/{}", held.as_raw_fd()))? == expected_path,
        "Changed owner directory"
    );
    let result = write_settings_held(held, source, uid, bytes, initial);
    ensure!(
        std::fs::read_link(format!("/proc/self/fd/{}", parent.as_raw_fd()))? == expected_path,
        "Changed owner directory"
    );
    result
}

fn write_settings_held(
    parent: File,
    source: &str,
    uid: u32,
    bytes: Option<&[u8]>,
    initial: bool,
) -> Result<()> {
    ensure!(
        unsafe { libc::flock(parent.as_raw_fd(), libc::LOCK_EX) } == 0,
        "owner lock unavailable"
    );
    let (record, pending) = names(source)?;
    ensure!(
        read(&parent, &pending, uid)?.is_none(),
        "Incomplete revision"
    );
    if initial {
        ensure!(
            read(&parent, &record, uid)?.is_none()
                && source_identity(&parent, source, uid)?.is_none(),
            "Changed initial settings"
        );
    }
    // A genuine explicit mutation may begin coverage of a legacy source; existing
    // coverage can never silently reset or endorse an unmediated intervening change.
    let previous = match read(&parent, &record, uid)? {
        Some(_) => Some(read_revision(&parent, source, uid)?),
        None => None,
    };
    let meta = parent.metadata()?;
    let mut revision = SettingsRevisionV1 {
        schema_version: 1,
        incarnation: previous
            .as_ref()
            .map(|r| r.incarnation.clone())
            .unwrap_or_else(|| uuid::Uuid::now_v7().to_string()),
        revision: previous
            .as_ref()
            .map_or(Some(1), |r| r.revision.checked_add(1))
            .ok_or_else(|| anyhow!("revision exhausted"))?,
        owner: uid,
        directory: (meta.dev(), meta.ino()),
        source: source.to_owned(),
        identity: None,
    };
    write_new(&parent, &pending, b"incomplete owner settings write\n")?;
    if let Some(bytes) = bytes {
        let temporary = format!(".{source}.owner-write.v1.pending");
        write_new(&parent, &temporary, bytes)?;
        rename(&parent, &temporary, source, initial)?;
    } else if !initial && source_identity(&parent, source, uid)?.is_some() {
        unlink(&parent, source)?;
    }
    revision.identity = source_identity(&parent, source, uid)?;
    if initial {
        ensure!(
            match (bytes, revision.identity.as_ref()) {
                (Some(bytes), Some(identity)) =>
                    identity.length == bytes.len() as u64
                        && identity.digest == format!("{:x}", Sha256::digest(bytes)),
                (None, None) => true,
                _ => false,
            },
            "Changed initial settings"
        );
    }
    let temporary = format!(".{source}.owner-revision.v1.written");
    write_new(&parent, &temporary, &serde_json::to_vec(&revision)?)?;
    rename(&parent, &temporary, &record, initial)?;
    unlink(&parent, &pending)?;
    Ok(())
}

/// Kernel watch from before collection. Directory events are filtered by exact
/// consumed names; an unrelated sibling update does not invalidate the cut.
pub(crate) struct ReadWatchV1 {
    file: File,
    names: BTreeMap<i32, Vec<Vec<u8>>>,
}
impl ReadWatchV1 {
    pub(crate) fn new() -> Result<Self> {
        let fd = unsafe { libc::inotify_init1(libc::IN_CLOEXEC | libc::IN_NONBLOCK) };
        if fd < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(Self {
            file: unsafe { File::from_raw_fd(fd) },
            names: BTreeMap::new(),
        })
    }
    pub(crate) fn add(&mut self, path: &Path, names: &[&str]) -> Result<()> {
        self.add_directory(path, names).map(|_| ())
    }
    pub(crate) fn add_held(&mut self, directory: &File, names: &[&str]) -> Result<i32> {
        self.add_directory(
            &Path::new("/proc/self/fd")
                .join(directory.as_raw_fd().to_string())
                .join("."),
            names,
        )
    }
    fn add_directory(&mut self, path: &Path, names: &[&str]) -> Result<i32> {
        let path = CString::new(path.as_os_str().as_bytes())?;
        let mask = libc::IN_MODIFY
            | libc::IN_ATTRIB
            | libc::IN_CREATE
            | libc::IN_DELETE
            | libc::IN_MOVED_FROM
            | libc::IN_MOVED_TO
            | libc::IN_DELETE_SELF
            | libc::IN_MOVE_SELF
            | libc::IN_ONLYDIR
            | libc::IN_DONT_FOLLOW;
        let wd = unsafe { libc::inotify_add_watch(self.file.as_raw_fd(), path.as_ptr(), mask) };
        ensure!(wd >= 0, "Unsupported watch");
        self.names
            .entry(wd)
            .or_default()
            .extend(names.iter().map(|s| s.as_bytes().to_vec()));
        Ok(wd)
    }
    pub(crate) fn verify(&mut self) -> Result<()> {
        self.verify_events(None).map(|_| ())
    }
    pub(crate) fn verify_created_directory_once(&mut self, wd: i32, name: &[u8]) -> Result<()> {
        ensure!(
            self.verify_events(Some((wd, name)))?,
            "Missing expected creation"
        );
        Ok(())
    }
    fn verify_events(&mut self, expected_creation: Option<(i32, &[u8])>) -> Result<bool> {
        let mut created = false;
        let mut buffer = [0u8; 65536];
        for _ in 0..4 {
            let count = match self.file.read(&mut buffer) {
                Ok(0) => bail!("watch lost"),
                Ok(n) => n,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(created),
                Err(e) => return Err(e.into()),
            };
            let mut at = 0;
            while at + std::mem::size_of::<libc::inotify_event>() <= count {
                let event = unsafe {
                    std::ptr::read_unaligned(buffer[at..].as_ptr().cast::<libc::inotify_event>())
                };
                let start = at + std::mem::size_of::<libc::inotify_event>();
                let end = start + event.len as usize;
                ensure!(end <= count, "watch malformed");
                let name = buffer[start..end]
                    .split(|b| *b == 0)
                    .next()
                    .unwrap_or_default();
                ensure!(
                    event.mask
                        & (libc::IN_Q_OVERFLOW
                            | libc::IN_IGNORED
                            | libc::IN_DELETE_SELF
                            | libc::IN_MOVE_SELF
                            | libc::IN_UNMOUNT)
                        == 0,
                    "Changed watch"
                );
                ensure!(
                    !(name.is_empty() && event.mask & libc::IN_ATTRIB != 0),
                    "Changed held directory attributes"
                );
                if self
                    .names
                    .get(&event.wd)
                    .is_none_or(|names| names.iter().any(|n| n == name))
                {
                    ensure!(
                        !created
                            && expected_creation == Some((event.wd, name))
                            && event.mask == libc::IN_CREATE | libc::IN_ISDIR,
                        "Changed consumed source"
                    );
                    created = true;
                }
                at = end;
            }
        }
        bail!("Changed watch event budget exhausted")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn workspace_read_cut_mediated_same_bytes_aba_and_unmediated_restore() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.yaml");
        write_settings(&path, Some(b"{}\n")).unwrap();
        let dir = directory(temp.path(), unsafe { libc::geteuid() }).unwrap();
        let first = read_revision(&dir, "config.yaml", unsafe { libc::geteuid() }).unwrap();
        write_settings(&path, Some(b"world: {enabled: false}\n")).unwrap();
        write_settings(&path, Some(b"{}\n")).unwrap();
        let last = read_revision(&dir, "config.yaml", unsafe { libc::geteuid() }).unwrap();
        assert_eq!(last.revision, first.revision + 2);
        assert_ne!(first, last);
        std::fs::write(&path, b"changed").unwrap();
        std::fs::write(&path, b"{}\n").unwrap();
        assert!(read_revision(&dir, "config.yaml", unsafe { libc::geteuid() }).is_err());
    }
    #[test]
    fn workspace_read_cut_watch_detects_aba_and_ignores_sibling() {
        let temp = tempfile::tempdir().unwrap();
        let mut watch = ReadWatchV1::new().unwrap();
        watch.add(temp.path(), &["selected"]).unwrap();
        std::fs::write(temp.path().join("sibling"), "ok").unwrap();
        watch.verify().unwrap();
        std::fs::write(temp.path().join("selected"), "a").unwrap();
        std::fs::remove_file(temp.path().join("selected")).unwrap();
        assert!(watch.verify().is_err());
    }
    #[test]
    fn workspace_read_cut_absence_requires_genuine_owner_tombstone_and_aba_counter() {
        let temp = tempfile::tempdir().unwrap();
        let parent = directory(temp.path(), unsafe { libc::geteuid() }).unwrap();
        assert!(
            optional_revision(&parent, "policy.yaml", unsafe { libc::geteuid() }).is_err(),
            "missing source is not owner coverage"
        );
        let path = temp.path().join("policy.yaml");
        write_settings(&path, None).unwrap();
        let first = read_revision(&parent, "policy.yaml", unsafe { libc::geteuid() }).unwrap();
        assert!(!path.exists());
        write_settings(&path, Some(b"{}\n")).unwrap();
        write_settings(&path, None).unwrap();
        let last = read_revision(&parent, "policy.yaml", unsafe { libc::geteuid() }).unwrap();
        assert_eq!(last.revision, first.revision + 2);
        assert!(!path.exists());
        assert_ne!(first, last);
    }
    #[test]
    fn workspace_read_cut_directory_attribute_aba_and_watch_loss_refuse() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("held");
        std::fs::create_dir(&path).unwrap();
        let mut watch = ReadWatchV1::new().unwrap();
        watch.add(&path, &["source"]).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o7777;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode ^ 0o010)).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
        assert!(
            watch.verify().is_err(),
            "empty-name IN_ATTRIB is a consumed directory invalidator"
        );
        let mut watch = ReadWatchV1::new().unwrap();
        watch.add(&path, &["source"]).unwrap();
        std::fs::remove_dir(&path).unwrap();
        assert!(watch.verify().is_err(), "lost held directory refuses");
    }
}

#[cfg(test)]
mod legacy_custody_remediation_tests {
    use super::*;
    #[test]
    fn workspace_read_cut_unsupported_tmp_pending_veto_is_not_legacy_fallback() {
        let temp = tempfile::tempdir_in("/tmp").unwrap();
        let path = temp.path().join("workspace.yaml");
        assert_eq!(
            try_write_legacy_settings(&path, Some(b"{}\n")).unwrap(),
            LegacySettingsWriteV1::UnsupportedCustody
        );
        assert!(std::fs::read_dir(temp.path()).unwrap().next().is_none());
        std::fs::write(
            temp.path().join(".workspace-init.v1.pending"),
            b"incomplete",
        )
        .unwrap();
        assert!(try_write_legacy_settings(&path, Some(b"{}\n")).is_err());
        assert!(!path.exists());
    }
}
