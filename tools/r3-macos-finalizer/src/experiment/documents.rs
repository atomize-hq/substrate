use anyhow::{bail, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signer as _, SigningKey};
use rand_core::OsRng;
use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, derive_frozen_finalization_intent_v2, derive_host_effect_plan_v2,
    document_sha256_v2, encode_base64url_v2, host_target_role_for_locator_v2, signature_payload_v2,
    DisposableCapabilityControlV2, FinalizationRequestV2, FinalizerResponseStateV2,
    FinalizerResponseV2, GuestToHostSuccessorCapsuleV2, HarnessDurabilityAcknowledgementV2,
    HostParityProofV2, HostResourceLocatorV2, HostRetirementStateV2, HostTargetIdentityV2,
    HostToFinalizerSuccessorCapsuleV2, MacR3SignatureV2, ProtectedCasBindingV2,
    PublisherPreRemovalReceiptV2, TargetSetKindV2, TerminalAcknowledgementV2,
    MAC_R3_FINALIZER_PROTOCOL_OWNER_V2, MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
    MAC_R3_GUEST_TO_HOST_CAPSULE_OWNER_V2, MAC_R3_HARNESS_ACK_OWNER_V2,
    MAC_R3_HARNESS_ACK_SIGNATURE_DOMAIN_V2, MAC_R3_HOST_RECEIPT_OWNER_V2,
    MAC_R3_HOST_RECEIPT_SIGNATURE_DOMAIN_V2, MAC_R3_PARITY_OWNER_V2,
    MAC_R3_PARITY_SIGNATURE_DOMAIN_V2, MAC_R3_PROTECTED_CAS_OWNER_V2,
    MAC_R3_PROTECTED_CAS_SIGNATURE_DOMAIN_V2, MAC_R3_TERMINAL_ACK_OWNER_V2,
    MAC_R3_TERMINAL_ACK_SIGNATURE_DOMAIN_V2,
};

use crate::contract::{TerminalBindingRequest, TERMINAL_BINDING_OWNER, TERMINAL_BINDING_VERSION};

use super::durable::DurableFileObservationV2;
use super::publisher_protocol::{IdentityBindingPacketV2, SurrogateCreationReceiptV2};
use super::{RepetitionV2, EXPERIMENT_ID_V2};

const RECEIPT_AUTHORITY_WINDOW_NS: u64 = 30 * 60 * 1_000_000_000;

pub struct EphemeralHarnessSignerV2 {
    key: SigningKey,
    public_key: String,
}

impl EphemeralHarnessSignerV2 {
    pub fn generate_in_memory() -> Self {
        Self::from_seed(SigningKey::generate(&mut OsRng).to_bytes())
    }

    pub fn generate_seed_in_memory() -> [u8; 32] {
        SigningKey::generate(&mut OsRng).to_bytes()
    }

    pub fn from_seed(seed: [u8; 32]) -> Self {
        let key = SigningKey::from_bytes(&seed);
        let public_key = URL_SAFE_NO_PAD.encode(key.verifying_key().as_bytes());
        Self { key, public_key }
    }

    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    fn sign_document<T: serde::Serialize>(
        &self,
        domain: &str,
        owner: &str,
        document: &T,
    ) -> Result<MacR3SignatureV2> {
        let payload = signature_payload_v2(domain, owner, document)?;
        Ok(MacR3SignatureV2 {
            algorithm: "ed25519-v1".to_string(),
            public_key: self.public_key.clone(),
            signature: URL_SAFE_NO_PAD.encode(self.key.sign(&payload).to_bytes()),
        })
    }
}

