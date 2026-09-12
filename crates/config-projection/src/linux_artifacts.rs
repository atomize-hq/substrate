use std::collections::{BTreeMap, BTreeSet};
use std::ffi::CString;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{FileExt, MetadataExt, OpenOptionsExt};
use std::path::{Component, Path, PathBuf};

use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    ConfigProjectionCodecV1, ConfigProjectionFailureV1, DirectoryPhysicalIdentityV1,
    E3ElfExecutionModelV1, E3StaticPieRelocationMetadataV1, InstallerArtifactBuildInputV1,
    InstallerArtifactSourceHeadV1, InstallerArtifactSourceRecordV1, InstallerArtifactSourceRefV1,
    InstallerArtifactSourceStoreV1, InstallerArtifactSourceStreamV1,
    RuntimeArtifactAuthorityRoleV1, RuntimeArtifactManifestEntryV1, RuntimeArtifactProvenanceV1,
    RuntimeSupportDirectoryV1, Timestamp, TrustedRuntimeArtifactManifestV1,
};

pub struct LinuxArtifactSourceV1;

struct HeldAbsoluteDirectoryV1 {
    descriptor: File,
    path: PathBuf,
    device_id: u64,
    inode: u64,
}

fn open_directory_component_v1(
    parent: BorrowedFd<'_>,
    component: &std::ffi::OsStr,
) -> Result<File, ConfigProjectionFailureV1> {
    let component =
        CString::new(component.as_bytes()).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
    // SAFETY: the parent descriptor and component string remain live for the call.
    let descriptor = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            component.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if descriptor < 0 {
        return Err(ConfigProjectionFailureV1::MissingPreparation);
    }
    // SAFETY: successful `openat` returned one uniquely owned descriptor.
    Ok(unsafe { File::from_raw_fd(descriptor) })
}

fn open_absolute_directory_v1(
    path: &Path,
) -> Result<HeldAbsoluteDirectoryV1, ConfigProjectionFailureV1> {
    if !path.is_absolute() {
        return Err(ConfigProjectionFailureV1::WrongBinding);
    }
    let mut descriptor = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open("/")
        .map_err(|_| ConfigProjectionFailureV1::MissingPreparation)?;
    for component in path.components() {
        match component {
            Component::RootDir => {}
            Component::Normal(component) => {
                descriptor = open_directory_component_v1(descriptor.as_fd(), component)?;
            }
            _ => return Err(ConfigProjectionFailureV1::WrongBinding),
        }
    }
    let metadata = descriptor
        .metadata()
        .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
    let descriptor_path = std::fs::read_link(format!("/proc/self/fd/{}", descriptor.as_raw_fd()))
        .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
    if !metadata.is_dir() || metadata.dev() == 0 || metadata.ino() == 0 || descriptor_path != path {
        return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
    }
    Ok(HeldAbsoluteDirectoryV1 {
        descriptor,
        path: path.to_path_buf(),
        device_id: metadata.dev(),
        inode: metadata.ino(),
    })
}

fn revalidate_absolute_directory_v1(
    held: &HeldAbsoluteDirectoryV1,
) -> Result<(), ConfigProjectionFailureV1> {
    let reopened = open_absolute_directory_v1(&held.path)?;
    let after = held
        .descriptor
        .metadata()
        .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
    if reopened.device_id != held.device_id
        || reopened.inode != held.inode
        || after.dev() != held.device_id
        || after.ino() != held.inode
    {
        return Err(ConfigProjectionFailureV1::PartialPublication);
    }
    Ok(())
}

fn open_store_directory_v1(
    parent: &File,
    name: &str,
    store_device: u64,
) -> Result<File, ConfigProjectionFailureV1> {
    if name.is_empty() || name.contains('/') || matches!(name, "." | "..") {
        return Err(ConfigProjectionFailureV1::WrongBinding);
    }
    let directory = open_directory_component_v1(parent.as_fd(), std::ffi::OsStr::new(name))?;
    let metadata = directory
        .metadata()
        .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
    if !metadata.is_dir() || metadata.dev() != store_device {
        return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
    }
    Ok(directory)
}

fn open_file_component_v1(parent: &File, name: &str) -> Result<File, ConfigProjectionFailureV1> {
    if name.is_empty() || name.contains('/') || matches!(name, "." | "..") {
        return Err(ConfigProjectionFailureV1::WrongBinding);
    }
    let name = CString::new(name).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
    // SAFETY: the parent descriptor and component string remain live for the call.
    let descriptor = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if descriptor < 0 {
        return Err(ConfigProjectionFailureV1::MissingPreparation);
    }
    // SAFETY: successful `openat` returned one uniquely owned descriptor.
    Ok(unsafe { File::from_raw_fd(descriptor) })
}

