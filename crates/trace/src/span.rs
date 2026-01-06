use crate::context::{create_span_builder, TraceContext};
use crate::FsDiff;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use substrate_common::{WorldFsStrategy, WorldFsStrategyFallbackReason};
use tracing::trace;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionOrigin {
    Host,
    World,
}

impl ExecutionOrigin {
    pub fn flipped(self) -> Self {
        match self {
            ExecutionOrigin::Host => ExecutionOrigin::World,
            ExecutionOrigin::World => ExecutionOrigin::Host,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionOrigin::Host => "host",
            ExecutionOrigin::World => "world",
        }
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
    pub execution_origin: Option<ExecutionOrigin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_edges: Option<Vec<GraphEdge>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_decision: Option<PolicyDecision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_fs_strategy_primary: Option<WorldFsStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_fs_strategy_final: Option<WorldFsStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_fs_strategy_fallback_reason: Option<WorldFsStrategyFallbackReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMeta {
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socket_activation: Option<bool>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub term: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_origin: Option<ExecutionOrigin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<TransportMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_root_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_root_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caged: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_fs_mode: Option<String>,
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

pub struct SpanBuilder {
    span: Span,
    context: TraceContext,
}

impl SpanBuilder {
    pub(crate) fn new(context: TraceContext) -> Self {
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
                execution_origin: None,
                graph_edges: None,
                policy_decision: None,
                world_fs_strategy_primary: None,
                world_fs_strategy_final: None,
                world_fs_strategy_fallback_reason: None,
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

        if let Some(ref mut output) = *self.context.output_write() {
            output.write_span(&self.span)?;
        }

        Ok(ActiveSpan {
            span_id,
            command: self.span.cmd,
            cwd: self.span.cwd,
            transport: None,
            execution_origin: Some(ExecutionOrigin::Host),
            context: self.context,
            world_fs_strategy_primary: None,
            world_fs_strategy_final: None,
            world_fs_strategy_fallback_reason: None,
        })
    }
}

pub struct ActiveSpan {
    pub span_id: String,
    command: String,
    cwd: String,
    transport: Option<TransportMeta>,
    execution_origin: Option<ExecutionOrigin>,
    context: TraceContext,
    world_fs_strategy_primary: Option<WorldFsStrategy>,
    world_fs_strategy_final: Option<WorldFsStrategy>,
    world_fs_strategy_fallback_reason: Option<WorldFsStrategyFallbackReason>,
}

impl ActiveSpan {
    pub fn set_transport(&mut self, transport: TransportMeta) {
        self.transport = Some(transport);
    }

    pub fn clear_transport(&mut self) {
        self.transport = None;
    }

    pub fn set_execution_origin(&mut self, origin: ExecutionOrigin) {
        self.execution_origin = Some(origin);
    }

    pub fn set_world_fs_strategy(
        &mut self,
        primary: WorldFsStrategy,
        final_strategy: WorldFsStrategy,
        fallback_reason: WorldFsStrategyFallbackReason,
    ) {
        self.world_fs_strategy_primary = Some(primary);
        self.world_fs_strategy_final = Some(final_strategy);
        self.world_fs_strategy_fallback_reason = Some(fallback_reason);
    }

    pub fn execution_origin(&self) -> ExecutionOrigin {
        self.execution_origin.unwrap_or(ExecutionOrigin::Host)
    }

    pub fn finish(
        self,
        exit_code: i32,
        scopes: Vec<String>,
        fs_diff: Option<FsDiff>,
    ) -> Result<()> {
        let origin = self.execution_origin.unwrap_or(ExecutionOrigin::Host);
        let replay_context = self
            .context
            .build_replay_context(self.transport.clone(), origin)?;

        // ADR-0004/WO0 trace contract: these fields must be present on command_complete events,
        // even when the caller did not set them explicitly (e.g., host-only execution paths).
        let world_fs_strategy_primary = Some(
            self.world_fs_strategy_primary
                .unwrap_or(WorldFsStrategy::Overlay),
        );
        let world_fs_strategy_final = Some(self.world_fs_strategy_final.unwrap_or(match origin {
            ExecutionOrigin::World => WorldFsStrategy::Overlay,
            ExecutionOrigin::Host => WorldFsStrategy::Host,
        }));
        let world_fs_strategy_fallback_reason = Some(
            self.world_fs_strategy_fallback_reason
                .unwrap_or(WorldFsStrategyFallbackReason::None),
        );

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
            execution_origin: Some(origin),
            graph_edges: None,
            policy_decision: None,
            world_fs_strategy_primary,
            world_fs_strategy_final,
            world_fs_strategy_fallback_reason,
        };

        if let Some(ref mut output) = *self.context.output_write() {
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

pub fn new_span(_parent: Option<&str>) -> String {
    format!("spn_{}", Uuid::now_v7())
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
