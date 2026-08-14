use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use p256::ecdsa::Signature as P256Signature;
use std::ffi::{c_char, c_void, CString};
use std::ptr;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::{
    AccessSnapshot, AclEntrySnapshot, CanonicalAccessDigest, DisposableSignerIdentityV2,
    ExactDeleteClassification, ExactDeleteReceipt, ExactSignerConfig, NegativeControl,
    NegativeControlOutcome, ProductEquivalentCreationReceipt, SignerCreationReadback,
    SignerReadback, TrustedApplicationDigest, AUTH_ANY, AUTH_CHANGE_ACL, AUTH_CHANGE_OWNER,
    AUTH_DECRYPT, AUTH_DELETE, AUTH_DERIVE, AUTH_ENCRYPT, AUTH_EXPORT_CLEAR, AUTH_EXPORT_WRAPPED,
    AUTH_IMPORT_CLEAR, AUTH_IMPORT_WRAPPED, AUTH_MAC, AUTH_SIGN, FINALIZER_DELETE_DESCRIPTION,
    MAX_SIGNING_PAYLOAD_BYTES, NONMATCH_OWNER_GID, NONMATCH_OWNER_UID,
    OWNER_MUTATION_DENY_DESCRIPTION, OWNER_TYPE_USE_ONLY_UID_AND_GID,
    PRIVATE_OPERATION_DENY_DESCRIPTION, PROMPT_SELECTOR_NONE, PUBLISHER_SIGN_DELETE_DESCRIPTION,
    SYSTEM_KEYCHAIN_PATH,
};
use substrate_common::macos_retirement_v2::{canonical_bytes_v2, sha256_hex_v2};
use substrate_r3_macos_finalizer::experiment::controls::NobodyOwnerAuthorityOperationV2;

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
const ERR_SEC_AUTH_FAILED: OsStatus = -25293;
const ERR_SEC_INTERACTION_NOT_ALLOWED: OsStatus = -25308;
const K_CF_NUMBER_SINT64: i32 = 4;
const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
const MAX_SYSTEM_KEYCHAIN_PATH_BYTES: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DisposableAclKind {
    Target,
    WrongSurrogate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExactAbsentKeyAttribute {
    ApplicationTag,
    Label,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FixedP256SignatureV2 {
    pub public_spki_der_base64url: String,
    pub signature_p1363_low_s_base64url: String,
}

#[derive(serde::Serialize)]
#[serde(deny_unknown_fields)]
struct KeyIdentityMaterial<'a> {
    application_tag_sha256: &'a str,
    label: &'a str,
    application_label_base64url: &'a str,
    persistent_reference_sha256: &'a str,
    public_spki_sha256: &'a str,
    access_control_sha256: &'a str,
}

// 0 = untouched, 1 = first Security call in progress, 2 = UI denied, 3 = poisoned,
// 4 = the process is permanently reserved for query-level UI-fail calls.
static SECURITY_UI_STATE: AtomicU8 = AtomicU8::new(0);

#[derive(Debug)]
pub struct NonInteractiveSecurity {
    _private: (),
}

/// A process mode that never changes Security's global interaction setting.  All native queries
/// reachable from this token carry `kSecUseAuthenticationUIFail`.  Claiming this mode prevents a
/// later false assertion that process-level denial was the process's first Security call.
#[derive(Debug)]
pub struct QueryUiFailSecurity {
    _private: (),
}

impl NonInteractiveSecurity {
    /// Disable process-wide Keychain interaction as this crate's first Security call.
    pub fn establish_first() -> Result<Self> {
        match SECURITY_UI_STATE.compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => {
                // SAFETY: scalar process-global API; the atomic admits exactly one first call.
                let status = unsafe { SecKeychainSetUserInteractionAllowed(0) };
                if status != ERR_SEC_SUCCESS {
                    SECURITY_UI_STATE.store(3, Ordering::SeqCst);
                    bail!(
                        "disable Keychain interaction as first Security call failed with OSStatus {status}"
                    )
                }
                SECURITY_UI_STATE.store(2, Ordering::SeqCst);
                Ok(Self { _private: () })
            }
            Err(2) => Ok(Self { _private: () }),
            Err(1) => bail!("Security interaction denial is concurrently initializing"),
            Err(3) => bail!("the first Security interaction-denial call previously failed"),
            Err(4) => {
                bail!("query-level UI-fail Security mode was already claimed in this process")
            }
            Err(other) => bail!("invalid Security interaction state {other}"),
        }
    }

    /// Create the exact permanent P-256 signer with a direct `kSecAttrAccess` dictionary value,
    /// then require strict in-memory and persisted ACL readback equality.
    pub fn create_signer(&mut self, config: &ExactSignerConfig) -> Result<SignerCreationReadback> {
        config.validate()?;
        // SAFETY: every create/copy-rule CF object remains in owned through its final use.
        unsafe {
            let mut owned = OwnedCf::new();
            let keychain = open_explicit_system_keychain(&mut owned)?;
            if exact_private_key_match(&mut owned, keychain, config)?.is_some() {
                bail!("exact experiment signer already exists")
            }

            let (access, in_memory_access) = build_exact_access(&mut owned, config)?;
            let _created = create_exact_key(&mut owned, keychain, config, Some(access))?;

            let (persisted_key, _) = exact_private_key_match(&mut owned, keychain, config)?
                .context("created signer did not reopen through its exact tag")?;
            let persisted_access = copy_and_validate_access(&mut owned, persisted_key, config)?;
            if in_memory_access != persisted_access {
                bail!("persisted SecAccess readback differs from the in-memory posture")
            }
            Ok(SignerCreationReadback {
                in_memory_access,
                persisted: signer_readback(config, persisted_access),
            })
        }
    }

    /// Read and strictly validate the exact signer and its persisted SecAccess posture.
    pub fn read_signer(&self, config: &ExactSignerConfig) -> Result<Option<SignerReadback>> {
        config.validate()?;
        // SAFETY: no raw reference escapes owned.
        unsafe {
            let mut owned = OwnedCf::new();
            let keychain = open_explicit_system_keychain(&mut owned)?;
            let Some((key, _)) = exact_private_key_match(&mut owned, keychain, config)? else {
                return Ok(None);
            };
            let access = copy_and_validate_access(&mut owned, key, config)?;
            Ok(Some(signer_readback(config, access)))
        }
    }

    pub(crate) fn create_disposable_signer(
        &mut self,
        config: &ExactSignerConfig,
        acl_kind: DisposableAclKind,
    ) -> Result<DisposableSignerIdentityV2> {
        config.validate()?;
        // SAFETY: all native objects remain held and only public/key-identity bytes escape.
        unsafe {
            let mut owned = OwnedCf::new();
            let keychain = open_explicit_system_keychain(&mut owned)?;
            if exact_private_key_match(&mut owned, keychain, config)?.is_some() {
                bail!("exact disposable signer already exists")
            }
            let (access, _) = build_disposable_access(&mut owned, config, acl_kind)?;
            let _created = create_exact_key(&mut owned, keychain, config, Some(access))?;
            let (key, attributes) = exact_private_key_match(&mut owned, keychain, config)?
                .context("created disposable signer did not reopen through its exact identity")?;
            disposable_signer_identity(&mut owned, key, attributes, config, acl_kind)
        }
    }

    pub(crate) fn read_disposable_signer(
        &self,
        config: &ExactSignerConfig,
        acl_kind: DisposableAclKind,
    ) -> Result<Option<DisposableSignerIdentityV2>> {
        config.validate()?;
        // SAFETY: all native objects remain held and only public/key-identity bytes escape.
        unsafe {
            let mut owned = OwnedCf::new();
            let keychain = open_explicit_system_keychain(&mut owned)?;
            let Some((key, attributes)) = exact_private_key_match(&mut owned, keychain, config)?
            else {
                return Ok(None);
            };
            Ok(Some(disposable_signer_identity(
                &mut owned, key, attributes, config, acl_kind,
            )?))
        }
    }

    /// Prove one exact compiled private-key application-tag or label predicate is absent. The
    /// query is restricted to the explicit System Keychain, private-key class, UI-fail, and an
    /// exact attribute value. Success or any alternate status is rejected as stale/ambiguous.
    pub(crate) fn require_exact_key_attribute_absent(
        &self,
        attribute: ExactAbsentKeyAttribute,
        value: &[u8],
    ) -> Result<OsStatus> {
        if value.is_empty() || value.len() > 1024 {
            bail!("exact absent key attribute is empty or exceeds its fixed bound")
        }
        // SAFETY: every CF object remains held through the exact query and no reference escapes.
        unsafe {
            let mut owned = OwnedCf::new();
            let keychain = open_explicit_system_keychain(&mut owned)?;
            let value = match attribute {
                ExactAbsentKeyAttribute::ApplicationTag => cf_data(&mut owned, value)?,
                ExactAbsentKeyAttribute::Label => {
                    let value = std::str::from_utf8(value)
                        .context("exact absent key label is not UTF-8")?;
                    cf_string(&mut owned, value)?
                }
            };
            let attribute_key = match attribute {
                ExactAbsentKeyAttribute::ApplicationTag => kSecAttrApplicationTag,
                ExactAbsentKeyAttribute::Label => kSecAttrLabel,
            };
            let search_list = cf_array(&mut owned, &[keychain.cast()])?;
            let query = dictionary(
                &mut owned,
                &[
                    (kSecClass, kSecClassKey),
                    (kSecAttrKeyClass, kSecAttrKeyClassPrivate),
                    (attribute_key, value),
                    (kSecMatchSearchList, search_list),
                    (kSecUseAuthenticationUI, kSecUseAuthenticationUIFail),
                    (kSecReturnAttributes, kCFBooleanTrue),
                    (kSecMatchLimit, kSecMatchLimitAll),
                ],
            )?;
            let mut result: CfType = ptr::null();
            let status = SecItemCopyMatching(query.cast(), &mut result);
            if !result.is_null() {
                owned.hold(result);
            }
            if status != ERR_SEC_ITEM_NOT_FOUND || !result.is_null() {
                bail!("exact absent key attribute returned OSStatus {status} or a stale result")
            }
            Ok(status)
        }
    }

    pub(crate) fn sign_disposable_payload(
        &self,
        config: &ExactSignerConfig,
        payload: &[u8],
    ) -> Result<FixedP256SignatureV2> {
        config.validate()?;
        if payload.is_empty() || payload.len() > MAX_SIGNING_PAYLOAD_BYTES {
            bail!("fixed disposable signing payload is empty or exceeds its bound")
        }
        // SAFETY: only a derived public representation and normalized signature escape. The
        // private key is never exported or returned.
        unsafe {
            let mut owned = OwnedCf::new();
            let keychain = open_explicit_system_keychain(&mut owned)?;
            let (key, attributes) = exact_private_key_match(&mut owned, keychain, config)?
                .context("fixed disposable target signer is absent")?;
            let _identity = disposable_signer_identity(
                &mut owned,
                key,
                attributes,
                config,
                DisposableAclKind::Target,
            )?;
            let public_spki = copy_public_spki(&mut owned, key)?;
            let data = cf_data(&mut owned, payload)?;
            let mut error: CfType = ptr::null();
            let signature = SecKeyCreateSignature(
                key,
                kSecKeyAlgorithmECDSASignatureMessageX962SHA256,
                data,
                &mut error,
            );
            let error_code = take_cf_error_code(&mut owned, error);
            if signature.is_null() {
                bail!("sign fixed disposable payload failed with CFError code {error_code}")
            }
            owned.hold(signature);
            let der = copy_cf_data(signature, "P-256 DER signature")?;
            let parsed =
                P256Signature::from_der(&der).context("decode Security P-256 signature")?;
            let canonical = parsed.normalize_s().unwrap_or(parsed);
            Ok(FixedP256SignatureV2 {
                public_spki_der_base64url: URL_SAFE_NO_PAD.encode(public_spki),
                signature_p1363_low_s_base64url: URL_SAFE_NO_PAD.encode(canonical.to_bytes()),
            })
        }
    }

    pub(crate) fn delete_disposable_signer(
        &mut self,
        config: &ExactSignerConfig,
        acl_kind: DisposableAclKind,
        expected_identity_sha256: &str,
    ) -> Result<ExactDeleteReceipt> {
        require_lower_sha256(expected_identity_sha256, "conditioned disposable identity")?;
        let observed = self
            .read_disposable_signer(config, acl_kind)?
            .context("conditioned disposable signer is absent before deletion")?;
        if observed.identity_sha256 != expected_identity_sha256 {
            bail!("conditioned disposable signer identity changed before deletion")
        }
        exact_delete_receipt_impl(config)
    }

    /// Create the product-equivalent permanent P-256 key without supplying `kSecAttrAccess`.
    /// This is intentionally separate from [`Self::create_signer`], whose purpose is to test the
    /// explicit legacy ACL posture.
    pub fn create_product_equivalent_signer_without_access(
        &mut self,
        config: &ExactSignerConfig,
    ) -> Result<ProductEquivalentCreationReceipt> {
        create_product_equivalent_signer_without_access_impl(config)
    }

    /// Check only the exact persisted identity and key attributes, without assuming an explicit
    /// legacy ACL.  This is the observation used by the product-equivalent creator-route probe.
    pub fn exact_identity_present(&self, config: &ExactSignerConfig) -> Result<bool> {
        exact_identity_present_impl(config)
    }

    /// Invoke the one intended private-key capability.  The current executable must match the
    /// publisher trusted-application identity for this to complete without UI.
    pub fn sign(&self, config: &ExactSignerConfig, payload: &[u8]) -> Result<Vec<u8>> {
        config.validate()?;
        if payload.is_empty() || payload.len() > MAX_SIGNING_PAYLOAD_BYTES {
            bail!("signing payload is empty or exceeds its fixed bound")
        }
        // SAFETY: all objects and returned bytes remain live until copied.
        unsafe {
            let mut owned = OwnedCf::new();
            let keychain = open_explicit_system_keychain(&mut owned)?;
            let (key, _) = exact_private_key_match(&mut owned, keychain, config)?
                .context("exact experiment signer is absent")?;
            let data = cf_data(&mut owned, payload)?;
            let mut error: CfType = ptr::null();
            let signature = SecKeyCreateSignature(
                key,
                kSecKeyAlgorithmECDSASignatureMessageX962SHA256,
                data,
                &mut error,
            );
            let error_code = take_cf_error_code(&mut owned, error);
            if signature.is_null() {
                bail!("sign exact experiment payload failed with CFError code {error_code}")
            }
            owned.hold(signature);
            copy_cf_data(signature, "P-256 signature")
        }
    }

    /// Delete only the already-resolved exact key reference, then prove final absence.  The
    /// canonical explicit posture grants this to the exact publisher as rollback and to the exact
    /// finalizer as its sole positive private-key authorization.
    pub fn delete_cleanup(&mut self, config: &ExactSignerConfig) -> Result<bool> {
        let receipt = exact_delete_receipt_impl(config)?;
        match receipt.classification {
            ExactDeleteClassification::DeletedAndAbsent => Ok(true),
            ExactDeleteClassification::AlreadyAbsent => Ok(false),
            _ => bail!(
                "delete exact experiment signer failed with OSStatus {}",
                receipt.raw_os_status
            ),
        }
    }

    /// Return the raw OSStatus/classification while retaining exact-reference and final-state
    /// checks.  No key bytes are returned.
    pub fn exact_delete_receipt(
        &mut self,
        config: &ExactSignerConfig,
    ) -> Result<ExactDeleteReceipt> {
        exact_delete_receipt_impl(config)
    }

    /// Run a negative operation without returning any accidentally exported key bytes.
    pub fn run_negative_control(
        &mut self,
        config: &ExactSignerConfig,
        control: NegativeControl,
    ) -> Result<NegativeControlOutcome> {
        config.validate()?;
        // SAFETY: all copied objects stay in owned; unexpected export bytes are never exposed.
        unsafe {
            let mut owned = OwnedCf::new();
            let keychain = open_explicit_system_keychain(&mut owned)?;
            let (key, _) = exact_private_key_match(&mut owned, keychain, config)?
                .context("exact experiment signer is absent")?;
            match control {
                NegativeControl::ExportClear => {
                    let mut error: CfType = ptr::null();
                    let exported = SecKeyCopyExternalRepresentation(key, &mut error);
                    if !exported.is_null() {
                        owned.hold(exported);
                        if !error.is_null() {
                            owned.hold(error);
                        }
                        return Ok(NegativeControlOutcome::UnexpectedlySucceeded);
                    }
                    if error.is_null() {
                        bail!("export-clear negative control failed without a CFError")
                    }
                    let code = CFErrorGetCode(error) as i64;
                    owned.hold(error);
                    Ok(NegativeControlOutcome::Denied { code })
                }
                NegativeControl::ReplaceAccess => {
                    let (replacement, _) = build_exact_access(&mut owned, config)?;
                    let status = SecKeychainItemSetAccess(key.cast(), replacement);
                    if status == ERR_SEC_SUCCESS {
                        Ok(NegativeControlOutcome::UnexpectedlySucceeded)
                    } else {
                        Ok(NegativeControlOutcome::Denied {
                            code: i64::from(status),
                        })
                    }
                }
            }
        }
    }

    pub(crate) fn run_nobody_owner_authority_operation(
        &mut self,
        config: &ExactSignerConfig,
        operation: NobodyOwnerAuthorityOperationV2,
    ) -> Result<OsStatus> {
        config.validate()?;
        // SAFETY: every copied or created native object remains held through the one operation;
        // only a scalar OSStatus escapes and no private key bytes are copied.
        unsafe {
            let mut owned = OwnedCf::new();
            let keychain = open_explicit_system_keychain(&mut owned)?;
            let (key, attributes) = exact_private_key_match(&mut owned, keychain, config)?
                .context("nobody-owner exact disposable target is absent")?;
            let _identity = disposable_signer_identity(
                &mut owned,
                key,
                attributes,
                config,
                DisposableAclKind::Target,
            )?;
            match operation {
                NobodyOwnerAuthorityOperationV2::SetOwnerAndAclReplacement => {
                    let replacement =
                        build_nobody_forbidden_replacement_access(&mut owned, config, 0, 0)?;
                    Ok(SecKeychainItemSetAccess(key.cast(), replacement))
                }
                NobodyOwnerAuthorityOperationV2::SetAclReplacement => {
                    let replacement = build_nobody_forbidden_replacement_access(
                        &mut owned,
                        config,
                        NONMATCH_OWNER_UID,
                        NONMATCH_OWNER_GID,
                    )?;
                    Ok(SecKeychainItemSetAccess(key.cast(), replacement))
                }
                NobodyOwnerAuthorityOperationV2::DeleteExactTarget => {
                    Ok(exact_delete_receipt_impl(config)?.raw_os_status)
                }
                NobodyOwnerAuthorityOperationV2::SignExactTarget => {
                    let payload = cf_data(
                        &mut owned,
                        b"substrate.r3-macos-disposable-nobody-owner-sign-control.v2",
                    )?;
                    let mut error: CfType = ptr::null();
                    let signature = SecKeyCreateSignature(
                        key,
                        kSecKeyAlgorithmECDSASignatureMessageX962SHA256,
                        payload,
                        &mut error,
                    );
                    if !signature.is_null() {
                        owned.hold(signature);
                        if !error.is_null() {
                            owned.hold(error);
                        }
                        return Ok(ERR_SEC_SUCCESS);
                    }
                    if error.is_null() {
                        bail!("nobody-owner sign denial lacked its exact CFError")
                    }
                    let code = CFErrorGetCode(error);
                    owned.hold(error);
                    i32::try_from(code).context("nobody-owner sign CFError exceeds OSStatus")
                }
            }
        }
    }
}

