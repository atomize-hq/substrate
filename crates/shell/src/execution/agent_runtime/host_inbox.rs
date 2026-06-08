use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

use super::obligation_ledger::{
    OrchestrationObligationAttachState, OrchestrationObligationKind, OrchestrationObligationRecord,
    OrchestrationObligationSeverity,
};

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
    #[serde(default)]
    pub orchestration_session_id: String,
    #[serde(default)]
    pub record_id: String,
    pub kind: OrchestrationObligationKind,
    #[serde(default)]
    pub severity: OrchestrationObligationSeverity,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_participant_id: Option<String>,
    #[serde(default)]
    pub ingress_source_kind: String,
    #[serde(default)]
    pub ingress_source_id: String,
    pub ingress_received_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_host_id: Option<String>,
    #[serde(default)]
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
    pub(crate) fn validate_record_id(record_id: &str) -> Result<()> {
        validate_host_inbox_record_id(record_id)
    }

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
        Self::validate_record_id(&self.record_id)?;
        validate_optional_exact_identity(
            self.source_participant_id.as_deref(),
            "source_participant_id",
            "host inbox record",
        )?;
        validate_optional_host_id(
            self.origin_host_id.as_deref(),
            "origin_host_id",
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
                validate_required_exact_identity(
                    Some(self.orchestration_session_id.as_str()),
                    "orchestration_session_id",
                    "host inbox record",
                )?;
                validate_required_exact_identity(
                    Some(self.summary.as_str()),
                    "summary",
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
                validate_required_host_id(
                    Some(self.target_host_id.as_str()),
                    "target_host_id",
                    "host inbox record",
                )?;
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
                validate_required_exact_identity(
                    Some(self.orchestration_session_id.as_str()),
                    "orchestration_session_id",
                    "host inbox record",
                )?;
                validate_required_exact_identity(
                    Some(self.summary.as_str()),
                    "summary",
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
                validate_required_host_id(
                    Some(self.target_host_id.as_str()),
                    "target_host_id",
                    "host inbox record",
                )?;
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
                validate_optional_exact_identity(
                    Some(self.orchestration_session_id.as_str())
                        .filter(|value| !value.is_empty()),
                    "orchestration_session_id",
                    "host inbox record",
                )?;
                validate_optional_exact_identity(
                    Some(self.summary.as_str()).filter(|value| !value.is_empty()),
                    "summary",
                    "host inbox record",
                )?;
                validate_optional_exact_identity(
                    Some(self.ingress_source_kind.as_str()).filter(|value| !value.is_empty()),
                    "ingress_source_kind",
                    "host inbox record",
                )?;
                validate_optional_exact_identity(
                    Some(self.ingress_source_id.as_str()).filter(|value| !value.is_empty()),
                    "ingress_source_id",
                    "host inbox record",
                )?;
                validate_optional_host_id(
                    Some(self.target_host_id.as_str()).filter(|value| !value.is_empty()),
                    "target_host_id",
                    "host inbox record",
                )?;
            }
        }

        Ok(())
    }

    pub(crate) fn local_obligation_id(&self) -> String {
        format!("host_inbox_{}", self.record_id)
    }

    pub(crate) fn ensure_pending_materialization_candidate(&self) -> Result<()> {
        self.validate()?;
        if self.materialization_state != HostInboxMaterializationState::Pending {
            anyhow::bail!(
                "host inbox materialization requires record {} to remain pending",
                self.record_id
            );
        }
        Ok(())
    }

    pub(crate) fn ensure_targets_local_host(&self, local_host_id: &str) -> Result<()> {
        validate_required_host_id(
            Some(local_host_id),
            "local_host_id",
            "host inbox materialization",
        )?;
        if self.target_host_id != local_host_id {
            anyhow::bail!(
                "wrong_target_host: host inbox record {} targets host {} not local host {}",
                self.record_id,
                self.target_host_id,
                local_host_id,
            );
        }
        Ok(())
    }

    pub(crate) fn materialize_as_local_obligation(
        &self,
        materialized_at: DateTime<Utc>,
    ) -> Result<OrchestrationObligationRecord> {
        self.ensure_pending_materialization_candidate()?;

        let mut obligation = OrchestrationObligationRecord::new(
            self.orchestration_session_id.clone(),
            self.local_obligation_id(),
            self.kind,
            self.summary.clone(),
        );
        obligation.severity = self.severity;
        obligation.attention_required = true;
        if self.kind.supports_router_auto_attach() {
            obligation.attach_state = OrchestrationObligationAttachState::Eligible;
        }
        obligation.created_at = materialized_at;
        obligation.updated_at = materialized_at;
        obligation.source_participant_id = self.source_participant_id.clone();
        obligation.ingress_source_kind = Some(self.ingress_source_kind.clone());
        obligation.ingress_source_id = Some(self.ingress_source_id.clone());
        obligation.ingress_received_at = Some(self.ingress_received_at);
        obligation.origin_host_id = self.origin_host_id.clone();
        obligation.target_host_id = Some(self.target_host_id.clone());
        obligation.causation_event_id = self.causation_event_id.clone();
        obligation.causation_message_id = self.causation_message_id.clone();
        obligation.causation_request_id = self.causation_request_id.clone();
        obligation.target_backend_id = self.target_backend_id.clone();
        obligation.payload = self.payload.clone();
        obligation.validate()?;

        Ok(obligation)
    }

    pub(crate) fn matches_materialized_obligation(
        &self,
        obligation: &OrchestrationObligationRecord,
    ) -> bool {
        obligation.orchestration_session_id == self.orchestration_session_id
            && obligation.obligation_id == self.local_obligation_id()
            && obligation.kind == self.kind
            && obligation.severity == self.severity
            && obligation.summary == self.summary
            && obligation.source_participant_id == self.source_participant_id
            && obligation.ingress_source_kind.as_deref() == Some(self.ingress_source_kind.as_str())
            && obligation.ingress_source_id.as_deref() == Some(self.ingress_source_id.as_str())
            && obligation.ingress_received_at == Some(self.ingress_received_at)
            && obligation.origin_host_id == self.origin_host_id
            && obligation.target_host_id.as_deref() == Some(self.target_host_id.as_str())
            && obligation.causation_event_id == self.causation_event_id
            && obligation.causation_message_id == self.causation_message_id
            && obligation.causation_request_id == self.causation_request_id
            && obligation.target_backend_id == self.target_backend_id
            && obligation.payload == self.payload
    }

    pub(crate) fn mark_materialized(
        &mut self,
        obligation_id: impl Into<String>,
        materialized_at: DateTime<Utc>,
    ) {
        self.materialization_state = HostInboxMaterializationState::Materialized;
        self.materialized_obligation_id = Some(obligation_id.into());
        self.materialized_at = Some(materialized_at);
        self.failed_closed_at = None;
        self.failed_closed_reason = None;
        self.updated_at = materialized_at;
    }

    pub(crate) fn mark_failed_closed(
        &mut self,
        failed_closed_reason: impl Into<String>,
        failed_closed_at: DateTime<Utc>,
    ) {
        self.materialization_state = HostInboxMaterializationState::FailedClosed;
        self.materialized_obligation_id = None;
        self.materialized_at = None;
        self.failed_closed_at = Some(failed_closed_at);
        self.failed_closed_reason = Some(failed_closed_reason.into());
        self.updated_at = failed_closed_at;
    }
}

