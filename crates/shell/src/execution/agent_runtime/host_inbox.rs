use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::obligation_ledger::{OrchestrationObligationKind, OrchestrationObligationSeverity};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HostInboxMaterializationState {
    #[default]
    Pending,
    Materialized,
    FailedClosed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct HostInboxRecord {
    pub orchestration_session_id: String,
    pub record_id: String,
    pub kind: OrchestrationObligationKind,
    #[serde(default)]
    pub severity: OrchestrationObligationSeverity,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_participant_id: Option<String>,
    pub ingress_source_kind: String,
    pub ingress_source_id: String,
    pub ingress_received_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_host_id: Option<String>,
    pub target_host_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_backend_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
    #[serde(default)]
    pub materialization_state: HostInboxMaterializationState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub materialized_obligation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub materialized_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed_closed_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed_closed_reason: Option<String>,
}

impl HostInboxRecord {
    #[allow(dead_code)]
    pub(crate) fn new(
        orchestration_session_id: impl Into<String>,
        record_id: impl Into<String>,
        kind: OrchestrationObligationKind,
        summary: impl Into<String>,
        target_host_id: impl Into<String>,
        ingress_source_kind: impl Into<String>,
        ingress_source_id: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            orchestration_session_id: orchestration_session_id.into(),
            record_id: record_id.into(),
            kind,
            severity: OrchestrationObligationSeverity::Info,
            created_at: now,
            updated_at: now,
            summary: summary.into(),
            source_participant_id: None,
            ingress_source_kind: ingress_source_kind.into(),
            ingress_source_id: ingress_source_id.into(),
            ingress_received_at: now,
            origin_host_id: None,
            target_host_id: target_host_id.into(),
            causation_event_id: None,
            causation_message_id: None,
            causation_request_id: None,
            target_backend_id: None,
            payload: None,
            materialization_state: HostInboxMaterializationState::Pending,
            materialized_obligation_id: None,
            materialized_at: None,
            failed_closed_at: None,
            failed_closed_reason: None,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn validate(&self) -> Result<()> {
        validate_required_exact_identity(
            Some(self.orchestration_session_id.as_str()),
            "orchestration_session_id",
            "host inbox record",
        )?;
        validate_required_exact_identity(
            Some(self.record_id.as_str()),
            "record_id",
            "host inbox record",
        )?;
        validate_required_exact_identity(Some(self.summary.as_str()), "summary", "host inbox record")?;
        validate_optional_exact_identity(
            self.source_participant_id.as_deref(),
            "source_participant_id",
            "host inbox record",
        )?;
        validate_required_exact_identity(
            Some(self.ingress_source_kind.as_str()),
            "ingress_source_kind",
            "host inbox record",
        )?;
        validate_required_exact_identity(
            Some(self.ingress_source_id.as_str()),
            "ingress_source_id",
            "host inbox record",
        )?;
        validate_optional_host_id(
            self.origin_host_id.as_deref(),
            "origin_host_id",
            "host inbox record",
        )?;
        validate_required_host_id(
            Some(self.target_host_id.as_str()),
            "target_host_id",
            "host inbox record",
        )?;
        validate_optional_exact_identity(
            self.causation_event_id.as_deref(),
            "causation_event_id",
            "host inbox record",
        )?;
        validate_optional_exact_identity(
            self.causation_message_id.as_deref(),
            "causation_message_id",
            "host inbox record",
        )?;
        validate_optional_exact_identity(
            self.causation_request_id.as_deref(),
            "causation_request_id",
            "host inbox record",
        )?;
        validate_optional_exact_identity(
            self.target_backend_id.as_deref(),
            "target_backend_id",
            "host inbox record",
        )?;
        validate_optional_exact_identity(
            self.materialized_obligation_id.as_deref(),
            "materialized_obligation_id",
            "host inbox record",
        )?;
        validate_optional_non_empty(
            self.failed_closed_reason.as_deref(),
            "failed_closed_reason",
            "host inbox record",
        )?;

        match self.materialization_state {
            HostInboxMaterializationState::Pending => {
                if self.materialized_obligation_id.is_some()
                    || self.materialized_at.is_some()
                    || self.failed_closed_at.is_some()
                    || self.failed_closed_reason.is_some()
                {
                    anyhow::bail!(
                        "pending host inbox records must not persist materialization outcomes"
                    );
                }
            }
            HostInboxMaterializationState::Materialized => {
                if self.materialized_obligation_id.is_none() || self.materialized_at.is_none() {
                    anyhow::bail!(
                        "materialized host inbox records must include obligation linkage"
                    );
                }
                if self.failed_closed_at.is_some() || self.failed_closed_reason.is_some() {
                    anyhow::bail!(
                        "materialized host inbox records must not persist failed_closed truth"
                    );
                }
            }
            HostInboxMaterializationState::FailedClosed => {
                if self.failed_closed_at.is_none() || self.failed_closed_reason.is_none() {
                    anyhow::bail!(
                        "failed_closed host inbox records must include explanation-ready failure truth"
                    );
                }
                if self.materialized_obligation_id.is_some() || self.materialized_at.is_some() {
                    anyhow::bail!(
                        "failed_closed host inbox records must not persist materialized obligation truth"
                    );
                }
            }
        }

        Ok(())
    }
}

#[allow(dead_code)]
fn validate_required_host_id(
    host_id: Option<&str>,
    field_name: &str,
    artifact_kind: &str,
) -> Result<()> {
    let Some(host_id) = host_id else {
        anyhow::bail!("{artifact_kind} must include {field_name}");
    };
    if host_id.trim().is_empty() {
        anyhow::bail!("{artifact_kind} must not persist an empty {field_name}");
    }
    Ok(())
}

#[allow(dead_code)]
fn validate_optional_host_id(
    host_id: Option<&str>,
    field_name: &str,
    artifact_kind: &str,
) -> Result<()> {
    if let Some(host_id) = host_id {
        if host_id.trim().is_empty() {
            anyhow::bail!("{artifact_kind} must not persist an empty {field_name}");
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn validate_required_exact_identity(
    field_value: Option<&str>,
    field_name: &str,
    artifact_kind: &str,
) -> Result<()> {
    let Some(field_value) = field_value else {
        anyhow::bail!("{artifact_kind} must include {field_name}");
    };
    if field_value.trim().is_empty() {
        anyhow::bail!("{artifact_kind} must not persist an empty {field_name}");
    }
    Ok(())
}

#[allow(dead_code)]
fn validate_optional_exact_identity(
    field_value: Option<&str>,
    field_name: &str,
    artifact_kind: &str,
) -> Result<()> {
    if let Some(field_value) = field_value {
        if field_value.trim().is_empty() {
            anyhow::bail!("{artifact_kind} must not persist an empty {field_name}");
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn validate_optional_non_empty(
    field_value: Option<&str>,
    field_name: &str,
    artifact_kind: &str,
) -> Result<()> {
    if let Some(field_value) = field_value {
        if field_value.trim().is_empty() {
            anyhow::bail!("{artifact_kind} must not persist an empty {field_name}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    #[test]
    fn host_inbox_pending_record_validates_minimum_shape() {
        let record = HostInboxRecord::new(
            "sess_packet_one",
            "host_record_one",
            OrchestrationObligationKind::ApprovalRequired,
            "approval requested",
            "host-local",
            "remote_router",
            "ingress-001",
        );

        record
            .validate()
            .expect("pending host inbox record validates");
    }

    #[test]
    fn host_inbox_record_requires_exact_target_and_ingress_truth() {
        let mut record = HostInboxRecord::new(
            "sess_packet_one",
            "host_record_two",
            OrchestrationObligationKind::FollowUpRequired,
            "follow up required",
            "host-local",
            "remote_router",
            "ingress-002",
        );

        record.target_host_id = " ".to_string();
        let err = record
            .validate()
            .expect_err("blank target host must fail validation");
        assert!(err.to_string().contains("empty target_host_id"));

        record.target_host_id = "host-local".to_string();
        record.ingress_source_kind = "\t".to_string();
        let err = record
            .validate()
            .expect_err("blank ingress_source_kind must fail validation");
        assert!(err.to_string().contains("empty ingress_source_kind"));

        record.ingress_source_kind = "remote_router".to_string();
        record.ingress_source_id = " ".to_string();
        let err = record
            .validate()
            .expect_err("blank ingress_source_id must fail validation");
        assert!(err.to_string().contains("empty ingress_source_id"));
    }

    #[test]
    fn host_inbox_materialization_state_requires_matching_outcome_fields() {
        let now = Utc::now();
        let mut materialized = HostInboxRecord::new(
            "sess_packet_one",
            "host_record_three",
            OrchestrationObligationKind::Blocked,
            "blocked by approval",
            "host-local",
            "remote_router",
            "ingress-003",
        );
        materialized.materialization_state = HostInboxMaterializationState::Materialized;
        materialized.materialized_at = Some(now);

        let err = materialized
            .validate()
            .expect_err("materialized state requires obligation linkage");
        assert!(err
            .to_string()
            .contains("materialized host inbox records must include obligation linkage"));

        materialized.materialized_obligation_id = Some("obl-003".to_string());
        materialized
            .validate()
            .expect("materialized host inbox state validates");

        let mut failed_closed = HostInboxRecord::new(
            "sess_packet_one",
            "host_record_four",
            OrchestrationObligationKind::RuntimeAlert,
            "failed closed",
            "host-local",
            "remote_router",
            "ingress-004",
        );
        failed_closed.materialization_state = HostInboxMaterializationState::FailedClosed;
        failed_closed.failed_closed_at = Some(now + Duration::seconds(1));
        failed_closed.failed_closed_reason = Some("wrong_target_host".to_string());
        failed_closed
            .validate()
            .expect("failed closed host inbox state validates");

        failed_closed.materialized_at = Some(now + Duration::seconds(2));
        failed_closed.materialized_obligation_id = Some("obl-004".to_string());
        let err = failed_closed
            .validate()
            .expect_err("failed_closed state must reject materialized truth");
        assert!(err.to_string().contains(
            "failed_closed host inbox records must not persist materialized obligation truth"
        ));
    }
}
