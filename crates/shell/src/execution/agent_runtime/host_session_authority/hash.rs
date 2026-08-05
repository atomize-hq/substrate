use std::fmt;

use sha2::{Digest, Sha256};

use super::canonical_json::{self, CanonicalJsonError};
use super::schema::{AuthorityObjectCommitmentV1, AuthorityObjectKindV1};
use super::validation::{CanonicalHashInputV1, ValidatedCanonicalV1, ValidationError};

const HMAC_INPUT_PREFIX: &[u8] = b"substrate.a1.hmac-input.v1\0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SensitiveDomainV1 {
    TransitionInput,
    ParticipantLeaseToken,
    RawTransportPayload,
}

impl SensitiveDomainV1 {
    pub(crate) fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::TransitionInput => b"substrate.a1.transition-input.v1",
            Self::ParticipantLeaseToken => b"substrate.a1.participant-lease-token.v1",
            Self::RawTransportPayload => b"substrate.a1.raw-transport-payload.v1",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HashError(&'static str);

impl fmt::Display for HashError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for HashError {}

#[derive(Debug)]
pub(crate) enum CanonicalHashError {
    Invalid(ValidationError),
    Encoding(CanonicalJsonError),
    WrongObjectKind,
}

impl fmt::Display for CanonicalHashError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(error) => error.fmt(formatter),
            Self::Encoding(error) => error.fmt(formatter),
            Self::WrongObjectKind => formatter.write_str("named object bytes do not match kind"),
        }
    }
}

impl std::error::Error for CanonicalHashError {}

pub(crate) fn canonical_sha256<T: CanonicalHashInputV1>(
    value: &T,
) -> Result<String, CanonicalHashError> {
    value.validate().map_err(CanonicalHashError::Invalid)?;
    let canonical_bytes = canonical_json::to_vec(value).map_err(CanonicalHashError::Encoding)?;
    Ok(lower_hex(&Sha256::digest(canonical_bytes)))
}

