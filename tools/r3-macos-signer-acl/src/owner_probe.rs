//! Pre-effect evidence for the directory-unresolved legacy SecAccess owner candidate.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::mem::MaybeUninit;

use crate::{NONMATCH_OWNER_GID, NONMATCH_OWNER_UID, OWNER_TYPE_USE_ONLY_UID_AND_GID};

const LOOKUP_BUFFER_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NonmatchOwnerProbeV3 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub owner_uid: u32,
    pub owner_gid: u32,
    pub owner_type: u32,
    pub passwd_record_absent: bool,
    pub group_record_absent: bool,
    pub root_child_adopted_owner_tuple: bool,
    pub root_adoptability_is_accepted_tcb_fact: bool,
    pub owner_tuple_confers_lifecycle_authority: bool,
    pub parent_remained_root: bool,
}

impl NonmatchOwnerProbeV3 {
    pub fn validate(&self) -> Result<()> {
        if self.schema_owner != "substrate.r3-macos-legacy-access-nonmatch-owner-probe"
            || self.schema_version != 3
            || self.owner_uid != NONMATCH_OWNER_UID
            || self.owner_gid != NONMATCH_OWNER_GID
            || self.owner_type != OWNER_TYPE_USE_ONLY_UID_AND_GID
            || !self.passwd_record_absent
            || !self.group_record_absent
            || !self.root_child_adopted_owner_tuple
            || !self.root_adoptability_is_accepted_tcb_fact
            || self.owner_tuple_confers_lifecycle_authority
            || !self.parent_remained_root
        {
            bail!("legacy SecAccess nonmatch-owner preflight is not exact")
        }
        Ok(())
    }
}

/// Prove that the fixed owner tuple is directory-unresolved and that root can adopt it, as expected
/// inside the accepted macOS software-key TCB. Adoption does not confer lifecycle authority.
/// This does not prove Security.framework accepts the owner; exact in-memory construction/readback
/// remains a separate mandatory gate immediately before any key creation.
pub fn probe_nonmatch_owner_candidate() -> Result<NonmatchOwnerProbeV3> {
    // SAFETY: scalar identity query.
    if unsafe { libc::geteuid() } != 0 {
        bail!("nonmatch-owner preflight requires canonical root")
    }
    if passwd_record_present(NONMATCH_OWNER_UID)? || group_record_present(NONMATCH_OWNER_GID)? {
        bail!("fixed owner candidate resolves to a live account or group")
    }

    // SAFETY: this preflight is called before threads and before any persistent/native authority
    // operation; the child performs only irreversible identity attempts and `_exit`.
    let child = unsafe { libc::fork() };
    if child < 0 {
        return Err(std::io::Error::last_os_error()).context("fork nonmatch-owner preflight");
    }
    if child == 0 {
        // SAFETY: scalar credential calls in the isolated preflight child.
        if unsafe { libc::setgid(NONMATCH_OWNER_GID) } != 0 {
            // SAFETY: the preflight child must not run Rust destructors after fork.
            unsafe { libc::_exit(41) }
        }
        // SAFETY: same isolated child credential attempt.
        if unsafe { libc::setuid(NONMATCH_OWNER_UID) } != 0 {
            // SAFETY: no Rust destructors in the child.
            unsafe { libc::_exit(42) }
        }
        // SAFETY: scalar credential readback in the isolated child.
        let tuple_matches = unsafe {
            libc::getuid() == NONMATCH_OWNER_UID
                && libc::geteuid() == NONMATCH_OWNER_UID
                && libc::getgid() == NONMATCH_OWNER_GID
                && libc::getegid() == NONMATCH_OWNER_GID
        };
        // SAFETY: no Rust destructors in the child.
        unsafe { libc::_exit(if tuple_matches { 0 } else { 43 }) }
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
        bail!("root child did not adopt the fixed directory-unresolved owner tuple")
    }
    // SAFETY: the credential attempt occurred only in the child.
    let parent_remained_root = unsafe { libc::geteuid() } == 0 && unsafe { libc::getegid() } == 0;
    let evidence = NonmatchOwnerProbeV3 {
        schema_owner: "substrate.r3-macos-legacy-access-nonmatch-owner-probe".to_owned(),
        schema_version: 3,
        owner_uid: NONMATCH_OWNER_UID,
        owner_gid: NONMATCH_OWNER_GID,
        owner_type: OWNER_TYPE_USE_ONLY_UID_AND_GID,
        passwd_record_absent: true,
        group_record_absent: true,
        root_child_adopted_owner_tuple: true,
        root_adoptability_is_accepted_tcb_fact: true,
        owner_tuple_confers_lifecycle_authority: false,
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
        bail!("fixed owner passwd lookup failed with errno {status} instead of exact absence")
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
        bail!("fixed owner group lookup failed with errno {status} instead of exact absence")
    }
    Ok(!result.is_null())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_is_fixed_directory_unresolved_pair_with_both_match_bits() {
        assert_eq!(NONMATCH_OWNER_UID, 4_294_967_293);
        assert_eq!(NONMATCH_OWNER_GID, 4_294_967_293);
        assert_ne!(NONMATCH_OWNER_UID, u32::MAX);
        assert_eq!(OWNER_TYPE_USE_ONLY_UID_AND_GID, 3);
        let source = include_str!("owner_probe.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        assert!(production.contains("getpwuid_r"));
        assert!(production.contains("getgrgid_r"));
        assert!(production.contains("setuid"));
        assert!(production.contains("setgid"));
        assert!(!production.contains("EINVAL"));
        let runner = include_str!("runner.rs");
        let runner_production = runner.split("#[cfg(test)]").next().unwrap();
        assert!(runner_production.contains("legacy-access-nonmatch-owner-probe.v3.json"));
        assert!(!runner_production.contains("legacy-access-nonmatch-owner-probe.v2.json"));
    }

    #[test]
    fn v3_receipt_closes_root_adoptability_and_non_authority_semantics() {
        let exact = NonmatchOwnerProbeV3 {
            schema_owner: "substrate.r3-macos-legacy-access-nonmatch-owner-probe".to_owned(),
            schema_version: 3,
            owner_uid: NONMATCH_OWNER_UID,
            owner_gid: NONMATCH_OWNER_GID,
            owner_type: OWNER_TYPE_USE_ONLY_UID_AND_GID,
            passwd_record_absent: true,
            group_record_absent: true,
            root_child_adopted_owner_tuple: true,
            root_adoptability_is_accepted_tcb_fact: true,
            owner_tuple_confers_lifecycle_authority: false,
            parent_remained_root: true,
        };
        exact.validate().unwrap();

        let mut changed = exact.clone();
        changed.root_child_adopted_owner_tuple = false;
        assert!(changed.validate().is_err());
        let mut changed = exact.clone();
        changed.root_adoptability_is_accepted_tcb_fact = false;
        assert!(changed.validate().is_err());
        let mut changed = exact.clone();
        changed.owner_tuple_confers_lifecycle_authority = true;
        assert!(changed.validate().is_err());

        let legacy = serde_json::json!({
            "schema_owner": "substrate.r3-macos-legacy-access-nonmatch-owner-probe",
            "schema_version": 2,
            "owner_uid": u32::MAX,
            "owner_gid": u32::MAX,
            "owner_type": OWNER_TYPE_USE_ONLY_UID_AND_GID,
            "passwd_record_absent": true,
            "group_record_absent": true,
            "setuid_rejected_with_einval": true,
            "setgid_rejected_with_einval": true,
            "parent_remained_root": true,
        });
        assert!(serde_json::from_value::<NonmatchOwnerProbeV3>(legacy).is_err());
    }
}
