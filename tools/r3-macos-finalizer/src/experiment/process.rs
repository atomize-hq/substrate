//! Exact, read-only process and executable measurement for the disposable coordinator.
//!
//! This module deliberately makes no Security.framework, Keychain, or launchd calls. Code
//! requirement and CDHash enforcement remain independent checks performed by the root finalizer;
//! the UID-501 harness binds the child PID/start identity and the frozen path/bytes/physical file
//! identity before asking the publisher to sign authority naming that process.

use std::ffi::{c_char, c_int, c_void, CString, OsString};
use std::fs::File;
use std::io::Read;
use std::mem::{size_of, MaybeUninit};
use std::os::fd::FromRawFd;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, document_sha256_v2, sha256_hex_v2, ExecutableIdentityV2, ProcessIdentityV2,
    MAC_R3_COORDINATOR_EFFECTIVE_GID_V2, MAC_R3_COORDINATOR_PATH_V2,
    MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2,
};

use super::{
    DISPOSABLE_HARNESS_ACCOUNT_V2, DISPOSABLE_HARNESS_UID_V2, EXPERIMENT_ID_V2,
    EXPERIMENT_OWNER_V2, EXPERIMENT_VERSION_V2,
};

const PROC_PIDTBSDINFO: c_int = 3;
const PROC_PIDPATHINFO_MAXSIZE: usize = 4 * 1024;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SupplementaryGroupEvidenceV2 {
    CurrentProcessGetgroups,
    SealedPreExecPostDropGetgroups,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SupplementaryGroupAttestationV2 {
    pub groups: Vec<u32>,
    pub evidence: SupplementaryGroupEvidenceV2,
}

impl SupplementaryGroupAttestationV2 {
    pub fn current_process() -> Result<Self> {
        let raw_groups = current_supplementary_groups()?;
        // SAFETY: getegid reads the current process credential without dereferencing pointers.
        let effective_gid = unsafe { libc::getegid() };
        Self::from_current_process_groups(raw_groups, effective_gid)
    }

    fn from_current_process_groups(raw_groups: Vec<u32>, effective_gid: u32) -> Result<Self> {
        let groups = match raw_groups.as_slice() {
            [] => Vec::new(),
            [group] if *group == effective_gid => Vec::new(),
            _ => bail!("process retained supplementary groups"),
        };
        let value = Self {
            groups,
            evidence: SupplementaryGroupEvidenceV2::CurrentProcessGetgroups,
        };
        value.validate_empty()?;
        Ok(value)
    }

    pub fn sealed_pre_exec(groups: Vec<u32>) -> Result<Self> {
        let value = Self {
            groups,
            evidence: SupplementaryGroupEvidenceV2::SealedPreExecPostDropGetgroups,
        };
        value.validate_empty()?;
        Ok(value)
    }

    pub fn validate_empty(&self) -> Result<()> {
        if !self.groups.is_empty() {
            bail!("process retained supplementary groups")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CoordinatorProcessAttestationV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub pid: i32,
    pub effective_uid: u32,
    pub effective_gid: u32,
    pub supplementary_groups: SupplementaryGroupAttestationV2,
    pub canonical_account: String,
    pub process_start_identity_sha256: String,
    pub executable_path: String,
    pub executable_sha256: String,
    pub executable_size: u64,
    pub executable_physical_identity_sha256: String,
    pub executable_identity_sha256: String,
}

impl CoordinatorProcessAttestationV2 {
    pub fn process_identity(&self) -> ProcessIdentityV2 {
        ProcessIdentityV2 {
            effective_uid: self.effective_uid,
            effective_gid: self.effective_gid,
            canonical_account: self.canonical_account.clone(),
            pidversion_required: true,
            process_start_identity_sha256: self.process_start_identity_sha256.clone(),
            executable_identity_sha256: self.executable_identity_sha256.clone(),
        }
    }

    pub fn validate(&self, expected: &ExecutableIdentityV2) -> Result<()> {
        substrate_common::macos_retirement_v2::validate_executable_identity_v2(
            expected,
            MAC_R3_COORDINATOR_PATH_V2,
            MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2,
        )?;
        self.validate_fixed(
            expected,
            DISPOSABLE_HARNESS_UID_V2,
            MAC_R3_COORDINATOR_EFFECTIVE_GID_V2,
            DISPOSABLE_HARNESS_ACCOUNT_V2,
        )
    }

    pub fn validate_fixed(
        &self,
        expected: &ExecutableIdentityV2,
        expected_uid: u32,
        expected_gid: u32,
        expected_account: &str,
    ) -> Result<()> {
        self.supplementary_groups.validate_empty()?;
        if self.schema_owner != EXPERIMENT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
            || self.pid <= 1
            || self.effective_uid != expected_uid
            || self.effective_gid != expected_gid
            || self.canonical_account != expected_account
            || self.executable_path != expected.intended_path
            || self.executable_sha256 != expected.executable_sha256
            || self.executable_size != expected.executable_size
            || self.executable_physical_identity_sha256 != expected.physical_identity_sha256
            || self.executable_identity_sha256 != document_sha256_v2(expected)?
        {
            bail!("coordinator child process does not exact-match the frozen executable identity")
        }
        require_digest(
            &self.process_start_identity_sha256,
            "coordinator process start identity",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProcessStartIdentity {
    seconds: u64,
    microseconds: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ExecutableFileIdentity {
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

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct ProcessStartJoin {
    pid: i32,
    seconds: u64,
    microseconds: u64,
}

pub fn attest_coordinator_process_v2(
    pid: i32,
    expected: &ExecutableIdentityV2,
) -> Result<CoordinatorProcessAttestationV2> {
    substrate_common::macos_retirement_v2::validate_executable_identity_v2(
        expected,
        MAC_R3_COORDINATOR_PATH_V2,
        MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2,
    )?;
    attest_fixed_peer_process_v2(
        pid,
        expected,
        DISPOSABLE_HARNESS_UID_V2,
        MAC_R3_COORDINATOR_EFFECTIVE_GID_V2,
        DISPOSABLE_HARNESS_ACCOUNT_V2,
    )
}

pub fn attest_fixed_peer_process_v2(
    pid: i32,
    expected: &ExecutableIdentityV2,
    expected_uid: u32,
    expected_gid: u32,
    expected_account: &str,
) -> Result<CoordinatorProcessAttestationV2> {
    if expected.intended_path.is_empty()
        || !Path::new(&expected.intended_path).is_absolute()
        || expected.executable_size == 0
    {
        bail!("fixed peer executable identity is incomplete")
    }
    if pid <= 1 {
        bail!("coordinator child PID is invalid")
    }
    let self_pid = i32::try_from(std::process::id()).context("current PID exceeds i32")?;
    if pid != self_pid {
        bail!("non-self peer attestation requires the child's self-measured getgroups receipt")
    }
    let supplementary_groups = SupplementaryGroupAttestationV2::current_process()?;
    let first = process_info(pid)?;
    if first.effective_uid != expected_uid || first.effective_gid != expected_gid {
        bail!("coordinator child effective UID/GID changed")
    }
    let start = first.start.clone();
    let executable_path = pid_process_path(pid)?;
    if executable_path != Path::new(&expected.intended_path) {
        bail!("coordinator child executable path is not the frozen physical path")
    }
    let (file_identity, executable_sha256) = hash_immutable_executable(&executable_path)?;
    let second = process_info(pid)?;
    if first != second || pid_process_path(pid)? != executable_path {
        bail!("coordinator PID/start/path changed while the harness measured it")
    }
    let attestation = CoordinatorProcessAttestationV2 {
        schema_owner: EXPERIMENT_OWNER_V2.to_string(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_string(),
        pid,
        effective_uid: first.effective_uid,
        effective_gid: first.effective_gid,
        supplementary_groups,
        canonical_account: expected_account.to_string(),
        process_start_identity_sha256: process_start_sha256_v2(pid, &start)?,
        executable_path: executable_path
            .to_str()
            .context("coordinator executable path is not UTF-8")?
            .to_string(),
        executable_sha256,
        executable_size: file_identity.size,
        executable_physical_identity_sha256: physical_identity_sha256_v2(&file_identity)?,
        executable_identity_sha256: document_sha256_v2(expected)?,
    };
    attestation.validate_fixed(expected, expected_uid, expected_gid, expected_account)?;
    Ok(attestation)
}

fn current_supplementary_groups() -> Result<Vec<u32>> {
    // SAFETY: a null buffer with count zero is the documented size query.
    let count = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
    if count < 0 {
        return Err(std::io::Error::last_os_error())
            .context("measure coordinator/harness supplementary-group count");
    }
    let mut groups = vec![0; usize::try_from(count)?];
    if count > 0 {
        // SAFETY: the buffer contains exactly `count` gid_t slots.
        let returned = unsafe { libc::getgroups(count, groups.as_mut_ptr()) };
        if returned != count {
            if returned < 0 {
                return Err(std::io::Error::last_os_error())
                    .context("measure coordinator/harness supplementary groups");
            }
            bail!("coordinator/harness supplementary groups changed while measured")
        }
    }
    Ok(groups)
}

fn process_start_sha256_v2(pid: i32, start: &ProcessStartIdentity) -> Result<String> {
    Ok(sha256_hex_v2(&canonical_bytes_v2(&ProcessStartJoin {
        pid,
        seconds: start.seconds,
        microseconds: start.microseconds,
    })?))
}

fn physical_identity_sha256_v2(identity: &ExecutableFileIdentity) -> Result<String> {
    Ok(sha256_hex_v2(&canonical_bytes_v2(identity)?))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProcessInfo {
    effective_uid: u32,
    effective_gid: u32,
    start: ProcessStartIdentity,
}

#[repr(C)]
struct ProcBsdInfo {
    pbi_flags: u32,
    pbi_status: u32,
    pbi_xstatus: u32,
    pbi_pid: u32,
    pbi_ppid: u32,
    pbi_uid: libc::uid_t,
    pbi_gid: libc::gid_t,
    pbi_ruid: libc::uid_t,
    pbi_rgid: libc::gid_t,
    pbi_svuid: libc::uid_t,
    pbi_svgid: libc::gid_t,
    rfu_1: u32,
    pbi_comm: [c_char; 16],
    pbi_name: [c_char; 32],
    pbi_nfiles: u32,
    pbi_pgid: u32,
    pbi_pjobc: u32,
    e_tdev: u32,
    e_tpgid: u32,
    pbi_nice: i32,
    pbi_start_tvsec: u64,
    pbi_start_tvusec: u64,
}

fn process_info(pid: i32) -> Result<ProcessInfo> {
    let mut info = MaybeUninit::<ProcBsdInfo>::zeroed();
    // SAFETY: the writable output matches the installed SDK PROC_PIDTBSDINFO layout.
    let returned = unsafe {
        proc_pidinfo(
            pid,
            PROC_PIDTBSDINFO,
            0,
            info.as_mut_ptr().cast(),
            size_of::<ProcBsdInfo>() as c_int,
        )
    };
    if returned < 0 {
        return Err(std::io::Error::last_os_error()).context("read coordinator child process info");
    }
    if returned as usize != size_of::<ProcBsdInfo>() {
        bail!("PROC_PIDTBSDINFO returned a truncated coordinator identity")
    }
    // SAFETY: proc_pidinfo initialized the complete structure.
    let info = unsafe { info.assume_init() };
    if info.pbi_pid != pid as u32 || info.pbi_start_tvusec >= 1_000_000 {
        bail!("coordinator child process table identity changed")
    }
    Ok(ProcessInfo {
        effective_uid: info.pbi_uid,
        effective_gid: info.pbi_gid,
        start: ProcessStartIdentity {
            seconds: info.pbi_start_tvsec,
            microseconds: info.pbi_start_tvusec,
        },
    })
}

fn pid_process_path(pid: i32) -> Result<PathBuf> {
    let mut bytes = vec![0_u8; PROC_PIDPATHINFO_MAXSIZE];
    // SAFETY: the output buffer is writable for the supplied exact size.
    let length = unsafe { proc_pidpath(pid, bytes.as_mut_ptr().cast(), bytes.len() as u32) };
    if length <= 0 || length as usize >= bytes.len() {
        return Err(std::io::Error::last_os_error()).context("resolve coordinator child PID path");
    }
    bytes.truncate(length as usize);
    if bytes.contains(&0) {
        bail!("coordinator child PID path contains an embedded NUL")
    }
    let path = PathBuf::from(OsString::from_vec(bytes));
    if !path.is_absolute() {
        bail!("coordinator child PID path is not absolute")
    }
    Ok(path)
}

fn hash_immutable_executable(path: &Path) -> Result<(ExecutableFileIdentity, String)> {
    require_root_owned_immutable_path(path)?;
    let encoded = CString::new(path.as_os_str().as_bytes())?;
    // SAFETY: the path is exact NUL-terminated input and O_NOFOLLOW protects the final component.
    let fd = unsafe {
        libc::open(
            encoded.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error()).context("open coordinator executable");
    }
    // SAFETY: successful open transferred ownership of the descriptor.
    let mut file = unsafe { File::from_raw_fd(fd) };
    let before = file_identity(&file.metadata()?)?;
    if before.owner_uid != 0
        || before.mode & 0o022 != 0
        || before.link_count != 1
        || before.mode & u32::from(libc::S_IFMT) != u32::from(libc::S_IFREG)
    {
        bail!("coordinator executable is not one root-owned immutable regular file")
    }
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    let after = file_identity(&file.metadata()?)?;
    if before != after {
        bail!("coordinator executable changed while hashing")
    }
    require_root_owned_immutable_path(path)?;
    Ok((before, format!("{:x}", digest.finalize())))
}

fn file_identity(metadata: &std::fs::Metadata) -> Result<ExecutableFileIdentity> {
    Ok(ExecutableFileIdentity {
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

fn require_root_owned_immutable_path(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        bail!("coordinator executable path is not absolute")
    }
    let mut current = PathBuf::from("/");
    for component in path.components().skip(1) {
        current.push(component.as_os_str());
        let metadata = std::fs::symlink_metadata(&current)
            .with_context(|| format!("inspect coordinator path component {}", current.display()))?;
        if metadata.file_type().is_symlink() || metadata.uid() != 0 || metadata.mode() & 0o022 != 0
        {
            bail!("coordinator path is not root-owned immutable no-follow state")
        }
        if current == path {
            if !metadata.file_type().is_file() || metadata.nlink() != 1 {
                bail!("coordinator executable is not one regular-file identity")
            }
        } else if !metadata.is_dir() {
            bail!("coordinator executable parent is not a directory")
        }
    }
    Ok(())
}

fn require_digest(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is not an exact lowercase SHA-256 digest")
    }
    Ok(())
}

unsafe extern "C" {
    fn proc_pidinfo(
        pid: c_int,
        flavor: c_int,
        arg: u64,
        buffer: *mut c_void,
        buffer_size: c_int,
    ) -> c_int;
    fn proc_pidpath(pid: c_int, buffer: *mut c_void, buffer_size: u32) -> c_int;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_process_groups_normalize_only_zero_or_singleton_effective_gid() {
        let expected = SupplementaryGroupAttestationV2 {
            groups: Vec::new(),
            evidence: SupplementaryGroupEvidenceV2::CurrentProcessGetgroups,
        };
        assert_eq!(
            SupplementaryGroupAttestationV2::from_current_process_groups(Vec::new(), 20).unwrap(),
            expected
        );
        assert_eq!(
            SupplementaryGroupAttestationV2::from_current_process_groups(vec![20], 20).unwrap(),
            expected
        );
        assert!(
            SupplementaryGroupAttestationV2::from_current_process_groups(vec![21], 20).is_err()
        );
        assert!(
            SupplementaryGroupAttestationV2::from_current_process_groups(vec![20, 20], 20).is_err()
        );
        assert!(
            SupplementaryGroupAttestationV2::from_current_process_groups(vec![20, 21], 20).is_err()
        );
        assert!(
            SupplementaryGroupAttestationV2::from_current_process_groups(vec![21, 22], 20).is_err()
        );
    }

    #[test]
    fn process_and_physical_join_domains_are_exact_and_pid_bound() {
        let start = ProcessStartIdentity {
            seconds: 123,
            microseconds: 456,
        };
        assert_ne!(
            process_start_sha256_v2(100, &start).unwrap(),
            process_start_sha256_v2(101, &start).unwrap()
        );
        let file = ExecutableFileIdentity {
            device: 1,
            inode: 2,
            owner_uid: 0,
            owner_gid: 0,
            mode: 0o100555,
            link_count: 1,
            size: 3,
            modified_seconds: 4,
            modified_nanoseconds: 5,
        };
        assert_eq!(physical_identity_sha256_v2(&file).unwrap().len(), 64);
    }
}