impl QueryUiFailSecurity {
    /// Permanently reserve this process for query-level UI-fail operations.  This performs no
    /// Security framework call; the first native operation occurs in a method below.
    pub fn claim_process() -> Result<Self> {
        match SECURITY_UI_STATE.compare_exchange(0, 4, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => Ok(Self { _private: () }),
            Err(4) => bail!("query-level UI-fail Security mode is already claimed"),
            Err(1) => bail!("Security interaction denial is concurrently initializing"),
            Err(2) => bail!("process-level Security interaction denial is already active"),
            Err(3) => bail!("the first Security interaction-denial call previously failed"),
            Err(other) => bail!("invalid Security interaction state {other}"),
        }
    }

    /// Product-equivalent create: the key dictionary deliberately omits `kSecAttrAccess`.
    pub fn create_product_equivalent_signer_without_access(
        &mut self,
        config: &ExactSignerConfig,
    ) -> Result<ProductEquivalentCreationReceipt> {
        create_product_equivalent_signer_without_access_impl(config)
    }

    /// Check exact tag/label/P-256/private/permanent/capability attributes, not a custom ACL.
    pub fn exact_identity_present(&self, config: &ExactSignerConfig) -> Result<bool> {
        exact_identity_present_impl(config)
    }

    /// Attempt exact-reference deletion with query-level `kSecUseAuthenticationUIFail` and
    /// return only raw status/classification/final presence.
    pub fn exact_delete_receipt(
        &mut self,
        config: &ExactSignerConfig,
    ) -> Result<ExactDeleteReceipt> {
        exact_delete_receipt_impl(config)
    }
}

