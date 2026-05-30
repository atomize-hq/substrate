use std::fs;
use std::io::{BufRead, BufReader};

use agent_drift_analyzer::Checkpoint;
use camino::{Utf8Path, Utf8PathBuf};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

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
    let mut checkpoints: Vec<Checkpoint> = read_jsonl_file(&checkpoints_path)?;
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
    if schema_version != "v0.1" {
        return Err(InputError::UnsupportedSchemaVersion {
            checkpoint_dir: checkpoint_dir.to_owned(),
            schema_version,
            expected_schema_version: "v0.1",
        });
    }

    Ok(ReplayCheckpointBundle {
        checkpoint_dir: checkpoint_dir.to_owned(),
        checkpoints_path,
        summary_path,
        summary_markdown,
        schema_version: "v0.1".to_string(),
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

fn read_jsonl_file<T>(path: &Utf8Path) -> Result<Vec<T>, InputError>
where
    T: DeserializeOwned,
{
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
    let mut items = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = line.map_err(|source| InputError::ReadArtifact {
            path: path.to_owned(),
            source,
        })?;
        if line.trim().is_empty() {
            continue;
        }
        let item = serde_json::from_str(&line).map_err(|source| InputError::ParseArtifactLine {
            path: path.to_owned(),
            line_number,
            source,
        })?;
        items.push(item);
    }

    Ok(items)
}
