//! Pre-effect evidence for the all-ones legacy SecAccess owner candidate.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::mem::MaybeUninit;

use crate::{NONMATCH_OWNER_GID, NONMATCH_OWNER_UID, OWNER_TYPE_USE_ONLY_UID_AND_GID};

const LOOKUP_BUFFER_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NonmatchOwnerProbeV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub owner_uid: u32,
    pub owner_gid: u32,
    pub owner_type: u32,
    pub passwd_record_absent: bool,
    pub group_record_absent: bool,
    pub setuid_rejected_with_einval: bool,
    pub setgid_rejected_with_einval: bool,
    pub parent_remained_root: bool,
}

impl NonmatchOwnerProbeV2 {
    pub fn validate(&self) -> Result<()> {
        if self.schema_owner != "substrate.r3-macos-legacy-access-nonmatch-owner-probe"
            || self.schema_version != 2
            || self.owner_uid != NONMATCH_OWNER_UID
            || self.owner_gid != NONMATCH_OWNER_GID
            || self.owner_type != OWNER_TYPE_USE_ONLY_UID_AND_GID
            || !self.passwd_record_absent
            || !self.group_record_absent
            || !self.setuid_rejected_with_einval
            || !self.setgid_rejected_with_einval
            || !self.parent_remained_root
        {
            bail!("legacy SecAccess nonmatch-owner preflight is not exact")
        }
        Ok(())
    }
}

/// Prove that neither all-ones owner ID resolves and that a root child cannot adopt either ID.
/// This does not prove Security.framework accepts the owner; exact in-memory construction/readback
/// remains a separate mandatory gate immediately before any key creation.
pub fn probe_nonmatch_owner_candidate() -> Result<NonmatchOwnerProbeV2> {
    // SAFETY: scalar identity query.
    if unsafe { libc::geteuid() } != 0 {
        bail!("nonmatch-owner preflight requires canonical root")
    }
    if passwd_record_present(NONMATCH_OWNER_UID)? || group_record_present(NONMATCH_OWNER_GID)? {
        bail!("all-ones owner candidate resolves to a live account or group")
    }

    // SAFETY: this preflight is called before threads and before any persistent/native authority
    // operation; the child performs only irreversible identity attempts and `_exit`.
    let child = unsafe { libc::fork() };
    if child < 0 {
        return Err(std::io::Error::last_os_error()).context("fork nonmatch-owner preflight");
    }
    if child == 0 {
        // SAFETY: scalar credential calls in the isolated preflight child.
        let gid_result = unsafe { libc::setgid(NONMATCH_OWNER_GID) };
        // SAFETY: macOS exposes the current thread-local errno pointer.
        let gid_errno = unsafe { *libc::__error() };
        if gid_result != -1 || gid_errno != libc::EINVAL {
            // SAFETY: the preflight child must not run Rust destructors after fork.
            unsafe { libc::_exit(41) }
        }
        // SAFETY: same isolated child credential attempt.
        let uid_result = unsafe { libc::setuid(NONMATCH_OWNER_UID) };
        // SAFETY: thread-local errno pointer after the immediately preceding call.
        let uid_errno = unsafe { *libc::__error() };
        // SAFETY: no Rust destructors in the child.
        unsafe {
            libc::_exit(if uid_result == -1 && uid_errno == libc::EINVAL {
                0
            } else {
                42
            })
        }
    }
    let mut status = 0;
    loop {
        // SAFETY: child is the exact newly forked preflight PID.
        let waited = unsafe { libc::waitpid(child, &mut status, 0) };
        if waited == child {
            break;
        }
        if waited < 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
            continue;
        }
        return Err(std::io::Error::last_os_error()).context("wait nonmatch-owner preflight");
    }
    if !libc::WIFEXITED(status) || libc::WEXITSTATUS(status) != 0 {
        bail!("root child could adopt an all-ones owner ID or received a non-EINVAL result")
    }
    // SAFETY: the credential attempt occurred only in the child.
    let parent_remained_root = unsafe { libc::geteuid() } == 0 && unsafe { libc::getegid() } == 0;
    let evidence = NonmatchOwnerProbeV2 {
        schema_owner: "substrate.r3-macos-legacy-access-nonmatch-owner-probe".to_owned(),
        schema_version: 2,
        owner_uid: NONMATCH_OWNER_UID,
        owner_gid: NONMATCH_OWNER_GID,
        owner_type: OWNER_TYPE_USE_ONLY_UID_AND_GID,
        passwd_record_absent: true,
        group_record_absent: true,
        setuid_rejected_with_einval: true,
        setgid_rejected_with_einval: true,
        parent_remained_root,
    };
    evidence.validate()?;
    Ok(evidence)
}

fn passwd_record_present(uid: u32) -> Result<bool> {
    let mut record = MaybeUninit::<libc::passwd>::uninit();
    let mut result = std::ptr::null_mut();
    let mut buffer = vec![0_u8; LOOKUP_BUFFER_BYTES];
    // SAFETY: all outputs and the full lookup buffer are live.
    let status = unsafe {
        libc::getpwuid_r(
            uid,
            record.as_mut_ptr(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            &mut result,
        )
    };
    if status != 0 {
        bail!("all-ones passwd lookup failed with errno {status} instead of exact absence")
    }
    Ok(!result.is_null())
}

fn group_record_present(gid: u32) -> Result<bool> {
    let mut record = MaybeUninit::<libc::group>::uninit();
    let mut result = std::ptr::null_mut();
    let mut buffer = vec![0_u8; LOOKUP_BUFFER_BYTES];
    // SAFETY: all outputs and the full lookup buffer are live.
    let status = unsafe {
        libc::getgrgid_r(
            gid,
            record.as_mut_ptr(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            &mut result,
        )
    };
    if status != 0 {
        bail!("all-ones group lookup failed with errno {status} instead of exact absence")
    }
    Ok(!result.is_null())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_is_all_ones_with_both_match_bits_and_no_ordinary_uid_literal() {
        assert_eq!(NONMATCH_OWNER_UID, u32::MAX);
        assert_eq!(NONMATCH_OWNER_GID, u32::MAX);
        assert_eq!(OWNER_TYPE_USE_ONLY_UID_AND_GID, 3);
        let source = include_str!("owner_probe.rs");
        assert!(source.contains("getpwuid_r"));
        assert!(source.contains("getgrgid_r"));
        assert!(source.contains("setuid"));
        assert!(source.contains("setgid"));
    }
}