fn create_product_equivalent_signer_without_access_impl(
    config: &ExactSignerConfig,
) -> Result<ProductEquivalentCreationReceipt> {
    config.validate()?;
    // SAFETY: every created/copied CF object is held through its last use.
    unsafe {
        let mut owned = OwnedCf::new();
        let keychain = open_explicit_system_keychain(&mut owned)?;
        if exact_private_key_match(&mut owned, keychain, config)?.is_some() {
            bail!("exact product-equivalent experiment signer already exists")
        }
        let _created = create_exact_key(&mut owned, keychain, config, None)?;
        if exact_private_key_match(&mut owned, keychain, config)?.is_none() {
            bail!("product-equivalent signer did not reopen through its exact identity")
        }
        Ok(ProductEquivalentCreationReceipt {
            raw_cferror_code: 0,
            present_after: true,
        })
    }
}

fn exact_identity_present_impl(config: &ExactSignerConfig) -> Result<bool> {
    config.validate()?;
    // SAFETY: no raw reference escapes the owned arena.
    unsafe {
        let mut owned = OwnedCf::new();
        let keychain = open_explicit_system_keychain(&mut owned)?;
        Ok(exact_private_key_match(&mut owned, keychain, config)?.is_some())
    }
}

fn exact_delete_receipt_impl(config: &ExactSignerConfig) -> Result<ExactDeleteReceipt> {
    config.validate()?;
    // SAFETY: the exact resolved key, search list, and query remain held through deletion and the
    // final state observation.
    unsafe {
        let mut owned = OwnedCf::new();
        let keychain = open_explicit_system_keychain(&mut owned)?;
        let Some((key, _)) = exact_private_key_match(&mut owned, keychain, config)? else {
            return Ok(ExactDeleteReceipt {
                raw_os_status: ERR_SEC_ITEM_NOT_FOUND,
                classification: ExactDeleteClassification::AlreadyAbsent,
                present_after: false,
            });
        };
        let query = exact_delete_query(&mut owned, keychain, key, config.application_tag())?;
        let status = SecItemDelete(query.cast());
        let present_after = exact_private_key_match(&mut owned, keychain, config)?.is_some();
        let classification = classify_delete_status(status);
        match classification {
            ExactDeleteClassification::DeletedAndAbsent if present_after => {
                bail!("exact experiment signer remains after successful deletion")
            }
            ExactDeleteClassification::DeletedAndAbsent => {}
            _ if !present_after => {
                bail!("exact experiment signer disappeared after failed deletion OSStatus {status}")
            }
            _ => {}
        }
        Ok(ExactDeleteReceipt {
            raw_os_status: status,
            classification,
            present_after,
        })
    }
}

