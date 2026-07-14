use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

/// Equality-only representation of a commitment owned by another authority boundary.
///
/// This type deliberately provides no minting, verification, or interpretation API.
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value")]
pub enum OpaqueAuthorityCommitmentV1 {
    CanonicalSha256 {
        digest_hex: String,
    },
    StoreHmacSha256 {
        key_id: String,
        domain: String,
        digest_hex: String,
    },
}

#[derive(Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
enum OpaqueAuthorityCommitmentDef {
    CanonicalSha256(CanonicalSha256Def),
    StoreHmacSha256(StoreHmacSha256Def),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CanonicalSha256Def {
    digest_hex: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoreHmacSha256Def {
    key_id: String,
    domain: String,
    digest_hex: String,
}

impl TryFrom<OpaqueAuthorityCommitmentDef> for OpaqueAuthorityCommitmentV1 {
    type Error = String;

    fn try_from(value: OpaqueAuthorityCommitmentDef) -> Result<Self, Self::Error> {
        let commitment = match value {
            OpaqueAuthorityCommitmentDef::CanonicalSha256(value) => Self::CanonicalSha256 {
                digest_hex: value.digest_hex,
            },
            OpaqueAuthorityCommitmentDef::StoreHmacSha256(value) => Self::StoreHmacSha256 {
                key_id: value.key_id,
                domain: value.domain,
                digest_hex: value.digest_hex,
            },
        };
        commitment.validate()?;
        Ok(commitment)
    }
}

impl<'de> Deserialize<'de> for OpaqueAuthorityCommitmentV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = OpaqueAuthorityCommitmentDef::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

impl fmt::Debug for OpaqueAuthorityCommitmentV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CanonicalSha256 { .. } => formatter
                .debug_struct("CanonicalSha256")
                .field("digest_hex", &"[REDACTED]")
                .finish(),
            Self::StoreHmacSha256 { .. } => formatter
                .debug_struct("StoreHmacSha256")
                .field("key_id", &"[REDACTED]")
                .field("domain", &"[REDACTED]")
                .field("digest_hex", &"[REDACTED]")
                .finish(),
        }
    }
}

impl OpaqueAuthorityCommitmentV1 {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::CanonicalSha256 { digest_hex } => validate_digest_hex(digest_hex),
            Self::StoreHmacSha256 {
                key_id,
                domain,
                digest_hex,
            } => {
                validate_required("commitment.key_id", key_id)?;
                validate_required("commitment.domain", domain)?;
                validate_digest_hex(digest_hex)
            }
        }
    }
}

/// Correlates accepted work with an already owner-supplied host transition by exact equality.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HostTransitionWorkCorrelationV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub orchestration_session_id: String,
    pub authoritative_participant_id: String,
    pub transition_intent_id: String,
    pub transition_intent_revision_observed: u64,
    pub transition_run_id: String,
    pub transition_payload_commitment: OpaqueAuthorityCommitmentV1,
    pub authority_revision_observed: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HostTransitionWorkCorrelationDef {
    schema_version: u32,
    authority_store_id: String,
    orchestration_session_id: String,
    authoritative_participant_id: String,
    transition_intent_id: String,
    transition_intent_revision_observed: u64,
    transition_run_id: String,
    transition_payload_commitment: OpaqueAuthorityCommitmentV1,
    authority_revision_observed: u64,
}

impl TryFrom<HostTransitionWorkCorrelationDef> for HostTransitionWorkCorrelationV1 {
    type Error = String;

    fn try_from(value: HostTransitionWorkCorrelationDef) -> Result<Self, Self::Error> {
        let correlation = Self {
            schema_version: value.schema_version,
            authority_store_id: value.authority_store_id,
            orchestration_session_id: value.orchestration_session_id,
            authoritative_participant_id: value.authoritative_participant_id,
            transition_intent_id: value.transition_intent_id,
            transition_intent_revision_observed: value.transition_intent_revision_observed,
            transition_run_id: value.transition_run_id,
            transition_payload_commitment: value.transition_payload_commitment,
            authority_revision_observed: value.authority_revision_observed,
        };
        correlation.validate()?;
        Ok(correlation)
    }
}

impl<'de> Deserialize<'de> for HostTransitionWorkCorrelationV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = HostTransitionWorkCorrelationDef::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

impl HostTransitionWorkCorrelationV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported host_transition_correlation.schema_version: {} (expected 1)",
                self.schema_version
            ));
        }
        validate_required(
            "host_transition_correlation.authority_store_id",
            &self.authority_store_id,
        )?;
        validate_required(
            "host_transition_correlation.orchestration_session_id",
            &self.orchestration_session_id,
        )?;
        validate_required(
            "host_transition_correlation.authoritative_participant_id",
            &self.authoritative_participant_id,
        )?;
        validate_required(
            "host_transition_correlation.transition_intent_id",
            &self.transition_intent_id,
        )?;
        validate_required(
            "host_transition_correlation.transition_run_id",
            &self.transition_run_id,
        )?;
        if self.transition_intent_revision_observed == 0 {
            return Err(
                "host_transition_correlation.transition_intent_revision_observed must be positive"
                    .to_string(),
            );
        }
        if self.authority_revision_observed == 0 {
            return Err(
                "host_transition_correlation.authority_revision_observed must be positive"
                    .to_string(),
            );
        }
        self.transition_payload_commitment.validate()
    }
}

