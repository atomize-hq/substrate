use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use tracing::{debug, trace};
use uuid::Uuid;

static TRACE_CONTEXT: OnceLock<TraceContext> = OnceLock::new();
static WORLD_IMAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

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
        if let Some(ref mut output) = *self.output.write() {
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
        use std::io::{BufRead, BufReader};

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

    fn build_replay_context(&self) -> Result<ReplayContext> {
        Ok(ReplayContext {
            path: env::var("PATH").ok(),
            env_hash: hash_env_vars()?,
            umask: get_umask()?,
            locale: env::var("LANG").ok(),
            cwd: env::current_dir()?.to_string_lossy().to_string(),
            policy_id: self.policy_id(),
            policy_commit: get_policy_git_hash().ok(),
            world_image_version: WORLD_IMAGE_VERSION.to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub ts: DateTime<Utc>,
    pub event_type: String,
    pub session_id: String,
    pub span_id: String,
    pub parent_span: Option<String>,
    pub component: String,
    pub world_id: Option<String>,
    pub policy_id: String,
    pub agent_id: String,
    pub cwd: String,
    pub cmd: String,
    pub exit: Option<i32>,
    pub scopes_used: Vec<String>,
    pub fs_diff: Option<FsDiff>,
    pub replay_context: Option<ReplayContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<TransportMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_edges: Option<Vec<GraphEdge>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_decision: Option<PolicyDecision>,
}

// FsDiff is now imported from substrate_common
pub use substrate_common::FsDiff;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMeta {
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayContext {
    pub path: Option<String>,
    pub env_hash: String,
    pub umask: u32,
    pub locale: Option<String>,
    pub cwd: String,
    pub policy_id: String,
    pub policy_commit: Option<String>,
    pub world_image_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub edge_type: EdgeType,
    pub from_span: String,
    pub to_span: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    ParentChild,
    DataFlow,
    CausedBy,
    DependsOn,
    Triggers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub action: String, // "allow", "allow_with_restrictions", "deny"
    pub reason: Option<String>,
    pub restrictions: Option<Vec<String>>,
}

pub struct TraceOutput {
    writer: BufWriter<File>,
    path: PathBuf,
}

impl TraceOutput {
    fn new(path: impl AsRef<Path>) -> Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(&path)?;

        Ok(TraceOutput {
            writer: BufWriter::new(file),
            path: path.as_ref().to_path_buf(),
        })
    }

    fn max_bytes() -> u64 {
        const DEFAULT_MB: u64 = 100; // ~100MB
                                     // Accept both TRACE_LOG_MAX_MB (preferred) and legacy SHIM_TRACE_LOG_MAX_MB for tests/back-compat
        let mb = env::var("TRACE_LOG_MAX_MB")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .or_else(|| {
                env::var("SHIM_TRACE_LOG_MAX_MB")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
            })
            .unwrap_or(DEFAULT_MB);
        mb * 1024 * 1024
    }

    fn keep_files() -> usize {
        // Keep last N rotated files
        env::var("TRACE_LOG_KEEP")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(3)
    }

    fn rotate_if_needed(&mut self) -> Result<()> {
        // Flush current writer to ensure size is accurate
        self.writer.flush()?;
        let path = self.path.clone();

        if let Ok(meta) = fs::metadata(&path) {
            if meta.len() >= Self::max_bytes() {
                // Shift older rotations first while honoring retention
                // Remove the oldest file (".keep") if present, then shift (keep-1)->keep, ..., 1->2
                let keep = Self::keep_files();
                if keep > 0 {
                    let oldest = path.with_extension(format!("jsonl.{}", keep));
                    let _ = fs::remove_file(&oldest);
                    for i in (2..=keep).rev() {
                        let from = path.with_extension(format!("jsonl.{}", i - 1));
                        let to = path.with_extension(format!("jsonl.{}", i));
                        let _ = fs::rename(&from, &to);
                    }
                }
                // Rename current to .1 (writer still holds the fd; rename is allowed on Unix)
                let bak = path.with_extension("jsonl.1");
                let _ = fs::rename(&path, &bak);

                // Recreate fresh file and swap writer to new handle
                let file = OpenOptions::new().create(true).append(true).open(&path)?;
                self.writer = BufWriter::new(file);
            }
        }
        Ok(())
    }

    fn write_span(&mut self, span: &Span) -> Result<()> {
        self.rotate_if_needed()?;

        // Serialize span and ensure both `cmd` and legacy `command` keys exist for compat.
        let mut value = serde_json::to_value(span)?;
        if let Some(obj) = value.as_object_mut() {
            if !obj.contains_key("command") {
                if let Some(cmd_value) = obj.get("cmd").cloned() {
                    obj.insert("command".to_string(), cmd_value);
                }
            }
        }

        let json = serde_json::to_string(&value)?;
        writeln!(self.writer, "{}", json)?;

        if env::var("SHIM_FSYNC").unwrap_or_default() == "1" {
            self.writer.flush()?;
            self.writer.get_ref().sync_all()?;
        } else {
            self.writer.flush()?;
        }

        Ok(())
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

pub fn new_span(_parent: Option<&str>) -> String {
    format!("spn_{}", Uuid::now_v7())
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

pub struct SpanBuilder {
    span: Span,
    context: TraceContext,
}

impl SpanBuilder {
    fn new(context: TraceContext) -> Self {
        let session_id =
            env::var("SHIM_SESSION_ID").unwrap_or_else(|_| format!("ses_{}", Uuid::now_v7()));

        let agent_id = env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());

        let component = if env::var("SUBSTRATE_SHELL").is_ok() {
            "shell"
        } else if env::var("SHIM_ORIGINAL_PATH").is_ok() {
            "shim"
        } else {
            "unknown"
        };

        SpanBuilder {
            context: context.clone(),
            span: Span {
                ts: Utc::now(),
                event_type: "command_start".to_string(),
                session_id,
                span_id: new_span(None),
                parent_span: env::var("SHIM_PARENT_SPAN").ok(),
                component: component.to_string(),
                world_id: env::var("SUBSTRATE_WORLD_ID").ok(),
                policy_id: context.policy_id(),
                agent_id,
                cwd: env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                cmd: String::new(),
                exit: None,
                scopes_used: Vec::new(),
                fs_diff: None,
                replay_context: None,
                transport: None,
                graph_edges: None,
                policy_decision: None,
            },
        }
    }

    pub fn with_command(mut self, cmd: &str) -> Self {
        self.span.cmd = cmd.to_string();
        self
    }

    pub fn with_parent(mut self, parent: &str) -> Self {
        self.span.parent_span = Some(parent.to_string());
        self
    }

    pub fn with_world_id(mut self, world_id: &str) -> Self {
        self.span.world_id = Some(world_id.to_string());
        self
    }

    pub fn with_cwd(mut self, cwd: &str) -> Self {
        self.span.cwd = cwd.to_string();
        self
    }

    pub fn with_policy_decision(mut self, decision: PolicyDecision) -> Self {
        self.span.policy_decision = Some(decision);
        self
    }

    pub fn with_graph_edge(mut self, edge: GraphEdge) -> Self {
        let edges = self.span.graph_edges.get_or_insert_with(Vec::new);
        edges.push(edge);
        self
    }

    pub fn start(self) -> Result<ActiveSpan> {
        let span_id = self.span.span_id.clone();

        if let Some(ref mut output) = *self.context.output.write() {
            output.write_span(&self.span)?;
        }

        Ok(ActiveSpan {
            span_id,
            command: self.span.cmd,
            cwd: self.span.cwd,
            transport: None,
            context: self.context,
        })
    }
}

pub struct ActiveSpan {
    pub span_id: String,
    command: String,
    cwd: String,
    transport: Option<TransportMeta>,
    context: TraceContext,
}

impl ActiveSpan {
    pub fn set_transport(&mut self, transport: TransportMeta) {
        self.transport = Some(transport);
    }

    pub fn finish(
        self,
        exit_code: i32,
        scopes: Vec<String>,
        fs_diff: Option<FsDiff>,
    ) -> Result<()> {
        let replay_context = self.context.build_replay_context()?;

        let span = Span {
            ts: Utc::now(),
            event_type: "command_complete".to_string(),
            session_id: env::var("SHIM_SESSION_ID")
                .unwrap_or_else(|_| format!("ses_{}", Uuid::now_v7())),
            span_id: self.span_id.clone(),
            parent_span: env::var("SHIM_PARENT_SPAN").ok(),
            component: if env::var("SUBSTRATE_SHELL").is_ok() {
                "shell"
            } else {
                "shim"
            }
            .to_string(),
            world_id: env::var("SUBSTRATE_WORLD_ID").ok(),
            policy_id: self.context.policy_id(),
            agent_id: env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string()),
            cwd: self.cwd,
            cmd: self.command,
            exit: Some(exit_code),
            scopes_used: scopes,
            fs_diff,
            replay_context: Some(replay_context),
            transport: self.transport,
            graph_edges: None,
            policy_decision: None,
        };

        if let Some(ref mut output) = *self.context.output.write() {
            output.write_span(&span)?;
        }

        trace!(
            "Finished span {} with exit code {}",
            self.span_id,
            exit_code
        );
        Ok(())
    }

    pub fn get_span_id(&self) -> &str {
        &self.span_id
    }
}

fn hash_env_vars() -> Result<String> {
    let mut hasher = Sha256::new();

    for (key, value) in env::vars() {
        if !key.starts_with("SHIM_") && !key.starts_with("SUBSTRATE_") {
            hasher.update(format!("{}={}\n", key, value));
        }
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn get_umask() -> Result<u32> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempfile()?;
        let metadata = temp.metadata()?;
        let mode = metadata.permissions().mode();
        Ok(0o777 - (mode & 0o777))
    }

    #[cfg(not(unix))]
    {
        Ok(0o022)
    }
}

fn get_policy_git_hash() -> Result<String> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dirs::home_dir().unwrap().join(".substrate"))
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn policy_violation(cmd: &str, violation_type: &str, decision: &str) -> Result<()> {
    let span = create_span_builder()?
        .with_command(cmd)
        .with_policy_decision(PolicyDecision {
            action: "violation_observed".to_string(),
            reason: Some(format!("{}: {}", violation_type, decision)),
            restrictions: None,
        })
        .start()?;

    span.finish(126, vec![], None)?;
    Ok(())
}

// Graph DB integration is provided by the substrate-graph crate.

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::sync::{Arc, Barrier};
    use tempfile::TempDir;

    fn ensure_trace_context() -> TraceContext {
        let ctx = TraceContext::default();
        let _ = set_global_trace_context(ctx.clone());
        ctx
    }

    #[test]
    fn test_span_creation() {
        let span_id = new_span(None);
        assert!(span_id.starts_with("spn_"));
    }

    #[test]
    fn test_span_builder() {
        ensure_trace_context();

        let span = create_span_builder()
            .unwrap()
            .with_command("echo test")
            .with_cwd("/tmp")
            .with_world_id("wld_123")
            .start();

        assert!(span.is_ok());
        let active = span.unwrap();
        assert!(active.span_id.starts_with("spn_"));
    }

    #[test]
    fn test_trace_initialization() {
        let tmp_dir = TempDir::new().unwrap();
        let trace_path = tmp_dir.path().join("trace.jsonl");

        ensure_trace_context();
        let result = init_trace(Some(trace_path.clone()));
        assert!(result.is_ok());
        assert!(trace_path.exists());
    }

    #[test]
    fn test_policy_decision_serialization() {
        let decision = PolicyDecision {
            action: "allow".to_string(),
            reason: None,
            restrictions: Some(vec!["no_network".to_string()]),
        };

        let json = serde_json::to_string(&decision).unwrap();
        assert!(json.contains("\"action\":\"allow\""));
        assert!(json.contains("\"restrictions\":[\"no_network\"]"));
    }

    #[test]
    fn test_fs_diff() {
        let mut diff = FsDiff {
            writes: vec!["file1.txt".into()],
            mods: vec!["file2.txt".into()],
            deletes: vec![],
            truncated: false,
            tree_hash: None,
            summary: None,
            display_path: None,
        };
        diff.display_path = Some(std::collections::HashMap::from([(
            "file1.txt".to_string(),
            "C:\\path\\file1.txt".to_string(),
        )]));

        let json = serde_json::to_string(&diff).unwrap();
        assert!(json.contains("\"writes\":[\"file1.txt\"]"));
        assert!(json.contains("\"mods\":[\"file2.txt\"]"));
        assert!(json.contains("display_path"));
    }

    #[test]
    fn test_graph_edge() {
        let mut metadata = HashMap::new();
        metadata.insert("latency_ms".to_string(), serde_json::json!(42));

        let edge = GraphEdge {
            edge_type: EdgeType::DataFlow,
            from_span: "spn_123".to_string(),
            to_span: "spn_456".to_string(),
            metadata,
        };

        let json = serde_json::to_string(&edge).unwrap();
        assert!(json.contains("\"edge_type\":\"data_flow\""));
        assert!(json.contains("\"latency_ms\":42"));
    }

    #[test]
    fn test_env_hash() {
        // Test that hash_env_vars returns a consistent hash for the same environment
        let hash1 = hash_env_vars().unwrap();
        let hash2 = hash_env_vars().unwrap();

        // The hash should be consistent within the same test run
        assert_eq!(
            hash1, hash2,
            "Hash should be deterministic for the same environment"
        );

        // The hash should not be empty
        assert!(!hash1.is_empty(), "Hash should not be empty");

        // The hash should be a valid hex string of correct length (SHA256 = 64 hex chars)
        assert_eq!(hash1.len(), 64, "SHA256 hash should be 64 hex characters");
        assert!(
            hash1.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should be valid hex"
        );
    }

    #[test]
    fn test_rotation_on_write() {
        let tmp = TempDir::new().unwrap();
        let log_path = tmp.path().join("trace.jsonl");

        // Pre-fill with >1MB file to trigger rotation
        let large = vec![b'x'; 2 * 1024 * 1024];
        std::fs::write(&log_path, &large).unwrap();

        // Configure small rotation threshold and retention
        std::env::set_var("TRACE_LOG_MAX_MB", "1");
        std::env::set_var("TRACE_LOG_KEEP", "2");

        let ctx = ensure_trace_context();
        ctx.init_trace(Some(log_path.clone())).unwrap();

        let entry = serde_json::json!({"event_type":"test","ts":Utc::now().to_rfc3339()});
        append_to_trace(&entry).unwrap();

        // Original should be rotated to .1 and new file should be small
        let rotated = log_path.with_extension("jsonl.1");
        assert!(rotated.exists());
        assert!(log_path.exists());

        let orig_size = std::fs::metadata(&rotated).unwrap().len();
        assert!(orig_size >= 2 * 1024 * 1024);

        let new_size = std::fs::metadata(&log_path).unwrap().len();
        assert!(new_size < 16 * 1024);
    }

    #[test]
    fn test_rotation_retention_policy() {
        let tmp = TempDir::new().unwrap();
        let log_path = tmp.path().join("trace.jsonl");

        // Seed rotated files .1 and .2
        std::fs::write(log_path.with_extension("jsonl.1"), b"a").unwrap();
        std::fs::write(log_path.with_extension("jsonl.2"), b"b").unwrap();
        // Current file large enough to trigger rotation
        let large = vec![b'x'; 2 * 1024 * 1024];
        std::fs::write(&log_path, &large).unwrap();

        std::env::set_var("TRACE_LOG_MAX_MB", "1");
        std::env::set_var("TRACE_LOG_KEEP", "2");

        let ctx = ensure_trace_context();
        ctx.init_trace(Some(log_path.clone())).unwrap();

        // After rotation, .2 should become .2 (shifted from .1), and .1 should be the old current
        assert!(log_path.with_extension("jsonl.1").exists());
        assert!(log_path.with_extension("jsonl.2").exists());
        // .3 should NOT exist because keep=2
        assert!(!log_path.with_extension("jsonl.3").exists());
    }

    #[test]
    fn trace_contexts_do_not_share_policy_or_outputs() {
        let tmp = TempDir::new().unwrap();
        let log_a = tmp.path().join("trace_a.jsonl");
        let log_b = tmp.path().join("trace_b.jsonl");

        let ctx_a = TraceContext::new();
        let ctx_b = TraceContext::new();
        ctx_a.init_trace(Some(log_a.clone())).unwrap();
        ctx_b.init_trace(Some(log_b.clone())).unwrap();
        ctx_a.set_policy_id("policy-a");
        ctx_b.set_policy_id("policy-b");

        let barrier = Arc::new(Barrier::new(2));

        let span_a = {
            let ctx = ctx_a.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                let span = ctx
                    .create_span_builder()
                    .with_command("echo alpha")
                    .start()
                    .expect("start span for context A");
                let span_id = span.get_span_id().to_string();
                span.finish(0, vec![], None)
                    .expect("finish span for context A");
                span_id
            })
        };

        let span_b = {
            let ctx = ctx_b.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                let span = ctx
                    .create_span_builder()
                    .with_command("echo beta")
                    .start()
                    .expect("start span for context B");
                let span_id = span.get_span_id().to_string();
                span.finish(0, vec![], None)
                    .expect("finish span for context B");
                span_id
            })
        };

        let span_id_a = span_a.join().expect("thread a panicked");
        let span_id_b = span_b.join().expect("thread b panicked");

        let records_a: Vec<Value> = std::fs::read_to_string(&log_a)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect();
        let records_b: Vec<Value> = std::fs::read_to_string(&log_b)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect();

        assert!(!records_a.is_empty(), "context A should write spans");
        assert!(!records_b.is_empty(), "context B should write spans");

        assert!(records_a
            .iter()
            .all(|value| value["policy_id"] == "policy-a"));
        assert!(records_b
            .iter()
            .all(|value| value["policy_id"] == "policy-b"));

        assert!(records_a.iter().any(|value| value["span_id"] == span_id_a));
        assert!(records_b.iter().any(|value| value["span_id"] == span_id_b));

        assert!(records_a.iter().all(|value| value["span_id"] != span_id_b));
        assert!(records_b.iter().all(|value| value["span_id"] != span_id_a));

        assert_eq!(ctx_a.policy_id(), "policy-a");
        assert_eq!(ctx_b.policy_id(), "policy-b");
    }
}