fn validate_host_inbox_record_id(record_id: &str) -> Result<()> {
    validate_required_exact_identity(Some(record_id), "record_id", "host inbox record")?;

    if Path::new(record_id).is_absolute() || looks_like_windows_drive_qualified_path(record_id) {
        anyhow::bail!("host inbox record must not persist an absolute record_id");
    }

    if record_id.contains('/') || record_id.contains('\\') {
        anyhow::bail!("host inbox record must not persist path separators in record_id");
    }

    Ok(())
}

fn looks_like_windows_drive_qualified_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic()
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

    #[test]
    fn host_inbox_record_rejects_separator_and_absolute_record_ids() {
        let mut record = HostInboxRecord::new(
            "sess_packet_one",
            "host_record_five",
            OrchestrationObligationKind::ApprovalRequired,
            "approval requested",
            "host-local",
            "remote_router",
            "ingress-005",
        );

        record.record_id = "nested/record".to_string();
        let err = record
            .validate()
            .expect_err("record_id with separators must fail validation");
        assert!(err.to_string().contains("path separators in record_id"));

        record.record_id = "/tmp/record".to_string();
        let err = record
            .validate()
            .expect_err("absolute record_id must fail validation");
        assert!(err.to_string().contains("absolute record_id"));

        record.record_id = r"C:\tmp\record".to_string();
        let err = record
            .validate()
            .expect_err("windows absolute record_id must fail validation");
        assert!(err.to_string().contains("absolute record_id"));

        record.record_id = "C:host_record".to_string();
        let err = record
            .validate()
            .expect_err("windows drive-relative record_id must fail validation");
        assert!(err.to_string().contains("absolute record_id"));
    }

    #[test]
    fn host_inbox_record_materializes_local_obligation_with_preserved_envelope_truth() {
        let materialized_at = Utc::now();
        let mut record = HostInboxRecord::new(
            "sess_packet_two",
            "host_record_six",
            OrchestrationObligationKind::ApprovalRequired,
            "approval requested",
            "host-local",
            "remote_router",
            "ingress-006",
        );
        record.source_participant_id = Some("worker-007".to_string());
        record.origin_host_id = Some("host-origin".to_string());
        record.causation_event_id = Some("evt-006".to_string());
        record.causation_message_id = Some("msg-006".to_string());
        record.causation_request_id = Some("req-006".to_string());
        record.target_backend_id = Some("cli:codex_world".to_string());
        record.payload = Some(serde_json::json!({
            "event_class": "approval_request",
        }));

        let obligation = record
            .materialize_as_local_obligation(materialized_at)
            .expect("pending host inbox record should materialize");

        assert_eq!(obligation.obligation_id, "host_inbox_host_record_six");
        assert!(record.matches_materialized_obligation(&obligation));
        assert_eq!(obligation.created_at, materialized_at);
        assert_eq!(obligation.updated_at, materialized_at);
        assert!(obligation.attention_required);
        assert_eq!(
            obligation.attach_state,
            OrchestrationObligationAttachState::Eligible
        );
    }

    #[test]
    fn host_inbox_record_tracks_materialized_and_failed_closed_outcomes() {
        let now = Utc::now();
        let mut record = HostInboxRecord::new(
            "sess_packet_two",
            "host_record_seven",
            OrchestrationObligationKind::RuntimeAlert,
            "runtime alert",
            "host-local",
            "remote_router",
            "ingress-007",
        );

        record.mark_materialized("host_inbox_host_record_seven", now);
        record
            .validate()
            .expect("materialized host inbox record remains valid");
        assert_eq!(
            record.materialized_obligation_id.as_deref(),
            Some("host_inbox_host_record_seven")
        );

        record.mark_failed_closed("wrong_target_host", now + Duration::seconds(1));
        record
            .validate()
            .expect("failed closed host inbox record remains valid");
        assert_eq!(
            record.failed_closed_reason.as_deref(),
            Some("wrong_target_host")
        );
        assert!(record.materialized_obligation_id.is_none());
        assert!(record.materialized_at.is_none());

        record.orchestration_session_id.clear();
        record.summary.clear();
        record.ingress_source_kind.clear();
        record.ingress_source_id.clear();
        record.target_host_id.clear();
        record
            .validate()
            .expect("failed closed host inbox record may preserve missing exact truth");
    }
}
