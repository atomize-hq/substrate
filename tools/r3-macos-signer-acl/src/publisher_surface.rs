//! Minimal closed publisher helpers used by the shared root supervisor.

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::Serialize;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::{Read, Write};
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;

use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, derive_host_effect_plan_v2, document_sha256_v2,
    validate_executable_identity_v2, validate_guest_to_host_successor_capsule_v2,
    validate_launch_identity_v2, validate_process_identity_v2, HarnessDurabilityAcknowledgementV2,
    HostRetirementStateV2, HostTargetRoleV2, MacR3SignatureV2, ProtectedCasBindingV2,
    PublisherPreRemovalReceiptV2, TargetSetKindV2, MAC_R3_COORDINATOR_PATH_V2,
    MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2, MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2,
    MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2, MAC_R3_FINALIZER_PATH_V2,
    MAC_R3_FINALIZER_SIGNING_IDENTIFIER_V2, MAC_R3_HOST_RECEIPT_OWNER_V2,
    MAC_R3_HOST_RECEIPT_SIGNATURE_DOMAIN_V2, MAC_R3_PROTECTED_CAS_OWNER_V2,
    MAC_R3_PROTECTED_CAS_SIGNATURE_DOMAIN_V2,
};

use crate::ffi::FixedP256SignatureV2;
use crate::{
    DisposableKeyPairCreationReceiptV2, DisposableSignerIdentityV2, FixedRepetitionV2,
    DISPOSABLE_PUBLISHER_EXECUTABLE_PATH, DISPOSABLE_PUBLISHER_ROOT,
};

const MAX_PASSWD_BUFFER_BYTES: usize = 1024 * 1024;
const WRAPPER_BASENAME: &str = "surrogate-wrapper.v2";
const FINALIZER_CAPABILITY_ROOT: &str =
    "/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2/capability";

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct WrapperPhysicalIdentity<'a> {
    path: &'a str,
    device: u64,
    inode: u64,
    uid: u32,
    gid: u32,
    mode: u32,
    link_count: u64,
}

pub fn verify_closed_publisher_process_surface() -> Result<()> {
    if std::env::args_os().count() != 1 {
        bail!("sealed disposable publisher rejects all arguments")
    }
    if std::env::vars_os().any(|(name, _)| name.as_bytes().starts_with(b"SUBSTRATE_")) {
        bail!("sealed disposable publisher rejects all SUBSTRATE_* environment inputs")
    }
    let cwd = std::env::current_dir().context("resolve disposable publisher cwd")?;
    if cwd.as_os_str().as_bytes() != DISPOSABLE_PUBLISHER_ROOT.as_bytes() {
        bail!("sealed disposable publisher cwd differs from its compiled root")
    }
    let executable = std::env::current_exe().context("resolve disposable publisher executable")?;
    if executable.as_os_str().as_bytes() != DISPOSABLE_PUBLISHER_EXECUTABLE_PATH.as_bytes()
        || DISPOSABLE_PUBLISHER_EXECUTABLE_PATH != MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2
    {
        bail!("sealed disposable publisher executable differs from its fixed identity")
    }
    // SAFETY: scalar process identity call.
    if unsafe { libc::geteuid() } != 0 || canonical_account_for_uid(0)? != "root" {
        bail!("sealed disposable publisher is not canonical root")
    }
    crate::experiment::clear_and_verify_environment()?;
    Ok(())
}

fn canonical_account_for_uid(uid: libc::uid_t) -> Result<String> {
    // SAFETY: sysconf is a scalar query.
    let suggested = unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) };
    let mut capacity = if suggested > 0 {
        suggested as usize
    } else {
        16 * 1024
    }
    .min(MAX_PASSWD_BUFFER_BYTES);
    loop {
        let mut record = MaybeUninit::<libc::passwd>::uninit();
        let mut result = std::ptr::null_mut();
        let mut buffer = vec![0_u8; capacity];
        // SAFETY: all output storage is live and buffer length is exact.
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
            bail!("resolve canonical root account failed with errno {status}")
        }
        // SAFETY: successful getpwuid_r initialized record into live buffer.
        let record = unsafe { record.assume_init() };
        if record.pw_uid != uid || record.pw_name.is_null() {
            bail!("canonical root account record is inconsistent")
        }
        // SAFETY: pw_name is NUL-terminated inside live buffer.
        return Ok(unsafe { CStr::from_ptr(record.pw_name) }
            .to_str()
            .context("canonical root account is not UTF-8")?
            .to_owned());
    }
}

