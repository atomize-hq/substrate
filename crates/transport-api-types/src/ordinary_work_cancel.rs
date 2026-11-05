//! Ordinary accepted-work cancellation references. Decoding supplies identity
//! data only: the service authenticates the actual Root/B1/E2/Supervisor lease,
//! current operation permission and its actual native owner before cancellation.
use crate::{validate_narrowing_identity, OrdinaryRetainedWorkerStopRequestV1};
use serde::{Deserialize, Serialize};

/// User Cancel and cleanup of an admitted Stop have separate operation grants.
/// Neither a caller label nor accepted-work history transfers control.
/// ```
/// use transport_api_types::OrdinaryAcceptedWorkCancelControlV1;
/// assert!(serde_json::from_str::<OrdinaryAcceptedWorkCancelControlV1>(r#"{"operation":"cancel_work","retired":true}"#).is_err());
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum OrdinaryAcceptedWorkCancelControlV1 {
    CancelWork {},
    StopWorker {
        stop: Box<OrdinaryRetainedWorkerStopRequestV1>,
    },
}

/// Exact delivery-lease reference, never a native terminal or retirement ACK.
/// Generation zero remains valid. The service joins every reference to original
/// held sources and its actual native handle; raw JSON grants no authority.
/// ```
/// use transport_api_types::OrdinaryAcceptedWorkCancelRequestV1;
/// assert!(serde_json::from_str::<OrdinaryAcceptedWorkCancelRequestV1>("{}").is_err());
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "OrdinaryAcceptedWorkCancelRequestV1Def")]
pub struct OrdinaryAcceptedWorkCancelRequestV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub orchestration_session_id: String,
    pub acceptance_record_id: String,
    pub cancel_request_id: String,
    pub delivery_claim_id: String,
    pub runtime_submission_id: String,
    pub target_backend_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub signal: String,
    pub control: OrdinaryAcceptedWorkCancelControlV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OrdinaryAcceptedWorkCancelRequestV1Def {
    schema_version: u32,
    authority_store_id: String,
    orchestration_session_id: String,
    acceptance_record_id: String,
    cancel_request_id: String,
    delivery_claim_id: String,
    runtime_submission_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    signal: String,
    control: OrdinaryAcceptedWorkCancelControlV1,
}

impl OrdinaryAcceptedWorkCancelRequestV1 {
    /// Representation only. Lease existence, caller/operation permission and
    /// actual native ownership require independent service checks.
    /// ```no_run
    /// # fn validate(data: &transport_api_types::OrdinaryAcceptedWorkCancelRequestV1) -> Result<(), String> {
    /// data.validate()
    /// # }
    /// ```
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 || !matches!(self.signal.as_str(), "INT" | "TERM") {
            return Err("ordinary accepted cancellation requires schema 1 and INT/TERM".into());
        }
        for (name, value) in [
            ("authority_store_id", &self.authority_store_id),
            ("orchestration_session_id", &self.orchestration_session_id),
            ("acceptance_record_id", &self.acceptance_record_id),
            ("cancel_request_id", &self.cancel_request_id),
            ("delivery_claim_id", &self.delivery_claim_id),
            ("runtime_submission_id", &self.runtime_submission_id),
            ("target_backend_id", &self.target_backend_id),
            ("world_id", &self.world_id),
        ] {
            validate_narrowing_identity(name, value)?;
        }
        if let OrdinaryAcceptedWorkCancelControlV1::StopWorker { stop } = &self.control {
            stop.validate()?;
            if stop.authority_store_id != self.authority_store_id
                || stop.orchestration_session_id != self.orchestration_session_id
                || stop.target_backend_id != self.target_backend_id
                || stop.world_id != self.world_id
                || stop.world_generation != self.world_generation
            {
                return Err("ordinary Stop cancellation changed original intent scope".into());
            }
        }
        Ok(())
    }
}

