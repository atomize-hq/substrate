//! Durable, effect-free construction of prospective macOS retirement authority.
//!
//! This module owns only authority documents beneath a constructor-bound root. It never creates
//! signing keys, invokes native target effects, or accepts caller-selected persistence paths.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, derive_frozen_finalization_intent_v2, derive_host_effect_plan_v2,
    document_sha256_v2, encode_base64url_v2, sha256_hex_v2, signature_payload_v2,
    validate_executable_identity_v2, validate_frozen_finalization_intent_v2,
    validate_guest_to_host_successor_capsule_v2, validate_launch_identity_v2,
    validate_process_executable_binding_v2, FinalizationRequestV2, GuestRetirementStateV2,
    HarnessDurabilityAcknowledgementV2, HostRetirementStateV2, HostToFinalizerSuccessorCapsuleV2,
    MacR3SignatureV2, ProtectedCasBindingV2, PublisherPreRemovalReceiptV2, TargetSetKindV2,
    MAC_R3_COORDINATOR_PATH_V2, MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2, MAC_R3_FINALIZER_PATH_V2,
    MAC_R3_FINALIZER_PROTOCOL_OWNER_V2, MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
    MAC_R3_FINALIZER_SIGNING_IDENTIFIER_V2, MAC_R3_HARNESS_ACK_OWNER_V2,
    MAC_R3_HARNESS_ACK_SIGNATURE_DOMAIN_V2, MAC_R3_HOST_RECEIPT_OWNER_V2,
    MAC_R3_HOST_RECEIPT_SIGNATURE_DOMAIN_V2, MAC_R3_PROTECTED_CAS_OWNER_V2,
    MAC_R3_PROTECTED_CAS_SIGNATURE_DOMAIN_V2,
};

pub const R6_TERMINAL_PAIRING_INPUT_OWNER_V2: &str = "substrate.r3-macos-r6-terminal-pairing-input";

const EXTERNAL_DIR: &str = "external";
const PROTECTED_DIR: &str = "protected";
const INBOX_DIR: &str = "inbox";
const RECEIPT_FILE: &str = "publisher-receipt.v2.json";
const ACKNOWLEDGEMENT_FILE: &str = "harness-acknowledgement.v2.json";
const CAS_BINDING_FILE: &str = "protected-cas-binding.v2.json";
const REQUEST_FILE: &str = "finalization-request.v2.json";
const FILE_MODE: u32 = 0o400;
const DIRECTORY_MODE: u32 = 0o700;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DurablePublishCheckpointV2 {
    TemporaryCreated,
    TemporaryWritten,
    TemporaryFsynced,
    PublishLinked,
    PublishParentFsynced,
    TemporaryUnlinked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct R6TerminalPairingHashInputV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub scope_id: String,
    pub predecessor_sha256: String,
    pub terminal_host_record_sha256: String,
    pub guest_anchor_acknowledgement_sha256: String,
    pub guest_consumption_marker_response_sha256: String,
}

