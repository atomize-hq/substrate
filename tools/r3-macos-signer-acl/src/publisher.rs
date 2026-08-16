//! Closed filesystem and Security boundary for the two-repetition disposable publisher.
//!
//! The only effect selector is a root-owned, fixed-path state marker. Every Keychain tag, trusted
//! application, input name, output name, signing domain, and scope is compiled below.

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::{Read, Write};
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, derive_host_effect_plan_v2, document_sha256_v2, parse_canonical_v2,
    signature_payload_v2, validate_executable_identity_v2,
    validate_guest_to_host_successor_capsule_v2, validate_harness_acknowledgement_v2,
    validate_launch_identity_v2, validate_process_identity_v2, validate_protected_cas_binding_v2,
    validate_publisher_pre_removal_receipt_v2, HarnessDurabilityAcknowledgementV2,
    HostRetirementStateV2, HostTargetRoleV2, MacR3SignatureV2, ProtectedCasBindingV2,
    PublisherPreRemovalReceiptV2, TargetSetKindV2, MAC_R3_COORDINATOR_PATH_V2,
    MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2, MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2,
    MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2, MAC_R3_FINALIZER_PATH_V2,
    MAC_R3_FINALIZER_SIGNING_IDENTIFIER_V2, MAC_R3_HOST_RECEIPT_OWNER_V2,
    MAC_R3_HOST_RECEIPT_SIGNATURE_DOMAIN_V2, MAC_R3_PROTECTED_CAS_OWNER_V2,
    MAC_R3_PROTECTED_CAS_SIGNATURE_DOMAIN_V2,
};

use crate::ffi::{DisposableAclKind, FixedP256SignatureV2};
use crate::{
    compiled_disposable_target_config, compiled_disposable_wrong_config,
    DisposableKeyPairCreationReceiptV2, DisposableSignerIdentityV2, ExactDeleteClassification,
    ExactDeleteReceipt, FixedRepetitionV2, NonInteractiveSecurity,
    DISPOSABLE_PUBLISHER_EXECUTABLE_PATH, DISPOSABLE_PUBLISHER_ROOT,
    DISPOSABLE_PUBLISHER_STATE_PATH,
};

const SCHEMA_OWNER: &str = "substrate.r3-macos-disposable-publisher";
const SCHEMA_VERSION: u32 = 2;
const STATE_NAME: &str = "state.v2";
const NEXT_STATE_NAME: &str = ".state.v2.next";
const ROOT_MODE: libc::mode_t = libc::S_IFDIR | 0o700;
const FILE_MODE: libc::mode_t = libc::S_IFREG | 0o600;
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisposablePublisherStateV2 {
    FirstCreatePrepared,
    FirstReceiptPrepared,
    FirstProtectedCasPrepared,
    FirstWrongCleanupPrepared,
    FirstWrongCleanupInvoked,
    FirstAwaitingFinalizer,
    FirstRollbackPrepared,
    FirstRollbackInvoked,
    SecondCreatePrepared,
    SecondReceiptPrepared,
    SecondProtectedCasPrepared,
    SecondWrongCleanupPrepared,
    SecondWrongCleanupInvoked,
    SecondAwaitingFinalizer,
    SecondRollbackPrepared,
    SecondRollbackInvoked,
    Complete,
}