fn validate_required(field: &str, value: &str) -> Result<(), String> {
    if value.is_empty() || value.trim() != value {
        return Err(format!("{field} must be non-empty and trimmed"));
    }
    Ok(())
}

fn validate_digest_hex(value: &str) -> Result<(), String> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(
            "commitment.digest_hex must be 64 lowercase hexadecimal characters".to_string(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{HostTransitionWorkCorrelationV1, OpaqueAuthorityCommitmentV1};

    const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn sample_correlation() -> HostTransitionWorkCorrelationV1 {
        HostTransitionWorkCorrelationV1 {
            schema_version: 1,
            authority_store_id: "as_store".to_string(),
            orchestration_session_id: "session-1".to_string(),
            authoritative_participant_id: "participant-1".to_string(),
            transition_intent_id: "intent-1".to_string(),
            transition_intent_revision_observed: 7,
            transition_run_id: "transition-run-1".to_string(),
            transition_payload_commitment: OpaqueAuthorityCommitmentV1::StoreHmacSha256 {
                key_id: "ak_key".to_string(),
                domain: "substrate.a1.transition-input.v1".to_string(),
                digest_hex: DIGEST.to_string(),
            },
            authority_revision_observed: 11,
        }
    }

    #[test]
    fn opaque_authority_commitment_v1_round_trips_exact_variants() {
        let canonical = OpaqueAuthorityCommitmentV1::CanonicalSha256 {
            digest_hex: DIGEST.to_string(),
        };
        let canonical_json = serde_json::to_value(&canonical).unwrap();
        assert_eq!(
            canonical_json,
            serde_json::json!({
                "kind": "CanonicalSha256",
                "value": { "digest_hex": DIGEST }
            })
        );
        assert_eq!(
            serde_json::from_value::<OpaqueAuthorityCommitmentV1>(canonical_json).unwrap(),
            canonical
        );

        let hmac = sample_correlation().transition_payload_commitment;
        let hmac_json = serde_json::to_value(&hmac).unwrap();
        assert_eq!(
            serde_json::from_value::<OpaqueAuthorityCommitmentV1>(hmac_json).unwrap(),
            hmac
        );
    }

    #[test]
    fn opaque_authority_commitment_v1_rejects_unknown_or_malformed_values() {
        let unknown = serde_json::json!({
            "kind": "FutureCommitment",
            "value": { "digest_hex": DIGEST }
        });
        assert!(serde_json::from_value::<OpaqueAuthorityCommitmentV1>(unknown).is_err());

        let uppercase = serde_json::json!({
            "kind": "CanonicalSha256",
            "value": { "digest_hex": DIGEST.to_ascii_uppercase() }
        });
        assert!(serde_json::from_value::<OpaqueAuthorityCommitmentV1>(uppercase).is_err());

        let extra = serde_json::json!({
            "kind": "StoreHmacSha256",
            "value": {
                "key_id": "ak_key",
                "domain": "substrate.a1.transition-input.v1",
                "digest_hex": DIGEST,
                "semantic_claim": true
            }
        });
        assert!(serde_json::from_value::<OpaqueAuthorityCommitmentV1>(extra).is_err());

        let top_level_extra = serde_json::json!({
            "kind": "CanonicalSha256",
            "value": { "digest_hex": DIGEST },
            "semantic_claim": true
        });
        assert!(serde_json::from_value::<OpaqueAuthorityCommitmentV1>(top_level_extra).is_err());
    }

    #[test]
    fn host_transition_work_correlation_v1_round_trips_and_validates_required_fields() {
        let correlation = sample_correlation();
        let json = serde_json::to_value(&correlation).unwrap();
        assert_eq!(
            serde_json::from_value::<HostTransitionWorkCorrelationV1>(json.clone()).unwrap(),
            correlation
        );

        let mut missing_store = json.clone();
        missing_store["authority_store_id"] = serde_json::Value::String(String::new());
        assert!(serde_json::from_value::<HostTransitionWorkCorrelationV1>(missing_store).is_err());

        let mut zero_revision = json.clone();
        zero_revision["authority_revision_observed"] = serde_json::Value::from(0);
        assert!(serde_json::from_value::<HostTransitionWorkCorrelationV1>(zero_revision).is_err());

        let mut unknown = json;
        unknown["semantic_authority"] = serde_json::Value::Bool(true);
        assert!(serde_json::from_value::<HostTransitionWorkCorrelationV1>(unknown).is_err());
    }

    #[test]
    fn authority_commitment_debug_output_redacts_commitment_material() {
        let commitment = sample_correlation().transition_payload_commitment;
        let rendered = format!("{commitment:?}");
        assert!(!rendered.contains(DIGEST));
        assert!(!rendered.contains("ak_key"));
        assert!(rendered.contains("REDACTED"));
    }
}
