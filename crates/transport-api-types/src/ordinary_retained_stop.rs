//! Data for explicit ordinary retained-worker retirement. Decoding grants no
//! control permission and proves no native cleanup. The runtime authenticates
//! its durable Stop intent; the service authenticates its actual member owner.
use crate::{
    validate_narrowing_identity, validate_prefixed_uuid_v7, DispatchPolicyCommitmentRefCarrierV1,
};
use serde::{Deserialize, Serialize};

/// Exact reference to a separately authenticated durable Stop intent.
/// Generation zero is valid; absence never means that a worker retired.
/// ```
/// use transport_api_types::OrdinaryRetainedWorkerStopRequestV1;
/// assert!(serde_json::from_str::<OrdinaryRetainedWorkerStopRequestV1>("{}").is_err());
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "OrdinaryRetainedWorkerStopRequestV1Def")]
pub struct OrdinaryRetainedWorkerStopRequestV1 {
    pub schema_version: u32,
    pub stop_request_id: String,
    pub authority_store_id: String,
    pub orchestration_session_id: String,
    pub caller_participant_id: String,
    pub target_participant_id: String,
    pub target_backend_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub registration_id: String,
    pub bootstrap_run_id: String,
    pub native_session_id: String,
    pub worker_cap_ref: DispatchPolicyCommitmentRefCarrierV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OrdinaryRetainedWorkerStopRequestV1Def {
    schema_version: u32,
    stop_request_id: String,
    authority_store_id: String,
    orchestration_session_id: String,
    caller_participant_id: String,
    target_participant_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    registration_id: String,
    bootstrap_run_id: String,
    native_session_id: String,
    worker_cap_ref: DispatchPolicyCommitmentRefCarrierV1,
}

impl OrdinaryRetainedWorkerStopRequestV1 {
    /// Validate representation only; the original runtime owner authenticates
    /// every field against its immutable issuance, R0 and current Stop grant.
    /// ```
    /// # use transport_api_types::OrdinaryRetainedWorkerStopRequestV1;
    /// # fn check(data: &OrdinaryRetainedWorkerStopRequestV1) -> Result<(), String> {
    /// data.validate()
    /// # }
    /// ```
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("ordinary retained Stop requires schema 1".into());
        }
        for (name, value) in [
            ("stop_request_id", &self.stop_request_id),
            ("authority_store_id", &self.authority_store_id),
            ("orchestration_session_id", &self.orchestration_session_id),
            ("caller_participant_id", &self.caller_participant_id),
            ("target_participant_id", &self.target_participant_id),
            ("target_backend_id", &self.target_backend_id),
            ("world_id", &self.world_id),
            ("registration_id", &self.registration_id),
            ("bootstrap_run_id", &self.bootstrap_run_id),
            ("native_session_id", &self.native_session_id),
        ] {
            validate_narrowing_identity(name, value)?;
        }
        self.worker_cap_ref.validate()?;
        if self.worker_cap_ref.authority_store_id != self.authority_store_id {
            return Err("ordinary retained Stop cap belongs to another authority store".into());
        }
        Ok(())
    }
}

impl TryFrom<OrdinaryRetainedWorkerStopRequestV1Def> for OrdinaryRetainedWorkerStopRequestV1 {
    type Error = String;
    fn try_from(value: OrdinaryRetainedWorkerStopRequestV1Def) -> Result<Self, Self::Error> {
        let request = Self {
            schema_version: value.schema_version,
            stop_request_id: value.stop_request_id,
            authority_store_id: value.authority_store_id,
            orchestration_session_id: value.orchestration_session_id,
            caller_participant_id: value.caller_participant_id,
            target_participant_id: value.target_participant_id,
            target_backend_id: value.target_backend_id,
            world_id: value.world_id,
            world_generation: value.world_generation,
            registration_id: value.registration_id,
            bootstrap_run_id: value.bootstrap_run_id,
            native_session_id: value.native_session_id,
            worker_cap_ref: value.worker_cap_ref,
        };
        request.validate()?;
        Ok(request)
    }
}