pub fn disposable_target_ledger_v2(
    creation: &SurrogateCreationReceiptV2,
) -> Result<Vec<HostTargetIdentityV2>> {
    creation.validate(repetition_for_scope(&creation.scope_id)?)?;
    use DisposableCapabilityControlV2 as C;
    use HostResourceLocatorV2 as L;
    let rows = [
        (
            L::DisposableCapabilityControl { control: C::Sign },
            creation.capability_pre_observation_sha256.clone(),
        ),
        (
            L::DisposableCapabilityControl {
                control: C::ExportPrivate,
            },
            creation.capability_pre_observation_sha256.clone(),
        ),
        (
            L::DisposableCapabilityControl {
                control: C::ReplaceAccess,
            },
            creation.capability_pre_observation_sha256.clone(),
        ),
        (
            L::DisposableCapabilityControl {
                control: C::DeleteWrongKey,
            },
            creation.capability_pre_observation_sha256.clone(),
        ),
        (
            L::DisposableProtectedWrapper,
            creation.protected_wrapper_identity_sha256.clone(),
        ),
        (L::SigningKey, creation.target.identity_sha256.clone()),
        (
            L::DisposableCurrentLock,
            creation.current_lock_identity_sha256.clone(),
        ),
        (
            L::RetirementTerminalLatch,
            creation.terminal_latch_identity_sha256.clone(),
        ),
    ];
    Ok(rows
        .into_iter()
        .enumerate()
        .map(
            |(index, (locator, expected_before_sha256))| HostTargetIdentityV2 {
                ordinal: u16::try_from(index + 1).expect("fixed disposable ledger fits u16"),
                role: host_target_role_for_locator_v2(&locator),
                locator,
                expected_before_sha256,
            },
        )
        .collect())
}

pub fn build_unsigned_publisher_receipt_v2(
    repetition: RepetitionV2,
    creation: &SurrogateCreationReceiptV2,
    identity: &IdentityBindingPacketV2,
    harness: &EphemeralHarnessSignerV2,
    issued_at_unix_ns: u64,
) -> Result<PublisherPreRemovalReceiptV2> {
    creation.validate(repetition)?;
    identity.validate_against_creation(repetition, creation)?;
    let expires_at_unix_ns = issued_at_unix_ns
        .checked_add(RECEIPT_AUTHORITY_WINDOW_NS)
        .ok_or_else(|| anyhow::anyhow!("disposable receipt expiry overflow"))?;
    let target_ledger = disposable_target_ledger_v2(creation)?;
    let effect_plan =
        derive_host_effect_plan_v2(TargetSetKindV2::DisposableCapability, &target_ledger)?;
    let guest_successor_capsule = disposable_guest_successor_capsule(repetition);
    let guest_successor_capsule_sha256 = document_sha256_v2(&guest_successor_capsule)?;
    Ok(PublisherPreRemovalReceiptV2 {
        schema_owner: MAC_R3_HOST_RECEIPT_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        signature_domain: MAC_R3_HOST_RECEIPT_SIGNATURE_DOMAIN_V2.to_string(),
        evidence_id: evidence_id(repetition),
        scope_id: repetition.scope_id().to_string(),
        target_set_kind: TargetSetKindV2::DisposableCapability,
        issued_at_unix_ns,
        expires_at_unix_ns,
        host_state: HostRetirementStateV2::PreRemovalReceiptSigned,
        guest_parity_sha256: guest_successor_capsule.guest_parity_proof_sha256.clone(),
        guest_successor_capsule,
        guest_successor_capsule_sha256,
        before_observation_sha256: creation.before_observation_sha256.clone(),
        quiesced_observation_sha256: creation.quiesced_observation_sha256.clone(),
        target_ledger_sha256: document_sha256_v2(&target_ledger)?,
        effect_plan_sha256: document_sha256_v2(&effect_plan)?,
        target_ledger,
        protected_cas_generation: creation.protected_cas_generation,
        protected_cas_head_sha256: creation.protected_cas_head_sha256.clone(),
        current_lock_identity_sha256: creation.current_lock_identity_sha256.clone(),
        signer_access_control_sha256: creation.target.access_control_sha256.clone(),
        publisher_signer_spki_der: creation.target.spki_der.clone(),
        harness_public_key: harness.public_key().to_string(),
        finalizer_identity: identity.finalizer_identity.clone(),
        coordinator_identity: identity.coordinator_identity.clone(),
        coordinator_process: identity.coordinator_process.clone(),
        launch_identity: identity.launch_identity.clone(),
        capability_digest: identity.capability_digest.clone(),
        signature: MacR3SignatureV2::unsigned_p256(creation.target.spki_der.clone()),
    })
}

