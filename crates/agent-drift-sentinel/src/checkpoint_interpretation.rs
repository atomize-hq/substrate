use agent_drift_analyzer::{Checkpoint, DelegationContext, DriftClass, DriftState, EvidenceRef};
use serde_json::Value;

use crate::input::CheckpointCursor;
use crate::operator_surface::CheckpointPosture;

const SUPPORTED_SCHEMA_DESCRIPTION: &str = "v0.2, v0.3, v0.4, v0.5, v0.6, v0.7, or v0.8";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckpointSchemaVersion {
    V0_2,
    V0_3,
    V0_4,
    V0_5,
    V0_6,
    V0_7,
    V0_8,
}

impl CheckpointSchemaVersion {
    fn from_literal(
        schema_version: &str,
        checkpoint_id: &str,
    ) -> Result<Self, CheckpointContractError> {
        match schema_version {
            "v0.2" => Ok(Self::V0_2),
            "v0.3" => Ok(Self::V0_3),
            "v0.4" => Ok(Self::V0_4),
            "v0.5" => Ok(Self::V0_5),
            "v0.6" => Ok(Self::V0_6),
            "v0.7" => Ok(Self::V0_7),
            "v0.8" => Ok(Self::V0_8),
            _ => Err(CheckpointContractError::UnsupportedSchema {
                checkpoint_id: checkpoint_id.to_string(),
                schema_version: schema_version.to_string(),
                expected: SUPPORTED_SCHEMA_DESCRIPTION,
            }),
        }
    }

    fn uses_explicit_state(self) -> bool {
        !matches!(self, Self::V0_2)
    }

    fn requires_turn_context(self) -> bool {
        matches!(
            self,
            Self::V0_4 | Self::V0_5 | Self::V0_6 | Self::V0_7 | Self::V0_8
        )
    }

    fn requires_session_archetype(self) -> bool {
        matches!(self, Self::V0_5 | Self::V0_6 | Self::V0_7 | Self::V0_8)
    }

    fn requires_session_progress(self) -> bool {
        matches!(self, Self::V0_6 | Self::V0_7 | Self::V0_8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckpointProjectionProfile {
    Schema(CheckpointSchemaVersion),
    CompatibilityOnly,
}

impl CheckpointProjectionProfile {
    fn uses_explicit_state(self) -> bool {
        matches!(self, Self::Schema(schema) if schema.uses_explicit_state())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CheckpointInterpretationInput<'a> {
    pub(crate) checkpoint: &'a Checkpoint,
    pub(crate) previous_same_session: Option<&'a Checkpoint>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CheckpointCompatibilityProjectionInput<'a> {
    pub(crate) checkpoint: &'a Checkpoint,
    pub(crate) previous_checkpoint: Option<&'a Checkpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckpointInterpretation {
    pub(crate) checkpoint: Checkpoint,
    pub(crate) projection_profile: CheckpointProjectionProfile,
    pub(crate) cursor: CheckpointCursor,
    pub(crate) warning_fingerprint: String,
    pub(crate) flagged: bool,
    pub(crate) max_flagged_score: Option<u8>,
    pub(crate) posture: Option<CheckpointPosture>,
    pub(crate) evidence: Vec<EvidenceRef>,
    pub(crate) evidence_groups: Vec<Vec<EvidenceRef>>,
    pub(crate) delegation: Option<DelegationContext>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum CheckpointContractError {
    #[error(
        "checkpoint {checkpoint_id} uses unsupported schema version {schema_version}; expected {expected}"
    )]
    UnsupportedSchema {
        checkpoint_id: String,
        schema_version: String,
        expected: &'static str,
    },
    #[error(
        "checkpoint {checkpoint_id} with schema {schema_version} has invalid field {field}: {reason}"
    )]
    FieldGap {
        checkpoint_id: String,
        schema_version: String,
        field: String,
        reason: String,
    },
    #[error(
        "checkpoint {checkpoint_id} in session {session_id} cannot use previous checkpoint {previous_checkpoint_id} from session {previous_session_id}"
    )]
    CrossSessionHistory {
        checkpoint_id: String,
        session_id: String,
        previous_checkpoint_id: String,
        previous_session_id: String,
    },
}

pub(crate) fn validate_serialized_checkpoint(
    value: &Value,
) -> Result<CheckpointSchemaVersion, CheckpointContractError> {
    let checkpoint_id = value
        .get("checkpoint_id")
        .and_then(Value::as_str)
        .unwrap_or("<unknown>");
    let Some(schema_version) = value.get("schema_version").and_then(Value::as_str) else {
        return Err(field_gap(
            checkpoint_id,
            "<unknown>",
            "schema_version",
            "must be a supported schema-version string",
        ));
    };
    let schema = CheckpointSchemaVersion::from_literal(schema_version, checkpoint_id)?;

    for (field, path) in [
        ("session_id", &["session_id"][..]),
        ("checkpoint_id", &["checkpoint_id"][..]),
        ("task_frame.objective", &["task_frame", "objective"][..]),
        ("expected_next_step", &["expected_next_step"][..]),
    ] {
        require_non_empty_string(value, path, checkpoint_id, schema_version, field)?;
    }

    if schema.requires_turn_context() {
        require_object(value, "turn_context", checkpoint_id, schema_version)?;
    }
    if schema.requires_session_archetype() {
        require_object(value, "session_archetype", checkpoint_id, schema_version)?;
    }
    if schema.requires_session_progress() {
        require_object(value, "session_progress", checkpoint_id, schema_version)?;
    }
    if matches!(schema, CheckpointSchemaVersion::V0_8) {
        require_object(value, "delegation", checkpoint_id, schema_version)?;
    }
    if schema.uses_explicit_state() {
        require_explicit_drift_states(value, checkpoint_id, schema_version)?;
    }

    Ok(schema)
}

