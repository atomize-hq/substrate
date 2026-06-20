use agent_session_compactor::RowRef;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum DriftClass {
    WrongPlanBranch,
    #[serde(alias = "ignoring_repo_truth")]
    TruthGroundingGap,
    DeadEndThrash,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceRef {
    pub row: RowRef,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructuredObjective {
    pub objective_class: ObjectiveClass,
    pub primary_intent: ObjectiveIntent,
    pub target: Option<ObjectiveTarget>,
    pub constraints: Vec<ObjectiveConstraint>,
    pub success_conditions: Vec<SuccessCondition>,
    pub deliverables: Vec<RequestedDeliverable>,
    pub verification_commands: Vec<String>,
    pub evidence_spans: Vec<ObjectiveEvidenceSpan>,
    pub confidence: Confidence,
    pub unknowns: Vec<ObjectiveUnknown>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveClass {
    TaskStatement,
    NotTaskStatement,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveIntent {
    Implement,
    Debug,
    Review,
    Research,
    Plan,
    Validate,
    Docs,
    OtherTask,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectiveTarget {
    pub display: String,
    pub kind: ObjectiveTargetKind,
    pub paths: Vec<String>,
    pub symbols: Vec<String>,
    pub named_artifacts: Vec<String>,
    pub workspace_refs: Vec<String>,
    pub evidence: Vec<ObjectiveEvidenceSpan>,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveTargetKind {
    RepoSlice,
    CrateOrPackage,
    FileOrDirectory,
    SpecOrDesignDoc,
    TestOrVerifier,
    SkillOrInstructionSurface,
    ExternalArtifact,
    ConceptualTopic,
    UnknownTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectiveConstraint {
    pub display: String,
    pub constraint_kind: ObjectiveConstraintKind,
    pub evidence: Vec<ObjectiveEvidenceSpan>,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveConstraintKind {
    ScopeBoundary,
    NoCode,
    DocsOnly,
    ReviewOnly,
    ValidateOnly,
    PlatformBoundary,
    DeliverableFormat,
    OtherConstraint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuccessCondition {
    pub display: String,
    pub evidence: Vec<ObjectiveEvidenceSpan>,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequestedDeliverable {
    pub display: String,
    pub deliverable_kind: RequestedDeliverableKind,
    pub evidence: Vec<ObjectiveEvidenceSpan>,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RequestedDeliverableKind {
    CodeChange,
    DesignDoc,
    Plan,
    Review,
    ValidationReport,
    ResearchSummary,
    OtherDeliverable,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveSourceKind {
    ThreadGoal,
    UserPrompt,
    AssistantContext,
    SystemInstruction,
    ToolOutput,
    UnknownSource,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveSectionKind {
    Scope,
    Mission,
    Checklist,
    Verification,
    Constraints,
    Deliverables,
    Context,
    Boilerplate,
    ToolingInstructions,
    UnknownSection,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveRole {
    Goal,
    Constraint,
    Verification,
    Context,
    OtherRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectiveEvidenceSpan {
    pub row: RowRef,
    pub source_kind: ObjectiveSourceKind,
    pub section_kind: ObjectiveSectionKind,
    pub role: ObjectiveRole,
    pub excerpt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clause_index: Option<usize>,
    pub start_char: Option<usize>,
    pub end_char: Option<usize>,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectiveUnknown {
    pub field_name: String,
    pub reason: String,
    pub evidence: Vec<ObjectiveEvidenceSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskFrame {
    pub objective: String,
    pub confidence: Confidence,
    pub truth_artifacts: Vec<String>,
    pub working_set_paths: Vec<String>,
    pub tools: Vec<String>,
    pub command_families: Vec<String>,
    pub verification_commands: Vec<String>,
    pub supporting_evidence: Vec<EvidenceRef>,
    pub counter_evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckpointBoundary {
    pub start: RowRef,
    pub end: RowRef,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckpointDiagnostics {
    pub task_frame_transitioned: bool,
    pub working_set_changed: bool,
    pub interval_command_count: usize,
    pub interval_verification_command_count: usize,
    pub evidence_item_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnContext {
    pub turn_id: Option<String>,
    pub turn_ordinal: usize,
    pub rows_since_turn_start: usize,
    pub seconds_since_turn_start: Option<i64>,
    pub checkpoints_in_turn: usize,
    pub prompts_observed_in_session: usize,
    pub execution_mode: TurnExecutionMode,
    pub activity_mix: TurnActivityMix,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnActivityMix {
    pub directive_row_count: usize,
    pub assistant_message_count: usize,
    pub tool_call_count: usize,
    pub read_like_command_count: usize,
    pub write_like_command_count: usize,
    pub verification_like_command_count: usize,
    pub tool_output_count: usize,
}

#[derive(
    Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum TurnExecutionMode {
    Conversational,
    Autonomous,
    VerificationHeavy,
    #[default]
    Mixed,
}

#[derive(
    Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum DriftState {
    Active,
    Recovered,
    HistoricalOnly,
    #[default]
    Cleared,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DriftScore {
    pub class: DriftClass,
    #[serde(default)]
    pub state: DriftState,
    pub raw_score: u8,
    pub confidence: Confidence,
    pub flagged: bool,
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SessionArchetypeLabel {
    Troubleshooting,
    Planning,
    AutonomousImplementation,
    VerificationCloseout,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionArchetype {
    pub label: SessionArchetypeLabel,
    pub confidence: Confidence,
    pub supporting_evidence: Vec<EvidenceRef>,
    pub counter_evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionProgress {
    pub status: ProgressStatus,
    pub dimension: ProgressDimension,
    pub confidence: Confidence,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signals: Vec<ProgressSignal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_evidence: Vec<EvidenceRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub counter_evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ProgressStatus {
    Advancing,
    Mixed,
    Stalled,
    Regressing,
    InsufficientEvidence,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ProgressDimension {
    TroubleshootingFrontier,
    PlanningConvergence,
    ImplementationVerificationWall,
    VerificationCloseoutNarrowing,
    ParentVisibleOrchestration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgressSignal {
    pub code: ProgressSignalCode,
    pub polarity: SignalPolarity,
    pub strength: SignalStrength,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ProgressSignalCode {
    FailureFrontierAdvanced,
    FailureSignatureRepeated,
    FailureCountReduced,
    FailureCountIncreased,
    FailingScopeEdited,
    FailingScopeUnchanged,
    VerificationClean,
    VerificationScopeBroadened,
    VerificationScopeNarrowed,
    PlanArtifactCreated,
    PlanArtifactRefined,
    CandidateSetNarrowed,
    CandidateSetExpanded,
    WorkingSetConcentrated,
    WorkingSetDiffused,
    ResidualScopeShrank,
    ResidualScopeReopened,
    PreviouslyCleanScopeBroken,
    DelegationVisibilityLimited,
    TargetNotExercised,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SignalPolarity {
    Positive,
    Negative,
    Mixed,
    Limiting,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SignalStrength {
    Weak,
    Moderate,
    Strong,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Checkpoint {
    pub schema_version: String,
    pub session_id: String,
    pub checkpoint_id: String,
    pub ordinal: usize,
    pub boundary: CheckpointBoundary,
    #[serde(default)]
    pub turn_context: Option<TurnContext>,
    pub diagnostics: CheckpointDiagnostics,
    pub task_frame: TaskFrame,
    /// Additive, observation-only projection of the structured objective sidecar produced during
    /// context assembly. The legacy `task_frame.objective` string remains the compatibility surface;
    /// this field exposes the richer `StructuredObjective` so downstream validation/smoke can inspect
    /// intent, target, success/deliverable, and unknown semantics. Optional and serde-defaulted, so
    /// it is backward/forward compatible and does not change the required-field contract that
    /// `schema_version` gates (consumers such as `agent-drift-sentinel` ignore it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured_objective: Option<StructuredObjective>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_archetype: Option<SessionArchetype>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_progress: Option<SessionProgress>,
    pub drift_scores: Vec<DriftScore>,
    pub expected_next_step: String,
    pub flagged: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct RawCheckpoint {
    pub schema_version: String,
    pub session_id: String,
    pub checkpoint_id: String,
    pub ordinal: usize,
    pub boundary: CheckpointBoundary,
    #[serde(default)]
    pub turn_context: Option<TurnContext>,
    pub diagnostics: CheckpointDiagnostics,
    pub task_frame: TaskFrame,
    #[serde(default)]
    pub structured_objective: Option<StructuredObjective>,
    #[serde(default)]
    pub session_archetype: Option<SessionArchetype>,
    #[serde(default)]
    pub session_progress: Option<SessionProgress>,
    pub drift_scores: Vec<DriftScore>,
    pub expected_next_step: String,
    pub flagged: bool,
}

impl RawCheckpoint {
    fn into_checkpoint(self) -> Result<Checkpoint, String> {
        if schema_requires_session_archetype(&self.schema_version)
            && self.session_archetype.is_none()
        {
            return Err(format!(
                "checkpoint schema {} requires session_archetype",
                self.schema_version
            ));
        }

        if schema_requires_session_progress(&self.schema_version) && self.session_progress.is_none()
        {
            return Err(format!(
                "checkpoint schema {} requires session_progress",
                self.schema_version
            ));
        }

        Ok(Checkpoint {
            schema_version: self.schema_version,
            session_id: self.session_id,
            checkpoint_id: self.checkpoint_id,
            ordinal: self.ordinal,
            boundary: self.boundary,
            turn_context: self.turn_context,
            diagnostics: self.diagnostics,
            task_frame: self.task_frame,
            structured_objective: self.structured_objective,
            session_archetype: self.session_archetype,
            session_progress: self.session_progress,
            drift_scores: self.drift_scores,
            expected_next_step: self.expected_next_step,
            flagged: self.flagged,
        })
    }
}

impl<'de> Deserialize<'de> for Checkpoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RawCheckpoint::deserialize(deserializer)?
            .into_checkpoint()
            .map_err(de::Error::custom)
    }
}

fn schema_requires_session_archetype(schema_version: &str) -> bool {
    matches!(schema_version, "v0.5" | "v0.6")
}

fn schema_requires_session_progress(schema_version: &str) -> bool {
    schema_version == "v0.6"
}