pub(crate) fn prevalidate_unsigned_receipt(
    receipt: &PublisherPreRemovalReceiptV2,
    repetition: FixedRepetitionV2,
    keys: &DisposableKeyPairCreationReceiptV2,
) -> Result<()> {
    let target = &keys.target;
    if receipt.schema_owner != MAC_R3_HOST_RECEIPT_OWNER_V2
        || receipt.schema_version != 2
        || receipt.signature_domain != MAC_R3_HOST_RECEIPT_SIGNATURE_DOMAIN_V2
        || receipt.scope_id != repetition.finalizer_scope()
        || receipt.target_set_kind != TargetSetKindV2::DisposableCapability
        || receipt.host_state != HostRetirementStateV2::PreRemovalReceiptSigned
        || receipt.issued_at_unix_ns >= receipt.expires_at_unix_ns
        || receipt.protected_cas_generation == 0
    {
        bail!("unsigned receipt is outside the fixed disposable lane")
    }
    validate_guest_to_host_successor_capsule_v2(&receipt.guest_successor_capsule)?;
    if receipt.guest_successor_capsule.scope_id != receipt.scope_id
        || receipt.guest_successor_capsule_sha256
            != document_sha256_v2(&receipt.guest_successor_capsule)?
        || receipt.guest_parity_sha256 != receipt.guest_successor_capsule.guest_parity_proof_sha256
    {
        bail!("unsigned receipt does not bind its validated guest capsule")
    }
    for value in [
        &receipt.guest_successor_capsule_sha256,
        &receipt.guest_parity_sha256,
        &receipt.before_observation_sha256,
        &receipt.quiesced_observation_sha256,
        &receipt.target_ledger_sha256,
        &receipt.effect_plan_sha256,
        &receipt.protected_cas_head_sha256,
        &receipt.current_lock_identity_sha256,
        &receipt.signer_access_control_sha256,
        &receipt.capability_digest,
    ] {
        require_digest(value)?;
    }
    validate_executable_identity_v2(
        &receipt.finalizer_identity,
        MAC_R3_FINALIZER_PATH_V2,
        MAC_R3_FINALIZER_SIGNING_IDENTIFIER_V2,
    )?;
    validate_executable_identity_v2(
        &receipt.coordinator_identity,
        MAC_R3_COORDINATOR_PATH_V2,
        MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2,
    )?;
    validate_process_identity_v2(&receipt.coordinator_process)?;
    if receipt.coordinator_process.executable_identity_sha256
        != document_sha256_v2(&receipt.coordinator_identity)?
    {
        bail!("unsigned receipt coordinator process does not bind its executable")
    }
    validate_launch_identity_v2(&receipt.launch_identity)?;
    let plan = derive_host_effect_plan_v2(receipt.target_set_kind, &receipt.target_ledger)?;
    if document_sha256_v2(&receipt.target_ledger)? != receipt.target_ledger_sha256
        || document_sha256_v2(&plan)? != receipt.effect_plan_sha256
    {
        bail!("unsigned receipt target ledger or plan digest changed")
    }
    let signer = receipt
        .target_ledger
        .iter()
        .find(|entry| entry.role == HostTargetRoleV2::Signer)
        .context("unsigned receipt lacks its exact signer target")?;
    let wrapper = receipt
        .target_ledger
        .iter()
        .find(|entry| entry.role == HostTargetRoleV2::ProtectedWrapper)
        .context("unsigned receipt lacks its exact protected wrapper target")?;
    if receipt
        .target_ledger
        .iter()
        .filter(|entry| entry.role == HostTargetRoleV2::Signer)
        .count()
        != 1
        || receipt
            .target_ledger
            .iter()
            .filter(|entry| entry.role == HostTargetRoleV2::ProtectedWrapper)
            .count()
            != 1
        || signer.expected_before_sha256 != target.identity_sha256
        || receipt.signer_access_control_sha256 != target.access_control_sha256
        || receipt.publisher_signer_spki_der != target.spki_der
        || wrapper.expected_before_sha256 != keys.wrapper_identity_sha256
    {
        bail!("unsigned receipt does not bind the created disposable target signer")
    }
    let harness = URL_SAFE_NO_PAD
        .decode(&receipt.harness_public_key)
        .context("decode fixed receipt harness key")?;
    if harness.len() != 32
        || receipt.signature.algorithm != "ecdsa-p256-sha256-p1363-low-s-v1"
        || receipt.signature.public_key != target.spki_der
        || !receipt.signature.signature.is_empty()
    {
        bail!("unsigned receipt signature placeholder is not exact")
    }
    Ok(())
}