fn classify_delete_status(status: OsStatus) -> ExactDeleteClassification {
    match status {
        ERR_SEC_SUCCESS => ExactDeleteClassification::DeletedAndAbsent,
        ERR_SEC_ITEM_NOT_FOUND => ExactDeleteClassification::AlreadyAbsent,
        ERR_SEC_INTERACTION_NOT_ALLOWED => ExactDeleteClassification::InteractionNotAllowed,
        ERR_SEC_AUTH_FAILED => ExactDeleteClassification::AuthorizationDenied,
        _ => ExactDeleteClassification::OtherFailure,
    }
}

fn signer_readback(config: &ExactSignerConfig, access: CanonicalAccessDigest) -> SignerReadback {
    SignerReadback {
        application_tag_base64url: URL_SAFE_NO_PAD.encode(config.application_tag()),
        label: config.label().to_owned(),
        access,
    }
}

#[cfg(test)]
pub(crate) fn expected_snapshot_for_test(
    publisher_data: &[u8],
    finalizer_data: &[u8],
) -> AccessSnapshot {
    let mut snapshot = expected_snapshot(publisher_data, finalizer_data);
    canonicalize_snapshot(&mut snapshot);
    snapshot
}

#[cfg(test)]
pub(crate) fn canonical_digest_for_test(
    snapshot: &mut AccessSnapshot,
) -> Result<CanonicalAccessDigest> {
    canonical_digest(snapshot.clone())
}

#[cfg(test)]
fn expected_snapshot(publisher_data: &[u8], finalizer_data: &[u8]) -> AccessSnapshot {
    expected_disposable_snapshot(publisher_data, finalizer_data, DisposableAclKind::Target)
}