pub fn build_signed_harness_acknowledgement_v2(
    harness: &EphemeralHarnessSignerV2,
    receipt: &PublisherPreRemovalReceiptV2,
    receipt_observation: &DurableFileObservationV2,
    external_store_identity_sha256: String,
    acknowledged_at_unix_ns: u64,
) -> Result<HarnessDurabilityAcknowledgementV2> {
    if receipt_observation.sha256 != document_sha256_v2(receipt)? {
        bail!("harness cannot acknowledge a receipt that was not durably reobserved")
    }
    let mut acknowledgement = HarnessDurabilityAcknowledgementV2 {
        schema_owner: MAC_R3_HARNESS_ACK_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        signature_domain: MAC_R3_HARNESS_ACK_SIGNATURE_DOMAIN_V2.to_string(),
        evidence_id: receipt.evidence_id.clone(),
        scope_id: receipt.scope_id.clone(),
        receipt_sha256: receipt_observation.sha256.clone(),
        external_store_identity_sha256,
        durable_observation_sha256: receipt_observation.identity_sha256()?,
        acknowledged_at_unix_ns,
        signature: MacR3SignatureV2::unsigned_ed25519(harness.public_key().to_string()),
    };
    acknowledgement.signature = harness.sign_document(
        &acknowledgement.signature_domain,
        &acknowledgement.schema_owner,
        &acknowledgement,
    )?;
    substrate_common::macos_retirement_v2::validate_harness_acknowledgement_v2(
        &acknowledgement,
        receipt,
    )?;
    Ok(acknowledgement)
}

pub fn build_unsigned_protected_cas_v2(
    receipt: &PublisherPreRemovalReceiptV2,
    acknowledgement: &HarnessDurabilityAcknowledgementV2,
    predecessor_journal_head_sha256: &str,
    retry_state_sha256: &str,
) -> Result<ProtectedCasBindingV2> {
    let mut binding = ProtectedCasBindingV2 {
        schema_owner: MAC_R3_PROTECTED_CAS_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        signature_domain: MAC_R3_PROTECTED_CAS_SIGNATURE_DOMAIN_V2.to_string(),
        evidence_id: receipt.evidence_id.clone(),
        scope_id: receipt.scope_id.clone(),
        generation: receipt.protected_cas_generation + 1,
        predecessor_head_sha256: receipt.protected_cas_head_sha256.clone(),
        receipt_sha256: document_sha256_v2(receipt)?,
        acknowledgement_sha256: document_sha256_v2(acknowledgement)?,
        request_digest: "00".repeat(32),
        target_ledger_sha256: receipt.target_ledger_sha256.clone(),
        effect_plan_sha256: receipt.effect_plan_sha256.clone(),
        guest_successor_capsule_sha256: receipt.guest_successor_capsule_sha256.clone(),
        guest_parity_sha256: receipt.guest_parity_sha256.clone(),
        current_lock_identity_sha256: receipt.current_lock_identity_sha256.clone(),
        signer_access_control_sha256: receipt.signer_access_control_sha256.clone(),
        signature: MacR3SignatureV2::unsigned_p256(receipt.publisher_signer_spki_der.clone()),
    };
    let intent = derive_frozen_finalization_intent_v2(receipt, acknowledgement, &binding)?;
    binding.request_digest = document_sha256_v2(&HostToFinalizerSuccessorCapsuleV2 {
        predecessor_journal_head_sha256: predecessor_journal_head_sha256.to_string(),
        retry_state_sha256: retry_state_sha256.to_string(),
        intent,
    })?;
    Ok(binding)
}