impl TryFrom<OrdinaryAcceptedWorkCancelRequestV1Def> for OrdinaryAcceptedWorkCancelRequestV1 {
    type Error = String;
    fn try_from(value: OrdinaryAcceptedWorkCancelRequestV1Def) -> Result<Self, Self::Error> {
        let request = Self {
            schema_version: value.schema_version,
            authority_store_id: value.authority_store_id,
            orchestration_session_id: value.orchestration_session_id,
            acceptance_record_id: value.acceptance_record_id,
            cancel_request_id: value.cancel_request_id,
            delivery_claim_id: value.delivery_claim_id,
            runtime_submission_id: value.runtime_submission_id,
            target_backend_id: value.target_backend_id,
            world_id: value.world_id,
            world_generation: value.world_generation,
            signal: value.signal,
            control: value.control,
        };
        request.validate()?;
        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ordinary_accepted_cancel_codec_is_closed_and_preserves_zero() {
        let data = OrdinaryAcceptedWorkCancelRequestV1 {
            schema_version: 1,
            authority_store_id: "authority-codec-only".into(),
            orchestration_session_id: "session-codec-only".into(),
            acceptance_record_id: "acceptance-codec-only".into(),
            cancel_request_id: "cancel-codec-only".into(),
            delivery_claim_id: "delivery-codec-only".into(),
            runtime_submission_id: "span-codec-only".into(),
            target_backend_id: "cli:codex-world".into(),
            world_id: "world-codec-only".into(),
            world_generation: 0,
            signal: "INT".into(),
            control: OrdinaryAcceptedWorkCancelControlV1::CancelWork {},
        };
        let encoded = serde_json::to_vec(&data).unwrap();
        assert_eq!(
            serde_json::from_slice::<OrdinaryAcceptedWorkCancelRequestV1>(&encoded).unwrap(),
            data
        );
        for fault in ["schema", "signal", "empty-lease", "extra", "extra-control"] {
            let mut value = serde_json::to_value(&data).unwrap();
            match fault {
                "schema" => value["schema_version"] = 2.into(),
                "signal" => value["signal"] = "KILL".into(),
                "empty-lease" => value["delivery_claim_id"] = "".into(),
                "extra" => value["native_terminal"] = true.into(),
                "extra-control" => value["control"]["can_retire"] = true.into(),
                _ => unreachable!(),
            }
            assert!(
                serde_json::from_value::<OrdinaryAcceptedWorkCancelRequestV1>(value).is_err(),
                "{fault}"
            );
        }
        let duplicate = String::from_utf8(encoded).unwrap().replacen(
            "\"signal\":\"INT\"",
            "\"signal\":\"INT\",\"signal\":\"TERM\"",
            1,
        );
        assert!(serde_json::from_str::<OrdinaryAcceptedWorkCancelRequestV1>(&duplicate).is_err());
    }
}

/// Delivery observation only; true is neither native terminal nor retirement.
/// ```
/// use transport_api_types::OrdinaryAcceptedWorkCancelResponseV1;
/// assert!(serde_json::from_str::<OrdinaryAcceptedWorkCancelResponseV1>(r#"{"schema_version":2,"delivered":true}"#).is_err());
/// assert!(serde_json::from_str::<OrdinaryAcceptedWorkCancelResponseV1>(r#"{"schema_version":1,"delivered":true,"terminal":true}"#).is_err());
/// ```
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "OrdinaryAcceptedWorkCancelResponseV1Def")]
pub struct OrdinaryAcceptedWorkCancelResponseV1 {
    pub schema_version: u32,
    pub delivered: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OrdinaryAcceptedWorkCancelResponseV1Def {
    schema_version: u32,
    delivered: bool,
}
impl TryFrom<OrdinaryAcceptedWorkCancelResponseV1Def> for OrdinaryAcceptedWorkCancelResponseV1 {
    type Error = String;
    fn try_from(value: OrdinaryAcceptedWorkCancelResponseV1Def) -> Result<Self, Self::Error> {
        if value.schema_version != 1 {
            return Err("unsupported ordinary cancellation response schema".into());
        }
        Ok(Self {
            schema_version: 1,
            delivered: value.delivered,
        })
    }
}