#[derive(Debug, Clone)]
pub struct ProspectiveAuthorityDocumentsV2 {
    pub r6_terminal_pairing: R6TerminalPairingHashInputV2,
    pub receipt: PublisherPreRemovalReceiptV2,
    pub acknowledgement: HarnessDurabilityAcknowledgementV2,
    pub protected_cas_binding: ProtectedCasBindingV2,
    pub predecessor_journal_head_sha256: String,
    pub retry_state_sha256: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityGateV2 {
    ReceiptSigned,
    ReceiptExternallyDurable,
    AcknowledgementExternallyDurable,
    AcknowledgementCasBound,
    FinalizationRequestFrozen,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RetirementFrontierV2 {
    pub gate: AuthorityGateV2,
    pub guest: GuestRetirementStateV2,
    pub host: HostRetirementStateV2,
}

pub const AUTHORITY_FRONTIERS_V2: [RetirementFrontierV2; 5] = [
    RetirementFrontierV2 {
        gate: AuthorityGateV2::ReceiptSigned,
        guest: GuestRetirementStateV2::PreRemovalReceiptSigned,
        host: HostRetirementStateV2::PreRemovalReceiptSigned,
    },
    RetirementFrontierV2 {
        gate: AuthorityGateV2::ReceiptExternallyDurable,
        guest: GuestRetirementStateV2::ReceiptExternallyDurable,
        host: HostRetirementStateV2::ReceiptExternallyDurable,
    },
    RetirementFrontierV2 {
        gate: AuthorityGateV2::AcknowledgementExternallyDurable,
        guest: GuestRetirementStateV2::AcknowledgementExternallyDurable,
        host: HostRetirementStateV2::AcknowledgementExternallyDurable,
    },
    RetirementFrontierV2 {
        gate: AuthorityGateV2::AcknowledgementCasBound,
        guest: GuestRetirementStateV2::AcknowledgementCasBound,
        host: HostRetirementStateV2::AcknowledgementCasBound,
    },
    RetirementFrontierV2 {
        gate: AuthorityGateV2::FinalizationRequestFrozen,
        guest: GuestRetirementStateV2::HandoffHostBound,
        host: HostRetirementStateV2::FinalizationRequestFrozen,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureLaneV2 {
    PublisherReceipt,
    HarnessAcknowledgement,
    ProtectedCasBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultCheckpointV2 {
    BeforeReceiptDurability,
    AfterReceiptDurability,
    BeforeAcknowledgementDurability,
    AfterAcknowledgementDurability,
    BeforeProtectedCasBinding,
    AfterProtectedCasBinding,
    BeforeFinalizationRequestDurability,
    AfterFinalizationRequestDurability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityProgressV2 {
    pub frontier: RetirementFrontierV2,
    pub ready_for_finalizer_delivery: bool,
    pub request_digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrozenProspectiveAuthorityV2 {
    pub frontier: RetirementFrontierV2,
    pub request: FinalizationRequestV2,
    pub canonical_request: Vec<u8>,
    pub request_sha256: String,
}

pub struct AuthorityPipelineV2 {
    root: PathBuf,
    expected_uid: u32,
}

impl AuthorityPipelineV2 {
    pub fn new(root: &Path, expected_uid: u32) -> Result<Self> {
        if !root.is_absolute() {
            bail!("authority root must be an absolute constructor-bound path")
        }
        match fs::symlink_metadata(root) {
            Ok(metadata) => validate_directory_metadata(&metadata, expected_uid, "authority root")?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let parent = root
                    .parent()
                    .ok_or_else(|| anyhow!("authority root lacks a parent"))?;
                let parent_metadata =
                    fs::symlink_metadata(parent).context("inspect authority root parent")?;
                if parent_metadata.file_type().is_symlink() || !parent_metadata.is_dir() {
                    bail!("authority root parent is not a direct directory")
                }
                fs::create_dir(root).context("create constructor-bound authority root")?;
                fs::set_permissions(root, fs::Permissions::from_mode(DIRECTORY_MODE))
                    .context("set authority root mode")?;
                sync_directory(parent)?;
                let metadata = fs::symlink_metadata(root)
                    .context("reopen constructor-bound authority root")?;
                validate_directory_metadata(&metadata, expected_uid, "authority root")?;
            }
            Err(error) => return Err(error).context("inspect constructor-bound authority root"),
        }
        let root =
            fs::canonicalize(root).context("canonicalize constructor-bound authority root")?;
        Ok(Self { root, expected_uid })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn advance<V, F>(
        &self,
        documents: &ProspectiveAuthorityDocumentsV2,
        mut verify_signature: V,
        mut fault: F,
    ) -> Result<FrozenProspectiveAuthorityV2>
    where
        V: FnMut(SignatureLaneV2, &[u8], &MacR3SignatureV2) -> Result<()>,
        F: FnMut(FaultCheckpointV2) -> Result<()>,
    {
        let request = validate_and_freeze_authority_v2(documents, &mut verify_signature)?;
        let receipt_bytes = canonical_bytes_v2(&documents.receipt)?;
        let acknowledgement_bytes = canonical_bytes_v2(&documents.acknowledgement)?;
        let binding_bytes = canonical_bytes_v2(&documents.protected_cas_binding)?;
        let request_bytes = canonical_bytes_v2(&request)?;

        let external = self.fixed_directory(EXTERNAL_DIR)?;
        let protected = self.fixed_directory(PROTECTED_DIR)?;
        let inbox = self.fixed_directory(INBOX_DIR)?;

        fault(FaultCheckpointV2::BeforeReceiptDurability)?;
        persist_exact_file(
            &external.join(RECEIPT_FILE),
            &receipt_bytes,
            self.expected_uid,
        )?;
        fault(FaultCheckpointV2::AfterReceiptDurability)?;

        fault(FaultCheckpointV2::BeforeAcknowledgementDurability)?;
        persist_exact_file(
            &external.join(ACKNOWLEDGEMENT_FILE),
            &acknowledgement_bytes,
            self.expected_uid,
        )?;
        fault(FaultCheckpointV2::AfterAcknowledgementDurability)?;

        fault(FaultCheckpointV2::BeforeProtectedCasBinding)?;
        persist_exact_file(
            &protected.join(CAS_BINDING_FILE),
            &binding_bytes,
            self.expected_uid,
        )?;
        fault(FaultCheckpointV2::AfterProtectedCasBinding)?;

        fault(FaultCheckpointV2::BeforeFinalizationRequestDurability)?;
        persist_exact_file(&inbox.join(REQUEST_FILE), &request_bytes, self.expected_uid)?;
        fault(FaultCheckpointV2::AfterFinalizationRequestDurability)?;

        let progress = self.inspect(documents)?;
        if !progress.ready_for_finalizer_delivery
            || progress.frontier.gate != AuthorityGateV2::FinalizationRequestFrozen
            || progress.request_digest.as_deref() != Some(request.request_digest.as_str())
        {
            bail!("authority pipeline did not reach an exact durable finalization request")
        }
        Ok(FrozenProspectiveAuthorityV2 {
            frontier: progress.frontier,
            request_sha256: sha256_hex_v2(&request_bytes),
            canonical_request: request_bytes,
            request,
        })
    }

    pub fn inspect(
        &self,
        documents: &ProspectiveAuthorityDocumentsV2,
    ) -> Result<AuthorityProgressV2> {
        let request = freeze_finalization_request_v2(documents)?;
        let receipt_bytes = canonical_bytes_v2(&documents.receipt)?;
        let acknowledgement_bytes = canonical_bytes_v2(&documents.acknowledgement)?;
        let binding_bytes = canonical_bytes_v2(&documents.protected_cas_binding)?;
        let request_bytes = canonical_bytes_v2(&request)?;

        let external = self.root.join(EXTERNAL_DIR);
        let protected = self.root.join(PROTECTED_DIR);
        let inbox = self.root.join(INBOX_DIR);
        let receipt = exact_file_matches(
            &external.join(RECEIPT_FILE),
            &receipt_bytes,
            self.expected_uid,
        )?;
        let acknowledgement = exact_file_matches(
            &external.join(ACKNOWLEDGEMENT_FILE),
            &acknowledgement_bytes,
            self.expected_uid,
        )?;
        let binding = exact_file_matches(
            &protected.join(CAS_BINDING_FILE),
            &binding_bytes,
            self.expected_uid,
        )?;
        let frozen =
            exact_file_matches(&inbox.join(REQUEST_FILE), &request_bytes, self.expected_uid)?;
        if acknowledgement && !receipt || binding && !acknowledgement || frozen && !binding {
            bail!("authority durability artifacts are out of exact gate order")
        }

        let (gate, ready_for_finalizer_delivery, request_digest) = if frozen {
            (
                AuthorityGateV2::FinalizationRequestFrozen,
                true,
                Some(request.request_digest),
            )
        } else if binding {
            (AuthorityGateV2::AcknowledgementCasBound, false, None)
        } else if acknowledgement {
            (
                AuthorityGateV2::AcknowledgementExternallyDurable,
                false,
                None,
            )
        } else if receipt {
            (AuthorityGateV2::ReceiptExternallyDurable, false, None)
        } else {
            (AuthorityGateV2::ReceiptSigned, false, None)
        };
        Ok(AuthorityProgressV2 {
            frontier: frontier_for_gate_v2(gate),
            ready_for_finalizer_delivery,
            request_digest,
        })
    }

    fn fixed_directory(&self, component: &str) -> Result<PathBuf> {
        if !matches!(component, EXTERNAL_DIR | PROTECTED_DIR | INBOX_DIR) {
            bail!("authority directory is not a closed component")
        }
        let path = self.root.join(component);
        if path.parent() != Some(self.root.as_path()) {
            bail!("authority directory escaped its constructor-bound root")
        }
        ensure_fixed_directory(&path, self.expected_uid)?;
        Ok(path)
    }
}

pub fn frontier_for_gate_v2(gate: AuthorityGateV2) -> RetirementFrontierV2 {
    *AUTHORITY_FRONTIERS_V2
        .iter()
        .find(|frontier| frontier.gate == gate)
        .expect("closed authority gate has an exact frontier")
}

pub fn validate_authority_frontier_transition_v2(
    current: RetirementFrontierV2,
    next: RetirementFrontierV2,
) -> Result<()> {
    let current_index = AUTHORITY_FRONTIERS_V2
        .iter()
        .position(|frontier| *frontier == current)
        .ok_or_else(|| anyhow!("current authority frontier is not closed"))?;
    let next_index = AUTHORITY_FRONTIERS_V2
        .iter()
        .position(|frontier| *frontier == next)
        .ok_or_else(|| anyhow!("next authority frontier is not closed"))?;
    if next_index != current_index + 1 {
        bail!("authority frontier transition is not the exact successor")
    }
    Ok(())
}

pub fn r6_terminal_pairing_sha256_v2(input: &R6TerminalPairingHashInputV2) -> Result<String> {
    if input.schema_owner != R6_TERMINAL_PAIRING_INPUT_OWNER_V2
        || input.schema_version != MAC_R3_FINALIZER_PROTOCOL_VERSION_V2
    {
        bail!("R6 terminal pairing input owner or version changed")
    }
    require_uuid_v7(&input.scope_id, "R6 terminal pairing scope")?;
    for (value, label) in [
        (&input.predecessor_sha256, "R6 predecessor"),
        (
            &input.terminal_host_record_sha256,
            "R6 terminal host record",
        ),
        (
            &input.guest_anchor_acknowledgement_sha256,
            "R6 guest-anchor acknowledgement",
        ),
        (
            &input.guest_consumption_marker_response_sha256,
            "R6 guest consumption-marker response",
        ),
    ] {
        require_digest(value, label)?;
    }
    document_sha256_v2(input)
}

pub fn freeze_finalization_request_v2(
    documents: &ProspectiveAuthorityDocumentsV2,
) -> Result<FinalizationRequestV2> {
    require_digest(
        &documents.predecessor_journal_head_sha256,
        "predecessor journal head",
    )?;
    require_digest(&documents.retry_state_sha256, "retry state")?;
    let intent = derive_frozen_finalization_intent_v2(
        &documents.receipt,
        &documents.acknowledgement,
        &documents.protected_cas_binding,
    )?;
    validate_frozen_finalization_intent_v2(
        &intent,
        &documents.receipt,
        &documents.acknowledgement,
        &documents.protected_cas_binding,
    )?;
    let successor_capsule = HostToFinalizerSuccessorCapsuleV2 {
        predecessor_journal_head_sha256: documents.predecessor_journal_head_sha256.clone(),
        retry_state_sha256: documents.retry_state_sha256.clone(),
        intent,
    };
    let request_digest = document_sha256_v2(&successor_capsule)?;
    if documents.protected_cas_binding.request_digest != request_digest {
        bail!("protected CAS does not bind the one frozen request digest")
    }
    Ok(FinalizationRequestV2 {
        schema_owner: MAC_R3_FINALIZER_PROTOCOL_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        request_digest,
        publisher_receipt: encode_base64url_v2(&canonical_bytes_v2(&documents.receipt)?),
        harness_acknowledgement: encode_base64url_v2(&canonical_bytes_v2(
            &documents.acknowledgement,
        )?),
        protected_cas_binding: encode_base64url_v2(&canonical_bytes_v2(
            &documents.protected_cas_binding,
        )?),
        successor_capsule,
    })
}

fn validate_and_freeze_authority_v2<V>(
    documents: &ProspectiveAuthorityDocumentsV2,
    verify_signature: &mut V,
) -> Result<FinalizationRequestV2>
where
    V: FnMut(SignatureLaneV2, &[u8], &MacR3SignatureV2) -> Result<()>,
{
    validate_receipt(documents, verify_signature)?;
    validate_acknowledgement(documents, verify_signature)?;
    validate_cas_binding(documents, verify_signature)?;
    freeze_finalization_request_v2(documents)
}

fn validate_receipt<V>(documents: &ProspectiveAuthorityDocumentsV2, verify: &mut V) -> Result<()>
where
    V: FnMut(SignatureLaneV2, &[u8], &MacR3SignatureV2) -> Result<()>,
{
    let receipt = &documents.receipt;
    if receipt.schema_owner != MAC_R3_HOST_RECEIPT_OWNER_V2
        || receipt.schema_version != MAC_R3_FINALIZER_PROTOCOL_VERSION_V2
        || receipt.signature_domain != MAC_R3_HOST_RECEIPT_SIGNATURE_DOMAIN_V2
        || receipt.host_state != HostRetirementStateV2::PreRemovalReceiptSigned
        || receipt.target_set_kind != TargetSetKindV2::ProspectiveHost
        || receipt.issued_at_unix_ns >= receipt.expires_at_unix_ns
        || receipt.protected_cas_generation == 0
    {
        bail!("prospective publisher receipt chronology or schema is not closed")
    }
    require_plain(&receipt.evidence_id, "receipt evidence")?;
    require_uuid_v7(&receipt.scope_id, "receipt scope")?;
    if documents.r6_terminal_pairing.scope_id != receipt.scope_id
        || receipt.guest_successor_capsule.scope_id != receipt.scope_id
        || receipt
            .guest_successor_capsule
            .r6_predecessor_consumed_sha256
            != documents.r6_terminal_pairing.predecessor_sha256
        || receipt
            .guest_successor_capsule
            .r6_terminal_host_record_sha256
            != documents.r6_terminal_pairing.terminal_host_record_sha256
        || receipt
            .guest_successor_capsule
            .guest_anchor_acknowledgement_sha256
            != documents
                .r6_terminal_pairing
                .guest_anchor_acknowledgement_sha256
        || receipt
            .guest_successor_capsule
            .guest_consumption_marker_acknowledgement_sha256
            != documents
                .r6_terminal_pairing
                .guest_consumption_marker_response_sha256
        || receipt.guest_successor_capsule.guest_parity_proof_sha256 != receipt.guest_parity_sha256
        || receipt.guest_successor_capsule_sha256
            != document_sha256_v2(&receipt.guest_successor_capsule)?
    {
        bail!("receipt does not bind the exact R6 terminal pairing input")
    }
    r6_terminal_pairing_sha256_v2(&documents.r6_terminal_pairing)?;
    validate_guest_to_host_successor_capsule_v2(&receipt.guest_successor_capsule)?;
    for (value, label) in [
        (&receipt.before_observation_sha256, "before observation"),
        (&receipt.quiesced_observation_sha256, "quiesced observation"),
        (&receipt.protected_cas_head_sha256, "protected CAS head"),
        (&receipt.current_lock_identity_sha256, "current lock"),
        (
            &receipt.signer_access_control_sha256,
            "signer access control",
        ),
        (&receipt.capability_digest, "capability"),
    ] {
        require_digest(value, label)?;
    }
    let plan = derive_host_effect_plan_v2(receipt.target_set_kind, &receipt.target_ledger)?;
    if receipt.target_ledger_sha256 != document_sha256_v2(&receipt.target_ledger)?
        || receipt.effect_plan_sha256 != document_sha256_v2(&plan)?
    {
        bail!("receipt target ledger or effect DAG does not match its digest")
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
    validate_process_executable_binding_v2(
        &receipt.coordinator_process,
        &receipt.coordinator_identity,
        MAC_R3_COORDINATOR_PATH_V2,
        MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2,
    )?;
    validate_launch_identity_v2(&receipt.launch_identity)?;
    require_base64url(&receipt.publisher_signer_spki_der, None, "publisher SPKI")?;
    require_base64url(&receipt.harness_public_key, Some(32), "harness public key")?;
    if receipt.signature.algorithm != "ecdsa-p256-sha256-p1363-low-s-v1"
        || receipt.signature.public_key != receipt.publisher_signer_spki_der
    {
        bail!("receipt signature identity is not the committed publisher")
    }
    let payload = signature_payload_v2(&receipt.signature_domain, &receipt.schema_owner, receipt)?;
    verify(
        SignatureLaneV2::PublisherReceipt,
        &payload,
        &receipt.signature,
    )
}

fn validate_acknowledgement<V>(
    documents: &ProspectiveAuthorityDocumentsV2,
    verify: &mut V,
) -> Result<()>
where
    V: FnMut(SignatureLaneV2, &[u8], &MacR3SignatureV2) -> Result<()>,
{
    let receipt = &documents.receipt;
    let acknowledgement = &documents.acknowledgement;
    if acknowledgement.schema_owner != MAC_R3_HARNESS_ACK_OWNER_V2
        || acknowledgement.schema_version != MAC_R3_FINALIZER_PROTOCOL_VERSION_V2
        || acknowledgement.signature_domain != MAC_R3_HARNESS_ACK_SIGNATURE_DOMAIN_V2
        || acknowledgement.evidence_id != receipt.evidence_id
        || acknowledgement.scope_id != receipt.scope_id
        || acknowledgement.receipt_sha256 != document_sha256_v2(receipt)?
        || acknowledgement.acknowledged_at_unix_ns < receipt.issued_at_unix_ns
        || acknowledgement.acknowledged_at_unix_ns > receipt.expires_at_unix_ns
        || acknowledgement.signature.algorithm != "ed25519-v1"
        || acknowledgement.signature.public_key != receipt.harness_public_key
    {
        bail!("harness acknowledgement does not exact-bind the durable receipt")
    }
    require_digest(
        &acknowledgement.external_store_identity_sha256,
        "external store identity",
    )?;
    require_digest(
        &acknowledgement.durable_observation_sha256,
        "durable receipt observation",
    )?;
    let payload = signature_payload_v2(
        &acknowledgement.signature_domain,
        &acknowledgement.schema_owner,
        acknowledgement,
    )?;
    verify(
        SignatureLaneV2::HarnessAcknowledgement,
        &payload,
        &acknowledgement.signature,
    )
}

fn validate_cas_binding<V>(
    documents: &ProspectiveAuthorityDocumentsV2,
    verify: &mut V,
) -> Result<()>
where
    V: FnMut(SignatureLaneV2, &[u8], &MacR3SignatureV2) -> Result<()>,
{
    let receipt = &documents.receipt;
    let acknowledgement = &documents.acknowledgement;
    let binding = &documents.protected_cas_binding;
    if binding.schema_owner != MAC_R3_PROTECTED_CAS_OWNER_V2
        || binding.schema_version != MAC_R3_FINALIZER_PROTOCOL_VERSION_V2
        || binding.signature_domain != MAC_R3_PROTECTED_CAS_SIGNATURE_DOMAIN_V2
        || binding.evidence_id != receipt.evidence_id
        || binding.scope_id != receipt.scope_id
        || binding.generation != receipt.protected_cas_generation + 1
        || binding.predecessor_head_sha256 != receipt.protected_cas_head_sha256
        || binding.receipt_sha256 != document_sha256_v2(receipt)?
        || binding.acknowledgement_sha256 != document_sha256_v2(acknowledgement)?
        || binding.target_ledger_sha256 != receipt.target_ledger_sha256
        || binding.effect_plan_sha256 != receipt.effect_plan_sha256
        || binding.guest_successor_capsule_sha256 != receipt.guest_successor_capsule_sha256
        || binding.guest_parity_sha256 != receipt.guest_parity_sha256
        || binding.current_lock_identity_sha256 != receipt.current_lock_identity_sha256
        || binding.signer_access_control_sha256 != receipt.signer_access_control_sha256
        || binding.signature.algorithm != "ecdsa-p256-sha256-p1363-low-s-v1"
        || binding.signature.public_key != receipt.publisher_signer_spki_der
    {
        bail!("protected CAS does not exact-bind receipt, acknowledgement, and authority")
    }
    require_digest(&binding.request_digest, "protected CAS request")?;
    let payload = signature_payload_v2(&binding.signature_domain, &binding.schema_owner, binding)?;
    verify(
        SignatureLaneV2::ProtectedCasBinding,
        &payload,
        &binding.signature,
    )
}

fn ensure_fixed_directory(path: &Path, expected_uid: u32) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => validate_directory_metadata(&metadata, expected_uid, "authority directory"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(path).context("create fixed authority directory")?;
            fs::set_permissions(path, fs::Permissions::from_mode(DIRECTORY_MODE))
                .context("set fixed authority directory mode")?;
            sync_directory(path.parent().expect("fixed directory has parent"))?;
            let metadata =
                fs::symlink_metadata(path).context("reopen fixed authority directory")?;
            validate_directory_metadata(&metadata, expected_uid, "authority directory")
        }
        Err(error) => Err(error).context("inspect fixed authority directory"),
    }
}

fn validate_directory_metadata(
    metadata: &fs::Metadata,
    expected_uid: u32,
    label: &str,
) -> Result<()> {
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != expected_uid
        || metadata.mode() & 0o777 != DIRECTORY_MODE
    {
        bail!("{label} is not the exact owned no-follow directory")
    }
    Ok(())
}

fn persist_exact_file(path: &Path, bytes: &[u8], expected_uid: u32) -> Result<String> {
    persist_exact_file_with_fault(path, bytes, expected_uid, |_| Ok(()))
}

fn persist_exact_file_with_fault<F>(
    path: &Path,
    bytes: &[u8],
    expected_uid: u32,
    mut fault: F,
) -> Result<String>
where
    F: FnMut(DurablePublishCheckpointV2) -> Result<()>,
{
    if bytes.is_empty() {
        bail!("authority persistence rejects empty bytes")
    }
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("fixed authority file lacks parent"))?;
    let parent_metadata = fs::symlink_metadata(parent).context("inspect authority file parent")?;
    validate_directory_metadata(&parent_metadata, expected_uid, "authority file parent")?;
    let expected_digest = sha256_hex_v2(bytes);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("fixed authority file name is not UTF-8"))?;
    let temporary = parent.join(format!(".{file_name}.tmp"));
    recover_authority_temporary(&temporary, path, bytes, expected_uid)?;
    if let Some(existing) = read_optional_exact_file(path, expected_uid, FILE_MODE)? {
        if existing != bytes || sha256_hex_v2(&existing) != expected_digest {
            bail!("durable authority file already exists with alternate bytes")
        }
        return Ok(expected_digest);
    }

    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .mode(FILE_MODE)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let mut file = options
        .open(&temporary)
        .context("create no-follow authority temporary file")?;
    fault(DurablePublishCheckpointV2::TemporaryCreated)?;
    file.write_all(bytes)
        .context("write durable authority bytes")?;
    fault(DurablePublishCheckpointV2::TemporaryWritten)?;
    file.sync_all().context("fsync durable authority file")?;
    fault(DurablePublishCheckpointV2::TemporaryFsynced)?;
    drop(file);

    fs::hard_link(&temporary, path).context("publish no-clobber authority file")?;
    fault(DurablePublishCheckpointV2::PublishLinked)?;
    sync_directory(parent)?;
    fault(DurablePublishCheckpointV2::PublishParentFsynced)?;
    fs::remove_file(&temporary).context("remove linked authority temporary name")?;
    fault(DurablePublishCheckpointV2::TemporaryUnlinked)?;
    sync_directory(parent)?;
    let reopened = read_required_exact_file(path, expected_uid, FILE_MODE)?;
    if reopened != bytes || sha256_hex_v2(&reopened) != expected_digest {
        bail!("authority file failed reopen byte/hash verification")
    }
    Ok(expected_digest)
}

fn recover_authority_temporary(
    temporary: &Path,
    published: &Path,
    expected_bytes: &[u8],
    expected_uid: u32,
) -> Result<()> {
    let Some((temporary_bytes, temporary_metadata)) =
        read_optional_authority_recovery_file(temporary, expected_uid)?
    else {
        return Ok(());
    };
    let published_value = read_optional_authority_recovery_file(published, expected_uid)?;
    if let Some((published_bytes, published_metadata)) = published_value {
        if temporary_bytes != expected_bytes
            || published_bytes != expected_bytes
            || temporary_metadata.dev() != published_metadata.dev()
            || temporary_metadata.ino() != published_metadata.ino()
            || temporary_metadata.nlink() != 2
            || published_metadata.nlink() != 2
        {
            bail!("authority temporary and published names are not one exact linked artifact")
        }
        remove_exact_recovery_name(temporary, &temporary_metadata)?;
        sync_directory(
            temporary
                .parent()
                .ok_or_else(|| anyhow!("authority temporary lacks parent"))?,
        )?;
        return Ok(());
    }

    if temporary_metadata.nlink() != 1 || !expected_bytes.starts_with(&temporary_bytes) {
        bail!("unpublished authority temporary is not one exact interrupted byte prefix")
    }
    remove_exact_recovery_name(temporary, &temporary_metadata)?;
    sync_directory(
        temporary
            .parent()
            .ok_or_else(|| anyhow!("authority temporary lacks parent"))?,
    )
}

fn read_optional_authority_recovery_file(
    path: &Path,
    expected_uid: u32,
) -> Result<Option<(Vec<u8>, fs::Metadata)>> {
    let mut file = match OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("open authority recovery file"),
    };
    let before = file.metadata().context("fstat authority recovery file")?;
    if !before.is_file()
        || before.uid() != expected_uid
        || before.mode() & 0o777 != FILE_MODE
        || !(1..=2).contains(&before.nlink())
    {
        bail!("authority recovery file changed type, owner, mode, or link count")
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .context("read authority recovery file")?;
    let after = file.metadata().context("refstat authority recovery file")?;
    if physical_identity(&before) != physical_identity(&after) {
        bail!("authority recovery file changed while reading")
    }
    Ok(Some((bytes, after)))
}

fn remove_exact_recovery_name(path: &Path, held: &fs::Metadata) -> Result<()> {
    let live = fs::symlink_metadata(path).context("reinspect authority recovery name")?;
    if physical_identity(held) != physical_identity(&live) {
        bail!("authority recovery name changed before exact removal")
    }
    fs::remove_file(path).context("remove exact authority recovery name")
}

fn physical_identity(metadata: &fs::Metadata) -> (u64, u64, u32, u32, u32, u64, u64, i64, i64) {
    (
        metadata.dev(),
        metadata.ino(),
        metadata.uid(),
        metadata.gid(),
        metadata.mode(),
        metadata.nlink(),
        metadata.size(),
        metadata.mtime(),
        metadata.mtime_nsec(),
    )
}

fn exact_file_matches(path: &Path, bytes: &[u8], expected_uid: u32) -> Result<bool> {
    match read_optional_exact_file(path, expected_uid, FILE_MODE)? {
        Some(existing) if existing == bytes && sha256_hex_v2(&existing) == sha256_hex_v2(bytes) => {
            Ok(true)
        }
        Some(_) => bail!("durable authority artifact differs from the expected exact bytes"),
        None => Ok(false),
    }
}

fn read_optional_exact_file(path: &Path, expected_uid: u32, mode: u32) -> Result<Option<Vec<u8>>> {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let mut file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("open no-follow authority file"),
    };
    let metadata = file
        .metadata()
        .context("inspect no-follow authority file")?;
    if !metadata.is_file()
        || metadata.uid() != expected_uid
        || metadata.mode() & 0o777 != mode
        || metadata.nlink() != 1
    {
        bail!("authority file identity, ownership, mode, or link count changed")
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .context("read no-follow authority file")?;
    Ok(Some(bytes))
}

fn read_required_exact_file(path: &Path, expected_uid: u32, mode: u32) -> Result<Vec<u8>> {
    read_optional_exact_file(path, expected_uid, mode)?
        .ok_or_else(|| anyhow!("required durable authority file is absent"))
}

fn sync_directory(path: &Path) -> Result<()> {
    File::open(path)
        .context("open authority parent directory for fsync")?
        .sync_all()
        .context("fsync authority parent directory")
}

fn require_base64url(value: &str, expected_len: Option<usize>, label: &str) -> Result<()> {
    if value.is_empty() || value.contains('=') {
        bail!("{label} is empty or padded")
    }
    let decoded = URL_SAFE_NO_PAD
        .decode(value)
        .with_context(|| format!("decode {label}"))?;
    if URL_SAFE_NO_PAD.encode(&decoded) != value
        || expected_len.is_some_and(|len| decoded.len() != len)
    {
        bail!("{label} is not canonical base64url with the expected size")
    }
    Ok(())
}

fn require_digest(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} must be lowercase SHA-256")
    }
    Ok(())
}

fn require_plain(value: &str, label: &str) -> Result<()> {
    if value.is_empty() || value.contains(['\0', '\n', '\r']) {
        bail!("{label} is empty or contains a control separator")
    }
    Ok(())
}

fn require_uuid_v7(value: &str, label: &str) -> Result<()> {
    let bytes = value.as_bytes();
    if bytes.len() != 36
        || ![8_usize, 13, 18, 23]
            .iter()
            .all(|offset| bytes[*offset] == b'-')
        || bytes[14] != b'7'
        || !matches!(bytes[19], b'8' | b'9' | b'a' | b'b')
        || bytes.iter().enumerate().any(|(offset, byte)| {
            !matches!(offset, 8 | 13 | 18 | 23)
                && (!byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
        })
    {
        bail!("{label} must be an exact UUIDv7")
    }
    Ok(())
}

#[cfg(test)]
mod durable_publish_tests {
    use super::*;

    #[test]
    fn every_intra_publish_crash_rejoins_one_exact_immutable_file() {
        let checkpoints = [
            DurablePublishCheckpointV2::TemporaryCreated,
            DurablePublishCheckpointV2::TemporaryWritten,
            DurablePublishCheckpointV2::TemporaryFsynced,
            DurablePublishCheckpointV2::PublishLinked,
            DurablePublishCheckpointV2::PublishParentFsynced,
            DurablePublishCheckpointV2::TemporaryUnlinked,
        ];
        for checkpoint in checkpoints {
            let temporary_root = tempfile::tempdir().unwrap();
            let authority = temporary_root.path().join("authority");
            fs::create_dir(&authority).unwrap();
            fs::set_permissions(&authority, fs::Permissions::from_mode(DIRECTORY_MODE)).unwrap();
            let target = authority.join("receipt.v2.json");
            let bytes = br#"{"closed":"authority"}"#;
            let interrupted = persist_exact_file_with_fault(
                &target,
                bytes,
                // SAFETY: geteuid has no pointer or lifetime preconditions.
                unsafe { libc::geteuid() },
                |observed| {
                    if observed == checkpoint {
                        bail!("injected durable-publish crash")
                    }
                    Ok(())
                },
            );
            assert!(interrupted.is_err(), "{checkpoint:?} must interrupt");
            let digest = persist_exact_file(
                &target,
                bytes,
                // SAFETY: geteuid has no pointer or lifetime preconditions.
                unsafe { libc::geteuid() },
            )
            .unwrap();
            assert_eq!(digest, sha256_hex_v2(bytes));
            assert_eq!(fs::read(&target).unwrap(), bytes);
            assert_eq!(fs::metadata(&target).unwrap().nlink(), 1);
            assert!(!authority.join(".receipt.v2.json.tmp").exists());
        }
    }

    #[test]
    fn alternate_temporary_bytes_are_preserved_and_fail_closed() {
        let temporary_root = tempfile::tempdir().unwrap();
        let authority = temporary_root.path().join("authority");
        fs::create_dir(&authority).unwrap();
        fs::set_permissions(&authority, fs::Permissions::from_mode(DIRECTORY_MODE)).unwrap();
        let target = authority.join("receipt.v2.json");
        let temporary = authority.join(".receipt.v2.json.tmp");
        let mut options = OpenOptions::new();
        options.write(true).create_new(true).mode(FILE_MODE);
        let mut file = options.open(&temporary).unwrap();
        file.write_all(b"alternate").unwrap();
        file.sync_all().unwrap();
        drop(file);
        assert!(persist_exact_file(
            &target,
            b"expected",
            // SAFETY: geteuid has no pointer or lifetime preconditions.
            unsafe { libc::geteuid() },
        )
        .is_err());
        assert_eq!(fs::read(&temporary).unwrap(), b"alternate");
        assert!(!target.exists());
    }
}
