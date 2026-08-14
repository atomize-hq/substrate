//! Sealed, disposable-only Keychain capability probe.
//!
//! The production prospective lane cannot name this module. Its negative-operation reachability is
//! limited to `SecurityUiDenied` methods which accept an already signed receipt and reject every
//! target-set kind except `DisposableCapability` before making a Keychain call. One separate
//! read-only helper supplies strict key/SPKI/ACL identity to the production observer; it has no
//! call edge to the signing, private-export, or ACL-mutation functions confined to this file.

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::ffi::{c_char, c_void, CString};
use std::ptr;

pub use substrate_common::macos_retirement_v2::DisposableCapabilityControlV2;
use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, sha256_hex_v2, validate_publisher_pre_removal_receipt_v2, HostTargetRoleV2,
    PublisherPreRemovalReceiptV2, TargetSetKindV2, MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2,
    MAC_R3_FINALIZER_PATH_V2, MAC_R3_PRODUCT_PUBLISHER_PATH_V2,
};

use crate::darwin::{ExactKeyDeleteOutcome, SecurityUiDenied, SYSTEM_KEYCHAIN_PATH};
use crate::targets::{derive_targets, FixedTarget};

const REPORT_OWNER: &str = "substrate.r3-macos-disposable-capability";
const REPORT_VERSION: u32 = 2;
const WRONG_SURROGATE_TAG_SUFFIX: &str = ":wrong-surrogate-signing-key";
const SIGN_PROBE_DOMAIN: &[u8] = b"SUBSTRATE_R3_DISPOSABLE_SIGN_DENIAL_V2\0";
const REPLACEMENT_DESCRIPTION: &str = "disposable-forbidden-finalizer-sign-replacement";

const UNMATCHABLE_OWNER_UID: u32 = u32::MAX;
const UNMATCHABLE_OWNER_GID: u32 = u32::MAX;
const OWNER_TYPE_USE_UID_AND_GID: u32 = 3;
const PROMPT_SELECTOR_NONE: u16 = 0;
const MAX_SYSTEM_KEYCHAIN_PATH_BYTES: usize = 1024;

const PUBLISHER_SIGN_DELETE_DESCRIPTION: &str = "publisher-sign-and-delete";
const FINALIZER_DELETE_DESCRIPTION: &str = "finalizer-delete-only";
const PRIVATE_OPERATION_DENY_DESCRIPTION: &str = "deny-private-key-operations";
const OWNER_MUTATION_DENY_DESCRIPTION: &str = "deny-owner-and-acl-mutation";

const AUTH_ANY: &str = "any";
const AUTH_SIGN: &str = "sign";
const AUTH_DELETE: &str = "delete";
const AUTH_EXPORT_WRAPPED: &str = "export_wrapped";
const AUTH_EXPORT_CLEAR: &str = "export_clear";
const AUTH_IMPORT_WRAPPED: &str = "import_wrapped";
const AUTH_IMPORT_CLEAR: &str = "import_clear";
const AUTH_ENCRYPT: &str = "encrypt";
const AUTH_DECRYPT: &str = "decrypt";
const AUTH_MAC: &str = "mac";
const AUTH_DERIVE: &str = "derive";
const AUTH_CHANGE_ACL: &str = "change_acl";
const AUTH_CHANGE_OWNER: &str = "change_owner";

type CfType = *const c_void;
type CfMutableDictionary = *mut c_void;
type SecAccess = *const c_void;
type SecAcl = *const c_void;
type SecKey = *const c_void;
type SecKeychain = *const c_void;
type SecTrustedApplication = *const c_void;
type OsStatus = i32;

const ERR_SEC_SUCCESS: OsStatus = 0;
const ERR_SEC_ITEM_NOT_FOUND: OsStatus = -25300;
const ERR_SEC_AUTH_FAILED: i64 = -25293;
const ERR_SEC_INTERACTION_NOT_ALLOWED: i64 = -25308;
const ERR_SEC_INTERACTION_REQUIRED: i64 = -25315;
const ERR_SEC_DATA_NOT_AVAILABLE: i64 = -25316;
const K_CF_NUMBER_SINT64: i32 = 4;
const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;

#[cfg(test)]
const DISPOSABLE_SEQUENCE: [DisposableCapabilityControlV2; 4] = [
    DisposableCapabilityControlV2::Sign,
    DisposableCapabilityControlV2::ExportPrivate,
    DisposableCapabilityControlV2::ReplaceAccess,
    DisposableCapabilityControlV2::DeleteWrongKey,
];