pub fn build_finalization_request_v2(
    receipt: &PublisherPreRemovalReceiptV2,
    acknowledgement: &HarnessDurabilityAcknowledgementV2,
    signed_binding: &ProtectedCasBindingV2,
    predecessor_journal_head_sha256: String,
    retry_state_sha256: String,
) -> Result<FinalizationRequestV2> {
    let intent = derive_frozen_finalization_intent_v2(receipt, acknowledgement, signed_binding)?;
    let successor_capsule = HostToFinalizerSuccessorCapsuleV2 {
        predecessor_journal_head_sha256,
        retry_state_sha256,
        intent,
    };
    let request = FinalizationRequestV2 {
        schema_owner: MAC_R3_FINALIZER_PROTOCOL_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        request_digest: document_sha256_v2(&successor_capsule)?,
        publisher_receipt: encode_base64url_v2(&canonical_bytes_v2(receipt)?),
        harness_acknowledgement: encode_base64url_v2(&canonical_bytes_v2(acknowledgement)?),
        protected_cas_binding: encode_base64url_v2(&canonical_bytes_v2(signed_binding)?),
        successor_capsule,
    };
    substrate_common::macos_retirement_v2::validate_finalization_request_v2(&request, None)?;
    Ok(request)
}

pub fn build_signed_parity_and_terminal_v2(
    repetition: RepetitionV2,
    harness: &EphemeralHarnessSignerV2,
    request: &FinalizationRequestV2,
    effects_response: &FinalizerResponseV2,
    baseline_sha256: String,
    after_observation_sha256: String,
    acknowledged_at_unix_ns: u64,
) -> Result<(HostParityProofV2, TerminalAcknowledgementV2)> {
    if effects_response.state != FinalizerResponseStateV2::EffectsComplete
        || effects_response.request_digest != request.request_digest
        || baseline_sha256 != after_observation_sha256
    {
        bail!("parity cannot precede exact EffectsComplete and restored baseline")
    }
    let mut parity = HostParityProofV2 {
        schema_owner: MAC_R3_PARITY_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        signature_domain: MAC_R3_PARITY_SIGNATURE_DOMAIN_V2.to_string(),
        evidence_id: evidence_id(repetition),
        scope_id: repetition.scope_id().to_string(),
        request_digest: request.request_digest.clone(),
        effects_response_sha256: document_sha256_v2(effects_response)?,
        journal_head_sha256: effects_response.journal_head_sha256.clone(),
        baseline_sha256,
        after_observation_sha256,
        exact_parity: true,
        signature: MacR3SignatureV2::unsigned_ed25519(harness.public_key().to_string()),
    };
    parity.signature =
        harness.sign_document(&parity.signature_domain, &parity.schema_owner, &parity)?;
    substrate_common::macos_retirement_v2::validate_host_parity_proof_v2(
        &parity,
        harness.public_key(),
    )?;
    let mut terminal = TerminalAcknowledgementV2 {
        schema_owner: MAC_R3_TERMINAL_ACK_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        signature_domain: MAC_R3_TERMINAL_ACK_SIGNATURE_DOMAIN_V2.to_string(),
        evidence_id: parity.evidence_id.clone(),
        scope_id: parity.scope_id.clone(),
        request_digest: parity.request_digest.clone(),
        effects_response_sha256: parity.effects_response_sha256.clone(),
        parity_proof_sha256: document_sha256_v2(&parity)?,
        journal_head_sha256: parity.journal_head_sha256.clone(),
        acknowledged_at_unix_ns,
        signature: MacR3SignatureV2::unsigned_ed25519(harness.public_key().to_string()),
    };
    terminal.signature = harness.sign_document(
        &terminal.signature_domain,
        &terminal.schema_owner,
        &terminal,
    )?;
    substrate_common::macos_retirement_v2::validate_terminal_acknowledgement_v2(
        &terminal,
        &parity,
        harness.public_key(),
    )?;
    Ok((parity, terminal))
}

