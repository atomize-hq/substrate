//! Narrow Darwin boundary for the root finalizer.
//!
//! All Security.framework calls are intentionally reachable only through
//! [`SecurityUiDenied`].  Constructing that token makes
//! `SecKeychainSetUserInteractionAllowed(false)` the first Security call made by this module for
//! the process.  The token is never reset: the finalizer must remain noninteractive until exit.
//!
//! This module deliberately has no private-key signing, private-key export, `SecAccess`, or
//! `SecACL` entry points.  Its only Keychain mutation is deleting one already-resolved P-256
//! private-key item and then proving that the same exact application-tag query is absent.

use anyhow::{bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::ffi::{c_char, c_int, c_void, CStr, CString, OsString};
use std::fs::File;
use std::io::Read;
use std::mem::{size_of, MaybeUninit};
use std::os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::atomic::{AtomicU8, Ordering};

use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, sha256_hex_v2, ExecutableIdentityV2, PublisherPreRemovalReceiptV2,
    MAC_R3_COORDINATOR_PATH_V2, MAC_R3_FINALIZER_ENDPOINT_V2, MAC_R3_FINALIZER_PATH_V2,
};

pub use crate::disposable_capability::{
    CanonicalDisposableCapabilityAfterV2, CanonicalDisposableCapabilityInvocationV2,
    CanonicalDisposableCapabilityStateV2, DisposableCapabilityAfterObservationV2,
    DisposableCapabilityControlV2, DisposableCapabilityInvocationV2,
    DisposableCapabilityStateObservationV2, DisposableRecoveryObservationClassV2,
    FrozenDenialClassV2, KeyIntegrityObservationV2,
};

use crate::contract::{
    ENDPOINT_GROUP_GID, ENDPOINT_MODE, ENDPOINT_OWNER_UID, FINALIZER_ACCOUNT, FINALIZER_UID,
    LISTENER_SOCKET_NAME,
};
use crate::disposable_capability::RetainedExactSystemKey;
use crate::frozen_identity::{
    EXPECTED_COORDINATOR_ACCOUNT, EXPECTED_COORDINATOR_CDHASH, EXPECTED_COORDINATOR_GID,
    EXPECTED_COORDINATOR_REQUIREMENT, EXPECTED_COORDINATOR_SHA256, EXPECTED_COORDINATOR_UID,
};

pub const ACCEPTED_SOCKET_FD: RawFd = 3;
pub const SYSTEM_KEYCHAIN_PATH: &str = "/Library/Keychains/System.keychain";

const LISTENER_SOCKET_NAME_C: &[u8] = b"Listener\0";
const SOL_LOCAL: c_int = 0;
const LOCAL_PEERPID: c_int = 0x002;
const LOCAL_PEERTOKEN: c_int = 0x006;
const PROC_PIDTBSDINFO: c_int = 3;
const PROC_PIDPATHINFO_MAXSIZE: usize = 4 * 1024;
const MAX_SYSTEM_KEYCHAIN_PATH_BYTES: usize = 1024;
const MAX_APPLICATION_TAG_BYTES: usize = 1024;
const MAX_GENERIC_PASSWORD_FIELD_BYTES: usize = 1024;
const MAX_GENERIC_PASSWORD_DATA_BYTES: usize = crate::contract::FRAME_MAX_BYTES;
const DEFAULT_PASSWD_BUFFER_BYTES: usize = 16 * 1024;
const MAX_PASSWD_BUFFER_BYTES: usize = 1024 * 1024;

type CfType = *const c_void;
type CfMutableDictionary = *mut c_void;
type SecCode = *const c_void;
type SecKey = *const c_void;
type SecKeychain = *const c_void;
type SecRequirement = *const c_void;
type OsStatus = i32;

const ERR_SEC_SUCCESS: OsStatus = 0;
const ERR_SEC_AUTH_FAILED: OsStatus = -25293;
const ERR_SEC_ITEM_NOT_FOUND: OsStatus = -25300;
const ERR_SEC_INTERACTION_NOT_ALLOWED: OsStatus = -25308;
const ERR_SEC_INTERACTION_REQUIRED: OsStatus = -25315;
const K_CF_NUMBER_SINT64: i32 = 4;
const K_SEC_CS_SIGNING_AND_REQUIREMENT_INFORMATION: u32 = (1 << 1) | (1 << 2);
/// `CS_ADHOC | CS_REQUIRE_LV | CS_RUNTIME`, emitted by the frozen
/// `codesign --sign - --options runtime,library` recipe.
const REQUIRED_CODE_DIRECTORY_FLAGS: u32 = 0x0001_2002;

// 0 = untouched, 1 = the one allowed call is in progress, 2 = interaction is denied,
// 3 = the first Security call failed.  There is intentionally no transition back to 0.
static SECURITY_UI_STATE: AtomicU8 = AtomicU8::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fd3Plan {
    AlreadyFd3,
    DuplicateToFd3,
}

/// Pure fail-closed decision helper used by the descriptor-normalization boundary.
pub fn classify_fd3_plan(source_fd: RawFd, fd3_is_open: bool) -> Result<Fd3Plan> {
    if source_fd < 0 {
        bail!("accepted socket descriptor is invalid")
    }
    if source_fd == ACCEPTED_SOCKET_FD {
        return Ok(Fd3Plan::AlreadyFd3);
    }
    if fd3_is_open {
        bail!("refuse to clobber an unrelated descriptor already occupying FD3")
    }
    Ok(Fd3Plan::DuplicateToFd3)
}