fn expected_disposable_snapshot(
    publisher_data: &[u8],
    finalizer_data: &[u8],
    acl_kind: DisposableAclKind,
) -> AccessSnapshot {
    let mut entries = vec![
        AclEntrySnapshot {
            description: PUBLISHER_SIGN_DELETE_DESCRIPTION.to_owned(),
            prompt_selector: PROMPT_SELECTOR_NONE,
            authorizations: vec![AUTH_SIGN.to_owned(), AUTH_DELETE.to_owned()],
            trusted_applications: vec![trusted_digest(publisher_data)],
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
    ];
    if acl_kind == DisposableAclKind::Target {
        entries.push(AclEntrySnapshot {
            description: FINALIZER_DELETE_DESCRIPTION.to_owned(),
            prompt_selector: PROMPT_SELECTOR_NONE,
            authorizations: vec![AUTH_DELETE.to_owned()],
            trusted_applications: vec![trusted_digest(finalizer_data)],
        });
    }
    AccessSnapshot {
        owner_uid: NONMATCH_OWNER_UID,
        owner_gid: NONMATCH_OWNER_GID,
        owner_type: OWNER_TYPE_USE_ONLY_UID_AND_GID,
        entries,
    }
}

#[cfg(test)]
pub(crate) fn expected_disposable_snapshot_for_test(
    publisher_data: &[u8],
    finalizer_data: &[u8],
    acl_kind: DisposableAclKind,
) -> AccessSnapshot {
    let mut snapshot = expected_disposable_snapshot(publisher_data, finalizer_data, acl_kind);
    canonicalize_snapshot(&mut snapshot);
    snapshot
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

fn canonical_digest(mut snapshot: AccessSnapshot) -> Result<CanonicalAccessDigest> {
    canonicalize_snapshot(&mut snapshot);
    let canonical_bytes = canonical_bytes_v2(&snapshot).context("serialize canonical SecAccess")?;
    let canonical_json = String::from_utf8(canonical_bytes.clone())
        .context("canonical SecAccess is not UTF-8 JSON")?;
    let sha256 = sha256_hex_v2(&canonical_bytes);
    Ok(CanonicalAccessDigest {
        snapshot,
        canonical_json,
        sha256,
    })
}

pub(crate) fn probe_nonmatch_owner_access_in_memory_impl() -> Result<CanonicalAccessDigest> {
    // This creates only an in-memory SecAccess object. It does not open a Keychain, create an item,
    // or invoke any authorization operation. Exact copy-owner readback must succeed before a
    // persistent signer constructor is allowed to run.
    unsafe {
        let mut owned = OwnedCf::new();
        let mut error: CfType = ptr::null();
        let access = SecAccessCreateWithOwnerAndACL(
            NONMATCH_OWNER_UID,
            NONMATCH_OWNER_GID,
            OWNER_TYPE_USE_ONLY_UID_AND_GID,
            ptr::null(),
            &mut error,
        );
        let error_code = take_cf_error_code(&mut owned, error);
        if access.is_null() {
            bail!("in-memory all-ones SecAccess owner probe failed with CFError code {error_code}")
        }
        owned.hold(access);
        let snapshot = snapshot_access(&mut owned, access)?;
        if snapshot.owner_uid != NONMATCH_OWNER_UID
            || snapshot.owner_gid != NONMATCH_OWNER_GID
            || snapshot.owner_type != OWNER_TYPE_USE_ONLY_UID_AND_GID
            || !snapshot.entries.is_empty()
        {
            bail!("in-memory SecAccess owner copy-readback changed the all-ones conjunctive owner")
        }
        canonical_digest(snapshot)
    }
}

fn validate_disposable_posture(
    snapshot: AccessSnapshot,
    publisher_data: &[u8],
    finalizer_data: &[u8],
    acl_kind: DisposableAclKind,
) -> Result<CanonicalAccessDigest> {
    let digest = canonical_digest(snapshot)?;
    let mut expected = expected_disposable_snapshot(publisher_data, finalizer_data, acl_kind);
    canonicalize_snapshot(&mut expected);
    if digest.snapshot != expected {
        bail!("SecAccess posture differs from the exact canonical ACL")
    }
    if digest
        .snapshot
        .entries
        .iter()
        .flat_map(|entry| &entry.authorizations)
        .any(|authorization| authorization == AUTH_ANY)
    {
        bail!("SecAccess posture contains forbidden Any authorization")
    }
    Ok(digest)
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
        // SAFETY: every entry follows a Security/CoreFoundation create or copy rule exactly once.
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

unsafe fn cf_number_i64(owned: &mut OwnedCf, value: i64) -> Result<CfType> {
    // SAFETY: value is live and correctly typed for kCFNumberSInt64Type.
    let result = unsafe {
        CFNumberCreate(
            kCFAllocatorDefault,
            K_CF_NUMBER_SINT64,
            (&value as *const i64).cast(),
        )
    };
    if result.is_null() {
        bail!("allocate CoreFoundation number")
    }
    Ok(owned.hold(result))
}

unsafe fn cf_array(owned: &mut OwnedCf, values: &[CfType]) -> Result<CfType> {
    // SAFETY: pointer is valid for count and every value remains held elsewhere in owned or static
    // framework storage while the array is used.
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
    // SAFETY: null callbacks match the existing product FFI pattern; all values remain held.
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
        // SAFETY: result is mutable and both objects remain live.
        unsafe { CFDictionarySetValue(result, *key, *value) };
    }
    Ok(result)
}

unsafe fn open_explicit_system_keychain(owned: &mut OwnedCf) -> Result<SecKeychain> {
    let path = CString::new(SYSTEM_KEYCHAIN_PATH).context("encode System Keychain path")?;
    let mut keychain: SecKeychain = ptr::null();
    // SAFETY: path and output pointer are valid.
    let status = unsafe { SecKeychainOpen(path.as_ptr(), &mut keychain) };
    if status != ERR_SEC_SUCCESS || keychain.is_null() {
        bail!("open explicit System Keychain failed with OSStatus {status}")
    }
    owned.hold(keychain);
    let mut returned = [0 as c_char; MAX_SYSTEM_KEYCHAIN_PATH_BYTES];
    let mut length = returned.len() as u32;
    // SAFETY: returned is writable for length and keychain is live.
    let status = unsafe { SecKeychainGetPath(keychain, &mut length, returned.as_mut_ptr()) };
    if status != ERR_SEC_SUCCESS {
        bail!("read explicit System Keychain path failed with OSStatus {status}")
    }
    let length = length as usize;
    if length >= returned.len() {
        bail!("explicit System Keychain returned an unterminated path")
    }
    // SAFETY: Security initialized the returned prefix.
    let bytes: &[u8] = unsafe { std::slice::from_raw_parts(returned.as_ptr().cast(), length) };
    if bytes != SYSTEM_KEYCHAIN_PATH.as_bytes() || returned[length] != 0 {
        bail!("opened keychain does not match the exact System Keychain path")
    }
    Ok(keychain)
}

unsafe fn trusted_application(owned: &mut OwnedCf, path: &str) -> Result<SecTrustedApplication> {
    let path = CString::new(path).context("encode trusted-application path")?;
    let mut application: SecTrustedApplication = ptr::null();
    // SAFETY: path and output pointer are valid.
    let status = unsafe { SecTrustedApplicationCreateFromPath(path.as_ptr(), &mut application) };
    if status != ERR_SEC_SUCCESS || application.is_null() {
        bail!("create exact trusted application failed with OSStatus {status}")
    }
    Ok(owned.hold(application))
}

unsafe fn trusted_application_data(
    owned: &mut OwnedCf,
    application: SecTrustedApplication,
) -> Result<Vec<u8>> {
    let mut data: CfType = ptr::null();
    // SAFETY: application is live and data is a writable copy-rule out pointer.
    let status = unsafe { SecTrustedApplicationCopyData(application, &mut data) };
    if status != ERR_SEC_SUCCESS || data.is_null() {
        bail!("copy trusted-application identity data failed with OSStatus {status}")
    }
    owned.hold(data);
    // SAFETY: SecTrustedApplicationCopyData returns CFData.
    unsafe { copy_cf_data(data, "trusted-application identity") }
}

unsafe fn expected_trusted_data(
    owned: &mut OwnedCf,
    config: &ExactSignerConfig,
) -> Result<(Vec<u8>, Vec<u8>)> {
    // SAFETY: returned applications remain held by owned.
    let publisher = unsafe { trusted_application(owned, config.publisher_path())? };
    // SAFETY: same ownership.
    let finalizer = unsafe { trusted_application(owned, config.finalizer_path())? };
    Ok((
        // SAFETY: application is live.
        unsafe { trusted_application_data(owned, publisher)? },
        // SAFETY: application is live.
        unsafe { trusted_application_data(owned, finalizer)? },
    ))
}

unsafe fn create_exact_key(
    owned: &mut OwnedCf,
    keychain: SecKeychain,
    config: &ExactSignerConfig,
    access: Option<SecAccess>,
) -> Result<SecKey> {
    // SAFETY: created values remain held through key creation.
    let tag = unsafe { cf_data(owned, config.application_tag())? };
    // SAFETY: same ownership.
    let label = unsafe { cf_string(owned, config.label())? };
    // SAFETY: same ownership.
    let bits = unsafe { cf_number_i64(owned, 256)? };
    let mut private_entries = vec![
        (unsafe { kSecAttrIsPermanent }, unsafe { kCFBooleanTrue }),
        (unsafe { kSecAttrApplicationTag }, tag),
        (unsafe { kSecAttrLabel }, label),
        (unsafe { kSecAttrIsSensitive }, unsafe { kCFBooleanTrue }),
        (unsafe { kSecAttrIsExtractable }, unsafe { kCFBooleanFalse }),
        (unsafe { kSecAttrCanEncrypt }, unsafe { kCFBooleanFalse }),
        (unsafe { kSecAttrCanDecrypt }, unsafe { kCFBooleanFalse }),
        (unsafe { kSecAttrCanDerive }, unsafe { kCFBooleanFalse }),
        (unsafe { kSecAttrCanSign }, unsafe { kCFBooleanTrue }),
        (unsafe { kSecAttrCanVerify }, unsafe { kCFBooleanFalse }),
        (unsafe { kSecAttrCanWrap }, unsafe { kCFBooleanFalse }),
        (unsafe { kSecAttrCanUnwrap }, unsafe { kCFBooleanFalse }),
    ];
    if let Some(access) = access {
        private_entries.push((unsafe { kSecAttrAccess }, access.cast()));
    }
    // SAFETY: every dictionary value remains live.
    let private_attributes = unsafe { dictionary(owned, &private_entries)? };
    // SAFETY: every dictionary value remains live.
    let parameters = unsafe {
        dictionary(
            owned,
            &[
                (kSecAttrKeyType, kSecAttrKeyTypeECSECPrimeRandom),
                (kSecAttrKeySizeInBits, bits),
                (kSecPrivateKeyAttrs, private_attributes.cast()),
                (kSecUseKeychain, keychain.cast()),
                (kSecUseAuthenticationUI, kSecUseAuthenticationUIFail),
            ],
        )?
    };
    let mut error: CfType = ptr::null();
    // SAFETY: parameters and output remain live.
    let created = unsafe { SecKeyCreateRandomKey(parameters.cast(), &mut error) };
    // SAFETY: error follows the CFError create rule when non-null.
    let error_code = unsafe { take_cf_error_code(owned, error) };
    if created.is_null() {
        bail!("create exact P-256 signer failed with CFError code {error_code}")
    }
    owned.hold(created);
    // SAFETY: created remains held.
    unsafe { validate_key_capabilities(owned, created)? };
    Ok(created)
}

unsafe fn build_exact_access(
    owned: &mut OwnedCf,
    config: &ExactSignerConfig,
) -> Result<(SecAccess, CanonicalAccessDigest)> {
    // SAFETY: forwarded ownership and fixed target posture.
    unsafe { build_disposable_access(owned, config, DisposableAclKind::Target) }
}

unsafe fn build_disposable_access(
    owned: &mut OwnedCf,
    config: &ExactSignerConfig,
    acl_kind: DisposableAclKind,
) -> Result<(SecAccess, CanonicalAccessDigest)> {
    let mut error: CfType = ptr::null();
    // The public SDK declares `acls` nullable.  Starting with no ACLs avoids SecAccessCreate's
    // release-dependent defaults and breaks the apparent SecAccess/SecACL construction cycle.
    // SAFETY: scalar owner values are exact and error is a writable CFError out pointer.
    let access = unsafe {
        SecAccessCreateWithOwnerAndACL(
            NONMATCH_OWNER_UID,
            NONMATCH_OWNER_GID,
            OWNER_TYPE_USE_ONLY_UID_AND_GID,
            ptr::null(),
            &mut error,
        )
    };
    let error_code = unsafe { take_cf_error_code(owned, error) };
    if access.is_null() {
        bail!("create empty exact SecAccess failed with CFError code {error_code}")
    }
    owned.hold(access);

    // SAFETY: applications and arrays remain held while ACL creation copies their contents.
    let publisher = unsafe { trusted_application(owned, config.publisher_path())? };
    // SAFETY: same ownership.
    let finalizer = unsafe { trusted_application(owned, config.finalizer_path())? };
    // SAFETY: referenced application remains live.
    let publisher_list = unsafe { cf_array(owned, &[publisher.cast()])? };
    // SAFETY: referenced application remains live.
    let finalizer_list = unsafe { cf_array(owned, &[finalizer.cast()])? };
    // Explicit empty array means no trusted applications; null is never used for a deny entry.
    // SAFETY: empty slice is valid for a zero-element array.
    let empty_list = unsafe { cf_array(owned, &[])? };

    // SAFETY: access and all list objects remain live.
    unsafe {
        create_acl(
            owned,
            access,
            publisher_list,
            PUBLISHER_SIGN_DELETE_DESCRIPTION,
            &[kSecACLAuthorizationSign, kSecACLAuthorizationDelete],
        )?;
        if acl_kind == DisposableAclKind::Target {
            create_acl(
                owned,
                access,
                finalizer_list,
                FINALIZER_DELETE_DESCRIPTION,
                &[kSecACLAuthorizationDelete],
            )?;
        }
        create_acl(
            owned,
            access,
            empty_list,
            PRIVATE_OPERATION_DENY_DESCRIPTION,
            &[
                kSecACLAuthorizationExportWrapped,
                kSecACLAuthorizationExportClear,
                kSecACLAuthorizationImportWrapped,
                kSecACLAuthorizationImportClear,
                kSecACLAuthorizationEncrypt,
                kSecACLAuthorizationDecrypt,
                kSecACLAuthorizationMAC,
                kSecACLAuthorizationDerive,
            ],
        )?;
        create_acl(
            owned,
            access,
            empty_list,
            OWNER_MUTATION_DENY_DESCRIPTION,
            &[
                kSecACLAuthorizationChangeACL,
                kSecACLAuthorizationChangeOwner,
            ],
        )?;
    }

    // SAFETY: applications are live and copy returns retained data.
    let publisher_data = unsafe { trusted_application_data(owned, publisher)? };
    // SAFETY: same ownership.
    let finalizer_data = unsafe { trusted_application_data(owned, finalizer)? };
    // SAFETY: access remains live.
    let snapshot = unsafe { snapshot_access(owned, access)? };
    let digest = validate_disposable_posture(snapshot, &publisher_data, &finalizer_data, acl_kind)?;
    Ok((access, digest))
}

unsafe fn build_nobody_forbidden_replacement_access(
    owned: &mut OwnedCf,
    config: &ExactSignerConfig,
    owner_uid: u32,
    owner_gid: u32,
) -> Result<SecAccess> {
    let mut error: CfType = ptr::null();
    // The replacement deliberately preserves publisher Sign+Delete rollback while changing the
    // exact finalizer authorization from Delete to Sign.  Either selected owner changes too, or
    // remains the all-ones conjunctive owner for the ACL-only arm.
    let access = unsafe {
        SecAccessCreateWithOwnerAndACL(
            owner_uid,
            owner_gid,
            OWNER_TYPE_USE_ONLY_UID_AND_GID,
            ptr::null(),
            &mut error,
        )
    };
    let error_code = unsafe { take_cf_error_code(owned, error) };
    if access.is_null() {
        bail!("create nobody-owner forbidden replacement failed with CFError {error_code}")
    }
    owned.hold(access);
    let publisher = unsafe { trusted_application(owned, config.publisher_path())? };
    let finalizer = unsafe { trusted_application(owned, config.finalizer_path())? };
    let publisher_list = unsafe { cf_array(owned, &[publisher.cast()])? };
    let finalizer_list = unsafe { cf_array(owned, &[finalizer.cast()])? };
    let empty_list = unsafe { cf_array(owned, &[])? };
    unsafe {
        create_acl(
            owned,
            access,
            publisher_list,
            PUBLISHER_SIGN_DELETE_DESCRIPTION,
            &[kSecACLAuthorizationSign, kSecACLAuthorizationDelete],
        )?;
        create_acl(
            owned,
            access,
            finalizer_list,
            "unauthorized-finalizer-sign-with-publisher-delete-rollback",
            &[kSecACLAuthorizationSign],
        )?;
        create_acl(
            owned,
            access,
            empty_list,
            PRIVATE_OPERATION_DENY_DESCRIPTION,
            &[
                kSecACLAuthorizationExportWrapped,
                kSecACLAuthorizationExportClear,
                kSecACLAuthorizationImportWrapped,
                kSecACLAuthorizationImportClear,
                kSecACLAuthorizationEncrypt,
                kSecACLAuthorizationDecrypt,
                kSecACLAuthorizationMAC,
                kSecACLAuthorizationDerive,
            ],
        )?;
        create_acl(
            owned,
            access,
            empty_list,
            OWNER_MUTATION_DENY_DESCRIPTION,
            &[
                kSecACLAuthorizationChangeACL,
                kSecACLAuthorizationChangeOwner,
            ],
        )?;
    }
    Ok(access)
}

unsafe fn create_acl(
    owned: &mut OwnedCf,
    access: SecAccess,
    application_list: CfType,
    description: &str,
    authorizations: &[CfType],
) -> Result<()> {
    // SAFETY: created string remains held.
    let description = unsafe { cf_string(owned, description)? };
    let mut acl: SecAcl = ptr::null();
    // SAFETY: all inputs are live and acl is a writable create-rule out pointer.
    let status = unsafe {
        SecACLCreateWithSimpleContents(
            access,
            application_list,
            description,
            PROMPT_SELECTOR_NONE,
            &mut acl,
        )
    };
    if status != ERR_SEC_SUCCESS || acl.is_null() {
        bail!("create exact SecACL entry failed with OSStatus {status}")
    }
    owned.hold(acl);
    // SAFETY: authorization statics remain live.
    let authorizations = unsafe { cf_array(owned, authorizations)? };
    // SAFETY: both ACL and authorization array are live.
    let status = unsafe { SecACLUpdateAuthorizations(acl, authorizations) };
    if status != ERR_SEC_SUCCESS {
        bail!("set exact SecACL authorizations failed with OSStatus {status}")
    }
    Ok(())
}

unsafe fn snapshot_access(owned: &mut OwnedCf, access: SecAccess) -> Result<AccessSnapshot> {
    let mut owner_uid = 0_u32;
    let mut owner_gid = 0_u32;
    let mut owner_type = 0_u32;
    let mut acl_list: CfType = ptr::null();
    // SAFETY: access is live and every output is writable.
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
    // SAFETY: returned object is documented CFArray.
    if unsafe { CFGetTypeID(acl_list) } != unsafe { CFArrayGetTypeID() } {
        bail!("SecAccess ACL list is not an array")
    }
    // SAFETY: validated array.
    let count = unsafe { CFArrayGetCount(acl_list) };
    if count < 0 {
        bail!("SecAccess ACL list has invalid length")
    }
    let mut entries = Vec::with_capacity(count as usize);
    for index in 0..count {
        // SAFETY: index is within the array count.
        let acl = unsafe { CFArrayGetValueAtIndex(acl_list, index) };
        // SAFETY: non-null array element is live.
        if acl.is_null() || unsafe { CFGetTypeID(acl) } != unsafe { SecACLGetTypeID() } {
            bail!("SecAccess ACL list contains a non-SecACL value")
        }
        // SAFETY: ACL is live with acl_list.
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
    // SAFETY: description is documented CFString.
    let description = unsafe { copy_cf_string(description)? };

    // SAFETY: copy-rule array returned for the live ACL.
    let authorizations = unsafe { SecACLCopyAuthorizations(acl) };
    if authorizations.is_null() {
        bail!("copy SecACL authorizations returned null")
    }
    owned.hold(authorizations);
    // SAFETY: both returned containers are documented arrays.
    if unsafe { CFGetTypeID(applications) } != unsafe { CFArrayGetTypeID() }
        || unsafe { CFGetTypeID(authorizations) } != unsafe { CFArrayGetTypeID() }
    {
        bail!("SecACL contents contain a non-array value")
    }

    // SAFETY: validated array.
    let authorization_count = unsafe { CFArrayGetCount(authorizations) };
    let mut authorization_names = Vec::with_capacity(authorization_count as usize);
    for index in 0..authorization_count {
        // SAFETY: index is in bounds.
        let authorization = unsafe { CFArrayGetValueAtIndex(authorizations, index) };
        // SAFETY: framework authorization values are live CFStrings.
        authorization_names.push(unsafe { authorization_name(authorization)? }.to_owned());
    }

    // SAFETY: validated array.
    let application_count = unsafe { CFArrayGetCount(applications) };
    let mut trusted_applications = Vec::with_capacity(application_count as usize);
    for index in 0..application_count {
        // SAFETY: index is in bounds.
        let application = unsafe { CFArrayGetValueAtIndex(applications, index) };
        // SAFETY: non-null element is live.
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
    // SAFETY: these are process-lifetime Security framework constants.
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
        // SAFETY: both are live CF objects.
        if unsafe { CFEqual(value, expected) } != 0 {
            return Ok(name);
        }
    }
    bail!("SecACL contains an unknown authorization tag")
}

unsafe fn copy_and_validate_access(
    owned: &mut OwnedCf,
    key: SecKey,
    config: &ExactSignerConfig,
) -> Result<CanonicalAccessDigest> {
    // SAFETY: forwarded live objects and exact target posture.
    unsafe { copy_and_validate_disposable_access(owned, key, config, DisposableAclKind::Target) }
}

unsafe fn copy_and_validate_disposable_access(
    owned: &mut OwnedCf,
    key: SecKey,
    config: &ExactSignerConfig,
    acl_kind: DisposableAclKind,
) -> Result<CanonicalAccessDigest> {
    let mut access: SecAccess = ptr::null();
    // SAFETY: key is a live legacy keychain item and access is a writable copy-rule pointer.
    let status = unsafe { SecKeychainItemCopyAccess(key.cast(), &mut access) };
    if status != ERR_SEC_SUCCESS || access.is_null() {
        bail!("copy persisted key SecAccess failed with OSStatus {status}")
    }
    owned.hold(access);
    // SAFETY: creates exact trusted applications and copies their identity bytes.
    let (publisher_data, finalizer_data) = unsafe { expected_trusted_data(owned, config)? };
    // SAFETY: access is live.
    let snapshot = unsafe { snapshot_access(owned, access)? };
    validate_disposable_posture(snapshot, &publisher_data, &finalizer_data, acl_kind)
}

unsafe fn exact_private_key_query(
    owned: &mut OwnedCf,
    keychain: SecKeychain,
    config: &ExactSignerConfig,
) -> Result<CfMutableDictionary> {
    // SAFETY: created data/array remain held.
    let tag = unsafe { cf_data(owned, config.application_tag())? };
    // SAFETY: keychain remains held.
    let search_list = unsafe { cf_array(owned, &[keychain.cast()])? };
    // SAFETY: all values remain live.
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
    config: &ExactSignerConfig,
) -> Result<Option<(SecKey, CfType)>> {
    // SAFETY: query remains held.
    let query = unsafe { exact_private_key_query(owned, keychain, config)? };
    let mut result: CfType = ptr::null();
    // SAFETY: query and output pointer are valid.
    let status = unsafe { SecItemCopyMatching(query.cast(), &mut result) };
    if status == ERR_SEC_ITEM_NOT_FOUND {
        return Ok(None);
    }
    if status != ERR_SEC_SUCCESS || result.is_null() {
        bail!("lookup exact experiment signer failed with OSStatus {status}")
    }
    owned.hold(result);
    // SAFETY: live CF object.
    if unsafe { CFGetTypeID(result) } != unsafe { CFArrayGetTypeID() } {
        bail!("exact signer lookup returned a non-array result")
    }
    // SAFETY: validated array.
    let count = unsafe { CFArrayGetCount(result) };
    if count != 1 {
        bail!("exact signer identity is ambiguous ({count} matches)")
    }
    // SAFETY: array has exactly one element.
    let attributes = unsafe { CFArrayGetValueAtIndex(result, 0) };
    // SAFETY: element is live.
    if attributes.is_null()
        || unsafe { CFGetTypeID(attributes) } != unsafe { CFDictionaryGetTypeID() }
    {
        bail!("exact signer lookup returned invalid attributes")
    }
    // SAFETY: validated dictionary and static key.
    let key: SecKey = unsafe { CFDictionaryGetValue(attributes, kSecValueRef) }.cast();
    if key.is_null() {
        bail!("exact signer lookup returned a null key")
    }
    // SAFETY: attributes and key remain live with result.
    unsafe { validate_persisted_identity(owned, attributes, key, config)? };
    Ok(Some((key, attributes)))
}

unsafe fn validate_persisted_identity(
    owned: &mut OwnedCf,
    attributes: CfType,
    key: SecKey,
    config: &ExactSignerConfig,
) -> Result<()> {
    // SAFETY: created comparison values remain held.
    let tag = unsafe { cf_data(owned, config.application_tag())? };
    // SAFETY: created comparison value remains held.
    let label = unsafe { cf_string(owned, config.label())? };
    // SAFETY: dictionary and expected values are live.
    unsafe {
        require_attribute(attributes, kSecAttrApplicationTag, tag, "application tag")?;
        require_attribute(attributes, kSecAttrLabel, label, "label")?;
        validate_key_capabilities(owned, key)?;
    }
    Ok(())
}

unsafe fn disposable_signer_identity(
    owned: &mut OwnedCf,
    key: SecKey,
    attributes: CfType,
    config: &ExactSignerConfig,
    acl_kind: DisposableAclKind,
) -> Result<DisposableSignerIdentityV2> {
    // SAFETY: all values are live and this validates the complete key/ACL posture first.
    unsafe { validate_persisted_identity(owned, attributes, key, config)? };
    // SAFETY: key remains live and only a canonical digest escapes.
    let access = unsafe { copy_and_validate_disposable_access(owned, key, config, acl_kind)? };
    // SAFETY: attributes is a validated result dictionary and framework keys are live statics.
    let application_label = unsafe { CFDictionaryGetValue(attributes, kSecAttrApplicationLabel) };
    // SAFETY: same dictionary and static key.
    let persistent_reference = unsafe { CFDictionaryGetValue(attributes, kSecValuePersistentRef) };
    if application_label.is_null() || persistent_reference.is_null() {
        bail!("exact disposable signer omitted application label or persistent reference")
    }
    // SAFETY: copy_cf_data validates both dynamic values.
    let application_label = unsafe { copy_cf_data(application_label, "application label")? };
    // SAFETY: same.
    let persistent_reference =
        unsafe { copy_cf_data(persistent_reference, "persistent reference")? };
    if application_label.is_empty() || persistent_reference.is_empty() {
        bail!("exact disposable signer has an empty stable identity component")
    }
    // SAFETY: derives and copies only the public key representation.
    let public_spki = unsafe { copy_public_spki(owned, key)? };
    let application_tag_sha256 = sha256_hex_v2(config.application_tag());
    let application_label_sha256 = sha256_hex_v2(&application_label);
    let application_label_base64url = URL_SAFE_NO_PAD.encode(&application_label);
    let persistent_reference_sha256 = sha256_hex_v2(&persistent_reference);
    let public_spki_sha256 = sha256_hex_v2(&public_spki);
    let identity_sha256 = sha256_hex_v2(&canonical_bytes_v2(&KeyIdentityMaterial {
        application_tag_sha256: &application_tag_sha256,
        label: config.label(),
        application_label_base64url: &application_label_base64url,
        persistent_reference_sha256: &persistent_reference_sha256,
        public_spki_sha256: &public_spki_sha256,
        access_control_sha256: &access.sha256,
    })?);
    Ok(DisposableSignerIdentityV2 {
        application_tag_sha256,
        label: config.label().to_owned(),
        application_label: application_label_base64url,
        application_label_sha256,
        persistent_reference_sha256,
        spki_der: URL_SAFE_NO_PAD.encode(public_spki),
        spki_der_sha256: public_spki_sha256,
        access_control_sha256: access.sha256,
        identity_sha256,
    })
}

unsafe fn copy_public_spki(owned: &mut OwnedCf, private_key: SecKey) -> Result<Vec<u8>> {
    // SAFETY: private key remains live; returned key follows the copy rule.
    let public_key = unsafe { SecKeyCopyPublicKey(private_key) };
    if public_key.is_null() {
        bail!("exact disposable signer has no copyable public key")
    }
    owned.hold(public_key);
    let mut error: CfType = ptr::null();
    // SAFETY: this exports only the derived public key object.
    let external = unsafe { SecKeyCopyExternalRepresentation(public_key, &mut error) };
    let error_code = unsafe { take_cf_error_code(owned, error) };
    if external.is_null() {
        bail!("copy P-256 public key failed with CFError code {error_code}")
    }
    owned.hold(external);
    // SAFETY: documented public CFData result.
    let raw = unsafe { copy_cf_data(external, "P-256 public representation")? };
    p256_spki_der(&raw)
}

fn p256_spki_der(raw: &[u8]) -> Result<Vec<u8>> {
    if raw.len() != 65 || raw[0] != 0x04 {
        bail!("exact signer public representation is not uncompressed P-256")
    }
    const PREFIX: [u8; 26] = [
        0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, 0x06, 0x08,
        0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, 0x03, 0x42, 0x00,
    ];
    let mut spki = Vec::with_capacity(PREFIX.len() + raw.len());
    spki.extend_from_slice(&PREFIX);
    spki.extend_from_slice(raw);
    Ok(spki)
}

fn require_lower_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is not one lowercase SHA-256")
    }
    Ok(())
}

unsafe fn validate_key_capabilities(owned: &mut OwnedCf, key: SecKey) -> Result<()> {
    // SAFETY: key is live and returned dictionary follows the copy rule.
    let attributes = unsafe { SecKeyCopyAttributes(key) };
    if attributes.is_null() {
        bail!("exact signer has no inspectable key attributes")
    }
    owned.hold(attributes);
    // SAFETY: attributes and framework statics are live.
    unsafe {
        require_attribute(
            attributes,
            kSecAttrKeyClass,
            kSecAttrKeyClassPrivate,
            "private key class",
        )?;
        require_attribute(
            attributes,
            kSecAttrKeyType,
            kSecAttrKeyTypeECSECPrimeRandom,
            "P-256 key type",
        )?;
        require_attribute(
            attributes,
            kSecAttrIsPermanent,
            kCFBooleanTrue,
            "permanent state",
        )?;
        require_attribute(
            attributes,
            kSecAttrIsSensitive,
            kCFBooleanTrue,
            "sensitive state",
        )?;
        require_attribute(
            attributes,
            kSecAttrIsExtractable,
            kCFBooleanFalse,
            "nonextractable state",
        )?;
        require_attribute(
            attributes,
            kSecAttrCanEncrypt,
            kCFBooleanFalse,
            "encrypt capability",
        )?;
        require_attribute(
            attributes,
            kSecAttrCanDecrypt,
            kCFBooleanFalse,
            "decrypt capability",
        )?;
        require_attribute(
            attributes,
            kSecAttrCanDerive,
            kCFBooleanFalse,
            "derive capability",
        )?;
        require_attribute(
            attributes,
            kSecAttrCanSign,
            kCFBooleanTrue,
            "sign capability",
        )?;
        require_attribute(
            attributes,
            kSecAttrCanVerify,
            kCFBooleanFalse,
            "verify capability",
        )?;
        require_attribute(
            attributes,
            kSecAttrCanWrap,
            kCFBooleanFalse,
            "wrap capability",
        )?;
        require_attribute(
            attributes,
            kSecAttrCanUnwrap,
            kCFBooleanFalse,
            "unwrap capability",
        )?;
    }
    // SAFETY: validated dictionary.
    let key_size = unsafe { CFDictionaryGetValue(attributes, kSecAttrKeySizeInBits) };
    let mut bits = 0_i64;
    // SAFETY: key_size is checked non-null and bits is writable.
    if key_size.is_null()
        || unsafe { CFNumberGetValue(key_size, K_CF_NUMBER_SINT64, (&mut bits as *mut i64).cast()) }
            == 0
        || bits != 256
    {
        bail!("exact signer is not exactly P-256")
    }
    Ok(())
}

unsafe fn require_attribute(
    attributes: CfType,
    key: CfType,
    expected: CfType,
    label: &str,
) -> Result<()> {
    // SAFETY: attributes is a live dictionary.
    let observed = unsafe { CFDictionaryGetValue(attributes, key) };
    // SAFETY: non-null values are live CF objects.
    if observed.is_null() || unsafe { CFEqual(observed, expected) } == 0 {
        bail!("exact signer has mismatched {label}")
    }
    Ok(())
}

unsafe fn exact_delete_query(
    owned: &mut OwnedCf,
    keychain: SecKeychain,
    key: SecKey,
    application_tag: &[u8],
) -> Result<CfMutableDictionary> {
    // SAFETY: created objects remain held.
    let tag = unsafe { cf_data(owned, application_tag)? };
    // SAFETY: keychain/key remain live.
    let search_list = unsafe { cf_array(owned, &[keychain.cast()])? };
    // SAFETY: exact key remains live through deletion.
    let item_list = unsafe { cf_array(owned, &[key.cast()])? };
    // SAFETY: all values remain live.
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

unsafe fn copy_cf_string(value: CfType) -> Result<String> {
    // SAFETY: value is a live CF object.
    if unsafe { CFGetTypeID(value) } != unsafe { CFStringGetTypeID() } {
        bail!("SecACL description is not CFString")
    }
    // SAFETY: validated CFString.
    let characters = unsafe { CFStringGetLength(value) };
    // SAFETY: validated length/encoding.
    let maximum =
        unsafe { CFStringGetMaximumSizeForEncoding(characters, K_CF_STRING_ENCODING_UTF8) };
    if characters < 0 || maximum < 0 {
        bail!("SecACL description has invalid length")
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
        bail!("decode SecACL UTF-8 description")
    }
    // SAFETY: successful CFStringGetCString NUL-terminated the destination.
    let string = unsafe { std::ffi::CStr::from_ptr(bytes.as_ptr()) }
        .to_str()
        .context("SecACL description is not UTF-8")?;
    Ok(string.to_owned())
}

unsafe fn take_cf_error_code(owned: &mut OwnedCf, error: CfType) -> isize {
    if error.is_null() {
        0
    } else {
        // SAFETY: error is a live CFError from a Security API.
        let code = unsafe { CFErrorGetCode(error) };
        owned.hold(error);
        code
    }
}

#[link(name = "Security", kind = "framework")]
unsafe extern "C" {
    static kSecClass: CfType;
    static kSecClassKey: CfType;
    static kSecAttrAccess: CfType;
    static kSecAttrApplicationTag: CfType;
    static kSecAttrApplicationLabel: CfType;
    static kSecAttrLabel: CfType;
    static kSecAttrKeyClass: CfType;
    static kSecAttrKeyClassPrivate: CfType;
    static kSecAttrKeyType: CfType;
    static kSecAttrKeyTypeECSECPrimeRandom: CfType;
    static kSecAttrKeySizeInBits: CfType;
    static kSecPrivateKeyAttrs: CfType;
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
    static kSecUseKeychain: CfType;
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

    fn SecKeychainSetUserInteractionAllowed(state: u8) -> OsStatus;
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
    fn SecKeyCreateRandomKey(parameters: CfType, error: *mut CfType) -> SecKey;
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
    fn CFDataCreate(allocator: CfType, bytes: *const u8, length: isize) -> CfType;
    fn CFDataGetTypeID() -> usize;
    fn CFDataGetLength(data: CfType) -> isize;
    fn CFDataGetBytePtr(data: CfType) -> *const u8;
    fn CFNumberCreate(allocator: CfType, number_type: i32, value: *const c_void) -> CfType;
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
    fn CFDictionaryGetTypeID() -> usize;
    fn CFDictionaryCreateMutable(
        allocator: CfType,
        capacity: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> CfMutableDictionary;
    fn CFDictionarySetValue(dictionary: CfMutableDictionary, key: CfType, value: CfType);
    fn CFDictionaryGetValue(dictionary: CfType, key: CfType) -> CfType;
}