pub fn build_terminal_binding_request_v2(
    request: &FinalizationRequestV2,
    effects: &FinalizerResponseV2,
    parity: &HostParityProofV2,
    terminal: &TerminalAcknowledgementV2,
) -> Result<TerminalBindingRequest> {
    if effects.request_digest != request.request_digest
        || terminal.request_digest != request.request_digest
        || terminal.parity_proof_sha256 != document_sha256_v2(parity)?
    {
        bail!("terminal binding inputs do not name one accepted request")
    }
    Ok(TerminalBindingRequest {
        schema_owner: TERMINAL_BINDING_OWNER.to_string(),
        schema_version: TERMINAL_BINDING_VERSION,
        request_digest: request.request_digest.clone(),
        authority_request: encode_base64url_v2(&canonical_bytes_v2(request)?),
        effects_response: encode_base64url_v2(&canonical_bytes_v2(effects)?),
        parity_proof: encode_base64url_v2(&canonical_bytes_v2(parity)?),
        terminal_acknowledgement: encode_base64url_v2(&canonical_bytes_v2(terminal)?),
    })
}

fn disposable_guest_successor_capsule(repetition: RepetitionV2) -> GuestToHostSuccessorCapsuleV2 {
    let digest = |label: &str| {
        substrate_common::macos_retirement_v2::sha256_hex_v2(
            format!(
                "substrate.r3-macos-disposable.guest-capsule.v2\0{}\0{label}",
                repetition.scope_id()
            )
            .as_bytes(),
        )
    };
    GuestToHostSuccessorCapsuleV2 {
        schema_owner: MAC_R3_GUEST_TO_HOST_CAPSULE_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        evidence_id: format!(
            "r3-macos-disposable-finalizer-repetition-{}",
            repetition.ordinal()
        ),
        scope_id: repetition.scope_id().to_string(),
        handoff_request_digest: digest("guest-handoff-request"),
        guest_handoff_capsule_sha256: digest("guest-handoff-capsule"),
        guest_host_acceptance_sha256: digest("guest-host-acceptance"),
        guest_receipt_sha256: digest("guest-receipt"),
        guest_acknowledgement_sha256: digest("guest-acknowledgement"),
        guest_protected_cas_head_sha256: digest("guest-protected-cas-head"),
        guest_effect_plan_sha256: digest("guest-effect-plan"),
        guest_effects_response_sha256: digest("guest-effects-response"),
        guest_parity_proof_sha256: digest("guest-parity-proof"),
        guest_parity_host_binding_sha256: digest("guest-parity-host-binding"),
        guest_journal_head_sha256: digest("guest-journal-head"),
        r6_predecessor_consumed_sha256: digest("r6-predecessor-consumed"),
        r6_terminal_host_record_sha256: digest("r6-terminal-host-record"),
        guest_anchor_acknowledgement_sha256: digest("guest-anchor-acknowledgement"),
        guest_consumption_marker_acknowledgement_sha256: digest(
            "guest-consumption-marker-acknowledgement",
        ),
        retry_state_sha256: digest("retry-state"),
    }
}

pub fn predecessor_journal_head_v2(repetition: RepetitionV2) -> String {
    substrate_common::macos_retirement_v2::sha256_hex_v2(
        format!(
            "substrate.r3-macos-disposable.predecessor-journal.v2\0{}",
            repetition.scope_id()
        )
        .as_bytes(),
    )
}

pub fn retry_state_v2(repetition: RepetitionV2) -> String {
    substrate_common::macos_retirement_v2::sha256_hex_v2(
        format!(
            "substrate.r3-macos-disposable.retry-state.v2\0{}",
            repetition.scope_id()
        )
        .as_bytes(),
    )
}