impl DisposablePublisherStateV2 {
    fn marker(self) -> &'static [u8] {
        match self {
            Self::FirstCreatePrepared => b"disposable-publisher-v2:first-create-prepared\n",
            Self::FirstReceiptPrepared => b"disposable-publisher-v2:first-receipt-prepared\n",
            Self::FirstProtectedCasPrepared => {
                b"disposable-publisher-v2:first-protected-cas-prepared\n"
            }
            Self::FirstWrongCleanupPrepared => {
                b"disposable-publisher-v2:first-wrong-cleanup-prepared\n"
            }
            Self::FirstWrongCleanupInvoked => {
                b"disposable-publisher-v2:first-wrong-cleanup-invoked\n"
            }
            Self::FirstAwaitingFinalizer => b"disposable-publisher-v2:first-awaiting-finalizer\n",
            Self::FirstRollbackPrepared => b"disposable-publisher-v2:first-rollback-prepared\n",
            Self::FirstRollbackInvoked => b"disposable-publisher-v2:first-rollback-invoked\n",
            Self::SecondCreatePrepared => b"disposable-publisher-v2:second-create-prepared\n",
            Self::SecondReceiptPrepared => b"disposable-publisher-v2:second-receipt-prepared\n",
            Self::SecondProtectedCasPrepared => {
                b"disposable-publisher-v2:second-protected-cas-prepared\n"
            }
            Self::SecondWrongCleanupPrepared => {
                b"disposable-publisher-v2:second-wrong-cleanup-prepared\n"
            }
            Self::SecondWrongCleanupInvoked => {
                b"disposable-publisher-v2:second-wrong-cleanup-invoked\n"
            }
            Self::SecondAwaitingFinalizer => b"disposable-publisher-v2:second-awaiting-finalizer\n",
            Self::SecondRollbackPrepared => b"disposable-publisher-v2:second-rollback-prepared\n",
            Self::SecondRollbackInvoked => b"disposable-publisher-v2:second-rollback-invoked\n",
            Self::Complete => b"disposable-publisher-v2:complete\n",
        }
    }

    fn parse(bytes: &[u8]) -> Result<Self> {
        Self::all()
            .into_iter()
            .find(|state| state.marker() == bytes)
            .context("disposable publisher state marker is unknown or noncanonical")
    }

    const fn all() -> [Self; 17] {
        [
            Self::FirstCreatePrepared,
            Self::FirstReceiptPrepared,
            Self::FirstProtectedCasPrepared,
            Self::FirstWrongCleanupPrepared,
            Self::FirstWrongCleanupInvoked,
            Self::FirstAwaitingFinalizer,
            Self::FirstRollbackPrepared,
            Self::FirstRollbackInvoked,
            Self::SecondCreatePrepared,
            Self::SecondReceiptPrepared,
            Self::SecondProtectedCasPrepared,
            Self::SecondWrongCleanupPrepared,
            Self::SecondWrongCleanupInvoked,
            Self::SecondAwaitingFinalizer,
            Self::SecondRollbackPrepared,
            Self::SecondRollbackInvoked,
            Self::Complete,
        ]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisposablePublisherActionV2 {
    CreateExactKeyPair,
    SignReceipt,
    SignProtectedCas,
    CleanupWrongSurrogate,
    ObserveFinalizerDeletion,
    RollbackTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DisposablePublisherInvocationReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub repetition: FixedRepetitionV2,
    pub scope_id: String,
    pub action: DisposablePublisherActionV2,
    pub state_before: DisposablePublisherStateV2,
    pub state_after: DisposablePublisherStateV2,
    pub artifact_path: Option<String>,
    pub artifact_sha256: Option<String>,
    pub key_pair: Option<DisposableKeyPairCreationReceiptV2>,
    pub exact_delete: Option<ExactDeleteReceipt>,
}

enum InvocationOutputV2 {
    None,
    Artifact {
        path: String,
        sha256: String,
    },
    KeyPair {
        path: String,
        sha256: String,
        receipt: Box<DisposableKeyPairCreationReceiptV2>,
    },
    ExactDelete {
        path: String,
        sha256: String,
        receipt: ExactDeleteReceipt,
    },
}

struct FixedNames {
    key_receipt: String,
    receipt_unsigned: String,
    receipt_signed: String,
    harness_ack: String,
    cas_unsigned: String,
    cas_signed: String,
    wrong_delete: String,
    rollback_delete: String,
}

impl FixedNames {
    fn for_repetition(repetition: FixedRepetitionV2) -> Self {
        let scope = repetition.finalizer_scope();
        Self {
            key_receipt: format!("{scope}.keys-created.v2.json"),
            receipt_unsigned: format!("{scope}.receipt.unsigned.v2.json"),
            receipt_signed: format!("{scope}.receipt.signed.v2.json"),
            harness_ack: format!("{scope}.harness-ack.v2.json"),
            cas_unsigned: format!("{scope}.protected-cas.unsigned.v2.json"),
            cas_signed: format!("{scope}.protected-cas.signed.v2.json"),
            wrong_delete: format!("{scope}.wrong-cleanup.v2.json"),
            rollback_delete: format!("{scope}.target-rollback.v2.json"),
        }
    }
}

pub struct DisposablePublisherRoot {
    directory: OwnedFd,
}

impl DisposablePublisherRoot {
    pub fn open() -> Result<Self> {
        if Path::new(DISPOSABLE_PUBLISHER_STATE_PATH).parent()
            != Some(Path::new(DISPOSABLE_PUBLISHER_ROOT))
        {
            bail!("compiled publisher state path escaped its compiled root")
        }
        let root = CString::new(DISPOSABLE_PUBLISHER_ROOT).context("encode publisher root")?;
        // SAFETY: fixed NUL-terminated path and no-follow directory flags.
        let raw = unsafe {
            libc::open(
                root.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if raw < 0 {
            return Err(std::io::Error::last_os_error()).context("open fixed publisher root");
        }
        // SAFETY: raw is newly owned.
        let directory = unsafe { OwnedFd::from_raw_fd(raw) };
        require_stat(&fstat(directory.as_raw_fd())?, ROOT_MODE, "publisher root")?;
        Ok(Self { directory })
    }

    pub fn read_state(&self) -> Result<DisposablePublisherStateV2> {
        DisposablePublisherStateV2::parse(&self.read_exact(STATE_NAME)?)
    }

    pub fn advance(
        &self,
        security: &mut NonInteractiveSecurity,
    ) -> Result<DisposablePublisherInvocationReceiptV2> {
        match self.read_state()? {
            DisposablePublisherStateV2::FirstCreatePrepared => {
                self.create_pair(security, FixedRepetitionV2::First)
            }
            DisposablePublisherStateV2::SecondCreatePrepared => {
                self.create_pair(security, FixedRepetitionV2::Second)
            }
            DisposablePublisherStateV2::FirstReceiptPrepared => {
                self.sign_receipt(security, FixedRepetitionV2::First)
            }
            DisposablePublisherStateV2::SecondReceiptPrepared => {
                self.sign_receipt(security, FixedRepetitionV2::Second)
            }
            DisposablePublisherStateV2::FirstProtectedCasPrepared => {
                self.sign_protected_cas(security, FixedRepetitionV2::First)
            }
            DisposablePublisherStateV2::SecondProtectedCasPrepared => {
                self.sign_protected_cas(security, FixedRepetitionV2::Second)
            }
            DisposablePublisherStateV2::FirstWrongCleanupPrepared
            | DisposablePublisherStateV2::FirstWrongCleanupInvoked => {
                self.cleanup_wrong(security, FixedRepetitionV2::First)
            }
            DisposablePublisherStateV2::SecondWrongCleanupPrepared
            | DisposablePublisherStateV2::SecondWrongCleanupInvoked => {
                self.cleanup_wrong(security, FixedRepetitionV2::Second)
            }
            DisposablePublisherStateV2::FirstAwaitingFinalizer => {
                self.observe_finalizer_delete(security, FixedRepetitionV2::First)
            }
            DisposablePublisherStateV2::SecondAwaitingFinalizer => {
                self.observe_finalizer_delete(security, FixedRepetitionV2::Second)
            }
            DisposablePublisherStateV2::FirstRollbackPrepared
            | DisposablePublisherStateV2::FirstRollbackInvoked => {
                self.rollback_target(security, FixedRepetitionV2::First)
            }
            DisposablePublisherStateV2::SecondRollbackPrepared
            | DisposablePublisherStateV2::SecondRollbackInvoked => {
                self.rollback_target(security, FixedRepetitionV2::Second)
            }
            DisposablePublisherStateV2::Complete => {
                bail!("disposable publisher state machine is already complete")
            }
        }
    }

    fn create_pair(
        &self,
        security: &mut NonInteractiveSecurity,
        repetition: FixedRepetitionV2,
    ) -> Result<DisposablePublisherInvocationReceiptV2> {
        let before = create_state(repetition);
        let after = receipt_state(repetition);
        let names = FixedNames::for_repetition(repetition);
        let wrapper_identity_sha256 = prepare_empty_wrapper(repetition)?;
        let target_config = compiled_disposable_target_config(repetition)?;
        let wrong_config = compiled_disposable_wrong_config(repetition)?;
        let target = match security
            .read_disposable_signer(&target_config, DisposableAclKind::Target)?
        {
            Some(identity) => identity,
            None => security.create_disposable_signer(&target_config, DisposableAclKind::Target)?,
        };
        let wrong_surrogate = match security
            .read_disposable_signer(&wrong_config, DisposableAclKind::WrongSurrogate)?
        {
            Some(identity) => identity,
            None => match security
                .create_disposable_signer(&wrong_config, DisposableAclKind::WrongSurrogate)
            {
                Ok(identity) => identity,
                Err(error) => {
                    let rollback = security.delete_disposable_signer(
                        &target_config,
                        DisposableAclKind::Target,
                        &target.identity_sha256,
                    )?;
                    require_deleted(
                        &rollback,
                        "target rollback after wrong-key creation failure",
                    )?;
                    return Err(error).context("create wrong-surrogate disposable signer");
                }
            },
        };
        if target.identity_sha256 == wrong_surrogate.identity_sha256
            || target.spki_der == wrong_surrogate.spki_der
            || target.access_control_sha256 == wrong_surrogate.access_control_sha256
        {
            bail!("target and wrong-surrogate identities are not strictly disjoint")
        }
        let key_pair = DisposableKeyPairCreationReceiptV2 {
            schema_owner: SCHEMA_OWNER.to_owned(),
            schema_version: SCHEMA_VERSION,
            repetition,
            scope_id: repetition.finalizer_scope().to_owned(),
            target,
            wrong_surrogate,
            wrapper_identity_sha256,
        };
        let bytes = canonical_bytes_v2(&key_pair)?;
        self.write_or_verify(&names.key_receipt, &bytes)?;
        self.transition(before, after)?;
        Ok(invocation(
            repetition,
            DisposablePublisherActionV2::CreateExactKeyPair,
            before,
            after,
            InvocationOutputV2::KeyPair {
                path: self.path(&names.key_receipt),
                sha256: substrate_common::macos_retirement_v2::sha256_hex_v2(&bytes),
                receipt: Box::new(key_pair),
            },
        ))
    }

    fn sign_receipt(
        &self,
        security: &NonInteractiveSecurity,
        repetition: FixedRepetitionV2,
    ) -> Result<DisposablePublisherInvocationReceiptV2> {
        let before = receipt_state(repetition);
        let after = cas_state(repetition);
        let names = FixedNames::for_repetition(repetition);
        let keys = self.read_key_receipt(&names, repetition)?;
        if self.exact_file_exists(&names.receipt_signed)? {
            let signed = self.read_exact(&names.receipt_signed)?;
            let receipt: PublisherPreRemovalReceiptV2 = parse_canonical_v2(&signed)?;
            validate_publisher_pre_removal_receipt_v2(&receipt)?;
            if receipt.scope_id != repetition.finalizer_scope()
                || receipt.target_set_kind != TargetSetKindV2::DisposableCapability
                || receipt.publisher_signer_spki_der != keys.target.spki_der
                || receipt.signer_access_control_sha256 != keys.target.access_control_sha256
            {
                bail!("recovered signed receipt crossed its fixed signer or repetition")
            }
            self.transition(before, after)?;
            return Ok(invocation(
                repetition,
                DisposablePublisherActionV2::SignReceipt,
                before,
                after,
                InvocationOutputV2::Artifact {
                    path: self.path(&names.receipt_signed),
                    sha256: substrate_common::macos_retirement_v2::sha256_hex_v2(&signed),
                },
            ));
        }
        let unsigned_bytes = self.read_exact(&names.receipt_unsigned)?;
        let mut receipt: PublisherPreRemovalReceiptV2 = parse_canonical_v2(&unsigned_bytes)?;
        prevalidate_unsigned_receipt(&receipt, repetition, &keys)?;
        let payload =
            signature_payload_v2(&receipt.signature_domain, &receipt.schema_owner, &receipt)?;
        let signature = security
            .sign_disposable_payload(&compiled_disposable_target_config(repetition)?, &payload)?;
        apply_signature(&mut receipt.signature, &signature, &keys.target)?;
        validate_publisher_pre_removal_receipt_v2(&receipt)?;
        let signed = canonical_bytes_v2(&receipt)?;
        self.write_or_verify(&names.receipt_signed, &signed)?;
        self.transition(before, after)?;
        Ok(invocation(
            repetition,
            DisposablePublisherActionV2::SignReceipt,
            before,
            after,
            InvocationOutputV2::Artifact {
                path: self.path(&names.receipt_signed),
                sha256: substrate_common::macos_retirement_v2::sha256_hex_v2(&signed),
            },
        ))
    }

    fn sign_protected_cas(
        &self,
        security: &NonInteractiveSecurity,
        repetition: FixedRepetitionV2,
    ) -> Result<DisposablePublisherInvocationReceiptV2> {
        let before = cas_state(repetition);
        // The wrong surrogate is required by the finalizer's fourth negative control and remains
        // present until the coordinator has durably completed every finalizer effect.
        let after = awaiting_state(repetition);
        let names = FixedNames::for_repetition(repetition);
        let keys = self.read_key_receipt(&names, repetition)?;
        let receipt: PublisherPreRemovalReceiptV2 =
            parse_canonical_v2(&self.read_exact(&names.receipt_signed)?)?;
        validate_publisher_pre_removal_receipt_v2(&receipt)?;
        let acknowledgement: HarnessDurabilityAcknowledgementV2 =
            parse_canonical_v2(&self.read_exact(&names.harness_ack)?)?;
        validate_harness_acknowledgement_v2(&acknowledgement, &receipt)?;
        if self.exact_file_exists(&names.cas_signed)? {
            let signed = self.read_exact(&names.cas_signed)?;
            let binding: ProtectedCasBindingV2 = parse_canonical_v2(&signed)?;
            validate_protected_cas_binding_v2(&binding, &receipt, &acknowledgement)?;
            if binding.scope_id != repetition.finalizer_scope()
                || binding.signature.public_key != keys.target.spki_der
                || binding.signer_access_control_sha256 != keys.target.access_control_sha256
            {
                bail!("recovered signed protected CAS crossed its fixed signer or repetition")
            }
            fill_or_verify_wrapper(repetition, &keys.wrapper_identity_sha256, &signed)?;
            self.transition(before, after)?;
            return Ok(invocation(
                repetition,
                DisposablePublisherActionV2::SignProtectedCas,
                before,
                after,
                InvocationOutputV2::Artifact {
                    path: self.path(&names.cas_signed),
                    sha256: substrate_common::macos_retirement_v2::sha256_hex_v2(&signed),
                },
            ));
        }
        let mut binding: ProtectedCasBindingV2 =
            parse_canonical_v2(&self.read_exact(&names.cas_unsigned)?)?;
        prevalidate_unsigned_binding(
            &binding,
            repetition,
            &keys.target,
            &receipt,
            &acknowledgement,
        )?;
        let payload =
            signature_payload_v2(&binding.signature_domain, &binding.schema_owner, &binding)?;
        let signature = security
            .sign_disposable_payload(&compiled_disposable_target_config(repetition)?, &payload)?;
        apply_signature(&mut binding.signature, &signature, &keys.target)?;
        validate_protected_cas_binding_v2(&binding, &receipt, &acknowledgement)?;
        let signed = canonical_bytes_v2(&binding)?;
        self.write_or_verify(&names.cas_signed, &signed)?;
        fill_or_verify_wrapper(repetition, &keys.wrapper_identity_sha256, &signed)?;
        self.transition(before, after)?;
        Ok(invocation(
            repetition,
            DisposablePublisherActionV2::SignProtectedCas,
            before,
            after,
            InvocationOutputV2::Artifact {
                path: self.path(&names.cas_signed),
                sha256: substrate_common::macos_retirement_v2::sha256_hex_v2(&signed),
            },
        ))
    }

    fn cleanup_wrong(
        &self,
        security: &mut NonInteractiveSecurity,
        repetition: FixedRepetitionV2,
    ) -> Result<DisposablePublisherInvocationReceiptV2> {
        let prepared = wrong_prepared_state(repetition);
        let invoked = wrong_invoked_state(repetition);
        let after = next_repetition_or_complete(repetition);
        let current = self.read_state()?;
        if current == prepared {
            self.transition(prepared, invoked)?;
        } else if current != invoked {
            bail!("wrong-surrogate cleanup is outside its closed cursor")
        }
        let names = FixedNames::for_repetition(repetition);
        let keys = self.read_key_receipt(&names, repetition)?;
        let config = compiled_disposable_wrong_config(repetition)?;
        let exact_delete =
            match security.read_disposable_signer(&config, DisposableAclKind::WrongSurrogate)? {
                Some(observed) => {
                    if observed != keys.wrong_surrogate {
                        bail!("wrong-surrogate identity changed before cleanup")
                    }
                    security.delete_disposable_signer(
                        &config,
                        DisposableAclKind::WrongSurrogate,
                        &observed.identity_sha256,
                    )?
                }
                None => ExactDeleteReceipt {
                    raw_os_status: -25_300,
                    classification: ExactDeleteClassification::AlreadyAbsent,
                    present_after: false,
                },
            };
        require_deleted_or_invoked_absent(&exact_delete)?;
        self.write_or_verify(&names.wrong_delete, &canonical_bytes_v2(&exact_delete)?)?;
        self.transition(invoked, after)?;
        Ok(invocation(
            repetition,
            DisposablePublisherActionV2::CleanupWrongSurrogate,
            current,
            after,
            InvocationOutputV2::ExactDelete {
                path: self.path(&names.wrong_delete),
                sha256: document_sha256_v2(&exact_delete)?,
                receipt: exact_delete,
            },
        ))
    }

    fn observe_finalizer_delete(
        &self,
        security: &NonInteractiveSecurity,
        repetition: FixedRepetitionV2,
    ) -> Result<DisposablePublisherInvocationReceiptV2> {
        let before = awaiting_state(repetition);
        let after = wrong_prepared_state(repetition);
        let names = FixedNames::for_repetition(repetition);
        let keys = self.read_key_receipt(&names, repetition)?;
        let target = security.read_disposable_signer(
            &compiled_disposable_target_config(repetition)?,
            DisposableAclKind::Target,
        )?;
        if let Some(observed) = target {
            if observed != keys.target {
                bail!("target identity changed while awaiting finalizer")
            }
            bail!("target signer is still present while awaiting exact finalizer deletion")
        }
        let wrong = security
            .read_disposable_signer(
                &compiled_disposable_wrong_config(repetition)?,
                DisposableAclKind::WrongSurrogate,
            )?
            .context("wrong-surrogate signer disappeared before residual cleanup")?;
        if wrong != keys.wrong_surrogate {
            bail!("wrong-surrogate identity changed before residual cleanup")
        }
        self.transition(before, after)?;
        Ok(invocation(
            repetition,
            DisposablePublisherActionV2::ObserveFinalizerDeletion,
            before,
            after,
            InvocationOutputV2::None,
        ))
    }

    fn rollback_target(
        &self,
        security: &mut NonInteractiveSecurity,
        repetition: FixedRepetitionV2,
    ) -> Result<DisposablePublisherInvocationReceiptV2> {
        let prepared = rollback_prepared_state(repetition);
        let invoked = rollback_invoked_state(repetition);
        let after = next_repetition_or_complete(repetition);
        let current = self.read_state()?;
        if current == prepared {
            self.transition(prepared, invoked)?;
        } else if current != invoked {
            bail!("target rollback is outside its closed cursor")
        }
        let names = FixedNames::for_repetition(repetition);
        let keys = self.read_key_receipt(&names, repetition)?;
        let config = compiled_disposable_target_config(repetition)?;
        let exact_delete =
            match security.read_disposable_signer(&config, DisposableAclKind::Target)? {
                Some(observed) => {
                    if observed != keys.target {
                        bail!("target identity changed before publisher rollback")
                    }
                    security.delete_disposable_signer(
                        &config,
                        DisposableAclKind::Target,
                        &observed.identity_sha256,
                    )?
                }
                None => ExactDeleteReceipt {
                    raw_os_status: -25_300,
                    classification: ExactDeleteClassification::AlreadyAbsent,
                    present_after: false,
                },
            };
        require_deleted_or_invoked_absent(&exact_delete)?;
        self.write_or_verify(&names.rollback_delete, &canonical_bytes_v2(&exact_delete)?)?;
        self.transition(invoked, after)?;
        Ok(invocation(
            repetition,
            DisposablePublisherActionV2::RollbackTarget,
            current,
            after,
            InvocationOutputV2::ExactDelete {
                path: self.path(&names.rollback_delete),
                sha256: document_sha256_v2(&exact_delete)?,
                receipt: exact_delete,
            },
        ))
    }

    fn read_key_receipt(
        &self,
        names: &FixedNames,
        repetition: FixedRepetitionV2,
    ) -> Result<DisposableKeyPairCreationReceiptV2> {
        let receipt: DisposableKeyPairCreationReceiptV2 =
            parse_canonical_v2(&self.read_exact(&names.key_receipt)?)?;
        if receipt.schema_owner != SCHEMA_OWNER
            || receipt.schema_version != SCHEMA_VERSION
            || receipt.repetition != repetition
            || receipt.scope_id != repetition.finalizer_scope()
        {
            bail!("key-pair receipt crossed its fixed repetition")
        }
        Ok(receipt)
    }

    fn path(&self, name: &str) -> String {
        format!("{DISPOSABLE_PUBLISHER_ROOT}/{name}")
    }

    fn read_exact(&self, name: &str) -> Result<Vec<u8>> {
        validate_component(name)?;
        let name = CString::new(name).context("encode fixed publisher component")?;
        // SAFETY: held directory and one fixed component; no final symlink traversal.
        let raw = unsafe {
            libc::openat(
                self.directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if raw < 0 {
            return Err(std::io::Error::last_os_error()).context("open fixed publisher artifact");
        }
        // SAFETY: raw is newly owned.
        let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
        let metadata = fstat(descriptor.as_raw_fd())?;
        require_stat(&metadata, FILE_MODE, "publisher artifact")?;
        if metadata.st_size < 0 || metadata.st_size as usize > MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2 {
            bail!("publisher artifact is outside its fixed byte bound")
        }
        let mut bytes = Vec::with_capacity(metadata.st_size as usize);
        File::from(descriptor)
            .read_to_end(&mut bytes)
            .context("read fixed publisher artifact")?;
        Ok(bytes)
    }

    fn exact_file_exists(&self, name: &str) -> Result<bool> {
        validate_component(name)?;
        let name = CString::new(name)?;
        let mut metadata = MaybeUninit::<libc::stat>::uninit();
        // SAFETY: held directory, fixed component, writable stat output, and no-follow flag.
        let status = unsafe {
            libc::fstatat(
                self.directory.as_raw_fd(),
                name.as_ptr(),
                metadata.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if status != 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ENOENT) {
                return Ok(false);
            }
            return Err(error).context("inspect fixed publisher artifact");
        }
        // SAFETY: successful fstatat initialized metadata.
        require_stat(
            &unsafe { metadata.assume_init() },
            FILE_MODE,
            "publisher artifact",
        )?;
        Ok(true)
    }

    fn write_or_verify(&self, name: &str, bytes: &[u8]) -> Result<()> {
        if bytes.is_empty() || bytes.len() > MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2 {
            bail!("publisher output is empty or exceeds its fixed bound")
        }
        if let Ok(existing) = self.read_exact(name) {
            if existing == bytes {
                return Ok(());
            }
            bail!("fixed publisher output already exists with different bytes")
        }
        validate_component(name)?;
        let temp_name = format!(".{name}.next");
        self.remove_stale_temp(&temp_name)?;
        self.create_exact(&temp_name, bytes)?;
        let temp = CString::new(temp_name.as_str())?;
        let destination = CString::new(name)?;
        // linkat is an atomic no-overwrite publication in the already-held directory.
        // SAFETY: both are fixed components under the live directory.
        let status = unsafe {
            libc::linkat(
                self.directory.as_raw_fd(),
                temp.as_ptr(),
                self.directory.as_raw_fd(),
                destination.as_ptr(),
                0,
            )
        };
        if status != 0 {
            let error = std::io::Error::last_os_error();
            self.unlink_component(&temp_name)?;
            if error.raw_os_error() == Some(libc::EEXIST) && self.read_exact(name)? == bytes {
                return Ok(());
            }
            return Err(error).context("publish fixed publisher artifact without replacement");
        }
        self.unlink_component(&temp_name)?;
        self.sync_directory()
    }

    fn transition(
        &self,
        expected: DisposablePublisherStateV2,
        next: DisposablePublisherStateV2,
    ) -> Result<()> {
        if self.read_state()? != expected {
            bail!("publisher state differs from its one expected predecessor")
        }
        self.remove_stale_temp(NEXT_STATE_NAME)?;
        self.create_exact(NEXT_STATE_NAME, next.marker())?;
        let next_name = CString::new(NEXT_STATE_NAME)?;
        let state_name = CString::new(STATE_NAME)?;
        // SAFETY: both are fixed components under the held root. Replacing only the state marker is
        // the durable closed-state transition.
        if unsafe {
            libc::renameat(
                self.directory.as_raw_fd(),
                next_name.as_ptr(),
                self.directory.as_raw_fd(),
                state_name.as_ptr(),
            )
        } != 0
        {
            return Err(std::io::Error::last_os_error()).context("replace publisher state marker");
        }
        self.sync_directory()?;
        if self.read_state()? != next {
            bail!("publisher state transition failed exact readback")
        }
        Ok(())
    }

    fn create_exact(&self, name: &str, bytes: &[u8]) -> Result<()> {
        validate_component(name)?;
        let name = CString::new(name)?;
        // SAFETY: fixed component, held root, no-follow, exclusive creation.
        let raw = unsafe {
            libc::openat(
                self.directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                0o600,
            )
        };
        if raw < 0 {
            return Err(std::io::Error::last_os_error()).context("create fixed publisher file");
        }
        // SAFETY: raw is newly owned.
        let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
        // SAFETY: live descriptor and literal ownership/mode.
        if unsafe { libc::fchown(descriptor.as_raw_fd(), 0, 0) } != 0
            || unsafe { libc::fchmod(descriptor.as_raw_fd(), 0o600) } != 0
        {
            return Err(std::io::Error::last_os_error()).context("bind publisher file identity");
        }
        let mut file = File::from(descriptor);
        file.write_all(bytes)
            .context("write fixed publisher file")?;
        file.sync_all().context("sync fixed publisher file")?;
        let metadata = fstat(file.as_raw_fd())?;
        require_stat(&metadata, FILE_MODE, "written publisher file")?;
        if metadata.st_size != bytes.len() as libc::off_t {
            bail!("written publisher file has an unexpected length")
        }
        Ok(())
    }

    fn remove_stale_temp(&self, name: &str) -> Result<()> {
        match self.read_exact(name) {
            Ok(_) => self.unlink_component(name),
            Err(error) => {
                let missing = error
                    .chain()
                    .filter_map(|cause| cause.downcast_ref::<std::io::Error>())
                    .any(|io| io.raw_os_error() == Some(libc::ENOENT));
                if missing {
                    Ok(())
                } else {
                    // openat errors are context-wrapped; a direct fstat/type failure must stop.
                    let component = CString::new(name)?;
                    // SAFETY: fixed component under held root; F_OK does not follow a final symlink
                    // into a useful object, and any present object is rejected rather than removed.
                    if unsafe {
                        libc::faccessat(
                            self.directory.as_raw_fd(),
                            component.as_ptr(),
                            libc::F_OK,
                            libc::AT_SYMLINK_NOFOLLOW,
                        )
                    } == 0
                    {
                        Err(error)
                    } else {
                        Ok(())
                    }
                }
            }
        }
    }

    fn unlink_component(&self, name: &str) -> Result<()> {
        validate_component(name)?;
        let name = CString::new(name)?;
        // SAFETY: fixed file component under held root.
        if unsafe { libc::unlinkat(self.directory.as_raw_fd(), name.as_ptr(), 0) } != 0 {
            return Err(std::io::Error::last_os_error())
                .context("unlink fixed publisher temp file");
        }
        Ok(())
    }

    fn sync_directory(&self) -> Result<()> {
        // SAFETY: held live directory descriptor.
        if unsafe { libc::fsync(self.directory.as_raw_fd()) } != 0 {
            return Err(std::io::Error::last_os_error()).context("sync publisher root");
        }
        Ok(())
    }
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

fn invocation(
    repetition: FixedRepetitionV2,
    action: DisposablePublisherActionV2,
    state_before: DisposablePublisherStateV2,
    state_after: DisposablePublisherStateV2,
    output: InvocationOutputV2,
) -> DisposablePublisherInvocationReceiptV2 {
    let (artifact_path, artifact_sha256, key_pair, exact_delete) = match output {
        InvocationOutputV2::None => (None, None, None, None),
        InvocationOutputV2::Artifact { path, sha256 } => (Some(path), Some(sha256), None, None),
        InvocationOutputV2::KeyPair {
            path,
            sha256,
            receipt,
        } => (Some(path), Some(sha256), Some(*receipt), None),
        InvocationOutputV2::ExactDelete {
            path,
            sha256,
            receipt,
        } => (Some(path), Some(sha256), None, Some(receipt)),
    };
    DisposablePublisherInvocationReceiptV2 {
        schema_owner: SCHEMA_OWNER.to_owned(),
        schema_version: SCHEMA_VERSION,
        repetition,
        scope_id: repetition.finalizer_scope().to_owned(),
        action,
        state_before,
        state_after,
        artifact_path,
        artifact_sha256,
        key_pair,
        exact_delete,
    }
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

fn require_deleted(receipt: &ExactDeleteReceipt, label: &str) -> Result<()> {
    if receipt.raw_os_status != 0
        || receipt.classification != ExactDeleteClassification::DeletedAndAbsent
        || receipt.present_after
    {
        bail!("{label} did not return exact deletion and absence")
    }
    Ok(())
}

fn require_deleted_or_invoked_absent(receipt: &ExactDeleteReceipt) -> Result<()> {
    match receipt.classification {
        ExactDeleteClassification::DeletedAndAbsent
            if receipt.raw_os_status == 0 && !receipt.present_after =>
        {
            Ok(())
        }
        ExactDeleteClassification::AlreadyAbsent
            if receipt.raw_os_status == -25_300 && !receipt.present_after =>
        {
            Ok(())
        }
        _ => bail!("publisher cleanup did not reach exact final absence"),
    }
}

fn create_state(repetition: FixedRepetitionV2) -> DisposablePublisherStateV2 {
    match repetition {
        FixedRepetitionV2::First => DisposablePublisherStateV2::FirstCreatePrepared,
        FixedRepetitionV2::Second => DisposablePublisherStateV2::SecondCreatePrepared,
    }
}
fn receipt_state(repetition: FixedRepetitionV2) -> DisposablePublisherStateV2 {
    match repetition {
        FixedRepetitionV2::First => DisposablePublisherStateV2::FirstReceiptPrepared,
        FixedRepetitionV2::Second => DisposablePublisherStateV2::SecondReceiptPrepared,
    }
}
fn cas_state(repetition: FixedRepetitionV2) -> DisposablePublisherStateV2 {
    match repetition {
        FixedRepetitionV2::First => DisposablePublisherStateV2::FirstProtectedCasPrepared,
        FixedRepetitionV2::Second => DisposablePublisherStateV2::SecondProtectedCasPrepared,
    }
}
fn wrong_prepared_state(repetition: FixedRepetitionV2) -> DisposablePublisherStateV2 {
    match repetition {
        FixedRepetitionV2::First => DisposablePublisherStateV2::FirstWrongCleanupPrepared,
        FixedRepetitionV2::Second => DisposablePublisherStateV2::SecondWrongCleanupPrepared,
    }
}
fn wrong_invoked_state(repetition: FixedRepetitionV2) -> DisposablePublisherStateV2 {
    match repetition {
        FixedRepetitionV2::First => DisposablePublisherStateV2::FirstWrongCleanupInvoked,
        FixedRepetitionV2::Second => DisposablePublisherStateV2::SecondWrongCleanupInvoked,
    }
}
fn awaiting_state(repetition: FixedRepetitionV2) -> DisposablePublisherStateV2 {
    match repetition {
        FixedRepetitionV2::First => DisposablePublisherStateV2::FirstAwaitingFinalizer,
        FixedRepetitionV2::Second => DisposablePublisherStateV2::SecondAwaitingFinalizer,
    }
}
fn rollback_prepared_state(repetition: FixedRepetitionV2) -> DisposablePublisherStateV2 {
    match repetition {
        FixedRepetitionV2::First => DisposablePublisherStateV2::FirstRollbackPrepared,
        FixedRepetitionV2::Second => DisposablePublisherStateV2::SecondRollbackPrepared,
    }
}
fn rollback_invoked_state(repetition: FixedRepetitionV2) -> DisposablePublisherStateV2 {
    match repetition {
        FixedRepetitionV2::First => DisposablePublisherStateV2::FirstRollbackInvoked,
        FixedRepetitionV2::Second => DisposablePublisherStateV2::SecondRollbackInvoked,
    }
}
fn next_repetition_or_complete(repetition: FixedRepetitionV2) -> DisposablePublisherStateV2 {
    match repetition {
        FixedRepetitionV2::First => DisposablePublisherStateV2::SecondCreatePrepared,
        FixedRepetitionV2::Second => DisposablePublisherStateV2::Complete,
    }
}

fn validate_component(value: &str) -> Result<()> {
    if value.is_empty() || value.contains(['/', '\0', '\n', '\r']) || value.len() > 255 {
        bail!("fixed publisher component is unsafe")
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
    let path = format!(
        "{FINALIZER_CAPABILITY_ROOT}/{}",
        repetition.finalizer_scope()
    );
    let path = CString::new(path).context("encode fixed wrapper parent")?;
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
        // Reopen below so creation and recovery use one read-only verification path.
        // Continue into one shared read-only verification path.
    } else if std::io::Error::last_os_error().raw_os_error() == Some(libc::EEXIST) {
        // Crash recovery: verify the existing exact empty inode below.
    } else {
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

fn require_stat(value: &libc::stat, mode: libc::mode_t, label: &str) -> Result<()> {
    if value.st_uid != 0
        || value.st_gid != 0
        || (value.st_mode & (libc::S_IFMT | 0o7777)) != mode
        || (mode & libc::S_IFMT == libc::S_IFREG && value.st_nlink != 1)
    {
        bail!("{label} is not exact root:wheel with its fixed type/mode/link count")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_machine_has_two_disjoint_closed_repetitions() {
        assert_eq!(FixedRepetitionV2::ALL.len(), 2);
        assert_ne!(
            FixedRepetitionV2::First.creator_scope(),
            FixedRepetitionV2::Second.creator_scope()
        );
        assert_ne!(
            FixedRepetitionV2::First.finalizer_scope(),
            FixedRepetitionV2::Second.finalizer_scope()
        );
        for state in DisposablePublisherStateV2::all() {
            assert_eq!(
                DisposablePublisherStateV2::parse(state.marker()).unwrap(),
                state
            );
        }
    }

    #[test]
    fn wrong_surrogate_cleanup_is_strictly_after_finalizer_target_absence() {
        let source = include_str!("publisher.rs");
        let cas = source
            .split("fn sign_protected_cas")
            .nth(1)
            .unwrap()
            .split("fn cleanup_wrong")
            .next()
            .unwrap();
        let cleanup = source
            .split("fn cleanup_wrong")
            .nth(1)
            .unwrap()
            .split("fn observe_finalizer_delete")
            .next()
            .unwrap();
        let observe = source
            .split("fn observe_finalizer_delete")
            .nth(1)
            .unwrap()
            .split("fn rollback_target")
            .next()
            .unwrap();
        assert!(cas.contains("let after = awaiting_state(repetition)"));
        assert!(observe.contains("let after = wrong_prepared_state(repetition)"));
        assert!(observe.contains("wrong-surrogate signer disappeared before residual cleanup"));
        assert!(cleanup.contains("let after = next_repetition_or_complete(repetition)"));
    }

    #[test]
    fn sealed_surface_has_only_fixed_tags_files_domains_and_actions() {
        let source = include_str!("publisher.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        assert!(!production.contains("std::env::args("));
        assert!(!production.contains("std::env::var("));
        assert!(production.contains("std::env::args_os().count() != 1"));
        assert!(production.contains("starts_with(b\"SUBSTRATE_\")"));
        assert!(production.contains("O_NOFOLLOW"));
        assert!(include_str!("ffi.rs").contains("kSecMatchItemList"));
        for scope in FixedRepetitionV2::ALL {
            let target = compiled_disposable_target_config(scope).unwrap();
            let wrong = compiled_disposable_wrong_config(scope).unwrap();
            assert_eq!(
                target.application_tag(),
                format!("{}:signing-key", scope.finalizer_scope()).as_bytes()
            );
            assert_eq!(
                wrong.application_tag(),
                format!("{}:wrong-surrogate-signing-key", scope.finalizer_scope()).as_bytes()
            );
            assert_ne!(target.application_tag(), wrong.application_tag());
        }
        assert_ne!(
            DISPOSABLE_PUBLISHER_EXECUTABLE_PATH,
            crate::EXPERIMENT_FINALIZER_EXECUTABLE_PATH
        );
    }

    #[test]
    fn wrong_surrogate_posture_has_no_finalizer_entry() {
        let target = super::super::ffi::expected_disposable_snapshot_for_test(
            b"publisher",
            b"finalizer",
            DisposableAclKind::Target,
        );
        let wrong = super::super::ffi::expected_disposable_snapshot_for_test(
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
                && entry
                    .authorizations
                    .iter()
                    .any(|authorization| authorization == crate::AUTH_SIGN)
                && entry
                    .authorizations
                    .iter()
                    .any(|authorization| authorization == crate::AUTH_DELETE)
        }));
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
            String::from_utf8(bytes.clone()).unwrap(),
            concat!(
                "{\"device\":7,\"gid\":0,\"inode\":11,\"link_count\":1,\"mode\":33024,",
                "\"path\":\"/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2/",
                "capability/01a00830-f0ef-71b0-a1bd-2a736ad2c6f4/surrogate-wrapper.v2\",",
                "\"uid\":0}"
            )
        );
        assert_eq!(
            substrate_common::macos_retirement_v2::sha256_hex_v2(&bytes),
            "f7981ceba9a9a04a5d1115a83b8121e1734805c6c83afb75f8b23ddf8fb2fcf1"
        );
        let finalizer = include_str!("../../r3-macos-finalizer/src/native_effects.rs");
        for field in [
            "path:",
            "device:",
            "inode:",
            "uid:",
            "gid:",
            "mode:",
            "link_count:",
        ] {
            assert!(finalizer.contains(field));
        }
        assert!(finalizer.contains("disposable_wrapper_observation_from_handle"));
        assert!(finalizer.contains("before.mode() & 0o7777 != 0o400"));
        assert!(finalizer.contains("before.nlink() != 1"));
    }
}