pub(crate) fn interpret_checkpoint(
    input: CheckpointInterpretationInput<'_>,
) -> Result<CheckpointInterpretation, CheckpointContractError> {
    let checkpoint = input.checkpoint;
    let schema = CheckpointSchemaVersion::from_literal(
        checkpoint.schema_version.as_str(),
        checkpoint.checkpoint_id.as_str(),
    )?;
    validate_typed_checkpoint(checkpoint, schema)?;

    if let Some(previous) = input.previous_same_session {
        if previous.session_id != checkpoint.session_id {
            return Err(CheckpointContractError::CrossSessionHistory {
                checkpoint_id: checkpoint.checkpoint_id.clone(),
                session_id: checkpoint.session_id.clone(),
                previous_checkpoint_id: previous.checkpoint_id.clone(),
                previous_session_id: previous.session_id.clone(),
            });
        }
        if previous.schema_version != checkpoint.schema_version {
            let reason = format!(
                "same-session history must use schema {}; previous checkpoint {} uses {}",
                checkpoint.schema_version, previous.checkpoint_id, previous.schema_version
            );
            return Err(field_gap(
                checkpoint.checkpoint_id.as_str(),
                checkpoint.schema_version.as_str(),
                "previous.schema_version",
                reason.as_str(),
            ));
        }
    }

    Ok(project_checkpoint(
        checkpoint,
        input.previous_same_session,
        CheckpointProjectionProfile::Schema(schema),
    ))
}

pub(crate) fn project_checkpoint_compatibility(
    input: CheckpointCompatibilityProjectionInput<'_>,
) -> CheckpointInterpretation {
    let profile = compatibility_projection_profile(input.checkpoint.schema_version.as_str());
    project_checkpoint(input.checkpoint, input.previous_checkpoint, profile)
}

fn compatibility_projection_profile(schema_version: &str) -> CheckpointProjectionProfile {
    let schema = match schema_version {
        "v0.2" => CheckpointSchemaVersion::V0_2,
        "v0.3" => CheckpointSchemaVersion::V0_3,
        "v0.4" => CheckpointSchemaVersion::V0_4,
        "v0.5" => CheckpointSchemaVersion::V0_5,
        "v0.6" => CheckpointSchemaVersion::V0_6,
        "v0.7" => CheckpointSchemaVersion::V0_7,
        "v0.8" => CheckpointSchemaVersion::V0_8,
        _ => return CheckpointProjectionProfile::CompatibilityOnly,
    };
    CheckpointProjectionProfile::Schema(schema)
}

fn project_checkpoint(
    checkpoint: &Checkpoint,
    previous_same_session: Option<&Checkpoint>,
    profile: CheckpointProjectionProfile,
) -> CheckpointInterpretation {
    let evidence_groups = checkpoint_evidence(checkpoint, profile);
    let mut evidence = Vec::new();
    for group in &evidence_groups {
        append_unique(&mut evidence, group);
    }

    CheckpointInterpretation {
        checkpoint: checkpoint.clone(),
        projection_profile: profile,
        cursor: CheckpointCursor::from(checkpoint),
        warning_fingerprint: warning_fingerprint(checkpoint),
        flagged: checkpoint.flagged,
        max_flagged_score: checkpoint
            .drift_scores
            .iter()
            .filter(|score| score.flagged)
            .map(|score| score.raw_score)
            .max(),
        posture: checkpoint_posture(checkpoint, previous_same_session, profile),
        evidence,
        evidence_groups,
        delegation: matches!(
            profile,
            CheckpointProjectionProfile::Schema(CheckpointSchemaVersion::V0_8)
        )
        .then(|| checkpoint.delegation.clone()),
    }
}

