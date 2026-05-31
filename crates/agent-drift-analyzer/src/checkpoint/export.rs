use std::collections::BTreeMap;
use std::fs;
use std::io::Write;

use camino::{Utf8Path, Utf8PathBuf};

use crate::checkpoint::Checkpoint;
use crate::input::BundleSession;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportResult {
    pub checkpoints_path: Utf8PathBuf,
    pub summary_path: Utf8PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("failed to create analyzer output directory {path}: {source}")]
    CreateOutputDirectory {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write analyzer artifact {path}: {source}")]
    WriteArtifact {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to serialize analyzer artifact {path}: {source}")]
    SerializeArtifact {
        path: Utf8PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

pub fn export_checkpoints(
    output_dir: &Utf8Path,
    sessions: &[BundleSession],
    checkpoints: &[Checkpoint],
) -> Result<ExportResult, ExportError> {
    fs::create_dir_all(output_dir).map_err(|source| ExportError::CreateOutputDirectory {
        path: output_dir.to_owned(),
        source,
    })?;

    let checkpoints_path = output_dir.join("checkpoints.jsonl");
    let summary_path = output_dir.join("summary.md");

    let mut checkpoints_file =
        fs::File::create(&checkpoints_path).map_err(|source| ExportError::WriteArtifact {
            path: checkpoints_path.clone(),
            source,
        })?;
    let mut sorted = checkpoints.to_vec();
    sorted.sort_by(|left, right| {
        left.session_id
            .cmp(&right.session_id)
            .then_with(|| left.ordinal.cmp(&right.ordinal))
    });
    for checkpoint in &sorted {
        let line =
            serde_json::to_string(checkpoint).map_err(|source| ExportError::SerializeArtifact {
                path: checkpoints_path.clone(),
                source,
            })?;
        writeln!(checkpoints_file, "{line}").map_err(|source| ExportError::WriteArtifact {
            path: checkpoints_path.clone(),
            source,
        })?;
    }

    let summary = render_summary(sessions, &sorted);
    fs::write(&summary_path, summary).map_err(|source| ExportError::WriteArtifact {
        path: summary_path.clone(),
        source,
    })?;

    Ok(ExportResult {
        checkpoints_path,
        summary_path,
    })
}

fn render_summary(sessions: &[BundleSession], checkpoints: &[Checkpoint]) -> String {
    let flagged = checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.flagged)
        .count();
    let session_count = checkpoints
        .iter()
        .map(|checkpoint| checkpoint.session_id.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let turns = sessions.iter().map(session_turn_count).sum::<usize>();
    let mut by_session = BTreeMap::<&str, Vec<&Checkpoint>>::new();
    for checkpoint in checkpoints {
        by_session
            .entry(checkpoint.session_id.as_str())
            .or_default()
            .push(checkpoint);
    }
    let turns_by_session = sessions
        .iter()
        .map(|session| (session.session_id.as_str(), session_turn_count(session)))
        .collect::<BTreeMap<_, _>>();
    let mut lines = vec![
        "# Agent Drift Analyzer Summary".to_string(),
        String::new(),
        format!("Sessions analyzed: `{session_count}`"),
        format!("Turns observed: `{turns}`"),
        format!("Checkpoints emitted: `{}`", checkpoints.len()),
        format!("Flagged checkpoints: `{flagged}`"),
        String::new(),
    ];

    for (session_id, session_checkpoints) in by_session {
        lines.push(format!("## {session_id}"));
        lines.push(format!(
            "- Turns observed: `{}`",
            turns_by_session.get(session_id).copied().unwrap_or(0)
        ));
        lines.push(format!("- Checkpoints: `{}`", session_checkpoints.len()));
        lines.push(format!(
            "- Flagged checkpoints: `{}`",
            session_checkpoints
                .iter()
                .filter(|checkpoint| checkpoint.flagged)
                .count()
        ));
        for checkpoint in session_checkpoints {
            let flagged_scores = checkpoint
                .drift_scores
                .iter()
                .filter(|score| score.flagged)
                .map(|score| format!("{:?}", score.class))
                .collect::<Vec<_>>();
            lines.push(format!(
                "- {}: flagged=`{}` next=`{}` drift=`{}`",
                checkpoint.checkpoint_id,
                if checkpoint.flagged { "yes" } else { "no" },
                checkpoint.expected_next_step,
                if flagged_scores.is_empty() {
                    "none".to_string()
                } else {
                    flagged_scores.join(", ")
                }
            ));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

fn session_turn_count(session: &BundleSession) -> usize {
    session
        .archival_rows
        .iter()
        .filter_map(|row| row.turn_id.as_deref())
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}
