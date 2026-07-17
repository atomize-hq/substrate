use std::fs;
use std::io::{BufRead, BufReader};

use agent_drift_analyzer::Checkpoint;
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::checkpoint_interpretation::{validate_serialized_checkpoint, CheckpointContractError};

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
    let (mut checkpoints, deferred_schema_error) =
        read_checkpoint_jsonl_file(checkpoint_dir, &checkpoints_path)?;
    if checkpoints.is_empty() {
        return Err(InputError::EmptyBundle {
            checkpoint_dir: checkpoint_dir.to_owned(),
        });
    }
    let schema_version = checkpoints
        .first()
        .map(|checkpoint| checkpoint.schema_version.clone())
        .ok_or_else(|| InputError::EmptyBundle {
            checkpoint_dir: checkpoint_dir.to_owned(),
        })?;

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
    if let Some(error) = deferred_schema_error {
        return Err(error);
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

fn read_checkpoint_jsonl_file(
    checkpoint_dir: &Utf8Path,
    path: &Utf8Path,
) -> Result<(Vec<Checkpoint>, Option<InputError>), InputError> {
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
    let mut deferred_schema_error = None;

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
        if let Err(error) = validate_serialized_checkpoint(&value) {
            let error = map_serialized_contract_error(checkpoint_dir, path, line_number, error);
            if matches!(&error, InputError::UnsupportedSchemaVersion { .. }) {
                if deferred_schema_error.is_none() {
                    deferred_schema_error = Some(error);
                }
            } else {
                return Err(error);
            }
        }

        let checkpoint =
            serde_json::from_value(value).map_err(|source| InputError::ParseArtifactLine {
                path: path.to_owned(),
                line_number,
                source,
            })?;
        checkpoints.push(checkpoint);
    }

    Ok((checkpoints, deferred_schema_error))
}

fn map_serialized_contract_error(
    checkpoint_dir: &Utf8Path,
    path: &Utf8Path,
    line_number: usize,
    error: CheckpointContractError,
) -> InputError {
    match error {
        CheckpointContractError::UnsupportedSchema {
            schema_version,
            expected,
            ..
        } => InputError::UnsupportedSchemaVersion {
            checkpoint_dir: checkpoint_dir.to_owned(),
            schema_version,
            expected_schema_version: expected,
        },
        CheckpointContractError::FieldGap {
            checkpoint_id,
            schema_version,
            field,
            reason,
        } => InputError::ContractGap {
            path: path.to_owned(),
            line_number,
            schema_version,
            field: replay_contract_field(field.as_str()),
            reason: format!("checkpoint {checkpoint_id}: {reason}"),
        },
        CheckpointContractError::CrossSessionHistory {
            checkpoint_id,
            session_id,
            previous_checkpoint_id,
            previous_session_id,
        } => InputError::ContractGap {
            path: path.to_owned(),
            line_number,
            schema_version: "<unknown>".to_string(),
            field: "previous_same_session".to_string(),
            reason: format!(
                "checkpoint {checkpoint_id} in session {session_id} cannot use checkpoint {previous_checkpoint_id} from session {previous_session_id}"
            ),
        },
    }
}

fn replay_contract_field(field: &str) -> String {
    if let Some(index) = field
        .strip_prefix("drift_scores.")
        .and_then(|suffix| suffix.strip_suffix(".state"))
    {
        return format!("drift_scores[{index}].state");
    }
    field.to_string()
}

pub(crate) fn map_typed_contract_error(
    bundle: &ReplayCheckpointBundle,
    error: CheckpointContractError,
) -> InputError {
    match error {
        CheckpointContractError::UnsupportedSchema {
            schema_version,
            expected,
            ..
        } => InputError::UnsupportedSchemaVersion {
            checkpoint_dir: bundle.checkpoint_dir.clone(),
            schema_version,
            expected_schema_version: expected,
        },
        CheckpointContractError::FieldGap {
            checkpoint_id,
            schema_version,
            field,
            reason,
        } => InputError::ContractGap {
            path: bundle.checkpoints_path.clone(),
            line_number: checkpoint_artifact_line(bundle, checkpoint_id.as_str()),
            schema_version,
            field: replay_contract_field(field.as_str()),
            reason: format!("checkpoint {checkpoint_id}: {reason}"),
        },
        CheckpointContractError::CrossSessionHistory {
            checkpoint_id,
            session_id,
            previous_checkpoint_id,
            previous_session_id,
        } => InputError::ContractGap {
            path: bundle.checkpoints_path.clone(),
            line_number: checkpoint_artifact_line(bundle, checkpoint_id.as_str()),
            schema_version: bundle
                .checkpoints
                .iter()
                .find(|checkpoint| checkpoint.checkpoint_id == checkpoint_id)
                .map_or_else(
                    || "<unknown>".to_string(),
                    |checkpoint| checkpoint.schema_version.clone(),
                ),
            field: "previous_same_session".to_string(),
            reason: format!(
                "checkpoint {checkpoint_id} in session {session_id} cannot use checkpoint {previous_checkpoint_id} from session {previous_session_id}"
            ),
        },
    }
}

fn checkpoint_artifact_line(bundle: &ReplayCheckpointBundle, checkpoint_id: &str) -> usize {
    if let Ok(contents) = fs::read_to_string(&bundle.checkpoints_path) {
        if let Some(line_number) = contents.lines().enumerate().find_map(|(index, line)| {
            serde_json::from_str::<Value>(line).ok().and_then(|value| {
                (value.get("checkpoint_id").and_then(Value::as_str) == Some(checkpoint_id))
                    .then_some(index + 1)
            })
        }) {
            return line_number;
        }
    }

    bundle
        .checkpoints
        .iter()
        .position(|checkpoint| checkpoint.checkpoint_id == checkpoint_id)
        .map_or(1, |index| index + 1)
}
