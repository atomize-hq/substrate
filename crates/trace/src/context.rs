use crate::output::TraceOutput;
use crate::span::{ExecutionOrigin, ReplayContext, Span, SpanBuilder, TransportMeta};
use crate::util::{get_policy_git_hash, get_umask, hash_env_vars};
use anyhow::{anyhow, Result};
use parking_lot::{RwLock, RwLockWriteGuard};
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tracing::debug;

static TRACE_CONTEXT: OnceLock<TraceContext> = OnceLock::new();
const WORLD_IMAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Per-session tracing context that owns the current trace output and policy metadata.
///
/// ```
/// use substrate_trace::TraceContext;
/// use tempfile::TempDir;
///
/// # fn main() -> anyhow::Result<()> {
/// let temp = TempDir::new()?;
/// let ctx = TraceContext::default();
/// ctx.init_trace(Some(temp.path().join("trace.jsonl")))?;
///
/// let span = ctx
///     .create_span_builder()
///     .with_command("echo trace-doc")
///     .with_cwd("/tmp")
///     .start()?;
/// let span_id = span.get_span_id().to_string();
/// span.finish(0, vec![], None)?;
///
/// let loaded = ctx.load_span(&span_id)?;
/// assert_eq!(loaded.cmd, "echo trace-doc");
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct TraceContext {
    output: Arc<RwLock<Option<TraceOutput>>>,
    policy_id: Arc<RwLock<String>>,
}

impl Default for TraceContext {
    fn default() -> Self {
        Self {
            output: Arc::new(RwLock::new(None)),
            policy_id: Arc::new(RwLock::new("default".to_string())),
        }
    }
}

impl TraceContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init_trace(&self, path: Option<PathBuf>) -> Result<()> {
        let trace_path = path.unwrap_or_else(|| {
            env::var("SHIM_TRACE_LOG")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    dirs::home_dir()
                        .unwrap_or_else(|| PathBuf::from("/tmp"))
                        .join(".substrate")
                        .join("trace.jsonl")
                })
        });

        if let Some(parent) = trace_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if let Ok(meta) = fs::metadata(&trace_path) {
            if meta.len() >= TraceOutput::max_bytes() {
                let keep = TraceOutput::keep_files();
                if keep > 0 {
                    let oldest = trace_path.with_extension(format!("jsonl.{}", keep));
                    let _ = fs::remove_file(&oldest);
                    for i in (2..=keep).rev() {
                        let from = trace_path.with_extension(format!("jsonl.{}", i - 1));
                        let to = trace_path.with_extension(format!("jsonl.{}", i));
                        let _ = fs::rename(&from, &to);
                    }
                }
                let bak = trace_path.with_extension("jsonl.1");
                let _ = fs::rename(&trace_path, &bak);
            }
        }

        let output = TraceOutput::new(trace_path)?;
        *self.output.write() = Some(output);

        debug!("Initialized trace output");
        Ok(())
    }

    pub fn set_policy_id(&self, policy_id: &str) {
        *self.policy_id.write() = policy_id.to_string();
    }

    pub fn policy_id(&self) -> String {
        self.policy_id.read().clone()
    }

    pub fn create_span_builder(&self) -> SpanBuilder {
        SpanBuilder::new(self.clone())
    }

    pub fn append_to_trace(&self, entry: &serde_json::Value) -> Result<()> {
        if let Some(ref mut output) = *self.output_write() {
            output.rotate_if_needed()?;
            writeln!(output.writer, "{}", entry)?;
            if env::var("SHIM_FSYNC").unwrap_or_default() == "1" {
                output.writer.flush()?;
                output.writer.get_ref().sync_all()?;
            } else {
                output.writer.flush()?;
            }
        }
        Ok(())
    }

    pub fn load_span(&self, span_id: &str) -> Result<Span> {
        let trace_path = self
            .output
            .read()
            .as_ref()
            .map(|o| o.path.clone())
            .ok_or_else(|| anyhow!("Trace not initialized"))?;

        let file = File::open(trace_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Ok(span) = serde_json::from_str::<Span>(&line) {
                if span.span_id == span_id {
                    return Ok(span);
                }
            }
        }

        Err(anyhow!("Span {} not found", span_id))
    }

    pub(crate) fn build_replay_context(
        &self,
        transport: Option<TransportMeta>,
        execution_origin: ExecutionOrigin,
    ) -> Result<ReplayContext> {
        let recorded_user = env::var("USER").ok().or_else(|| env::var("USERNAME").ok());
        Ok(ReplayContext {
            path: env::var("PATH").ok(),
            env_hash: hash_env_vars()?,
            umask: get_umask()?,
            locale: env::var("LANG").ok(),
            cwd: env::current_dir()?.to_string_lossy().to_string(),
            policy_id: self.policy_id(),
            policy_commit: get_policy_git_hash()?,
            world_image_version: WORLD_IMAGE_VERSION.to_string(),
            hostname: env::var("HOSTNAME").ok(),
            user: recorded_user,
            shell: env::var("SHELL").ok(),
            term: env::var("TERM").ok(),
            world_image: Some(WORLD_IMAGE_VERSION.to_string()),
            execution_origin: Some(execution_origin),
            transport,
            anchor_mode: env::var("SUBSTRATE_ANCHOR_MODE").ok(),
            anchor_path: env::var("SUBSTRATE_ANCHOR_PATH").ok(),
            world_root_mode: env::var("SUBSTRATE_WORLD_ROOT_MODE").ok(),
            world_root_path: env::var("SUBSTRATE_WORLD_ROOT_PATH").ok(),
            caged: env::var("SUBSTRATE_CAGED")
                .ok()
                .and_then(|value| match value.as_str() {
                    "1" => Some(true),
                    "0" => Some(false),
                    _ => None,
                }),
        })
    }

    pub(crate) fn output_write(&self) -> RwLockWriteGuard<'_, Option<TraceOutput>> {
        self.output.write()
    }
}

pub fn set_global_trace_context(context: TraceContext) -> Result<()> {
    if TRACE_CONTEXT.get().is_some() {
        return Ok(());
    }
    TRACE_CONTEXT
        .set(context)
        .map_err(|_| anyhow!("Trace context already initialized"))?;
    Ok(())
}

fn trace_context() -> Result<&'static TraceContext> {
    TRACE_CONTEXT
        .get()
        .ok_or_else(|| anyhow!("Trace context not initialized; call set_global_trace_context"))
}

pub fn init_trace(path: Option<PathBuf>) -> Result<()> {
    trace_context()?.init_trace(path)
}

pub fn set_policy_id(policy_id: &str) -> Result<()> {
    trace_context()?.set_policy_id(policy_id);
    Ok(())
}

pub fn get_policy_id() -> String {
    trace_context()
        .map(|ctx| ctx.policy_id())
        .unwrap_or_else(|_| "default".to_string())
}

pub fn create_span_builder() -> Result<SpanBuilder> {
    Ok(trace_context()?.create_span_builder())
}

pub fn append_to_trace(entry: &serde_json::Value) -> Result<()> {
    trace_context()?.append_to_trace(entry)
}

pub fn load_span(span_id: &str) -> Result<Span> {
    trace_context()?.load_span(span_id)
}
