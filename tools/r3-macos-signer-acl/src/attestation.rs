use anyhow::{bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::ffi::{c_char, c_int, c_void, CStr, CString, OsString};
use std::fs::File;
use std::io::Read;
use std::mem::{size_of, MaybeUninit};
use std::os::fd::FromRawFd;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::ptr;

use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, sha256_hex_v2, validate_executable_identity_v2, ExecutableIdentityV2,
    MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2, MAC_R3_DISPOSABLE_PUBLISHER_SIGNING_IDENTIFIER_V2,
};
use substrate_r3_macos_finalizer::experiment::publisher_protocol::{
    PublisherPreparedInputV2, PublisherProcessAttestationV2, PUBLISHER_PROTOCOL_OWNER_V2,
};
use substrate_r3_macos_finalizer::experiment::process::SupplementaryGroupAttestationV2;
use substrate_r3_macos_finalizer::experiment::{
    AD_HOC_HARDENED_RUNTIME_FLAGS_V2, EXPERIMENT_ID_V2, EXPERIMENT_VERSION_V2,
};

use crate::NonInteractiveSecurity;

type CfType = *const c_void;
type SecCode = *const c_void;
type SecRequirement = *const c_void;
type OsStatus = i32;

const ERR_SEC_SUCCESS: OsStatus = 0;
const K_SEC_CS_SIGNING_AND_REQUIREMENT_INFORMATION: u32 = (1 << 1) | (1 << 2);
const K_CF_NUMBER_SINT64_TYPE: i32 = 4;
const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
const PROC_PIDTBSDINFO: c_int = 3;
const PROC_PIDPATHINFO_MAXSIZE: usize = 4 * 1024;
const MAX_PASSWD_BUFFER_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProcessStartIdentity {
    seconds: u64,
    microseconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProcessMeasurement {
    start: ProcessStartIdentity,
    executable_path: PathBuf,
    executable_file: ExecutableFileIdentity,
    executable_sha256: String,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct ProcessStartJoin {
    pid: i32,
    seconds: u64,
    microseconds: u64,
}

/// Attest the live publisher only after the unforgeable no-interaction token exists. The token is
/// constructed solely by the process's successful first Security.framework call.
pub(crate) fn attest_publisher_process(
    _security: &NonInteractiveSecurity,
    prepared: &PublisherPreparedInputV2,
) -> Result<PublisherProcessAttestationV2> {
    prepared.validate()?;
    let expected: &ExecutableIdentityV2 = &prepared.publisher_identity;
    validate_executable_identity_v2(
        expected,
        MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2,
        MAC_R3_DISPOSABLE_PUBLISHER_SIGNING_IDENTIFIER_V2,
    )?;
    let expected_cdhash = decode_exact_hex(&expected.cdhash, 20)?;

    // SAFETY: scalar queries of this live process.
    let effective_uid = unsafe { libc::geteuid() };
    // SAFETY: scalar queries of this live process.
    let effective_gid = unsafe { libc::getegid() };
    let supplementary_groups = SupplementaryGroupAttestationV2::current_process()?;
    // SAFETY: scalar queries of this live process.
    let pid = unsafe { libc::getpid() };
    if effective_uid != 0 || canonical_account_for_uid(effective_uid)? != "root" || pid <= 1 {
        bail!("running disposable publisher is not the canonical root process")
    }

    let path = Path::new(MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2);
    let first = measure_self_process(pid, effective_uid, effective_gid, path)?;
    let physical_identity_sha256 = sha256_hex_v2(&canonical_bytes_v2(&first.executable_file)?);
    if first.executable_sha256 != expected.executable_sha256
        || first.executable_file.size != expected.executable_size
        || physical_identity_sha256 != expected.physical_identity_sha256
    {
        bail!("running disposable publisher image differs from the prepared identity")
    }
    let observed_cdhash = verify_self_code_requirement_and_cdhash(
        &expected.designated_requirement,
        &expected_cdhash,
    )?;
    let second = measure_self_process(pid, effective_uid, effective_gid, path)?;
    if first != second {
        bail!("publisher PID/start/path/image changed during self-attestation")
    }

    let attestation = PublisherProcessAttestationV2 {
        schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        effective_uid,
        supplementary_groups,
        canonical_account: "root".to_owned(),
        pid,
        process_start_identity_sha256: sha256_hex_v2(&canonical_bytes_v2(&ProcessStartJoin {
            pid,
            seconds: first.start.seconds,
            microseconds: first.start.microseconds,
        })?),
        executable_path: first
            .executable_path
            .to_str()
            .context("publisher executable path is not UTF-8")?
            .to_owned(),
        executable_physical_identity_sha256: physical_identity_sha256,
        executable_sha256: first.executable_sha256,
        designated_requirement: expected.designated_requirement.clone(),
        cdhash: hex_lower(&observed_cdhash),
        interaction_denial_established_first: true,
    };
    attestation.validate(prepared)?;
    Ok(attestation)
}

fn measure_self_process(
    pid: libc::pid_t,
    effective_uid: libc::uid_t,
    effective_gid: libc::gid_t,
    expected_path: &Path,
) -> Result<ProcessMeasurement> {
    let start = process_start_identity(pid, effective_uid, effective_gid)?;
    let executable_path = pid_process_path(pid)?;
    if executable_path.as_os_str().as_bytes() != expected_path.as_os_str().as_bytes() {
        bail!("running publisher executable path differs from its fixed path")
    }
    let (executable_file, executable_sha256) = hash_immutable_executable(expected_path)?;
    if process_start_identity(pid, effective_uid, effective_gid)? != start
        || pid_process_path(pid)? != executable_path
    {
        bail!("publisher PID/start/path changed while measuring its image")
    }
    Ok(ProcessMeasurement {
        start,
        executable_path,
        executable_file,
        executable_sha256,
    })
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

fn process_start_identity(
    pid: libc::pid_t,
    effective_uid: libc::uid_t,
    effective_gid: libc::gid_t,
) -> Result<ProcessStartIdentity> {
    let mut info = MaybeUninit::<ProcBsdInfo>::zeroed();
    // SAFETY: output is writable and matches the installed SDK PROC_PIDTBSDINFO layout.
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
        return Err(std::io::Error::last_os_error()).context("read publisher process start");
    }
    if returned as usize != size_of::<ProcBsdInfo>() {
        bail!("PROC_PIDTBSDINFO returned a truncated publisher identity")
    }
    // SAFETY: the full structure was initialized.
    let info = unsafe { info.assume_init() };
    if info.pbi_pid != pid as u32
        || info.pbi_uid != effective_uid
        || info.pbi_gid != effective_gid
        || info.pbi_start_tvusec >= 1_000_000
    {
        bail!("publisher process table identity changed")
    }
    Ok(ProcessStartIdentity {
        seconds: info.pbi_start_tvsec,
        microseconds: info.pbi_start_tvusec,
    })
}

fn pid_process_path(pid: libc::pid_t) -> Result<PathBuf> {
    let mut bytes = vec![0_u8; PROC_PIDPATHINFO_MAXSIZE];
    // SAFETY: the buffer is writable for its full exact length.
    let length = unsafe { proc_pidpath(pid, bytes.as_mut_ptr().cast(), bytes.len() as u32) };
    if length <= 0 || length as usize >= bytes.len() {
        return Err(std::io::Error::last_os_error()).context("resolve publisher PID path");
    }
    bytes.truncate(length as usize);
    if bytes.contains(&0) {
        bail!("publisher PID path contains an embedded NUL")
    }
    let path = PathBuf::from(OsString::from_vec(bytes));
    if !path.is_absolute() {
        bail!("publisher PID path is not absolute")
    }
    Ok(path)
}

fn hash_immutable_executable(path: &Path) -> Result<(ExecutableFileIdentity, String)> {
    require_root_owned_immutable_path(path)?;
    let encoded = CString::new(path.as_os_str().as_bytes())?;
    // SAFETY: exact NUL-terminated path and O_NOFOLLOW terminal protection.
    let fd = unsafe {
        libc::open(
            encoded.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error()).context("open publisher executable");
    }
    // SAFETY: open transferred ownership of this descriptor.
    let mut file = unsafe { File::from_raw_fd(fd) };
    let before = file_identity(&file.metadata()?);
    if before.owner_uid != 0
        || before.mode & 0o022 != 0
        || before.link_count != 1
        || before.mode & u32::from(libc::S_IFMT) != u32::from(libc::S_IFREG)
    {
        bail!("publisher executable is not one root-owned immutable regular file")
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
    let after = file_identity(&file.metadata()?);
    if before != after {
        bail!("publisher executable changed while hashing")
    }
    require_root_owned_immutable_path(path)?;
    Ok((before, hex_lower(&digest.finalize())))
}

fn require_root_owned_immutable_path(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        bail!("publisher executable path is not absolute")
    }
    let mut current = PathBuf::from("/");
    for component in path.components().skip(1) {
        current.push(component.as_os_str());
        let metadata = std::fs::symlink_metadata(&current)
            .with_context(|| format!("inspect publisher path component {}", current.display()))?;
        if metadata.file_type().is_symlink() || metadata.uid() != 0 || metadata.mode() & 0o022 != 0
        {
            bail!("publisher path is not root-owned immutable no-follow state")
        }
        if current == path {
            if !metadata.file_type().is_file() || metadata.nlink() != 1 {
                bail!("publisher executable is not one regular-file identity")
            }
        } else if !metadata.is_dir() {
            bail!("publisher executable parent is not a directory")
        }
    }
    Ok(())
}

fn file_identity(metadata: &std::fs::Metadata) -> ExecutableFileIdentity {
    ExecutableFileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        owner_uid: metadata.uid(),
        owner_gid: metadata.gid(),
        mode: metadata.mode(),
        link_count: metadata.nlink(),
        size: metadata.size(),
        modified_seconds: metadata.mtime(),
        modified_nanoseconds: metadata.mtime_nsec(),
    }
}

fn verify_self_code_requirement_and_cdhash(
    requirement_text: &str,
    expected_cdhash: &[u8],
) -> Result<Vec<u8>> {
    if requirement_text.is_empty() || requirement_text.contains('\0') {
        bail!("publisher designated requirement is invalid")
    }
    // SAFETY: every create/copy-rule object is retained in `owned` through its last use.
    unsafe {
        let mut owned = OwnedCf::default();
        let mut code: SecCode = ptr::null();
        let status = SecCodeCopySelf(0, &mut code);
        if status != ERR_SEC_SUCCESS || code.is_null() {
            bail!("resolve publisher SecCode failed with OSStatus {status}")
        }
        owned.hold(code);
        let requirement_string = cf_string(&mut owned, requirement_text)?;
        let mut requirement: SecRequirement = ptr::null();
        let status = SecRequirementCreateWithString(requirement_string, 0, &mut requirement);
        if status != ERR_SEC_SUCCESS || requirement.is_null() {
            bail!("compile publisher requirement failed with OSStatus {status}")
        }
        owned.hold(requirement);
        let status = SecCodeCheckValidity(code, 0, requirement);
        if status != ERR_SEC_SUCCESS {
            bail!("publisher requirement rejected live code with OSStatus {status}")
        }
        let mut information: CfType = ptr::null();
        let status = SecCodeCopySigningInformation(
            code,
            K_SEC_CS_SIGNING_AND_REQUIREMENT_INFORMATION,
            &mut information,
        );
        if status != ERR_SEC_SUCCESS || information.is_null() {
            bail!("copy publisher signing information failed with OSStatus {status}")
        }
        owned.hold(information);
        if CFGetTypeID(information) != CFDictionaryGetTypeID() {
            bail!("publisher signing information is not a dictionary")
        }
        let unique = CFDictionaryGetValue(information, kSecCodeInfoUnique);
        if unique.is_null() || CFGetTypeID(unique) != CFDataGetTypeID() {
            bail!("publisher signing information has no CDHash")
        }
        let length = CFDataGetLength(unique);
        let bytes = CFDataGetBytePtr(unique);
        if length < 0 || bytes.is_null() {
            bail!("publisher CDHash data is invalid")
        }
        let observed = std::slice::from_raw_parts(bytes, length as usize).to_vec();
        if observed != expected_cdhash {
            bail!("publisher CDHash differs from the prepared identity")
        }
        let flags = CFDictionaryGetValue(information, kSecCodeInfoFlags);
        if flags.is_null() || CFGetTypeID(flags) != CFNumberGetTypeID() {
            bail!("publisher signing information has no CodeDirectory flags")
        }
        let mut signed_flags = 0_i64;
        if CFNumberGetValue(
            flags,
            K_CF_NUMBER_SINT64_TYPE,
            (&mut signed_flags as *mut i64).cast(),
        ) == 0
            || signed_flags != i64::from(AD_HOC_HARDENED_RUNTIME_FLAGS_V2)
        {
            bail!("publisher is not exact ad-hoc Hardened Runtime plus library validation code")
        }
        if !CFDictionaryGetValue(information, kSecCodeInfoTeamIdentifier).is_null()
            || !CFDictionaryGetValue(information, kSecCodeInfoEntitlements).is_null()
            || !CFDictionaryGetValue(information, kSecCodeInfoEntitlementsDict).is_null()
        {
            bail!("publisher carries TeamID or entitlement authority")
        }
        Ok(observed)
    }
}

fn canonical_account_for_uid(uid: libc::uid_t) -> Result<String> {
    // SAFETY: scalar system configuration query.
    let suggested = unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) };
    let mut capacity = if suggested > 0 {
        suggested as usize
    } else {
        16 * 1024
    }
    .min(MAX_PASSWD_BUFFER_BYTES);
    loop {
        let mut record = MaybeUninit::<libc::passwd>::uninit();
        let mut result = ptr::null_mut();
        let mut buffer = vec![0_u8; capacity];
        // SAFETY: all output storage is live for the exact buffer length.
        let status = unsafe {
            libc::getpwuid_r(
                uid,
                record.as_mut_ptr(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                &mut result,
            )
        };
        if status == libc::ERANGE && capacity < MAX_PASSWD_BUFFER_BYTES {
            capacity = (capacity * 2).min(MAX_PASSWD_BUFFER_BYTES);
            continue;
        }
        if status != 0 || result.is_null() {
            bail!("resolve canonical publisher account failed with errno {status}")
        }
        // SAFETY: successful getpwuid_r initialized the record into the live buffer.
        let record = unsafe { record.assume_init() };
        if record.pw_uid != uid || record.pw_name.is_null() {
            bail!("canonical publisher account record is inconsistent")
        }
        // SAFETY: pw_name is NUL-terminated within the live buffer.
        return Ok(unsafe { CStr::from_ptr(record.pw_name) }
            .to_str()
            .context("canonical publisher account is not UTF-8")?
            .to_owned());
    }
}