fn resolve_exact_installed_ca_bundle_v1(
    filesystem_root: &File,
) -> Result<File, ConfigProjectionFailureV1> {
    const LOGICAL_LEAF: &str = "ca-certificates.crt";
    const LINK_TARGET: &[u8] = b"../../ca-certificates/extracted/tls-ca-bundle.pem";
    const MAX_SUPPORT_BYTES: u64 = 64 * 1024 * 1024;

    fn mount_id(descriptor: &File) -> Result<u64, ConfigProjectionFailureV1> {
        let path = CString::new(format!("/proc/self/fdinfo/{}", descriptor.as_raw_fd())).unwrap();
        // SAFETY: the fixed procfs path remains live for the call.
        let fd = unsafe {
            libc::open(
                path.as_ptr(),
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if fd < 0 {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        // SAFETY: successful `open` returned one uniquely owned descriptor.
        let mut fdinfo = unsafe { File::from_raw_fd(fd) };
        let mut payload = Vec::new();
        fdinfo
            .by_ref()
            .take(4097)
            .read_to_end(&mut payload)
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        if payload.len() > 4096 {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let prefix = b"mnt_id:\t";
        let values = payload
            .split(|byte| *byte == b'\n')
            .filter_map(|line| line.strip_prefix(prefix))
            .collect::<Vec<_>>();
        if values.len() != 1 || values[0].is_empty() || !values[0].iter().all(u8::is_ascii_digit) {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let value = std::str::from_utf8(values[0])
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value != 0)
            .ok_or(ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        Ok(value)
    }

    fn open_trusted_directory(
        parent: &File,
        name: &str,
        boundary_device: Option<u64>,
        boundary_mount_id: Option<u64>,
    ) -> Result<File, ConfigProjectionFailureV1> {
        let name = CString::new(name).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        // SAFETY: the parent descriptor and component string remain live for the call.
        let descriptor = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if descriptor < 0 {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        // SAFETY: successful `openat` returned one uniquely owned descriptor.
        let descriptor = unsafe { File::from_raw_fd(descriptor) };
        let metadata = descriptor
            .metadata()
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        if !metadata.is_dir()
            || metadata.uid() != 0
            || metadata.mode() & 0o022 != 0
            || boundary_device.is_some_and(|device| metadata.dev() != device)
            || boundary_mount_id.is_some_and(|id| mount_id(&descriptor).ok() != Some(id))
        {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        Ok(descriptor)
    }

    fn named_metadata(parent: &File, name: &str) -> Result<libc::stat, ConfigProjectionFailureV1> {
        let name = CString::new(name).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: the parent and component are live and the output storage is writable.
        if unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                name.as_ptr(),
                metadata.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } != 0
        {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        // SAFETY: successful `fstatat` initialized the structure.
        Ok(unsafe { metadata.assume_init() })
    }

    fn open_trusted_endpoint(
        parent: &File,
        name: &str,
        boundary_device: u64,
        boundary_mount_id: u64,
    ) -> Result<(File, std::fs::Metadata, String), ConfigProjectionFailureV1> {
        let name = CString::new(name).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        // SAFETY: the parent descriptor and component string remain live for the call.
        let descriptor = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if descriptor < 0 {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        // SAFETY: successful `openat` returned one uniquely owned descriptor.
        let descriptor = unsafe { File::from_raw_fd(descriptor) };
        let before = descriptor
            .metadata()
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        if !before.is_file()
            || before.dev() != boundary_device
            || mount_id(&descriptor)? != boundary_mount_id
            || before.uid() != 0
            || before.nlink() != 1
            || before.mode() & 0o022 != 0
            || before.len() > MAX_SUPPORT_BYTES
        {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let mut digest = Sha256::new();
        let mut offset = 0_u64;
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let count = descriptor
                .read_at(&mut buffer, offset)
                .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
            if count == 0 {
                break;
            }
            digest.update(&buffer[..count]);
            offset = offset
                .checked_add(count as u64)
                .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
        }
        let after = descriptor
            .metadata()
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        if offset != before.len()
            || (
                before.dev(),
                before.ino(),
                before.mode(),
                before.uid(),
                before.gid(),
                before.nlink(),
                before.len(),
            ) != (
                after.dev(),
                after.ino(),
                after.mode(),
                after.uid(),
                after.gid(),
                after.nlink(),
                after.len(),
            )
        {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        Ok((descriptor, before, format!("{:x}", digest.finalize())))
    }

    fn resolve_once(
        filesystem_root: &File,
    ) -> Result<(File, std::fs::Metadata, String), ConfigProjectionFailureV1> {
        let etc = open_trusted_directory(filesystem_root, "etc", None, None)?;
        let boundary_device = etc
            .metadata()
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?
            .dev();
        let boundary_mount_id = mount_id(&etc)?;
        let ssl =
            open_trusted_directory(&etc, "ssl", Some(boundary_device), Some(boundary_mount_id))?;
        let certs = open_trusted_directory(
            &ssl,
            "certs",
            Some(boundary_device),
            Some(boundary_mount_id),
        )?;
        let leaf = named_metadata(&certs, LOGICAL_LEAF)?;
        if leaf.st_mode & libc::S_IFMT == libc::S_IFREG {
            return open_trusted_endpoint(&certs, LOGICAL_LEAF, boundary_device, boundary_mount_id);
        }
        if leaf.st_mode & libc::S_IFMT != libc::S_IFLNK
            || leaf.st_uid != 0
            || leaf.st_dev != boundary_device
        {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let leaf_name = CString::new(LOGICAL_LEAF).unwrap();
        let mut target = [0_u8; LINK_TARGET.len() + 1];
        // SAFETY: the parent, component, and bounded output buffer remain live for the call.
        let length = unsafe {
            libc::readlinkat(
                certs.as_raw_fd(),
                leaf_name.as_ptr(),
                target.as_mut_ptr().cast(),
                target.len(),
            )
        };
        if length != LINK_TARGET.len() as isize || &target[..LINK_TARGET.len()] != LINK_TARGET {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }

        // The only admitted target starts at /etc/ssl/certs, pops exactly twice,
        // and therefore remains rooted at the held /etc descriptor.
        let mut stack = vec![etc, ssl, certs];
        for _ in 0..2 {
            if stack.len() <= 1 {
                return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
            }
            stack.pop();
        }
        let ca_certificates = open_trusted_directory(
            stack.last().unwrap(),
            "ca-certificates",
            Some(boundary_device),
            Some(boundary_mount_id),
        )?;
        let extracted = open_trusted_directory(
            &ca_certificates,
            "extracted",
            Some(boundary_device),
            Some(boundary_mount_id),
        )?;
        open_trusted_endpoint(
            &extracted,
            "tls-ca-bundle.pem",
            boundary_device,
            boundary_mount_id,
        )
    }

    let (first, first_metadata, first_digest) = resolve_once(filesystem_root)?;
    let (_, second_metadata, second_digest) = resolve_once(filesystem_root)?;
    if (
        first_metadata.dev(),
        first_metadata.ino(),
        first_metadata.mode(),
        first_metadata.uid(),
        first_metadata.gid(),
        first_metadata.nlink(),
        first_metadata.len(),
        first_digest,
    ) != (
        second_metadata.dev(),
        second_metadata.ino(),
        second_metadata.mode(),
        second_metadata.uid(),
        second_metadata.gid(),
        second_metadata.nlink(),
        second_metadata.len(),
        second_digest,
    ) {
        return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
    }
    Ok(first)
}

impl LinuxArtifactSourceV1 {
    pub fn validate_store(
        root: &Path,
        expected_stream: InstallerArtifactSourceStreamV1,
    ) -> Result<
        (
            InstallerArtifactSourceStoreV1,
            InstallerArtifactSourceHeadV1,
            InstallerArtifactSourceRecordV1,
        ),
        ConfigProjectionFailureV1,
    > {
        fn hash_omitting<T: Serialize>(
            domain: &str,
            field: &str,
            value: &T,
            omitted: &str,
        ) -> Result<String, ConfigProjectionFailureV1> {
            let mut value =
                serde_json::to_value(value).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
            value
                .as_object_mut()
                .ok_or(ConfigProjectionFailureV1::Malformed)?
                .remove(omitted);
            ConfigProjectionCodecV1::domain_sha256("", &json!({"domain": domain, field: value}))
        }
        fn read_canonical<T: serde::de::DeserializeOwned + Serialize>(
            parent: &File,
            name: &str,
            expected_gid: u32,
        ) -> Result<T, ConfigProjectionFailureV1> {
            let mut file = open_file_component_v1(parent, name)?;
            let metadata = file
                .metadata()
                .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            if !metadata.is_file()
                || metadata.nlink() != 1
                || metadata.mode() & 0o7777 != 0o640
                || metadata.uid() != 0
                || metadata.gid() != expected_gid
                || metadata.len() > 16 * 1024 * 1024
            {
                return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
            }
            let mut bytes = Vec::with_capacity(metadata.len() as usize);
            file.read_to_end(&mut bytes)
                .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
            let after = file
                .metadata()
                .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            let name = CString::new(name).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
            let mut named = std::mem::MaybeUninit::<libc::stat>::uninit();
            // SAFETY: the parent, component string, and writable stat storage are live.
            if unsafe {
                libc::fstatat(
                    parent.as_raw_fd(),
                    name.as_ptr(),
                    named.as_mut_ptr(),
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            } != 0
            {
                return Err(ConfigProjectionFailureV1::PartialPublication);
            }
            // SAFETY: successful `fstatat` initialized the structure.
            let named = unsafe { named.assume_init() };
            if (
                metadata.dev(),
                metadata.ino(),
                metadata.mode(),
                metadata.len(),
            ) != (after.dev(), after.ino(), after.mode(), after.len())
                || named.st_dev != metadata.dev()
                || named.st_ino != metadata.ino()
                || named.st_mode != metadata.mode()
                || named.st_size as u64 != metadata.len()
            {
                return Err(ConfigProjectionFailureV1::PartialPublication);
            }
            ConfigProjectionCodecV1::decode_canonical_json(&bytes)
        }
        fn valid_id(value: &str, prefix: &str) -> bool {
            value
                .strip_prefix(prefix)
                .and_then(|uuid| Uuid::parse_str(uuid).ok().map(|parsed| (uuid, parsed)))
                .is_some_and(|(uuid, parsed)| {
                    parsed.get_version_num() == 7 && parsed.to_string() == uuid
                })
        }
        fn digest(value: &str) -> bool {
            value.len() == 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        }
        fn git_object_id(value: &str) -> bool {
            matches!(value.len(), 40 | 64)
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        }
        fn valid_timestamp(value: &Timestamp) -> bool {
            let bytes = value.0.as_bytes();
            bytes.len() == 27
                && bytes.get(4) == Some(&b'-')
                && bytes.get(7) == Some(&b'-')
                && bytes.get(10) == Some(&b'T')
                && bytes.get(13) == Some(&b':')
                && bytes.get(16) == Some(&b':')
                && bytes.get(19) == Some(&b'.')
                && bytes.get(26) == Some(&b'Z')
                && bytes[20..26].iter().all(u8::is_ascii_digit)
                && chrono::DateTime::parse_from_rfc3339(&value.0).is_ok()
        }
        fn validate_record(
            record: &InstallerArtifactSourceRecordV1,
            reference: &InstallerArtifactSourceRefV1,
            source_store_id: &str,
            expected_stream: InstallerArtifactSourceStreamV1,
        ) -> Result<(), ConfigProjectionFailureV1> {
            if record.schema_version != 1
                || record.source_store_id != source_store_id
                || record.source_record_id != reference.source_record_id
                || !valid_id(&record.source_record_id, "iar_")
                || record.source_stream != expected_stream
                || record.revision != reference.revision
                || !valid_timestamp(&record.created_at)
                || record.record_hash != reference.record_hash
                || record.record_hash
                    != hash_omitting(
                        "substrate.e3.installer-artifact-source-record.v1",
                        "record",
                        record,
                        "record_hash",
                    )?
                || record.revision == 1 && record.predecessor_ref.is_some()
                || record.revision > 1
                    && record.predecessor_ref.as_ref().is_none_or(|predecessor| {
                        predecessor.source_store_id != record.source_store_id
                            || predecessor.revision + 1 != record.revision
                    })
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            let expected_entries: &[(&str, &str)] = match expected_stream {
                InstallerArtifactSourceStreamV1::SubstrateSourceBuild => &[
                    (
                        "substrate-gateway",
                        "/usr/local/lib/substrate/e3/substrate-gateway",
                    ),
                    (
                        "substrate-world-entry",
                        "/usr/local/lib/substrate/e3/substrate-world-entry",
                    ),
                ],
                InstallerArtifactSourceStreamV1::Codex0125OfficialArchive => &[(
                    "codex",
                    "/var/lib/substrate/world-deps/codex-runtime/bin/codex",
                )],
            };
            if record.entries.len() != expected_entries.len() {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            let mut paths = BTreeSet::new();
            for (entry, (component, installed_path)) in
                record.entries.iter().zip(expected_entries.iter())
            {
                if entry.component != *component
                    || entry.installed_absolute_path != *installed_path
                    || !paths.insert(entry.installed_absolute_path.as_str())
                    || entry.file_type != "regular"
                    || entry.mode != 0o755
                    || entry.owner_uid != 0
                    || entry.device_id == 0
                    || entry.inode == 0
                    || !digest(&entry.sha256)
                    || entry.runtime_support.schema_version != 1
                    || entry.runtime_support.support_policy_version != 1
                    || entry.runtime_support.elf_interpreter.is_some()
                    || entry.runtime_support.dynamic_loader_cache.is_some()
                    || !entry.runtime_support.ordered_elf_dependencies.is_empty()
                    || entry
                        .runtime_support
                        .ordered_present_common_files
                        .iter()
                        .map(|file| file.absolute_path.as_str())
                        .ne([
                            "/etc/hosts",
                            "/etc/nsswitch.conf",
                            "/etc/passwd",
                            "/etc/group",
                            "/etc/resolv.conf",
                            "/etc/ssl/certs/ca-certificates.crt",
                        ])
                    || entry
                        .runtime_support
                        .ordered_present_common_files
                        .iter()
                        .any(|file| {
                            file.device_id == 0
                                || file.inode == 0
                                || file.mode & 0o022 != 0
                                || !digest(&file.sha256)
                        })
                    || entry
                        .runtime_support
                        .system_config_mount_target
                        .absolute_path
                        != "/etc/codex"
                    || entry.runtime_support.system_config_mount_target.device_id == 0
                    || entry.runtime_support.system_config_mount_target.inode == 0
                    || entry.runtime_support.system_config_mount_target.mode != 0o755
                    || entry.runtime_support.system_config_mount_target.owner_uid != 0
                    || entry.runtime_support.system_config_mount_target.owner_gid != 0
                    || !entry
                        .runtime_support
                        .system_config_mount_target
                        .ordered_entry_names
                        .is_empty()
                    || entry.runtime_support.manifest_hash
                        != hash_omitting(
                            "substrate.e3.runtime-support-manifest.v1",
                            "manifest",
                            &entry.runtime_support,
                            "manifest_hash",
                        )?
                    || entry.entry_hash
                        != hash_omitting(
                            "substrate.e3.installer-artifact-source-entry.v1",
                            "entry",
                            entry,
                            "entry_hash",
                        )?
                {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
            }
            match (&expected_stream, &record.build_input) {
                (
                    InstallerArtifactSourceStreamV1::Codex0125OfficialArchive,
                    InstallerArtifactBuildInputV1::CodexOfficialArchive {
                        version,
                        target_triple,
                        archive_name,
                        archive_url,
                        archive_sha256,
                        archive_entry_path,
                        extracted_executable_sha256,
                    },
                ) if version == "0.125.0"
                    && target_triple == "x86_64-unknown-linux-musl"
                    && archive_name == "codex-x86_64-unknown-linux-musl.tar.gz"
                    && archive_url == "https://github.com/openai/codex/releases/download/rust-v0.125.0/codex-x86_64-unknown-linux-musl.tar.gz"
                    && archive_sha256 == "4a20a53943a7e6a0c5fa4463d4e47c58dd8e553ecebde455a4107e9906bfb001"
                    && archive_entry_path == "codex-x86_64-unknown-linux-musl"
                    && extracted_executable_sha256 == "86dc42ac5823f25233d6dc4ec5ff34693afd8c32ff2d17b54c5eb0d15bc7d902"
                    && record.entries[0].sha256 == *extracted_executable_sha256 => {}
                (
                    InstallerArtifactSourceStreamV1::SubstrateSourceBuild,
                    InstallerArtifactBuildInputV1::SubstrateSourceBuild {
                        source_commit,
                        source_tree,
                        cargo_lock_sha256,
                        rustc_version,
                        target_triple,
                        profile,
                    },
                ) if git_object_id(source_commit)
                    && git_object_id(source_tree)
                    && digest(cargo_lock_sha256)
                    && !rustc_version.is_empty()
                    && target_triple == "x86_64-unknown-linux-musl"
                    && profile == "release" => {}
                _ => return Err(ConfigProjectionFailureV1::WrongBinding),
            }
            Ok(())
        }

        let held_root = open_absolute_directory_v1(root)?;
        let root_meta = held_root
            .descriptor
            .metadata()
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        if !root_meta.is_dir()
            || root_meta.mode() & 0o7777 != 0o750
            || root_meta.mode() & 0o022 != 0
            || root_meta.uid() != 0
        {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let lock = open_file_component_v1(&held_root.descriptor, "lock")?;
        let lock_meta = lock
            .metadata()
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        if !lock_meta.is_file()
            || lock_meta.nlink() != 1
            || lock_meta.mode() & 0o7777 != 0o640
            || lock_meta.uid() != 0
            || lock_meta.gid() != root_meta.gid()
        {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_SH) } != 0 {
            return Err(ConfigProjectionFailureV1::Conflict);
        }
        let result = (|| {
            let store: InstallerArtifactSourceStoreV1 =
                read_canonical(&held_root.descriptor, "source-store.json", root_meta.gid())?;
            if store.schema_version != 1
                || !valid_id(&store.source_store_id, "ias_")
                || store.root.physical_path != root.to_string_lossy()
                || store.root.physical_identity
                    != (crate::DirectoryPhysicalIdentityV1::Linux {
                        device_id: root_meta.dev(),
                        inode: root_meta.ino(),
                    })
                || !valid_timestamp(&store.created_at)
                || store.store_hash
                    != hash_omitting(
                        "substrate.e3.installer-artifact-source-store.v1",
                        "store",
                        &store,
                        "store_hash",
                    )?
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            let stream_name = match expected_stream {
                InstallerArtifactSourceStreamV1::SubstrateSourceBuild => "substrate-source-build",
                InstallerArtifactSourceStreamV1::Codex0125OfficialArchive => {
                    "codex-0.125.0-x86_64-unknown-linux-musl"
                }
            };
            let heads = open_store_directory_v1(&held_root.descriptor, "heads", root_meta.dev())?;
            let head: InstallerArtifactSourceHeadV1 =
                read_canonical(&heads, &format!("{stream_name}.json"), root_meta.gid())?;
            if head.schema_version != 1
                || head.source_store_id != store.source_store_id
                || head.source_stream != expected_stream
                || head.head_revision == 0
                || head.head_ref.source_store_id != store.source_store_id
                || !valid_id(&head.head_ref.source_record_id, "iar_")
                || head.head_ref.revision != head.head_revision
                || !digest(&head.head_ref.record_hash)
                || head.head_revision == 1 && head.predecessor_head_hash.is_some()
                || head.head_revision > 1
                    && head
                        .predecessor_head_hash
                        .as_ref()
                        .is_none_or(|hash| !digest(hash))
                || !valid_timestamp(&head.updated_at)
                || head.head_hash
                    != hash_omitting(
                        "substrate.e3.installer-artifact-source-head.v1",
                        "head",
                        &head,
                        "head_hash",
                    )?
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            let records =
                open_store_directory_v1(&held_root.descriptor, "records", root_meta.dev())?;
            let stream = open_store_directory_v1(&records, stream_name, root_meta.dev())?;
            let record: InstallerArtifactSourceRecordV1 = read_canonical(
                &stream,
                &format!(
                    "{:020}-{}.json",
                    head.head_ref.revision, head.head_ref.source_record_id
                ),
                root_meta.gid(),
            )?;
            validate_record(
                &record,
                &head.head_ref,
                &store.source_store_id,
                expected_stream,
            )?;
            let mut predecessor_ref = record.predecessor_ref.clone();
            while let Some(reference) = predecessor_ref {
                let predecessor: InstallerArtifactSourceRecordV1 = read_canonical(
                    &stream,
                    &format!(
                        "{:020}-{}.json",
                        reference.revision, reference.source_record_id
                    ),
                    root_meta.gid(),
                )?;
                validate_record(
                    &predecessor,
                    &reference,
                    &store.source_store_id,
                    expected_stream,
                )?;
                predecessor_ref = predecessor.predecessor_ref;
            }
            Ok((store, head, record))
        })();
        let root_revalidation = revalidate_absolute_directory_v1(&held_root);
        let unlock = unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_UN) };
        if unlock != 0 {
            return Err(ConfigProjectionFailureV1::Conflict);
        }
        root_revalidation?;
        result
    }

    pub fn resolve_ref(
        root: &Path,
        expected_stream: InstallerArtifactSourceStreamV1,
        reference: &InstallerArtifactSourceRefV1,
    ) -> Result<InstallerArtifactSourceRecordV1, ConfigProjectionFailureV1> {
        fn resolved_record_is_complete(
            record: &InstallerArtifactSourceRecordV1,
            expected_ref: &InstallerArtifactSourceRefV1,
            expected_stream: InstallerArtifactSourceStreamV1,
        ) -> Result<bool, ConfigProjectionFailureV1> {
            fn digest(value: &str) -> bool {
                value.len() == 64
                    && value
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            }
            fn object_id(value: &str) -> bool {
                matches!(value.len(), 40 | 64)
                    && value
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            }
            fn hash_omitting<T: Serialize>(
                domain: &str,
                field: &str,
                value: &T,
                omitted: &str,
            ) -> Result<String, ConfigProjectionFailureV1> {
                let mut value = serde_json::to_value(value)
                    .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
                value
                    .as_object_mut()
                    .ok_or(ConfigProjectionFailureV1::Malformed)?
                    .remove(omitted);
                ConfigProjectionCodecV1::domain_sha256("", &json!({"domain": domain, field: value}))
            }
            let valid_record_id = record
                .source_record_id
                .strip_prefix("iar_")
                .and_then(|value| Uuid::parse_str(value).ok().map(|parsed| (value, parsed)))
                .is_some_and(|(value, parsed)| {
                    parsed.get_version_num() == 7 && parsed.to_string() == value
                });
            let timestamp = record.created_at.0.as_bytes();
            if record.schema_version != 1
                || record.source_store_id != expected_ref.source_store_id
                || record.source_record_id != expected_ref.source_record_id
                || !valid_record_id
                || record.source_stream != expected_stream
                || record.revision != expected_ref.revision
                || record.record_hash != expected_ref.record_hash
                || !digest(&record.record_hash)
                || timestamp.len() != 27
                || timestamp.get(19) != Some(&b'.')
                || timestamp.get(26) != Some(&b'Z')
                || !timestamp[20..26].iter().all(u8::is_ascii_digit)
                || chrono::DateTime::parse_from_rfc3339(&record.created_at.0).is_err()
                || record.record_hash
                    != hash_omitting(
                        "substrate.e3.installer-artifact-source-record.v1",
                        "record",
                        record,
                        "record_hash",
                    )?
                || record.revision == 1 && record.predecessor_ref.is_some()
                || record.revision > 1
                    && record.predecessor_ref.as_ref().is_none_or(|predecessor| {
                        predecessor.source_store_id != record.source_store_id
                            || predecessor.revision + 1 != record.revision
                            || !digest(&predecessor.record_hash)
                    })
            {
                return Ok(false);
            }
            let expected_entries: &[(&str, &str)] = match expected_stream {
                InstallerArtifactSourceStreamV1::SubstrateSourceBuild => &[
                    (
                        "substrate-gateway",
                        "/usr/local/lib/substrate/e3/substrate-gateway",
                    ),
                    (
                        "substrate-world-entry",
                        "/usr/local/lib/substrate/e3/substrate-world-entry",
                    ),
                ],
                InstallerArtifactSourceStreamV1::Codex0125OfficialArchive => &[(
                    "codex",
                    "/var/lib/substrate/world-deps/codex-runtime/bin/codex",
                )],
            };
            if record.entries.len() != expected_entries.len() {
                return Ok(false);
            }
            for (entry, (component, installed_path)) in
                record.entries.iter().zip(expected_entries.iter())
            {
                let support = &entry.runtime_support;
                if entry.component != *component
                    || entry.installed_absolute_path != *installed_path
                    || entry.file_type != "regular"
                    || entry.mode != 0o755
                    || entry.owner_uid != 0
                    || entry.device_id == 0
                    || entry.inode == 0
                    || !digest(&entry.sha256)
                    || support.schema_version != 1
                    || support.support_policy_version != 1
                    || support.elf_interpreter.is_some()
                    || support.dynamic_loader_cache.is_some()
                    || !support.ordered_elf_dependencies.is_empty()
                    || support
                        .ordered_present_common_files
                        .iter()
                        .map(|file| file.absolute_path.as_str())
                        .ne([
                            "/etc/hosts",
                            "/etc/nsswitch.conf",
                            "/etc/passwd",
                            "/etc/group",
                            "/etc/resolv.conf",
                            "/etc/ssl/certs/ca-certificates.crt",
                        ])
                    || support.ordered_present_common_files.iter().any(|file| {
                        file.device_id == 0
                            || file.inode == 0
                            || file.mode & 0o022 != 0
                            || !digest(&file.sha256)
                    })
                    || support.system_config_mount_target.absolute_path != "/etc/codex"
                    || support.system_config_mount_target.device_id == 0
                    || support.system_config_mount_target.inode == 0
                    || support.system_config_mount_target.mode != 0o755
                    || support.system_config_mount_target.owner_uid != 0
                    || support.system_config_mount_target.owner_gid != 0
                    || !support
                        .system_config_mount_target
                        .ordered_entry_names
                        .is_empty()
                    || support.manifest_hash
                        != hash_omitting(
                            "substrate.e3.runtime-support-manifest.v1",
                            "manifest",
                            support,
                            "manifest_hash",
                        )?
                    || entry.entry_hash
                        != hash_omitting(
                            "substrate.e3.installer-artifact-source-entry.v1",
                            "entry",
                            entry,
                            "entry_hash",
                        )?
                {
                    return Ok(false);
                }
            }
            let build_is_exact = match (&expected_stream, &record.build_input) {
                (
                    InstallerArtifactSourceStreamV1::SubstrateSourceBuild,
                    InstallerArtifactBuildInputV1::SubstrateSourceBuild {
                        source_commit,
                        source_tree,
                        cargo_lock_sha256,
                        rustc_version,
                        target_triple,
                        profile,
                    },
                ) => {
                    object_id(source_commit)
                        && object_id(source_tree)
                        && digest(cargo_lock_sha256)
                        && !rustc_version.is_empty()
                        && target_triple == "x86_64-unknown-linux-musl"
                        && profile == "release"
                }
                (
                    InstallerArtifactSourceStreamV1::Codex0125OfficialArchive,
                    InstallerArtifactBuildInputV1::CodexOfficialArchive {
                        version,
                        target_triple,
                        archive_name,
                        archive_url,
                        archive_sha256,
                        archive_entry_path,
                        extracted_executable_sha256,
                    },
                ) => {
                    version == "0.125.0"
                        && target_triple == "x86_64-unknown-linux-musl"
                        && archive_name == "codex-x86_64-unknown-linux-musl.tar.gz"
                        && archive_url == "https://github.com/openai/codex/releases/download/rust-v0.125.0/codex-x86_64-unknown-linux-musl.tar.gz"
                        && archive_sha256 == "4a20a53943a7e6a0c5fa4463d4e47c58dd8e553ecebde455a4107e9906bfb001"
                        && archive_entry_path == "codex-x86_64-unknown-linux-musl"
                        && extracted_executable_sha256 == "86dc42ac5823f25233d6dc4ec5ff34693afd8c32ff2d17b54c5eb0d15bc7d902"
                        && record.entries[0].sha256 == *extracted_executable_sha256
                }
                _ => false,
            };
            Ok(build_is_exact)
        }

        let (store, _head, mut record) = Self::validate_store(root, expected_stream)?;
        let held_root = open_absolute_directory_v1(root)?;
        let DirectoryPhysicalIdentityV1::Linux { device_id, inode } = &store.root.physical_identity;
        if held_root.device_id != *device_id
            || held_root.inode != *inode
            || store.root.physical_path != root.to_string_lossy()
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let root_metadata = held_root
            .descriptor
            .metadata()
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        let lock = open_file_component_v1(&held_root.descriptor, "lock")?;
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_SH) } != 0 {
            return Err(ConfigProjectionFailureV1::Conflict);
        }
        let stream_name = match expected_stream {
            InstallerArtifactSourceStreamV1::SubstrateSourceBuild => "substrate-source-build",
            InstallerArtifactSourceStreamV1::Codex0125OfficialArchive => {
                "codex-0.125.0-x86_64-unknown-linux-musl"
            }
        };
        let records =
            open_store_directory_v1(&held_root.descriptor, "records", root_metadata.dev())?;
        let stream = open_store_directory_v1(&records, stream_name, root_metadata.dev())?;
        let result = (|| {
            if reference.source_store_id != store.source_store_id {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            loop {
                if record.source_record_id == reference.source_record_id
                    && record.revision == reference.revision
                    && record.record_hash == reference.record_hash
                {
                    return Ok(record);
                }
                let predecessor = record
                    .predecessor_ref
                    .ok_or(ConfigProjectionFailureV1::WrongBinding)?;
                let mut file = open_file_component_v1(
                    &stream,
                    &format!(
                        "{:020}-{}.json",
                        predecessor.revision, predecessor.source_record_id
                    ),
                )?;
                let metadata = file
                    .metadata()
                    .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                if !metadata.is_file()
                    || metadata.nlink() != 1
                    || metadata.mode() & 0o7777 != 0o640
                    || metadata.uid() != 0
                    || metadata.gid() != root_metadata.gid()
                    || metadata.len() > 16 * 1024 * 1024
                {
                    return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
                }
                let mut bytes = Vec::with_capacity(metadata.len() as usize);
                file.read_to_end(&mut bytes)
                    .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
                record = ConfigProjectionCodecV1::decode_canonical_json(&bytes)?;
                if !resolved_record_is_complete(&record, &predecessor, expected_stream)? {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
            }
        })();
        let root_revalidation = revalidate_absolute_directory_v1(&held_root);
        let unlock = unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_UN) };
        if unlock != 0 {
            return Err(ConfigProjectionFailureV1::Conflict);
        }
        root_revalidation?;
        result
    }

    pub fn import_manifest<T>(
        substrate_store_root: &Path,
        codex_store_root: &Path,
        authority_store_id: &str,
        created_at: Timestamp,
        publish: impl FnOnce(&TrustedRuntimeArtifactManifestV1) -> Result<T, ConfigProjectionFailureV1>,
    ) -> Result<T, ConfigProjectionFailureV1> {
        fn hold_shared_lock(
            root: &HeldAbsoluteDirectoryV1,
        ) -> Result<File, ConfigProjectionFailureV1> {
            let root_metadata = root
                .descriptor
                .metadata()
                .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            let lock = open_file_component_v1(&root.descriptor, "lock")?;
            let metadata = lock
                .metadata()
                .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            if !metadata.is_file()
                || metadata.nlink() != 1
                || metadata.mode() & 0o7777 != 0o640
                || metadata.uid() != 0
                || metadata.gid() != root_metadata.gid()
            {
                return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
            }
            if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_SH) } != 0 {
                return Err(ConfigProjectionFailureV1::Conflict);
            }
            Ok(lock)
        }
        fn hash_omitting<T: Serialize>(
            domain: &str,
            field: &str,
            value: &T,
            omitted: &str,
        ) -> Result<String, ConfigProjectionFailureV1> {
            let mut value =
                serde_json::to_value(value).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
            value
                .as_object_mut()
                .ok_or(ConfigProjectionFailureV1::Malformed)?
                .remove(omitted);
            ConfigProjectionCodecV1::domain_sha256("", &json!({"domain": domain, field: value}))
        }
        fn open_absolute_file(
            filesystem_root: &File,
            path: &str,
        ) -> Result<File, ConfigProjectionFailureV1> {
            let mut components = Path::new(path).components();
            if components.next() != Some(Component::RootDir) {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            let mut directory = filesystem_root
                .try_clone()
                .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            let mut remaining = components.peekable();
            loop {
                let component = match remaining.next() {
                    Some(Component::Normal(component)) => component,
                    _ => return Err(ConfigProjectionFailureV1::WrongBinding),
                };
                if remaining.peek().is_none() {
                    let component = component
                        .to_str()
                        .ok_or(ConfigProjectionFailureV1::WrongBinding)?;
                    break open_file_component_v1(&directory, component);
                }
                directory = open_directory_component_v1(directory.as_fd(), component)?;
            }
        }
        fn file_identity(
            filesystem_root: &File,
            path: &str,
        ) -> Result<(File, std::fs::Metadata, String), ConfigProjectionFailureV1> {
            let mut file = open_absolute_file(filesystem_root, path)?;
            let metadata = file
                .metadata()
                .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            if !metadata.is_file()
                || metadata.nlink() != 1
                || metadata.mode() & 0o7777 != 0o755
                || metadata.uid() != 0
            {
                return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
            }
            // `system.posix_acl_access`, `security.capability`, and every other
            // executable-affecting xattr are outside the V1 authority vocabulary.
            // SAFETY: the descriptor is live and a null list buffer asks only for size.
            let xattr_bytes =
                unsafe { libc::flistxattr(file.as_raw_fd(), std::ptr::null_mut(), 0) };
            if xattr_bytes != 0 {
                return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
            }
            let mut hasher = Sha256::new();
            let mut buffer = [0u8; 64 * 1024];
            loop {
                let count = file
                    .read(&mut buffer)
                    .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
                if count == 0 {
                    break;
                }
                hasher.update(&buffer[..count]);
            }
            let after = file
                .metadata()
                .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            if (
                metadata.dev(),
                metadata.ino(),
                metadata.mode(),
                metadata.nlink(),
                metadata.len(),
            ) != (
                after.dev(),
                after.ino(),
                after.mode(),
                after.nlink(),
                after.len(),
            ) {
                return Err(ConfigProjectionFailureV1::PartialPublication);
            }
            Ok((file, metadata, format!("{:x}", hasher.finalize())))
        }
        fn support_file_identity(
            filesystem_root: &File,
            path: &str,
        ) -> Result<(File, crate::RuntimeSupportFileV1), ConfigProjectionFailureV1> {
            let mut file = if path == "/etc/ssl/certs/ca-certificates.crt" {
                resolve_exact_installed_ca_bundle_v1(filesystem_root)?
            } else {
                open_absolute_file(filesystem_root, path)?
            };
            let metadata = file
                .metadata()
                .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            if !metadata.is_file()
                || metadata.nlink() != 1
                || metadata.mode() & 0o022 != 0
                || metadata.len() > 64 * 1024 * 1024
            {
                return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
            }
            let mut hasher = Sha256::new();
            let mut buffer = [0u8; 64 * 1024];
            loop {
                let count = file
                    .read(&mut buffer)
                    .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
                if count == 0 {
                    break;
                }
                hasher.update(&buffer[..count]);
            }
            let after = file
                .metadata()
                .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            if (
                metadata.dev(),
                metadata.ino(),
                metadata.mode(),
                metadata.nlink(),
                metadata.len(),
            ) != (
                after.dev(),
                after.ino(),
                after.mode(),
                after.nlink(),
                after.len(),
            ) {
                return Err(ConfigProjectionFailureV1::PartialPublication);
            }
            Ok((
                file,
                crate::RuntimeSupportFileV1 {
                    absolute_path: path.to_string(),
                    device_id: metadata.dev(),
                    inode: metadata.ino(),
                    mode: metadata.mode() & 0o7777,
                    byte_length: metadata.len(),
                    sha256: format!("{:x}", hasher.finalize()),
                },
            ))
        }
        fn source_ref(record: &InstallerArtifactSourceRecordV1) -> InstallerArtifactSourceRefV1 {
            InstallerArtifactSourceRefV1 {
                source_store_id: record.source_store_id.clone(),
                source_record_id: record.source_record_id.clone(),
                revision: record.revision,
                record_hash: record.record_hash.clone(),
            }
        }

        let authority_uuid = authority_store_id
            .strip_prefix("cpa_")
            .and_then(|value| Uuid::parse_str(value).ok().map(|parsed| (value, parsed)))
            .filter(|(value, parsed)| {
                parsed.get_version_num() == 7 && parsed.to_string() == *value
            });
        let timestamp = created_at.0.as_bytes();
        if authority_uuid.is_none()
            || timestamp.len() != 27
            || timestamp.get(19) != Some(&b'.')
            || timestamp.get(26) != Some(&b'Z')
            || !timestamp[20..26].iter().all(u8::is_ascii_digit)
            || chrono::DateTime::parse_from_rfc3339(&created_at.0).is_err()
        {
            return Err(ConfigProjectionFailureV1::Malformed);
        }
        if substrate_store_root != Path::new("/var/lib/substrate/runtime-artifacts-v1")
            || codex_store_root != Path::new("/var/lib/substrate/world-deps/runtime-artifacts-v1")
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let substrate_source_root = open_absolute_directory_v1(substrate_store_root)?;
        let codex_source_root = open_absolute_directory_v1(codex_store_root)?;
        let _substrate_lock = hold_shared_lock(&substrate_source_root)?;
        let _codex_lock = hold_shared_lock(&codex_source_root)?;
        let (substrate_store, _, substrate) = Self::validate_store(
            substrate_store_root,
            InstallerArtifactSourceStreamV1::SubstrateSourceBuild,
        )?;
        let (codex_store, _, codex) = Self::validate_store(
            codex_store_root,
            InstallerArtifactSourceStreamV1::Codex0125OfficialArchive,
        )?;
        let DirectoryPhysicalIdentityV1::Linux {
            device_id: substrate_device,
            inode: substrate_inode,
        } = substrate_store.root.physical_identity;
        let DirectoryPhysicalIdentityV1::Linux {
            device_id: codex_device,
            inode: codex_inode,
        } = codex_store.root.physical_identity;
        if substrate_source_root.device_id != substrate_device
            || substrate_source_root.inode != substrate_inode
            || codex_source_root.device_id != codex_device
            || codex_source_root.inode != codex_inode
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let filesystem_root = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
            .open("/")
            .map_err(|_| ConfigProjectionFailureV1::MissingPreparation)?;
        let mut held_support_files = Vec::new();
        let mut common_files = Vec::new();
        for path in [
            "/etc/hosts",
            "/etc/nsswitch.conf",
            "/etc/passwd",
            "/etc/group",
            "/etc/resolv.conf",
            "/etc/ssl/certs/ca-certificates.crt",
        ] {
            let (file, identity) = support_file_identity(&filesystem_root, path)?;
            held_support_files.push(file);
            common_files.push(identity);
        }
        let system_config_mount_target =
            Self::validate_system_config_mount_target_v1(Path::new("/etc/codex"))?;
        let held_system_config_mount_target = open_absolute_directory_v1(Path::new("/etc/codex"))?;
        let held_mount_metadata = held_system_config_mount_target
            .descriptor
            .metadata()
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        if held_mount_metadata.dev() != system_config_mount_target.device_id
            || held_mount_metadata.ino() != system_config_mount_target.inode
        {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        let ordered = [
            (
                RuntimeArtifactAuthorityRoleV1::Codex0125,
                &codex.entries[0],
                source_ref(&codex),
                RuntimeArtifactProvenanceV1::OfficialCodexRelease {
                    version: "0.125.0".to_string(),
                    target_triple: "x86_64-unknown-linux-musl".to_string(),
                    archive_name: "codex-x86_64-unknown-linux-musl.tar.gz".to_string(),
                    archive_url: "https://github.com/openai/codex/releases/download/rust-v0.125.0/codex-x86_64-unknown-linux-musl.tar.gz".to_string(),
                    archive_sha256: "4a20a53943a7e6a0c5fa4463d4e47c58dd8e553ecebde455a4107e9906bfb001".to_string(),
                    archive_entry_path: "codex-x86_64-unknown-linux-musl".to_string(),
                    extracted_executable_sha256: "86dc42ac5823f25233d6dc4ec5ff34693afd8c32ff2d17b54c5eb0d15bc7d902".to_string(),
                },
            ),
            (
                RuntimeArtifactAuthorityRoleV1::ManagedGateway,
                &substrate.entries[0],
                source_ref(&substrate),
                RuntimeArtifactProvenanceV1::SubstrateSourceBuild {
                    component: "substrate-gateway".to_string(),
                    source_commit: match &substrate.build_input {
                        InstallerArtifactBuildInputV1::SubstrateSourceBuild { source_commit, .. } => source_commit.clone(),
                        _ => return Err(ConfigProjectionFailureV1::WrongBinding),
                    },
                    source_tree: match &substrate.build_input {
                        InstallerArtifactBuildInputV1::SubstrateSourceBuild { source_tree, .. } => source_tree.clone(),
                        _ => return Err(ConfigProjectionFailureV1::WrongBinding),
                    },
                    cargo_lock_sha256: match &substrate.build_input {
                        InstallerArtifactBuildInputV1::SubstrateSourceBuild { cargo_lock_sha256, .. } => cargo_lock_sha256.clone(),
                        _ => return Err(ConfigProjectionFailureV1::WrongBinding),
                    },
                    target_triple: "x86_64-unknown-linux-musl".to_string(),
                    profile: "release".to_string(),
                    executable_sha256: substrate.entries[0].sha256.clone(),
                },
            ),
            (
                RuntimeArtifactAuthorityRoleV1::WorldEntryWrapper,
                &substrate.entries[1],
                source_ref(&substrate),
                RuntimeArtifactProvenanceV1::SubstrateSourceBuild {
                    component: "substrate-world-entry".to_string(),
                    source_commit: match &substrate.build_input {
                        InstallerArtifactBuildInputV1::SubstrateSourceBuild { source_commit, .. } => source_commit.clone(),
                        _ => return Err(ConfigProjectionFailureV1::WrongBinding),
                    },
                    source_tree: match &substrate.build_input {
                        InstallerArtifactBuildInputV1::SubstrateSourceBuild { source_tree, .. } => source_tree.clone(),
                        _ => return Err(ConfigProjectionFailureV1::WrongBinding),
                    },
                    cargo_lock_sha256: match &substrate.build_input {
                        InstallerArtifactBuildInputV1::SubstrateSourceBuild { cargo_lock_sha256, .. } => cargo_lock_sha256.clone(),
                        _ => return Err(ConfigProjectionFailureV1::WrongBinding),
                    },
                    target_triple: "x86_64-unknown-linux-musl".to_string(),
                    profile: "release".to_string(),
                    executable_sha256: substrate.entries[1].sha256.clone(),
                },
            ),
        ];
        let mut entries = Vec::new();
        let mut held_executables = Vec::new();
        for (role, source, installer_source_ref, provenance) in ordered {
            let (file, metadata, sha256) =
                file_identity(&filesystem_root, &source.installed_absolute_path)?;
            let elf_execution_model = Self::validate_e3_static_elf_v1(&file)?;
            let mut recomputed_support = crate::E3RuntimeSupportManifestV1 {
                schema_version: 1,
                support_policy_version: 1,
                elf_execution_model,
                elf_interpreter: None,
                dynamic_loader_cache: None,
                ordered_elf_dependencies: Vec::new(),
                ordered_present_common_files: common_files.clone(),
                system_config_mount_target: system_config_mount_target.clone(),
                manifest_hash: String::new(),
            };
            recomputed_support.manifest_hash = hash_omitting(
                "substrate.e3.runtime-support-manifest.v1",
                "manifest",
                &recomputed_support,
                "manifest_hash",
            )?;
            if metadata.dev() != source.device_id
                || metadata.ino() != source.inode
                || metadata.uid() as u64 != source.owner_uid
                || metadata.len() != source.byte_length
                || sha256 != source.sha256
                || recomputed_support != source.runtime_support
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            let mut entry = RuntimeArtifactManifestEntryV1 {
                manifest_entry_id: format!("rae_{}", Uuid::now_v7()),
                authority_role: role,
                configured_absolute_path: source.installed_absolute_path.clone(),
                device_id: source.device_id,
                inode: source.inode,
                mode: source.mode,
                owner_uid: source.owner_uid,
                byte_length: source.byte_length,
                sha256: source.sha256.clone(),
                installer_source_ref,
                provenance,
                runtime_support: source.runtime_support.clone(),
                entry_hash: String::new(),
            };
            entry.entry_hash = hash_omitting(
                "substrate.e3.runtime-artifact-entry.v1",
                "entry",
                &entry,
                "entry_hash",
            )?;
            entries.push(entry);
            held_executables.push(file);
        }
        let mut manifest = TrustedRuntimeArtifactManifestV1 {
            schema_version: 1,
            authority_store_id: authority_store_id.to_string(),
            manifest_id: format!("ram_{}", Uuid::now_v7()),
            revision: 1,
            entries,
            created_at,
            manifest_hash: String::new(),
        };
        manifest.manifest_hash = hash_omitting(
            "substrate.e3.runtime-artifact-manifest.v1",
            "manifest",
            &manifest,
            "manifest_hash",
        )?;
        let result = publish(&manifest);
        revalidate_absolute_directory_v1(&substrate_source_root)?;
        revalidate_absolute_directory_v1(&codex_source_root)?;
        revalidate_absolute_directory_v1(&held_system_config_mount_target)?;
        let (_, revalidated_ca) =
            support_file_identity(&filesystem_root, "/etc/ssl/certs/ca-certificates.crt")?;
        if common_files.last() != Some(&revalidated_ca) {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        drop(held_executables);
        drop(held_support_files);
        drop(held_system_config_mount_target);
        result
    }

    pub fn validate_e3_static_elf_v1(
        file: &File,
    ) -> Result<E3ElfExecutionModelV1, ConfigProjectionFailureV1> {
        let metadata = file
            .metadata()
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        if !metadata.is_file() || metadata.nlink() != 1 || metadata.len() > 512 * 1024 * 1024 {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let mut bytes = vec![0; metadata.len() as usize];
        file.read_exact_at(&mut bytes, 0)
            .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
        if bytes.len() as u64 != metadata.len() {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        let u16_at = |offset: usize| -> Option<u16> {
            Some(u16::from_le_bytes(
                bytes.get(offset..offset + 2)?.try_into().ok()?,
            ))
        };
        let u32_at = |offset: usize| -> Option<u32> {
            Some(u32::from_le_bytes(
                bytes.get(offset..offset + 4)?.try_into().ok()?,
            ))
        };
        let u64_at = |offset: usize| -> Option<u64> {
            Some(u64::from_le_bytes(
                bytes.get(offset..offset + 8)?.try_into().ok()?,
            ))
        };
        if bytes.get(..16) != Some(b"\x7fELF\x02\x01\x01\0\0\0\0\0\0\0\0\0")
            || u16_at(18) != Some(62)
            || u32_at(20) != Some(1)
            || u16_at(52) != Some(64)
        {
            return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion);
        }
        let entry = u64_at(24).ok_or(ConfigProjectionFailureV1::Malformed)?;
        let elf_type = u16_at(16).ok_or(ConfigProjectionFailureV1::Malformed)?;
        let phoff = usize::try_from(u64_at(32).ok_or(ConfigProjectionFailureV1::Malformed)?)
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        let phentsize = usize::from(u16_at(54).ok_or(ConfigProjectionFailureV1::Malformed)?);
        let phnum = usize::from(u16_at(56).ok_or(ConfigProjectionFailureV1::Malformed)?);
        let program_table_end = phoff
            .checked_add(
                phnum
                    .checked_mul(phentsize)
                    .ok_or(ConfigProjectionFailureV1::Malformed)?,
            )
            .ok_or(ConfigProjectionFailureV1::Malformed)?;
        if phentsize != 56 || phnum == 0 || program_table_end > bytes.len() {
            return Err(ConfigProjectionFailureV1::Malformed);
        }
        let mut dynamic = None;
        let mut load_segments = Vec::new();
        for index in 0..phnum {
            let offset = phoff
                .checked_add(
                    index
                        .checked_mul(phentsize)
                        .ok_or(ConfigProjectionFailureV1::Malformed)?,
                )
                .ok_or(ConfigProjectionFailureV1::Malformed)?;
            let segment_type = u32_at(offset).ok_or(ConfigProjectionFailureV1::Malformed)?;
            let flags = u32_at(offset + 4).ok_or(ConfigProjectionFailureV1::Malformed)?;
            let file_offset = u64_at(offset + 8).ok_or(ConfigProjectionFailureV1::Malformed)?;
            let virtual_address =
                u64_at(offset + 16).ok_or(ConfigProjectionFailureV1::Malformed)?;
            let file_size = u64_at(offset + 32).ok_or(ConfigProjectionFailureV1::Malformed)?;
            let memory_size = u64_at(offset + 40).ok_or(ConfigProjectionFailureV1::Malformed)?;
            let alignment = u64_at(offset + 48).ok_or(ConfigProjectionFailureV1::Malformed)?;
            let file_end = file_offset
                .checked_add(file_size)
                .ok_or(ConfigProjectionFailureV1::Malformed)?;
            virtual_address
                .checked_add(memory_size)
                .ok_or(ConfigProjectionFailureV1::Malformed)?;
            if file_size > memory_size
                || file_end > bytes.len() as u64
                || alignment > 1
                    && (!alignment.is_power_of_two()
                        || file_offset % alignment != virtual_address % alignment)
            {
                return Err(ConfigProjectionFailureV1::Malformed);
            }
            match segment_type {
                1 => load_segments.push((
                    file_offset,
                    virtual_address,
                    file_size,
                    memory_size,
                    flags,
                )),
                3 => return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion),
                2 => {
                    if dynamic.is_some() {
                        return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion);
                    }
                    if file_size != memory_size {
                        return Err(ConfigProjectionFailureV1::Malformed);
                    }
                    dynamic = Some((file_offset, virtual_address, file_size));
                }
                _ => {}
            }
        }
        if load_segments.is_empty()
            || !load_segments
                .iter()
                .any(|(_, segment_virtual, segment_file_size, _, flags)| {
                    flags & 1 != 0
                        && entry >= *segment_virtual
                        && entry
                            < segment_virtual
                                .checked_add(*segment_file_size)
                                .unwrap_or(*segment_virtual)
                })
        {
            return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion);
        }
        if elf_type == 2 && dynamic.is_none() {
            return Ok(E3ElfExecutionModelV1::StaticExec);
        }
        if elf_type != 3 {
            return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion);
        }
        let (dynamic_offset, dynamic_virtual_address, dynamic_length) =
            dynamic.ok_or(ConfigProjectionFailureV1::UnsupportedRuntimeVersion)?;
        if dynamic_length == 0 || dynamic_length % 16 != 0 {
            return Err(ConfigProjectionFailureV1::Malformed);
        }
        let start =
            usize::try_from(dynamic_offset).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        let end = usize::try_from(
            dynamic_offset
                .checked_add(dynamic_length)
                .ok_or(ConfigProjectionFailureV1::Malformed)?,
        )
        .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        let dynamic_bytes = bytes
            .get(start..end)
            .ok_or(ConfigProjectionFailureV1::Malformed)?;
        let allowed: BTreeSet<u64> = [
            0, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 21, 25, 26, 27, 28, 30, 0x6ffffef5, 0x6ffffffb,
            0x6ffffff9,
        ]
        .into_iter()
        .collect();
        let singleton: BTreeSet<u64> = allowed.iter().copied().filter(|tag| *tag != 0).collect();
        let mut seen = BTreeSet::new();
        let mut terminal_length = None;
        let mut rela = None;
        let mut rela_size = None;
        let mut rela_entry = None;
        let mut rela_count = None;
        let mut dynamic_values = BTreeMap::new();
        for (index, entry) in dynamic_bytes.chunks_exact(16).enumerate() {
            let tag = u64::from_le_bytes(entry[..8].try_into().unwrap());
            let value = u64::from_le_bytes(entry[8..].try_into().unwrap());
            if !allowed.contains(&tag) || singleton.contains(&tag) && !seen.insert(tag) {
                return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion);
            }
            match tag {
                0 => {
                    terminal_length = Some((index + 1) * 16);
                    break;
                }
                7 => rela = Some(value),
                8 => rela_size = Some(value),
                9 => rela_entry = Some(value),
                0x6ffffff9 => rela_count = Some(value),
                11 if value != 24 => {
                    return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion)
                }
                30 if value != 8 => {
                    return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion)
                }
                0x6ffffffb if value != 0x08000001 => {
                    return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion)
                }
                21 if value != 0 => {
                    return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion)
                }
                _ => {}
            }
            dynamic_values.insert(tag, value);
        }
        let terminal_length =
            terminal_length.ok_or(ConfigProjectionFailureV1::UnsupportedRuntimeVersion)?;
        if dynamic_bytes[terminal_length..]
            .iter()
            .any(|byte| *byte != 0)
        {
            return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion);
        }
        let rela = rela.ok_or(ConfigProjectionFailureV1::UnsupportedRuntimeVersion)?;
        let rela_size = rela_size.ok_or(ConfigProjectionFailureV1::UnsupportedRuntimeVersion)?;
        let rela_entry = rela_entry.ok_or(ConfigProjectionFailureV1::UnsupportedRuntimeVersion)?;
        let rela_count = rela_count.ok_or(ConfigProjectionFailureV1::UnsupportedRuntimeVersion)?;
        if rela_entry != 24
            || rela_size == 0
            || rela_size % rela_entry != 0
            || rela_size / rela_entry != rela_count
            || dynamic_values.get(&10) != Some(&1)
            || dynamic_values.get(&11) != Some(&24)
            || !dynamic_values.contains_key(&5)
            || !dynamic_values.contains_key(&6)
            || !(dynamic_values.contains_key(&4) || dynamic_values.contains_key(&0x6ffffef5))
            || dynamic_values.get(&30) != Some(&8)
            || dynamic_values.get(&0x6ffffffb) != Some(&0x08000001)
            || dynamic_values.contains_key(&25) != dynamic_values.contains_key(&27)
            || dynamic_values.contains_key(&26) != dynamic_values.contains_key(&28)
        {
            return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion);
        }
        let file_extent = |virtual_address: u64,
                           byte_length: u64|
         -> Result<std::ops::Range<usize>, ConfigProjectionFailureV1> {
            let virtual_end = virtual_address
                .checked_add(byte_length)
                .ok_or(ConfigProjectionFailureV1::Malformed)?;
            let (file_offset, segment_virtual, _, _, _) = load_segments
                .iter()
                .copied()
                .find(|(_, segment_virtual, segment_length, _, _)| {
                    virtual_address >= *segment_virtual
                        && segment_virtual
                            .checked_add(*segment_length)
                            .is_some_and(|segment_end| virtual_end <= segment_end)
                })
                .ok_or(ConfigProjectionFailureV1::UnsupportedRuntimeVersion)?;
            let start = file_offset
                .checked_add(virtual_address - segment_virtual)
                .ok_or(ConfigProjectionFailureV1::Malformed)?;
            let end = start
                .checked_add(byte_length)
                .ok_or(ConfigProjectionFailureV1::Malformed)?;
            let start = usize::try_from(start).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
            let end = usize::try_from(end).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
            if end > bytes.len() {
                return Err(ConfigProjectionFailureV1::Malformed);
            }
            Ok(start..end)
        };
        let dynamic_extent = file_extent(dynamic_virtual_address, dynamic_length)?;
        if dynamic_extent != (start..end) {
            return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion);
        }
        let string_table = file_extent(dynamic_values[&5], 1)?;
        if bytes.get(string_table) != Some([0].as_slice()) {
            return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion);
        }
        let mut derived_symbol_counts = Vec::new();
        if let Some(address) = dynamic_values.get(&4) {
            let header = file_extent(*address, 8)?;
            let header = bytes
                .get(header)
                .ok_or(ConfigProjectionFailureV1::Malformed)?;
            let buckets = u32::from_le_bytes(header[..4].try_into().unwrap()) as u64;
            let chains = u32::from_le_bytes(header[4..8].try_into().unwrap()) as u64;
            let table_length = 8_u64
                .checked_add(
                    buckets
                        .checked_add(chains)
                        .and_then(|count| count.checked_mul(4))
                        .ok_or(ConfigProjectionFailureV1::Malformed)?,
                )
                .ok_or(ConfigProjectionFailureV1::Malformed)?;
            let table = file_extent(*address, table_length)?;
            if chains != 1
                || bytes
                    .get(table.start + 8..table.end)
                    .ok_or(ConfigProjectionFailureV1::Malformed)?
                    .chunks_exact(4)
                    .any(|word| u32::from_le_bytes(word.try_into().unwrap()) != 0)
            {
                return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion);
            }
            derived_symbol_counts.push(chains);
        }
        if let Some(address) = dynamic_values.get(&0x6ffffef5) {
            let header = file_extent(*address, 16)?;
            let header = bytes
                .get(header)
                .ok_or(ConfigProjectionFailureV1::Malformed)?;
            let buckets = u32::from_le_bytes(header[..4].try_into().unwrap()) as u64;
            let symbol_offset = u32::from_le_bytes(header[4..8].try_into().unwrap()) as u64;
            let bloom_words = u32::from_le_bytes(header[8..12].try_into().unwrap()) as u64;
            let prefix_length = 16_u64
                .checked_add(
                    bloom_words
                        .checked_mul(8)
                        .and_then(|length| {
                            buckets
                                .checked_mul(4)
                                .and_then(|bucket_length| length.checked_add(bucket_length))
                        })
                        .ok_or(ConfigProjectionFailureV1::Malformed)?,
                )
                .ok_or(ConfigProjectionFailureV1::Malformed)?;
            let table = file_extent(*address, prefix_length)?;
            let bloom_start = table.start + 16;
            let bucket_start = bloom_start
                .checked_add(
                    usize::try_from(
                        bloom_words
                            .checked_mul(8)
                            .ok_or(ConfigProjectionFailureV1::Malformed)?,
                    )
                    .map_err(|_| ConfigProjectionFailureV1::Malformed)?,
                )
                .ok_or(ConfigProjectionFailureV1::Malformed)?;
            if buckets == 0
                || bloom_words == 0
                || !bloom_words.is_power_of_two()
                || symbol_offset != 1
                || bytes
                    .get(bloom_start..bucket_start)
                    .ok_or(ConfigProjectionFailureV1::Malformed)?
                    .iter()
                    .any(|byte| *byte != 0)
                || bytes
                    .get(bucket_start..table.end)
                    .ok_or(ConfigProjectionFailureV1::Malformed)?
                    .chunks_exact(4)
                    .any(|word| u32::from_le_bytes(word.try_into().unwrap()) != 0)
            {
                return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion);
            }
            derived_symbol_counts.push(symbol_offset);
        }
        if derived_symbol_counts.is_empty() || derived_symbol_counts.iter().any(|count| *count != 1)
        {
            return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion);
        }
        let symbol_table = file_extent(dynamic_values[&6], 24)?;
        if bytes
            .get(symbol_table)
            .ok_or(ConfigProjectionFailureV1::Malformed)?
            .iter()
            .any(|byte| *byte != 0)
        {
            return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion);
        }
        for tag in [3_u64, 12, 13] {
            if let Some(address) = dynamic_values.get(&tag) {
                file_extent(*address, 1)?;
            }
        }
        for (address_tag, size_tag) in [(25_u64, 27_u64), (26, 28)] {
            if let (Some(address), Some(size)) = (
                dynamic_values.get(&address_tag),
                dynamic_values.get(&size_tag),
            ) {
                if *size == 0 {
                    return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion);
                }
                file_extent(*address, *size)?;
            }
        }
        let relocation_table = file_extent(rela, rela_size)?;
        for relocation in bytes
            .get(relocation_table)
            .ok_or(ConfigProjectionFailureV1::Malformed)?
            .chunks_exact(24)
        {
            let info = u64::from_le_bytes(relocation[8..16].try_into().unwrap());
            if info >> 32 != 0 || info as u32 != 8 {
                return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion);
            }
        }
        Ok(E3ElfExecutionModelV1::StaticPie(
            E3StaticPieRelocationMetadataV1 {
                dynamic_segment_file_offset: dynamic_offset,
                dynamic_segment_byte_length: dynamic_length,
                rela_virtual_address: rela,
                rela_byte_length: rela_size,
                rela_entry_byte_length: rela_entry,
                relative_relocation_count: rela_count,
                ordered_dynamic_entries_sha256: format!(
                    "{:x}",
                    Sha256::digest(&dynamic_bytes[..terminal_length])
                ),
            },
        ))
    }

    pub fn validate_system_config_mount_target_v1(
        path: &Path,
    ) -> Result<RuntimeSupportDirectoryV1, ConfigProjectionFailureV1> {
        use std::ffi::{CStr, CString};
        use std::mem::MaybeUninit;
        use std::os::fd::FromRawFd;

        if path != Path::new("/etc/codex") {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let root = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
            .open("/")
            .map_err(|_| ConfigProjectionFailureV1::MissingPreparation)?;
        let open_directory =
            |parent: &File, name: &str| -> Result<File, ConfigProjectionFailureV1> {
                let name = CString::new(name).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
                // SAFETY: the parent descriptor and C string remain live for the call.
                let fd = unsafe {
                    libc::openat(
                        parent.as_raw_fd(),
                        name.as_ptr(),
                        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                    )
                };
                if fd < 0 {
                    return Err(ConfigProjectionFailureV1::MissingPreparation);
                }
                // SAFETY: successful `openat` returned a uniquely owned descriptor.
                Ok(unsafe { File::from_raw_fd(fd) })
            };
        let etc = open_directory(&root, "etc")?;
        let target = open_directory(&etc, "codex")?;
        let metadata = target
            .metadata()
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        if !metadata.is_dir()
            || metadata.uid() != 0
            || metadata.gid() != 0
            || metadata.mode() & 0o7777 != 0o755
            || metadata.dev()
                != etc
                    .metadata()
                    .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?
                    .dev()
        {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        // `fdopendir` consumes its descriptor, so enumerate through a duplicate and
        // retain `target` for the post-enumeration identity check.
        // SAFETY: duplicating a live descriptor is safe.
        let enumeration_fd = unsafe { libc::dup(target.as_raw_fd()) };
        if enumeration_fd < 0 {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        // SAFETY: ownership of `enumeration_fd` transfers to the returned DIR stream.
        let stream = unsafe { libc::fdopendir(enumeration_fd) };
        if stream.is_null() {
            // SAFETY: `fdopendir` failed and therefore did not consume the descriptor.
            unsafe { libc::close(enumeration_fd) };
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let mut unexpected_entry = false;
        loop {
            // SAFETY: `stream` remains live until the matching `closedir` below.
            let entry = unsafe { libc::readdir(stream) };
            if entry.is_null() {
                break;
            }
            // SAFETY: `d_name` is a NUL-terminated array supplied by `readdir`.
            let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) }.to_bytes();
            if name != b"." && name != b".." {
                unexpected_entry = true;
                break;
            }
        }
        // SAFETY: the DIR stream is live and uniquely owned here.
        let close_result = unsafe { libc::closedir(stream) };
        if close_result != 0 || unexpected_entry {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let after = target
            .metadata()
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        let name = CString::new("codex").unwrap();
        let mut named = MaybeUninit::<libc::stat>::uninit();
        // SAFETY: the parent and C string are live and `named` is writable.
        if unsafe {
            libc::fstatat(
                etc.as_raw_fd(),
                name.as_ptr(),
                named.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } != 0
        {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        // SAFETY: successful `fstatat` initialized the structure.
        let named = unsafe { named.assume_init() };
        if (
            metadata.dev(),
            metadata.ino(),
            metadata.mode(),
            metadata.uid(),
            metadata.gid(),
        ) != (
            after.dev(),
            after.ino(),
            after.mode(),
            after.uid(),
            after.gid(),
        ) || named.st_dev != metadata.dev()
            || named.st_ino != metadata.ino()
            || named.st_mode != metadata.mode()
        {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        Ok(RuntimeSupportDirectoryV1 {
            absolute_path: "/etc/codex".to_string(),
            device_id: metadata.dev(),
            inode: metadata.ino(),
            mode: 0o755,
            owner_uid: 0,
            owner_gid: 0,
            ordered_entry_names: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn e3d_exact_ca_resolver_accepts_only_direct_or_observed_symlink_layouts() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        use std::process::Command;

        const CHILD: &str = "SUBSTRATE_E3D_CA_RESOLVER_TEST_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let status = Command::new("fakeroot")
                .arg("--")
                .arg(std::env::current_exe().unwrap())
                .arg("e3d_exact_ca_resolver_accepts_only_direct_or_observed_symlink_layouts")
                .arg("--nocapture")
                .env(CHILD, "1")
                .status()
                .expect("run exact CA resolver fixture as synthetic root");
            assert!(status.success(), "synthetic-root CA resolver test failed");
            return;
        }

        fn root_descriptor(path: &Path) -> File {
            OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
                .open(path)
                .unwrap()
        }

        fn create_direct(root: &Path, bytes: &[u8]) {
            let certs = root.join("etc/ssl/certs");
            std::fs::create_dir_all(&certs).unwrap();
            std::fs::write(certs.join("ca-certificates.crt"), bytes).unwrap();
            std::fs::set_permissions(
                certs.join("ca-certificates.crt"),
                std::fs::Permissions::from_mode(0o444),
            )
            .unwrap();
        }

        fn create_link(root: &Path, target: &str, endpoint: &[u8]) {
            std::fs::create_dir_all(root.join("etc/ssl/certs")).unwrap();
            std::fs::create_dir_all(root.join("etc/ca-certificates/extracted")).unwrap();
            std::fs::write(
                root.join("etc/ca-certificates/extracted/tls-ca-bundle.pem"),
                endpoint,
            )
            .unwrap();
            std::fs::set_permissions(
                root.join("etc/ca-certificates/extracted/tls-ca-bundle.pem"),
                std::fs::Permissions::from_mode(0o444),
            )
            .unwrap();
            symlink(target, root.join("etc/ssl/certs/ca-certificates.crt")).unwrap();
        }

        let fixtures = tempfile::tempdir().unwrap();

        let direct = fixtures.path().join("direct");
        create_direct(&direct, b"direct-ca\n");
        let direct_file = resolve_exact_installed_ca_bundle_v1(&root_descriptor(&direct)).unwrap();
        assert_eq!(format!("{:x}", Sha256::digest(b"direct-ca\n")), {
            let mut bytes = Vec::new();
            (&direct_file).read_to_end(&mut bytes).unwrap();
            format!("{:x}", Sha256::digest(bytes))
        });

        let linked = fixtures.path().join("linked");
        create_link(
            &linked,
            "../../ca-certificates/extracted/tls-ca-bundle.pem",
            b"linked-ca\n",
        );
        let linked_file = resolve_exact_installed_ca_bundle_v1(&root_descriptor(&linked)).unwrap();
        let linked_metadata = linked_file.metadata().unwrap();
        let endpoint_metadata =
            std::fs::metadata(linked.join("etc/ca-certificates/extracted/tls-ca-bundle.pem"))
                .unwrap();
        assert_eq!(
            (linked_metadata.dev(), linked_metadata.ino()),
            (endpoint_metadata.dev(), endpoint_metadata.ino())
        );
        assert_eq!(
            std::fs::read_link(linked.join("etc/ssl/certs/ca-certificates.crt")).unwrap(),
            Path::new("../../ca-certificates/extracted/tls-ca-bundle.pem")
        );

        let alternate = fixtures.path().join("alternate");
        create_link(
            &alternate,
            "../../ca-certificates/extracted/other.pem",
            b"alternate-ca\n",
        );
        assert!(resolve_exact_installed_ca_bundle_v1(&root_descriptor(&alternate)).is_err());

        let further_link = fixtures.path().join("further-link");
        create_link(
            &further_link,
            "../../ca-certificates/extracted/tls-ca-bundle.pem",
            b"placeholder\n",
        );
        std::fs::rename(
            further_link.join("etc/ca-certificates/extracted/tls-ca-bundle.pem"),
            further_link.join("etc/ca-certificates/extracted/real.pem"),
        )
        .unwrap();
        symlink(
            "real.pem",
            further_link.join("etc/ca-certificates/extracted/tls-ca-bundle.pem"),
        )
        .unwrap();
        assert!(resolve_exact_installed_ca_bundle_v1(&root_descriptor(&further_link)).is_err());

        let writable = fixtures.path().join("writable");
        create_direct(&writable, b"writable-ca\n");
        std::fs::set_permissions(
            writable.join("etc/ssl"),
            std::fs::Permissions::from_mode(0o777),
        )
        .unwrap();
        assert!(resolve_exact_installed_ca_bundle_v1(&root_descriptor(&writable)).is_err());

        let nonregular = fixtures.path().join("nonregular");
        std::fs::create_dir_all(nonregular.join("etc/ssl/certs/ca-certificates.crt")).unwrap();
        assert!(resolve_exact_installed_ca_bundle_v1(&root_descriptor(&nonregular)).is_err());
    }

    #[test]
    fn e3c_held_absolute_directory_rejects_root_substitution() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("source");
        std::fs::create_dir(&root).unwrap();
        let held = open_absolute_directory_v1(&root).unwrap();
        std::fs::rename(&root, temp.path().join("displaced-source")).unwrap();
        std::fs::create_dir(&root).unwrap();
        assert_eq!(
            revalidate_absolute_directory_v1(&held),
            Err(ConfigProjectionFailureV1::PartialPublication)
        );
    }

    #[test]
    fn e3c_held_absolute_directory_rejects_ancestor_substitution() {
        let temp = tempfile::tempdir().unwrap();
        let ancestor = temp.path().join("authority");
        let root = ancestor.join("source");
        std::fs::create_dir_all(&root).unwrap();
        let held = open_absolute_directory_v1(&root).unwrap();
        std::fs::rename(&ancestor, temp.path().join("displaced-authority")).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        assert_eq!(
            revalidate_absolute_directory_v1(&held),
            Err(ConfigProjectionFailureV1::PartialPublication)
        );
    }

    fn minimal_elf64(elf_type: u16, entry: u64, program_type: u32) -> Vec<u8> {
        let mut bytes = vec![0_u8; 64 + 56];
        bytes[..16].copy_from_slice(b"\x7fELF\x02\x01\x01\0\0\0\0\0\0\0\0\0");
        bytes[16..18].copy_from_slice(&elf_type.to_le_bytes());
        bytes[18..20].copy_from_slice(&62_u16.to_le_bytes());
        bytes[20..24].copy_from_slice(&1_u32.to_le_bytes());
        bytes[24..32].copy_from_slice(&entry.to_le_bytes());
        bytes[32..40].copy_from_slice(&64_u64.to_le_bytes());
        bytes[52..54].copy_from_slice(&64_u16.to_le_bytes());
        bytes[54..56].copy_from_slice(&56_u16.to_le_bytes());
        bytes[56..58].copy_from_slice(&1_u16.to_le_bytes());
        bytes[64..68].copy_from_slice(&program_type.to_le_bytes());
        bytes
    }

    #[test]
    fn e3c_static_elf_rejects_non_elf_input() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), b"not an elf").unwrap();
        let file = File::open(temp.path()).unwrap();
        assert_eq!(
            LinuxArtifactSourceV1::validate_e3_static_elf_v1(&file),
            Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion)
        );
    }

    #[test]
    fn e3c_static_exec_rejects_missing_or_malformed_load_segments_and_entrypoint() {
        let cases = [
            minimal_elf64(2, 64, 0),
            {
                let mut bytes = minimal_elf64(2, 64, 1);
                bytes[68..72].copy_from_slice(&5_u32.to_le_bytes());
                bytes[96..104].copy_from_slice(&4096_u64.to_le_bytes());
                bytes[104..112].copy_from_slice(&4096_u64.to_le_bytes());
                bytes[112..120].copy_from_slice(&4096_u64.to_le_bytes());
                bytes
            },
            {
                let mut bytes = minimal_elf64(2, 0x500000, 1);
                bytes[68..72].copy_from_slice(&4_u32.to_le_bytes());
                bytes[80..88].copy_from_slice(&0x400000_u64.to_le_bytes());
                bytes[96..104].copy_from_slice(&(64_u64 + 56).to_le_bytes());
                bytes[104..112].copy_from_slice(&(64_u64 + 56).to_le_bytes());
                bytes[112..120].copy_from_slice(&1_u64.to_le_bytes());
                bytes
            },
        ];

        for bytes in cases {
            let temp = tempfile::NamedTempFile::new().unwrap();
            std::fs::write(temp.path(), bytes).unwrap();
            let file = File::open(temp.path()).unwrap();
            assert!(LinuxArtifactSourceV1::validate_e3_static_elf_v1(&file).is_err());
        }
    }

    #[test]
    fn e3c_pinned_codex_elf_matches_the_static_execution_contract() {
        let Ok(path) = std::env::var("SUBSTRATE_E3C_PINNED_CODEX") else {
            return;
        };
        let file = File::open(path).unwrap();
        assert!(matches!(
            LinuxArtifactSourceV1::validate_e3_static_elf_v1(&file).unwrap(),
            E3ElfExecutionModelV1::StaticExec | E3ElfExecutionModelV1::StaticPie(_)
        ));
    }

    #[test]
    fn e3c_static_pie_rejects_runpath_even_when_the_value_is_empty() {
        let Ok(path) = std::env::var("SUBSTRATE_E3C_PINNED_CODEX") else {
            return;
        };
        let mut bytes = std::fs::read(path).unwrap();
        let original = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(original.path(), &bytes).unwrap();
        let original_file = File::open(original.path()).unwrap();
        let E3ElfExecutionModelV1::StaticPie(metadata) =
            LinuxArtifactSourceV1::validate_e3_static_elf_v1(&original_file).unwrap()
        else {
            panic!("pinned Codex must use the admitted static-PIE form")
        };
        let debug_entry = [21_u64.to_le_bytes(), 0_u64.to_le_bytes()].concat();
        let dynamic_start = usize::try_from(metadata.dynamic_segment_file_offset).unwrap();
        let dynamic_end =
            dynamic_start + usize::try_from(metadata.dynamic_segment_byte_length).unwrap();
        let offset = bytes[dynamic_start..dynamic_end]
            .windows(debug_entry.len())
            .position(|window| window == debug_entry)
            .map(|offset| offset + dynamic_start)
            .expect("pinned Codex dynamic table contains DT_DEBUG=0");
        bytes[offset..offset + 8].copy_from_slice(&29_u64.to_le_bytes());
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), bytes).unwrap();
        let file = File::open(temp.path()).unwrap();
        assert_eq!(
            LinuxArtifactSourceV1::validate_e3_static_elf_v1(&file),
            Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion)
        );
    }

    #[test]
    fn e3c_static_pie_rejects_a_hash_table_claiming_an_extra_dynamic_symbol() {
        let Ok(path) = std::env::var("SUBSTRATE_E3C_PINNED_CODEX") else {
            return;
        };
        let mut bytes = std::fs::read(path).unwrap();
        assert_eq!(&bytes[0x238..0x23c], &0_u32.to_le_bytes());
        assert_eq!(&bytes[0x23c..0x240], &1_u32.to_le_bytes());
        bytes[0x23c..0x240].copy_from_slice(&2_u32.to_le_bytes());
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), bytes).unwrap();
        let file = File::open(temp.path()).unwrap();
        assert_eq!(
            LinuxArtifactSourceV1::validate_e3_static_elf_v1(&file),
            Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion)
        );
    }
}