/// Validate the closed dynamic-loader posture independently from CoreFoundation parsing.
/// Hardened Runtime alone is not accepted: explicit library validation and zero entitlements
/// are part of the executable identity frozen by the capability packet.
pub fn validate_frozen_code_posture(
    code_directory_flags: u32,
    has_team_identifier: bool,
    has_entitlements_blob: bool,
    has_entitlements_dictionary: bool,
) -> Result<()> {
    if code_directory_flags != REQUIRED_CODE_DIRECTORY_FLAGS {
        bail!("code identity lacks exact ad-hoc Hardened Runtime library validation")
    }
    if has_team_identifier {
        bail!("ad-hoc code identity unexpectedly carries a TeamIdentifier")
    }
    if has_entitlements_blob || has_entitlements_dictionary {
        bail!("code identity carries an entitlement blob or dictionary")
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeychainStatusClass {
    Success,
    ItemNotFound,
    InteractionNotAllowed,
    InteractionRequired,
    AuthorizationDenied,
    Other(OsStatus),
}

/// Pure OSStatus classifier.  Both interaction-denial classes are preserving failures; callers
/// must never retry them with UI enabled.
pub fn classify_keychain_status(status: OsStatus) -> KeychainStatusClass {
    match status {
        ERR_SEC_SUCCESS => KeychainStatusClass::Success,
        ERR_SEC_ITEM_NOT_FOUND => KeychainStatusClass::ItemNotFound,
        ERR_SEC_INTERACTION_NOT_ALLOWED => KeychainStatusClass::InteractionNotAllowed,
        ERR_SEC_INTERACTION_REQUIRED => KeychainStatusClass::InteractionRequired,
        ERR_SEC_AUTH_FAILED => KeychainStatusClass::AuthorizationDenied,
        other => KeychainStatusClass::Other(other),
    }
}

fn keychain_error(operation: &str, status: OsStatus) -> anyhow::Error {
    anyhow::anyhow!(
        "{operation} failed with OSStatus {status} ({:?})",
        classify_keychain_status(status)
    )
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AuditToken {
    values: [u32; 8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerProcessIdentity {
    pub pid: libc::pid_t,
    pub effective_uid: libc::uid_t,
    pub effective_gid: libc::gid_t,
    pub canonical_account: String,
    pub audit_token: [u32; 8],
}

/// Reconcile the independent `LOCAL_PEERPID`/`getpeereid` and `LOCAL_PEERTOKEN` projections.
/// The function is public only to permit native-effect-free tests.
pub fn reconcile_peer_identity(
    socket_pid: libc::pid_t,
    socket_euid: libc::uid_t,
    socket_egid: libc::gid_t,
    token_pid: libc::pid_t,
    token_euid: libc::uid_t,
    token_egid: libc::gid_t,
) -> Result<()> {
    if socket_pid <= 1 || token_pid <= 1 {
        bail!("peer process identity has an invalid PID")
    }
    if socket_pid != token_pid || socket_euid != token_euid || socket_egid != token_egid {
        bail!("LOCAL_PEERPID/LOCAL_PEERTOKEN/getpeereid identities do not agree exactly")
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessStartIdentity {
    pub seconds: u64,
    pub microseconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableFileIdentity {
    pub device: u64,
    pub inode: u64,
    pub owner_uid: u32,
    pub owner_gid: u32,
    pub mode: u32,
    pub link_count: u64,
    pub size: u64,
    pub modified_seconds: i64,
    pub modified_nanoseconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessMeasurement {
    pub start: ProcessStartIdentity,
    pub executable_path: PathBuf,
    pub executable_file: ExecutableFileIdentity,
    pub executable_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedCoordinatorProcess {
    pub peer: PeerProcessIdentity,
    pub measurement: ProcessMeasurement,
    pub cdhash: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedFinalizerProcess {
    pub measurement: ProcessMeasurement,
    pub cdhash: Vec<u8>,
}

/// Coordinator-side attestation of the root finalizer connected over the inherited Unix socket.
/// Every field participates in equality so callers can require an unchanged peer before and after
/// framing without retaining any raw Security or process-table references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedFinalizerPeerProcess {
    pub peer: PeerProcessIdentity,
    pub measurement: ProcessMeasurement,
    pub cdhash: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExactKeyState {
    Absent,
    Present,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExactKeyDeleteOutcome {
    AlreadyAbsent,
    DeletedAndAbsent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactGenericPasswordIdentityV2 {
    pub service: String,
    pub account: String,
    pub persistent_reference_sha256: String,
    pub identity_sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExactGenericPasswordDeleteOutcome {
    DeletedAndAbsent,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct GenericPasswordIdentityMaterial<'a> {
    service: &'a str,
    account: &'a str,
    persistent_reference_sha256: &'a str,
}

pub(crate) struct RetainedExactGenericPassword {
    owned: OwnedCf,
    keychain: SecKeychain,
    item: CfType,
    service: String,
    account: String,
    identity: ExactGenericPasswordIdentityV2,
}

impl RetainedExactGenericPassword {
    pub(crate) fn identity(&self) -> &ExactGenericPasswordIdentityV2 {
        &self.identity
    }
}

struct ExactGenericPasswordDataMatch {
    data: Vec<u8>,
    persistent_reference: CfType,
}

/// Process-lifetime proof that optional Security/Keychain interaction was disabled before this
/// module made any other Security.framework call.
///
/// There is no `Drop` reset.  Once established, the finalizer stays noninteractive until exit.
#[derive(Debug)]
pub struct SecurityUiDenied {
    _private: (),
}

impl SecurityUiDenied {
    /// Make process-level interaction denial the first Security.framework call in this module.
    /// A failed first call permanently poisons this module for the lifetime of the process.
    pub fn establish_first() -> Result<Self> {
        match SECURITY_UI_STATE.compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => {
                // SAFETY: this is a scalar process-global Security API.  The atomic state ensures
                // it is the first and only invocation from this module.
                let status = unsafe { SecKeychainSetUserInteractionAllowed(0) };
                if status != ERR_SEC_SUCCESS {
                    SECURITY_UI_STATE.store(3, Ordering::SeqCst);
                    return Err(keychain_error(
                        "disable process-level Keychain interaction as the first Security call",
                        status,
                    ));
                }
                SECURITY_UI_STATE.store(2, Ordering::SeqCst);
                Ok(Self { _private: () })
            }
            Err(2) => Ok(Self { _private: () }),
            Err(1) => bail!("Security interaction denial is concurrently initializing"),
            Err(3) => bail!("the first Security interaction-denial call previously failed"),
            Err(other) => bail!("invalid Security interaction-denial state {other}"),
        }
    }

    /// Bind an already-connected Unix peer to kernel socket credentials, an audit-token path,
    /// an immutable descriptor hash, an unchanged process-start identity, the frozen code
    /// requirement, and the frozen CDHash.
    pub fn verify_expected_coordinator(
        &self,
        peer_fd: BorrowedFd<'_>,
    ) -> Result<VerifiedCoordinatorProcess> {
        validate_connected_unix_stream(peer_fd.as_raw_fd())?;
        let (peer, audit_token) = peer_process_identity(peer_fd)?;
        if peer.effective_uid != EXPECTED_COORDINATOR_UID {
            bail!(
                "FD3 coordinator UID {} does not match frozen UID {}",
                peer.effective_uid,
                EXPECTED_COORDINATOR_UID
            )
        }
        if peer.effective_gid != EXPECTED_COORDINATOR_GID {
            bail!(
                "FD3 coordinator GID {} does not match frozen GID {}",
                peer.effective_gid,
                EXPECTED_COORDINATOR_GID
            )
        }
        if peer.canonical_account != EXPECTED_COORDINATOR_ACCOUNT {
            bail!(
                "FD3 coordinator account {:?} does not match frozen canonical account {:?}",
                peer.canonical_account,
                EXPECTED_COORDINATOR_ACCOUNT
            )
        }

        let expected_sha = decode_exact_hex(EXPECTED_COORDINATOR_SHA256, 32)
            .context("validate frozen coordinator SHA-256")?;
        let expected_cdhash = decode_exact_hex(EXPECTED_COORDINATOR_CDHASH, 20)
            .context("validate frozen coordinator CDHash")?;
        if EXPECTED_COORDINATOR_REQUIREMENT.contains('\0') {
            bail!("frozen coordinator requirement is invalid")
        }

        let expected_path = Path::new(MAC_R3_COORDINATOR_PATH_V2);
        let first = measure_process(&peer, &audit_token, expected_path)?;
        if first.executable_sha256.as_bytes() != hex_lower(&expected_sha).as_bytes() {
            bail!("FD3 coordinator executable SHA-256 does not match its frozen identity")
        }
        let observed_cdhash = verify_code_requirement_and_cdhash(
            &audit_token,
            EXPECTED_COORDINATOR_REQUIREMENT,
            &expected_cdhash,
            "FD3 coordinator",
        )?;
        let second = measure_process(&peer, &audit_token, expected_path)?;
        if first != second {
            bail!("FD3 coordinator process start/path/image changed during admission")
        }

        Ok(VerifiedCoordinatorProcess {
            peer,
            measurement: first,
            cdhash: observed_cdhash,
        })
    }

    /// Coordinator-side mutual attestation of the finalizer peer.  The expected executable
    /// identity must come from the already signature-validated receipt; this boundary additionally
    /// pins its fixed path and the finalizer's root account before consulting any receipt field.
    pub fn verify_expected_finalizer_peer(
        &self,
        peer_fd: BorrowedFd<'_>,
        signed_identity: &ExecutableIdentityV2,
    ) -> Result<VerifiedFinalizerPeerProcess> {
        validate_connected_unix_stream(peer_fd.as_raw_fd())?;
        let (peer, audit_token) = peer_process_identity(peer_fd)?;
        if peer.effective_uid != FINALIZER_UID || peer.canonical_account != FINALIZER_ACCOUNT {
            bail!("connected finalizer peer is not the fixed root account")
        }
        if signed_identity.intended_path != MAC_R3_FINALIZER_PATH_V2 {
            bail!("signed finalizer peer identity does not name the fixed finalizer path")
        }
        if signed_identity.designated_requirement.is_empty()
            || signed_identity.designated_requirement.contains('\0')
        {
            bail!("signed finalizer peer designated requirement is invalid")
        }
        let expected_sha = decode_exact_hex(&signed_identity.executable_sha256, 32)
            .context("validate signed finalizer peer SHA-256")?;
        let expected_cdhash = decode_exact_hex(&signed_identity.cdhash, 20)
            .context("validate signed finalizer peer CDHash")?;

        let expected_path = Path::new(MAC_R3_FINALIZER_PATH_V2);
        let first = measure_process(&peer, &audit_token, expected_path)?;
        if first.executable_sha256 != hex_lower(&expected_sha)
            || first.executable_file.size != signed_identity.executable_size
        {
            bail!("connected finalizer image does not match the signed receipt identity")
        }
        let observed_cdhash = verify_code_requirement_and_cdhash(
            &audit_token,
            &signed_identity.designated_requirement,
            &expected_cdhash,
            "connected finalizer peer",
        )?;
        let second = measure_process(&peer, &audit_token, expected_path)?;
        if first != second {
            bail!("finalizer peer process start/path/image changed during mutual attestation")
        }

        Ok(VerifiedFinalizerPeerProcess {
            peer,
            measurement: first,
            cdhash: observed_cdhash,
        })
    }

    /// Attest this process against the fixed finalizer path and the finalizer executable identity
    /// carried by the already signature-validated publisher receipt.  Textual requirement
    /// formatting is not compared: Security compiles and evaluates the signed designated
    /// requirement, which is the SDK-defined semantic comparison.
    pub fn verify_finalizer_self(
        &self,
        signed_identity: &ExecutableIdentityV2,
    ) -> Result<VerifiedFinalizerProcess> {
        if signed_identity.intended_path != MAC_R3_FINALIZER_PATH_V2 {
            bail!("signed finalizer identity does not name the fixed finalizer path")
        }
        if signed_identity.designated_requirement.is_empty()
            || signed_identity.designated_requirement.contains('\0')
        {
            bail!("signed finalizer designated requirement is invalid")
        }
        let expected_sha = decode_exact_hex(&signed_identity.executable_sha256, 32)
            .context("validate signed finalizer SHA-256")?;
        let expected_cdhash = decode_exact_hex(&signed_identity.cdhash, 20)
            .context("validate signed finalizer CDHash")?;

        // SAFETY: geteuid/getegid/getpid have no preconditions and return current process state.
        let effective_uid = unsafe { libc::geteuid() };
        // SAFETY: same process-local scalar query.
        let effective_gid = unsafe { libc::getegid() };
        // SAFETY: same process-local scalar query.
        let pid = unsafe { libc::getpid() };
        if effective_uid != FINALIZER_UID {
            bail!("finalizer is not running as its fixed effective UID")
        }

        let expected_path = Path::new(MAC_R3_FINALIZER_PATH_V2);
        let first = measure_self_process(pid, effective_uid, effective_gid, expected_path)?;
        if first.executable_sha256 != hex_lower(&expected_sha)
            || first.executable_file.size != signed_identity.executable_size
        {
            bail!("running finalizer image does not match the signed receipt identity")
        }
        let observed_cdhash = verify_self_code_requirement_and_cdhash(
            &signed_identity.designated_requirement,
            &expected_cdhash,
        )?;
        let second = measure_self_process(pid, effective_uid, effective_gid, expected_path)?;
        if first != second {
            bail!("finalizer process start/path/image changed during self-attestation")
        }
        Ok(VerifiedFinalizerProcess {
            measurement: first,
            cdhash: observed_cdhash,
        })
    }

    /// Strictly read the two internally derived disposable keys and receipt-bound target ACL before
    /// the engine begins its per-control Prepared/Invoked/Observed journal sequence.
    pub fn observe_disposable_capability_pre_probe(
        &self,
        receipt: &PublisherPreRemovalReceiptV2,
    ) -> Result<CanonicalDisposableCapabilityStateV2> {
        crate::disposable_capability::observe_pre_probe(self, receipt)
    }

    /// Invoke exactly one member of the closed disposable control sequence. The sealed module
    /// derives the fixed System-Keychain predicate, tags, path, and payload from the signed receipt.
    pub fn invoke_one_disposable_capability_control(
        &mut self,
        receipt: &PublisherPreRemovalReceiptV2,
        control: DisposableCapabilityControlV2,
    ) -> Result<CanonicalDisposableCapabilityInvocationV2> {
        crate::disposable_capability::invoke_one(self, receipt, control)
    }

    /// Observe both keys after one journaled control. Recovery from `Invoked` is deliberately
    /// ambiguous for sign/export because their success leaves no persistent marker; this method
    /// never reinvokes a control.
    pub fn observe_after_disposable_capability_control(
        &self,
        receipt: &PublisherPreRemovalReceiptV2,
        control: DisposableCapabilityControlV2,
    ) -> Result<CanonicalDisposableCapabilityAfterV2> {
        crate::disposable_capability::observe_after(self, receipt, control)
    }

    /// Read only the exact private P-256 key selected by one application tag in the one explicit
    /// System Keychain search list.  No fallback/default search list is used.
    pub fn exact_system_key_state(&self, application_tag: &[u8]) -> Result<ExactKeyState> {
        validate_application_tag(application_tag)?;
        // SAFETY: all raw objects are retained by `OwnedCf`; the query uses one fixed keychain
        // and a copied application-tag value.  No object escapes the call.
        unsafe {
            let mut owned = OwnedCf::new();
            let keychain = open_explicit_system_keychain(&mut owned)?;
            Ok(
                match exact_private_key_match(&mut owned, keychain, application_tag)? {
                    Some(_) => ExactKeyState::Present,
                    None => ExactKeyState::Absent,
                },
            )
        }
    }

    /// Return the unique exact signer's immutable key identity, P-256 public SPKI digest, fixed
    /// capability attributes, persistent identity, and strict receipt-bound legacy SecAccess
    /// digest. This read-only path cannot sign, export the private key, or mutate ACLs.
    pub fn exact_system_key_identity(
        &self,
        application_tag: &[u8],
        expected_access_sha256: &str,
        target_set_kind: substrate_common::macos_retirement_v2::TargetSetKindV2,
    ) -> Result<Option<KeyIntegrityObservationV2>> {
        validate_application_tag(application_tag)?;
        crate::disposable_capability::observe_exact_key(
            self,
            application_tag,
            expected_access_sha256,
            target_set_kind,
        )
    }

    /// Retain the one exact resolved SecKey and its full strict pre-observed identity. The opaque
    /// handle is the only input accepted by the identity-conditioned deletion method below.
    pub(crate) fn retain_exact_system_key_identity(
        &self,
        application_tag: &[u8],
        expected_access_sha256: &str,
        target_set_kind: substrate_common::macos_retirement_v2::TargetSetKindV2,
    ) -> Result<Option<RetainedExactSystemKey>> {
        validate_application_tag(application_tag)?;
        crate::disposable_capability::retain_exact_key(
            self,
            application_tag,
            expected_access_sha256,
            target_set_kind,
        )
    }

    /// Delete only the retained SecKey after a fresh same-reference/full-identity recheck, then
    /// require the same exact tag to be absent. No tag-only re-resolution can replace the handle.
    pub(crate) fn delete_retained_exact_system_key(
        &mut self,
        retained: RetainedExactSystemKey,
        expected_identity_sha256: &str,
    ) -> Result<ExactKeyDeleteOutcome> {
        crate::disposable_capability::delete_retained_exact_key(
            self,
            retained,
            expected_identity_sha256,
        )
    }

    /// Resolve and retain exactly one protected generic-password item by the already-derived
    /// service/account in the one explicit System Keychain. Every query carries UI-fail.
    pub(crate) fn retain_exact_generic_password_identity(
        &self,
        service: &str,
        account: &str,
    ) -> Result<Option<RetainedExactGenericPassword>> {
        validate_generic_password_field(service, "service")?;
        validate_generic_password_field(account, "account")?;
        // SAFETY: the owner retains the keychain, query result, item reference, and persistent
        // reference for the complete observe/invoke lifetime.
        unsafe {
            let mut owned = OwnedCf::new();
            let keychain = open_explicit_system_keychain(&mut owned)?;
            let Some((item, persistent_reference)) =
                exact_generic_password_match(&mut owned, keychain, service, account)?
            else {
                return Ok(None);
            };
            let persistent_reference_sha256 = sha256_cf_data(persistent_reference)?;
            let material = GenericPasswordIdentityMaterial {
                service,
                account,
                persistent_reference_sha256: &persistent_reference_sha256,
            };
            let identity_sha256 = sha256_hex_v2(&canonical_bytes_v2(&material)?);
            let identity = ExactGenericPasswordIdentityV2 {
                service: service.to_owned(),
                account: account.to_owned(),
                persistent_reference_sha256,
                identity_sha256,
            };
            Ok(Some(RetainedExactGenericPassword {
                owned,
                keychain,
                item,
                service: service.to_owned(),
                account: account.to_owned(),
                identity,
            }))
        }
    }

    /// Read the bytes of exactly one generic-password item from the explicit System Keychain.
    ///
    /// The first lookup requires one unique exact service/account match and returns attributes,
    /// data, and a persistent reference with query-level UI-fail. A second lookup is additionally
    /// conditioned on that persistent reference and must return the same bytes and reference. The
    /// raw item reference is deliberately not exposed, and this method has no mutation path.
    pub fn read_exact_system_generic_password_data(
        &self,
        service: &str,
        account: &str,
    ) -> Result<Option<Vec<u8>>> {
        validate_generic_password_field(service, "service")?;
        validate_generic_password_field(account, "account")?;
        // SAFETY: every query object and both create-rule results remain held until the stability
        // check is complete. Both lookups are exact, noninteractive, and read-only.
        unsafe {
            let mut owned = OwnedCf::new();
            let keychain = open_explicit_system_keychain(&mut owned)?;
            let Some(first) =
                exact_generic_password_data_match(&mut owned, keychain, service, account, None)?
            else {
                return Ok(None);
            };
            let second = exact_generic_password_data_match(
                &mut owned,
                keychain,
                service,
                account,
                Some(first.persistent_reference),
            )?
            .context("exact generic-password item disappeared during stable data read")?;
            if CFEqual(first.persistent_reference, second.persistent_reference) == 0
                || first.data != second.data
            {
                bail!("exact generic-password item or data changed during stable read")
            }
            Ok(Some(first.data))
        }
    }

    /// Delete only the retained generic-password reference after a fresh same-reference and
    /// persistent-reference recheck, then prove the exact service/account query is absent.
    pub(crate) fn delete_retained_exact_generic_password(
        &mut self,
        mut retained: RetainedExactGenericPassword,
        expected_identity_sha256: &str,
    ) -> Result<ExactGenericPasswordDeleteOutcome> {
        require_lower_sha256_literal(expected_identity_sha256, "generic-password identity")?;
        if retained.identity.identity_sha256 != expected_identity_sha256 {
            bail!("retained generic-password identity differs from conditioned deletion")
        }
        // SAFETY: all original and fresh exact query objects remain held by retained.
        unsafe {
            let current = exact_generic_password_match(
                &mut retained.owned,
                retained.keychain,
                &retained.service,
                &retained.account,
            )?
            .context("retained generic-password item disappeared before deletion")?;
            if CFEqual(current.0, retained.item) == 0
                || sha256_cf_data(current.1)? != retained.identity.persistent_reference_sha256
            {
                bail!("exact generic-password item was substituted after pre-observation")
            }
            let query = exact_generic_password_delete_query(
                &mut retained.owned,
                retained.keychain,
                retained.item,
                &retained.service,
                &retained.account,
            )?;
            let status = SecItemDelete(query.cast());
            if status != ERR_SEC_SUCCESS {
                return Err(keychain_error(
                    "delete exact protected generic-password item",
                    status,
                ));
            }
            if exact_generic_password_match(
                &mut retained.owned,
                retained.keychain,
                &retained.service,
                &retained.account,
            )?
            .is_some()
            {
                bail!("exact protected generic-password item remains after deletion")
            }
            Ok(ExactGenericPasswordDeleteOutcome::DeletedAndAbsent)
        }
    }
}

/// Activate exactly the launchd socket named `Listener` and require exactly one descriptor.
pub fn activate_listener_socket() -> Result<OwnedFd> {
    if LISTENER_SOCKET_NAME != "Listener" {
        bail!("compiled listener socket name drifted")
    }
    let mut descriptors: *mut c_int = ptr::null_mut();
    let mut count = 0_usize;
    // SAFETY: the NUL-terminated name and out-parameters satisfy launch.h.  The returned array is
    // released with free(3), as required by launch_activate_socket(3).
    let status = unsafe {
        launch_activate_socket(
            LISTENER_SOCKET_NAME_C.as_ptr().cast(),
            &mut descriptors,
            &mut count,
        )
    };
    if status != 0 {
        if !descriptors.is_null() {
            // SAFETY: any non-null launch_activate_socket allocation uses malloc(3).
            unsafe { libc::free(descriptors.cast()) };
        }
        bail!("launch_activate_socket(Listener) failed with errno {status}")
    }
    if descriptors.is_null() || count != 1 {
        if !descriptors.is_null() {
            // A successful multi-descriptor activation transferred those descriptors to us.  Do
            // not leak them when rejecting the widened listener set.
            for index in 0..count {
                // SAFETY: launchd returned an array of exactly `count` descriptors.
                let fd = unsafe { *descriptors.add(index) };
                if fd >= 0 {
                    // SAFETY: close is the required rejection cleanup for each transferred fd.
                    unsafe { libc::close(fd) };
                }
            }
            // SAFETY: launch.h assigns allocation ownership to the caller.
            unsafe { libc::free(descriptors.cast()) };
        }
        bail!("launch_activate_socket(Listener) returned {count} descriptors, expected exactly 1")
    }
    // SAFETY: the one array element is initialized on successful activation.
    let fd = unsafe { *descriptors };
    // SAFETY: launch.h assigns allocation ownership to the caller.
    unsafe { libc::free(descriptors.cast()) };
    if fd < 0 {
        bail!("launch_activate_socket(Listener) returned an invalid descriptor")
    }
    // SAFETY: successful activation transfers ownership of the descriptor to the caller.
    let listener = unsafe { OwnedFd::from_raw_fd(fd) };
    validate_listener_socket(listener.as_raw_fd())?;
    set_close_on_exec(listener.as_raw_fd())?;
    Ok(listener)
}

/// Accept exactly one peer, close the launchd listener, and normalize the accepted Unix stream to
/// FD3 without ever clobbering an unrelated descriptor.
pub fn accept_listener_peer_on_fd3(listener: OwnedFd) -> Result<OwnedFd> {
    validate_listener_socket(listener.as_raw_fd())?;
    let accepted_fd = loop {
        // SAFETY: listener is a live owned listening descriptor.  The peer address is not used.
        let fd = unsafe { libc::accept(listener.as_raw_fd(), ptr::null_mut(), ptr::null_mut()) };
        if fd >= 0 {
            break fd;
        }
        let error = std::io::Error::last_os_error();
        if error.kind() != std::io::ErrorKind::Interrupted {
            return Err(error).context("accept exact launchd Listener peer");
        }
    };
    // SAFETY: accept transferred a new descriptor to this process.
    let accepted = unsafe { OwnedFd::from_raw_fd(accepted_fd) };
    set_close_on_exec(accepted.as_raw_fd())?;
    validate_connected_unix_stream(accepted.as_raw_fd())?;
    // Re-read both the listener's kernel SockPathName and the physical endpoint after accept,
    // before releasing the listener descriptor.
    validate_listener_socket(listener.as_raw_fd())?;

    // launchd commonly supplies the listener on FD3.  It must be closed before the accepted
    // connection is normalized to FD3.
    drop(listener);
    normalize_accepted_socket_to_fd3(accepted)
}

/// Normalize an already-accepted Unix stream to FD3.  This is public for a coordinator that does
/// the accept loop itself, but it refuses non-Unix/non-stream/unconnected descriptors.
pub fn normalize_accepted_socket_to_fd3(accepted: OwnedFd) -> Result<OwnedFd> {
    let source_fd = accepted.as_raw_fd();
    validate_connected_unix_stream(source_fd)?;
    let fd3_is_open = if source_fd == ACCEPTED_SOCKET_FD {
        true
    } else {
        descriptor_is_open(ACCEPTED_SOCKET_FD)?
    };
    match classify_fd3_plan(source_fd, fd3_is_open)? {
        Fd3Plan::AlreadyFd3 => {
            set_close_on_exec(source_fd)?;
            Ok(accepted)
        }
        Fd3Plan::DuplicateToFd3 => {
            // SAFETY: FD3 was observed closed and source_fd is owned.  dup2 atomically installs a
            // duplicate at the exact destination.  The finalizer is single-threaded during this
            // pre-frame admission step, so no other thread can race an FD3 allocation.
            let duplicated = unsafe { libc::dup2(source_fd, ACCEPTED_SOCKET_FD) };
            if duplicated != ACCEPTED_SOCKET_FD {
                return Err(std::io::Error::last_os_error())
                    .context("normalize accepted launchd socket to FD3");
            }
            // SAFETY: dup2 created a new descriptor independent of the source descriptor.
            let fd3 = unsafe { OwnedFd::from_raw_fd(ACCEPTED_SOCKET_FD) };
            set_close_on_exec(fd3.as_raw_fd())?;
            drop(accepted);
            Ok(fd3)
        }
    }
}

fn descriptor_is_open(fd: RawFd) -> Result<bool> {
    // SAFETY: F_GETFD only reads descriptor-table state.
    let result = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if result >= 0 {
        return Ok(true);
    }
    let error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::EBADF) {
        Ok(false)
    } else {
        Err(error).context("inspect FD3 occupancy")
    }
}

fn set_close_on_exec(fd: RawFd) -> Result<()> {
    // SAFETY: F_GETFD/F_SETFD operate on the supplied live descriptor only.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 {
        return Err(std::io::Error::last_os_error()).context("read descriptor flags");
    }
    // SAFETY: the flags are the preceding F_GETFD result with FD_CLOEXEC added.
    if unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0 {
        return Err(std::io::Error::last_os_error()).context("set descriptor close-on-exec");
    }
    Ok(())
}

fn socket_option_i32(fd: RawFd, level: c_int, option: c_int) -> Result<c_int> {
    let mut value: c_int = 0;
    let mut length = size_of::<c_int>() as libc::socklen_t;
    // SAFETY: value/length describe a correctly sized writable c_int result.
    if unsafe {
        libc::getsockopt(
            fd,
            level,
            option,
            (&mut value as *mut c_int).cast(),
            &mut length,
        )
    } < 0
    {
        return Err(std::io::Error::last_os_error()).context("read socket option");
    }
    if length as usize != size_of::<c_int>() {
        bail!("socket option returned an unexpected length")
    }
    Ok(value)
}

fn unix_socket_family(fd: RawFd, peer: bool) -> Result<c_int> {
    let mut address = MaybeUninit::<libc::sockaddr_storage>::zeroed();
    let mut length = size_of::<libc::sockaddr_storage>() as libc::socklen_t;
    // SAFETY: address and length describe a full writable sockaddr_storage.
    let result = unsafe {
        if peer {
            libc::getpeername(fd, address.as_mut_ptr().cast(), &mut length)
        } else {
            libc::getsockname(fd, address.as_mut_ptr().cast(), &mut length)
        }
    };
    if result < 0 {
        return Err(std::io::Error::last_os_error()).with_context(|| {
            if peer {
                "read accepted socket peer address"
            } else {
                "read listener socket address"
            }
        });
    }
    if length as usize > size_of::<libc::sockaddr_storage>()
        || (length as usize) < size_of::<libc::sa_family_t>()
    {
        bail!("Unix socket address returned an invalid length")
    }
    // SAFETY: the successful call initialized at least the family field.
    let address = unsafe { address.assume_init() };
    Ok(c_int::from(address.ss_family))
}

fn exact_listener_socket_path(fd: RawFd) -> Result<()> {
    let mut address = MaybeUninit::<libc::sockaddr_un>::zeroed();
    let mut length = size_of::<libc::sockaddr_un>() as libc::socklen_t;
    // SAFETY: address and length describe a full writable sockaddr_un.
    if unsafe { libc::getsockname(fd, address.as_mut_ptr().cast(), &mut length) } < 0 {
        return Err(std::io::Error::last_os_error()).context("read Listener SockPathName");
    }
    if length as usize > size_of::<libc::sockaddr_un>() || length < 3 {
        bail!("Listener SockPathName returned an invalid sockaddr_un length")
    }
    // SAFETY: getsockname initialized the returned sockaddr_un bytes.
    let address = unsafe { address.assume_init() };
    if c_int::from(address.sun_family) != libc::AF_UNIX
        || usize::from(address.sun_len) != length as usize
    {
        bail!("Listener SockPathName is not one exact Unix pathname socket")
    }
    // SAFETY: sun_path is an initialized fixed array within address.
    let path_bytes = unsafe {
        std::slice::from_raw_parts(
            address.sun_path.as_ptr().cast::<u8>(),
            address.sun_path.len(),
        )
    };
    // Darwin's public SUN_LEN macro defines the bound address length as the two-byte
    // sun_len/sun_family prefix plus strlen(sun_path), excluding the trailing NUL.
    let expected = MAC_R3_FINALIZER_ENDPOINT_V2.as_bytes();
    if expected.len() > path_bytes.len()
        || length as usize != 2 + expected.len()
        || &path_bytes[..expected.len()] != expected
    {
        bail!("Listener SockPathName does not match the fixed finalizer endpoint")
    }
    Ok(())
}

fn validate_listener_endpoint_identity() -> Result<()> {
    let path = Path::new(MAC_R3_FINALIZER_ENDPOINT_V2);
    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspect Listener endpoint {}", path.display()))?;
    if metadata.file_type().is_symlink()
        || !metadata.file_type().is_socket()
        || metadata.uid() != ENDPOINT_OWNER_UID
        || metadata.gid() != ENDPOINT_GROUP_GID
        || metadata.mode() & 0o777 != ENDPOINT_MODE
    {
        bail!("Listener endpoint physical owner/group/mode/type identity is not exact")
    }
    Ok(())
}

fn validate_listener_socket(fd: RawFd) -> Result<()> {
    if socket_option_i32(fd, libc::SOL_SOCKET, libc::SO_TYPE)? != libc::SOCK_STREAM
        || socket_option_i32(fd, libc::SOL_SOCKET, libc::SO_ACCEPTCONN)? != 1
        || unix_socket_family(fd, false)? != libc::AF_UNIX
    {
        bail!("launch_activate_socket(Listener) did not return one listening Unix stream")
    }
    exact_listener_socket_path(fd)?;
    validate_listener_endpoint_identity()?;
    Ok(())
}

fn validate_connected_unix_stream(fd: RawFd) -> Result<()> {
    if socket_option_i32(fd, libc::SOL_SOCKET, libc::SO_TYPE)? != libc::SOCK_STREAM
        || unix_socket_family(fd, true)? != libc::AF_UNIX
    {
        bail!("accepted descriptor is not a connected Unix stream")
    }
    Ok(())
}

fn canonical_account_for_uid(uid: libc::uid_t) -> Result<String> {
    // SAFETY: sysconf is a scalar query with no pointer arguments.
    let recommended = unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) };
    let mut capacity = if recommended > 0 {
        usize::try_from(recommended)
            .unwrap_or(DEFAULT_PASSWD_BUFFER_BYTES)
            .min(MAX_PASSWD_BUFFER_BYTES)
    } else {
        DEFAULT_PASSWD_BUFFER_BYTES
    };
    loop {
        let mut entry = MaybeUninit::<libc::passwd>::zeroed();
        let entry_pointer = entry.as_mut_ptr();
        let mut result: *mut libc::passwd = ptr::null_mut();
        let mut buffer = vec![0_u8; capacity];
        // SAFETY: entry, buffer, and result are writable for the documented getpwuid_r outputs.
        let status = unsafe {
            libc::getpwuid_r(
                uid,
                entry_pointer,
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                &mut result,
            )
        };
        if status == libc::ERANGE && capacity < MAX_PASSWD_BUFFER_BYTES {
            capacity = (capacity * 2).min(MAX_PASSWD_BUFFER_BYTES);
            continue;
        }
        if status != 0 {
            bail!("getpwuid_r({uid}) failed with errno {status}")
        }
        if result.is_null() {
            bail!("getpwuid_r({uid}) returned no canonical account")
        }
        if result != entry_pointer {
            bail!("getpwuid_r({uid}) returned an unexpected passwd pointer")
        }
        // SAFETY: successful getpwuid_r initialized the passwd structure and pw_name points into
        // buffer, which remains live until after the string is copied.
        let entry = unsafe { entry.assume_init() };
        if entry.pw_uid != uid || entry.pw_name.is_null() {
            bail!("getpwuid_r({uid}) returned mismatched account identity")
        }
        // SAFETY: getpwuid_r guarantees a NUL-terminated pw_name in the caller's live buffer.
        let name = unsafe { CStr::from_ptr(entry.pw_name) }
            .to_str()
            .context("canonical account is not UTF-8")?;
        if name.is_empty() {
            bail!("getpwuid_r({uid}) returned an empty canonical account")
        }
        return Ok(name.to_owned());
    }
}

fn peer_process_identity(fd: BorrowedFd<'_>) -> Result<(PeerProcessIdentity, AuditToken)> {
    let socket_pid = socket_option_i32(fd.as_raw_fd(), SOL_LOCAL, LOCAL_PEERPID)?;
    let mut socket_euid: libc::uid_t = 0;
    let mut socket_egid: libc::gid_t = 0;
    // SAFETY: both output pointers are live and getpeereid only reads peer credentials.
    if unsafe { getpeereid(fd.as_raw_fd(), &mut socket_euid, &mut socket_egid) } < 0 {
        return Err(std::io::Error::last_os_error()).context("read getpeereid credentials");
    }

    let mut audit_token = AuditToken { values: [0; 8] };
    let mut token_length = size_of::<AuditToken>() as libc::socklen_t;
    // SAFETY: audit_token and token_length describe the exact LOCAL_PEERTOKEN result buffer.
    if unsafe {
        libc::getsockopt(
            fd.as_raw_fd(),
            SOL_LOCAL,
            LOCAL_PEERTOKEN,
            (&mut audit_token as *mut AuditToken).cast(),
            &mut token_length,
        )
    } < 0
    {
        return Err(std::io::Error::last_os_error()).context("read LOCAL_PEERTOKEN");
    }
    if token_length as usize != size_of::<AuditToken>() {
        bail!("LOCAL_PEERTOKEN returned an unexpected length")
    }

    // SAFETY: the BSM APIs are the SDK-prescribed parsers for an initialized audit_token_t.
    let token_pid = unsafe { audit_token_to_pid(audit_token) };
    // SAFETY: same initialized audit token.
    let token_euid = unsafe { audit_token_to_euid(audit_token) };
    // SAFETY: same initialized audit token.
    let token_egid = unsafe { audit_token_to_egid(audit_token) };
    reconcile_peer_identity(
        socket_pid,
        socket_euid,
        socket_egid,
        token_pid,
        token_euid,
        token_egid,
    )?;
    let canonical_account = canonical_account_for_uid(socket_euid)?;
    Ok((
        PeerProcessIdentity {
            pid: socket_pid,
            effective_uid: socket_euid,
            effective_gid: socket_egid,
            canonical_account,
            audit_token: audit_token.values,
        },
        audit_token,
    ))
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

fn process_start_identity(peer: &PeerProcessIdentity) -> Result<ProcessStartIdentity> {
    process_start_identity_exact(peer.pid, peer.effective_uid, peer.effective_gid)
}

fn process_start_identity_exact(
    pid: libc::pid_t,
    effective_uid: libc::uid_t,
    effective_gid: libc::gid_t,
) -> Result<ProcessStartIdentity> {
    let mut info = MaybeUninit::<ProcBsdInfo>::zeroed();
    // SAFETY: info is a writable buffer with the exact PROC_PIDTBSDINFO SDK layout.
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
        return Err(std::io::Error::last_os_error()).context("read coordinator process start");
    }
    if returned as usize != size_of::<ProcBsdInfo>() {
        bail!("PROC_PIDTBSDINFO returned a truncated process identity")
    }
    // SAFETY: proc_pidinfo returned the full structure size.
    let info = unsafe { info.assume_init() };
    if info.pbi_pid != pid as u32
        || info.pbi_uid != effective_uid
        || info.pbi_gid != effective_gid
        || info.pbi_start_tvusec >= 1_000_000
    {
        bail!("process-table identity disagrees with the FD3 peer")
    }
    Ok(ProcessStartIdentity {
        seconds: info.pbi_start_tvsec,
        microseconds: info.pbi_start_tvusec,
    })
}

fn pid_process_path(pid: libc::pid_t) -> Result<PathBuf> {
    let mut buffer = vec![0_u8; PROC_PIDPATHINFO_MAXSIZE];
    // SAFETY: buffer is writable for the full fixed length and pid is this live process.
    let length = unsafe { proc_pidpath(pid, buffer.as_mut_ptr().cast(), buffer.len() as u32) };
    if length <= 0 || length as usize >= buffer.len() {
        return Err(std::io::Error::last_os_error())
            .context("resolve finalizer executable from its live PID");
    }
    buffer.truncate(length as usize);
    if buffer.contains(&0) {
        bail!("PID process path contains an embedded NUL")
    }
    let path = PathBuf::from(OsString::from_vec(buffer));
    if !path.is_absolute() {
        bail!("PID process path is not absolute")
    }
    Ok(path)
}

fn audit_bound_process_path(audit_token: &AuditToken) -> Result<PathBuf> {
    let mut buffer = vec![0_u8; PROC_PIDPATHINFO_MAXSIZE];
    let mut token = *audit_token;
    // SAFETY: token is an initialized LOCAL_PEERTOKEN and buffer is writable for its full length.
    let length = unsafe {
        proc_pidpath_audittoken(&mut token, buffer.as_mut_ptr().cast(), buffer.len() as u32)
    };
    if length <= 0 || length as usize >= buffer.len() {
        return Err(std::io::Error::last_os_error())
            .context("resolve coordinator executable from LOCAL_PEERTOKEN");
    }
    buffer.truncate(length as usize);
    if buffer.contains(&0) {
        bail!("audit-token process path contains an embedded NUL")
    }
    let path = PathBuf::from(OsString::from_vec(buffer));
    if !path.is_absolute() {
        bail!("audit-token process path is not absolute")
    }
    Ok(path)
}

fn require_root_owned_immutable_path(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        bail!("expected executable path is not absolute")
    }
    let mut current = PathBuf::from("/");
    let mut components = path.components();
    let _ = components.next();
    for component in components {
        current.push(component.as_os_str());
        let metadata = std::fs::symlink_metadata(&current)
            .with_context(|| format!("inspect executable path component {}", current.display()))?;
        if metadata.file_type().is_symlink() || metadata.uid() != 0 || metadata.mode() & 0o022 != 0
        {
            bail!(
                "executable path component {} is not root-owned immutable no-follow state",
                current.display()
            )
        }
        if current == path {
            if !metadata.file_type().is_file() || metadata.nlink() != 1 {
                bail!("expected executable is not one regular-file identity")
            }
        } else if !metadata.is_dir() {
            bail!("expected executable path parent is not a directory")
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

fn hash_immutable_executable(path: &Path) -> Result<(ExecutableFileIdentity, String)> {
    require_root_owned_immutable_path(path)?;
    let encoded =
        CString::new(path.as_os_str().as_bytes()).context("encode fixed executable path")?;
    // SAFETY: encoded is NUL-terminated; O_NOFOLLOW prevents terminal symlink traversal.
    let fd = unsafe {
        libc::open(
            encoded.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("open fixed executable image {}", path.display()));
    }
    // SAFETY: open transferred ownership of this descriptor.
    let mut file = unsafe { File::from_raw_fd(fd) };
    let before_metadata = file
        .metadata()
        .context("fstat fixed executable image before hash")?;
    let before = file_identity(&before_metadata);
    if before.owner_uid != 0
        || before.mode & 0o022 != 0
        || before.link_count != 1
        || before.mode & u32::from(libc::S_IFMT) != u32::from(libc::S_IFREG)
    {
        bail!("opened executable image is not one root-owned immutable regular file")
    }
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .context("hash fixed executable descriptor")?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    let after = file_identity(
        &file
            .metadata()
            .context("fstat fixed executable image after hash")?,
    );
    if before != after {
        bail!("fixed executable descriptor changed while hashing")
    }
    require_root_owned_immutable_path(path)?;
    Ok((before, hex_lower(&digest.finalize())))
}

fn measure_process(
    peer: &PeerProcessIdentity,
    audit_token: &AuditToken,
    expected_path: &Path,
) -> Result<ProcessMeasurement> {
    let start = process_start_identity(peer)?;
    let executable_path = audit_bound_process_path(audit_token)?;
    if executable_path.as_os_str().as_bytes() != expected_path.as_os_str().as_bytes() {
        bail!("FD3 coordinator executable path does not match the fixed path")
    }
    let (executable_file, executable_sha256) = hash_immutable_executable(expected_path)?;
    if process_start_identity(peer)? != start
        || audit_bound_process_path(audit_token)? != executable_path
    {
        bail!("FD3 coordinator process identity changed during image measurement")
    }
    Ok(ProcessMeasurement {
        start,
        executable_path,
        executable_file,
        executable_sha256,
    })
}

fn measure_self_process(
    pid: libc::pid_t,
    effective_uid: libc::uid_t,
    effective_gid: libc::gid_t,
    expected_path: &Path,
) -> Result<ProcessMeasurement> {
    let start = process_start_identity_exact(pid, effective_uid, effective_gid)?;
    let executable_path = pid_process_path(pid)?;
    if executable_path.as_os_str().as_bytes() != expected_path.as_os_str().as_bytes() {
        bail!("running finalizer executable path does not match the fixed path")
    }
    let (executable_file, executable_sha256) = hash_immutable_executable(expected_path)?;
    if process_start_identity_exact(pid, effective_uid, effective_gid)? != start
        || pid_process_path(pid)? != executable_path
    {
        bail!("finalizer process identity changed during image measurement")
    }
    Ok(ProcessMeasurement {
        start,
        executable_path,
        executable_file,
        executable_sha256,
    })
}

/// Decode one exact, lowercase, nonzero fixed-length hexadecimal literal.
pub fn decode_exact_hex(value: &str, expected_bytes: usize) -> Result<Vec<u8>> {
    if value.len() != expected_bytes * 2
        || !value
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
    {
        bail!("identity literal is not exact lowercase hexadecimal")
    }
    let mut decoded = Vec::with_capacity(expected_bytes);
    for pair in value.as_bytes().chunks_exact(2) {
        let high = hex_nibble(pair[0])?;
        let low = hex_nibble(pair[1])?;
        decoded.push((high << 4) | low);
    }
    if decoded.iter().all(|byte| *byte == 0) {
        bail!("identity literal must not be the all-zero build placeholder")
    }
    Ok(decoded)
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

fn verify_code_requirement_and_cdhash(
    audit_token: &AuditToken,
    requirement_text: &str,
    expected_cdhash: &[u8],
    subject: &str,
) -> Result<Vec<u8>> {
    // SAFETY: every created CF/Security object is retained by owned until after the last use.
    unsafe {
        let mut owned = OwnedCf::new();
        let audit_bytes = std::slice::from_raw_parts(
            (audit_token as *const AuditToken).cast::<u8>(),
            size_of::<AuditToken>(),
        );
        let audit_data = cf_data(&mut owned, audit_bytes)?;
        let attributes = dictionary(&mut owned, &[(kSecGuestAttributeAudit, audit_data.cast())])?;
        let mut code: SecCode = ptr::null();
        let status = SecCodeCopyGuestWithAttributes(ptr::null(), attributes.cast(), 0, &mut code);
        if status != ERR_SEC_SUCCESS || code.is_null() {
            return Err(keychain_error(
                "resolve FD3 peer SecCode from LOCAL_PEERTOKEN",
                status,
            ));
        }
        owned.hold(code);
        validate_resolved_code_requirement_and_cdhash(
            &mut owned,
            code,
            requirement_text,
            expected_cdhash,
            subject,
        )
    }
}

fn verify_self_code_requirement_and_cdhash(
    requirement_text: &str,
    expected_cdhash: &[u8],
) -> Result<Vec<u8>> {
    // SAFETY: SecCodeCopySelf returns a create-rule reference retained by owned.
    unsafe {
        let mut owned = OwnedCf::new();
        let mut code: SecCode = ptr::null();
        let status = SecCodeCopySelf(0, &mut code);
        if status != ERR_SEC_SUCCESS || code.is_null() {
            return Err(keychain_error("resolve running finalizer SecCode", status));
        }
        owned.hold(code);
        validate_resolved_code_requirement_and_cdhash(
            &mut owned,
            code,
            requirement_text,
            expected_cdhash,
            "running finalizer",
        )
    }
}

unsafe fn validate_resolved_code_requirement_and_cdhash(
    owned: &mut OwnedCf,
    code: SecCode,
    requirement_text: &str,
    expected_cdhash: &[u8],
    subject: &str,
) -> Result<Vec<u8>> {
    // SAFETY: created string is retained by owned.
    let requirement_string = unsafe { cf_string(owned, requirement_text)? };
    let mut requirement: SecRequirement = ptr::null();
    // SAFETY: inputs are live and requirement is a writable create-rule out pointer.
    let status = unsafe { SecRequirementCreateWithString(requirement_string, 0, &mut requirement) };
    if status != ERR_SEC_SUCCESS || requirement.is_null() {
        return Err(keychain_error(
            &format!("compile {subject} code requirement"),
            status,
        ));
    }
    owned.hold(requirement);
    // SAFETY: code and requirement are live Security objects.
    let status = unsafe { SecCodeCheckValidity(code, 0, requirement) };
    if status != ERR_SEC_SUCCESS {
        return Err(keychain_error(
            &format!("validate {subject} code requirement"),
            status,
        ));
    }

    let mut signing_info: CfType = ptr::null();
    // SAFETY: code is valid and signing_info is a writable create-rule out pointer.
    let status = unsafe {
        SecCodeCopySigningInformation(
            code,
            K_SEC_CS_SIGNING_AND_REQUIREMENT_INFORMATION,
            &mut signing_info,
        )
    };
    if status != ERR_SEC_SUCCESS || signing_info.is_null() {
        return Err(keychain_error(
            &format!("copy validated {subject} signing information"),
            status,
        ));
    }
    owned.hold(signing_info);
    // SAFETY: signing_info is a live CF object.
    if unsafe { CFGetTypeID(signing_info) } != unsafe { CFDictionaryGetTypeID() } {
        bail!("{subject} SecCode signing information is not a dictionary")
    }
    // SAFETY: signing_info is a validated dictionary and the key is a framework static.
    let unique = unsafe { CFDictionaryGetValue(signing_info, kSecCodeInfoUnique) };
    // SAFETY: a non-null unique is a live CF object retained by signing_info.
    if unique.is_null() || unsafe { CFGetTypeID(unique) } != unsafe { CFDataGetTypeID() } {
        bail!("{subject} SecCode signing information has no CDHash")
    }
    // SAFETY: unique has been validated as CFData.
    let length = unsafe { CFDataGetLength(unique.cast()) };
    // SAFETY: same live CFData.
    let bytes = unsafe { CFDataGetBytePtr(unique.cast()) };
    if length < 0 || bytes.is_null() {
        bail!("{subject} SecCode CDHash data is invalid")
    }
    // SAFETY: bytes is valid for the CFData length while signing_info remains held.
    let observed = unsafe { std::slice::from_raw_parts(bytes, length as usize) }.to_vec();
    if observed.as_slice() != expected_cdhash {
        bail!("{subject} CDHash does not match its exact expected identity")
    }
    // SAFETY: signing_info is a validated dictionary and all keys are framework statics.
    let flags = unsafe { CFDictionaryGetValue(signing_info, kSecCodeInfoFlags) };
    // SAFETY: a non-null flags value is retained by signing_info.
    if flags.is_null() || unsafe { CFGetTypeID(flags) } != unsafe { CFNumberGetTypeID() } {
        bail!("{subject} SecCode signing information has no CodeDirectory flags")
    }
    let mut signed_flags = 0_i64;
    // SAFETY: flags is a live CFNumber and signed_flags is a writable i64.
    if unsafe {
        CFNumberGetValue(
            flags,
            K_CF_NUMBER_SINT64,
            (&mut signed_flags as *mut i64).cast(),
        )
    } == 0
        || signed_flags < 0
        || signed_flags > i64::from(u32::MAX)
    {
        bail!("{subject} CodeDirectory flags are not a bounded unsigned value")
    }
    // SAFETY: signing_info is a live dictionary and the values, when present, are retained by it.
    let has_team_identifier =
        !unsafe { CFDictionaryGetValue(signing_info, kSecCodeInfoTeamIdentifier) }.is_null();
    // SAFETY: same retained signing information dictionary.
    let has_entitlements_blob =
        !unsafe { CFDictionaryGetValue(signing_info, kSecCodeInfoEntitlements) }.is_null();
    // SAFETY: same retained signing information dictionary.
    let has_entitlements_dictionary =
        !unsafe { CFDictionaryGetValue(signing_info, kSecCodeInfoEntitlementsDict) }.is_null();
    validate_frozen_code_posture(
        signed_flags as u32,
        has_team_identifier,
        has_entitlements_blob,
        has_entitlements_dictionary,
    )
    .with_context(|| format!("validate {subject} dynamic-loader posture"))?;
    Ok(observed)
}

fn validate_application_tag(application_tag: &[u8]) -> Result<()> {
    if application_tag.is_empty() || application_tag.len() > MAX_APPLICATION_TAG_BYTES {
        bail!("System-Keychain application tag is empty or exceeds its fixed bound")
    }
    Ok(())
}

fn validate_generic_password_field(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > MAX_GENERIC_PASSWORD_FIELD_BYTES
        || value.contains('\0')
        || value.contains(['\n', '\r'])
    {
        bail!("generic-password {label} is empty, unsafe, or exceeds its fixed bound")
    }
    Ok(())
}

fn require_lower_sha256_literal(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is not one lowercase SHA-256")
    }
    Ok(())
}

struct OwnedCf(Vec<CfType>);

impl OwnedCf {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn hold<T>(&mut self, value: *const T) -> *const T {
        if !value.is_null() {
            self.0.push(value.cast());
        }
        value
    }
}

impl Drop for OwnedCf {
    fn drop(&mut self) {
        // SAFETY: each entry is a create/copy-rule CF object held exactly once by this owner.
        unsafe {
            for value in self.0.drain(..).rev() {
                CFRelease(value);
            }
        }
    }
}

unsafe fn cf_string(owned: &mut OwnedCf, value: &str) -> Result<CfType> {
    // SAFETY: the byte pointer is valid for the specified length; UTF-8 encoding is explicit.
    let result = unsafe {
        CFStringCreateWithBytes(
            kCFAllocatorDefault,
            value.as_bytes().as_ptr(),
            value.len() as isize,
            0x0800_0100,
            0,
        )
    };
    if result.is_null() {
        bail!("allocate CoreFoundation UTF-8 string")
    }
    Ok(owned.hold(result).cast())
}

unsafe fn cf_data(owned: &mut OwnedCf, value: &[u8]) -> Result<CfType> {
    // SAFETY: the byte pointer is valid for the specified length.
    let result = unsafe { CFDataCreate(kCFAllocatorDefault, value.as_ptr(), value.len() as isize) };
    if result.is_null() {
        bail!("allocate CoreFoundation data")
    }
    Ok(owned.hold(result).cast())
}

unsafe fn cf_array(owned: &mut OwnedCf, values: &[CfType]) -> Result<CfType> {
    // SAFETY: values remains live through the call; individual objects remain held by owned.
    let array = unsafe {
        CFArrayCreate(
            kCFAllocatorDefault,
            values.as_ptr(),
            values.len() as isize,
            ptr::null(),
        )
    };
    if array.is_null() {
        bail!("allocate CoreFoundation array")
    }
    Ok(owned.hold(array).cast())
}

unsafe fn dictionary(
    owned: &mut OwnedCf,
    entries: &[(CfType, CfType)],
) -> Result<CfMutableDictionary> {
    // SAFETY: allocator and callbacks match the established product FFI pattern.  The caller's
    // OwnedCf retains every dynamic entry for at least as long as the dictionary.
    let result = unsafe {
        CFDictionaryCreateMutable(
            kCFAllocatorDefault,
            entries.len() as isize,
            ptr::null(),
            ptr::null(),
        )
    };
    if result.is_null() {
        bail!("allocate CoreFoundation dictionary")
    }
    owned.hold(result);
    for (key, value) in entries {
        // SAFETY: result is mutable and both raw values remain live in OwnedCf or framework
        // static storage.
        unsafe { CFDictionarySetValue(result, *key, *value) };
    }
    Ok(result)
}

unsafe fn open_explicit_system_keychain(owned: &mut OwnedCf) -> Result<SecKeychain> {
    let path = CString::new(SYSTEM_KEYCHAIN_PATH).context("encode fixed System Keychain path")?;
    let mut keychain: SecKeychain = ptr::null();
    // SAFETY: path is NUL-terminated and keychain is a writable result pointer.
    let status = unsafe { SecKeychainOpen(path.as_ptr(), &mut keychain) };
    if status != ERR_SEC_SUCCESS || keychain.is_null() {
        return Err(keychain_error("open explicit System Keychain", status));
    }
    owned.hold(keychain);

    let mut returned = [0 as c_char; MAX_SYSTEM_KEYCHAIN_PATH_BYTES];
    let mut length = u32::try_from(returned.len()).context("bound System Keychain path")?;
    // SAFETY: keychain is live and returned is writable for length bytes.
    let status = unsafe { SecKeychainGetPath(keychain, &mut length, returned.as_mut_ptr()) };
    if status != ERR_SEC_SUCCESS {
        return Err(keychain_error("read explicit System Keychain path", status));
    }
    let length = usize::try_from(length).context("decode System Keychain path length")?;
    if length >= returned.len() {
        bail!("explicit System Keychain returned an unterminated path")
    }
    // SAFETY: Security initialized the first `length` bytes.
    let path_bytes: &[u8] = unsafe { std::slice::from_raw_parts(returned.as_ptr().cast(), length) };
    if path_bytes != SYSTEM_KEYCHAIN_PATH.as_bytes() || returned[length] != 0 {
        bail!("explicit System Keychain identity does not match its fixed path")
    }
    Ok(keychain)
}

unsafe fn single_keychain_search_list(
    owned: &mut OwnedCf,
    keychain: SecKeychain,
) -> Result<CfType> {
    // SAFETY: keychain remains owned by the same OwnedCf.
    unsafe { cf_array(owned, &[keychain.cast()]) }
}

unsafe fn exact_private_key_query(
    owned: &mut OwnedCf,
    keychain: SecKeychain,
    application_tag: &[u8],
) -> Result<CfMutableDictionary> {
    // SAFETY: each created object is immediately retained by owned.
    let tag = unsafe { cf_data(owned, application_tag)? };
    // SAFETY: keychain remains held by owned.
    let search_list = unsafe { single_keychain_search_list(owned, keychain)? };
    // SAFETY: all keys are framework statics and all values outlive the dictionary.
    unsafe {
        dictionary(
            owned,
            &[
                (kSecClass, kSecClassKey),
                (kSecAttrApplicationTag, tag),
                (kSecMatchSearchList, search_list),
                (kSecUseAuthenticationUI, kSecUseAuthenticationUIFail),
                (kSecReturnAttributes, kCFBooleanTrue),
                (kSecReturnRef, kCFBooleanTrue),
                (kSecMatchLimit, kSecMatchLimitAll),
            ],
        )
    }
}

unsafe fn exact_generic_password_query(
    owned: &mut OwnedCf,
    keychain: SecKeychain,
    service: &str,
    account: &str,
) -> Result<CfMutableDictionary> {
    // SAFETY: created values are retained by owned.
    let service = unsafe { cf_string(owned, service)? };
    // SAFETY: same ownership.
    let account = unsafe { cf_string(owned, account)? };
    // SAFETY: keychain remains held by owned.
    let search_list = unsafe { single_keychain_search_list(owned, keychain)? };
    // SAFETY: all keys are framework statics and all values outlive the dictionary.
    unsafe {
        dictionary(
            owned,
            &[
                (kSecClass, kSecClassGenericPassword),
                (kSecAttrService, service),
                (kSecAttrAccount, account),
                (kSecMatchSearchList, search_list),
                (kSecUseAuthenticationUI, kSecUseAuthenticationUIFail),
                (kSecReturnAttributes, kCFBooleanTrue),
                (kSecReturnRef, kCFBooleanTrue),
                (kSecReturnPersistentRef, kCFBooleanTrue),
                (kSecMatchLimit, kSecMatchLimitAll),
            ],
        )
    }
}

unsafe fn exact_generic_password_data_query(
    owned: &mut OwnedCf,
    keychain: SecKeychain,
    service: &str,
    account: &str,
    persistent_reference: Option<CfType>,
) -> Result<CfMutableDictionary> {
    // SAFETY: created values are retained by owned.
    let service = unsafe { cf_string(owned, service)? };
    // SAFETY: same ownership.
    let account = unsafe { cf_string(owned, account)? };
    // SAFETY: keychain remains held by owned.
    let search_list = unsafe { single_keychain_search_list(owned, keychain)? };
    let item_list = if let Some(persistent_reference) = persistent_reference {
        // SAFETY: the first create-rule query result remains held by owned, so its persistent
        // reference remains live through the identity-conditioned second query.
        Some(unsafe { cf_array(owned, &[persistent_reference])? })
    } else {
        None
    };
    // SAFETY: these are immutable framework statics.
    let mut entries = unsafe {
        vec![
            (kSecClass, kSecClassGenericPassword),
            (kSecAttrService, service),
            (kSecAttrAccount, account),
            (kSecMatchSearchList, search_list),
            (kSecUseAuthenticationUI, kSecUseAuthenticationUIFail),
            (kSecReturnAttributes, kCFBooleanTrue),
            (kSecReturnData, kCFBooleanTrue),
            (kSecReturnPersistentRef, kCFBooleanTrue),
            (kSecMatchLimit, kSecMatchLimitAll),
        ]
    };
    if let Some(item_list) = item_list {
        // SAFETY: kSecMatchItemList is an immutable framework static.
        unsafe { entries.push((kSecMatchItemList, item_list)) };
    }
    // SAFETY: all keys are framework statics and all dynamic values remain held by owned.
    unsafe { dictionary(owned, &entries) }
}

unsafe fn exact_generic_password_delete_query(
    owned: &mut OwnedCf,
    keychain: SecKeychain,
    item: CfType,
    service: &str,
    account: &str,
) -> Result<CfMutableDictionary> {
    // SAFETY: created values remain held by owned.
    let service = unsafe { cf_string(owned, service)? };
    // SAFETY: same ownership.
    let account = unsafe { cf_string(owned, account)? };
    // SAFETY: keychain remains held.
    let search_list = unsafe { single_keychain_search_list(owned, keychain)? };
    // SAFETY: the exact query-result owner retains item through deletion.
    let item_list = unsafe { cf_array(owned, &[item])? };
    // SAFETY: all values remain live.
    unsafe {
        dictionary(
            owned,
            &[
                (kSecClass, kSecClassGenericPassword),
                (kSecAttrService, service),
                (kSecAttrAccount, account),
                (kSecMatchSearchList, search_list),
                (kSecMatchItemList, item_list),
                (kSecUseAuthenticationUI, kSecUseAuthenticationUIFail),
            ],
        )
    }
}

unsafe fn require_cf_attribute(
    dictionary_value: CfType,
    key: CfType,
    expected: CfType,
    label: &str,
) -> Result<()> {
    // SAFETY: dictionary_value is a validated live dictionary and key is a framework static.
    let observed = unsafe { CFDictionaryGetValue(dictionary_value, key) };
    // SAFETY: non-null observed/expected are live CF objects.
    if observed.is_null() || unsafe { CFEqual(observed, expected) } == 0 {
        bail!("System-Keychain key has mismatched {label}")
    }
    Ok(())
}

unsafe fn sha256_cf_data(value: CfType) -> Result<String> {
    // SAFETY: value is live for the call.
    if value.is_null() || unsafe { CFGetTypeID(value) } != unsafe { CFDataGetTypeID() } {
        bail!("generic-password persistent reference is not CFData")
    }
    // SAFETY: validated CFData.
    let length = unsafe { CFDataGetLength(value) };
    // SAFETY: same live CFData.
    let bytes = unsafe { CFDataGetBytePtr(value) };
    if length < 0 || (length > 0 && bytes.is_null()) {
        bail!("generic-password persistent reference has invalid bytes")
    }
    let data = if length == 0 {
        &[][..]
    } else {
        // SAFETY: bytes is valid for length while value is held.
        unsafe { std::slice::from_raw_parts(bytes, length as usize) }
    };
    Ok(sha256_hex_v2(data))
}

unsafe fn copy_bounded_generic_password_data(value: CfType) -> Result<Vec<u8>> {
    // SAFETY: value is live for the call.
    if value.is_null() || unsafe { CFGetTypeID(value) } != unsafe { CFDataGetTypeID() } {
        bail!("generic-password data is not CFData")
    }
    // SAFETY: validated CFData.
    let length = unsafe { CFDataGetLength(value) };
    let length = usize::try_from(length).context("decode generic-password data length")?;
    if length > MAX_GENERIC_PASSWORD_DATA_BYTES {
        bail!("generic-password data exceeds the protected-CAS byte bound")
    }
    if length == 0 {
        return Ok(Vec::new());
    }
    // SAFETY: same live CFData.
    let bytes = unsafe { CFDataGetBytePtr(value) };
    if bytes.is_null() {
        bail!("generic-password data has invalid bytes")
    }
    // SAFETY: bytes is valid for length while the create-rule result remains held.
    Ok(unsafe { std::slice::from_raw_parts(bytes, length) }.to_vec())
}

unsafe fn exact_generic_password_data_match(
    owned: &mut OwnedCf,
    keychain: SecKeychain,
    service: &str,
    account: &str,
    persistent_reference: Option<CfType>,
) -> Result<Option<ExactGenericPasswordDataMatch>> {
    // SAFETY: all dynamic query values remain held.
    let query = unsafe {
        exact_generic_password_data_query(owned, keychain, service, account, persistent_reference)?
    };
    let mut result: CfType = ptr::null();
    // SAFETY: query is live and result is a writable create-rule out pointer.
    let status = unsafe { SecItemCopyMatching(query.cast(), &mut result) };
    if status == ERR_SEC_ITEM_NOT_FOUND {
        return Ok(None);
    }
    if status != ERR_SEC_SUCCESS || result.is_null() {
        return Err(keychain_error(
            "read exact protected generic-password data",
            status,
        ));
    }
    owned.hold(result);
    // SAFETY: result is a live object.
    if unsafe { CFGetTypeID(result) } != unsafe { CFArrayGetTypeID() } {
        bail!("exact generic-password data lookup returned a non-array result")
    }
    // SAFETY: validated array.
    let count = unsafe { CFArrayGetCount(result) };
    if count != 1 {
        bail!("exact generic-password data identity is ambiguous ({count} matches)")
    }
    // SAFETY: array contains exactly one element.
    let attributes = unsafe { CFArrayGetValueAtIndex(result, 0) };
    if attributes.is_null()
        || unsafe { CFGetTypeID(attributes) } != unsafe { CFDictionaryGetTypeID() }
    {
        bail!("exact generic-password data lookup returned invalid attributes")
    }
    // SAFETY: created expectations remain held.
    let expected_service = unsafe { cf_string(owned, service)? };
    // SAFETY: same ownership.
    let expected_account = unsafe { cf_string(owned, account)? };
    // SAFETY: validated dictionary and live values.
    unsafe {
        require_cf_attribute(
            attributes,
            kSecAttrService,
            expected_service,
            "generic-password service",
        )?;
        require_cf_attribute(
            attributes,
            kSecAttrAccount,
            expected_account,
            "generic-password account",
        )?;
    }
    // SAFETY: validated dictionary and framework keys.
    let data = unsafe { CFDictionaryGetValue(attributes, kSecValueData) };
    // SAFETY: same.
    let observed_persistent_reference =
        unsafe { CFDictionaryGetValue(attributes, kSecValuePersistentRef) };
    if data.is_null() || observed_persistent_reference.is_null() {
        bail!("exact generic-password data lookup omitted data or persistent reference")
    }
    // SAFETY: validates both framework-returned representations and copies only bounded bytes.
    let data = unsafe { copy_bounded_generic_password_data(data)? };
    // SAFETY: validates persistent-reference representation.
    unsafe { sha256_cf_data(observed_persistent_reference)? };
    if let Some(expected_persistent_reference) = persistent_reference {
        // SAFETY: both references remain live in the same owner.
        if unsafe { CFEqual(expected_persistent_reference, observed_persistent_reference) } == 0 {
            bail!("identity-conditioned generic-password read returned another item")
        }
    }
    Ok(Some(ExactGenericPasswordDataMatch {
        data,
        persistent_reference: observed_persistent_reference,
    }))
}

unsafe fn exact_generic_password_match(
    owned: &mut OwnedCf,
    keychain: SecKeychain,
    service: &str,
    account: &str,
) -> Result<Option<(CfType, CfType)>> {
    // SAFETY: all dynamic query values remain held.
    let query = unsafe { exact_generic_password_query(owned, keychain, service, account)? };
    let mut result: CfType = ptr::null();
    // SAFETY: query is live and result is a writable create-rule out pointer.
    let status = unsafe { SecItemCopyMatching(query.cast(), &mut result) };
    if status == ERR_SEC_ITEM_NOT_FOUND {
        return Ok(None);
    }
    if status != ERR_SEC_SUCCESS || result.is_null() {
        return Err(keychain_error(
            "look up exact protected generic-password item",
            status,
        ));
    }
    owned.hold(result);
    // SAFETY: result is a live object.
    if unsafe { CFGetTypeID(result) } != unsafe { CFArrayGetTypeID() } {
        bail!("exact generic-password lookup returned a non-array result")
    }
    // SAFETY: validated array.
    let count = unsafe { CFArrayGetCount(result) };
    if count != 1 {
        bail!("exact generic-password identity is ambiguous ({count} matches)")
    }
    // SAFETY: array contains exactly one element.
    let attributes = unsafe { CFArrayGetValueAtIndex(result, 0) };
    if attributes.is_null()
        || unsafe { CFGetTypeID(attributes) } != unsafe { CFDictionaryGetTypeID() }
    {
        bail!("exact generic-password lookup returned invalid attributes")
    }
    // SAFETY: created expectations remain held.
    let expected_service = unsafe { cf_string(owned, service)? };
    // SAFETY: same ownership.
    let expected_account = unsafe { cf_string(owned, account)? };
    // SAFETY: validated dictionary and live values.
    unsafe {
        require_cf_attribute(
            attributes,
            kSecAttrService,
            expected_service,
            "generic-password service",
        )?;
        require_cf_attribute(
            attributes,
            kSecAttrAccount,
            expected_account,
            "generic-password account",
        )?;
    }
    // SAFETY: validated dictionary and framework keys.
    let item = unsafe { CFDictionaryGetValue(attributes, kSecValueRef) };
    // SAFETY: same.
    let persistent_reference = unsafe { CFDictionaryGetValue(attributes, kSecValuePersistentRef) };
    if item.is_null() || persistent_reference.is_null() {
        bail!("exact generic-password lookup omitted its item or persistent reference")
    }
    // SAFETY: validates persistent-reference representation.
    unsafe { sha256_cf_data(persistent_reference)? };
    Ok(Some((item, persistent_reference)))
}

unsafe fn validate_private_p256_key(
    owned: &mut OwnedCf,
    persisted_attributes: CfType,
    private_key: SecKey,
    application_tag: &[u8],
) -> Result<()> {
    // SAFETY: created data is retained by owned.
    let expected_tag = unsafe { cf_data(owned, application_tag)? };
    // SAFETY: persisted_attributes is a validated live dictionary.
    unsafe {
        require_cf_attribute(
            persisted_attributes,
            kSecAttrApplicationTag,
            expected_tag,
            "application tag",
        )?;
    }
    // SAFETY: private_key is a live SecKey returned by the query.
    let attributes = unsafe { SecKeyCopyAttributes(private_key) };
    if attributes.is_null() {
        bail!("System-Keychain key has no inspectable attributes")
    }
    owned.hold(attributes);
    // SAFETY: attributes is a live dictionary and expected values are framework statics.
    unsafe {
        require_cf_attribute(
            attributes,
            kSecAttrKeyClass,
            kSecAttrKeyClassPrivate,
            "private-key class",
        )?;
        require_cf_attribute(
            attributes,
            kSecAttrKeyType,
            kSecAttrKeyTypeECSECPrimeRandom,
            "EC key type",
        )?;
        require_cf_attribute(
            attributes,
            kSecAttrIsPermanent,
            kCFBooleanTrue,
            "permanent-key state",
        )?;
        require_cf_attribute(
            attributes,
            kSecAttrCanSign,
            kCFBooleanTrue,
            "signing capability",
        )?;
    }
    // SAFETY: attributes is a live dictionary.
    let key_size = unsafe { CFDictionaryGetValue(attributes, kSecAttrKeySizeInBits) };
    let mut bits = 0_i64;
    // SAFETY: key_size is checked non-null and bits is writable for an SInt64 result.
    if key_size.is_null()
        || unsafe { CFNumberGetValue(key_size, K_CF_NUMBER_SINT64, (&mut bits as *mut i64).cast()) }
            == 0
        || bits != 256
    {
        bail!("System-Keychain key is not exactly P-256")
    }
    Ok(())
}

unsafe fn exact_private_key_match(
    owned: &mut OwnedCf,
    keychain: SecKeychain,
    application_tag: &[u8],
) -> Result<Option<SecKey>> {
    // SAFETY: query objects remain held in owned.
    let query = unsafe { exact_private_key_query(owned, keychain, application_tag)? };
    let mut result: CfType = ptr::null();
    // SAFETY: query is a live dictionary and result is a writable create-rule out pointer.
    let status = unsafe { SecItemCopyMatching(query.cast(), &mut result) };
    if status == ERR_SEC_ITEM_NOT_FOUND {
        return Ok(None);
    }
    if status != ERR_SEC_SUCCESS || result.is_null() {
        return Err(keychain_error(
            "look up exact System-Keychain P-256 key",
            status,
        ));
    }
    owned.hold(result);
    // SAFETY: result is a live CF object.
    if unsafe { CFGetTypeID(result) } != unsafe { CFArrayGetTypeID() } {
        bail!("exact System-Keychain key lookup returned a non-array result")
    }
    // SAFETY: result has been validated as an array.
    let count = unsafe { CFArrayGetCount(result) };
    if count != 1 {
        bail!("exact System-Keychain key identity is ambiguous ({count} matches)")
    }
    // SAFETY: the array has exactly one element.
    let item = unsafe { CFArrayGetValueAtIndex(result, 0) };
    // SAFETY: item is non-owned but remains live with result.
    if item.is_null() || unsafe { CFGetTypeID(item) } != unsafe { CFDictionaryGetTypeID() } {
        bail!("exact System-Keychain key lookup returned invalid attributes")
    }
    // SAFETY: item is a live dictionary and kSecValueRef is a framework static.
    let private_key: SecKey = unsafe { CFDictionaryGetValue(item, kSecValueRef) }.cast();
    if private_key.is_null() {
        bail!("exact System-Keychain key lookup returned a null key reference")
    }
    // SAFETY: both item and private_key remain live through the retained result array.
    unsafe {
        validate_private_p256_key(owned, item, private_key, application_tag)?;
    }
    Ok(Some(private_key))
}

#[link(name = "System")]
unsafe extern "C" {
    fn launch_activate_socket(
        name: *const c_char,
        descriptors: *mut *mut c_int,
        count: *mut usize,
    ) -> c_int;
    fn getpeereid(
        socket: c_int,
        effective_uid: *mut libc::uid_t,
        effective_gid: *mut libc::gid_t,
    ) -> c_int;
}

#[link(name = "bsm")]
unsafe extern "C" {
    fn audit_token_to_pid(token: AuditToken) -> libc::pid_t;
    fn audit_token_to_euid(token: AuditToken) -> libc::uid_t;
    fn audit_token_to_egid(token: AuditToken) -> libc::gid_t;
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
    fn proc_pidpath_audittoken(
        audit_token: *mut AuditToken,
        buffer: *mut c_void,
        buffer_size: u32,
    ) -> c_int;
    fn proc_pidpath(pid: c_int, buffer: *mut c_void, buffer_size: u32) -> c_int;
}

#[link(name = "Security", kind = "framework")]
unsafe extern "C" {
    static kSecClass: CfType;
    static kSecClassKey: CfType;
    static kSecClassGenericPassword: CfType;
    static kSecAttrApplicationTag: CfType;
    static kSecAttrService: CfType;
    static kSecAttrAccount: CfType;
    static kSecAttrKeyClass: CfType;
    static kSecAttrKeyClassPrivate: CfType;
    static kSecAttrKeyType: CfType;
    static kSecAttrKeyTypeECSECPrimeRandom: CfType;
    static kSecAttrKeySizeInBits: CfType;
    static kSecAttrIsPermanent: CfType;
    static kSecAttrCanSign: CfType;
    static kSecValueRef: CfType;
    static kSecReturnAttributes: CfType;
    static kSecReturnRef: CfType;
    static kSecReturnData: CfType;
    static kSecReturnPersistentRef: CfType;
    static kSecValueData: CfType;
    static kSecValuePersistentRef: CfType;
    static kSecMatchSearchList: CfType;
    static kSecMatchItemList: CfType;
    static kSecMatchLimit: CfType;
    static kSecMatchLimitAll: CfType;
    static kSecUseAuthenticationUI: CfType;
    static kSecUseAuthenticationUIFail: CfType;
    static kSecGuestAttributeAudit: CfType;
    static kSecCodeInfoUnique: CfType;
    static kSecCodeInfoFlags: CfType;
    static kSecCodeInfoTeamIdentifier: CfType;
    static kSecCodeInfoEntitlements: CfType;
    static kSecCodeInfoEntitlementsDict: CfType;

    fn SecKeychainSetUserInteractionAllowed(state: u8) -> OsStatus;
    fn SecKeychainOpen(path_name: *const c_char, keychain: *mut SecKeychain) -> OsStatus;
    fn SecKeychainGetPath(
        keychain: SecKeychain,
        path_length: *mut u32,
        path_name: *mut c_char,
    ) -> OsStatus;
    fn SecItemCopyMatching(query: CfType, result: *mut CfType) -> OsStatus;
    fn SecItemDelete(query: CfType) -> OsStatus;
    fn SecKeyCopyAttributes(key: SecKey) -> CfType;
    fn SecRequirementCreateWithString(
        text: CfType,
        flags: u32,
        requirement: *mut SecRequirement,
    ) -> OsStatus;
    fn SecCodeCopyGuestWithAttributes(
        host: SecCode,
        attributes: CfType,
        flags: u32,
        guest: *mut SecCode,
    ) -> OsStatus;
    fn SecCodeCopySelf(flags: u32, code: *mut SecCode) -> OsStatus;
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
    static kCFBooleanTrue: CfType;
    fn CFRelease(value: CfType);
    fn CFGetTypeID(value: CfType) -> usize;
    fn CFEqual(left: CfType, right: CfType) -> u8;
    fn CFStringCreateWithBytes(
        allocator: CfType,
        bytes: *const u8,
        byte_count: isize,
        encoding: u32,
        external_representation: u8,
    ) -> CfType;
    fn CFDataCreate(allocator: CfType, bytes: *const u8, length: isize) -> CfType;
    fn CFDataGetTypeID() -> usize;
    fn CFDataGetLength(data: CfType) -> isize;
    fn CFDataGetBytePtr(data: CfType) -> *const u8;
    fn CFNumberGetTypeID() -> usize;
    fn CFDictionaryGetTypeID() -> usize;
    fn CFDictionaryCreateMutable(
        allocator: CfType,
        capacity: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> CfMutableDictionary;
    fn CFDictionarySetValue(dictionary: CfMutableDictionary, key: CfType, value: CfType);
    fn CFDictionaryGetValue(dictionary: CfType, key: CfType) -> CfType;
    fn CFNumberGetValue(number: CfType, number_type: i32, value: *mut c_void) -> u8;
    fn CFArrayCreate(
        allocator: CfType,
        values: *const CfType,
        count: isize,
        callbacks: *const c_void,
    ) -> CfType;
    fn CFArrayGetTypeID() -> usize;
    fn CFArrayGetCount(array: CfType) -> isize;
    fn CFArrayGetValueAtIndex(array: CfType, index: isize) -> CfType;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fd3_plan_never_clobbers_an_unrelated_descriptor() {
        assert_eq!(classify_fd3_plan(3, true).unwrap(), Fd3Plan::AlreadyFd3);
        assert_eq!(
            classify_fd3_plan(9, false).unwrap(),
            Fd3Plan::DuplicateToFd3
        );
        assert!(classify_fd3_plan(9, true).is_err());
    }

    #[test]
    fn code_posture_requires_explicit_library_validation_and_zero_entitlements() {
        validate_frozen_code_posture(REQUIRED_CODE_DIRECTORY_FLAGS, false, false, false).unwrap();
        assert!(validate_frozen_code_posture(0x0001_0002, false, false, false).is_err());
        assert!(
            validate_frozen_code_posture(REQUIRED_CODE_DIRECTORY_FLAGS, true, false, false)
                .is_err()
        );
        assert!(
            validate_frozen_code_posture(REQUIRED_CODE_DIRECTORY_FLAGS, false, true, false)
                .is_err()
        );
        assert!(
            validate_frozen_code_posture(REQUIRED_CODE_DIRECTORY_FLAGS, false, false, true)
                .is_err()
        );
    }

    #[test]
    fn peer_credential_sources_must_agree() {
        assert!(reconcile_peer_identity(42, 501, 20, 42, 501, 20).is_ok());
        assert!(reconcile_peer_identity(42, 501, 20, 43, 501, 20).is_err());
        assert!(reconcile_peer_identity(42, 501, 20, 42, 502, 20).is_err());
        assert!(reconcile_peer_identity(42, 501, 20, 42, 501, 21).is_err());
    }

    #[test]
    fn only_exact_lowercase_hash_literals_are_accepted() {
        assert!(decode_exact_hex(&"ab".repeat(20), 20).is_ok());
        assert!(decode_exact_hex(&"AB".repeat(20), 20).is_err());
        assert!(decode_exact_hex(&"00".repeat(19), 20).is_err());
        assert!(decode_exact_hex(&"00".repeat(20), 20).is_err());
    }

    #[test]
    fn expected_keychain_denials_are_classified_without_native_calls() {
        assert_eq!(
            classify_keychain_status(-25308),
            KeychainStatusClass::InteractionNotAllowed
        );
        assert_eq!(
            classify_keychain_status(-25315),
            KeychainStatusClass::InteractionRequired
        );
        assert_eq!(
            classify_keychain_status(-25293),
            KeychainStatusClass::AuthorizationDenied
        );
        assert_eq!(classify_keychain_status(-1), KeychainStatusClass::Other(-1));
    }

    #[test]
    fn hex_round_trip_is_lowercase_and_exact() {
        let bytes = decode_exact_hex(&"0123456789abcdef".repeat(5), 40).unwrap();
        assert_eq!(hex_lower(&bytes), "0123456789abcdef".repeat(5));
    }

    #[test]
    fn darwin_process_identity_layouts_match_sdk_abi() {
        assert_eq!(size_of::<AuditToken>(), 32);
        assert_eq!(size_of::<ProcBsdInfo>(), 136);
    }

    #[test]
    fn application_tag_must_be_one_nonempty_bounded_value() {
        assert!(validate_application_tag(b"scope:signing-key").is_ok());
        assert!(validate_application_tag(b"").is_err());
        assert!(validate_application_tag(&vec![0_u8; MAX_APPLICATION_TAG_BYTES + 1]).is_err());
    }

    #[test]
    fn generic_password_fields_and_identity_digests_are_exact() {
        assert!(validate_generic_password_field("com.substrate.lifecycle.v1", "service").is_ok());
        assert!(validate_generic_password_field("scope:current-anchor", "account").is_ok());
        assert!(validate_generic_password_field("", "account").is_err());
        assert!(validate_generic_password_field("scope\nother", "account").is_err());
        assert!(require_lower_sha256_literal(&"ab".repeat(32), "identity").is_ok());
        assert!(require_lower_sha256_literal(&"AB".repeat(32), "identity").is_err());
    }

    #[test]
    fn keychain_mutations_are_exact_ref_and_query_ui_fail_only() {
        let source = include_str!("darwin.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source before tests");
        assert!(production.contains("kSecClassGenericPassword"));
        assert!(production.contains("kSecAttrService"));
        assert!(production.contains("kSecAttrAccount"));
        assert!(production.contains("kSecMatchItemList"));
        assert!(production.contains("kSecUseAuthenticationUIFail"));
    }

    #[test]
    fn generic_password_data_read_is_exact_stable_bounded_and_nonmutating() {
        let source = include_str!("darwin.rs");
        let method = source
            .split("pub fn read_exact_system_generic_password_data")
            .nth(1)
            .expect("read-only generic-password API")
            .split("pub(crate) fn delete_retained_exact_generic_password")
            .next()
            .expect("read-only API boundary");
        assert!(method.contains("open_explicit_system_keychain"));
        assert_eq!(
            method.matches("exact_generic_password_data_match(").count(),
            2
        );
        assert!(method.contains("Some(first.persistent_reference)"));
        assert!(!method.contains("SecItemDelete"));

        let query = source
            .split("unsafe fn exact_generic_password_data_query")
            .nth(1)
            .expect("read-only generic-password query")
            .split("unsafe fn exact_generic_password_delete_query")
            .next()
            .expect("read-only query boundary");
        for required in [
            "kSecClassGenericPassword",
            "kSecAttrService",
            "kSecAttrAccount",
            "kSecMatchSearchList",
            "kSecMatchItemList",
            "kSecUseAuthenticationUIFail",
            "kSecReturnAttributes",
            "kSecReturnData",
            "kSecReturnPersistentRef",
            "kSecMatchLimitAll",
        ] {
            assert!(
                query.contains(required),
                "missing exact query key: {required}"
            );
        }
        assert!(!query.contains("SecItemDelete"));
        assert_eq!(
            MAX_GENERIC_PASSWORD_DATA_BYTES,
            crate::contract::FRAME_MAX_BYTES
        );
    }

    #[test]
    fn production_ffi_surface_has_no_acl_sign_or_export_symbols() {
        let source = include_str!("darwin.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source before tests");
        for forbidden in [
            "SecAccessCreate",
            "SecACLCreate",
            "SecKeyCreateSignature",
            "SecKeyCopyExternalRepresentation",
            "SecItemExport",
        ] {
            assert!(
                !production.contains(forbidden),
                "forbidden FFI: {forbidden}"
            );
        }
    }
}
