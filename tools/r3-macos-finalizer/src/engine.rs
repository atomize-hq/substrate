use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, derive_host_effect_plan_v2, parse_canonical_v2, sha256_hex_v2,
    validate_finalization_request_v2, validate_finalizer_response_v2,
    validate_host_parity_proof_v2, validate_terminal_acknowledgement_v2, FinalizationRequestV2,
    FinalizerResponseStateV2, FinalizerResponseV2, HostEffectPlanEntryV2, HostParityProofV2,
    HostRetirementStateV2, HostTargetIdentityV2, HostTargetRoleV2, PreservingStopClassificationV2,
    PublisherPreRemovalReceiptV2, TargetSetKindV2, TerminalAcknowledgementV2,
    MAC_R3_FINALIZER_PROTOCOL_OWNER_V2, MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
};

use crate::contract::{
    JournalEvent, JournalEventKind, PeerAttestation, MAX_EFFECT_INVOCATION_ATTEMPTS_V2,
};
use crate::journal::{JournalClaimIdentity, JournalHead, LockedJournal};
use crate::targets::{derive_targets_from_ledger, DerivedTarget};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObservationClass {
    ExactBefore,
    ExactFinal,
    Ambiguous,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EffectObservation {
    pub class: ObservationClass,
    pub observation_sha256: String,
}

const EFFECT_OBSERVATION_OWNER: &str = "substrate.mac-r3-finalizer-effect-observation";
const EFFECT_OBSERVATION_VERSION: u32 = 2;

/// Root-journal artifact bound by every `EffectObserved` generation.  Platform-specific evidence
/// stays closed and canonical while the engine retains a platform-neutral schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EffectObservationArtifactV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub effect_ordinal: u16,
    pub effect_role: substrate_common::macos_retirement_v2::HostTargetRoleV2,
    pub effect_identity_sha256: String,
    pub effect_invocation_attempt: u16,
    pub state_observation_sha256: String,
    pub invocation_evidence_sha256: Option<String>,
    pub invocation_evidence_canonical_json: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffectObservationPhase {
    Prepared,
    Invoked,
}

pub trait ClosedEffects {
    fn observe(
        &mut self,
        target: &DerivedTarget,
        expected_before_sha256: &str,
        phase: EffectObservationPhase,
    ) -> Result<EffectObservation>;

    fn invoke(&mut self, target: &DerivedTarget, expected_before_sha256: &str) -> Result<()>;

    /// Return canonical evidence from the exact invocation, when that invocation has a bounded
    /// native receipt.  The default deliberately exposes no ambient or caller-selected data.
    fn invocation_evidence(&self, _target: &DerivedTarget) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub struct AcceptedRequest {
    validated_receipt: Option<PublisherPreRemovalReceiptV2>,
    scope_id: String,
    request_digest: String,
    canonical_request: Vec<u8>,
    successor_capsule_sha256: String,
    peer_attestation: Vec<u8>,
    peer_attestation_sha256: String,
    /// Restart-stable coordinator principal digest. Unlike the full attestation digest, this
    /// deliberately excludes PID, audit token, and process-start identity.
    accepted_peer_identity_sha256: String,
    target_set_kind: TargetSetKindV2,
    target_ledger: Vec<HostTargetIdentityV2>,
    first_acceptance_time_verified: bool,
}

impl AcceptedRequest {
    pub fn admit_canonical_request(
        canonical_request: Vec<u8>,
        peer_attestation: Vec<u8>,
        first_acceptance_at_unix_ns: Option<u64>,
    ) -> Result<Self> {
        if first_acceptance_at_unix_ns.is_none() {
            bail!("first acceptance requires an explicitly verified authority time")
        }
        Self::admit_canonical_request_for_mode(
            canonical_request,
            peer_attestation,
            first_acceptance_at_unix_ns,
            PeerAdmissionMode::Initial,
        )
    }

    /// Re-admit the exact signed request for an already-durable claim from a new process instance
    /// of the same coordinator principal. The journal must still exact-join the request and stable
    /// executable/code identity before this token can authorize replay or recovery.
    pub fn admit_canonical_rejoin_request(
        canonical_request: Vec<u8>,
        peer_attestation: Vec<u8>,
    ) -> Result<Self> {
        Self::admit_canonical_request_for_mode(
            canonical_request,
            peer_attestation,
            None,
            PeerAdmissionMode::Rejoin,
        )
    }

    fn admit_canonical_request_for_mode(
        canonical_request: Vec<u8>,
        peer_attestation: Vec<u8>,
        first_acceptance_at_unix_ns: Option<u64>,
        mode: PeerAdmissionMode,
    ) -> Result<Self> {
        let first_acceptance_time_verified = mode == PeerAdmissionMode::Initial;
        let request: FinalizationRequestV2 = parse_canonical_v2(&canonical_request)?;
        let (receipt, _, _) =
            validate_finalization_request_v2(&request, first_acceptance_at_unix_ns)?;
        let peer: PeerAttestation = parse_canonical_v2(&peer_attestation)?;
        validate_peer_attestation(&peer, &receipt, mode)?;
        let peer_attestation_sha256 = sha256_hex_v2(&peer_attestation);
        let accepted_peer_identity_sha256 = stable_peer_identity_sha256(&peer)?;
        let successor_capsule_sha256 =
            substrate_common::macos_retirement_v2::document_sha256_v2(&request.successor_capsule)?;
        Ok(Self {
            scope_id: receipt.scope_id.clone(),
            request_digest: request.request_digest,
            canonical_request,
            successor_capsule_sha256,
            peer_attestation,
            peer_attestation_sha256,
            accepted_peer_identity_sha256,
            target_set_kind: receipt.target_set_kind,
            target_ledger: receipt.target_ledger.clone(),
            validated_receipt: Some(receipt),
            first_acceptance_time_verified,
        })
    }

    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    fn from_test_parts(
        scope_id: String,
        request_digest: String,
        canonical_request: Vec<u8>,
        successor_capsule_sha256: String,
        peer_attestation: Vec<u8>,
        accepted_peer_identity_sha256: String,
        target_set_kind: TargetSetKindV2,
        target_ledger: Vec<HostTargetIdentityV2>,
    ) -> Result<Self> {
        require_scope_uuid_v7(&scope_id)?;
        require_digest(&request_digest)?;
        require_digest(&successor_capsule_sha256)?;
        require_digest(&accepted_peer_identity_sha256)?;
        if canonical_request.is_empty() || peer_attestation.is_empty() {
            bail!("test accepted request lacks its exact authority or peer-attestation bytes")
        }
        let peer_attestation_sha256 = sha256_hex_v2(&peer_attestation);
        derive_host_effect_plan_v2(target_set_kind, &target_ledger)?;
        Ok(Self {
            validated_receipt: None,
            scope_id,
            request_digest,
            canonical_request,
            successor_capsule_sha256,
            peer_attestation,
            peer_attestation_sha256,
            accepted_peer_identity_sha256,
            target_set_kind,
            target_ledger,
            first_acceptance_time_verified: true,
        })
    }

    pub fn scope_id(&self) -> &str {
        &self.scope_id
    }

    /// The exact signed receipt returned by canonical request admission. Native effect adapters
    /// must consume this value rather than independently decoded or caller-supplied authority.
    pub fn validated_receipt(&self) -> Result<&PublisherPreRemovalReceiptV2> {
        self.validated_receipt
            .as_ref()
            .context("accepted request lacks its validated signed receipt")
    }

    pub fn request_digest(&self) -> &str {
        &self.request_digest
    }

    pub fn canonical_request(&self) -> &[u8] {
        &self.canonical_request
    }

    pub fn successor_capsule_sha256(&self) -> &str {
        &self.successor_capsule_sha256
    }

    pub fn accepted_peer_identity_sha256(&self) -> &str {
        &self.accepted_peer_identity_sha256
    }

    pub fn peer_attestation_sha256(&self) -> &str {
        &self.peer_attestation_sha256
    }

    pub fn target_ledger(&self) -> &[HostTargetIdentityV2] {
        &self.target_ledger
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExistingClaimDisposition {
    NoExistingClaim,
    RequiresEffectsRecovery(ValidatedRecoveryCursor),
    Response(FinalizerResponseV2),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryEffectPhase {
    NotPrepared,
    Prepared,
    Invoked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryEffectCursor {
    ordinal: u16,
    role: substrate_common::macos_retirement_v2::HostTargetRoleV2,
    effect_identity_sha256: String,
    phase: RecoveryEffectPhase,
    effect_invocation_attempt: u16,
}

impl RecoveryEffectCursor {
    pub fn ordinal(&self) -> u16 {
        self.ordinal
    }

    pub fn role(&self) -> substrate_common::macos_retirement_v2::HostTargetRoleV2 {
        self.role
    }

    pub fn effect_identity_sha256(&self) -> &str {
        &self.effect_identity_sha256
    }

    pub fn phase(&self) -> RecoveryEffectPhase {
        self.phase
    }

    pub fn effect_invocation_attempt(&self) -> u16 {
        self.effect_invocation_attempt
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedRecoveryCursor {
    journal_generation: u64,
    journal_head_sha256: String,
    completed_effect_ordinal: u16,
    effect_count: u16,
    pending: Option<RecoveryEffectCursor>,
}

impl ValidatedRecoveryCursor {
    pub fn journal_generation(&self) -> u64 {
        self.journal_generation
    }

    pub fn journal_head_sha256(&self) -> &str {
        &self.journal_head_sha256
    }

    pub fn completed_effect_ordinal(&self) -> u16 {
        self.completed_effect_ordinal
    }

    pub fn effect_count(&self) -> u16 {
        self.effect_count
    }

    pub fn pending(&self) -> Option<&RecoveryEffectCursor> {
        self.pending.as_ref()
    }
}

/// Rejoin an accepted claim before constructing a native effects adapter. Immutable terminal,
/// preserving-stop, and EffectsComplete responses are returned without deriving physical targets
/// or inspecting any product/disposable resource.
pub fn replay_existing_claim(
    journal_root: &Path,
    expected_journal_uid: u32,
    accepted: &AcceptedRequest,
) -> Result<ExistingClaimDisposition> {
    let Some(journal) =
        LockedJournal::open_existing_fixed(journal_root, &accepted.scope_id, expected_journal_uid)?
    else {
        return Ok(ExistingClaimDisposition::NoExistingClaim);
    };
    let Some(head) = journal.head()? else {
        return Ok(ExistingClaimDisposition::NoExistingClaim);
    };
    require_same_claim(&head, accepted)?;
    let effect_plan =
        derive_host_effect_plan_v2(accepted.target_set_kind, &accepted.target_ledger)?;
    validate_journal_effect_plan(&journal, &head, &effect_plan)?;
    journal.persist_acceptance_artifact(
        "authority.request",
        &accepted.canonical_request,
        &sha256_hex_v2(&accepted.canonical_request),
    )?;
    persist_rejoin_peer_attestation(&journal, &head, accepted)?;
    match immutable_replay_response(&journal, &head, accepted)? {
        Some(response) => Ok(ExistingClaimDisposition::Response(response)),
        None => Ok(ExistingClaimDisposition::RequiresEffectsRecovery(
            validated_recovery_cursor(&head, &effect_plan)?,
        )),
    }
}

/// Convert failure to reconstruct the native recovery adapter into one fixed, immutable
/// preserving result. This is valid only after the exact request has already crossed the durable
/// FinalizerAccepted boundary. It neither accepts a new claim nor derives or inspects a target.
pub fn record_accepted_recovery_preparation_failure(
    journal_root: &Path,
    expected_journal_uid: u32,
    accepted: &AcceptedRequest,
) -> Result<FinalizerResponseV2> {
    let journal =
        LockedJournal::open_existing_fixed(journal_root, &accepted.scope_id, expected_journal_uid)?
            .context("native recovery preparation failed before an accepted journal existed")?;
    let head = journal
        .head()?
        .context("native recovery preparation failed before FinalizerAccepted")?;
    require_same_claim(&head, accepted)?;
    persist_rejoin_peer_attestation(&journal, &head, accepted)?;
    let effect_plan =
        derive_host_effect_plan_v2(accepted.target_set_kind, &accepted.target_ledger)?;
    validate_journal_effect_plan(&journal, &head, &effect_plan)?;
    if let Some(response) = immutable_replay_response(&journal, &head, accepted)? {
        return Ok(response);
    }
    validated_recovery_cursor(&head, &effect_plan)?;
    preserving_stop(
        &journal,
        Some(&head),
        accepted,
        PreservingStopClassificationV2::AmbiguousEffectState,
    )
}

pub struct FinalizerEngine<'a, E> {
    journal_root: &'a Path,
    expected_journal_uid: u32,
    effects: E,
}

impl<'a, E: ClosedEffects> FinalizerEngine<'a, E> {
    pub fn new(journal_root: &'a Path, expected_journal_uid: u32, effects: E) -> Self {
        Self {
            journal_root,
            expected_journal_uid,
            effects,
        }
    }

    pub fn execute(mut self, accepted: &AcceptedRequest) -> Result<FinalizerResponseV2> {
        require_digest(&accepted.request_digest)?;
        require_digest(&accepted.successor_capsule_sha256)?;
        require_digest(&accepted.peer_attestation_sha256)?;
        require_digest(&accepted.accepted_peer_identity_sha256)?;
        if accepted.canonical_request.is_empty()
            || accepted.peer_attestation.is_empty()
            || sha256_hex_v2(&accepted.peer_attestation) != accepted.peer_attestation_sha256
        {
            bail!("accepted request lacks its exact authority or peer-attestation bytes")
        }
        let authority_bytes_sha256 = sha256_hex_v2(&accepted.canonical_request);
        let effect_plan =
            derive_host_effect_plan_v2(accepted.target_set_kind, &accepted.target_ledger)?;
        match replay_existing_claim(self.journal_root, self.expected_journal_uid, accepted)? {
            ExistingClaimDisposition::Response(response) => return Ok(response),
            ExistingClaimDisposition::NoExistingClaim => {
                if !accepted.first_acceptance_time_verified {
                    bail!("first acceptance requires an explicitly verified authority time")
                }
            }
            ExistingClaimDisposition::RequiresEffectsRecovery(_) => {}
        }
        let derived = derive_targets_from_ledger(
            accepted.target_set_kind,
            &accepted.scope_id,
            &accepted.target_ledger,
        )?;
        if derived.len() != effect_plan.len()
            || derived.iter().zip(&effect_plan).any(|(target, effect)| {
                target.ordinal != effect.ordinal
                    || target.role != effect.role
                    || target.effect_identity_sha256 != effect.target_identity_sha256
            })
        {
            bail!("accepted target ledger is not the compiled target expansion")
        }
        let journal = LockedJournal::open_fixed(
            self.journal_root,
            &accepted.scope_id,
            self.expected_journal_uid,
        )?;
        let mut head = journal.head()?;
        if let Some(existing) = &head {
            require_same_claim(existing, accepted)?;
            persist_rejoin_peer_attestation(&journal, existing, accepted)?;
            validate_journal_effect_plan(&journal, existing, &effect_plan)?;
            if let Some(response) = immutable_replay_response(&journal, existing, accepted)? {
                return Ok(response);
            }
        } else {
            // Both authority and exact initial process-instance evidence are durable before the
            // irreversible FinalizerAccepted generation. A crash can therefore never leave an
            // accepted claim whose original peer evidence is reconstructible only from a rejoin.
            journal.persist_acceptance_artifact(
                "authority.request",
                &accepted.canonical_request,
                &authority_bytes_sha256,
            )?;
            journal.persist_acceptance_artifact(
                "peer.attestation",
                &accepted.peer_attestation,
                &accepted.peer_attestation_sha256,
            )?;
            head = Some(append_event(
                &journal,
                None,
                accepted,
                JournalEvent {
                    kind: JournalEventKind::FinalizerAccepted,
                    host_state: HostRetirementStateV2::FinalizerAccepted,
                    effect_ordinal: None,
                    effect_role: None,
                    effect_identity_sha256: None,
                    effect_invocation_attempt: None,
                    observation_sha256: None,
                    response_sha256: None,
                    preserving_classification: None,
                },
            )?);
        }
        journal.persist_acceptance_artifact(
            "authority.request",
            &accepted.canonical_request,
            &authority_bytes_sha256,
        )?;

        for (target, identity) in derived.iter().zip(&accepted.target_ledger) {
            let phase = effect_phase(head.as_ref().expect("accepted head"), target)?;
            match phase {
                EffectRecoveryPhase::NotPrepared => {
                    head = Some(append_effect_event(
                        &journal,
                        head.as_ref(),
                        accepted,
                        JournalEventKind::EffectPrepared,
                        target,
                        0,
                        None,
                    )?);
                }
                EffectRecoveryPhase::Observed => continue,
                EffectRecoveryPhase::Prepared | EffectRecoveryPhase::Invoked(_) => {}
            }

            let phase = effect_phase(head.as_ref().expect("prepared head"), target)?;
            let (observation_phase, next_invocation_attempt) = match phase {
                EffectRecoveryPhase::Prepared => (EffectObservationPhase::Prepared, 1),
                EffectRecoveryPhase::Invoked(attempt) => (
                    EffectObservationPhase::Invoked,
                    match attempt.checked_add(1) {
                        Some(next) if next <= MAX_EFFECT_INVOCATION_ATTEMPTS_V2 => next,
                        _ => 0,
                    },
                ),
                _ => bail!("finalizer effect cursor changed outside Prepared or Invoked recovery"),
            };
            let observation = match self.effects.observe(
                target,
                &identity.expected_before_sha256,
                observation_phase,
            ) {
                Ok(observation) => observation,
                Err(_) => {
                    return preserving_stop(
                        &journal,
                        head.as_ref(),
                        accepted,
                        PreservingStopClassificationV2::AmbiguousEffectState,
                    )
                }
            };
            match observation.class {
                ObservationClass::ExactFinal => {
                    head = Some(
                        match append_durable_effect_observed(
                            &self.effects,
                            &journal,
                            head.as_ref(),
                            accepted,
                            target,
                            &observation,
                        ) {
                            Ok(head) => head,
                            Err(_) => {
                                return preserving_stop(
                                    &journal,
                                    head.as_ref(),
                                    accepted,
                                    PreservingStopClassificationV2::AmbiguousEffectState,
                                )
                            }
                        },
                    );
                    continue;
                }
                ObservationClass::ExactBefore if next_invocation_attempt == 0 => {
                    // The exact-before observation authorizes a retry, but the closed retry
                    // budget is exhausted. Preserve state before appending another invocation or
                    // calling the one fixed system effect.
                    return preserving_stop(
                        &journal,
                        head.as_ref(),
                        accepted,
                        PreservingStopClassificationV2::AmbiguousEffectState,
                    );
                }
                ObservationClass::ExactBefore => {}
                ObservationClass::Ambiguous => {
                    return preserving_stop(
                        &journal,
                        head.as_ref(),
                        accepted,
                        PreservingStopClassificationV2::AmbiguousEffectState,
                    )
                }
            }

            head = Some(append_effect_event(
                &journal,
                head.as_ref(),
                accepted,
                JournalEventKind::EffectInvoked,
                target,
                next_invocation_attempt,
                None,
            )?);
            let invocation_failed = self
                .effects
                .invoke(target, &identity.expected_before_sha256)
                .is_err();
            let observation = match self.effects.observe(
                target,
                &identity.expected_before_sha256,
                EffectObservationPhase::Invoked,
            ) {
                Ok(observation) => observation,
                Err(_) => {
                    return preserving_stop(
                        &journal,
                        head.as_ref(),
                        accepted,
                        PreservingStopClassificationV2::AmbiguousEffectState,
                    )
                }
            };
            if invocation_failed && observation.class != ObservationClass::ExactFinal {
                return preserving_stop(
                    &journal,
                    head.as_ref(),
                    accepted,
                    PreservingStopClassificationV2::AmbiguousEffectState,
                );
            }
            if observation.class != ObservationClass::ExactFinal {
                return preserving_stop(
                    &journal,
                    head.as_ref(),
                    accepted,
                    PreservingStopClassificationV2::AmbiguousEffectState,
                );
            }
            head = Some(
                match append_durable_effect_observed(
                    &self.effects,
                    &journal,
                    head.as_ref(),
                    accepted,
                    target,
                    &observation,
                ) {
                    Ok(head) => head,
                    Err(_) => {
                        return preserving_stop(
                            &journal,
                            head.as_ref(),
                            accepted,
                            PreservingStopClassificationV2::AmbiguousEffectState,
                        )
                    }
                },
            );
        }

        let effects_observation_sha256 = sha256_hex_v2(&canonical_bytes_v2(
            &head.as_ref().expect("effect head").record,
        )?);
        head = Some(append_event(
            &journal,
            head.as_ref(),
            accepted,
            JournalEvent {
                kind: JournalEventKind::EffectsComplete,
                host_state: HostRetirementStateV2::EffectsComplete,
                effect_ordinal: None,
                effect_role: None,
                effect_identity_sha256: None,
                effect_invocation_attempt: None,
                observation_sha256: Some(effects_observation_sha256.clone()),
                response_sha256: None,
                preserving_classification: None,
            },
        )?);
        Ok(FinalizerResponseV2 {
            schema_owner: MAC_R3_FINALIZER_PROTOCOL_OWNER_V2.to_string(),
            schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
            request_digest: accepted.request_digest.clone(),
            state: FinalizerResponseStateV2::EffectsComplete,
            journal_head_sha256: head.expect("effects head").sha256,
            effects_observation_sha256: Some(effects_observation_sha256),
            terminal_acknowledgement_sha256: None,
            preserving_classification: None,
        })
    }

    pub fn into_effects(self) -> E {
        self.effects
    }

    pub fn bind_terminal_proofs(
        self,
        accepted: &AcceptedRequest,
        effects_response_bytes: &[u8],
        parity_proof_bytes: &[u8],
        terminal_acknowledgement_bytes: &[u8],
        harness_public_key: &str,
    ) -> Result<FinalizerResponseV2> {
        let binding = match validate_terminal_binding_input(
            accepted,
            effects_response_bytes,
            parity_proof_bytes,
            terminal_acknowledgement_bytes,
            harness_public_key,
        ) {
            Ok(binding) => binding,
            Err(error) => return self.record_terminal_proof_mismatch(accepted, error),
        };
        self.bind_validated_terminal(accepted, &binding)
    }

    fn record_terminal_proof_mismatch(
        self,
        accepted: &AcceptedRequest,
        validation_error: anyhow::Error,
    ) -> Result<FinalizerResponseV2> {
        let journal = LockedJournal::open_fixed(
            self.journal_root,
            &accepted.scope_id,
            self.expected_journal_uid,
        )?;
        let head = journal
            .head()?
            .context("terminal proof arrived before finalizer acceptance")?;
        require_same_claim(&head, accepted)?;
        persist_rejoin_peer_attestation(&journal, &head, accepted)?;
        let effect_plan =
            derive_host_effect_plan_v2(accepted.target_set_kind, &accepted.target_ledger)?;
        validate_journal_effect_plan(&journal, &head, &effect_plan)?;
        match head.record.event.kind {
            JournalEventKind::EffectsComplete => preserving_stop(
                &journal,
                Some(&head),
                accepted,
                PreservingStopClassificationV2::TerminalProofMismatch,
            ),
            JournalEventKind::PreservingStop => replay_preserving_stop(&journal, &head, accepted),
            JournalEventKind::TerminalAcknowledgementBound | JournalEventKind::Complete => {
                Err(validation_error.context(
                    "terminal proof substitution rejected after immutable terminal binding",
                ))
            }
            _ => Err(validation_error.context(
                "terminal proof mismatch occurred before exact EffectsComplete acceptance",
            )),
        }
    }

    pub fn bind_terminal_acknowledgement(
        self,
        _accepted: &AcceptedRequest,
        parity_proof_sha256: &str,
        terminal_acknowledgement_sha256: &str,
    ) -> Result<FinalizerResponseV2> {
        require_digest(parity_proof_sha256)?;
        require_digest(terminal_acknowledgement_sha256)?;
        bail!("digest-only terminal binding is disabled; canonical proof bytes are required")
    }

    fn bind_validated_terminal(
        self,
        accepted: &AcceptedRequest,
        binding: &ValidatedTerminalBinding<'_>,
    ) -> Result<FinalizerResponseV2> {
        let effects = &binding.effects;
        let effects_response_bytes = binding.effects_response_bytes;
        let parity_proof_bytes = binding.parity_proof_bytes;
        let terminal_acknowledgement_bytes = binding.terminal_acknowledgement_bytes;
        let effects_response_sha256 = binding.effects_response_sha256.clone();
        let parity_proof_sha256 = binding.parity_proof_sha256.clone();
        let terminal_acknowledgement_sha256 = binding.terminal_acknowledgement_sha256.clone();

        let journal = LockedJournal::open_fixed(
            self.journal_root,
            &accepted.scope_id,
            self.expected_journal_uid,
        )?;
        let head = journal
            .head()?
            .context("terminal acknowledgement arrived before finalizer acceptance")?;
        require_same_claim(&head, accepted)?;
        persist_rejoin_peer_attestation(&journal, &head, accepted)?;
        let effect_plan =
            derive_host_effect_plan_v2(accepted.target_set_kind, &accepted.target_ledger)?;
        validate_journal_effect_plan(&journal, &head, &effect_plan)?;
        if !matches!(
            head.record.event.kind,
            JournalEventKind::EffectsComplete
                | JournalEventKind::TerminalAcknowledgementBound
                | JournalEventKind::Complete
        ) {
            bail!("terminal acknowledgement arrived before exact effects completion")
        }

        let (effects_head_sha256, effects_observation, bound_record) = match head.record.event.kind
        {
            JournalEventKind::EffectsComplete => (
                head.sha256.clone(),
                head.record
                    .event
                    .observation_sha256
                    .clone()
                    .context("effects-complete journal lacks its observation")?,
                None,
            ),
            JournalEventKind::TerminalAcknowledgementBound => {
                let predecessor = journal.generation(head.generation - 1)?;
                if predecessor.event.kind != JournalEventKind::EffectsComplete {
                    bail!("terminal-bound predecessor is not EffectsComplete")
                }
                (
                    head.record.predecessor_head_sha256.clone(),
                    predecessor
                        .event
                        .observation_sha256
                        .context("terminal-bound predecessor lacks effects observation")?,
                    Some(head.record.clone()),
                )
            }
            JournalEventKind::Complete => {
                let bound = journal.generation(head.generation - 1)?;
                let effects_generation = journal.generation(head.generation - 2)?;
                if bound.event.kind != JournalEventKind::TerminalAcknowledgementBound
                    || effects_generation.event.kind != JournalEventKind::EffectsComplete
                    || head.record.predecessor_head_sha256
                        != sha256_hex_v2(&canonical_bytes_v2(&bound)?)
                {
                    bail!("complete journal does not have the exact terminal predecessor chain")
                }
                (
                    bound.predecessor_head_sha256.clone(),
                    effects_generation
                        .event
                        .observation_sha256
                        .context("complete predecessor lacks effects observation")?,
                    Some(bound),
                )
            }
            _ => unreachable!("closed terminal journal states"),
        };
        if effects.journal_head_sha256 != effects_head_sha256 {
            bail!("terminal effects response differs from the durable EffectsComplete head")
        }

        journal.persist_terminal_binding_artifact(
            "effects.response",
            effects_response_bytes,
            &effects_response_sha256,
        )?;
        journal.persist_terminal_binding_artifact(
            "parity.proof",
            parity_proof_bytes,
            &parity_proof_sha256,
        )?;
        journal.persist_terminal_binding_artifact(
            "terminal.acknowledgement",
            terminal_acknowledgement_bytes,
            &terminal_acknowledgement_sha256,
        )?;

        if let Some(bound) = bound_record.as_ref() {
            if bound.event.observation_sha256.as_deref() != Some(parity_proof_sha256.as_str())
                || bound.event.response_sha256.as_deref()
                    != Some(terminal_acknowledgement_sha256.as_str())
            {
                bail!("terminal acknowledgement replay differs from its durable binding")
            }
        }
        if head.record.event.kind == JournalEventKind::Complete {
            let bytes = journal
                .terminal_response()?
                .context("complete journal lacks immutable terminal response")?;
            let response: FinalizerResponseV2 = parse_canonical_v2(&bytes)?;
            validate_finalizer_response_v2(&response)?;
            if response.request_digest != accepted.request_digest
                || response.state != FinalizerResponseStateV2::HostComplete
                || response.terminal_acknowledgement_sha256.as_deref()
                    != Some(terminal_acknowledgement_sha256.as_str())
                || head.record.event.response_sha256.as_deref()
                    != Some(sha256_hex_v2(&bytes).as_str())
            {
                bail!("completed response differs from exact terminal proof replay")
            }
            return Ok(response);
        }

        let bound = if head.record.event.kind == JournalEventKind::EffectsComplete {
            append_event(
                &journal,
                Some(&head),
                accepted,
                JournalEvent {
                    kind: JournalEventKind::TerminalAcknowledgementBound,
                    host_state: HostRetirementStateV2::TerminalAcknowledgementBound,
                    effect_ordinal: None,
                    effect_role: None,
                    effect_identity_sha256: None,
                    effect_invocation_attempt: None,
                    observation_sha256: Some(parity_proof_sha256.clone()),
                    response_sha256: Some(terminal_acknowledgement_sha256.clone()),
                    preserving_classification: None,
                },
            )?
        } else {
            head
        };
        let response = FinalizerResponseV2 {
            schema_owner: MAC_R3_FINALIZER_PROTOCOL_OWNER_V2.to_string(),
            schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
            request_digest: accepted.request_digest.clone(),
            state: FinalizerResponseStateV2::HostComplete,
            journal_head_sha256: bound.sha256.clone(),
            effects_observation_sha256: Some(effects_observation.clone()),
            terminal_acknowledgement_sha256: Some(terminal_acknowledgement_sha256),
            preserving_classification: None,
        };
        validate_finalizer_response_v2(&response)?;
        let bytes = canonical_bytes_v2(&response)?;
        let response_sha256 =
            journal.persist_terminal_response(&accepted.request_digest, &bytes)?;
        append_event(
            &journal,
            Some(&bound),
            accepted,
            JournalEvent {
                kind: JournalEventKind::Complete,
                host_state: HostRetirementStateV2::Complete,
                effect_ordinal: None,
                effect_role: None,
                effect_identity_sha256: None,
                effect_invocation_attempt: None,
                observation_sha256: Some(effects_observation),
                response_sha256: Some(response_sha256),
                preserving_classification: None,
            },
        )?;
        Ok(response)
    }
}

struct ValidatedTerminalBinding<'a> {
    effects: FinalizerResponseV2,
    effects_response_bytes: &'a [u8],
    parity_proof_bytes: &'a [u8],
    terminal_acknowledgement_bytes: &'a [u8],
    effects_response_sha256: String,
    parity_proof_sha256: String,
    terminal_acknowledgement_sha256: String,
}

fn validate_terminal_binding_input<'a>(
    accepted: &AcceptedRequest,
    effects_response_bytes: &'a [u8],
    parity_proof_bytes: &'a [u8],
    terminal_acknowledgement_bytes: &'a [u8],
    harness_public_key: &str,
) -> Result<ValidatedTerminalBinding<'a>> {
    let effects: FinalizerResponseV2 = parse_canonical_v2(effects_response_bytes)?;
    validate_finalizer_response_v2(&effects)?;
    if effects.request_digest != accepted.request_digest
        || effects.state != FinalizerResponseStateV2::EffectsComplete
    {
        bail!("terminal binding does not carry the accepted EffectsComplete response")
    }
    let parity: HostParityProofV2 = parse_canonical_v2(parity_proof_bytes)?;
    validate_host_parity_proof_v2(&parity, harness_public_key)?;
    if parity.scope_id != accepted.scope_id
        || parity.request_digest != accepted.request_digest
        || parity.effects_response_sha256 != sha256_hex_v2(effects_response_bytes)
        || parity.journal_head_sha256 != effects.journal_head_sha256
    {
        bail!("terminal parity does not exact-bind accepted effects and scope")
    }
    let acknowledgement: TerminalAcknowledgementV2 =
        parse_canonical_v2(terminal_acknowledgement_bytes)?;
    validate_terminal_acknowledgement_v2(&acknowledgement, &parity, harness_public_key)?;
    if acknowledgement.scope_id != accepted.scope_id
        || acknowledgement.request_digest != accepted.request_digest
    {
        bail!("terminal acknowledgement does not bind the accepted scope and request")
    }
    Ok(ValidatedTerminalBinding {
        effects,
        effects_response_bytes,
        parity_proof_bytes,
        terminal_acknowledgement_bytes,
        effects_response_sha256: sha256_hex_v2(effects_response_bytes),
        parity_proof_sha256: sha256_hex_v2(parity_proof_bytes),
        terminal_acknowledgement_sha256: sha256_hex_v2(terminal_acknowledgement_bytes),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EffectRecoveryPhase {
    NotPrepared,
    Prepared,
    Invoked(u16),
    Observed,
}

fn effect_phase(head: &JournalHead, target: &DerivedTarget) -> Result<EffectRecoveryPhase> {
    let event = &head.record.event;
    match (
        event.effect_ordinal,
        event.effect_role,
        event.effect_identity_sha256.as_deref(),
    ) {
        (Some(existing), Some(_), Some(_)) if existing > target.ordinal => {
            Ok(EffectRecoveryPhase::Observed)
        }
        (Some(existing), Some(existing_role), Some(existing_identity))
            if existing == target.ordinal
                && existing_role == target.role
                && existing_identity == target.effect_identity_sha256 =>
        {
            match event.kind {
                JournalEventKind::EffectPrepared => Ok(EffectRecoveryPhase::Prepared),
                JournalEventKind::EffectInvoked => Ok(EffectRecoveryPhase::Invoked(
                    event
                        .effect_invocation_attempt
                        .context("invoked journal head lacks its immutable attempt")?,
                )),
                JournalEventKind::EffectObserved => Ok(EffectRecoveryPhase::Observed),
                _ => bail!("journal effect cursor has a non-effect event"),
            }
        }
        (Some(existing), Some(_), Some(_)) if existing < target.ordinal => {
            Ok(EffectRecoveryPhase::NotPrepared)
        }
        (None, None, None) if matches!(event.kind, JournalEventKind::FinalizerAccepted) => {
            Ok(EffectRecoveryPhase::NotPrepared)
        }
        (None, None, None) if matches!(event.kind, JournalEventKind::EffectsComplete) => {
            Ok(EffectRecoveryPhase::Observed)
        }
        _ => bail!("journal effect cursor is inconsistent with the closed plan"),
    }
}

fn append_effect_event(
    journal: &LockedJournal,
    head: Option<&JournalHead>,
    accepted: &AcceptedRequest,
    kind: JournalEventKind,
    target: &DerivedTarget,
    effect_invocation_attempt: u16,
    observation_sha256: Option<String>,
) -> Result<JournalHead> {
    if let Some(value) = &observation_sha256 {
        require_digest(value)?;
    }
    append_event(
        journal,
        head,
        accepted,
        JournalEvent {
            kind,
            host_state: HostRetirementStateV2::Removing,
            effect_ordinal: Some(target.ordinal),
            effect_role: Some(target.role),
            effect_identity_sha256: Some(target.effect_identity_sha256.clone()),
            effect_invocation_attempt: Some(effect_invocation_attempt),
            observation_sha256,
            response_sha256: None,
            preserving_classification: None,
        },
    )
}

fn append_durable_effect_observed<E: ClosedEffects>(
    effects: &E,
    journal: &LockedJournal,
    head: Option<&JournalHead>,
    accepted: &AcceptedRequest,
    target: &DerivedTarget,
    observation: &EffectObservation,
) -> Result<JournalHead> {
    if observation.class != ObservationClass::ExactFinal {
        bail!("only an exact-final observation may become EffectObserved")
    }
    require_digest(&observation.observation_sha256)?;
    let effect_invocation_attempt = head
        .context("EffectObserved lacks its durable Prepared or Invoked predecessor")?
        .record
        .event
        .effect_invocation_attempt
        .context("EffectObserved predecessor lacks its immutable attempt")?;

    let invocation_evidence = effects.invocation_evidence(target)?;
    let (invocation_evidence_sha256, invocation_evidence_canonical_json) = match invocation_evidence
    {
        Some(bytes) => {
            let _: serde_json::Value = parse_canonical_v2(&bytes)
                .context("validate canonical native invocation evidence")?;
            let digest = sha256_hex_v2(&bytes);
            let json = String::from_utf8(bytes)
                .context("native invocation evidence is not canonical UTF-8 JSON")?;
            (Some(digest), Some(json))
        }
        None => (None, None),
    };
    let proposed = EffectObservationArtifactV2 {
        schema_owner: EFFECT_OBSERVATION_OWNER.to_owned(),
        schema_version: EFFECT_OBSERVATION_VERSION,
        effect_ordinal: target.ordinal,
        effect_role: target.role,
        effect_identity_sha256: target.effect_identity_sha256.clone(),
        effect_invocation_attempt,
        state_observation_sha256: observation.observation_sha256.clone(),
        invocation_evidence_sha256,
        invocation_evidence_canonical_json,
    };
    validate_effect_observation_artifact(&proposed)?;
    let proposed_bytes = canonical_bytes_v2(&proposed)?;

    let artifact_bytes = match journal.effect_observation_artifact(target.ordinal)? {
        Some(existing_bytes) => {
            let existing: EffectObservationArtifactV2 = parse_canonical_v2(&existing_bytes)
                .context("parse durable effect-observation artifact")?;
            validate_effect_observation_artifact(&existing)?;
            if existing.effect_ordinal != proposed.effect_ordinal
                || existing.effect_role != proposed.effect_role
                || existing.effect_identity_sha256 != proposed.effect_identity_sha256
                || existing.effect_invocation_attempt != proposed.effect_invocation_attempt
                || existing.state_observation_sha256 != proposed.state_observation_sha256
            {
                bail!("durable effect-observation artifact differs from the exact live result")
            }
            if proposed.invocation_evidence_sha256.is_some()
                && (existing.invocation_evidence_sha256 != proposed.invocation_evidence_sha256
                    || existing.invocation_evidence_canonical_json
                        != proposed.invocation_evidence_canonical_json)
            {
                bail!("durable native invocation evidence differs from the same-process receipt")
            }
            existing_bytes
        }
        None => {
            let digest = sha256_hex_v2(&proposed_bytes);
            journal.persist_effect_observation_artifact(
                target.ordinal,
                &proposed_bytes,
                &digest,
            )?;
            proposed_bytes
        }
    };
    let artifact_sha256 = sha256_hex_v2(&artifact_bytes);
    append_effect_event(
        journal,
        head,
        accepted,
        JournalEventKind::EffectObserved,
        target,
        effect_invocation_attempt,
        Some(artifact_sha256),
    )
}

fn validate_effect_observation_artifact(value: &EffectObservationArtifactV2) -> Result<()> {
    if value.schema_owner != EFFECT_OBSERVATION_OWNER
        || value.schema_version != EFFECT_OBSERVATION_VERSION
        || value.effect_ordinal == 0
        || value.effect_invocation_attempt > MAX_EFFECT_INVOCATION_ATTEMPTS_V2
        || value.invocation_evidence_sha256.is_some()
            != value.invocation_evidence_canonical_json.is_some()
        || ((matches!(
            value.effect_role,
            HostTargetRoleV2::Signer
                | HostTargetRoleV2::CapabilitySignControl
                | HostTargetRoleV2::CapabilityExportControl
                | HostTargetRoleV2::CapabilityAclMutationControl
                | HostTargetRoleV2::CapabilityWrongKeyDeleteControl
        ) || (value.effect_invocation_attempt > 0
            && !matches!(
                value.effect_role,
                HostTargetRoleV2::PublisherService | HostTargetRoleV2::PublisherEndpoint
            )))
            && value.invocation_evidence_sha256.is_none())
    {
        bail!("effect-observation artifact contract is invalid")
    }
    require_digest(&value.effect_identity_sha256)?;
    require_digest(&value.state_observation_sha256)?;
    if let (Some(expected), Some(json)) = (
        value.invocation_evidence_sha256.as_deref(),
        value.invocation_evidence_canonical_json.as_deref(),
    ) {
        require_digest(expected)?;
        if sha256_hex_v2(json.as_bytes()) != expected {
            bail!("effect-observation native evidence digest is invalid")
        }
        let _: serde_json::Value = parse_canonical_v2(json.as_bytes())
            .context("effect-observation native evidence is not canonical JSON")?;
    }
    Ok(())
}

fn append_event(
    journal: &LockedJournal,
    head: Option<&JournalHead>,
    accepted: &AcceptedRequest,
    event: JournalEvent,
) -> Result<JournalHead> {
    if head.is_none() {
        if event.kind != JournalEventKind::FinalizerAccepted {
            bail!("first journal generation is not FinalizerAccepted")
        }
        journal.persist_acceptance_artifact(
            "authority.request",
            &accepted.canonical_request,
            &sha256_hex_v2(&accepted.canonical_request),
        )?;
        journal.persist_acceptance_artifact(
            "peer.attestation",
            &accepted.peer_attestation,
            &accepted.peer_attestation_sha256,
        )?;
    }
    journal.append(
        head,
        &JournalClaimIdentity {
            request_digest: accepted.request_digest.clone(),
            authority_bytes_sha256: sha256_hex_v2(&accepted.canonical_request),
            successor_capsule_sha256: accepted.successor_capsule_sha256.clone(),
            accepted_peer_attestation_sha256: head.map_or_else(
                || accepted.peer_attestation_sha256.clone(),
                |head| head.record.accepted_peer_attestation_sha256.clone(),
            ),
            accepted_peer_identity_sha256: accepted.accepted_peer_identity_sha256.clone(),
        },
        event,
    )
}

fn persist_rejoin_peer_attestation(
    journal: &LockedJournal,
    head: &JournalHead,
    accepted: &AcceptedRequest,
) -> Result<()> {
    journal.persist_rejoin_peer_attestation(
        &accepted.peer_attestation,
        &accepted.peer_attestation_sha256,
        &head.record.accepted_peer_attestation_sha256,
    )
}

fn require_same_claim(head: &JournalHead, accepted: &AcceptedRequest) -> Result<()> {
    if head.record.request_digest != accepted.request_digest
        || head.record.authority_bytes_sha256 != sha256_hex_v2(&accepted.canonical_request)
        || head.record.successor_capsule_sha256 != accepted.successor_capsule_sha256
        || head.record.accepted_peer_identity_sha256 != accepted.accepted_peer_identity_sha256
    {
        bail!("alternate digest or successor identity is rejected after acceptance")
    }
    Ok(())
}

fn immutable_replay_response(
    journal: &LockedJournal,
    head: &JournalHead,
    accepted: &AcceptedRequest,
) -> Result<Option<FinalizerResponseV2>> {
    match head.record.event.kind {
        JournalEventKind::Complete => {
            let bytes = journal
                .terminal_response()?
                .context("complete journal lacks immutable terminal response")?;
            let response: FinalizerResponseV2 = parse_canonical_v2(&bytes)?;
            validate_finalizer_response_v2(&response)?;
            let bound = journal.generation(head.generation - 1)?;
            let effects_generation = journal.generation(head.generation - 2)?;
            let stored_effects = journal
                .terminal_binding_artifact("effects.response")?
                .context("complete journal lacks immutable EffectsComplete response")?;
            let stored_effects: FinalizerResponseV2 = parse_canonical_v2(&stored_effects)?;
            let stored_parity = journal
                .terminal_binding_artifact("parity.proof")?
                .context("complete journal lacks immutable parity proof")?;
            let stored_acknowledgement = journal
                .terminal_binding_artifact("terminal.acknowledgement")?
                .context("complete journal lacks immutable terminal acknowledgement")?;
            if response.request_digest != accepted.request_digest
                || response.state != FinalizerResponseStateV2::HostComplete
                || bound.event.kind != JournalEventKind::TerminalAcknowledgementBound
                || effects_generation.event.kind != JournalEventKind::EffectsComplete
                || response.journal_head_sha256 != head.record.predecessor_head_sha256
                || response.terminal_acknowledgement_sha256.as_deref()
                    != bound.event.response_sha256.as_deref()
                || response.effects_observation_sha256
                    != effects_generation.event.observation_sha256
                || head.record.event.observation_sha256
                    != effects_generation.event.observation_sha256
                || stored_effects.request_digest != accepted.request_digest
                || stored_effects.state != FinalizerResponseStateV2::EffectsComplete
                || stored_effects.journal_head_sha256 != bound.predecessor_head_sha256
                || bound.event.observation_sha256.as_deref()
                    != Some(sha256_hex_v2(&stored_parity).as_str())
                || bound.event.response_sha256.as_deref()
                    != Some(sha256_hex_v2(&stored_acknowledgement).as_str())
                || head.record.event.response_sha256.as_deref()
                    != Some(sha256_hex_v2(&bytes).as_str())
            {
                bail!("stored terminal response does not bind the completed request")
            }
            Ok(Some(response))
        }
        JournalEventKind::PreservingStop => {
            replay_preserving_stop(journal, head, accepted).map(Some)
        }
        JournalEventKind::EffectsComplete => {
            let observation = head
                .record
                .event
                .observation_sha256
                .clone()
                .context("effects-complete journal lacks its exact observation")?;
            Ok(Some(FinalizerResponseV2 {
                schema_owner: MAC_R3_FINALIZER_PROTOCOL_OWNER_V2.to_string(),
                schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
                request_digest: accepted.request_digest.clone(),
                state: FinalizerResponseStateV2::EffectsComplete,
                journal_head_sha256: head.sha256.clone(),
                effects_observation_sha256: Some(observation),
                terminal_acknowledgement_sha256: None,
                preserving_classification: None,
            }))
        }
        _ => Ok(None),
    }
}

fn validate_journal_effect_plan(
    journal: &LockedJournal,
    head: &JournalHead,
    effect_plan: &[HostEffectPlanEntryV2],
) -> Result<()> {
    let mut previous_event: Option<JournalEvent> = None;
    for generation in 1..=head.generation {
        let record = if generation == head.generation {
            head.record.clone()
        } else {
            journal.generation(generation)?
        };
        if matches!(
            record.event.kind,
            JournalEventKind::EffectPrepared
                | JournalEventKind::EffectInvoked
                | JournalEventKind::EffectObserved
        ) {
            let ordinal = record
                .event
                .effect_ordinal
                .context("effect journal generation lacks its ordinal")?;
            let planned = effect_plan
                .get(usize::from(ordinal) - 1)
                .context("effect journal ordinal is outside the signed plan")?;
            if record.event.effect_role != Some(planned.role)
                || record.event.effect_identity_sha256.as_deref()
                    != Some(planned.target_identity_sha256.as_str())
            {
                bail!("effect journal generation differs from its signed plan identity")
            }
            if record.event.kind == JournalEventKind::EffectObserved {
                let artifact_bytes = journal
                    .effect_observation_artifact(ordinal)?
                    .context("EffectObserved lacks its immutable observation artifact")?;
                let artifact: EffectObservationArtifactV2 = parse_canonical_v2(&artifact_bytes)
                    .context("parse EffectObserved immutable artifact")?;
                validate_effect_observation_artifact(&artifact)?;
                if record.event.observation_sha256.as_deref()
                    != Some(sha256_hex_v2(&artifact_bytes).as_str())
                    || artifact.effect_ordinal != ordinal
                    || artifact.effect_role != planned.role
                    || artifact.effect_identity_sha256 != planned.target_identity_sha256
                    || Some(artifact.effect_invocation_attempt)
                        != record.event.effect_invocation_attempt
                {
                    bail!("EffectObserved artifact differs from its signed plan or journal digest")
                }
            }
        }
        if record.event.kind == JournalEventKind::EffectsComplete {
            let final_effect = effect_plan.last().context("signed effect plan is empty")?;
            let previous = previous_event
                .as_ref()
                .context("EffectsComplete lacks its final observed predecessor")?;
            if previous.kind != JournalEventKind::EffectObserved
                || previous.effect_ordinal != Some(final_effect.ordinal)
                || previous.effect_role != Some(final_effect.role)
                || previous.effect_identity_sha256.as_deref()
                    != Some(final_effect.target_identity_sha256.as_str())
            {
                bail!("EffectsComplete was not reached after the exact final signed effect")
            }
        }
        previous_event = Some(record.event);
    }
    Ok(())
}

fn validated_recovery_cursor(
    head: &JournalHead,
    effect_plan: &[HostEffectPlanEntryV2],
) -> Result<ValidatedRecoveryCursor> {
    let effect_count = u16::try_from(effect_plan.len()).context("effect plan exceeds u16")?;
    let (completed_effect_ordinal, pending_ordinal, phase, effect_invocation_attempt) =
        match head.record.event.kind {
            JournalEventKind::FinalizerAccepted => (0, 1, RecoveryEffectPhase::NotPrepared, 0),
            JournalEventKind::EffectPrepared => {
                let ordinal = head
                    .record
                    .event
                    .effect_ordinal
                    .context("prepared recovery head lacks its ordinal")?;
                (
                    ordinal.checked_sub(1).context("prepared ordinal is zero")?,
                    ordinal,
                    RecoveryEffectPhase::Prepared,
                    0,
                )
            }
            JournalEventKind::EffectInvoked => {
                let ordinal = head
                    .record
                    .event
                    .effect_ordinal
                    .context("invoked recovery head lacks its ordinal")?;
                (
                    ordinal.checked_sub(1).context("invoked ordinal is zero")?,
                    ordinal,
                    RecoveryEffectPhase::Invoked,
                    head.record
                        .event
                        .effect_invocation_attempt
                        .context("invoked recovery head lacks its immutable attempt")?,
                )
            }
            JournalEventKind::EffectObserved => {
                let ordinal = head
                    .record
                    .event
                    .effect_ordinal
                    .context("observed recovery head lacks its ordinal")?;
                (
                    ordinal,
                    ordinal
                        .checked_add(1)
                        .context("observed recovery ordinal overflow")?,
                    RecoveryEffectPhase::NotPrepared,
                    0,
                )
            }
            _ => bail!("immutable journal state does not require native effect recovery"),
        };
    let pending = if pending_ordinal <= effect_count {
        let planned = &effect_plan[usize::from(pending_ordinal) - 1];
        Some(RecoveryEffectCursor {
            ordinal: planned.ordinal,
            role: planned.role,
            effect_identity_sha256: planned.target_identity_sha256.clone(),
            phase,
            effect_invocation_attempt,
        })
    } else {
        None
    };
    Ok(ValidatedRecoveryCursor {
        journal_generation: head.generation,
        journal_head_sha256: head.sha256.clone(),
        completed_effect_ordinal,
        effect_count,
        pending,
    })
}

fn preserving_stop(
    journal: &LockedJournal,
    head: Option<&JournalHead>,
    accepted: &AcceptedRequest,
    classification: PreservingStopClassificationV2,
) -> Result<FinalizerResponseV2> {
    let host_state = head
        .map(|value| value.record.event.host_state)
        .unwrap_or(HostRetirementStateV2::Removing);
    let next = append_event(
        journal,
        head,
        accepted,
        JournalEvent {
            kind: JournalEventKind::PreservingStop,
            host_state,
            effect_ordinal: None,
            effect_role: None,
            effect_identity_sha256: None,
            effect_invocation_attempt: None,
            observation_sha256: None,
            response_sha256: None,
            preserving_classification: Some(
                preserving_classification_literal(classification).to_string(),
            ),
        },
    )?;
    let response = FinalizerResponseV2 {
        schema_owner: MAC_R3_FINALIZER_PROTOCOL_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        request_digest: accepted.request_digest.clone(),
        state: FinalizerResponseStateV2::PreservingStop,
        journal_head_sha256: next.sha256,
        effects_observation_sha256: None,
        terminal_acknowledgement_sha256: None,
        preserving_classification: Some(classification),
    };
    validate_finalizer_response_v2(&response)?;
    let bytes = canonical_bytes_v2(&response)?;
    journal.persist_preserving_response(&accepted.request_digest, &bytes)?;
    Ok(response)
}

fn replay_preserving_stop(
    journal: &LockedJournal,
    head: &JournalHead,
    accepted: &AcceptedRequest,
) -> Result<FinalizerResponseV2> {
    let classification = parse_preserving_classification(
        head.record
            .event
            .preserving_classification
            .as_deref()
            .context("preserving-stop journal lacks its classification")?,
    )?;
    let response = FinalizerResponseV2 {
        schema_owner: MAC_R3_FINALIZER_PROTOCOL_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        request_digest: accepted.request_digest.clone(),
        state: FinalizerResponseStateV2::PreservingStop,
        journal_head_sha256: head.sha256.clone(),
        effects_observation_sha256: None,
        terminal_acknowledgement_sha256: None,
        preserving_classification: Some(classification),
    };
    validate_finalizer_response_v2(&response)?;
    let expected = canonical_bytes_v2(&response)?;
    match journal.preserving_response()? {
        Some(existing) if existing != expected => {
            bail!("immutable preserving response differs from its journal head")
        }
        Some(_) => {}
        None => {
            journal.persist_preserving_response(&accepted.request_digest, &expected)?;
        }
    }
    Ok(response)
}

fn preserving_classification_literal(
    classification: PreservingStopClassificationV2,
) -> &'static str {
    match classification {
        PreservingStopClassificationV2::IdentityOrAuthorityMismatch => {
            "identity_or_authority_mismatch"
        }
        PreservingStopClassificationV2::InteractionRequired => "interaction_required",
        PreservingStopClassificationV2::AmbiguousEffectState => "ambiguous_effect_state",
        PreservingStopClassificationV2::TerminalProofMismatch => "terminal_proof_mismatch",
    }
}

fn parse_preserving_classification(value: &str) -> Result<PreservingStopClassificationV2> {
    match value {
        "identity_or_authority_mismatch" => {
            Ok(PreservingStopClassificationV2::IdentityOrAuthorityMismatch)
        }
        "interaction_required" => Ok(PreservingStopClassificationV2::InteractionRequired),
        "ambiguous_effect_state" => Ok(PreservingStopClassificationV2::AmbiguousEffectState),
        "terminal_proof_mismatch" => Ok(PreservingStopClassificationV2::TerminalProofMismatch),
        _ => bail!("preserving-stop classification is not closed"),
    }
}

fn require_digest(value: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("engine authority join must be lowercase SHA-256")
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PeerAdmissionMode {
    Initial,
    Rejoin,
}

#[derive(Serialize)]
struct StablePeerIdentityV2<'a> {
    schema_owner: &'static str,
    schema_version: u32,
    effective_uid: u32,
    effective_gid: u32,
    canonical_account: &'a str,
    executable_path: &'a str,
    executable_physical_identity_sha256: &'a str,
    executable_sha256: &'a str,
    cdhash: &'a str,
}

fn stable_peer_identity_sha256(peer: &PeerAttestation) -> Result<String> {
    Ok(sha256_hex_v2(&canonical_bytes_v2(&StablePeerIdentityV2 {
        schema_owner: "substrate.r3-macos-finalizer-stable-peer-identity",
        schema_version: 2,
        effective_uid: peer.effective_uid,
        effective_gid: peer.effective_gid,
        canonical_account: &peer.canonical_account,
        executable_path: &peer.executable_path,
        executable_physical_identity_sha256: &peer.executable_physical_identity_sha256,
        executable_sha256: &peer.executable_sha256,
        cdhash: &peer.cdhash,
    })?))
}

fn validate_peer_attestation(
    peer: &PeerAttestation,
    receipt: &PublisherPreRemovalReceiptV2,
    mode: PeerAdmissionMode,
) -> Result<()> {
    for digest in [
        &peer.audit_token_sha256,
        &peer.process_start_sha256,
        &peer.executable_physical_identity_sha256,
        &peer.executable_sha256,
    ] {
        require_digest(digest)?;
    }
    if peer.schema_owner != "substrate.r3-macos-finalizer-peer-attestation"
        || peer.schema_version != 1
        || peer.pid <= 0
        || peer.effective_uid != receipt.coordinator_process.effective_uid
        || peer.effective_gid != receipt.coordinator_process.effective_gid
        || peer.canonical_account != receipt.coordinator_process.canonical_account
        || peer.executable_path != receipt.coordinator_identity.intended_path
        || peer.executable_physical_identity_sha256
            != receipt.coordinator_identity.physical_identity_sha256
        || peer.executable_sha256 != receipt.coordinator_identity.executable_sha256
        || peer.cdhash != receipt.coordinator_identity.cdhash
    {
        bail!("peer attestation does not exact-bind the signed coordinator process")
    }
    if mode == PeerAdmissionMode::Initial
        && peer.process_start_sha256 != receipt.coordinator_process.process_start_identity_sha256
    {
        bail!("first-acceptance peer attestation does not bind the frozen process start")
    }
    Ok(())
}

#[cfg(test)]
fn require_scope_uuid_v7(value: &str) -> Result<()> {
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
        bail!("accepted request scope must be an exact UUIDv7")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use substrate_common::macos_retirement_v2::{
        host_target_role_for_locator_v2, DisposableCapabilityControlV2, HostResourceLocatorV2,
        HostTargetRoleV2,
    };

    struct MemoryEffects {
        final_state: BTreeMap<HostTargetRoleV2, bool>,
        invokes: Vec<HostTargetRoleV2>,
        observations: usize,
        invocation_evidence: Option<Vec<u8>>,
    }

    impl Default for MemoryEffects {
        fn default() -> Self {
            Self {
                final_state: BTreeMap::new(),
                invokes: Vec::new(),
                observations: 0,
                // The general engine fixture models a successful canonical native invocation.
                // Tests for missing evidence construct the artifact/fixture explicitly.
                invocation_evidence: Some(b"{}".to_vec()),
            }
        }
    }

    impl ClosedEffects for MemoryEffects {
        fn observe(
            &mut self,
            target: &DerivedTarget,
            expected_before_sha256: &str,
            _phase: EffectObservationPhase,
        ) -> Result<EffectObservation> {
            self.observations += 1;
            Ok(EffectObservation {
                class: if self.final_state.get(&target.role).copied().unwrap_or(false) {
                    ObservationClass::ExactFinal
                } else {
                    ObservationClass::ExactBefore
                },
                observation_sha256: expected_before_sha256.to_string(),
            })
        }

        fn invoke(&mut self, target: &DerivedTarget, _expected_before_sha256: &str) -> Result<()> {
            self.invokes.push(target.role);
            self.final_state.insert(target.role, true);
            Ok(())
        }

        fn invocation_evidence(&self, _target: &DerivedTarget) -> Result<Option<Vec<u8>>> {
            Ok(self.invocation_evidence.clone())
        }
    }

    struct AmbiguousEffects;

    impl ClosedEffects for AmbiguousEffects {
        fn observe(
            &mut self,
            _target: &DerivedTarget,
            expected_before_sha256: &str,
            _phase: EffectObservationPhase,
        ) -> Result<EffectObservation> {
            Ok(EffectObservation {
                class: ObservationClass::Ambiguous,
                observation_sha256: expected_before_sha256.to_string(),
            })
        }

        fn invoke(&mut self, _target: &DerivedTarget, _expected_before_sha256: &str) -> Result<()> {
            panic!("ambiguous target must never be invoked")
        }
    }

    struct PanicEffects;

    impl ClosedEffects for PanicEffects {
        fn observe(
            &mut self,
            _target: &DerivedTarget,
            _expected_before_sha256: &str,
            _phase: EffectObservationPhase,
        ) -> Result<EffectObservation> {
            panic!("immutable replay must not inspect a target")
        }

        fn invoke(&mut self, _target: &DerivedTarget, _expected_before_sha256: &str) -> Result<()> {
            panic!("immutable replay must not invoke a target")
        }
    }

    #[derive(Default)]
    struct InvokeErrorEffects {
        observations: usize,
        invocations: usize,
    }

    impl ClosedEffects for InvokeErrorEffects {
        fn observe(
            &mut self,
            _target: &DerivedTarget,
            expected_before_sha256: &str,
            _phase: EffectObservationPhase,
        ) -> Result<EffectObservation> {
            self.observations += 1;
            Ok(EffectObservation {
                class: ObservationClass::ExactBefore,
                observation_sha256: expected_before_sha256.to_string(),
            })
        }

        fn invoke(&mut self, _target: &DerivedTarget, _expected_before_sha256: &str) -> Result<()> {
            self.invocations += 1;
            bail!("injected native invocation failure")
        }
    }

    struct ExactBeforePanicInvokeEffects;

    impl ClosedEffects for ExactBeforePanicInvokeEffects {
        fn observe(
            &mut self,
            _target: &DerivedTarget,
            expected_before_sha256: &str,
            _phase: EffectObservationPhase,
        ) -> Result<EffectObservation> {
            Ok(EffectObservation {
                class: ObservationClass::ExactBefore,
                observation_sha256: expected_before_sha256.to_string(),
            })
        }

        fn invoke(&mut self, _target: &DerivedTarget, _expected_before_sha256: &str) -> Result<()> {
            panic!("exhausted invocation budget must stop before another system call")
        }
    }

    fn accepted() -> AcceptedRequest {
        use DisposableCapabilityControlV2 as C;
        use HostResourceLocatorV2 as L;
        let locators = [
            L::DisposableCapabilityControl { control: C::Sign },
            L::DisposableCapabilityControl {
                control: C::ExportPrivate,
            },
            L::DisposableCapabilityControl {
                control: C::ReplaceAccess,
            },
            L::DisposableCapabilityControl {
                control: C::DeleteWrongKey,
            },
            L::DisposableProtectedWrapper,
            L::SigningKey,
            L::DisposableCurrentLock,
            L::RetirementTerminalLatch,
        ];
        let target_ledger = locators
            .into_iter()
            .enumerate()
            .map(|(index, locator)| HostTargetIdentityV2 {
                ordinal: u16::try_from(index + 1).unwrap(),
                role: host_target_role_for_locator_v2(&locator),
                locator,
                expected_before_sha256: format!("{:02x}", index + 4).repeat(32),
            })
            .collect();
        AcceptedRequest::from_test_parts(
            "019fffe0-0000-7000-8000-000000000001".to_string(),
            "11".repeat(32),
            b"canonical-authority".to_vec(),
            "22".repeat(32),
            b"peer-attestation".to_vec(),
            sha256_hex_v2(b"peer-attestation"),
            TargetSetKindV2::DisposableCapability,
            target_ledger,
        )
        .unwrap()
    }

    fn test_terminal_binding<'a>(
        effects: FinalizerResponseV2,
        effects_bytes: &'a [u8],
        parity_bytes: &'a [u8],
        acknowledgement_bytes: &'a [u8],
    ) -> ValidatedTerminalBinding<'a> {
        ValidatedTerminalBinding {
            effects,
            effects_response_bytes: effects_bytes,
            parity_proof_bytes: parity_bytes,
            terminal_acknowledgement_bytes: acknowledgement_bytes,
            effects_response_sha256: sha256_hex_v2(effects_bytes),
            parity_proof_sha256: sha256_hex_v2(parity_bytes),
            terminal_acknowledgement_sha256: sha256_hex_v2(acknowledgement_bytes),
        }
    }

    fn append_first_effect_attempts(
        root: &Path,
        uid: u32,
        request: &AcceptedRequest,
        attempts: u16,
    ) -> JournalHead {
        assert!(attempts <= MAX_EFFECT_INVOCATION_ATTEMPTS_V2);
        let plan =
            derive_host_effect_plan_v2(request.target_set_kind, &request.target_ledger).unwrap();
        let first = &plan[0];
        let journal = LockedJournal::open_fixed(root, &request.scope_id, uid).unwrap();
        let accepted_head = append_event(
            &journal,
            None,
            request,
            JournalEvent {
                kind: JournalEventKind::FinalizerAccepted,
                host_state: HostRetirementStateV2::FinalizerAccepted,
                effect_ordinal: None,
                effect_role: None,
                effect_identity_sha256: None,
                effect_invocation_attempt: None,
                observation_sha256: None,
                response_sha256: None,
                preserving_classification: None,
            },
        )
        .unwrap();
        let mut head = append_event(
            &journal,
            Some(&accepted_head),
            request,
            JournalEvent {
                kind: JournalEventKind::EffectPrepared,
                host_state: HostRetirementStateV2::Removing,
                effect_ordinal: Some(first.ordinal),
                effect_role: Some(first.role),
                effect_identity_sha256: Some(first.target_identity_sha256.clone()),
                effect_invocation_attempt: Some(0),
                observation_sha256: None,
                response_sha256: None,
                preserving_classification: None,
            },
        )
        .unwrap();
        for attempt in 1..=attempts {
            head = append_event(
                &journal,
                Some(&head),
                request,
                JournalEvent {
                    kind: JournalEventKind::EffectInvoked,
                    host_state: HostRetirementStateV2::Removing,
                    effect_ordinal: Some(first.ordinal),
                    effect_role: Some(first.role),
                    effect_identity_sha256: Some(first.target_identity_sha256.clone()),
                    effect_invocation_attempt: Some(attempt),
                    observation_sha256: None,
                    response_sha256: None,
                    preserving_classification: None,
                },
            )
            .unwrap();
        }
        head
    }

    #[test]
    fn invoked_recovery_persists_the_bounded_retry_before_reinvocation() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let request = accepted();
        let invoked = append_first_effect_attempts(&root, uid, &request, 1);
        let cursor = validated_recovery_cursor(
            &invoked,
            &derive_host_effect_plan_v2(request.target_set_kind, &request.target_ledger).unwrap(),
        )
        .unwrap();
        assert_eq!(cursor.pending().unwrap().effect_invocation_attempt(), 1);

        let response = FinalizerEngine::new(&root, uid, MemoryEffects::default())
            .execute(&request)
            .unwrap();
        assert_eq!(response.state, FinalizerResponseStateV2::EffectsComplete);

        let journal = LockedJournal::open_fixed(&root, &request.scope_id, uid).unwrap();
        let head = journal.head().unwrap().unwrap();
        let attempts = (1..=head.generation)
            .map(|generation| journal.generation(generation).unwrap().event)
            .filter(|event| {
                event.kind == JournalEventKind::EffectInvoked && event.effect_ordinal == Some(1)
            })
            .map(|event| event.effect_invocation_attempt.unwrap())
            .collect::<Vec<_>>();
        assert_eq!(attempts, [1, 2]);
        let observed = (1..=head.generation)
            .map(|generation| journal.generation(generation).unwrap().event)
            .find(|event| {
                event.kind == JournalEventKind::EffectObserved && event.effect_ordinal == Some(1)
            })
            .unwrap();
        assert_eq!(observed.effect_invocation_attempt, Some(2));
    }

    #[test]
    fn exhausted_invocation_budget_stops_before_journal_growth_or_another_call() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let request = accepted();
        let invoked =
            append_first_effect_attempts(&root, uid, &request, MAX_EFFECT_INVOCATION_ATTEMPTS_V2);
        assert_eq!(invoked.generation, 4);

        let stopped = FinalizerEngine::new(&root, uid, ExactBeforePanicInvokeEffects)
            .execute(&request)
            .unwrap();
        assert_eq!(stopped.state, FinalizerResponseStateV2::PreservingStop);
        assert_eq!(
            stopped.preserving_classification,
            Some(PreservingStopClassificationV2::AmbiguousEffectState)
        );
        let journal = LockedJournal::open_fixed(&root, &request.scope_id, uid).unwrap();
        let head = journal.head().unwrap().unwrap();
        assert_eq!(head.generation, 5);
        assert_eq!(head.record.event.kind, JournalEventKind::PreservingStop);
        drop(journal);

        let replay = FinalizerEngine::new(&root, uid, PanicEffects)
            .execute(&request)
            .unwrap();
        assert_eq!(replay, stopped);
    }

    #[test]
    fn acceptance_precedes_every_effect_and_replay_inspects_nothing() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let response = FinalizerEngine::new(&root, uid, MemoryEffects::default())
            .execute(&accepted())
            .unwrap();
        assert_eq!(response.state, FinalizerResponseStateV2::EffectsComplete);

        let mut already_final = MemoryEffects::default();
        for identity in &accepted().target_ledger {
            already_final.final_state.insert(identity.role, true);
        }
        let replay = FinalizerEngine::new(&root, uid, already_final)
            .execute(&accepted())
            .unwrap();
        assert_eq!(replay.state, FinalizerResponseStateV2::EffectsComplete);
    }

    #[test]
    fn effect_observed_binds_exact_canonical_native_status_evidence() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let native_evidence = canonical_bytes_v2(&serde_json::json!({
            "classification": "interaction_not_allowed_and_preserved",
            "raw_os_status": -25308
        }))
        .unwrap();
        let effects = MemoryEffects {
            invocation_evidence: Some(native_evidence.clone()),
            ..MemoryEffects::default()
        };
        let request = accepted();
        let response = FinalizerEngine::new(&root, uid, effects)
            .execute(&request)
            .unwrap();
        assert_eq!(response.state, FinalizerResponseStateV2::EffectsComplete);

        let journal = LockedJournal::open_fixed(&root, &request.scope_id, uid).unwrap();
        let bytes = journal.effect_observation_artifact(1).unwrap().unwrap();
        let artifact: EffectObservationArtifactV2 = parse_canonical_v2(&bytes).unwrap();
        assert_eq!(artifact.effect_invocation_attempt, 1);
        assert_eq!(
            artifact.invocation_evidence_sha256,
            Some(sha256_hex_v2(&native_evidence))
        );
        assert_eq!(
            artifact.invocation_evidence_canonical_json.as_deref(),
            Some(std::str::from_utf8(&native_evidence).unwrap())
        );
        assert!(journal
            .persist_effect_observation_artifact(1, b"{}", &sha256_hex_v2(b"{}"))
            .is_err());
    }

    #[test]
    fn signer_and_capability_effects_require_canonical_invocation_evidence() {
        for role in [
            HostTargetRoleV2::Signer,
            HostTargetRoleV2::CapabilitySignControl,
            HostTargetRoleV2::CapabilityExportControl,
            HostTargetRoleV2::CapabilityAclMutationControl,
            HostTargetRoleV2::CapabilityWrongKeyDeleteControl,
        ] {
            let mut artifact = EffectObservationArtifactV2 {
                schema_owner: EFFECT_OBSERVATION_OWNER.to_owned(),
                schema_version: EFFECT_OBSERVATION_VERSION,
                effect_ordinal: 1,
                effect_role: role,
                effect_identity_sha256: "a".repeat(64),
                effect_invocation_attempt: 1,
                state_observation_sha256: "b".repeat(64),
                invocation_evidence_sha256: None,
                invocation_evidence_canonical_json: None,
            };
            assert!(validate_effect_observation_artifact(&artifact).is_err());
            let evidence = "{}".to_owned();
            artifact.invocation_evidence_sha256 = Some(sha256_hex_v2(evidence.as_bytes()));
            artifact.invocation_evidence_canonical_json = Some(evidence);
            validate_effect_observation_artifact(&artifact).unwrap();
        }
    }

    #[test]
    fn invoked_filesystem_effect_requires_canonical_durability_evidence() {
        let mut artifact = EffectObservationArtifactV2 {
            schema_owner: EFFECT_OBSERVATION_OWNER.to_owned(),
            schema_version: EFFECT_OBSERVATION_VERSION,
            effect_ordinal: 1,
            effect_role: HostTargetRoleV2::ProtectedWrapper,
            effect_identity_sha256: "a".repeat(64),
            effect_invocation_attempt: 1,
            state_observation_sha256: "b".repeat(64),
            invocation_evidence_sha256: None,
            invocation_evidence_canonical_json: None,
        };
        assert!(validate_effect_observation_artifact(&artifact).is_err());
        artifact.effect_invocation_attempt = 0;
        validate_effect_observation_artifact(&artifact).unwrap();
    }

    #[test]
    fn native_error_after_acceptance_returns_and_replays_immutable_preserving_response() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let response = FinalizerEngine::new(&root, uid, InvokeErrorEffects::default())
            .execute(&accepted())
            .unwrap();
        assert_eq!(response.state, FinalizerResponseStateV2::PreservingStop);
        assert_eq!(
            response.preserving_classification,
            Some(PreservingStopClassificationV2::AmbiguousEffectState)
        );

        let replay = FinalizerEngine::new(&root, uid, PanicEffects)
            .execute(&accepted())
            .unwrap();
        assert_eq!(replay, response);
    }

    #[test]
    fn alternate_digest_is_rejected_after_acceptance() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        FinalizerEngine::new(&root, uid, MemoryEffects::default())
            .execute(&accepted())
            .unwrap();
        let mut alternate = accepted();
        alternate.request_digest = "44".repeat(32);
        assert!(FinalizerEngine::new(&root, uid, MemoryEffects::default())
            .execute(&alternate)
            .is_err());
    }

    #[test]
    fn unverified_time_cannot_create_a_first_claim() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let mut request = accepted();
        request.first_acceptance_time_verified = false;
        assert!(FinalizerEngine::new(&root, uid, PanicEffects)
            .execute(&request)
            .is_err());
        assert!(!root.exists());
    }

    #[test]
    fn terminal_binding_is_separate_and_complete_replay_is_immutable() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let effects = FinalizerEngine::new(&root, uid, MemoryEffects::default())
            .execute(&accepted())
            .unwrap();
        let effects_bytes = canonical_bytes_v2(&effects).unwrap();
        let binding = test_terminal_binding(
            effects,
            &effects_bytes,
            b"canonical-parity-proof",
            b"canonical-terminal-acknowledgement",
        );
        let complete = FinalizerEngine::new(&root, uid, MemoryEffects::default())
            .bind_validated_terminal(&accepted(), &binding)
            .unwrap();
        assert_eq!(complete.state, FinalizerResponseStateV2::HostComplete);
        let replay = FinalizerEngine::new(&root, uid, MemoryEffects::default())
            .execute(&accepted())
            .unwrap();
        assert_eq!(replay, complete);
    }

    #[test]
    fn preserving_stop_replay_is_immutable_and_reconstructs_after_response_write_crash() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let stopped = FinalizerEngine::new(&root, uid, AmbiguousEffects)
            .execute(&accepted())
            .unwrap();
        assert_eq!(stopped.state, FinalizerResponseStateV2::PreservingStop);

        std::fs::remove_file(root.join(&accepted().scope_id).join("preserving.response")).unwrap();
        let replay = FinalizerEngine::new(&root, uid, PanicEffects)
            .execute(&accepted())
            .unwrap();
        assert_eq!(replay, stopped);
        assert!(root
            .join(&accepted().scope_id)
            .join("preserving.response")
            .is_file());
    }

    #[test]
    fn terminal_bound_crash_rejects_acknowledgement_substitution() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let effects = FinalizerEngine::new(&root, uid, MemoryEffects::default())
            .execute(&accepted())
            .unwrap();
        let effects_bytes = canonical_bytes_v2(&effects).unwrap();
        let parity_bytes = b"canonical-parity-proof";
        let acknowledgement_bytes = b"canonical-terminal-acknowledgement";
        let original = test_terminal_binding(
            effects.clone(),
            &effects_bytes,
            parity_bytes,
            acknowledgement_bytes,
        );

        let journal = LockedJournal::open_fixed(&root, &accepted().scope_id, uid).unwrap();
        let head = journal.head().unwrap().unwrap();
        journal
            .persist_terminal_binding_artifact(
                "effects.response",
                &effects_bytes,
                &original.effects_response_sha256,
            )
            .unwrap();
        journal
            .persist_terminal_binding_artifact(
                "parity.proof",
                parity_bytes,
                &original.parity_proof_sha256,
            )
            .unwrap();
        journal
            .persist_terminal_binding_artifact(
                "terminal.acknowledgement",
                acknowledgement_bytes,
                &original.terminal_acknowledgement_sha256,
            )
            .unwrap();
        append_event(
            &journal,
            Some(&head),
            &accepted(),
            JournalEvent {
                kind: JournalEventKind::TerminalAcknowledgementBound,
                host_state: HostRetirementStateV2::TerminalAcknowledgementBound,
                effect_ordinal: None,
                effect_role: None,
                effect_identity_sha256: None,
                effect_invocation_attempt: None,
                observation_sha256: Some(original.parity_proof_sha256.clone()),
                response_sha256: Some(original.terminal_acknowledgement_sha256.clone()),
                preserving_classification: None,
            },
        )
        .unwrap();
        drop(journal);

        let alternate_ack = b"alternate-terminal-acknowledgement";
        let alternate = test_terminal_binding(effects, &effects_bytes, parity_bytes, alternate_ack);
        assert!(FinalizerEngine::new(&root, uid, PanicEffects)
            .bind_validated_terminal(&accepted(), &alternate)
            .is_err());
        let complete = FinalizerEngine::new(&root, uid, PanicEffects)
            .bind_validated_terminal(&accepted(), &original)
            .unwrap();
        assert_eq!(complete.state, FinalizerResponseStateV2::HostComplete);
    }

    #[test]
    fn replay_preflight_is_side_effect_free_when_absent_and_returns_validated_cursor() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        assert_eq!(
            replay_existing_claim(&root, uid, &accepted()).unwrap(),
            ExistingClaimDisposition::NoExistingClaim
        );
        assert!(!root.exists());

        let request = accepted();
        let journal = LockedJournal::open_fixed(&root, &request.scope_id, uid).unwrap();
        append_event(
            &journal,
            None,
            &request,
            JournalEvent {
                kind: JournalEventKind::FinalizerAccepted,
                host_state: HostRetirementStateV2::FinalizerAccepted,
                effect_ordinal: None,
                effect_role: None,
                effect_identity_sha256: None,
                effect_invocation_attempt: None,
                observation_sha256: None,
                response_sha256: None,
                preserving_classification: None,
            },
        )
        .unwrap();
        drop(journal);

        let ExistingClaimDisposition::RequiresEffectsRecovery(cursor) =
            replay_existing_claim(&root, uid, &request).unwrap()
        else {
            panic!("accepted journal must require effect recovery")
        };
        assert_eq!(cursor.completed_effect_ordinal(), 0);
        assert_eq!(cursor.effect_count(), 8);
        let pending = cursor.pending().unwrap();
        assert_eq!(pending.ordinal(), 1);
        assert_eq!(pending.role(), HostTargetRoleV2::CapabilitySignControl);
        assert_eq!(pending.phase(), RecoveryEffectPhase::NotPrepared);
    }

    #[test]
    fn accepted_recovery_preparation_failure_is_immutable_and_alternate_safe() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let request = accepted();
        let journal = LockedJournal::open_fixed(&root, &request.scope_id, uid).unwrap();
        append_event(
            &journal,
            None,
            &request,
            JournalEvent {
                kind: JournalEventKind::FinalizerAccepted,
                host_state: HostRetirementStateV2::FinalizerAccepted,
                effect_ordinal: None,
                effect_role: None,
                effect_identity_sha256: None,
                effect_invocation_attempt: None,
                observation_sha256: None,
                response_sha256: None,
                preserving_classification: None,
            },
        )
        .unwrap();
        drop(journal);

        let stopped = record_accepted_recovery_preparation_failure(&root, uid, &request).unwrap();
        assert_eq!(stopped.state, FinalizerResponseStateV2::PreservingStop);
        assert_eq!(
            stopped.preserving_classification,
            Some(PreservingStopClassificationV2::AmbiguousEffectState)
        );

        std::fs::remove_file(root.join(&request.scope_id).join("preserving.response")).unwrap();
        assert_eq!(
            record_accepted_recovery_preparation_failure(&root, uid, &request).unwrap(),
            stopped
        );

        let before = LockedJournal::open_fixed(&root, &request.scope_id, uid)
            .unwrap()
            .head()
            .unwrap()
            .unwrap();
        let mut alternate = accepted();
        alternate.request_digest = "ad".repeat(32);
        assert!(record_accepted_recovery_preparation_failure(&root, uid, &alternate).is_err());
        let after = LockedJournal::open_fixed(&root, &request.scope_id, uid)
            .unwrap()
            .head()
            .unwrap()
            .unwrap();
        assert_eq!(after, before);
    }

    #[test]
    fn recovery_preparation_failure_after_observed_effect_replays_without_targets() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let request = accepted();
        let plan =
            derive_host_effect_plan_v2(request.target_set_kind, &request.target_ledger).unwrap();
        let first = &plan[0];
        let journal = LockedJournal::open_fixed(&root, &request.scope_id, uid).unwrap();
        let accepted_head = append_event(
            &journal,
            None,
            &request,
            JournalEvent {
                kind: JournalEventKind::FinalizerAccepted,
                host_state: HostRetirementStateV2::FinalizerAccepted,
                effect_ordinal: None,
                effect_role: None,
                effect_identity_sha256: None,
                effect_invocation_attempt: None,
                observation_sha256: None,
                response_sha256: None,
                preserving_classification: None,
            },
        )
        .unwrap();
        let prepared = append_event(
            &journal,
            Some(&accepted_head),
            &request,
            JournalEvent {
                kind: JournalEventKind::EffectPrepared,
                host_state: HostRetirementStateV2::Removing,
                effect_ordinal: Some(first.ordinal),
                effect_role: Some(first.role),
                effect_identity_sha256: Some(first.target_identity_sha256.clone()),
                effect_invocation_attempt: Some(0),
                observation_sha256: None,
                response_sha256: None,
                preserving_classification: None,
            },
        )
        .unwrap();
        let target = derive_targets_from_ledger(
            request.target_set_kind,
            &request.scope_id,
            &request.target_ledger,
        )
        .unwrap()
        .remove(0);
        append_durable_effect_observed(
            &MemoryEffects::default(),
            &journal,
            Some(&prepared),
            &request,
            &target,
            &EffectObservation {
                class: ObservationClass::ExactFinal,
                observation_sha256: "bc".repeat(32),
            },
        )
        .unwrap();
        drop(journal);

        let stopped = record_accepted_recovery_preparation_failure(&root, uid, &request).unwrap();
        std::fs::remove_file(root.join(&request.scope_id).join("preserving.response")).unwrap();
        assert_eq!(
            record_accepted_recovery_preparation_failure(&root, uid, &request).unwrap(),
            stopped
        );
    }

    #[test]
    fn journal_effect_identity_must_match_the_signed_plan_ordinal() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let request = accepted();
        let journal = LockedJournal::open_fixed(&root, &request.scope_id, uid).unwrap();
        let accepted_head = append_event(
            &journal,
            None,
            &request,
            JournalEvent {
                kind: JournalEventKind::FinalizerAccepted,
                host_state: HostRetirementStateV2::FinalizerAccepted,
                effect_ordinal: None,
                effect_role: None,
                effect_identity_sha256: None,
                effect_invocation_attempt: None,
                observation_sha256: None,
                response_sha256: None,
                preserving_classification: None,
            },
        )
        .unwrap();
        append_event(
            &journal,
            Some(&accepted_head),
            &request,
            JournalEvent {
                kind: JournalEventKind::EffectPrepared,
                host_state: HostRetirementStateV2::Removing,
                effect_ordinal: Some(1),
                effect_role: Some(HostTargetRoleV2::CapabilitySignControl),
                effect_identity_sha256: Some("ff".repeat(32)),
                effect_invocation_attempt: Some(0),
                observation_sha256: None,
                response_sha256: None,
                preserving_classification: None,
            },
        )
        .unwrap();
        drop(journal);

        assert!(FinalizerEngine::new(&root, uid, PanicEffects)
            .execute(&request)
            .is_err());
    }

    #[test]
    fn effects_complete_requires_the_exact_final_signed_effect() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let request = accepted();
        let journal = LockedJournal::open_fixed(&root, &request.scope_id, uid).unwrap();
        let accepted_head = append_event(
            &journal,
            None,
            &request,
            JournalEvent {
                kind: JournalEventKind::FinalizerAccepted,
                host_state: HostRetirementStateV2::FinalizerAccepted,
                effect_ordinal: None,
                effect_role: None,
                effect_identity_sha256: None,
                effect_invocation_attempt: None,
                observation_sha256: None,
                response_sha256: None,
                preserving_classification: None,
            },
        )
        .unwrap();
        let plan =
            derive_host_effect_plan_v2(request.target_set_kind, &request.target_ledger).unwrap();
        let first = &plan[0];
        let prepared = append_event(
            &journal,
            Some(&accepted_head),
            &request,
            JournalEvent {
                kind: JournalEventKind::EffectPrepared,
                host_state: HostRetirementStateV2::Removing,
                effect_ordinal: Some(first.ordinal),
                effect_role: Some(first.role),
                effect_identity_sha256: Some(first.target_identity_sha256.clone()),
                effect_invocation_attempt: Some(0),
                observation_sha256: None,
                response_sha256: None,
                preserving_classification: None,
            },
        )
        .unwrap();
        let observed = append_event(
            &journal,
            Some(&prepared),
            &request,
            JournalEvent {
                kind: JournalEventKind::EffectObserved,
                host_state: HostRetirementStateV2::Removing,
                effect_ordinal: Some(first.ordinal),
                effect_role: Some(first.role),
                effect_identity_sha256: Some(first.target_identity_sha256.clone()),
                effect_invocation_attempt: Some(0),
                observation_sha256: Some("ee".repeat(32)),
                response_sha256: None,
                preserving_classification: None,
            },
        )
        .unwrap();
        append_event(
            &journal,
            Some(&observed),
            &request,
            JournalEvent {
                kind: JournalEventKind::EffectsComplete,
                host_state: HostRetirementStateV2::EffectsComplete,
                effect_ordinal: None,
                effect_role: None,
                effect_identity_sha256: None,
                effect_invocation_attempt: None,
                observation_sha256: Some("ef".repeat(32)),
                response_sha256: None,
                preserving_classification: None,
            },
        )
        .unwrap();
        drop(journal);

        assert!(replay_existing_claim(&root, uid, &request).is_err());
    }

    #[test]
    fn terminal_proof_mismatch_is_durable_but_alternate_claim_cannot_mutate_it() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let request = accepted();
        let effects = FinalizerEngine::new(&root, uid, MemoryEffects::default())
            .execute(&request)
            .unwrap();
        let effects_bytes = canonical_bytes_v2(&effects).unwrap();
        let stopped = FinalizerEngine::new(&root, uid, PanicEffects)
            .bind_terminal_proofs(
                &request,
                &effects_bytes,
                b"invalid parity",
                b"invalid acknowledgement",
                "invalid-key",
            )
            .unwrap();
        assert_eq!(stopped.state, FinalizerResponseStateV2::PreservingStop);
        assert_eq!(
            stopped.preserving_classification,
            Some(PreservingStopClassificationV2::TerminalProofMismatch)
        );
        assert_eq!(
            replay_existing_claim(&root, uid, &request).unwrap(),
            ExistingClaimDisposition::Response(stopped)
        );

        let before = LockedJournal::open_fixed(&root, &request.scope_id, uid)
            .unwrap()
            .head()
            .unwrap()
            .unwrap();
        let mut alternate = accepted();
        alternate.request_digest = "aa".repeat(32);
        assert!(FinalizerEngine::new(&root, uid, PanicEffects)
            .bind_terminal_proofs(
                &alternate,
                &effects_bytes,
                b"alternate",
                b"alternate",
                "alternate",
            )
            .is_err());
        let after = LockedJournal::open_fixed(&root, &request.scope_id, uid)
            .unwrap()
            .head()
            .unwrap()
            .unwrap();
        assert_eq!(after, before);
    }

    #[test]
    fn same_digest_new_process_rejoin_replays_without_target_inspection() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let initial = accepted();
        let response = FinalizerEngine::new(&root, uid, MemoryEffects::default())
            .execute(&initial)
            .unwrap();

        let mut rejoin = initial.clone();
        rejoin.peer_attestation = b"peer-attestation-new-process-instance".to_vec();
        rejoin.peer_attestation_sha256 = sha256_hex_v2(&rejoin.peer_attestation);
        rejoin.first_acceptance_time_verified = false;
        let replay = FinalizerEngine::new(&root, uid, PanicEffects)
            .execute(&rejoin)
            .unwrap();
        assert_eq!(replay, response);

        let scope_root = root.join(initial.scope_id());
        assert_eq!(
            std::fs::read(scope_root.join("peer.attestation")).unwrap(),
            initial.peer_attestation
        );
        assert_eq!(
            std::fs::read(scope_root.join(format!(
                "peer.rejoin.{}.attestation",
                rejoin.peer_attestation_sha256
            )))
            .unwrap(),
            rejoin.peer_attestation
        );
        let journal = LockedJournal::open_fixed(&root, initial.scope_id(), uid).unwrap();
        let head = journal.head().unwrap().unwrap();
        assert_eq!(
            head.record.accepted_peer_attestation_sha256,
            initial.peer_attestation_sha256
        );
        assert_eq!(
            head.record.accepted_peer_identity_sha256,
            initial.accepted_peer_identity_sha256
        );
    }

    #[test]
    fn same_request_from_alternate_stable_peer_is_rejected_before_inspection() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let initial = accepted();
        FinalizerEngine::new(&root, uid, MemoryEffects::default())
            .execute(&initial)
            .unwrap();

        let before = LockedJournal::open_fixed(&root, initial.scope_id(), uid)
            .unwrap()
            .head()
            .unwrap()
            .unwrap();
        let mut alternate = initial.clone();
        alternate.peer_attestation = b"alternate-code-process-attestation".to_vec();
        alternate.peer_attestation_sha256 = sha256_hex_v2(&alternate.peer_attestation);
        alternate.accepted_peer_identity_sha256 = "fe".repeat(32);
        alternate.first_acceptance_time_verified = false;
        assert!(FinalizerEngine::new(&root, uid, PanicEffects)
            .execute(&alternate)
            .is_err());
        let after = LockedJournal::open_fixed(&root, initial.scope_id(), uid)
            .unwrap()
            .head()
            .unwrap()
            .unwrap();
        assert_eq!(after, before);
        assert!(!root
            .join(initial.scope_id())
            .join(format!(
                "peer.rejoin.{}.attestation",
                alternate.peer_attestation_sha256
            ))
            .exists());
    }

    #[test]
    fn stable_peer_identity_excludes_only_process_instance_fields() {
        let peer = PeerAttestation {
            schema_owner: "substrate.r3-macos-finalizer-peer-attestation".to_string(),
            schema_version: 1,
            effective_uid: 501,
            effective_gid: 20,
            canonical_account: "coordinator".to_string(),
            pid: 123,
            audit_token_sha256: "11".repeat(32),
            process_start_sha256: "22".repeat(32),
            executable_path: "/fixed/coordinator".to_string(),
            executable_physical_identity_sha256: "33".repeat(32),
            executable_sha256: "44".repeat(32),
            cdhash: "55".repeat(20),
        };
        let mut restarted = peer.clone();
        restarted.pid = 456;
        restarted.audit_token_sha256 = "66".repeat(32);
        restarted.process_start_sha256 = "77".repeat(32);
        assert_eq!(
            stable_peer_identity_sha256(&peer).unwrap(),
            stable_peer_identity_sha256(&restarted).unwrap()
        );

        for mutate in [
            |value: &mut PeerAttestation| value.effective_gid = 80,
            |value: &mut PeerAttestation| value.executable_path.push_str("-alternate"),
            |value: &mut PeerAttestation| value.executable_sha256 = "88".repeat(32),
            |value: &mut PeerAttestation| value.cdhash = "99".repeat(20),
        ] {
            let mut alternate = restarted.clone();
            mutate(&mut alternate);
            assert_ne!(
                stable_peer_identity_sha256(&peer).unwrap(),
                stable_peer_identity_sha256(&alternate).unwrap()
            );
        }
    }
}
