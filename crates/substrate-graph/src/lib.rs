// Basic scaffold for graph database integration
// TODO: Expand this after Phase 4 completion

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    pub backend: String,
    pub db_path: std::path::PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub change: String,
}

#[derive(thiserror::Error, Debug)]
pub enum GraphError {
    #[error("internal: {0}")]
    Internal(String),
}

pub trait GraphService {
    fn ensure_schema(&mut self) -> Result<(), GraphError>;
    fn ingest_span(&mut self, span: &substrate_trace::Span) -> Result<(), GraphError>;
    fn what_changed(&self, span_id: &str, limit: usize) -> Result<Vec<FileChange>, GraphError>;
    fn status(&self) -> Result<String, GraphError>;
}

pub struct MockGraphService {
    spans: HashMap<String, substrate_trace::Span>,
}

impl MockGraphService {
    pub fn connect(_cfg: GraphConfig) -> Result<Self, GraphError> {
        Ok(Self {
            spans: HashMap::new(),
        })
    }
}

impl GraphService for MockGraphService {
    fn ensure_schema(&mut self) -> Result<(), GraphError> {
        Ok(())
    }

    fn ingest_span(&mut self, span: &substrate_trace::Span) -> Result<(), GraphError> {
        self.spans.insert(span.span_id.clone(), span.clone());
        Ok(())
    }

    fn what_changed(&self, span_id: &str, limit: usize) -> Result<Vec<FileChange>, GraphError> {
        let mut out = Vec::new();
        if let Some(span) = self.spans.get(span_id) {
            if let Some(diff) = &span.fs_diff {
                for p in &diff.writes {
                    if out.len() >= limit {
                        break;
                    }
                    out.push(FileChange {
                        path: p.to_string_lossy().into_owned(),
                        change: "write".into(),
                    });
                }
                for p in &diff.mods {
                    if out.len() >= limit {
                        break;
                    }
                    out.push(FileChange {
                        path: p.to_string_lossy().into_owned(),
                        change: "modify".into(),
                    });
                }
                for p in &diff.deletes {
                    if out.len() >= limit {
                        break;
                    }
                    out.push(FileChange {
                        path: p.to_string_lossy().into_owned(),
                        change: "delete".into(),
                    });
                }
            }
        }
        Ok(out)
    }

    fn status(&self) -> Result<String, GraphError> {
        Ok(format!("mock backend | spans: {}", self.spans.len()))
    }
}

pub fn connect_mock(cfg: GraphConfig) -> Result<MockGraphService, GraphError> {
    MockGraphService::connect(cfg)
}

pub fn default_graph_path() -> Result<std::path::PathBuf> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
    Ok(home.join(".substrate").join("graph"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default_graph_path() {
        let path = default_graph_path().unwrap();
        assert!(path.ends_with(".substrate/graph"));
    }
}