fn validate_typed_checkpoint(
    checkpoint: &Checkpoint,
    schema: CheckpointSchemaVersion,
) -> Result<(), CheckpointContractError> {
    for (field, value) in [
        ("session_id", checkpoint.session_id.as_str()),
        ("checkpoint_id", checkpoint.checkpoint_id.as_str()),
        (
            "task_frame.objective",
            checkpoint.task_frame.objective.as_str(),
        ),
        ("expected_next_step", checkpoint.expected_next_step.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(field_gap(
                checkpoint.checkpoint_id.as_str(),
                checkpoint.schema_version.as_str(),
                field,
                "must be a non-empty string",
            ));
        }
    }

    for (required, present) in [
        (
            "turn_context",
            !schema.requires_turn_context() || checkpoint.turn_context.is_some(),
        ),
        (
            "session_archetype",
            !schema.requires_session_archetype() || checkpoint.session_archetype.is_some(),
        ),
        (
            "session_progress",
            !schema.requires_session_progress() || checkpoint.session_progress.is_some(),
        ),
    ] {
        if !present {
            return Err(field_gap(
                checkpoint.checkpoint_id.as_str(),
                checkpoint.schema_version.as_str(),
                required,
                "must be present for this schema version",
            ));
        }
    }

    Ok(())
}

fn require_non_empty_string(
    value: &Value,
    path: &[&str],
    checkpoint_id: &str,
    schema_version: &str,
    field: &str,
) -> Result<(), CheckpointContractError> {
    let field_value = path
        .iter()
        .try_fold(value, |current, segment| current.get(*segment))
        .and_then(Value::as_str);
    if field_value.is_some_and(|text| !text.trim().is_empty()) {
        return Ok(());
    }

    Err(field_gap(
        checkpoint_id,
        schema_version,
        field,
        "must be a non-empty string",
    ))
}

fn require_object(
    value: &Value,
    field: &str,
    checkpoint_id: &str,
    schema_version: &str,
) -> Result<(), CheckpointContractError> {
    if value.get(field).is_some_and(Value::is_object) {
        return Ok(());
    }

    Err(field_gap(
        checkpoint_id,
        schema_version,
        field,
        "must serialize a non-null object for this schema version",
    ))
}

fn require_explicit_drift_states(
    value: &Value,
    checkpoint_id: &str,
    schema_version: &str,
) -> Result<(), CheckpointContractError> {
    let Some(scores) = value.get("drift_scores").and_then(Value::as_array) else {
        return Err(field_gap(
            checkpoint_id,
            schema_version,
            "drift_scores",
            "must serialize an array with explicit analyzer state",
        ));
    };

    for (index, score) in scores.iter().enumerate() {
        let field = format!("drift_scores.{index}.state");
        let valid_state = score
            .get("state")
            .and_then(Value::as_str)
            .is_some_and(|state| {
                matches!(
                    state,
                    "active" | "recovered" | "historical_only" | "cleared"
                )
            });
        if !valid_state {
            return Err(field_gap(
                checkpoint_id,
                schema_version,
                field.as_str(),
                "must serialize an explicit analyzer DriftState",
            ));
        }
    }
    Ok(())
}

fn checkpoint_posture(
    checkpoint: &Checkpoint,
    previous_same_session: Option<&Checkpoint>,
    profile: CheckpointProjectionProfile,
) -> Option<CheckpointPosture> {
    if profile.uses_explicit_state() {
        return [
            (DriftState::Active, CheckpointPosture::Active),
            (DriftState::Recovered, CheckpointPosture::Recovered),
            (
                DriftState::HistoricalOnly,
                CheckpointPosture::HistoricalOnly,
            ),
        ]
        .into_iter()
        .find_map(|(state, posture)| {
            checkpoint
                .drift_scores
                .iter()
                .any(|score| score.state == state)
                .then_some(posture)
        });
    }

    if checkpoint.flagged || checkpoint.drift_scores.iter().any(|score| score.flagged) {
        return Some(CheckpointPosture::Active);
    }
    let historical_classes = checkpoint
        .drift_scores
        .iter()
        .filter(|score| has_legacy_historical_evidence(score.class, &score.evidence))
        .map(|score| score.class)
        .collect::<Vec<_>>();
    if historical_classes.is_empty() {
        return None;
    }
    if previous_same_session.is_some_and(|previous| {
        historical_classes.iter().any(|class| {
            previous
                .drift_scores
                .iter()
                .any(|score| score.class == *class && score.flagged)
        })
    }) {
        return Some(CheckpointPosture::Recovered);
    }
    Some(CheckpointPosture::HistoricalOnly)
}

fn checkpoint_evidence(
    checkpoint: &Checkpoint,
    profile: CheckpointProjectionProfile,
) -> Vec<Vec<EvidenceRef>> {
    let mut selected = Vec::new();
    if profile.uses_explicit_state() {
        for state in [
            DriftState::Active,
            DriftState::Recovered,
            DriftState::HistoricalOnly,
        ] {
            for score in checkpoint
                .drift_scores
                .iter()
                .filter(|score| score.state == state)
            {
                selected.push(score.evidence.clone());
            }
        }
        return selected;
    }

    for score in checkpoint.drift_scores.iter().filter(|score| score.flagged) {
        selected.push(score.evidence.clone());
    }
    for score in checkpoint.drift_scores.iter().filter(|score| {
        !score.flagged && has_legacy_historical_evidence(score.class, &score.evidence)
    }) {
        selected.push(score.evidence.clone());
    }
    selected
}

