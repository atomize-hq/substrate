use agent_session_compactor::RowRef;
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
    IgnoringRepoTruth,
    DeadEndThrash,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceRef {
    pub row: RowRef,
    pub reason: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DriftScore {
    pub class: DriftClass,
    pub raw_score: u8,
    pub confidence: Confidence,
    pub flagged: bool,
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Checkpoint {
    pub schema_version: String,
    pub session_id: String,
    pub checkpoint_id: String,
    pub ordinal: usize,
    pub boundary: CheckpointBoundary,
    pub task_frame: TaskFrame,
    pub drift_scores: Vec<DriftScore>,
    pub expected_next_step: String,
    pub flagged: bool,
}