pub(crate) enum CanonicalObjectHashInputV1<'a> {
    AgentDescriptor(&'a super::schema::AgentDescriptorHashInputV1),
    RetainedWorker(&'a super::schema::RetainedWorkerObjectHashInputV1),
    ResumeHandle(&'a super::schema::ResumeHandleHashInputV1),
    Policy(&'a super::schema::PolicyObjectHashInputV1),
    HostAttachContract(&'a super::schema::HostAttachContractHashInputV1),
    TransitionTransportPayload(&'a super::schema::TransitionTransportPayloadObjectV1),
    ApplicationResult(&'a super::schema::ApplicationResultHashInputV1),
    InputAcceptance(&'a super::schema::InputAcceptanceHashInputV1),
    StartupOwnershipResult(&'a super::schema::StartupOwnershipResultHashInputV1),
    ObligationSnapshot(&'a super::schema::ObligationSnapshotHashInputV1),
    PostTurnProtocolEvent(&'a super::schema::PostTurnProtocolEventHashInputV1),
    PostTurnCompletion(&'a super::schema::PostTurnCompletionHashInputV1),
    TerminalHandoff(&'a super::schema::TerminalHandoffHashInputV1),
    TerminalHandoffV2(&'a super::schema::TerminalHandoffHashInputV2),
}

impl CanonicalObjectHashInputV1<'_> {
    fn object_kind(&self) -> AuthorityObjectKindV1 {
        match self {
            Self::AgentDescriptor(_) => AuthorityObjectKindV1::AgentDescriptor,
            Self::RetainedWorker(_) => AuthorityObjectKindV1::RetainedWorker,
            Self::ResumeHandle(_) => AuthorityObjectKindV1::ResumeHandle,
            Self::Policy(_) => AuthorityObjectKindV1::Policy,
            Self::HostAttachContract(_) => AuthorityObjectKindV1::HostAttachContract,
            Self::TransitionTransportPayload(_) => {
                AuthorityObjectKindV1::TransitionTransportPayload
            }
            Self::ApplicationResult(_) => AuthorityObjectKindV1::ApplicationResult,
            Self::InputAcceptance(_) => AuthorityObjectKindV1::InputAcceptance,
            Self::StartupOwnershipResult(_) => AuthorityObjectKindV1::StartupOwnershipResult,
            Self::ObligationSnapshot(_) => AuthorityObjectKindV1::ObligationSnapshot,
            Self::PostTurnProtocolEvent(_) => AuthorityObjectKindV1::PostTurnProtocolEvent,
            Self::PostTurnCompletion(_) => AuthorityObjectKindV1::PostTurnCompletion,
            Self::TerminalHandoff(_) | Self::TerminalHandoffV2(_) => {
                AuthorityObjectKindV1::TerminalHandoff
            }
        }
    }
}

pub(crate) fn canonical_object_bytes(
    expected_kind: AuthorityObjectKindV1,
    input: CanonicalObjectHashInputV1<'_>,
) -> Result<Vec<u8>, CanonicalHashError> {
    if expected_kind != input.object_kind() {
        return Err(CanonicalHashError::WrongObjectKind);
    }
    macro_rules! encode {
        ($value:expr) => {{
            $value.validate().map_err(CanonicalHashError::Invalid)?;
            canonical_json::to_vec($value).map_err(CanonicalHashError::Encoding)
        }};
    }
    match input {
        CanonicalObjectHashInputV1::AgentDescriptor(value) => encode!(value),
        CanonicalObjectHashInputV1::RetainedWorker(value) => encode!(value),
        CanonicalObjectHashInputV1::ResumeHandle(value) => encode!(value),
        CanonicalObjectHashInputV1::Policy(value) => encode!(value),
        CanonicalObjectHashInputV1::HostAttachContract(value) => encode!(value),
        CanonicalObjectHashInputV1::TransitionTransportPayload(value) => encode!(value),
        CanonicalObjectHashInputV1::ApplicationResult(value) => encode!(value),
        CanonicalObjectHashInputV1::InputAcceptance(value) => encode!(value),
        CanonicalObjectHashInputV1::StartupOwnershipResult(value) => encode!(value),
        CanonicalObjectHashInputV1::ObligationSnapshot(value) => encode!(value),
        CanonicalObjectHashInputV1::PostTurnProtocolEvent(value) => encode!(value),
        CanonicalObjectHashInputV1::PostTurnCompletion(value) => encode!(value),
        CanonicalObjectHashInputV1::TerminalHandoff(value) => encode!(value),
        CanonicalObjectHashInputV1::TerminalHandoffV2(value) => encode!(value),
    }
}

pub(crate) fn store_hmac_sha256(
    key: &[u8; 32],
    domain: SensitiveDomainV1,
    authority_store_id: &str,
    intent_id: &str,
    run_id: Option<&str>,
    raw: &[u8],
) -> Result<String, HashError> {
    if authority_store_id.is_empty() {
        return Err(HashError("authority store ID must be non-empty"));
    }
    if intent_id.is_empty() {
        return Err(HashError("intent ID must be non-empty"));
    }
    let run_id = run_id.filter(|run_id| !run_id.is_empty()).ok_or(HashError(
        "A1 sensitive commitments require a present non-empty run ID",
    ))?;
    match domain {
        SensitiveDomainV1::TransitionInput => {}
        SensitiveDomainV1::ParticipantLeaseToken => {
            if raw.is_empty() || std::str::from_utf8(raw).is_err() {
                return Err(HashError("participant lease token must be non-empty UTF-8"));
            }
        }
        SensitiveDomainV1::RawTransportPayload => {
            let payload: super::schema::TransitionTransportPayloadObjectV1 =
                canonical_json::from_slice(raw)
                    .map_err(|_| HashError("transport payload is not canonical V1"))?;
            payload
                .validate()
                .map_err(|_| HashError("transport payload is not valid V1"))?;
            if payload.intent_id != intent_id || payload.run_id != run_id {
                return Err(HashError(
                    "transport payload does not match parent intent and run",
                ));
            }
        }
    }

    let mut input = Vec::with_capacity(
        HMAC_INPUT_PREFIX.len()
            + domain.as_bytes().len()
            + authority_store_id.len()
            + intent_id.len()
            + run_id.len()
            + raw.len()
            + 41,
    );
    input.extend_from_slice(HMAC_INPUT_PREFIX);
    append_len64(&mut input, domain.as_bytes());
    append_len64(&mut input, authority_store_id.as_bytes());
    append_len64(&mut input, intent_id.as_bytes());
    input.push(0x01);
    append_len64(&mut input, run_id.as_bytes());
    append_len64(&mut input, raw);

    Ok(lower_hex(&hmac_sha256(key, &input)))
}

pub(crate) fn verify_store_hmac_sha256(
    expected_digest_hex: &str,
    key: &[u8; 32],
    domain: SensitiveDomainV1,
    authority_store_id: &str,
    intent_id: &str,
    parent_run_id: Option<&str>,
    raw: &[u8],
) -> Result<(), HashError> {
    if expected_digest_hex
        != store_hmac_sha256(
            key,
            domain,
            authority_store_id,
            intent_id,
            parent_run_id,
            raw,
        )?
    {
        return Err(HashError("sensitive commitment does not match parent run"));
    }
    Ok(())
}

pub(crate) fn validate_object_commitment_rule(
    object_kind: AuthorityObjectKindV1,
    schema_version: u32,
    commitment: &AuthorityObjectCommitmentV1,
) -> Result<(), HashError> {
    super::validation::validate_object_commitment_rule(object_kind, schema_version, commitment)
        .map_err(|_| HashError("authority object kind and commitment rule do not match"))
}

fn append_len64(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(&(value.len() as u64).to_be_bytes());
    output.extend_from_slice(value);
}

fn hmac_sha256(key: &[u8; 32], input: &[u8]) -> [u8; 32] {
    const BLOCK_LEN: usize = 64;

    let mut inner_key = [0x36_u8; BLOCK_LEN];
    let mut outer_key = [0x5c_u8; BLOCK_LEN];
    for (index, byte) in key.iter().enumerate() {
        inner_key[index] ^= byte;
        outer_key[index] ^= byte;
    }

    let mut inner = Sha256::new();
    inner.update(inner_key);
    inner.update(input);
    let inner_digest = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(outer_key);
    outer.update(inner_digest);
    outer.finalize().into()
}

fn lower_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: [u8; 32] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
        0x1e, 0x1f,
    ];

    #[test]
    fn store_hmac_sha256_matches_independent_binary_framing_vector() {
        assert_eq!(
            store_hmac_sha256(
                &TEST_KEY,
                SensitiveDomainV1::TransitionInput,
                "as_11111111111111111111111111111111",
                "intent-vector",
                Some("run-vector"),
                b"prompt\nbytes",
            )
            .unwrap(),
            "b36127c24806ea110d1ac175557f46741ebfd83a81b3e724de47cb714ca6ddd6"
        );
    }

    #[test]
    fn store_hmac_sha256_binds_every_context_member_and_domain() {
        let base = store_hmac_sha256(
            &TEST_KEY,
            SensitiveDomainV1::TransitionInput,
            "as_11111111111111111111111111111111",
            "intent-vector",
            Some("run-vector"),
            b"raw",
        )
        .unwrap();

        for changed in [
            store_hmac_sha256(
                &TEST_KEY,
                SensitiveDomainV1::ParticipantLeaseToken,
                "as_11111111111111111111111111111111",
                "intent-vector",
                Some("run-vector"),
                b"raw",
            )
            .unwrap(),
            store_hmac_sha256(
                &TEST_KEY,
                SensitiveDomainV1::TransitionInput,
                "as_22222222222222222222222222222222",
                "intent-vector",
                Some("run-vector"),
                b"raw",
            )
            .unwrap(),
            store_hmac_sha256(
                &TEST_KEY,
                SensitiveDomainV1::TransitionInput,
                "as_11111111111111111111111111111111",
                "other-intent",
                Some("run-vector"),
                b"raw",
            )
            .unwrap(),
            store_hmac_sha256(
                &TEST_KEY,
                SensitiveDomainV1::TransitionInput,
                "as_11111111111111111111111111111111",
                "intent-vector",
                Some("other-run"),
                b"raw",
            )
            .unwrap(),
            store_hmac_sha256(
                &TEST_KEY,
                SensitiveDomainV1::TransitionInput,
                "as_11111111111111111111111111111111",
                "intent-vector",
                Some("run-vector"),
                b"other-raw",
            )
            .unwrap(),
        ] {
            assert_ne!(changed, base);
        }
    }

    #[test]
    fn store_hmac_sha256_fails_before_hashing_missing_binding_context() {
        assert!(store_hmac_sha256(
            &TEST_KEY,
            SensitiveDomainV1::TransitionInput,
            "",
            "intent-vector",
            Some("run-vector"),
            b"raw",
        )
        .is_err());
        assert!(store_hmac_sha256(
            &TEST_KEY,
            SensitiveDomainV1::TransitionInput,
            "as_11111111111111111111111111111111",
            "",
            Some("run-vector"),
            b"raw",
        )
        .is_err());
        assert!(store_hmac_sha256(
            &TEST_KEY,
            SensitiveDomainV1::TransitionInput,
            "as_11111111111111111111111111111111",
            "intent-vector",
            Some(""),
            b"raw",
        )
        .is_err());
        assert!(store_hmac_sha256(
            &TEST_KEY,
            SensitiveDomainV1::TransitionInput,
            "as_11111111111111111111111111111111",
            "intent-vector",
            None,
            b"raw",
        )
        .is_err());
    }

    #[test]
    fn object_kind_commitment_rules_are_closed_and_cross_kind_safe() {
        let canonical = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "0".repeat(64),
        };
        for kind in [
            AuthorityObjectKindV1::AgentDescriptor,
            AuthorityObjectKindV1::RetainedWorker,
            AuthorityObjectKindV1::ResumeHandle,
            AuthorityObjectKindV1::Policy,
            AuthorityObjectKindV1::HostAttachContract,
            AuthorityObjectKindV1::ApplicationResult,
            AuthorityObjectKindV1::InputAcceptance,
            AuthorityObjectKindV1::StartupOwnershipResult,
            AuthorityObjectKindV1::ObligationSnapshot,
            AuthorityObjectKindV1::PostTurnProtocolEvent,
            AuthorityObjectKindV1::PostTurnCompletion,
            AuthorityObjectKindV1::TerminalHandoff,
        ] {
            assert!(validate_object_commitment_rule(kind, 1, &canonical).is_ok());
        }
        assert!(validate_object_commitment_rule(
            AuthorityObjectKindV1::TerminalHandoff,
            2,
            &canonical,
        )
        .is_ok());

        for (kind, domain) in [
            (
                AuthorityObjectKindV1::TransitionInput,
                SensitiveDomainV1::TransitionInput,
            ),
            (
                AuthorityObjectKindV1::LeaseToken,
                SensitiveDomainV1::ParticipantLeaseToken,
            ),
            (
                AuthorityObjectKindV1::TransitionTransportPayload,
                SensitiveDomainV1::RawTransportPayload,
            ),
        ] {
            let commitment = AuthorityObjectCommitmentV1::StoreHmacSha256 {
                key_id: "ak_00000000000000000000000000000000".to_string(),
                domain: std::str::from_utf8(domain.as_bytes()).unwrap().to_string(),
                digest_hex: "0".repeat(64),
            };
            assert!(validate_object_commitment_rule(kind, 1, &commitment).is_ok());
            assert!(validate_object_commitment_rule(
                AuthorityObjectKindV1::AgentDescriptor,
                1,
                &commitment,
            )
            .is_err());
        }
        assert!(validate_object_commitment_rule(
            AuthorityObjectKindV1::TransitionInput,
            1,
            &AuthorityObjectCommitmentV1::StoreHmacSha256 {
                key_id: "ak_00000000000000000000000000000000".to_string(),
                domain: "substrate.a1.participant-lease-token.v1".to_string(),
                digest_hex: "0".repeat(64),
            },
        )
        .is_err());
        assert!(validate_object_commitment_rule(
            AuthorityObjectKindV1::AgentDescriptor,
            2,
            &canonical,
        )
        .is_err());
        assert!(validate_object_commitment_rule(
            AuthorityObjectKindV1::AgentDescriptor,
            1,
            &AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "A".repeat(64),
            },
        )
        .is_err());
        assert!(validate_object_commitment_rule(
            AuthorityObjectKindV1::TransitionInput,
            1,
            &AuthorityObjectCommitmentV1::StoreHmacSha256 {
                key_id: "ak_short".to_string(),
                domain: "substrate.a1.transition-input.v1".to_string(),
                digest_hex: "0".repeat(64),
            },
        )
        .is_err());
    }
}