pub(crate) fn prevalidate_unsigned_binding(
    binding: &ProtectedCasBindingV2,
    repetition: FixedRepetitionV2,
    target: &DisposableSignerIdentityV2,
    receipt: &PublisherPreRemovalReceiptV2,
    acknowledgement: &HarnessDurabilityAcknowledgementV2,
) -> Result<()> {
    if binding.schema_owner != MAC_R3_PROTECTED_CAS_OWNER_V2
        || binding.schema_version != 2
        || binding.signature_domain != MAC_R3_PROTECTED_CAS_SIGNATURE_DOMAIN_V2
        || binding.scope_id != repetition.finalizer_scope()
        || binding.evidence_id != receipt.evidence_id
        || binding.generation != receipt.protected_cas_generation + 1
        || binding.predecessor_head_sha256 != receipt.protected_cas_head_sha256
        || binding.receipt_sha256 != document_sha256_v2(receipt)?
        || binding.acknowledgement_sha256 != document_sha256_v2(acknowledgement)?
        || binding.target_ledger_sha256 != receipt.target_ledger_sha256
        || binding.effect_plan_sha256 != receipt.effect_plan_sha256
        || binding.guest_successor_capsule_sha256 != receipt.guest_successor_capsule_sha256
        || binding.guest_parity_sha256 != receipt.guest_parity_sha256
        || binding.current_lock_identity_sha256 != receipt.current_lock_identity_sha256
        || binding.signer_access_control_sha256 != target.access_control_sha256
        || binding.signature.algorithm != "ecdsa-p256-sha256-p1363-low-s-v1"
        || binding.signature.public_key != target.spki_der
        || !binding.signature.signature.is_empty()
    {
        bail!("unsigned protected CAS is outside its exact signed receipt binding")
    }
    require_digest(&binding.request_digest)
}

pub(crate) fn apply_signature(
    destination: &mut MacR3SignatureV2,
    signature: &FixedP256SignatureV2,
    target: &DisposableSignerIdentityV2,
) -> Result<()> {
    if signature.public_spki_der_base64url != target.spki_der
        || destination.algorithm != "ecdsa-p256-sha256-p1363-low-s-v1"
        || destination.public_key != target.spki_der
        || !destination.signature.is_empty()
    {
        bail!("fixed signer result or signature placeholder changed")
    }
    destination.signature = signature.signature_p1363_low_s_base64url.clone();
    Ok(())
}

fn require_digest(value: &str) -> Result<()> {
    if value.len() != 64
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte))
    {
        bail!("fixed signed document contains a non-SHA-256 digest")
    }
    Ok(())
}

pub(crate) fn wrapper_path(repetition: FixedRepetitionV2) -> String {
    format!(
        "{FINALIZER_CAPABILITY_ROOT}/{}/{WRAPPER_BASENAME}",
        repetition.finalizer_scope()
    )
}

