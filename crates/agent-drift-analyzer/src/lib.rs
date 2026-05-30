#![forbid(unsafe_code)]
#![allow(unused_crate_dependencies)]

#[cfg(test)]
use tempfile as _;

use camino::Utf8PathBuf;
use checkpoint::export_checkpoints;
use context::{assemble_context, ContextPack};
use inference::infer_task_frame;
use input::load_bundle;
use scoring::score_session;

pub mod checkpoint;
pub mod cli;
pub mod context;
pub mod inference;
pub mod input;
pub mod scoring;

pub use checkpoint::{
    Checkpoint, CheckpointBoundary, Confidence, DriftClass, DriftScore, EvidenceRef, TaskFrame,
};
pub use input::{AnalyzerSurface, BundleSession, InputBundle, InputError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzeRequest {
    pub input_dir: Utf8PathBuf,
    pub output_dir: Utf8PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionAnalysis {
    pub session_id: String,
    pub context: ContextPack,
    pub task_frame: TaskFrame,
    pub checkpoints: Vec<Checkpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzeResult {
    pub sessions: Vec<SessionAnalysis>,
    pub checkpoints_path: Utf8PathBuf,
    pub summary_path: Utf8PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum AnalyzerError {
    #[error(transparent)]
    Input(#[from] InputError),
    #[error(transparent)]
    Export(#[from] checkpoint::ExportError),
}

pub fn run() -> anyhow::Result<()> {
    cli::run()
}

pub fn analyze_bundle(request: &AnalyzeRequest) -> Result<AnalyzeResult, AnalyzerError> {
    let bundle = load_bundle(&request.input_dir)?;
    analyze_loaded_bundle(&bundle, &request.output_dir)
}

pub fn analyze_loaded_bundle(
    bundle: &InputBundle,
    output_dir: &camino::Utf8Path,
) -> Result<AnalyzeResult, AnalyzerError> {
    let mut analyses = Vec::with_capacity(bundle.sessions.len());
    let mut exported_checkpoints = Vec::new();

    for session in &bundle.sessions {
        let context = assemble_context(session);
        let task_frame = infer_task_frame(&context);
        let scores = score_session(session, &context, &task_frame);
        let checkpoints = checkpoint::build_session_checkpoints(session, &task_frame, scores);
        exported_checkpoints.extend(checkpoints.iter().cloned());
        analyses.push(SessionAnalysis {
            session_id: session.session_id.clone(),
            context,
            task_frame,
            checkpoints,
        });
    }

    let export_result = export_checkpoints(output_dir, &exported_checkpoints)?;
    Ok(AnalyzeResult {
        sessions: analyses,
        checkpoints_path: export_result.checkpoints_path,
        summary_path: export_result.summary_path,
    })
}