fn decode_exact_hex(value: &str, expected_bytes: usize) -> Result<Vec<u8>> {
    if value.len() != expected_bytes * 2
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("publisher identity contains invalid lowercase hexadecimal")
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| Ok((hex_nibble(pair[0])? << 4) | hex_nibble(pair[1])?))
        .collect()
}

fn hex_nibble(value: u8) -> Result<u8> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => bail!("invalid lowercase hexadecimal nibble"),
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(char::from(HEX[(byte >> 4) as usize]));
        result.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    result
}

#[derive(Default)]
struct OwnedCf(Vec<CfType>);

impl OwnedCf {
    fn hold<T>(&mut self, value: *const T) {
        if !value.is_null() {
            self.0.push(value.cast());
        }
    }
}

impl Drop for OwnedCf {
    fn drop(&mut self) {
        // SAFETY: each entry follows one CoreFoundation/Security create or copy rule.
        unsafe {
            for value in self.0.drain(..).rev() {
                CFRelease(value);
            }
        }
    }
}

unsafe fn cf_string(owned: &mut OwnedCf, value: &str) -> Result<CfType> {
    // SAFETY: bytes are live for the call and the result follows the create rule.
    let string = unsafe {
        CFStringCreateWithBytes(
            kCFAllocatorDefault,
            value.as_bytes().as_ptr(),
            value.len() as isize,
            K_CF_STRING_ENCODING_UTF8,
            0,
        )
    };
    if string.is_null() {
        bail!("allocate publisher requirement string")
    }
    owned.hold(string);
    Ok(string)
}