/// Return the one closed control sequence. The production target-set kind deliberately has no
/// negative-operation sequence and therefore cannot be routed to this FFI by generic code.
#[cfg(test)]
fn disposable_capability_sequence_for(
    kind: TargetSetKindV2,
) -> &'static [DisposableCapabilityControlV2] {
    match kind {
        TargetSetKindV2::DisposableCapability => &DISPOSABLE_SEQUENCE,
        TargetSetKindV2::ProspectiveHost => &[],
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FrozenDenialClassV2 {
    AuthorizationFailed,
    InteractionNotAllowed,
    InteractionRequired,
    DataNotAvailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DisposableCapabilityInvocationV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub target_set_kind: TargetSetKindV2,
    pub scope_id: String,
    pub control: DisposableCapabilityControlV2,
    pub denial: FrozenDenialClassV2,
    pub code: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct KeyIntegrityObservationV2 {
    pub application_tag_sha256: String,
    pub label: String,
    pub application_label_base64url: String,
    pub persistent_reference_sha256: String,
    pub public_spki_sha256: String,
    pub access_control_sha256: String,
    pub identity_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DisposableCapabilityStateObservationV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub target_set_kind: TargetSetKindV2,
    pub scope_id: String,
    pub signer_access_control_sha256: String,
    pub target: KeyIntegrityObservationV2,
    pub wrong_surrogate: KeyIntegrityObservationV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CanonicalDisposableCapabilityStateV2 {
    pub observation: DisposableCapabilityStateObservationV2,
    pub canonical_json: String,
    pub sha256: String,
}

impl CanonicalDisposableCapabilityStateV2 {
    /// Pure comparison used by the journal engine to prove that both exact key identities remain
    /// unchanged across a control. It also revalidates both canonical envelopes before comparing.
    pub fn require_same_keys(&self, after: &Self) -> Result<()> {
        validate_canonical_state(self)?;
        validate_canonical_state(after)?;
        if self.observation.target_set_kind != TargetSetKindV2::DisposableCapability
            || after.observation.target_set_kind != TargetSetKindV2::DisposableCapability
            || self.observation.scope_id != after.observation.scope_id
            || self.observation.signer_access_control_sha256
                != after.observation.signer_access_control_sha256
            || self.observation.target != after.observation.target
            || self.observation.wrong_surrogate != after.observation.wrong_surrogate
        {
            bail!("disposable target or wrong-surrogate identity changed across the control")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisposableRecoveryObservationClassV2 {
    DeniedAndPreserved,
    AmbiguousPreserving,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DisposableCapabilityAfterObservationV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub target_set_kind: TargetSetKindV2,
    pub scope_id: String,
    pub control: DisposableCapabilityControlV2,
    pub recovery_class: DisposableRecoveryObservationClassV2,
    pub state: DisposableCapabilityStateObservationV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CanonicalDisposableCapabilityAfterV2 {
    pub observation: DisposableCapabilityAfterObservationV2,
    pub canonical_json: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CanonicalDisposableCapabilityInvocationV2 {
    pub invocation: DisposableCapabilityInvocationV2,
    pub canonical_json: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
struct TrustedApplicationDigest {
    data_base64url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
struct AclEntrySnapshot {
    description: String,
    prompt_selector: u16,
    authorizations: Vec<String>,
    trusted_applications: Vec<TrustedApplicationDigest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct AccessSnapshot {
    owner_uid: u32,
    owner_gid: u32,
    owner_type: u32,
    entries: Vec<AclEntrySnapshot>,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct KeyIdentityMaterial<'a> {
    application_tag_sha256: &'a str,
    label: &'a str,
    application_label_base64url: &'a str,
    persistent_reference_sha256: &'a str,
    public_spki_sha256: &'a str,
    access_control_sha256: &'a str,
}

/// Opaque process-local ownership of the one exact SecKey resolution and its complete strict
/// identity.  It is deliberately non-Clone and never exposes the raw key reference.
pub(crate) struct RetainedExactSystemKey {
    owned: OwnedCf,
    keychain: SecKeychain,
    key: SecKey,
    application_tag: Vec<u8>,
    expected_access_sha256: String,
    target_set_kind: TargetSetKindV2,
    identity: KeyIntegrityObservationV2,
}

impl RetainedExactSystemKey {
    pub(crate) fn identity(&self) -> &KeyIntegrityObservationV2 {
        &self.identity
    }
}

pub(crate) fn observe_exact_key(
    _security: &SecurityUiDenied,
    application_tag: &[u8],
    expected_access_sha256: &str,
    target_set_kind: TargetSetKindV2,
) -> Result<Option<KeyIntegrityObservationV2>> {
    Ok(retain_exact_key(
        _security,
        application_tag,
        expected_access_sha256,
        target_set_kind,
    )?
    .map(|retained| retained.identity))
}

pub(crate) fn retain_exact_key(
    _security: &SecurityUiDenied,
    application_tag: &[u8],
    expected_access_sha256: &str,
    target_set_kind: TargetSetKindV2,
) -> Result<Option<RetainedExactSystemKey>> {
    if application_tag.is_empty() || application_tag.len() > 1024 {
        bail!("exact System-Keychain application tag is empty or exceeds its bound")
    }
    require_lower_hex_sha256(
        expected_access_sha256,
        "expected signer access-control digest",
    )?;
    // SAFETY: the query is fixed to the explicit System Keychain and exact tag. No raw reference
    // escapes; signing/export-private/ACL-mutation functions are not reachable from this path.
    unsafe {
        let mut owned = OwnedCf::new();
        let keychain = open_explicit_system_keychain(&mut owned)?;
        let Some((key, attributes)) = exact_private_key_match(
            &mut owned,
            keychain,
            application_tag,
            "exact production signer",
        )?
        else {
            return Ok(None);
        };
        let observation = observe_key(
            &mut owned,
            key,
            attributes,
            application_tag,
            Some((expected_access_sha256, target_set_kind)),
        )?;
        if observation.access_control_sha256 != expected_access_sha256 {
            bail!("persisted production signer SecAccess digest differs from signed receipt")
        }
        Ok(Some(RetainedExactSystemKey {
            owned,
            keychain,
            key,
            application_tag: application_tag.to_vec(),
            expected_access_sha256: expected_access_sha256.to_owned(),
            target_set_kind,
            identity: observation,
        }))
    }
}

pub(crate) fn delete_retained_exact_key(
    _security: &mut SecurityUiDenied,
    mut retained: RetainedExactSystemKey,
    expected_identity_sha256: &str,
) -> Result<ExactKeyDeleteOutcome> {
    require_lower_hex_sha256(
        expected_identity_sha256,
        "expected retained signer identity",
    )?;
    if retained.identity.identity_sha256 != expected_identity_sha256
        || retained.identity.access_control_sha256 != retained.expected_access_sha256
    {
        bail!("retained signer identity differs from the identity-conditioned delete")
    }
    // SAFETY: the original exact key/keychain/result remain owned by retained. A fresh exact query
    // must return that same CF object and full identity immediately before the one exact-ref delete.
    unsafe {
        let current = exact_private_key_match(
            &mut retained.owned,
            retained.keychain,
            &retained.application_tag,
            "identity-conditioned production signer",
        )?
        .context("retained production signer disappeared before deletion")?;
        if CFEqual(current.0.cast(), retained.key.cast()) == 0 {
            bail!("exact signer reference was substituted after pre-observation")
        }
        let current_identity = observe_key(
            &mut retained.owned,
            current.0,
            current.1,
            &retained.application_tag,
            Some((&retained.expected_access_sha256, retained.target_set_kind)),
        )?;
        if current_identity != retained.identity {
            bail!("exact signer full identity changed after pre-observation")
        }
        let query = exact_delete_query(
            &mut retained.owned,
            retained.keychain,
            retained.key,
            &retained.application_tag,
        )?;
        let status = SecItemDelete(query.cast());
        if status != ERR_SEC_SUCCESS {
            bail!("identity-conditioned exact signer deletion failed with OSStatus {status}")
        }
        if exact_private_key_match(
            &mut retained.owned,
            retained.keychain,
            &retained.application_tag,
            "post-delete production signer",
        )?
        .is_some()
        {
            bail!("exact signer tag is not absent after identity-conditioned deletion")
        }
        Ok(ExactKeyDeleteOutcome::DeletedAndAbsent)
    }
}

pub(crate) fn observe_pre_probe(
    _security: &SecurityUiDenied,
    receipt: &PublisherPreRemovalReceiptV2,
) -> Result<CanonicalDisposableCapabilityStateV2> {
    // The disposable-kind gate intentionally precedes every FFI call in this module.
    let (target_tag, wrong_tag) =
        derive_disposable_tags(receipt.target_set_kind, &receipt.scope_id)?;
    validate_publisher_pre_removal_receipt_v2(receipt)
        .context("validate disposable publisher receipt structure")?;
    // SAFETY: every raw reference remains retained and the two predicates are internally derived.
    unsafe {
        let mut owned = OwnedCf::new();
        let keychain = open_explicit_system_keychain(&mut owned)?;
        let (target_key, target_attributes) = exact_private_key_match(
            &mut owned,
            keychain,
            &target_tag,
            "disposable target signer",
        )?
        .context("disposable target signer is absent")?;
        let (wrong_key, wrong_attributes) =
            exact_private_key_match(&mut owned, keychain, &wrong_tag, "wrong-surrogate signer")?
                .context("wrong-surrogate signer is absent")?;

        let target = observe_key(
            &mut owned,
            target_key,
            target_attributes,
            &target_tag,
            Some((
                &receipt.signer_access_control_sha256,
                receipt.target_set_kind,
            )),
        )?;
        let wrong_surrogate =
            observe_key(&mut owned, wrong_key, wrong_attributes, &wrong_tag, None)?;

        canonical_state(DisposableCapabilityStateObservationV2 {
            schema_owner: REPORT_OWNER.to_owned(),
            schema_version: REPORT_VERSION,
            target_set_kind: TargetSetKindV2::DisposableCapability,
            scope_id: receipt.scope_id.clone(),
            signer_access_control_sha256: receipt.signer_access_control_sha256.clone(),
            target,
            wrong_surrogate,
        })
    }
}

pub(crate) fn invoke_one(
    security: &mut SecurityUiDenied,
    receipt: &PublisherPreRemovalReceiptV2,
    control: DisposableCapabilityControlV2,
) -> Result<CanonicalDisposableCapabilityInvocationV2> {
    // Strictly validate both exact keys and the receipt-bound target ACL immediately before the
    // one negative system call. This also repeats the kind gate before any FFI.
    let _pre_call = observe_pre_probe(security, receipt)?;
    let (target_tag, wrong_tag) =
        derive_disposable_tags(receipt.target_set_kind, &receipt.scope_id)?;
    // SAFETY: all raw references remain retained; the selected control admits exactly one call.
    let code = unsafe {
        let mut owned = OwnedCf::new();
        let keychain = open_explicit_system_keychain(&mut owned)?;
        let (target_key, target_attributes) = exact_private_key_match(
            &mut owned,
            keychain,
            &target_tag,
            "disposable target signer before invocation",
        )?
        .context("disposable target signer is absent before invocation")?;
        let (wrong_key, wrong_attributes) = exact_private_key_match(
            &mut owned,
            keychain,
            &wrong_tag,
            "wrong-surrogate signer before invocation",
        )?
        .context("wrong-surrogate signer is absent before invocation")?;
        let _target_call_identity = observe_key(
            &mut owned,
            target_key,
            target_attributes,
            &target_tag,
            Some((
                &receipt.signer_access_control_sha256,
                receipt.target_set_kind,
            )),
        )?;
        let _wrong_call_identity =
            observe_key(&mut owned, wrong_key, wrong_attributes, &wrong_tag, None)?;
        match control {
            DisposableCapabilityControlV2::Sign => {
                let payload = sign_probe_payload(&receipt.scope_id);
                attempt_sign(&mut owned, target_key, &payload)?
            }
            DisposableCapabilityControlV2::ExportPrivate => {
                attempt_private_export(&mut owned, target_key)?
            }
            DisposableCapabilityControlV2::ReplaceAccess => {
                let replacement = build_forbidden_replacement_access(&mut owned)?;
                let status = SecKeychainItemSetAccess(target_key.cast(), replacement);
                if status == ERR_SEC_SUCCESS {
                    bail!("disposable ACL/trusted-application replacement unexpectedly succeeded")
                }
                i64::from(status)
            }
            DisposableCapabilityControlV2::DeleteWrongKey => {
                let query = exact_delete_query(&mut owned, keychain, wrong_key, &wrong_tag)?;
                let status = SecItemDelete(query.cast());
                if status == ERR_SEC_SUCCESS {
                    bail!("wrong-surrogate exact-reference deletion unexpectedly succeeded")
                }
                i64::from(status)
            }
        }
    };
    let (denial, code) = require_frozen_denial(control, code)?;
    canonical_invocation(DisposableCapabilityInvocationV2 {
        schema_owner: REPORT_OWNER.to_owned(),
        schema_version: REPORT_VERSION,
        target_set_kind: TargetSetKindV2::DisposableCapability,
        scope_id: receipt.scope_id.clone(),
        control,
        denial,
        code,
    })
}

pub(crate) fn observe_after(
    security: &SecurityUiDenied,
    receipt: &PublisherPreRemovalReceiptV2,
    control: DisposableCapabilityControlV2,
) -> Result<CanonicalDisposableCapabilityAfterV2> {
    let state = observe_pre_probe(security, receipt)?.observation;
    let recovery_class = match control {
        DisposableCapabilityControlV2::ReplaceAccess
        | DisposableCapabilityControlV2::DeleteWrongKey => {
            DisposableRecoveryObservationClassV2::DeniedAndPreserved
        }
        DisposableCapabilityControlV2::Sign | DisposableCapabilityControlV2::ExportPrivate => {
            // These operations have no persistent success marker. After a crash in Invoked, an
            // unchanged key proves preservation but cannot reconstruct whether the call denied.
            DisposableRecoveryObservationClassV2::AmbiguousPreserving
        }
    };
    canonical_after(DisposableCapabilityAfterObservationV2 {
        schema_owner: REPORT_OWNER.to_owned(),
        schema_version: REPORT_VERSION,
        target_set_kind: TargetSetKindV2::DisposableCapability,
        scope_id: receipt.scope_id.clone(),
        control,
        recovery_class,
        state,
    })
}

fn derive_disposable_tags(kind: TargetSetKindV2, scope: &str) -> Result<(Vec<u8>, Vec<u8>)> {
    if kind != TargetSetKindV2::DisposableCapability {
        bail!("negative capability probe is sealed to DisposableCapability receipts")
    }
    let targets = derive_targets(kind, scope)?;
    let target_tag = targets
        .iter()
        .find(|target| target.role == HostTargetRoleV2::Signer)
        .and_then(|target| match &target.target {
            FixedTarget::SigningKey { application_tag } => Some(application_tag.clone()),
            _ => None,
        })
        .context("disposable target set lacks its one exact signer")?;
    let wrong_tag = format!("{scope}{WRONG_SURROGATE_TAG_SUFFIX}").into_bytes();
    if wrong_tag == target_tag {
        bail!("compiled wrong-surrogate tag aliases the target signer")
    }
    Ok((target_tag, wrong_tag))
}

fn sign_probe_payload(scope: &str) -> Vec<u8> {
    let mut payload = Vec::with_capacity(SIGN_PROBE_DOMAIN.len() + scope.len());
    payload.extend_from_slice(SIGN_PROBE_DOMAIN);
    payload.extend_from_slice(scope.as_bytes());
    Sha256::digest(payload).to_vec()
}

fn require_frozen_denial(
    control: DisposableCapabilityControlV2,
    code: i64,
) -> Result<(FrozenDenialClassV2, i64)> {
    let denial = match code {
        ERR_SEC_AUTH_FAILED => FrozenDenialClassV2::AuthorizationFailed,
        ERR_SEC_INTERACTION_NOT_ALLOWED => FrozenDenialClassV2::InteractionNotAllowed,
        ERR_SEC_INTERACTION_REQUIRED => FrozenDenialClassV2::InteractionRequired,
        ERR_SEC_DATA_NOT_AVAILABLE if control == DisposableCapabilityControlV2::ExportPrivate => {
            FrozenDenialClassV2::DataNotAvailable
        }
        _ => bail!("{control:?} returned non-frozen denial code {code}"),
    };
    Ok((denial, code))
}

fn canonical_state(
    observation: DisposableCapabilityStateObservationV2,
) -> Result<CanonicalDisposableCapabilityStateV2> {
    let bytes = canonical_bytes_v2(&observation).context("serialize canonical capability state")?;
    let sha256 = sha256_hex_v2(&bytes);
    let canonical_json = String::from_utf8(bytes).context("canonical capability state is UTF-8")?;
    Ok(CanonicalDisposableCapabilityStateV2 {
        observation,
        canonical_json,
        sha256,
    })
}

fn validate_canonical_state(value: &CanonicalDisposableCapabilityStateV2) -> Result<()> {
    let bytes = canonical_bytes_v2(&value.observation)
        .context("re-serialize canonical capability state")?;
    if value.canonical_json.as_bytes() != bytes
        || value.sha256 != sha256_hex_v2(&bytes)
        || value.observation.schema_owner != REPORT_OWNER
        || value.observation.schema_version != REPORT_VERSION
    {
        bail!("disposable capability state canonical envelope is invalid")
    }
    Ok(())
}

fn canonical_invocation(
    invocation: DisposableCapabilityInvocationV2,
) -> Result<CanonicalDisposableCapabilityInvocationV2> {
    let bytes =
        canonical_bytes_v2(&invocation).context("serialize canonical capability invocation")?;
    let sha256 = sha256_hex_v2(&bytes);
    let canonical_json =
        String::from_utf8(bytes).context("canonical capability invocation is UTF-8")?;
    Ok(CanonicalDisposableCapabilityInvocationV2 {
        invocation,
        canonical_json,
        sha256,
    })
}

fn canonical_after(
    observation: DisposableCapabilityAfterObservationV2,
) -> Result<CanonicalDisposableCapabilityAfterV2> {
    let bytes = canonical_bytes_v2(&observation)
        .context("serialize canonical capability after-observation")?;
    let sha256 = sha256_hex_v2(&bytes);
    let canonical_json =
        String::from_utf8(bytes).context("canonical capability after-observation is UTF-8")?;
    Ok(CanonicalDisposableCapabilityAfterV2 {
        observation,
        canonical_json,
        sha256,
    })
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
        // SAFETY: every entry is one Security/CoreFoundation create/copy-rule reference.
        unsafe {
            for value in self.0.drain(..).rev() {
                CFRelease(value);
            }
        }
    }
}

unsafe fn cf_string(owned: &mut OwnedCf, value: &str) -> Result<CfType> {
    // SAFETY: UTF-8 bytes remain live for the call and the result follows the create rule.
    let result = unsafe {
        CFStringCreateWithBytes(
            kCFAllocatorDefault,
            value.as_bytes().as_ptr(),
            value.len() as isize,
            K_CF_STRING_ENCODING_UTF8,
            0,
        )
    };
    if result.is_null() {
        bail!("allocate CoreFoundation string")
    }
    Ok(owned.hold(result))
}

unsafe fn cf_data(owned: &mut OwnedCf, value: &[u8]) -> Result<CfType> {
    // SAFETY: bytes remain live for the call and the result follows the create rule.
    let result = unsafe { CFDataCreate(kCFAllocatorDefault, value.as_ptr(), value.len() as isize) };
    if result.is_null() {
        bail!("allocate CoreFoundation data")
    }
    Ok(owned.hold(result))
}

unsafe fn cf_array(owned: &mut OwnedCf, values: &[CfType]) -> Result<CfType> {
    // SAFETY: the input slice is valid for count; its values remain live in owned or framework
    // static storage for the lifetime of the returned array.
    let result = unsafe {
        CFArrayCreate(
            kCFAllocatorDefault,
            values.as_ptr(),
            values.len() as isize,
            ptr::null(),
        )
    };
    if result.is_null() {
        bail!("allocate CoreFoundation array")
    }
    Ok(owned.hold(result))
}

unsafe fn dictionary(
    owned: &mut OwnedCf,
    entries: &[(CfType, CfType)],
) -> Result<CfMutableDictionary> {
    // SAFETY: the dictionary remains owned and every key/value remains live while it is used.
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
        // SAFETY: result is mutable and both values remain live.
        unsafe { CFDictionarySetValue(result, *key, *value) };
    }
    Ok(result)
}

unsafe fn open_explicit_system_keychain(owned: &mut OwnedCf) -> Result<SecKeychain> {
    let path = CString::new(SYSTEM_KEYCHAIN_PATH).context("encode fixed System Keychain path")?;
    let mut keychain: SecKeychain = ptr::null();
    // SAFETY: path and the writable output pointer are valid.
    let status = unsafe { SecKeychainOpen(path.as_ptr(), &mut keychain) };
    if status != ERR_SEC_SUCCESS || keychain.is_null() {
        bail!("open explicit System Keychain failed with OSStatus {status}")
    }
    owned.hold(keychain);

    let mut returned = [0 as c_char; MAX_SYSTEM_KEYCHAIN_PATH_BYTES];
    let mut length = u32::try_from(returned.len()).context("bound System Keychain path")?;
    // SAFETY: returned is writable for length bytes and keychain is live.
    let status = unsafe { SecKeychainGetPath(keychain, &mut length, returned.as_mut_ptr()) };
    if status != ERR_SEC_SUCCESS {
        bail!("read explicit System Keychain path failed with OSStatus {status}")
    }
    let length = usize::try_from(length).context("decode System Keychain path length")?;
    if length >= returned.len() {
        bail!("explicit System Keychain returned an unterminated path")
    }
    // SAFETY: Security initialized the returned prefix.
    let bytes: &[u8] = unsafe { std::slice::from_raw_parts(returned.as_ptr().cast(), length) };
    if bytes != SYSTEM_KEYCHAIN_PATH.as_bytes() || returned[length] != 0 {
        bail!("opened keychain does not match the fixed System Keychain path")
    }
    Ok(keychain)
}

unsafe fn exact_private_key_query(
    owned: &mut OwnedCf,
    keychain: SecKeychain,
    application_tag: &[u8],
) -> Result<CfMutableDictionary> {
    // SAFETY: created values remain retained by owned.
    let tag = unsafe { cf_data(owned, application_tag)? };
    // SAFETY: keychain remains live.
    let search_list = unsafe { cf_array(owned, &[keychain.cast()])? };
    // SAFETY: all dynamic values remain live and framework keys are process-lifetime statics.
    unsafe {
        dictionary(
            owned,
            &[
                (kSecClass, kSecClassKey),
                (kSecAttrKeyClass, kSecAttrKeyClassPrivate),
                (kSecAttrApplicationTag, tag),
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

unsafe fn exact_private_key_match(
    owned: &mut OwnedCf,
    keychain: SecKeychain,
    application_tag: &[u8],
    subject: &str,
) -> Result<Option<(SecKey, CfType)>> {
    // SAFETY: query remains retained through the matching call.
    let query = unsafe { exact_private_key_query(owned, keychain, application_tag)? };
    let mut result: CfType = ptr::null();
    // SAFETY: query and output pointer are valid.
    let status = unsafe { SecItemCopyMatching(query.cast(), &mut result) };
    if status == ERR_SEC_ITEM_NOT_FOUND {
        return Ok(None);
    }
    if status != ERR_SEC_SUCCESS || result.is_null() {
        bail!("lookup {subject} failed with OSStatus {status}")
    }
    owned.hold(result);
    // SAFETY: result is a live CF object.
    if unsafe { CFGetTypeID(result) } != unsafe { CFArrayGetTypeID() } {
        bail!("{subject} lookup returned a non-array result")
    }
    // SAFETY: validated array.
    let count = unsafe { CFArrayGetCount(result) };
    if count != 1 {
        bail!("{subject} identity is ambiguous ({count} matches)")
    }
    // SAFETY: array has exactly one live element.
    let attributes = unsafe { CFArrayGetValueAtIndex(result, 0) };
    if attributes.is_null()
        || unsafe { CFGetTypeID(attributes) } != unsafe { CFDictionaryGetTypeID() }
    {
        bail!("{subject} lookup returned invalid attributes")
    }
    // SAFETY: validated dictionary and static key.
    let key: SecKey = unsafe { CFDictionaryGetValue(attributes, kSecValueRef) }.cast();
    if key.is_null() {
        bail!("{subject} lookup returned a null key reference")
    }
    // SAFETY: attributes and key remain live with result.
    unsafe { validate_private_p256_signer(owned, attributes, key, application_tag, subject)? };
    Ok(Some((key, attributes)))
}

unsafe fn validate_private_p256_signer(
    owned: &mut OwnedCf,
    persisted_attributes: CfType,
    key: SecKey,
    application_tag: &[u8],
    subject: &str,
) -> Result<()> {
    // SAFETY: comparison data remains owned.
    let expected_tag = unsafe { cf_data(owned, application_tag)? };
    // SAFETY: dictionary and expected value are live.
    unsafe {
        require_attribute(
            persisted_attributes,
            kSecAttrApplicationTag,
            expected_tag,
            subject,
            "application tag",
        )?;
    }
    // SAFETY: key is live and the returned dictionary follows the copy rule.
    let attributes = unsafe { SecKeyCopyAttributes(key) };
    if attributes.is_null() {
        bail!("{subject} has no inspectable key attributes")
    }
    owned.hold(attributes);
    // SAFETY: all values are live.
    unsafe {
        for (attribute, expected, label) in [
            (
                kSecAttrKeyClass,
                kSecAttrKeyClassPrivate,
                "private-key class",
            ),
            (
                kSecAttrKeyType,
                kSecAttrKeyTypeECSECPrimeRandom,
                "EC key type",
            ),
            (kSecAttrIsPermanent, kCFBooleanTrue, "permanent state"),
            (kSecAttrIsSensitive, kCFBooleanTrue, "sensitive state"),
            (
                kSecAttrIsExtractable,
                kCFBooleanFalse,
                "nonextractable state",
            ),
            (kSecAttrCanEncrypt, kCFBooleanFalse, "encrypt capability"),
            (kSecAttrCanDecrypt, kCFBooleanFalse, "decrypt capability"),
            (kSecAttrCanDerive, kCFBooleanFalse, "derive capability"),
            (kSecAttrCanSign, kCFBooleanTrue, "sign capability"),
            (kSecAttrCanVerify, kCFBooleanFalse, "verify capability"),
            (kSecAttrCanWrap, kCFBooleanFalse, "wrap capability"),
            (kSecAttrCanUnwrap, kCFBooleanFalse, "unwrap capability"),
        ] {
            require_attribute(attributes, attribute, expected, subject, label)?;
        }
    }
    // SAFETY: attributes is a live dictionary.
    let key_size = unsafe { CFDictionaryGetValue(attributes, kSecAttrKeySizeInBits) };
    let mut bits = 0_i64;
    // SAFETY: a non-null CFNumber is live and bits is writable.
    if key_size.is_null()
        || unsafe { CFNumberGetValue(key_size, K_CF_NUMBER_SINT64, (&mut bits as *mut i64).cast()) }
            == 0
        || bits != 256
    {
        bail!("{subject} is not exactly P-256")
    }
    Ok(())
}

unsafe fn require_attribute(
    attributes: CfType,
    key: CfType,
    expected: CfType,
    subject: &str,
    label: &str,
) -> Result<()> {
    // SAFETY: attributes is a live dictionary.
    let observed = unsafe { CFDictionaryGetValue(attributes, key) };
    if observed.is_null() || unsafe { CFEqual(observed, expected) } == 0 {
        bail!("{subject} has mismatched {label}")
    }
    Ok(())
}

unsafe fn observe_key(
    owned: &mut OwnedCf,
    key: SecKey,
    attributes: CfType,
    application_tag: &[u8],
    expected_target_access: Option<(&str, TargetSetKindV2)>,
) -> Result<KeyIntegrityObservationV2> {
    // SAFETY: attributes is a validated dictionary and framework keys are live statics.
    let label_value = unsafe { CFDictionaryGetValue(attributes, kSecAttrLabel) };
    if label_value.is_null() {
        bail!("disposable signer has no persisted label")
    }
    // SAFETY: the value is checked by copy_cf_string.
    let label = unsafe { copy_cf_string(label_value, "disposable signer label")? };
    if label.is_empty() || label.contains(['\0', '\n', '\r']) {
        bail!("disposable signer label is empty or unsafe")
    }

    // SAFETY: same validated dictionary and static key.
    let application_label_value =
        unsafe { CFDictionaryGetValue(attributes, kSecAttrApplicationLabel) };
    if application_label_value.is_null() {
        bail!("disposable signer has no application label")
    }
    // SAFETY: copy_cf_data validates the dynamic type.
    let application_label = unsafe {
        copy_cf_data(
            application_label_value,
            "disposable signer application label",
        )?
    };
    if application_label.is_empty() {
        bail!("disposable signer application label is empty")
    }

    // SAFETY: same validated dictionary and static key.
    let persistent_value = unsafe { CFDictionaryGetValue(attributes, kSecValuePersistentRef) };
    if persistent_value.is_null() {
        bail!("disposable signer has no persistent reference")
    }
    // SAFETY: copy_cf_data validates the dynamic type.
    let persistent_reference =
        unsafe { copy_cf_data(persistent_value, "disposable signer persistent reference")? };
    if persistent_reference.is_empty() {
        bail!("disposable signer persistent reference is empty")
    }

    // SAFETY: key is a live legacy keychain item and access is a writable copy-rule pointer.
    let access_sha256 = unsafe { copy_access_digest(owned, key)? };
    if let Some((expected, target_set_kind)) = expected_target_access {
        require_lower_hex_sha256(expected, "receipt signer access-control digest")?;
        if access_sha256 != expected {
            bail!("persisted target SecAccess digest differs from the signed receipt")
        }
        // SAFETY: creates the two fixed trusted-application identities and compares the entire
        // canonical snapshot, including owner, prompt, descriptions, applications, and auth tags.
        unsafe { validate_exact_target_access(owned, key, expected, target_set_kind)? };
    }

    let application_tag_sha256 = sha256_hex(application_tag);
    let application_label_base64url = URL_SAFE_NO_PAD.encode(application_label);
    let persistent_reference_sha256 = sha256_hex(&persistent_reference);
    // SAFETY: private key is live; this function copies only the public key and exports that public
    // object, never the private key reference.
    let public_spki_sha256 = unsafe { copy_public_spki_sha256(owned, key)? };
    let identity_sha256 = sha256_hex(
        &canonical_bytes_v2(&KeyIdentityMaterial {
            application_tag_sha256: &application_tag_sha256,
            label: &label,
            application_label_base64url: &application_label_base64url,
            persistent_reference_sha256: &persistent_reference_sha256,
            public_spki_sha256: &public_spki_sha256,
            access_control_sha256: &access_sha256,
        })
        .context("serialize disposable key identity")?,
    );
    Ok(KeyIntegrityObservationV2 {
        application_tag_sha256,
        label,
        application_label_base64url,
        persistent_reference_sha256,
        public_spki_sha256,
        access_control_sha256: access_sha256,
        identity_sha256,
    })
}

fn require_lower_hex_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is not an exact lowercase SHA-256")
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

unsafe fn copy_access_snapshot(owned: &mut OwnedCf, key: SecKey) -> Result<AccessSnapshot> {
    let mut access: SecAccess = ptr::null();
    // SAFETY: key is live and access is a writable copy-rule pointer.
    let status = unsafe { SecKeychainItemCopyAccess(key.cast(), &mut access) };
    if status != ERR_SEC_SUCCESS || access.is_null() {
        bail!("copy persisted disposable signer SecAccess failed with OSStatus {status}")
    }
    owned.hold(access);
    // SAFETY: copied access remains held by owned.
    unsafe { snapshot_access(owned, access) }
}

unsafe fn copy_access_digest(owned: &mut OwnedCf, key: SecKey) -> Result<String> {
    // SAFETY: key is live and no raw object escapes.
    let snapshot = unsafe { copy_access_snapshot(owned, key)? };
    canonical_access_digest(snapshot)
}

unsafe fn validate_exact_target_access(
    owned: &mut OwnedCf,
    key: SecKey,
    expected_sha256: &str,
    target_set_kind: TargetSetKindV2,
) -> Result<()> {
    // SAFETY: key is live and no raw object escapes.
    let snapshot = unsafe { copy_access_snapshot(owned, key)? };
    // SAFETY: paths are fixed compiled literals and returned applications/data remain held.
    let publisher_path = match target_set_kind {
        TargetSetKindV2::DisposableCapability => MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2,
        TargetSetKindV2::ProspectiveHost => MAC_R3_PRODUCT_PUBLISHER_PATH_V2,
    };
    let publisher = unsafe { trusted_application(owned, publisher_path)? };
    // SAFETY: same ownership rule.
    let finalizer = unsafe { trusted_application(owned, MAC_R3_FINALIZER_PATH_V2)? };
    // SAFETY: applications remain live.
    let publisher_data = unsafe { trusted_application_data(owned, publisher)? };
    // SAFETY: same ownership rule.
    let finalizer_data = unsafe { trusted_application_data(owned, finalizer)? };
    let mut expected = expected_target_snapshot(&publisher_data, &finalizer_data);
    canonicalize_snapshot(&mut expected);
    let mut observed = snapshot;
    canonicalize_snapshot(&mut observed);
    if observed != expected {
        bail!("persisted target SecAccess differs from the exact legacy ACL posture")
    }
    if observed
        .entries
        .iter()
        .flat_map(|entry| &entry.authorizations)
        .any(|authorization| authorization == AUTH_ANY)
    {
        bail!("persisted target SecAccess contains forbidden Any authorization")
    }
    if canonical_access_digest(observed)? != expected_sha256 {
        bail!("exact target SecAccess posture digest differs from the signed receipt")
    }
    Ok(())
}

fn expected_target_snapshot(publisher_data: &[u8], finalizer_data: &[u8]) -> AccessSnapshot {
    AccessSnapshot {
        owner_uid: UNMATCHABLE_OWNER_UID,
        owner_gid: UNMATCHABLE_OWNER_GID,
        owner_type: OWNER_TYPE_USE_UID_AND_GID,
        entries: vec![
            AclEntrySnapshot {
                description: PUBLISHER_SIGN_DELETE_DESCRIPTION.to_owned(),
                prompt_selector: PROMPT_SELECTOR_NONE,
                authorizations: vec![AUTH_SIGN.to_owned(), AUTH_DELETE.to_owned()],
                trusted_applications: vec![trusted_digest(publisher_data)],
            },
            AclEntrySnapshot {
                description: FINALIZER_DELETE_DESCRIPTION.to_owned(),
                prompt_selector: PROMPT_SELECTOR_NONE,
                authorizations: vec![AUTH_DELETE.to_owned()],
                trusted_applications: vec![trusted_digest(finalizer_data)],
            },
            AclEntrySnapshot {
                description: PRIVATE_OPERATION_DENY_DESCRIPTION.to_owned(),
                prompt_selector: PROMPT_SELECTOR_NONE,
                authorizations: vec![
                    AUTH_EXPORT_WRAPPED.to_owned(),
                    AUTH_EXPORT_CLEAR.to_owned(),
                    AUTH_IMPORT_WRAPPED.to_owned(),
                    AUTH_IMPORT_CLEAR.to_owned(),
                    AUTH_ENCRYPT.to_owned(),
                    AUTH_DECRYPT.to_owned(),
                    AUTH_MAC.to_owned(),
                    AUTH_DERIVE.to_owned(),
                ],
                trusted_applications: Vec::new(),
            },
            AclEntrySnapshot {
                description: OWNER_MUTATION_DENY_DESCRIPTION.to_owned(),
                prompt_selector: PROMPT_SELECTOR_NONE,
                authorizations: vec![AUTH_CHANGE_ACL.to_owned(), AUTH_CHANGE_OWNER.to_owned()],
                trusted_applications: Vec::new(),
            },
        ],
    }
}

fn trusted_digest(data: &[u8]) -> TrustedApplicationDigest {
    TrustedApplicationDigest {
        data_base64url: URL_SAFE_NO_PAD.encode(data),
    }
}

fn canonicalize_snapshot(snapshot: &mut AccessSnapshot) {
    for entry in &mut snapshot.entries {
        entry.authorizations.sort();
        entry.trusted_applications.sort();
    }
    snapshot.entries.sort();
}

fn canonical_access_digest(mut snapshot: AccessSnapshot) -> Result<String> {
    canonicalize_snapshot(&mut snapshot);
    // This intentionally matches the standalone ACL constructor's persisted digest schema.
    let json = serde_json::to_string(&snapshot).context("serialize canonical SecAccess")?;
    Ok(sha256_hex(json.as_bytes()))
}

unsafe fn snapshot_access(owned: &mut OwnedCf, access: SecAccess) -> Result<AccessSnapshot> {
    let mut owner_uid: libc::uid_t = 0;
    let mut owner_gid: libc::gid_t = 0;
    let mut owner_type = 0_u32;
    let mut acl_list: CfType = ptr::null();
    // SAFETY: access is live and all outputs are writable.
    let status = unsafe {
        SecAccessCopyOwnerAndACL(
            access,
            &mut owner_uid,
            &mut owner_gid,
            &mut owner_type,
            &mut acl_list,
        )
    };
    if status != ERR_SEC_SUCCESS || acl_list.is_null() {
        bail!("copy SecAccess owner/ACL failed with OSStatus {status}")
    }
    owned.hold(acl_list);
    // SAFETY: acl_list is a live CF object.
    if unsafe { CFGetTypeID(acl_list) } != unsafe { CFArrayGetTypeID() } {
        bail!("SecAccess ACL list is not an array")
    }
    // SAFETY: validated array.
    let count = unsafe { CFArrayGetCount(acl_list) };
    let mut entries = Vec::with_capacity(count as usize);
    for index in 0..count {
        // SAFETY: index is in bounds.
        let acl: SecAcl = unsafe { CFArrayGetValueAtIndex(acl_list, index) }.cast();
        if acl.is_null() || unsafe { CFGetTypeID(acl.cast()) } != unsafe { SecACLGetTypeID() } {
            bail!("SecAccess ACL list contains a non-SecACL value")
        }
        // SAFETY: ACL remains live with acl_list.
        entries.push(unsafe { snapshot_acl(owned, acl)? });
    }
    Ok(AccessSnapshot {
        owner_uid,
        owner_gid,
        owner_type,
        entries,
    })
}

unsafe fn snapshot_acl(owned: &mut OwnedCf, acl: SecAcl) -> Result<AclEntrySnapshot> {
    let mut applications: CfType = ptr::null();
    let mut description: CfType = ptr::null();
    let mut prompt_selector = u16::MAX;
    // SAFETY: ACL is live and outputs are writable copy-rule pointers.
    let status = unsafe {
        SecACLCopyContents(
            acl,
            &mut applications,
            &mut description,
            &mut prompt_selector,
        )
    };
    if status != ERR_SEC_SUCCESS || applications.is_null() || description.is_null() {
        bail!("copy SecACL contents failed with OSStatus {status}")
    }
    owned.hold(applications);
    owned.hold(description);
    // SAFETY: documented CFString.
    let description = unsafe { copy_cf_string(description, "SecACL description")? };

    // SAFETY: ACL is live and returned array follows the copy rule.
    let authorizations = unsafe { SecACLCopyAuthorizations(acl) };
    if authorizations.is_null() {
        bail!("copy SecACL authorizations returned null")
    }
    owned.hold(authorizations);
    if unsafe { CFGetTypeID(applications) } != unsafe { CFArrayGetTypeID() }
        || unsafe { CFGetTypeID(authorizations) } != unsafe { CFArrayGetTypeID() }
    {
        bail!("SecACL contents contain a non-array value")
    }

    // SAFETY: validated array.
    let authorization_count = unsafe { CFArrayGetCount(authorizations) };
    let mut authorization_names = Vec::with_capacity(authorization_count as usize);
    for index in 0..authorization_count {
        // SAFETY: index is in bounds and framework authorizations are live CFStrings.
        let authorization = unsafe { CFArrayGetValueAtIndex(authorizations, index) };
        // SAFETY: authorization is live through authorizations.
        authorization_names.push(unsafe { authorization_name(authorization)? }.to_owned());
    }

    // SAFETY: validated array.
    let application_count = unsafe { CFArrayGetCount(applications) };
    let mut trusted_applications = Vec::with_capacity(application_count as usize);
    for index in 0..application_count {
        // SAFETY: index is in bounds.
        let application = unsafe { CFArrayGetValueAtIndex(applications, index) };
        if application.is_null()
            || unsafe { CFGetTypeID(application) } != unsafe { SecTrustedApplicationGetTypeID() }
        {
            bail!("SecACL application list contains a non-trusted-application value")
        }
        // SAFETY: application remains live with applications.
        let data = unsafe { trusted_application_data(owned, application)? };
        trusted_applications.push(trusted_digest(&data));
    }
    Ok(AclEntrySnapshot {
        description,
        prompt_selector,
        authorizations: authorization_names,
        trusted_applications,
    })
}

unsafe fn authorization_name(value: CfType) -> Result<&'static str> {
    if value.is_null() {
        bail!("SecACL contains a null authorization")
    }
    // SAFETY: these are process-lifetime framework constants.
    let known = unsafe {
        [
            (kSecACLAuthorizationAny, AUTH_ANY),
            (kSecACLAuthorizationSign, AUTH_SIGN),
            (kSecACLAuthorizationDelete, AUTH_DELETE),
            (kSecACLAuthorizationExportWrapped, AUTH_EXPORT_WRAPPED),
            (kSecACLAuthorizationExportClear, AUTH_EXPORT_CLEAR),
            (kSecACLAuthorizationImportWrapped, AUTH_IMPORT_WRAPPED),
            (kSecACLAuthorizationImportClear, AUTH_IMPORT_CLEAR),
            (kSecACLAuthorizationEncrypt, AUTH_ENCRYPT),
            (kSecACLAuthorizationDecrypt, AUTH_DECRYPT),
            (kSecACLAuthorizationMAC, AUTH_MAC),
            (kSecACLAuthorizationDerive, AUTH_DERIVE),
            (kSecACLAuthorizationChangeACL, AUTH_CHANGE_ACL),
            (kSecACLAuthorizationChangeOwner, AUTH_CHANGE_OWNER),
        ]
    };
    for (expected, name) in known {
        // SAFETY: both values are live CF objects.
        if unsafe { CFEqual(value, expected) } != 0 {
            return Ok(name);
        }
    }
    bail!("SecACL contains an unknown authorization tag")
}

unsafe fn trusted_application(owned: &mut OwnedCf, path: &str) -> Result<SecTrustedApplication> {
    let path = CString::new(path).context("encode fixed trusted-application path")?;
    let mut application: SecTrustedApplication = ptr::null();
    // SAFETY: path and writable output pointer are valid.
    let status = unsafe { SecTrustedApplicationCreateFromPath(path.as_ptr(), &mut application) };
    if status != ERR_SEC_SUCCESS || application.is_null() {
        bail!("create fixed trusted application failed with OSStatus {status}")
    }
    Ok(owned.hold(application))
}

unsafe fn trusted_application_data(
    owned: &mut OwnedCf,
    application: SecTrustedApplication,
) -> Result<Vec<u8>> {
    let mut data: CfType = ptr::null();
    // SAFETY: application is live and data is a writable copy-rule pointer.
    let status = unsafe { SecTrustedApplicationCopyData(application, &mut data) };
    if status != ERR_SEC_SUCCESS || data.is_null() {
        bail!("copy trusted-application identity data failed with OSStatus {status}")
    }
    owned.hold(data);
    // SAFETY: documented CFData result.
    unsafe { copy_cf_data(data, "trusted-application identity") }
}

unsafe fn build_forbidden_replacement_access(owned: &mut OwnedCf) -> Result<SecAccess> {
    let mut error: CfType = ptr::null();
    // SAFETY: scalar owner fields are exact and error is writable. A null ACL list constructs an
    // empty SecAccess to which this module adds one explicit replacement ACL.
    let access = unsafe {
        SecAccessCreateWithOwnerAndACL(
            UNMATCHABLE_OWNER_UID,
            UNMATCHABLE_OWNER_GID,
            OWNER_TYPE_USE_UID_AND_GID,
            ptr::null(),
            &mut error,
        )
    };
    let error_code = unsafe { take_cf_error_code(owned, error) };
    if access.is_null() {
        bail!("create forbidden replacement SecAccess failed with CFError code {error_code}")
    }
    owned.hold(access);
    // Preserve the disposable publisher's independently usable rollback Delete authority even in
    // the unexpected-success case. The mutation remains forbidden because it would grant the
    // finalizer Sign and replace the exact target posture.
    // SAFETY: fixed publisher path and resulting object remain owned.
    let publisher = unsafe { trusted_application(owned, MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2)? };
    // SAFETY: application remains live.
    let publisher_applications = unsafe { cf_array(owned, &[publisher.cast()])? };
    // SAFETY: description remains owned.
    let publisher_description = unsafe { cf_string(owned, PUBLISHER_SIGN_DELETE_DESCRIPTION)? };
    let mut publisher_acl: SecAcl = ptr::null();
    // SAFETY: access, application array, and description all remain live.
    let status = unsafe {
        SecACLCreateWithSimpleContents(
            access,
            publisher_applications,
            publisher_description,
            PROMPT_SELECTOR_NONE,
            &mut publisher_acl,
        )
    };
    if status != ERR_SEC_SUCCESS || publisher_acl.is_null() {
        bail!("create rollback-preserving publisher ACL failed with OSStatus {status}")
    }
    owned.hold(publisher_acl);
    // SAFETY: authorization statics remain live.
    let publisher_authorizations = unsafe {
        cf_array(
            owned,
            &[kSecACLAuthorizationSign, kSecACLAuthorizationDelete],
        )?
    };
    // SAFETY: both ACL and authorization array are live.
    let status = unsafe { SecACLUpdateAuthorizations(publisher_acl, publisher_authorizations) };
    if status != ERR_SEC_SUCCESS {
        bail!("set rollback-preserving publisher authorizations failed with OSStatus {status}")
    }

    // SAFETY: fixed finalizer path and resulting object remain owned.
    let finalizer = unsafe { trusted_application(owned, MAC_R3_FINALIZER_PATH_V2)? };
    // SAFETY: application remains live.
    let applications = unsafe { cf_array(owned, &[finalizer.cast()])? };
    // SAFETY: description remains owned.
    let description = unsafe { cf_string(owned, REPLACEMENT_DESCRIPTION)? };
    let mut acl: SecAcl = ptr::null();
    // SAFETY: access, application array, and description all remain live.
    let status = unsafe {
        SecACLCreateWithSimpleContents(
            access,
            applications,
            description,
            PROMPT_SELECTOR_NONE,
            &mut acl,
        )
    };
    if status != ERR_SEC_SUCCESS || acl.is_null() {
        bail!("create forbidden replacement ACL failed with OSStatus {status}")
    }
    owned.hold(acl);
    // SAFETY: authorization static remains live.
    let authorizations = unsafe { cf_array(owned, &[kSecACLAuthorizationSign])? };
    // SAFETY: both ACL and authorization array are live.
    let status = unsafe { SecACLUpdateAuthorizations(acl, authorizations) };
    if status != ERR_SEC_SUCCESS {
        bail!("set forbidden replacement authorization failed with OSStatus {status}")
    }
    Ok(access)
}

unsafe fn attempt_sign(owned: &mut OwnedCf, key: SecKey, payload: &[u8]) -> Result<i64> {
    // SAFETY: created data remains held through the Security call.
    let payload = unsafe { cf_data(owned, payload)? };
    let mut error: CfType = ptr::null();
    // SAFETY: key, algorithm, and data are live; error is writable.
    let signature = unsafe {
        SecKeyCreateSignature(
            key,
            kSecKeyAlgorithmECDSASignatureMessageX962SHA256,
            payload,
            &mut error,
        )
    };
    if !signature.is_null() {
        owned.hold(signature);
        if !error.is_null() {
            owned.hold(error);
        }
        bail!("disposable finalizer signing unexpectedly succeeded")
    }
    if error.is_null() {
        bail!("disposable finalizer signing denied without a CFError")
    }
    // SAFETY: error is a live CFError returned by Security.
    let code = unsafe { CFErrorGetCode(error) } as i64;
    owned.hold(error);
    Ok(code)
}

unsafe fn attempt_private_export(owned: &mut OwnedCf, key: SecKey) -> Result<i64> {
    let mut error: CfType = ptr::null();
    // SAFETY: key is live and error is writable.
    let exported = unsafe { SecKeyCopyExternalRepresentation(key, &mut error) };
    if !exported.is_null() {
        owned.hold(exported);
        if !error.is_null() {
            owned.hold(error);
        }
        bail!("disposable finalizer private export unexpectedly succeeded")
    }
    if error.is_null() {
        bail!("disposable finalizer private export denied without a CFError")
    }
    // SAFETY: error is a live CFError returned by Security.
    let code = unsafe { CFErrorGetCode(error) } as i64;
    owned.hold(error);
    Ok(code)
}

unsafe fn exact_delete_query(
    owned: &mut OwnedCf,
    keychain: SecKeychain,
    key: SecKey,
    application_tag: &[u8],
) -> Result<CfMutableDictionary> {
    // SAFETY: all created objects remain retained by owned.
    let tag = unsafe { cf_data(owned, application_tag)? };
    // SAFETY: keychain remains live.
    let search_list = unsafe { cf_array(owned, &[keychain.cast()])? };
    // SAFETY: exact key remains live through the deletion call.
    let item_list = unsafe { cf_array(owned, &[key.cast()])? };
    // SAFETY: all values remain live and keys are framework statics.
    unsafe {
        dictionary(
            owned,
            &[
                (kSecClass, kSecClassKey),
                (kSecAttrKeyClass, kSecAttrKeyClassPrivate),
                (kSecAttrApplicationTag, tag),
                (kSecMatchSearchList, search_list),
                (kSecMatchItemList, item_list),
                (kSecUseAuthenticationUI, kSecUseAuthenticationUIFail),
            ],
        )
    }
}

unsafe fn copy_cf_data(value: CfType, label: &str) -> Result<Vec<u8>> {
    // SAFETY: value is a live CF object.
    if unsafe { CFGetTypeID(value) } != unsafe { CFDataGetTypeID() } {
        bail!("{label} is not CFData")
    }
    // SAFETY: validated CFData.
    let length = unsafe { CFDataGetLength(value) };
    // SAFETY: same live CFData.
    let bytes = unsafe { CFDataGetBytePtr(value) };
    if length < 0 || (length > 0 && bytes.is_null()) {
        bail!("{label} has invalid bytes")
    }
    if length == 0 {
        return Ok(Vec::new());
    }
    // SAFETY: bytes is valid for length while value remains live.
    Ok(unsafe { std::slice::from_raw_parts(bytes, length as usize) }.to_vec())
}

unsafe fn copy_public_spki_sha256(owned: &mut OwnedCf, private_key: SecKey) -> Result<String> {
    // SAFETY: private_key is live; returned public key follows the create/copy ownership rule.
    let public_key = unsafe { SecKeyCopyPublicKey(private_key) };
    if public_key.is_null() {
        bail!("exact signer has no copyable public key")
    }
    owned.hold(public_key);
    let mut error: CfType = ptr::null();
    // SAFETY: this is the derived public key, never the private key reference.
    let external = unsafe { SecKeyCopyExternalRepresentation(public_key, &mut error) };
    let error_code = unsafe { take_cf_error_code(owned, error) };
    if external.is_null() {
        bail!("copy exact signer public representation failed with CFError code {error_code}")
    }
    owned.hold(external);
    // SAFETY: documented CFData result.
    let raw = unsafe { copy_cf_data(external, "P-256 public representation")? };
    let spki = p256_spki_der(&raw)?;
    Ok(sha256_hex(&spki))
}

fn p256_spki_der(raw: &[u8]) -> Result<Vec<u8>> {
    if raw.len() != 65 || raw[0] != 0x04 {
        bail!("exact signer public representation is not uncompressed P-256")
    }
    const P256_SPKI_PREFIX: [u8; 26] = [
        0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, 0x06, 0x08,
        0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, 0x03, 0x42, 0x00,
    ];
    let mut spki = Vec::with_capacity(P256_SPKI_PREFIX.len() + raw.len());
    spki.extend_from_slice(&P256_SPKI_PREFIX);
    spki.extend_from_slice(raw);
    Ok(spki)
}

unsafe fn copy_cf_string(value: CfType, label: &str) -> Result<String> {
    // SAFETY: value is a live CF object.
    if unsafe { CFGetTypeID(value) } != unsafe { CFStringGetTypeID() } {
        bail!("{label} is not CFString")
    }
    // SAFETY: validated CFString.
    let characters = unsafe { CFStringGetLength(value) };
    // SAFETY: validated length and encoding.
    let maximum =
        unsafe { CFStringGetMaximumSizeForEncoding(characters, K_CF_STRING_ENCODING_UTF8) };
    if characters < 0 || maximum < 0 {
        bail!("{label} has invalid length")
    }
    let mut bytes = vec![0 as c_char; maximum as usize + 1];
    // SAFETY: destination is writable for its full length.
    if unsafe {
        CFStringGetCString(
            value,
            bytes.as_mut_ptr(),
            bytes.len() as isize,
            K_CF_STRING_ENCODING_UTF8,
        )
    } == 0
    {
        bail!("decode {label} as UTF-8")
    }
    // SAFETY: successful CFStringGetCString NUL-terminated the destination.
    Ok(unsafe { std::ffi::CStr::from_ptr(bytes.as_ptr()) }
        .to_str()
        .with_context(|| format!("{label} is not UTF-8"))?
        .to_owned())
}

unsafe fn take_cf_error_code(owned: &mut OwnedCf, error: CfType) -> isize {
    if error.is_null() {
        0
    } else {
        // SAFETY: error is a live CFError returned by Security.
        let code = unsafe { CFErrorGetCode(error) };
        owned.hold(error);
        code
    }
}

#[link(name = "Security", kind = "framework")]
unsafe extern "C" {
    static kSecClass: CfType;
    static kSecClassKey: CfType;
    static kSecAttrApplicationTag: CfType;
    static kSecAttrApplicationLabel: CfType;
    static kSecAttrLabel: CfType;
    static kSecAttrKeyClass: CfType;
    static kSecAttrKeyClassPrivate: CfType;
    static kSecAttrKeyType: CfType;
    static kSecAttrKeyTypeECSECPrimeRandom: CfType;
    static kSecAttrKeySizeInBits: CfType;
    static kSecAttrIsPermanent: CfType;
    static kSecAttrIsSensitive: CfType;
    static kSecAttrIsExtractable: CfType;
    static kSecAttrCanEncrypt: CfType;
    static kSecAttrCanDecrypt: CfType;
    static kSecAttrCanDerive: CfType;
    static kSecAttrCanSign: CfType;
    static kSecAttrCanVerify: CfType;
    static kSecAttrCanWrap: CfType;
    static kSecAttrCanUnwrap: CfType;
    static kSecValueRef: CfType;
    static kSecValuePersistentRef: CfType;
    static kSecReturnAttributes: CfType;
    static kSecReturnRef: CfType;
    static kSecReturnPersistentRef: CfType;
    static kSecMatchSearchList: CfType;
    static kSecMatchItemList: CfType;
    static kSecMatchLimit: CfType;
    static kSecMatchLimitAll: CfType;
    static kSecUseAuthenticationUI: CfType;
    static kSecUseAuthenticationUIFail: CfType;
    static kSecKeyAlgorithmECDSASignatureMessageX962SHA256: CfType;

    static kSecACLAuthorizationAny: CfType;
    static kSecACLAuthorizationDelete: CfType;
    static kSecACLAuthorizationExportWrapped: CfType;
    static kSecACLAuthorizationExportClear: CfType;
    static kSecACLAuthorizationImportWrapped: CfType;
    static kSecACLAuthorizationImportClear: CfType;
    static kSecACLAuthorizationSign: CfType;
    static kSecACLAuthorizationEncrypt: CfType;
    static kSecACLAuthorizationDecrypt: CfType;
    static kSecACLAuthorizationMAC: CfType;
    static kSecACLAuthorizationDerive: CfType;
    static kSecACLAuthorizationChangeACL: CfType;
    static kSecACLAuthorizationChangeOwner: CfType;

    fn SecKeychainOpen(path_name: *const c_char, keychain: *mut SecKeychain) -> OsStatus;
    fn SecKeychainGetPath(
        keychain: SecKeychain,
        path_length: *mut u32,
        path_name: *mut c_char,
    ) -> OsStatus;
    fn SecAccessCreateWithOwnerAndACL(
        user_id: libc::uid_t,
        group_id: libc::gid_t,
        owner_type: u32,
        acls: CfType,
        error: *mut CfType,
    ) -> SecAccess;
    fn SecAccessCopyOwnerAndACL(
        access: SecAccess,
        user_id: *mut libc::uid_t,
        group_id: *mut libc::gid_t,
        owner_type: *mut u32,
        acl_list: *mut CfType,
    ) -> OsStatus;
    fn SecACLGetTypeID() -> usize;
    fn SecACLCreateWithSimpleContents(
        access: SecAccess,
        application_list: CfType,
        description: CfType,
        prompt_selector: u16,
        acl: *mut SecAcl,
    ) -> OsStatus;
    fn SecACLUpdateAuthorizations(acl: SecAcl, authorizations: CfType) -> OsStatus;
    fn SecACLCopyAuthorizations(acl: SecAcl) -> CfType;
    fn SecACLCopyContents(
        acl: SecAcl,
        application_list: *mut CfType,
        description: *mut CfType,
        prompt_selector: *mut u16,
    ) -> OsStatus;
    fn SecTrustedApplicationGetTypeID() -> usize;
    fn SecTrustedApplicationCreateFromPath(
        path: *const c_char,
        application: *mut SecTrustedApplication,
    ) -> OsStatus;
    fn SecTrustedApplicationCopyData(
        application: SecTrustedApplication,
        data: *mut CfType,
    ) -> OsStatus;
    fn SecKeyCopyAttributes(key: SecKey) -> CfType;
    fn SecKeyCopyPublicKey(key: SecKey) -> SecKey;
    fn SecKeyCreateSignature(
        key: SecKey,
        algorithm: CfType,
        data: CfType,
        error: *mut CfType,
    ) -> CfType;
    fn SecKeyCopyExternalRepresentation(key: SecKey, error: *mut CfType) -> CfType;
    fn SecItemCopyMatching(query: CfType, result: *mut CfType) -> OsStatus;
    fn SecItemDelete(query: CfType) -> OsStatus;
    fn SecKeychainItemCopyAccess(item: CfType, access: *mut SecAccess) -> OsStatus;
    fn SecKeychainItemSetAccess(item: CfType, access: SecAccess) -> OsStatus;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFAllocatorDefault: CfType;
    static kCFBooleanTrue: CfType;
    static kCFBooleanFalse: CfType;
    fn CFRelease(value: CfType);
    fn CFGetTypeID(value: CfType) -> usize;
    fn CFEqual(left: CfType, right: CfType) -> u8;
    fn CFErrorGetCode(error: CfType) -> isize;
    fn CFStringGetTypeID() -> usize;
    fn CFStringCreateWithBytes(
        allocator: CfType,
        bytes: *const u8,
        byte_count: isize,
        encoding: u32,
        external_representation: u8,
    ) -> CfType;
    fn CFStringGetLength(value: CfType) -> isize;
    fn CFStringGetMaximumSizeForEncoding(length: isize, encoding: u32) -> isize;
    fn CFStringGetCString(
        value: CfType,
        buffer: *mut c_char,
        buffer_size: isize,
        encoding: u32,
    ) -> u8;
    fn CFDataGetTypeID() -> usize;
    fn CFDataCreate(allocator: CfType, bytes: *const u8, byte_count: isize) -> CfType;
    fn CFDataGetLength(data: CfType) -> isize;
    fn CFDataGetBytePtr(data: CfType) -> *const u8;
    fn CFArrayGetTypeID() -> usize;
    fn CFArrayCreate(
        allocator: CfType,
        values: *const CfType,
        value_count: isize,
        callbacks: CfType,
    ) -> CfType;
    fn CFArrayGetCount(array: CfType) -> isize;
    fn CFArrayGetValueAtIndex(array: CfType, index: isize) -> CfType;
    fn CFDictionaryGetTypeID() -> usize;
    fn CFDictionaryCreateMutable(
        allocator: CfType,
        capacity: isize,
        key_callbacks: CfType,
        value_callbacks: CfType,
    ) -> CfMutableDictionary;
    fn CFDictionarySetValue(dictionary: CfMutableDictionary, key: CfType, value: CfType);
    fn CFDictionaryGetValue(dictionary: CfType, key: CfType) -> CfType;
    fn CFNumberGetValue(number: CfType, number_type: i32, value: *mut c_void) -> u8;
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCOPE: &str = "019fffe0-0000-7000-8000-000000000001";

    #[test]
    fn disposable_surface_is_closed_and_recovery_is_fail_closed() {
        assert!(disposable_capability_sequence_for(TargetSetKindV2::ProspectiveHost).is_empty());
        assert_eq!(
            disposable_capability_sequence_for(TargetSetKindV2::DisposableCapability),
            &[
                DisposableCapabilityControlV2::Sign,
                DisposableCapabilityControlV2::ExportPrivate,
                DisposableCapabilityControlV2::ReplaceAccess,
                DisposableCapabilityControlV2::DeleteWrongKey,
            ]
        );
        assert!(derive_disposable_tags(TargetSetKindV2::ProspectiveHost, SCOPE).is_err());
        let (target, wrong) =
            derive_disposable_tags(TargetSetKindV2::DisposableCapability, SCOPE).unwrap();
        assert_eq!(target, format!("{SCOPE}:signing-key").as_bytes());
        assert_eq!(
            wrong,
            format!("{SCOPE}{WRONG_SURROGATE_TAG_SUFFIX}").as_bytes()
        );

        assert_eq!(
            require_frozen_denial(DisposableCapabilityControlV2::Sign, ERR_SEC_AUTH_FAILED)
                .unwrap()
                .0,
            FrozenDenialClassV2::AuthorizationFailed
        );
        assert!(require_frozen_denial(
            DisposableCapabilityControlV2::Sign,
            ERR_SEC_DATA_NOT_AVAILABLE
        )
        .is_err());
        assert_eq!(
            require_frozen_denial(
                DisposableCapabilityControlV2::ExportPrivate,
                ERR_SEC_DATA_NOT_AVAILABLE
            )
            .unwrap()
            .0,
            FrozenDenialClassV2::DataNotAvailable
        );
        let mut raw_public = vec![0_u8; 65];
        raw_public[0] = 0x04;
        let spki = p256_spki_der(&raw_public).unwrap();
        assert_eq!(spki.len(), 91);
        assert_eq!(&spki[26..], raw_public.as_slice());
        assert!(p256_spki_der(&raw_public[..64]).is_err());

        let key = KeyIntegrityObservationV2 {
            application_tag_sha256: "11".repeat(32),
            label: "surrogate".to_owned(),
            application_label_base64url: "AQ".to_owned(),
            persistent_reference_sha256: "22".repeat(32),
            public_spki_sha256: "66".repeat(32),
            access_control_sha256: "33".repeat(32),
            identity_sha256: "44".repeat(32),
        };
        let before = canonical_state(DisposableCapabilityStateObservationV2 {
            schema_owner: REPORT_OWNER.to_owned(),
            schema_version: REPORT_VERSION,
            target_set_kind: TargetSetKindV2::DisposableCapability,
            scope_id: SCOPE.to_owned(),
            signer_access_control_sha256: "33".repeat(32),
            target: key.clone(),
            wrong_surrogate: key,
        })
        .unwrap();
        before.require_same_keys(&before).unwrap();
        let mut changed = before.clone();
        changed.observation.wrong_surrogate.identity_sha256 = "55".repeat(32);
        changed = canonical_state(changed.observation).unwrap();
        assert!(before.require_same_keys(&changed).is_err());

        let production = concat!(
            include_str!("native_effects.rs"),
            include_str!("engine.rs"),
            include_str!("bin/finalizer.rs"),
            include_str!("bin/coordinator.rs")
        );
        assert!(production.contains("FixedTarget::DisposableCapabilityControl"));
        assert!(production.contains("TargetSetKindV2::DisposableCapability"));
        assert!(production.contains("invoke_one_disposable_capability_control"));
        for forbidden in [
            "SecKeyCreateSignature",
            "SecKeyCopyExternalRepresentation",
            "SecKeychainItemSetAccess",
        ] {
            assert!(
                !production.contains(forbidden),
                "prospective production surface unexpectedly reaches {forbidden}"
            );
        }
    }

    #[test]
    fn exact_acl_posture_is_lane_specific_sign_delete_and_finalizer_delete_only() {
        let mut snapshot = expected_target_snapshot(b"publisher", b"finalizer");
        canonicalize_snapshot(&mut snapshot);
        assert_eq!(snapshot.owner_uid, u32::MAX);
        assert_eq!(snapshot.owner_gid, u32::MAX);
        assert_eq!(snapshot.owner_type, OWNER_TYPE_USE_UID_AND_GID);
        let publisher = snapshot
            .entries
            .iter()
            .find(|entry| entry.description == PUBLISHER_SIGN_DELETE_DESCRIPTION)
            .unwrap();
        let finalizer = snapshot
            .entries
            .iter()
            .find(|entry| entry.description == FINALIZER_DELETE_DESCRIPTION)
            .unwrap();
        assert_eq!(publisher.authorizations, [AUTH_DELETE, AUTH_SIGN]);
        assert_eq!(finalizer.authorizations, [AUTH_DELETE]);
        assert_ne!(
            MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2,
            MAC_R3_PRODUCT_PUBLISHER_PATH_V2
        );
        assert_ne!(
            MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2,
            MAC_R3_FINALIZER_PATH_V2
        );
        let source = include_str!("disposable_capability.rs");
        assert!(source.contains("TargetSetKindV2::DisposableCapability =>"));
        assert!(source.contains("MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2"));
        assert!(source.contains("TargetSetKindV2::ProspectiveHost =>"));
        assert!(source.contains("MAC_R3_PRODUCT_PUBLISHER_PATH_V2"));
        let replacement = source
            .split("unsafe fn build_forbidden_replacement_access")
            .nth(1)
            .unwrap()
            .split("unsafe fn attempt_sign")
            .next()
            .unwrap();
        assert!(replacement.contains("MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2"));
        assert!(replacement.contains("PUBLISHER_SIGN_DELETE_DESCRIPTION"));
        assert!(replacement.contains("kSecACLAuthorizationDelete"));
        assert!(replacement.contains("MAC_R3_FINALIZER_PATH_V2"));
        assert!(replacement.contains("REPLACEMENT_DESCRIPTION"));
        assert!(replacement.contains("kSecACLAuthorizationSign"));
    }
}