fn append_unique(selected: &mut Vec<EvidenceRef>, evidence: &[EvidenceRef]) {
    for item in evidence {
        if !selected.contains(item) {
            selected.push(item.clone());
        }
    }
}

fn has_legacy_historical_evidence(class: DriftClass, evidence: &[EvidenceRef]) -> bool {
    let prefixes: &[&str] = match class {
        DriftClass::TruthGroundingGap => &["historical truth-grounding gap:"],
        DriftClass::DeadEndThrash => &[
            "historical repeated failure evidence:",
            "historical repeated verification evidence:",
        ],
        DriftClass::WrongPlanBranch | DriftClass::SemanticGoalDrift => &[],
    };
    prefixes
        .iter()
        .any(|prefix| evidence.iter().any(|item| item.reason.starts_with(prefix)))
}

fn warning_fingerprint(checkpoint: &Checkpoint) -> String {
    let classes = checkpoint
        .drift_scores
        .iter()
        .filter(|score| score.flagged)
        .map(|score| match score.class {
            DriftClass::WrongPlanBranch => "wrong_plan_branch",
            DriftClass::TruthGroundingGap => "truth_grounding_gap",
            DriftClass::DeadEndThrash => "dead_end_thrash",
            DriftClass::SemanticGoalDrift => "semantic_goal_drift",
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{}:{}:{}",
        checkpoint.session_id, classes, checkpoint.expected_next_step
    )
}

fn field_gap(
    checkpoint_id: &str,
    schema_version: &str,
    field: &str,
    reason: &str,
) -> CheckpointContractError {
    CheckpointContractError::FieldGap {
        checkpoint_id: checkpoint_id.to_string(),
        schema_version: schema_version.to_string(),
        field: field.to_string(),
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use agent_drift_analyzer::{
        checkpoint::CheckpointDiagnostics, Checkpoint, CheckpointBoundary, ChildWorkVisibility,
        Confidence, DelegationContext, DelegationTopology, DriftClass, DriftScore, DriftState,
        EvidenceRef, ProgressDimension, ProgressStatus, SessionArchetype, SessionArchetypeLabel,
        SessionProgress, TaskFrame, TurnActivityMix, TurnContext, TurnExecutionMode,
    };
    use agent_session_compactor::RowRef;
    use camino::Utf8PathBuf;
    use serde_json::{json, Value};

    use super::{
        interpret_checkpoint, project_checkpoint_compatibility, validate_serialized_checkpoint,
        CheckpointCompatibilityProjectionInput, CheckpointContractError,
        CheckpointInterpretationInput, CheckpointProjectionProfile, CheckpointSchemaVersion,
    };
    use crate::operator_surface::CheckpointPosture;

    const SUPPORTED: [(&str, CheckpointSchemaVersion); 7] = [
        ("v0.2", CheckpointSchemaVersion::V0_2),
        ("v0.3", CheckpointSchemaVersion::V0_3),
        ("v0.4", CheckpointSchemaVersion::V0_4),
        ("v0.5", CheckpointSchemaVersion::V0_5),
        ("v0.6", CheckpointSchemaVersion::V0_6),
        ("v0.7", CheckpointSchemaVersion::V0_7),
        ("v0.8", CheckpointSchemaVersion::V0_8),
    ];

    fn evidence(reason: &str) -> EvidenceRef {
        EvidenceRef {
            row: RowRef {
                source_file: Utf8PathBuf::from("/tmp/session.jsonl"),
                event_index: 1,
                row_ordinal: 0,
            },
            reason: reason.to_string(),
        }
    }

    fn turn_context() -> TurnContext {
        TurnContext {
            turn_id: Some("turn-1".to_string()),
            turn_ordinal: 1,
            rows_since_turn_start: 1,
            seconds_since_turn_start: Some(1),
            checkpoints_in_turn: 1,
            prompts_observed_in_session: 1,
            execution_mode: TurnExecutionMode::Autonomous,
            activity_mix: TurnActivityMix::default(),
        }
    }

    fn checkpoint_for_version(version: &str) -> Checkpoint {
        let row = evidence("boundary evidence").row;
        let mut checkpoint = Checkpoint {
            schema_version: version.to_string(),
            session_id: "session-a".to_string(),
            checkpoint_id: "session-a:0001".to_string(),
            ordinal: 1,
            boundary: CheckpointBoundary {
                start: row.clone(),
                end: row,
            },
            turn_context: None,
            diagnostics: CheckpointDiagnostics::default(),
            task_frame: TaskFrame {
                objective: "implement the selected packet".to_string(),
                confidence: Confidence::High,
                truth_artifacts: Vec::new(),
                working_set_paths: Vec::new(),
                tools: Vec::new(),
                command_families: Vec::new(),
                verification_commands: Vec::new(),
                supporting_evidence: Vec::new(),
                counter_evidence: Vec::new(),
            },
            structured_objective: None,
            session_archetype: None,
            session_progress: None,
            delegation: DelegationContext::default(),
            drift_scores: vec![DriftScore {
                class: DriftClass::DeadEndThrash,
                state: DriftState::Cleared,
                raw_score: 20,
                confidence: Confidence::High,
                flagged: false,
                evidence: Vec::new(),
            }],
            expected_next_step: "run focused proof".to_string(),
            flagged: false,
        };

        if matches!(version, "v0.4" | "v0.5" | "v0.6" | "v0.7" | "v0.8") {
            checkpoint.turn_context = Some(turn_context());
        }
        if matches!(version, "v0.5" | "v0.6" | "v0.7" | "v0.8") {
            checkpoint.session_archetype = Some(SessionArchetype {
                label: SessionArchetypeLabel::AutonomousImplementation,
                confidence: Confidence::High,
                supporting_evidence: Vec::new(),
                counter_evidence: Vec::new(),
            });
        }
        if matches!(version, "v0.6" | "v0.7" | "v0.8") {
            checkpoint.session_progress = Some(SessionProgress {
                status: ProgressStatus::Advancing,
                dimension: ProgressDimension::ImplementationVerificationWall,
                confidence: Confidence::High,
                signals: Vec::new(),
                supporting_evidence: Vec::new(),
                counter_evidence: Vec::new(),
            });
        }
        checkpoint
    }

    fn serialized_checkpoint(version: &str) -> Value {
        serde_json::to_value(checkpoint_for_version(version))
            .expect("test checkpoint should serialize")
    }

    fn interpret<'a>(
        checkpoint: &'a Checkpoint,
        previous_same_session: Option<&'a Checkpoint>,
    ) -> Result<super::CheckpointInterpretation, CheckpointContractError> {
        interpret_checkpoint(CheckpointInterpretationInput {
            checkpoint,
            previous_same_session,
        })
    }

    #[test]
    fn serialized_schema_matrix_is_the_exact_literal_v0_2_through_v0_8_set() {
        for (literal, expected) in SUPPORTED {
            assert_eq!(
                validate_serialized_checkpoint(&serialized_checkpoint(literal)),
                Ok(expected)
            );
        }

        for unsupported in ["v0.1", "v0.9", "0.8", "v1.0"] {
            let mut value = serialized_checkpoint("v0.8");
            value["schema_version"] = json!(unsupported);
            assert!(matches!(
                validate_serialized_checkpoint(&value),
                Err(CheckpointContractError::UnsupportedSchema { schema_version, .. })
                    if schema_version == unsupported
            ));
        }
    }

    #[test]
    fn explicit_state_and_version_specific_serialized_fields_fail_closed() {
        for version in ["v0.3", "v0.4", "v0.5", "v0.6", "v0.7", "v0.8"] {
            let mut value = serialized_checkpoint(version);
            value["drift_scores"][0]["state"] = Value::Null;
            assert!(matches!(
                validate_serialized_checkpoint(&value),
                Err(CheckpointContractError::FieldGap {
                    schema_version,
                    field,
                    ..
                }) if schema_version == version && field == "drift_scores.0.state"
            ));
        }

        for (version, pointer) in [
            ("v0.4", "/turn_context"),
            ("v0.5", "/turn_context"),
            ("v0.5", "/session_archetype"),
            ("v0.6", "/turn_context"),
            ("v0.6", "/session_archetype"),
            ("v0.6", "/session_progress"),
            ("v0.7", "/turn_context"),
            ("v0.7", "/session_archetype"),
            ("v0.7", "/session_progress"),
            ("v0.8", "/turn_context"),
            ("v0.8", "/session_archetype"),
            ("v0.8", "/session_progress"),
            ("v0.8", "/delegation"),
        ] {
            let mut value = serialized_checkpoint(version);
            *value
                .pointer_mut(pointer)
                .expect("required test field should exist") = Value::Null;
            assert!(matches!(
                validate_serialized_checkpoint(&value),
                Err(CheckpointContractError::FieldGap {
                    schema_version,
                    field,
                    ..
                }) if schema_version == version && field == pointer.trim_start_matches('/').replace('/', ".")
            ));
        }

        let mut missing = serialized_checkpoint("v0.4");
        missing
            .as_object_mut()
            .expect("serialized checkpoint should be an object")
            .remove("turn_context");
        assert!(matches!(
            validate_serialized_checkpoint(&missing),
            Err(CheckpointContractError::FieldGap { field, .. }) if field == "turn_context"
        ));

        let mut malformed = serialized_checkpoint("v0.8");
        malformed["delegation"] = json!("not an object");
        assert!(matches!(
            validate_serialized_checkpoint(&malformed),
            Err(CheckpointContractError::FieldGap { field, .. }) if field == "delegation"
        ));

        for (version, missing_field) in [
            ("v0.4", "turn_context"),
            ("v0.5", "session_archetype"),
            ("v0.6", "session_progress"),
            ("v0.7", "session_progress"),
            ("v0.8", "session_progress"),
        ] {
            let mut checkpoint = checkpoint_for_version(version);
            match missing_field {
                "turn_context" => checkpoint.turn_context = None,
                "session_archetype" => checkpoint.session_archetype = None,
                "session_progress" => checkpoint.session_progress = None,
                _ => unreachable!("test matrix contains only known fields"),
            }
            assert!(matches!(
                interpret(&checkpoint, None),
                Err(CheckpointContractError::FieldGap { field, .. }) if field == missing_field
            ));
        }
    }

    #[test]
    fn sentinel_owned_non_empty_validation_is_exactly_four_fields() {
        for pointer in [
            "/session_id",
            "/checkpoint_id",
            "/task_frame/objective",
            "/expected_next_step",
        ] {
            let mut value = serialized_checkpoint("v0.8");
            *value
                .pointer_mut(pointer)
                .expect("required test field should exist") = json!(" \t\n ");
            assert!(matches!(
                validate_serialized_checkpoint(&value),
                Err(CheckpointContractError::FieldGap { field, .. })
                    if field == pointer.trim_start_matches('/').replace('/', ".")
            ));

            let mut checkpoint = checkpoint_for_version("v0.8");
            let field = pointer.trim_start_matches('/').replace('/', ".");
            match field.as_str() {
                "session_id" => checkpoint.session_id = " \t\n ".to_string(),
                "checkpoint_id" => checkpoint.checkpoint_id = " \t\n ".to_string(),
                "task_frame.objective" => checkpoint.task_frame.objective = " \t\n ".to_string(),
                "expected_next_step" => checkpoint.expected_next_step = " \t\n ".to_string(),
                _ => unreachable!("test matrix contains only known fields"),
            }
            assert!(matches!(
                interpret(&checkpoint, None),
                Err(CheckpointContractError::FieldGap { field: actual, .. }) if actual == field
            ));
        }

        let mut value = serialized_checkpoint("v0.8");
        value["task_frame"]["truth_artifacts"] = json!([""]);
        value["delegation"]["markers"] = json!([""]);
        value["delegation"]["child_session_ids"] = json!([""]);
        value["drift_scores"][0]["evidence"] = json!([{
            "row": evidence("").row,
            "reason": ""
        }]);
        assert_eq!(
            validate_serialized_checkpoint(&value),
            Ok(CheckpointSchemaVersion::V0_8)
        );
    }

    #[test]
    fn v0_2_legacy_posture_is_bounded_to_flag_and_historical_evidence() {
        let mut active = checkpoint_for_version("v0.2");
        active.flagged = true;
        active.drift_scores[0].flagged = true;
        active.drift_scores[0].state = DriftState::Cleared;
        assert_eq!(
            interpret(&active, None)
                .expect("v0.2 active should interpret")
                .posture,
            Some(CheckpointPosture::Active)
        );

        let mut historical = checkpoint_for_version("v0.2");
        historical.ordinal = 2;
        historical.checkpoint_id = "session-a:0002".to_string();
        historical.drift_scores[0].evidence = vec![evidence(
            "historical repeated failure evidence: prior command failed",
        )];
        assert_eq!(
            interpret(&historical, None)
                .expect("v0.2 history should interpret")
                .posture,
            Some(CheckpointPosture::HistoricalOnly)
        );
        assert_eq!(
            interpret(&historical, Some(&active))
                .expect("same-session v0.2 history should interpret")
                .posture,
            Some(CheckpointPosture::Recovered)
        );

        let mut unrelated = checkpoint_for_version("v0.2");
        unrelated.drift_scores[0].evidence = vec![evidence("spawn_agent result available")];
        assert_eq!(
            interpret(&unrelated, None)
                .expect("unrelated evidence should interpret")
                .posture,
            None
        );
    }

    #[test]
    fn history_from_another_session_is_a_structured_contract_error() {
        let checkpoint = checkpoint_for_version("v0.2");
        let mut previous = checkpoint_for_version("v0.2");
        previous.session_id = "session-b".to_string();
        previous.checkpoint_id = "session-b:0001".to_string();

        assert!(matches!(
            interpret(&checkpoint, Some(&previous)),
            Err(CheckpointContractError::CrossSessionHistory {
                checkpoint_id,
                previous_checkpoint_id,
                ..
            }) if checkpoint_id == "session-a:0001" && previous_checkpoint_id == "session-b:0001"
        ));

        let mut conflicting_schema = checkpoint_for_version("v0.3");
        conflicting_schema.session_id = checkpoint.session_id.clone();
        assert!(matches!(
            interpret(&checkpoint, Some(&conflicting_schema)),
            Err(CheckpointContractError::FieldGap { field, reason, .. })
                if field == "previous.schema_version" && reason.contains("v0.3")
        ));
    }

    #[test]
    fn v0_3_through_v0_8_explicit_state_has_precedence_over_legacy_flags() {
        for (version, _) in SUPPORTED.into_iter().skip(1) {
            let mut checkpoint = checkpoint_for_version(version);
            checkpoint.flagged = true;
            checkpoint.drift_scores[0].flagged = true;
            checkpoint.drift_scores[0].state = DriftState::Cleared;
            assert_eq!(
                interpret(&checkpoint, None)
                    .expect("explicit cleared state should interpret")
                    .posture,
                None,
                "{version} must not use legacy flagged inference"
            );

            checkpoint.drift_scores[0].state = DriftState::Recovered;
            assert_eq!(
                interpret(&checkpoint, None)
                    .expect("explicit recovered state should interpret")
                    .posture,
                Some(CheckpointPosture::Recovered),
                "{version} must use analyzer-owned state"
            );
        }
    }

    #[test]
    fn typed_v0_8_delegation_accepts_analyzer_owned_ambiguous_and_visibility_states() {
        for visibility in [ChildWorkVisibility::Partial, ChildWorkVisibility::Opaque] {
            let mut checkpoint = checkpoint_for_version("v0.8");
            checkpoint.delegation = DelegationContext {
                topology: DelegationTopology::MixedOrAmbiguous,
                parent_session_id: Some(String::new()),
                child_session_ids: vec![String::new()],
                child_work_visibility: visibility,
                confidence: Confidence::Low,
                markers: vec![String::new()],
                supporting_evidence: vec![evidence("")],
                counter_evidence: Vec::new(),
            };

            let interpreted = interpret(&checkpoint, None)
                .expect("analyzer-owned delegation projection should be accepted");
            assert_eq!(interpreted.delegation, Some(checkpoint.delegation.clone()));
        }
    }

    #[test]
    fn parent_orchestration_text_never_infers_typed_delegation() {
        let mut legacy = checkpoint_for_version("v0.7");
        legacy.task_frame.objective =
            "parent spawned three child agents and is waiting for results".to_string();
        legacy.drift_scores[0].evidence = vec![evidence("spawn_agent wait_agent child result")];
        assert_eq!(
            interpret(&legacy, None)
                .expect("legacy orchestration text should be accepted")
                .delegation,
            None
        );

        let mut v0_8 = checkpoint_for_version("v0.8");
        v0_8.task_frame.objective = legacy.task_frame.objective;
        v0_8.drift_scores[0].evidence = legacy.drift_scores[0].evidence.clone();
        assert_eq!(
            interpret(&v0_8, None)
                .expect("typed v0.8 delegation should be projected")
                .delegation,
            Some(DelegationContext::default())
        );
    }

    #[test]
    fn interpretation_carries_only_validated_typed_facts() {
        let mut checkpoint = checkpoint_for_version("v0.8");
        checkpoint.flagged = true;
        checkpoint.drift_scores[0].flagged = true;
        checkpoint.drift_scores[0].state = DriftState::Active;
        checkpoint.drift_scores[0].raw_score = 87;
        checkpoint.drift_scores[0].evidence = vec![evidence("typed active evidence")];

        let interpreted = interpret(&checkpoint, None).expect("checkpoint should interpret");
        assert_eq!(interpreted.checkpoint, checkpoint);
        assert_eq!(
            interpreted.projection_profile,
            CheckpointProjectionProfile::Schema(CheckpointSchemaVersion::V0_8)
        );
        assert_eq!(interpreted.cursor.session_id, "session-a");
        assert_eq!(interpreted.cursor.ordinal, 1);
        assert_eq!(
            interpreted.warning_fingerprint,
            "session-a:dead_end_thrash:run focused proof"
        );
        assert!(interpreted.flagged);
        assert_eq!(interpreted.max_flagged_score, Some(87));
        assert_eq!(interpreted.posture, Some(CheckpointPosture::Active));
        assert_eq!(interpreted.evidence, checkpoint.drift_scores[0].evidence);
        assert_eq!(interpreted.delegation, Some(DelegationContext::default()));
    }

    #[test]
    fn total_projection_matches_validated_facts_for_every_supported_typed_schema() {
        for (literal, schema) in SUPPORTED {
            let mut checkpoint = checkpoint_for_version(literal);
            checkpoint.flagged = true;
            checkpoint.drift_scores[0].flagged = true;
            checkpoint.drift_scores[0].raw_score = 87;
            checkpoint.drift_scores[0].state = if literal == "v0.2" {
                DriftState::Cleared
            } else {
                DriftState::Active
            };
            checkpoint.drift_scores[0].evidence = vec![evidence("typed projection evidence")];

            let validated = interpret(&checkpoint, None).expect("supported checkpoint interprets");
            let projected =
                project_checkpoint_compatibility(CheckpointCompatibilityProjectionInput {
                    checkpoint: &checkpoint,
                    previous_checkpoint: None,
                });

            assert_eq!(
                projected.projection_profile,
                CheckpointProjectionProfile::Schema(schema),
                "{literal} must retain its exact supported profile"
            );
            assert_eq!(projected, validated, "{literal} projection facts diverged");
        }
    }

    #[test]
    fn total_projection_is_deterministic_for_supported_but_core_invalid_typed_shapes() {
        let mut empty_required_strings = checkpoint_for_version("v0.8");
        empty_required_strings.session_id.clear();
        empty_required_strings.checkpoint_id.clear();
        empty_required_strings.task_frame.objective.clear();
        empty_required_strings.expected_next_step.clear();

        let mut missing_turn_context = checkpoint_for_version("v0.4");
        missing_turn_context.turn_context = None;

        let mut missing_session_archetype = checkpoint_for_version("v0.5");
        missing_session_archetype.session_archetype = None;

        let mut missing_session_progress = checkpoint_for_version("v0.8");
        missing_session_progress.session_progress = None;

        for (checkpoint, schema) in [
            (empty_required_strings, CheckpointSchemaVersion::V0_8),
            (missing_turn_context, CheckpointSchemaVersion::V0_4),
            (missing_session_archetype, CheckpointSchemaVersion::V0_5),
            (missing_session_progress, CheckpointSchemaVersion::V0_8),
        ] {
            assert!(
                interpret(&checkpoint, None).is_err(),
                "the fallible core must reject the supported-but-invalid typed shape"
            );

            let project = || {
                project_checkpoint_compatibility(CheckpointCompatibilityProjectionInput {
                    checkpoint: &checkpoint,
                    previous_checkpoint: None,
                })
            };
            let first = project();
            let second = project();

            assert_eq!(first, second, "the total projection must be deterministic");
            assert_eq!(
                first.projection_profile,
                CheckpointProjectionProfile::Schema(schema),
                "supported literals must retain their exact compatibility profile"
            );
            assert_eq!(first.checkpoint, checkpoint);
        }
    }

    #[test]
    fn total_projection_preserves_supported_profile_without_validating_history_relationships() {
        let mut checkpoint = checkpoint_for_version("v0.2");
        checkpoint.ordinal = 2;
        checkpoint.checkpoint_id = "session-a:0002".to_string();
        checkpoint.drift_scores[0].evidence = vec![evidence(
            "historical repeated failure evidence: facade-compatible history",
        )];

        let mut cross_session = checkpoint_for_version("v0.2");
        cross_session.session_id = "session-b".to_string();
        cross_session.checkpoint_id = "session-b:0001".to_string();
        cross_session.flagged = true;
        cross_session.drift_scores[0].flagged = true;

        let mut mismatched_schema = checkpoint_for_version("v0.3");
        mismatched_schema.flagged = true;
        mismatched_schema.drift_scores[0].flagged = true;

        assert!(matches!(
            interpret(&checkpoint, Some(&cross_session)),
            Err(CheckpointContractError::CrossSessionHistory { .. })
        ));
        assert!(matches!(
            interpret(&checkpoint, Some(&mismatched_schema)),
            Err(CheckpointContractError::FieldGap { field, .. })
                if field == "previous.schema_version"
        ));

        for previous in [&cross_session, &mismatched_schema] {
            let project = || {
                project_checkpoint_compatibility(CheckpointCompatibilityProjectionInput {
                    checkpoint: &checkpoint,
                    previous_checkpoint: Some(previous),
                })
            };
            let first = project();
            let second = project();

            assert_eq!(first, second, "the total projection must be deterministic");
            assert_eq!(
                first.projection_profile,
                CheckpointProjectionProfile::Schema(CheckpointSchemaVersion::V0_2)
            );
            assert_eq!(first.checkpoint, checkpoint);
            assert_eq!(first.posture, Some(CheckpointPosture::Recovered));
            assert_eq!(first.evidence, checkpoint.drift_scores[0].evidence);
        }
    }

    #[test]
    fn unsupported_typed_facade_input_uses_explicit_total_compatibility_profile() {
        let mut previous = checkpoint_for_version("v0.2");
        previous.schema_version = "v-next".to_string();
        previous.flagged = true;
        previous.drift_scores[0].flagged = true;

        let mut checkpoint = checkpoint_for_version("v0.2");
        checkpoint.schema_version = "v-next".to_string();
        checkpoint.session_id = String::new();
        checkpoint.checkpoint_id = String::new();
        checkpoint.task_frame.objective = String::new();
        checkpoint.expected_next_step = String::new();
        checkpoint.drift_scores[0].evidence = vec![evidence(
            "historical repeated failure evidence: facade-compatible history",
        )];

        let projected = project_checkpoint_compatibility(CheckpointCompatibilityProjectionInput {
            checkpoint: &checkpoint,
            previous_checkpoint: Some(&previous),
        });

        assert_eq!(
            projected.projection_profile,
            CheckpointProjectionProfile::CompatibilityOnly
        );
        assert_eq!(projected.posture, Some(CheckpointPosture::Recovered));
        assert_eq!(projected.evidence, checkpoint.drift_scores[0].evidence);
        assert_eq!(projected.delegation, None);
    }

    #[test]
    fn structured_field_errors_retain_schema_checkpoint_and_field_detail() {
        let mut value = serialized_checkpoint("v0.6");
        value["task_frame"]["objective"] = json!("");
        let error = validate_serialized_checkpoint(&value)
            .expect_err("empty required field must fail closed");
        assert!(matches!(
            error,
            CheckpointContractError::FieldGap {
                checkpoint_id,
                schema_version,
                field,
                reason,
            } if checkpoint_id == "session-a:0001"
                && schema_version == "v0.6"
                && field == "task_frame.objective"
                && reason.contains("non-empty")
        ));
    }
}