/// The actual service member owner reports explicit parked-member retirement.
/// This data is distinct from bootstrap Exit or a cancellation delivery boolean.
/// The receiver still authenticates the service, exact intent and source joins.
/// ```
/// use transport_api_types::OrdinaryRetainedWorkerStopAcknowledgmentV1;
/// assert!(serde_json::from_str::<OrdinaryRetainedWorkerStopAcknowledgmentV1>("{}").is_err());
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "OrdinaryRetainedWorkerStopAcknowledgmentV1Def")]
pub struct OrdinaryRetainedWorkerStopAcknowledgmentV1 {
    pub schema_version: u32,
    pub request: OrdinaryRetainedWorkerStopRequestV1,
    pub retirement_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OrdinaryRetainedWorkerStopAcknowledgmentV1Def {
    schema_version: u32,
    request: OrdinaryRetainedWorkerStopRequestV1,
    retirement_id: String,
}

impl OrdinaryRetainedWorkerStopAcknowledgmentV1 {
    /// Check data against the exact original request, without granting control.
    /// ```
    /// # use transport_api_types::{OrdinaryRetainedWorkerStopAcknowledgmentV1, OrdinaryRetainedWorkerStopRequestV1};
    /// # fn check(data: &OrdinaryRetainedWorkerStopAcknowledgmentV1, expected: &OrdinaryRetainedWorkerStopRequestV1) -> Result<(), String> {
    /// data.validate(expected)
    /// # }
    /// ```
    pub fn validate(&self, expected: &OrdinaryRetainedWorkerStopRequestV1) -> Result<(), String> {
        self.request.validate()?;
        expected.validate()?;
        if self.schema_version != 1 || &self.request != expected {
            return Err(
                "ordinary retained Stop acknowledgment changed schema or exact request".into(),
            );
        }
        validate_prefixed_uuid_v7("retirement_id", &self.retirement_id, "ormr_")
    }
}

impl TryFrom<OrdinaryRetainedWorkerStopAcknowledgmentV1Def>
    for OrdinaryRetainedWorkerStopAcknowledgmentV1
{
    type Error = String;
    fn try_from(value: OrdinaryRetainedWorkerStopAcknowledgmentV1Def) -> Result<Self, Self::Error> {
        let acknowledgment = Self {
            schema_version: value.schema_version,
            request: value.request,
            retirement_id: value.retirement_id,
        };
        acknowledgment.validate(&acknowledgment.request)?;
        Ok(acknowledgment)
    }
}

/// Positive owner observations that prevent parked-member retirement.
/// ```
/// use transport_api_types::OrdinaryRetainedWorkerStopBusyReasonV1;
/// assert!(serde_json::from_str::<OrdinaryRetainedWorkerStopBusyReasonV1>("\"missing\"").is_err());
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrdinaryRetainedWorkerStopBusyReasonV1 {
    Bootstrap,
    AcceptedTurn,
    NativeHistoryUse,
}

/// Only `Retired` carries a retirement acknowledgment. Unknown owner and busy
/// responses neither close lifecycle state nor release retained-worker capacity.
/// ```
/// use transport_api_types::OrdinaryRetainedWorkerStopResponseV1;
/// assert!(serde_json::from_str::<OrdinaryRetainedWorkerStopResponseV1>(r#"{"state":"unknown_owner","retired":true}"#).is_err());
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum OrdinaryRetainedWorkerStopResponseV1 {
    Retired {
        acknowledgment: Box<OrdinaryRetainedWorkerStopAcknowledgmentV1>,
    },
    NotQuiescent {
        reason: OrdinaryRetainedWorkerStopBusyReasonV1,
    },
    UnknownOwner {},
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ordinary_stop_request_codec_is_closed_and_generation_zero_is_valid() {
        let request = OrdinaryRetainedWorkerStopRequestV1 {
            schema_version: 1,
            stop_request_id: "req_stop_codec_only".into(),
            authority_store_id: "authority-codec-only".into(),
            orchestration_session_id: "session-codec-only".into(),
            caller_participant_id: "caller-codec-only".into(),
            target_participant_id: "target-codec-only".into(),
            target_backend_id: "cli:codex-world".into(),
            world_id: "world-codec-only".into(),
            world_generation: 0,
            registration_id: "registration-codec-only".into(),
            bootstrap_run_id: "bootstrap-codec-only".into(),
            native_session_id: "actual-native-locator-codec-only".into(),
            worker_cap_ref: DispatchPolicyCommitmentRefCarrierV1 {
                authority_store_id: "authority-codec-only".into(),
                commitment_id: "dpc_019c1e90-7cb4-7aa0-8aaa-aaaaaaaaaaaa".into(),
                exact_linkage_hash: "a".repeat(64),
            },
        };
        request.validate().unwrap();
        let encoded = serde_json::to_string(&request).unwrap();
        assert_eq!(
            serde_json::from_str::<OrdinaryRetainedWorkerStopRequestV1>(&encoded).unwrap(),
            request
        );
        for fault in ["unknown", "schema", "foreign-cap", "empty-native"] {
            let mut value = serde_json::to_value(&request).unwrap();
            match fault {
                "unknown" => {
                    value["bootstrap_exit_means_stopped"] = true.into();
                }
                "schema" => {
                    value["schema_version"] = 2.into();
                }
                "foreign-cap" => {
                    value["worker_cap_ref"]["authority_store_id"] = "foreign".into();
                }
                "empty-native" => {
                    value["native_session_id"] = "".into();
                }
                _ => unreachable!(),
            }
            assert!(
                serde_json::from_value::<OrdinaryRetainedWorkerStopRequestV1>(value).is_err(),
                "{fault}"
            );
        }
        assert!(
            serde_json::from_str::<OrdinaryRetainedWorkerStopRequestV1>(&encoded.replacen(
                "{",
                "{\"schema_version\":1,",
                1
            ))
            .is_err()
        );
    }

    #[test]
    fn test_ordinary_stop_acknowledgment_requires_exact_request_and_retirement_identity() {
        let request: OrdinaryRetainedWorkerStopRequestV1 = serde_json::from_value(serde_json::json!({
            "schema_version":1,"stop_request_id":"req_stop_codec_only","authority_store_id":"authority-codec-only",
            "orchestration_session_id":"session-codec-only","caller_participant_id":"caller-codec-only",
            "target_participant_id":"target-codec-only","target_backend_id":"cli:codex-world","world_id":"world-codec-only","world_generation":0,
            "registration_id":"registration-codec-only","bootstrap_run_id":"bootstrap-codec-only","native_session_id":"native-codec-only",
            "worker_cap_ref":{"authority_store_id":"authority-codec-only","commitment_id":"dpc_019c1e90-7cb4-7aa0-8aaa-aaaaaaaaaaaa","exact_linkage_hash":"a".repeat(64)}
        })).unwrap();
        let acknowledgment = OrdinaryRetainedWorkerStopAcknowledgmentV1 {
            schema_version: 1,
            request: request.clone(),
            retirement_id: "ormr_019c1e90-7cb4-7aa0-8aaa-aaaaaaaaaaaa".into(),
        };
        acknowledgment.validate(&request).unwrap();
        let mut foreign = request.clone();
        foreign.world_generation = 1;
        assert!(acknowledgment.validate(&foreign).is_err());
        let mut invalid = acknowledgment.clone();
        invalid.retirement_id = "not-an-owner-retirement-id".into();
        assert!(invalid.validate(&request).is_err());
        let response = OrdinaryRetainedWorkerStopResponseV1::Retired {
            acknowledgment: Box::new(acknowledgment),
        };
        assert_eq!(
            serde_json::from_slice::<OrdinaryRetainedWorkerStopResponseV1>(
                &serde_json::to_vec(&response).unwrap()
            )
            .unwrap(),
            response
        );
        let unknown = OrdinaryRetainedWorkerStopResponseV1::UnknownOwner {};
        assert_eq!(
            serde_json::from_slice::<OrdinaryRetainedWorkerStopResponseV1>(
                &serde_json::to_vec(&unknown).unwrap()
            )
            .unwrap(),
            unknown
        );
        for wire in [
            r#"{"state":"unknown_owner","retired":true}"#,
            r#"{"state":"not_quiescent","reason":"missing"}"#,
            r#"{"state":"retired","delivered":true}"#,
        ] {
            assert!(
                serde_json::from_str::<OrdinaryRetainedWorkerStopResponseV1>(wire).is_err(),
                "{wire}"
            );
        }
    }
}
