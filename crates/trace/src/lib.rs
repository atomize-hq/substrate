use anyhow::Result;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, trace, warn};
use uuid::Uuid;

static TRACE_OUTPUT: Lazy<RwLock<Option<TraceOutput>>> = Lazy::new(|| RwLock::new(None));

static CURRENT_POLICY_ID: Lazy<RwLock<String>> = 
    Lazy::new(|| RwLock::new("default".to_string()));

static WORLD_IMAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    pub graph_edges: Option<Vec<GraphEdge>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_decision: Option<PolicyDecision>,
}

// FsDiff is now imported from substrate_common
pub use substrate_common::FsDiff;

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
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        
        Ok(TraceOutput {
            writer: BufWriter::new(file),
            path: path.as_ref().to_path_buf(),
        })
    }
    
    fn max_bytes() -> u64 {
        const DEFAULT_MB: u64 = 100; // ~100MB
        env::var("TRACE_LOG_MAX_MB")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_MB)
            * 1024 * 1024
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
                // Drop current file handle before renaming
                // Replace writer with a sink, then drop explicitly
                let writer = std::mem::replace(&mut self.writer, BufWriter::new(
                    OpenOptions::new().create(true).append(true).open(&path)?
                ));
                drop(writer);

                // Shift older rotations: .(keep-1) -> .keep
                let keep = Self::keep_files();
                if keep > 0 {
                    for i in (1..=keep).rev() {
                        let from = path.with_extension(format!("jsonl.{}", i));
                        let to = path.with_extension(format!("jsonl.{}", i + 1));
                        let _ = fs::rename(&from, &to);
                    }
                    // Move current to .1
                    let bak = path.with_extension("jsonl.1");
                    let _ = fs::rename(&path, &bak);
                } else {
                    // No retention, just truncate by renaming away
                    let bak = path.with_extension("jsonl.1");
                    let _ = fs::rename(&path, &bak);
                }

                // Recreate fresh file and writer
                let file = OpenOptions::new().create(true).append(true).open(&path)?;
                self.writer = BufWriter::new(file);
            }
        }
        Ok(())
    }
    
    fn write_span(&mut self, span: &Span) -> Result<()> {
        self.rotate_if_needed()?;
        let json = serde_json::to_string(span)?;
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

pub fn init_trace(path: Option<PathBuf>) -> Result<()> {
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
    
    let output = TraceOutput::new(trace_path)?;
    *TRACE_OUTPUT.write() = Some(output);
    
    debug!("Initialized trace output");
    Ok(())
}

pub fn new_span(_parent: Option<&str>) -> String {
    format!("spn_{}", Uuid::now_v7())
}

pub fn set_policy_id(policy_id: &str) {
    *CURRENT_POLICY_ID.write() = policy_id.to_string();
}

pub fn get_policy_id() -> String {
    CURRENT_POLICY_ID.read().clone()
}

pub fn create_span_builder() -> SpanBuilder {
    SpanBuilder::new()
}

pub struct SpanBuilder {
    span: Span,
}

impl SpanBuilder {
    fn new() -> Self {
        let session_id = env::var("SHIM_SESSION_ID")
            .unwrap_or_else(|_| format!("ses_{}", Uuid::now_v7()));
        
        let agent_id = env::var("SUBSTRATE_AGENT_ID")
            .unwrap_or_else(|_| "human".to_string());
        
        let component = if env::var("SUBSTRATE_SHELL").is_ok() {
            "shell"
        } else if env::var("SHIM_ORIGINAL_PATH").is_ok() {
            "shim"
        } else {
            "unknown"
        };
        
        SpanBuilder {
            span: Span {
                ts: Utc::now(),
                event_type: "command_start".to_string(),
                session_id,
                span_id: new_span(None),
                parent_span: env::var("SHIM_PARENT_SPAN").ok(),
                component: component.to_string(),
                world_id: env::var("SUBSTRATE_WORLD_ID").ok(),
                policy_id: get_policy_id(),
                agent_id,
                cwd: env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                cmd: String::new(),
                exit: None,
                scopes_used: Vec::new(),
                fs_diff: None,
                replay_context: None,
                graph_edges: None,
                policy_decision: None,
            }
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
        
        if let Some(ref mut output) = *TRACE_OUTPUT.write() {
            output.write_span(&self.span)?;
        }
        
        Ok(ActiveSpan {
            span_id,
            start_time: Utc::now(),
            command: self.span.cmd,
            cwd: self.span.cwd,
        })
    }
}

pub struct ActiveSpan {
    pub span_id: String,
    start_time: DateTime<Utc>,
    command: String,
    cwd: String,
}

impl ActiveSpan {
    pub fn finish(
        self, 
        exit_code: i32, 
        scopes: Vec<String>, 
        fs_diff: Option<FsDiff>
    ) -> Result<()> {
        let replay_context = build_replay_context()?;
        
        let span = Span {
            ts: Utc::now(),
            event_type: "command_complete".to_string(),
            session_id: env::var("SHIM_SESSION_ID")
                .unwrap_or_else(|_| format!("ses_{}", Uuid::now_v7())),
            span_id: self.span_id.clone(),
            parent_span: env::var("SHIM_PARENT_SPAN").ok(),
            component: if env::var("SUBSTRATE_SHELL").is_ok() { "shell" } else { "shim" }.to_string(),
            world_id: env::var("SUBSTRATE_WORLD_ID").ok(),
            policy_id: get_policy_id(),
            agent_id: env::var("SUBSTRATE_AGENT_ID")
                .unwrap_or_else(|_| "human".to_string()),
            cwd: self.cwd,
            cmd: self.command,
            exit: Some(exit_code),
            scopes_used: scopes,
            fs_diff,
            replay_context: Some(replay_context),
            graph_edges: None,
            policy_decision: None,
        };
        
        if let Some(ref mut output) = *TRACE_OUTPUT.write() {
            output.write_span(&span)?;
        }
        
        trace!("Finished span {} with exit code {}", self.span_id, exit_code);
        Ok(())
    }
    
    pub fn get_span_id(&self) -> &str {
        &self.span_id
    }
}

fn build_replay_context() -> Result<ReplayContext> {
    Ok(ReplayContext {
        path: env::var("PATH").ok(),
        env_hash: hash_env_vars()?,
        umask: get_umask()?,
        locale: env::var("LANG").ok(),
        cwd: env::current_dir()?.to_string_lossy().to_string(),
        policy_id: get_policy_id(),
        policy_commit: get_policy_git_hash().ok(),
        world_image_version: WORLD_IMAGE_VERSION.to_string(),
    })
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
        .args(&["rev-parse", "HEAD"])
        .current_dir(dirs::home_dir().unwrap().join(".substrate"))
        .output()?;
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn policy_violation(cmd: &str, violation_type: &str, decision: &str) -> Result<()> {
    let span = create_span_builder()
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

pub fn budget_exceeded(agent_id: &str, budget_type: &str) -> Result<()> {
    warn!("Budget exceeded for agent {}: {}", agent_id, budget_type);
    
    let span = create_span_builder()
        .with_command(&format!("budget_exceeded:{}", budget_type))
        .with_policy_decision(PolicyDecision {
            action: "deny".to_string(),
            reason: Some(format!("Budget {} exceeded for agent {}", budget_type, agent_id)),
            restrictions: None,
        })
        .start()?;
    
    span.finish(126, vec![], None)?;
    Ok(())
}

pub fn append_to_trace(entry: &serde_json::Value) -> Result<()> {
    if let Some(ref mut output) = *TRACE_OUTPUT.write() {
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

pub fn load_span(span_id: &str) -> Result<Span> {
    use std::io::{BufRead, BufReader};
    
    let trace_path = TRACE_OUTPUT.read()
        .as_ref()
        .map(|o| o.path.clone())
        .ok_or_else(|| anyhow::anyhow!("Trace not initialized"))?;
    
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
    
    Err(anyhow::anyhow!("Span {} not found", span_id))
}

// Graph DB integration is provided by the substrate-graph crate.

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_span_creation() {
        let span_id = new_span(None);
        assert!(span_id.starts_with("spn_"));
    }
    
    #[test]
    fn test_span_builder() {
        let span = create_span_builder()
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
        let diff = FsDiff {
            writes: vec!["file1.txt".into()],
            mods: vec!["file2.txt".into()],
            deletes: vec![],
            truncated: false,
            tree_hash: None,
            summary: None,
        };
        
        let json = serde_json::to_string(&diff).unwrap();
        assert!(json.contains("\"writes\":[\"file1.txt\"]"));
        assert!(json.contains("\"mods\":[\"file2.txt\"]"));
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
        let hash1 = hash_env_vars().unwrap();
        let hash2 = hash_env_vars().unwrap();
        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }
    
}