#[link(name = "proc")]
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

#[link(name = "Security", kind = "framework")]
unsafe extern "C" {
    static kSecCodeInfoUnique: CfType;
    static kSecCodeInfoFlags: CfType;
    static kSecCodeInfoTeamIdentifier: CfType;
    static kSecCodeInfoEntitlements: CfType;
    static kSecCodeInfoEntitlementsDict: CfType;
    fn SecCodeCopySelf(flags: u32, code: *mut SecCode) -> OsStatus;
    fn SecRequirementCreateWithString(
        requirement_text: CfType,
        flags: u32,
        requirement: *mut SecRequirement,
    ) -> OsStatus;
    fn SecCodeCheckValidity(code: SecCode, flags: u32, requirement: SecRequirement) -> OsStatus;
    fn SecCodeCopySigningInformation(
        code: SecCode,
        flags: u32,
        information: *mut CfType,
    ) -> OsStatus;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFAllocatorDefault: CfType;
    fn CFRelease(value: CfType);
    fn CFGetTypeID(value: CfType) -> usize;
    fn CFStringCreateWithBytes(
        allocator: CfType,
        bytes: *const u8,
        byte_count: isize,
        encoding: u32,
        external_representation: u8,
    ) -> CfType;
    fn CFDictionaryGetTypeID() -> usize;
    fn CFNumberGetTypeID() -> usize;
    fn CFDictionaryGetValue(dictionary: CfType, key: CfType) -> CfType;
    fn CFNumberGetValue(number: CfType, number_type: i32, value: *mut c_void) -> u8;
    fn CFDataGetTypeID() -> usize;
    fn CFDataGetLength(data: CfType) -> isize;
    fn CFDataGetBytePtr(data: CfType) -> *const u8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_identity_schema_matches_finalizer_join_literal() {
        let value = ExecutableFileIdentity {
            device: 1,
            inode: 2,
            owner_uid: 0,
            owner_gid: 0,
            mode: 0o100755,
            link_count: 1,
            size: 3,
            modified_seconds: 4,
            modified_nanoseconds: 5,
        };
        assert_eq!(
            String::from_utf8(canonical_bytes_v2(&value).unwrap()).unwrap(),
            r#"{"device":1,"inode":2,"link_count":1,"mode":33261,"modified_nanoseconds":5,"modified_seconds":4,"owner_gid":0,"owner_uid":0,"size":3}"#
        );
    }

    #[test]
    fn self_attestation_requires_no_interaction_token_in_its_api() {
        let source = include_str!("attestation.rs");
        let signature = source
            .split("pub(crate) fn attest_publisher_process")
            .nth(1)
            .unwrap()
            .split('{')
            .next()
            .unwrap();
        assert!(signature.contains("&NonInteractiveSecurity"));
        assert!(source.contains("SecCodeCopySelf"));
        assert!(source.contains("process_start_identity"));
        assert!(source.contains("O_NOFOLLOW"));
    }
}
