use std::fs;
use std::io::{BufRead, BufReader};

use agent_drift_analyzer::Checkpoint;
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const SUPPORTED_ANALYZER_CHECKPOINT_SCHEMAS: &[&str] = &["v0.2", "v0.3"];
const SUPPORTED_ANALYZER_CHECKPOINT_SCHEMA_DESCRIPTION: &str = "v0.2 or v0.3";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CheckpointCursor {
    pub session_id: String,
    pub ordinal: usize,
}

impl From<&Checkpoint> for CheckpointCursor {
    fn from(value: &Checkpoint) -> Self {
        Self {
            session_id: value.session_id.clone(),
            ordinal: value.ordinal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayCheckpointBundle {
    pub checkpoint_dir: Utf8PathBuf,
    pub checkpoints_path: Utf8PathBuf,
    pub summary_path: Utf8PathBuf,
    pub summary_markdown: String,
    pub schema_version: String,
    pub checkpoints: Vec<Checkpoint>,
}

impl ReplayCheckpointBundle {
    pub fn checkpoints_after(&self, cursor: Option<&CheckpointCursor>) -> Vec<Checkpoint> {
        match cursor {
            Some(cursor) => self
                .checkpoints
                .iter()
                .filter(|checkpoint| {
                    checkpoint.session_id > cursor.session_id
                        || (checkpoint.session_id == cursor.session_id
                            && checkpoint.ordinal > cursor.ordinal)
                })
                .cloned()
                .collect(),
            None => self.checkpoints.clone(),
        }
    }

    pub fn summary_excerpt(&self, max_lines: usize) -> Vec<String> {
        self.summary_markdown
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .take(max_lines)
            .map(ToOwned::to_owned)
            .collect()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("required analyzer checkpoint artifact is missing: {path}")]
    MissingArtifact { path: Utf8PathBuf },
    #[error("failed to read analyzer checkpoint artifact {path}: {source}")]
    ReadArtifact {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse analyzer checkpoint artifact {path}: {source}")]
    ParseArtifact {
        path: Utf8PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to parse analyzer checkpoint artifact {path} at line {line_number}: {source}")]
    ParseArtifactLine {
        path: Utf8PathBuf,
        line_number: usize,
        #[source]
        source: serde_json::Error,
    },
    #[error(
        "analyzer checkpoint artifact {path} at line {line_number} violates the {schema_version} contract: missing {field} ({reason})"
    )]
    ContractGap {
        path: Utf8PathBuf,
        line_number: usize,
        schema_version: String,
        field: String,
        reason: String,
    },
    #[error("checkpoint bundle {checkpoint_dir} does not contain any checkpoints")]
    EmptyBundle { checkpoint_dir: Utf8PathBuf },
    #[error("checkpoint bundle {checkpoint_dir} mixes schema versions: {versions:?}")]
    MixedSchemaVersions {
        checkpoint_dir: Utf8PathBuf,
        versions: Vec<String>,
    },
    #[error(
        "checkpoint bundle {checkpoint_dir} uses unsupported schema version {schema_version}; expected {expected_schema_version}"
    )]
    UnsupportedSchemaVersion {
        checkpoint_dir: Utf8PathBuf,
        schema_version: String,
        expected_schema_version: &'static str,
    },
}

pub fn load_replay_bundle(checkpoint_dir: &Utf8Path) -> Result<ReplayCheckpointBundle, InputError> {
    let checkpoints_path = checkpoint_dir.join("checkpoints.jsonl");
    let summary_path = checkpoint_dir.join("summary.md");

    let summary_markdown = read_text_file(&summary_path)?;
    let mut checkpoints = read_checkpoint_jsonl_file(&checkpoints_path)?;
    if checkpoints.is_empty() {
        return Err(InputError::EmptyBundle {
            checkpoint_dir: checkpoint_dir.to_owned(),
        });
    }

    checkpoints.sort_by(|left, right| {
        left.session_id
            .cmp(&right.session_id)
            .then_with(|| left.ordinal.cmp(&right.ordinal))
            .then_with(|| left.checkpoint_id.cmp(&right.checkpoint_id))
    });

    let mut versions = checkpoints
        .iter()
        .map(|checkpoint| checkpoint.schema_version.clone())
        .collect::<Vec<_>>();
    versions.sort();
    versions.dedup();
    if versions.len() != 1 {
        return Err(InputError::MixedSchemaVersions {
            checkpoint_dir: checkpoint_dir.to_owned(),
            versions,
        });
    }

    let schema_version = versions.pop().expect("versions is not empty");
    if !SUPPORTED_ANALYZER_CHECKPOINT_SCHEMAS.contains(&schema_version.as_str()) {
        return Err(InputError::UnsupportedSchemaVersion {
            checkpoint_dir: checkpoint_dir.to_owned(),
            schema_version,
            expected_schema_version: SUPPORTED_ANALYZER_CHECKPOINT_SCHEMA_DESCRIPTION,
        });
    }

    Ok(ReplayCheckpointBundle {
        checkpoint_dir: checkpoint_dir.to_owned(),
        checkpoints_path,
        summary_path,
        summary_markdown,
        schema_version,
        checkpoints,
    })
}

fn read_text_file(path: &Utf8Path) -> Result<String, InputError> {
    if !path.exists() {
        return Err(InputError::MissingArtifact {
            path: path.to_owned(),
        });
    }
    fs::read_to_string(path).map_err(|source| InputError::ReadArtifact {
        path: path.to_owned(),
        source,
    })
}

fn read_checkpoint_jsonl_file(path: &Utf8Path) -> Result<Vec<Checkpoint>, InputError> {
    if !path.exists() {
        return Err(InputError::MissingArtifact {
            path: path.to_owned(),
        });
    }

    let file = fs::File::open(path).map_err(|source| InputError::ReadArtifact {
        path: path.to_owned(),
        source,
    })?;
    let reader = BufReader::new(file);
    let mut checkpoints = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = line.map_err(|source| InputError::ReadArtifact {
            path: path.to_owned(),
            source,
        })?;
        if line.trim().is_empty() {
            continue;
        }

        let value: Value =
            serde_json::from_str(&line).map_err(|source| InputError::ParseArtifactLine {
                path: path.to_owned(),
                line_number,
                source,
            })?;
        validate_checkpoint_contract(path, line_number, &value)?;

        let checkpoint =
            serde_json::from_value(value).map_err(|source| InputError::ParseArtifactLine {
                path: path.to_owned(),
                line_number,
                source,
            })?;
        checkpoints.push(checkpoint);
    }

    Ok(checkpoints)
}

fn validate_checkpoint_contract(
    path: &Utf8Path,
    line_number: usize,
    checkpoint: &Value,
) -> Result<(), InputError> {
    if checkpoint.get("schema_version").and_then(Value::as_str) != Some("v0.3") {
        return Ok(());
    }

    let Some(drift_scores) = checkpoint.get("drift_scores").and_then(Value::as_array) else {
        return Ok(());
    };

    for (index, score) in drift_scores.iter().enumerate() {
        if score.get("state").is_none() {
            return Err(InputError::ContractGap {
                path: path.to_owned(),
                line_number,
                schema_version: "v0.3".to_string(),
                field: format!("drift_scores[{index}].state"),
                reason:
                    "v0.3 checkpoints must serialize explicit drift state for every drift score"
                        .to_string(),
            });
        }
    }

    Ok(())
}