fn evidence_id(repetition: RepetitionV2) -> String {
    format!(
        "r3-macos-exact-finalizer-disposable-{}-repetition-{}",
        EXPERIMENT_ID_V2,
        repetition.ordinal()
    )
}

fn repetition_for_scope(scope: &str) -> Result<RepetitionV2> {
    RepetitionV2::ALL
        .into_iter()
        .find(|repetition| repetition.scope_id() == scope)
        .ok_or_else(|| anyhow::anyhow!("scope is not one of the two frozen repetitions"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experiment::publisher_protocol::DisposableSignerIdentityV2;
    use substrate_common::macos_retirement_v2::{
        ExecutableIdentityV2, LaunchIdentityV2, ProcessIdentityV2, MAC_R3_COORDINATOR_PATH_V2,
        MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2, MAC_R3_FINALIZER_ENDPOINT_V2,
        MAC_R3_FINALIZER_LAUNCHD_LABEL_V2, MAC_R3_FINALIZER_PATH_V2,
        MAC_R3_FINALIZER_PLIST_PATH_V2, MAC_R3_FINALIZER_SIGNING_IDENTIFIER_V2,
    };

    fn digest(seed: u8) -> String {
        format!("{seed:02x}").repeat(32)
    }

    fn executable(path: &str, identifier: &str, seed: u8) -> ExecutableIdentityV2 {
        ExecutableIdentityV2 {
            source_commit: format!("{:x}", seed % 16).repeat(40),
            source_tree: format!("{:x}", (seed + 1) % 16).repeat(40),
            source_hashes_sha256: digest(seed + 2),
            build_inputs_sha256: digest(seed + 3),
            executable_sha256: digest(seed + 4),
            executable_size: 100,
            intended_path: path.to_string(),
            physical_identity_sha256: digest(seed + 5),
            signing_identifier: identifier.to_string(),
            designated_requirement: format!("identifier {identifier}"),
            cdhash: format!("{:x}", (seed + 6) % 16).repeat(40),
        }
    }

    fn creation() -> SurrogateCreationReceiptV2 {
        let signer = |wrong: bool| DisposableSignerIdentityV2 {
            application_tag_sha256: digest(if wrong { 2 } else { 1 }),
            label: if wrong {
                format!(
                    "{}:wrong-surrogate-signing-key",
                    RepetitionV2::One.scope_id()
                )
            } else {
                format!("{}:signing-key", RepetitionV2::One.scope_id())
            },
            application_label: URL_SAFE_NO_PAD.encode([if wrong { 2 } else { 1 }; 20]),
            application_label_sha256: digest(if wrong { 4 } else { 3 }),
            persistent_reference_sha256: digest(if wrong { 6 } else { 5 }),
            spki_der: URL_SAFE_NO_PAD.encode([if wrong { 8 } else { 7 }; 91]),
            spki_der_sha256: digest(if wrong { 8 } else { 7 }),
            access_control_sha256: digest(if wrong { 10 } else { 9 }),
            identity_sha256: digest(if wrong { 12 } else { 11 }),
        };
        SurrogateCreationReceiptV2 {
            schema_owner: crate::experiment::publisher_protocol::PUBLISHER_PROTOCOL_OWNER_V2.to_string(),
            schema_version: 2,
            experiment_id: EXPERIMENT_ID_V2.to_string(),
            repetition: 1,
            scope_id: RepetitionV2::One.scope_id().to_string(),
            target_set_kind: TargetSetKindV2::DisposableCapability,
            publisher_identity: executable(
                substrate_common::macos_retirement_v2::MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2,
                substrate_common::macos_retirement_v2::MAC_R3_DISPOSABLE_PUBLISHER_SIGNING_IDENTIFIER_V2,
                1,
            ),
            publisher_process_attestation_sha256: digest(21),
            target: signer(false),
            wrong: signer(true),
            capability_pre_observation_sha256: digest(13),
            protected_wrapper_identity_sha256: digest(14),
            current_lock_identity_sha256: digest(15),
            terminal_latch_identity_sha256: digest(16),
            before_observation_sha256: digest(17),
            quiesced_observation_sha256: digest(18),
            protected_cas_generation: 1,
            protected_cas_head_sha256: digest(19),
            securityagent_baseline_sha256: digest(20),
        }
    }

    fn binding(creation: &SurrogateCreationReceiptV2) -> IdentityBindingPacketV2 {
        let coordinator = executable(
            MAC_R3_COORDINATOR_PATH_V2,
            MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2,
            3,
        );
        IdentityBindingPacketV2 {
            schema_owner: crate::experiment::publisher_protocol::PUBLISHER_PROTOCOL_OWNER_V2
                .to_string(),
            schema_version: 2,
            experiment_id: EXPERIMENT_ID_V2.to_string(),
            repetition: 1,
            scope_id: RepetitionV2::One.scope_id().to_string(),
            creation_receipt_sha256: document_sha256_v2(creation).unwrap(),
            candidate_identity_packet_sha256: digest(29),
            finalizer_identity: executable(
                MAC_R3_FINALIZER_PATH_V2,
                MAC_R3_FINALIZER_SIGNING_IDENTIFIER_V2,
                2,
            ),
            coordinator_process: ProcessIdentityV2 {
                effective_uid: 501,
                effective_gid: 20,
                canonical_account: "spensermcconnell".to_string(),
                pidversion_required: true,
                process_start_identity_sha256: digest(30),
                executable_identity_sha256: document_sha256_v2(&coordinator).unwrap(),
            },
            coordinator_identity: coordinator,
            launch_identity: LaunchIdentityV2 {
                launchd_label: MAC_R3_FINALIZER_LAUNCHD_LABEL_V2.to_string(),
                launchd_plist_path: MAC_R3_FINALIZER_PLIST_PATH_V2.to_string(),
                launchd_plist_sha256: digest(31),
                endpoint: MAC_R3_FINALIZER_ENDPOINT_V2.to_string(),
                endpoint_owner_uid: 0,
                endpoint_group_gid: 20,
                endpoint_mode: "0660".to_string(),
                launch_socket_name: "Listener".to_string(),
                finalizer_effective_uid: 0,
            },
            capability_digest: digest(32),
            current_lock_identity_sha256: creation.current_lock_identity_sha256.clone(),
            signer_access_control_sha256: creation.target.access_control_sha256.clone(),
        }
    }

    #[test]
    fn ledger_is_exact_and_documents_preserve_causal_hashes() {
        let creation = creation();
        let ledger = disposable_target_ledger_v2(&creation).unwrap();
        assert_eq!(ledger.len(), 8);
        assert_eq!(
            ledger[0].expected_before_sha256,
            creation.capability_pre_observation_sha256
        );
        assert_eq!(
            ledger[5].expected_before_sha256,
            creation.target.identity_sha256
        );
        assert_eq!(
            ledger[7].expected_before_sha256,
            creation.terminal_latch_identity_sha256
        );

        let harness = EphemeralHarnessSignerV2::generate_in_memory();
        let unsigned = build_unsigned_publisher_receipt_v2(
            RepetitionV2::One,
            &creation,
            &binding(&creation),
            &harness,
            100,
        )
        .unwrap();
        assert!(unsigned.signature.signature.is_empty());
        assert_eq!(unsigned.harness_public_key, harness.public_key());
    }

    #[test]
    fn durable_seed_reload_recovers_the_same_harness_successor_key() {
        let seed = EphemeralHarnessSignerV2::generate_seed_in_memory();
        let first = EphemeralHarnessSignerV2::from_seed(seed);
        let after_crash = EphemeralHarnessSignerV2::from_seed(seed);
        assert_eq!(first.public_key(), after_crash.public_key());
        assert_ne!(
            first.public_key(),
            EphemeralHarnessSignerV2::generate_in_memory().public_key()
        );
    }
}
