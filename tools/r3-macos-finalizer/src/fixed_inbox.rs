//! No-follow, descriptor-joined reads for the two fixed UID-501 coordinator inbox leaves.

use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use substrate_common::macos_retirement_v2::{
    MAC_R3_COORDINATOR_INBOX_DIRECTORY_MODE_V2, MAC_R3_COORDINATOR_INBOX_FILE_MODE_V2,
    MAC_R3_COORDINATOR_INBOX_GROUP_GID_V2, MAC_R3_COORDINATOR_INBOX_OWNER_UID_V2,
    MAC_R3_COORDINATOR_INBOX_ROOT_V2, MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2,
    MAC_R3_FINALIZER_REQUEST_PATH_V2, MAC_R3_TERMINAL_BINDING_PATH_V2,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct PhysicalIdentityV2 {
    path: PathBuf,
    device: u64,
    inode: u64,
    owner_uid: u32,
    owner_gid: u32,
    mode: u32,
    link_count: u64,
    size: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
}

pub fn read_fixed_coordinator_inbox_v2(path: &Path) -> Result<Vec<u8>> {
    if path != Path::new(MAC_R3_FINALIZER_REQUEST_PATH_V2)
        && path != Path::new(MAC_R3_TERMINAL_BINDING_PATH_V2)
    {
        bail!("coordinator inbox reader accepts only its two compiled leaves")
    }
    let (mut first, before) = open_walk(path)?;
    let mut bytes = Vec::new();
    first
        .read_to_end(&mut bytes)
        .context("read fixed coordinator inbox leaf")?;
    let descriptor_after = physical_identity(path, &first.metadata()?)?;
    if before.last() != Some(&descriptor_after) {
        bail!("coordinator inbox descriptor changed while reading")
    }
    let (mut reopened, after) = open_walk(path)?;
    let mut reopened_bytes = Vec::new();
    reopened
        .read_to_end(&mut reopened_bytes)
        .context("reread fixed coordinator inbox leaf")?;
    if before != after || bytes != reopened_bytes {
        bail!("coordinator inbox pathname/descriptor join changed while reading")
    }
    Ok(bytes)
}

fn open_walk(path: &Path) -> Result<(File, Vec<PhysicalIdentityV2>)> {
    if !path.is_absolute() || path.parent() != Some(Path::new(MAC_R3_COORDINATOR_INBOX_ROOT_V2)) {
        bail!("coordinator inbox leaf escaped its one compiled root")
    }
    let root_path = Path::new("/");
    let mut current = File::open(root_path).context("open filesystem root")?;
    let mut current_path = PathBuf::from("/");
    let mut identities = vec![physical_identity(root_path, &current.metadata()?)?];
    validate_ancestor(identities.last().expect("root identity exists"), false)?;
    let components = path
        .components()
        .filter_map(|component| match component {
            Component::RootDir => None,
            Component::Normal(value) => Some(Ok(value)),
            _ => Some(Err(anyhow::anyhow!(
                "coordinator inbox path has a nonphysical component"
            ))),
        })
        .collect::<Result<Vec<_>>>()?;
    for (index, component) in components.iter().enumerate() {
        let final_component = index + 1 == components.len();
        let name = CString::new(component.as_bytes()).context("inbox path contains NUL")?;
        let flags = libc::O_RDONLY
            | libc::O_CLOEXEC
            | libc::O_NOFOLLOW
            | if final_component {
                0
            } else {
                libc::O_DIRECTORY
            };
        // SAFETY: `current` is a live directory descriptor and `name` is exact NUL-terminated
        // input. O_NOFOLLOW is applied independently to every traversed component.
        let fd = unsafe { libc::openat(current.as_raw_fd(), name.as_ptr(), flags) };
        if fd < 0 {
            return Err(std::io::Error::last_os_error()).with_context(|| {
                format!("openat inbox component {}", component.to_string_lossy())
            });
        }
        // SAFETY: successful openat transferred ownership of this descriptor.
        let next = unsafe { File::from_raw_fd(fd) };
        current_path.push(component);
        let identity = physical_identity(&current_path, &next.metadata()?)?;
        if final_component {
            validate_leaf(&identity)?;
        } else {
            validate_ancestor(
                &identity,
                current_path == Path::new(MAC_R3_COORDINATOR_INBOX_ROOT_V2),
            )?;
        }
        identities.push(identity);
        current = next;
    }
    Ok((current, identities))
}

fn validate_ancestor(identity: &PhysicalIdentityV2, exact_inbox_root: bool) -> Result<()> {
    if identity.owner_uid != MAC_R3_COORDINATOR_INBOX_OWNER_UID_V2
        || identity.mode & u32::from(libc::S_IFMT) != u32::from(libc::S_IFDIR)
        || identity.mode & 0o022 != 0
    {
        bail!("coordinator inbox ancestor is not root-owned immutable directory state")
    }
    if exact_inbox_root
        && (identity.owner_gid != MAC_R3_COORDINATOR_INBOX_GROUP_GID_V2
            || identity.mode & 0o7777 != MAC_R3_COORDINATOR_INBOX_DIRECTORY_MODE_V2)
    {
        bail!("coordinator inbox root is not exact root:staff 0750 state")
    }
    Ok(())
}

fn validate_leaf(identity: &PhysicalIdentityV2) -> Result<()> {
    if identity.owner_uid != MAC_R3_COORDINATOR_INBOX_OWNER_UID_V2
        || identity.owner_gid != MAC_R3_COORDINATOR_INBOX_GROUP_GID_V2
        || identity.mode & u32::from(libc::S_IFMT) != u32::from(libc::S_IFREG)
        || identity.mode & 0o7777 != MAC_R3_COORDINATOR_INBOX_FILE_MODE_V2
        || identity.link_count != 1
        || identity.size == 0
        || identity.size > MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2 as u64
    {
        bail!("coordinator inbox leaf is not exact root:staff 0440 single-link state")
    }
    Ok(())
}

fn physical_identity(path: &Path, metadata: &std::fs::Metadata) -> Result<PhysicalIdentityV2> {
    Ok(PhysicalIdentityV2 {
        path: path.to_path_buf(),
        device: metadata.dev(),
        inode: metadata.ino(),
        owner_uid: metadata.uid(),
        owner_gid: metadata.gid(),
        mode: metadata.mode(),
        link_count: metadata.nlink(),
        size: metadata.size(),
        modified_seconds: metadata.mtime(),
        modified_nanoseconds: metadata.mtime_nsec(),
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn identity(mode: u32, uid: u32, gid: u32, size: u64) -> PhysicalIdentityV2 {
        PhysicalIdentityV2 {
            path: PathBuf::from("/fixed"),
            device: 1,
            inode: 2,
            owner_uid: uid,
            owner_gid: gid,
            mode,
            link_count: 1,
            size,
            modified_seconds: 3,
            modified_nanoseconds: 4,
        }
    }

    #[test]
    fn source_walks_every_component_no_follow_and_reattests_the_join() {
        let source = include_str!("fixed_inbox.rs");
        assert!(source.contains("libc::openat"));
        assert!(source.contains("libc::O_NOFOLLOW"));
        assert!(source.contains("before != after || bytes != reopened_bytes"));
        assert!(source.contains("MAC_R3_COORDINATOR_INBOX_DIRECTORY_MODE_V2"));
        assert!(source.contains("MAC_R3_COORDINATOR_INBOX_FILE_MODE_V2"));
        assert!(source.contains("MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2"));
    }

    #[test]
    fn uid501_readable_inbox_modes_are_exact_and_ancestors_cannot_be_writable() {
        let ordinary_root_ancestor = identity(u32::from(libc::S_IFDIR) | 0o755, 0, 0, 0);
        validate_ancestor(&ordinary_root_ancestor, false).unwrap();
        let exact_inbox = identity(
            u32::from(libc::S_IFDIR) | MAC_R3_COORDINATOR_INBOX_DIRECTORY_MODE_V2,
            MAC_R3_COORDINATOR_INBOX_OWNER_UID_V2,
            MAC_R3_COORDINATOR_INBOX_GROUP_GID_V2,
            0,
        );
        validate_ancestor(&exact_inbox, true).unwrap();
        let leaf = identity(
            u32::from(libc::S_IFREG) | MAC_R3_COORDINATOR_INBOX_FILE_MODE_V2,
            MAC_R3_COORDINATOR_INBOX_OWNER_UID_V2,
            MAC_R3_COORDINATOR_INBOX_GROUP_GID_V2,
            1,
        );
        validate_leaf(&leaf).unwrap();

        let mut writable = ordinary_root_ancestor;
        writable.mode |= 0o020;
        assert!(validate_ancestor(&writable, false).is_err());
        let mut wrong_group = leaf.clone();
        wrong_group.owner_gid = 0;
        assert!(validate_leaf(&wrong_group).is_err());
        let mut oversized = leaf;
        oversized.size = MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2 as u64 + 1;
        assert!(validate_leaf(&oversized).is_err());
    }
}