fn open_wrapper_parent(repetition: FixedRepetitionV2) -> Result<OwnedFd> {
    let path = CString::new(format!(
        "{FINALIZER_CAPABILITY_ROOT}/{}",
        repetition.finalizer_scope()
    ))?;
    // SAFETY: fixed NUL-terminated directory path and no-follow flags.
    let raw = unsafe {
        libc::open(
            path.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if raw < 0 {
        return Err(std::io::Error::last_os_error()).context("open fixed wrapper parent");
    }
    // SAFETY: raw is newly owned.
    let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
    let metadata = fstat(descriptor.as_raw_fd())?;
    if metadata.st_uid != 0
        || metadata.st_gid != 0
        || (metadata.st_mode & (libc::S_IFMT | 0o7777)) != (libc::S_IFDIR | 0o700)
    {
        bail!("fixed wrapper parent is not exact root:wheel 0700 directory")
    }
    Ok(descriptor)
}

pub(crate) fn prepare_empty_wrapper(repetition: FixedRepetitionV2) -> Result<String> {
    let parent = open_wrapper_parent(repetition)?;
    let name = CString::new(WRAPPER_BASENAME)?;
    // SAFETY: fixed component under held exact parent, exclusive and no-follow creation.
    let raw = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDWR | libc::O_CREAT | libc::O_EXCL | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            0o400,
        )
    };
    if raw >= 0 {
        // SAFETY: raw is newly owned.
        let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
        // SAFETY: live descriptor and frozen root ownership/mode.
        if unsafe { libc::fchown(descriptor.as_raw_fd(), 0, 0) } != 0
            || unsafe { libc::fchmod(descriptor.as_raw_fd(), 0o400) } != 0
        {
            return Err(std::io::Error::last_os_error()).context("bind empty wrapper identity");
        }
        File::from(descriptor)
            .sync_all()
            .context("sync empty disposable wrapper")?;
    } else if std::io::Error::last_os_error().raw_os_error() != Some(libc::EEXIST) {
        return Err(std::io::Error::last_os_error()).context("create exact empty wrapper");
    }
    // SAFETY: fixed component under held exact parent and no-follow.
    let raw = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if raw < 0 {
        return Err(std::io::Error::last_os_error()).context("reopen exact empty wrapper");
    }
    // SAFETY: raw is newly owned.
    let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
    let metadata = fstat(descriptor.as_raw_fd())?;
    require_wrapper_stat(&metadata)?;
    if metadata.st_size != 0 {
        bail!("disposable wrapper is not empty before receipt construction")
    }
    // SAFETY: held directory descriptor.
    if unsafe { libc::fsync(parent.as_raw_fd()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("sync wrapper parent");
    }
    wrapper_identity_sha256(repetition, &metadata)
}

pub(crate) fn fill_or_verify_wrapper(
    repetition: FixedRepetitionV2,
    expected_identity_sha256: &str,
    bytes: &[u8],
) -> Result<()> {
    require_digest(expected_identity_sha256)?;
    let parent = open_wrapper_parent(repetition)?;
    let name = CString::new(WRAPPER_BASENAME)?;
    let (existing, metadata) = read_wrapper(&parent, &name)?;
    if wrapper_identity_sha256(repetition, &metadata)? != expected_identity_sha256 {
        bail!("disposable wrapper physical identity changed before CAS fill")
    }
    if existing == bytes {
        return Ok(());
    }
    if !existing.is_empty() {
        bail!("disposable wrapper contains different bytes before CAS fill")
    }
    // SAFETY: fixed component under held exact parent and no-follow.
    let raw = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            libc::O_WRONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if raw < 0 {
        return Err(std::io::Error::last_os_error())
            .context("open exact wrapper for in-place fill");
    }
    // SAFETY: raw is newly owned.
    let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
    let before = fstat(descriptor.as_raw_fd())?;
    if wrapper_identity_sha256(repetition, &before)? != expected_identity_sha256
        || before.st_size != 0
    {
        bail!("disposable wrapper changed between readback and in-place fill")
    }
    let mut file = File::from(descriptor);
    file.write_all(bytes)
        .context("fill signed CAS into exact wrapper inode")?;
    file.sync_all().context("sync exact signed CAS wrapper")?;
    let after = fstat(file.as_raw_fd())?;
    if wrapper_identity_sha256(repetition, &after)? != expected_identity_sha256 {
        bail!("disposable wrapper identity changed during in-place fill")
    }
    drop(file);
    let (readback, reopened) = read_wrapper(&parent, &name)?;
    if wrapper_identity_sha256(repetition, &reopened)? != expected_identity_sha256
        || readback != bytes
    {
        bail!("filled disposable wrapper failed same-inode exact readback")
    }
    Ok(())
}

fn read_wrapper(parent: &OwnedFd, name: &CString) -> Result<(Vec<u8>, libc::stat)> {
    // SAFETY: fixed component under held exact parent and no-follow.
    let raw = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if raw < 0 {
        return Err(std::io::Error::last_os_error()).context("read exact disposable wrapper");
    }
    // SAFETY: raw is newly owned.
    let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
    let metadata = fstat(descriptor.as_raw_fd())?;
    require_wrapper_stat(&metadata)?;
    if metadata.st_size < 0 || metadata.st_size as usize > MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2 {
        bail!("disposable wrapper content exceeds its fixed bound")
    }
    let mut bytes = Vec::with_capacity(metadata.st_size as usize);
    File::from(descriptor)
        .read_to_end(&mut bytes)
        .context("read disposable wrapper bytes")?;
    Ok((bytes, metadata))
}

fn require_wrapper_stat(value: &libc::stat) -> Result<()> {
    if value.st_uid != 0
        || value.st_gid != 0
        || value.st_nlink != 1
        || (value.st_mode & (libc::S_IFMT | 0o7777)) != (libc::S_IFREG | 0o400)
    {
        bail!("disposable wrapper is not exact root:wheel regular 0400 nlink=1")
    }
    Ok(())
}

fn wrapper_identity_sha256(repetition: FixedRepetitionV2, value: &libc::stat) -> Result<String> {
    require_wrapper_stat(value)?;
    Ok(substrate_common::macos_retirement_v2::sha256_hex_v2(
        &canonical_bytes_v2(&WrapperPhysicalIdentity {
            path: &wrapper_path(repetition),
            device: value.st_dev as u64,
            inode: value.st_ino,
            uid: value.st_uid,
            gid: value.st_gid,
            mode: u32::from(value.st_mode),
            link_count: u64::from(value.st_nlink),
        })?,
    ))
}

fn fstat(fd: i32) -> Result<libc::stat> {
    let mut value = MaybeUninit::<libc::stat>::uninit();
    // SAFETY: value is writable and fd is live.
    if unsafe { libc::fstat(fd, value.as_mut_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("fstat fixed publisher object");
    }
    // SAFETY: successful fstat initialized the structure.
    Ok(unsafe { value.assume_init() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::DisposableAclKind;
    use crate::{
        compiled_disposable_target_config, compiled_disposable_wrong_config,
        DISPOSABLE_PUBLISHER_EXECUTABLE_PATH, EXPERIMENT_FINALIZER_EXECUTABLE_PATH,
    };

    #[test]
    fn sealed_surface_has_only_fixed_tags_files_and_domains() {
        let source = include_str!("publisher_surface.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        assert!(!production.contains("std::env::args("));
        assert!(!production.contains("std::env::var("));
        assert!(production.contains("std::env::args_os().count() != 1"));
        assert!(production.contains("starts_with(b\"SUBSTRATE_\")"));
        assert!(production.contains("O_NOFOLLOW"));
        for repetition in FixedRepetitionV2::ALL {
            let target = compiled_disposable_target_config(repetition).unwrap();
            let wrong = compiled_disposable_wrong_config(repetition).unwrap();
            assert_eq!(
                target.application_tag(),
                format!("{}:signing-key", repetition.finalizer_scope()).as_bytes()
            );
            assert_eq!(
                wrong.application_tag(),
                format!(
                    "{}:wrong-surrogate-signing-key",
                    repetition.finalizer_scope()
                )
                .as_bytes()
            );
            assert_ne!(target.application_tag(), wrong.application_tag());
        }
        assert_ne!(
            DISPOSABLE_PUBLISHER_EXECUTABLE_PATH,
            EXPERIMENT_FINALIZER_EXECUTABLE_PATH
        );
    }

    #[test]
    fn wrapper_physical_identity_matches_finalizer_canonical_fixture() {
        let path = wrapper_path(FixedRepetitionV2::First);
        let bytes = canonical_bytes_v2(&WrapperPhysicalIdentity {
            path: &path,
            device: 7,
            inode: 11,
            uid: 0,
            gid: 0,
            mode: 0o100400,
            link_count: 1,
        })
        .unwrap();
        assert_eq!(
            substrate_common::macos_retirement_v2::sha256_hex_v2(&bytes),
            "028d96aa2d5b510318cccb15e8fc0b044ca96df58664455970641b1629912a55"
        );
    }

    #[test]
    fn wrong_surrogate_omits_finalizer_delete_but_keeps_publisher_rollback() {
        let target = crate::ffi::expected_disposable_snapshot_for_test(
            b"publisher",
            b"finalizer",
            DisposableAclKind::Target,
        );
        let wrong = crate::ffi::expected_disposable_snapshot_for_test(
            b"publisher",
            b"finalizer",
            DisposableAclKind::WrongSurrogate,
        );
        assert!(target
            .entries
            .iter()
            .any(|entry| entry.description == crate::FINALIZER_DELETE_DESCRIPTION));
        assert!(!wrong
            .entries
            .iter()
            .any(|entry| entry.description == crate::FINALIZER_DELETE_DESCRIPTION));
        assert!(wrong.entries.iter().any(|entry| {
            entry.description == crate::PUBLISHER_SIGN_DELETE_DESCRIPTION
                && entry.authorizations == [crate::AUTH_DELETE, crate::AUTH_SIGN]
        }));
    }
}
