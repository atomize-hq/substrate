//! Crash-recoverable, no-clobber publication for the experiment's fixed immutable leaves.

use anyhow::{bail, Context, Result};
use std::ffi::CString;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub(crate) enum PublishMode<'a> {
    Immutable,
    ReplaceExact { predecessor: &'a [u8] },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecoveryStateV2 {
    StartOrResumeTemp,
    Complete,
    RemoveLinkedTempThenComplete,
    ReplaceFromPredecessor,
    PreserveAndFail,
}

fn classify_recovery_state(
    final_matches_new: bool,
    final_matches_predecessor: bool,
    temp_present: bool,
    temp_is_exact_prefix: bool,
    final_and_temp_same_inode: bool,
    replace: bool,
) -> RecoveryStateV2 {
    if final_matches_new {
        return if !temp_present {
            RecoveryStateV2::Complete
        } else if final_and_temp_same_inode {
            RecoveryStateV2::RemoveLinkedTempThenComplete
        } else {
            RecoveryStateV2::PreserveAndFail
        };
    }
    if temp_present && !temp_is_exact_prefix {
        return RecoveryStateV2::PreserveAndFail;
    }
    if replace {
        if final_matches_predecessor {
            RecoveryStateV2::ReplaceFromPredecessor
        } else {
            RecoveryStateV2::PreserveAndFail
        }
    } else if !final_matches_predecessor {
        // For immutable publication `final_matches_predecessor` means the final leaf is absent.
        RecoveryStateV2::PreserveAndFail
    } else {
        RecoveryStateV2::StartOrResumeTemp
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PublishIdentityV2 {
    pub owner_uid: u32,
    pub owner_gid: u32,
    pub permissions: libc::mode_t,
    pub parent_uid: u32,
    pub parent_gid: u32,
    pub parent_mode: libc::mode_t,
    pub maximum_bytes: usize,
}

pub(crate) fn publish_exact_file(
    path: &Path,
    expected: &[u8],
    identity: PublishIdentityV2,
    mode: PublishMode<'_>,
) -> Result<()> {
    if expected.is_empty() || expected.len() > identity.maximum_bytes {
        bail!("fixed publication is empty or outside its compiled bound")
    }
    let parent = path.parent().context("fixed publication has no parent")?;
    require_parent(parent, identity)?;
    let leaf = path
        .file_name()
        .context("fixed publication has no leaf")?
        .to_string_lossy();
    if leaf.is_empty() || leaf.contains(['/', '\0', '\n', '\r']) {
        bail!("fixed publication leaf is invalid")
    }
    let temp = parent.join(format!(".{leaf}.next"));

    let final_state = read_leaf(path, identity, true)?;
    let temp_state = read_leaf(&temp, identity, true)?;
    let final_matches_new = final_state
        .as_ref()
        .is_some_and(|leaf| leaf.bytes == expected);
    let (replace, final_matches_predecessor) = match mode {
        PublishMode::Immutable => (false, final_state.is_none()),
        PublishMode::ReplaceExact { predecessor } => (
            true,
            final_state
                .as_ref()
                .is_some_and(|leaf| leaf.bytes == predecessor),
        ),
    };
    let temp_is_exact_prefix = temp_state
        .as_ref()
        .is_none_or(|leaf| leaf.bytes.len() <= expected.len() && expected.starts_with(&leaf.bytes));
    let same_inode = matches!((&final_state, &temp_state), (Some(final_leaf), Some(temp_leaf)) if final_leaf.device == temp_leaf.device && final_leaf.inode == temp_leaf.inode);
    match classify_recovery_state(
        final_matches_new,
        final_matches_predecessor,
        temp_state.is_some(),
        temp_is_exact_prefix,
        same_inode,
        replace,
    ) {
        RecoveryStateV2::Complete => {
            // This may be recovery from a crash after link/rename or staging unlink but before the
            // directory durability barrier. Re-sync the validated parent before claiming success.
            File::open(parent)?.sync_all()?;
            return reattest_final(path, expected, identity);
        }
        RecoveryStateV2::RemoveLinkedTempThenComplete => {
            let final_leaf = final_state.as_ref().expect("classified final exists");
            let temp_leaf = temp_state.as_ref().expect("classified temp exists");
            if final_leaf.link_count != 2 || temp_leaf.link_count != 2 {
                bail!("linked publication residue has an unexpected link count")
            }
            unlink_exact(&temp, temp_leaf)?;
            File::open(parent)?.sync_all()?;
            return reattest_final(path, expected, identity);
        }
        RecoveryStateV2::PreserveAndFail => {
            bail!("fixed publication found an alternate final or staging object")
        }
        RecoveryStateV2::StartOrResumeTemp | RecoveryStateV2::ReplaceFromPredecessor => {}
    }

    let mut temp_file = open_or_create_prefix_temp(&temp, expected, identity)?;
    let current_length =
        usize::try_from(temp_file.metadata()?.len()).context("staging length exceeds usize")?;
    temp_file.seek(SeekFrom::Start(u64::try_from(current_length)?))?;
    temp_file.write_all(&expected[current_length..])?;
    temp_file.sync_all()?;
    let staged = read_leaf(&temp, identity, false)?
        .context("completed staging leaf disappeared before publication")?;
    if staged.bytes != expected || staged.link_count != 1 {
        bail!("completed staging leaf changed before publication")
    }

    match mode {
        PublishMode::Immutable => {
            let temp_c = path_cstring(&temp)?;
            let final_c = path_cstring(path)?;
            // SAFETY: both exact leaves are in the validated fixed parent; link is no-clobber.
            if unsafe { libc::link(temp_c.as_ptr(), final_c.as_ptr()) } != 0 {
                let error = std::io::Error::last_os_error();
                // Do not delete the staging object on a conflict. Recovery may remove it only
                // after proving final+temp are the same expected inode.
                if error.raw_os_error() != Some(libc::EEXIST) {
                    return Err(error).context("link immutable publication into place");
                }
                return publish_exact_file(path, expected, identity, mode);
            }
            File::open(parent)?.sync_all()?;
            let linked =
                read_leaf(path, identity, true)?.context("linked immutable final disappeared")?;
            let staged = read_leaf(&temp, identity, true)?
                .context("linked immutable staging leaf disappeared")?;
            if linked.bytes != expected
                || staged.bytes != expected
                || linked.device != staged.device
                || linked.inode != staged.inode
                || linked.link_count != 2
                || staged.link_count != 2
            {
                bail!("immutable publication link identity changed")
            }
            unlink_exact(&temp, &staged)?;
        }
        PublishMode::ReplaceExact { predecessor } => {
            let current = read_leaf(path, identity, false)?
                .context("replace publication predecessor disappeared")?;
            if current.bytes != predecessor || current.link_count != 1 {
                bail!("replace publication predecessor changed before rename")
            }
            std::fs::rename(&temp, path).context("atomically replace exact predecessor")?;
        }
    }
    File::open(parent)?.sync_all()?;
    reattest_final(path, expected, identity)
}

/// Durably completes only the deterministic staging leaf for a later immutable publication.
/// The final name must remain absent. A caller may perform a separately journaled destructive
/// transition, then call `publish_exact_file` to atomically commit these exact staged bytes.
pub(crate) fn stage_exact_file(
    path: &Path,
    expected: &[u8],
    identity: PublishIdentityV2,
) -> Result<()> {
    if expected.is_empty() || expected.len() > identity.maximum_bytes {
        bail!("fixed staging publication is empty or outside its compiled bound")
    }
    let parent = path
        .parent()
        .context("fixed staging publication has no parent")?;
    require_parent(parent, identity)?;
    if read_leaf(path, identity, true)?.is_some() {
        bail!("fixed staging publication final name is already present")
    }
    let leaf = path
        .file_name()
        .context("fixed staging publication has no leaf")?
        .to_string_lossy();
    if leaf.is_empty() || leaf.contains(['/', '\0', '\n', '\r']) {
        bail!("fixed staging publication leaf is invalid")
    }
    let temp = parent.join(format!(".{leaf}.next"));
    let mut temp_file = open_or_create_prefix_temp(&temp, expected, identity)?;
    let current_length =
        usize::try_from(temp_file.metadata()?.len()).context("staging length exceeds usize")?;
    temp_file.seek(SeekFrom::Start(u64::try_from(current_length)?))?;
    temp_file.write_all(&expected[current_length..])?;
    temp_file.sync_all()?;
    let staged = read_leaf(&temp, identity, false)?
        .context("completed fixed staging publication disappeared")?;
    if staged.bytes != expected || staged.link_count != 1 {
        bail!("fixed staging publication changed after durable completion")
    }
    File::open(parent)?.sync_all()?;
    Ok(())
}

pub(crate) fn read_staged_file(
    path: &Path,
    identity: PublishIdentityV2,
) -> Result<Option<Vec<u8>>> {
    let parent = path.parent().context("fixed staged read has no parent")?;
    require_parent(parent, identity)?;
    if read_leaf(path, identity, true)?.is_some() {
        return Ok(None);
    }
    let leaf = path
        .file_name()
        .context("fixed staged read has no leaf")?
        .to_string_lossy();
    let temp = parent.join(format!(".{leaf}.next"));
    let Some(staged) = read_leaf(&temp, identity, false)? else {
        return Ok(None);
    };
    if staged.bytes.is_empty()
        || staged.bytes.len() > identity.maximum_bytes
        || staged.link_count != 1
    {
        bail!("fixed staged read changed its bounded identity")
    }
    Ok(Some(staged.bytes))
}

/// Stable-read an immutable published leaf, completing only the uniquely recognizable
/// post-link/pre-unlink crash state (final and deterministic staging names are the same inode with
/// nlink=2). No alternate staging inode is ever removed.
pub(crate) fn read_exact_published_file(
    path: &Path,
    identity: PublishIdentityV2,
) -> Result<Option<Vec<u8>>> {
    let parent = path.parent().context("fixed publication has no parent")?;
    require_parent(parent, identity)?;
    let Some(final_leaf) = read_leaf(path, identity, true)? else {
        return Ok(None);
    };
    if final_leaf.link_count == 2 {
        let leaf = path
            .file_name()
            .context("fixed publication has no leaf")?
            .to_string_lossy();
        let temp = parent.join(format!(".{leaf}.next"));
        let staged = read_leaf(&temp, identity, true)?
            .context("nlink=2 final publication lacks its deterministic staging name")?;
        if staged.device != final_leaf.device
            || staged.inode != final_leaf.inode
            || staged.link_count != 2
            || staged.bytes != final_leaf.bytes
        {
            bail!("nlink=2 final publication has an alternate staging identity")
        }
        unlink_exact(&temp, &staged)?;
        File::open(parent)?.sync_all()?;
        let repaired =
            read_leaf(path, identity, false)?.context("recovered final publication disappeared")?;
        if repaired.bytes != final_leaf.bytes || repaired.link_count != 1 {
            bail!("recovered final publication failed nlink=1 readback")
        }
        return Ok(Some(repaired.bytes));
    }
    Ok(Some(final_leaf.bytes))
}

#[derive(Debug)]
struct LeafStateV2 {
    bytes: Vec<u8>,
    device: libc::dev_t,
    inode: libc::ino_t,
    link_count: libc::nlink_t,
}

fn open_or_create_prefix_temp(
    temp: &Path,
    expected: &[u8],
    identity: PublishIdentityV2,
) -> Result<File> {
    let encoded = path_cstring(temp)?;
    // SAFETY: exact deterministic staging leaf; exclusive creation and terminal no-follow.
    let raw = unsafe {
        libc::open(
            encoded.as_ptr(),
            libc::O_RDWR | libc::O_CREAT | libc::O_EXCL | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            libc::c_uint::from(identity.permissions),
        )
    };
    let descriptor = if raw >= 0 {
        // SAFETY: successful exclusive open transfers descriptor ownership.
        let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
        // SAFETY: descriptor is live and all identity fields are compiled by its caller.
        if unsafe {
            libc::fchown(
                descriptor.as_raw_fd(),
                identity.owner_uid,
                identity.owner_gid,
            )
        } != 0
            || unsafe { libc::fchmod(descriptor.as_raw_fd(), identity.permissions) } != 0
        {
            return Err(std::io::Error::last_os_error()).context("bind fixed staging identity");
        }
        descriptor
    } else {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EEXIST) {
            return Err(error).context("create fixed staging leaf");
        }
        // SAFETY: exact existing staging leaf, opened no-follow for prefix recovery.
        let raw = unsafe {
            libc::open(
                encoded.as_ptr(),
                libc::O_RDWR | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if raw < 0 {
            return Err(std::io::Error::last_os_error()).context("open fixed staging residue");
        }
        // SAFETY: successful open transfers descriptor ownership.
        unsafe { OwnedFd::from_raw_fd(raw) }
    };
    let stat = fstat(descriptor.as_raw_fd())?;
    require_leaf_stat(&stat, identity, false)?;
    let mut file = File::from(descriptor);
    let mut prefix = Vec::new();
    file.read_to_end(&mut prefix)?;
    if prefix.len() > expected.len() || !expected.starts_with(&prefix) {
        bail!("fixed staging residue is not an exact expected prefix")
    }
    Ok(file)
}

fn read_leaf(
    path: &Path,
    identity: PublishIdentityV2,
    allow_linked: bool,
) -> Result<Option<LeafStateV2>> {
    let encoded = path_cstring(path)?;
    // SAFETY: exact fixed leaf, terminal no-follow.
    let raw = unsafe {
        libc::open(
            encoded.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if raw < 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ENOENT) {
            return Ok(None);
        }
        return Err(error).context("open fixed publication leaf");
    }
    // SAFETY: successful open transfers descriptor ownership.
    let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
    let before = fstat(descriptor.as_raw_fd())?;
    require_leaf_stat(&before, identity, allow_linked)?;
    if before.st_size < 0 || usize::try_from(before.st_size)? > identity.maximum_bytes {
        bail!("fixed publication leaf size is outside its bound")
    }
    let mut bytes = Vec::with_capacity(usize::try_from(before.st_size)?);
    File::from(descriptor).read_to_end(&mut bytes)?;
    let after = lstat(path)?.context("fixed publication leaf disappeared during read")?;
    if !same_identity(&before, &after) || after.st_size != libc::off_t::try_from(bytes.len())? {
        bail!("fixed publication leaf changed during stable read")
    }
    Ok(Some(LeafStateV2 {
        bytes,
        device: before.st_dev,
        inode: before.st_ino,
        link_count: before.st_nlink,
    }))
}

fn reattest_final(path: &Path, expected: &[u8], identity: PublishIdentityV2) -> Result<()> {
    let final_leaf =
        read_leaf(path, identity, false)?.context("fixed publication final leaf is absent")?;
    if final_leaf.bytes != expected || final_leaf.link_count != 1 {
        bail!("fixed publication failed exact final readback")
    }
    Ok(())
}

fn unlink_exact(path: &Path, expected: &LeafStateV2) -> Result<()> {
    let live = lstat(path)?.context("fixed staging leaf disappeared before unlink")?;
    if live.st_dev != expected.device
        || live.st_ino != expected.inode
        || live.st_nlink != expected.link_count
    {
        bail!("fixed staging identity changed before unlink")
    }
    let encoded = path_cstring(path)?;
    // SAFETY: exact deterministic staging path, identity-conditioned immediately above.
    if unsafe { libc::unlink(encoded.as_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("unlink exact staging residue");
    }
    Ok(())
}

fn require_parent(path: &Path, identity: PublishIdentityV2) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path).context("inspect fixed publication parent")?;
    if !metadata.file_type().is_dir()
        || metadata.uid() != identity.parent_uid
        || metadata.gid() != identity.parent_gid
        || (metadata.mode() & u32::from(libc::S_IFMT | 0o7777)) != u32::from(identity.parent_mode)
    {
        bail!("fixed publication parent identity changed")
    }
    Ok(())
}

fn require_leaf_stat(
    stat: &libc::stat,
    identity: PublishIdentityV2,
    allow_linked: bool,
) -> Result<()> {
    let valid_links = stat.st_nlink == 1 || (allow_linked && stat.st_nlink == 2);
    if stat.st_uid != identity.owner_uid
        || stat.st_gid != identity.owner_gid
        || !valid_links
        || (stat.st_mode & (libc::S_IFMT | 0o7777))
            != libc::mode_t::from(libc::S_IFREG | identity.permissions)
    {
        bail!("fixed publication leaf owner, mode, type, or link count changed")
    }
    Ok(())
}

fn fstat(fd: i32) -> Result<libc::stat> {
    let mut value = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: live descriptor and writable stat storage.
    if unsafe { libc::fstat(fd, value.as_mut_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("fstat fixed publication leaf");
    }
    // SAFETY: successful fstat initialized the value.
    Ok(unsafe { value.assume_init() })
}

fn lstat(path: &Path) -> Result<Option<libc::stat>> {
    let encoded = path_cstring(path)?;
    let mut value = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: exact path and writable stat storage.
    if unsafe { libc::lstat(encoded.as_ptr(), value.as_mut_ptr()) } == 0 {
        // SAFETY: successful lstat initialized the value.
        return Ok(Some(unsafe { value.assume_init() }));
    }
    let error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ENOENT) {
        return Ok(None);
    }
    Err(error).context("lstat fixed publication leaf")
}

fn same_identity(left: &libc::stat, right: &libc::stat) -> bool {
    left.st_dev == right.st_dev
        && left.st_ino == right.st_ino
        && left.st_uid == right.st_uid
        && left.st_gid == right.st_gid
        && left.st_mode == right.st_mode
        && left.st_nlink == right.st_nlink
        && left.st_size == right.st_size
        && left.st_mtime == right.st_mtime
        && left.st_mtime_nsec == right.st_mtime_nsec
}

fn path_cstring(path: &Path) -> Result<CString> {
    CString::new(path.as_os_str().as_bytes()).context("fixed publication path contains NUL")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    fn fixture(name: &str) -> (PathBuf, PublishIdentityV2) {
        let root = std::env::temp_dir().join(format!(
            "substrate-r3-immutable-publish-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir(&root).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        // SAFETY: read-only process credential queries.
        let uid = unsafe { libc::geteuid() };
        // SAFETY: read-only process credential queries.
        let gid = unsafe { libc::getegid() };
        (
            root,
            PublishIdentityV2 {
                owner_uid: uid,
                owner_gid: gid,
                permissions: 0o600,
                parent_uid: uid,
                parent_gid: gid,
                parent_mode: libc::S_IFDIR | 0o700,
                maximum_bytes: 1024,
            },
        )
    }

    fn write_leaf(path: &Path, bytes: &[u8]) {
        std::fs::write(path, bytes).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
        File::open(path).unwrap().sync_all().unwrap();
    }

    #[test]
    fn immutable_fault_matrix_never_discards_an_alternate_object() {
        use RecoveryStateV2 as S;
        assert_eq!(
            classify_recovery_state(false, true, false, true, false, false),
            S::StartOrResumeTemp
        );
        assert_eq!(
            classify_recovery_state(false, true, true, true, false, false),
            S::StartOrResumeTemp
        );
        assert_eq!(
            classify_recovery_state(true, false, true, true, true, false),
            S::RemoveLinkedTempThenComplete
        );
        assert_eq!(
            classify_recovery_state(true, false, false, true, false, false),
            S::Complete
        );
        assert_eq!(
            classify_recovery_state(true, false, true, true, false, false),
            S::PreserveAndFail
        );
        assert_eq!(
            classify_recovery_state(false, true, true, false, false, false),
            S::PreserveAndFail
        );
    }

    #[test]
    fn replacement_fault_matrix_requires_exact_predecessor_or_completed_successor() {
        use RecoveryStateV2 as S;
        assert_eq!(
            classify_recovery_state(false, true, false, true, false, true),
            S::ReplaceFromPredecessor
        );
        assert_eq!(
            classify_recovery_state(false, true, true, true, false, true),
            S::ReplaceFromPredecessor
        );
        assert_eq!(
            classify_recovery_state(true, false, false, true, false, true),
            S::Complete
        );
        assert_eq!(
            classify_recovery_state(false, false, true, true, false, true),
            S::PreserveAndFail
        );
        assert_eq!(
            classify_recovery_state(false, true, true, false, false, true),
            S::PreserveAndFail
        );
    }

    #[test]
    fn recovers_partial_write_and_linked_nlink_two_residue() {
        let (root, identity) = fixture("immutable");
        let final_path = root.join("receipt.json");
        let temp = root.join(".receipt.json.next");
        let expected = br#"{"complete":true}"#;
        write_leaf(&temp, &expected[..5]);
        publish_exact_file(&final_path, expected, identity, PublishMode::Immutable).unwrap();
        assert_eq!(std::fs::read(&final_path).unwrap(), expected);
        assert!(!temp.exists());

        // Recreate the exact post-link/pre-unlink crash state and prove recovery removes only the
        // same inode's staging name before accepting the final nlink=1 leaf.
        std::fs::hard_link(&final_path, &temp).unwrap();
        assert_eq!(std::fs::metadata(&final_path).unwrap().nlink(), 2);
        publish_exact_file(&final_path, expected, identity, PublishMode::Immutable).unwrap();
        assert_eq!(std::fs::metadata(&final_path).unwrap().nlink(), 1);
        assert!(!temp.exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn replacement_resumes_prefix_but_preserves_alternate_staging_bytes() {
        let (root, identity) = fixture("replace");
        let final_path = root.join("progress.json");
        let temp = root.join(".progress.json.next");
        let old = br#"{"generation":1}"#;
        let new = br#"{"generation":2}"#;
        write_leaf(&final_path, old);
        write_leaf(&temp, &new[..7]);
        publish_exact_file(
            &final_path,
            new,
            identity,
            PublishMode::ReplaceExact { predecessor: old },
        )
        .unwrap();
        assert_eq!(std::fs::read(&final_path).unwrap(), new);
        assert!(!temp.exists());

        let alternate = root.join(".progress.json.next");
        write_leaf(&alternate, b"alternate");
        let error = publish_exact_file(
            &final_path,
            br#"{"generation":3}"#,
            identity,
            PublishMode::ReplaceExact { predecessor: new },
        )
        .unwrap_err();
        assert!(error.to_string().contains("alternate final or staging"));
        assert_eq!(std::fs::read(&alternate).unwrap(), b"alternate");
        std::fs::remove_dir_all(root).unwrap();
    }
}
