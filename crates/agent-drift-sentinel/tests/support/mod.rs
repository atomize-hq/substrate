#![allow(dead_code)]

use std::fs;

use agent_drift_analyzer::{
    Checkpoint, CheckpointBoundary, Confidence, DriftClass, DriftScore, EvidenceRef, TaskFrame,
};
use agent_session_compactor::RowRef;
use camino::{Utf8Path, Utf8PathBuf};
use tempfile::TempDir;

pub(crate) struct ReplayFixture {
    pub _temp_dir: TempDir,
    pub checkpoint_dir: Utf8PathBuf,
}

impl ReplayFixture {
    pub(crate) fn sample() -> Self {
        Self::from_checkpoints(sample_checkpoints(), sample_summary())
    }

    pub(crate) fn from_checkpoints(checkpoints: Vec<Checkpoint>, summary: &str) -> Self {
        let temp_dir = TempDir::new().expect("temp dir");
        let root = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp dir");
        let checkpoint_dir = root.join("checkpoint");
        fs::create_dir_all(&checkpoint_dir).expect("create checkpoint dir");
        write_jsonl(checkpoint_dir.join("checkpoints.jsonl"), &checkpoints);
        fs::write(checkpoint_dir.join("summary.md"), summary).expect("write summary");
        Self {
            _temp_dir: temp_dir,
            checkpoint_dir,
        }
    }
}

pub(crate) fn sample_summary() -> &'static str {
    "# Agent Drift Analyzer Summary\n\nSessions analyzed: `2`\nFlagged checkpoints: `3`\n"
}

pub(crate) fn sample_checkpoints() -> Vec<Checkpoint> {
    vec![
        checkpoint("session-alpha", 1, 82, true, "align plan to repo truth"),
        checkpoint(
            "session-alpha",
            2,
            40,
            true,
            "re-read the task doc before editing",
        ),
        checkpoint(
            "session-alpha",
            3,
            86,
            true,
            "re-read the task doc before editing",
        ),
        checkpoint(
            "session-beta",
            1,
            0,
            false,
            "continue on the current task frame",
        ),
    ]
}

pub(crate) fn checkpoint(
    session_id: &str,
    ordinal: usize,
    raw_score: u8,
    flagged: bool,
    expected_next_step: &str,
) -> Checkpoint {
    let row = RowRef {
        source_file: Utf8PathBuf::from(format!("/tmp/{session_id}.jsonl")),
        event_index: ordinal,
        row_ordinal: 0,
    };
    Checkpoint {
        schema_version: "v0.1".to_string(),
        session_id: session_id.to_string(),
        checkpoint_id: format!("{session_id}:{ordinal:04}"),
        ordinal,
        boundary: CheckpointBoundary {
            start: row.clone(),
            end: row.clone(),
        },
        task_frame: TaskFrame {
            objective: format!(
                "/goal Complete replay validation for {session_id} checkpoint {ordinal}"
            ),
            confidence: Confidence::Medium,
            truth_artifacts: vec!["docs/specs/agent-drift-sentinel-v0.2-spec.md".to_string()],
            working_set_paths: vec!["crates/agent-drift-sentinel/src/lib.rs".to_string()],
            tools: vec!["functions.shell_command".to_string()],
            command_families: vec!["cargo".to_string()],
            verification_commands: vec![
                "cargo test -p agent-drift-sentinel -- --nocapture".to_string()
            ],
            supporting_evidence: vec![EvidenceRef {
                row: row.clone(),
                reason: "objective row".to_string(),
            }],
            counter_evidence: Vec::new(),
        },
        drift_scores: vec![DriftScore {
            class: DriftClass::WrongPlanBranch,
            raw_score,
            confidence: Confidence::Medium,
            flagged,
            evidence: if flagged {
                vec![EvidenceRef {
                    row,
                    reason: format!("flagged score for {session_id}:{ordinal}"),
                }]
            } else {
                Vec::new()
            },
        }],
        expected_next_step: expected_next_step.to_string(),
        flagged,
    }
}

fn write_jsonl<T: serde::Serialize>(path: Utf8PathBuf, items: &[T]) {
    let body = items
        .iter()
        .map(|item| serde_json::to_string(item).expect("json"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(path, format!("{body}\n")).expect("write jsonl");
}
