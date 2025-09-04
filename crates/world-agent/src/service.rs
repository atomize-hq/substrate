//! Core service implementation for world agent.

use agent_api_types::{Budget, ExecuteRequest, ExecuteResponse};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use world_api::{WorldBackend, WorldHandle, WorldSpec};

/// Main service running inside the world.
#[derive(Clone)]
pub struct WorldAgentService {
    backend: Arc<dyn WorldBackend>,
    worlds: Arc<RwLock<HashMap<String, WorldHandle>>>,
    budgets: Arc<RwLock<HashMap<String, AgentBudgetTracker>>>,
}

pub struct AgentBudgetTracker {
    agent_id: String,
    execs_remaining: std::sync::atomic::AtomicU32,
    runtime_remaining_ms: std::sync::atomic::AtomicU64,
    egress_remaining: std::sync::atomic::AtomicU64,
}

impl AgentBudgetTracker {
    pub fn new(agent_id: &str, budget: Budget) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            execs_remaining: std::sync::atomic::AtomicU32::new(budget.max_execs.unwrap_or(1000)),
            runtime_remaining_ms: std::sync::atomic::AtomicU64::new(
                budget.max_runtime_ms.unwrap_or(300_000),
            ),
            egress_remaining: std::sync::atomic::AtomicU64::new(
                budget.max_egress_bytes.unwrap_or(100_000_000),
            ),
        }
    }

    pub fn can_execute(&self) -> bool {
        use std::sync::atomic::Ordering;
        self.execs_remaining.load(Ordering::SeqCst) > 0
    }

    pub fn decrement_exec(&self) {
        use std::sync::atomic::Ordering;
        self.execs_remaining.fetch_sub(1, Ordering::SeqCst);
    }
}

impl WorldAgentService {
    pub fn new() -> Result<Self> {
        let backend = Self::create_backend()?;

        Ok(Self {
            backend,
            worlds: Arc::new(RwLock::new(HashMap::new())),
            budgets: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    #[cfg(target_os = "linux")]
    fn create_backend() -> Result<Arc<dyn WorldBackend>> {
        use world::LinuxLocalBackend;
        Ok(Arc::new(LinuxLocalBackend::new()))
    }

    #[cfg(not(target_os = "linux"))]
    fn create_backend() -> Result<Arc<dyn WorldBackend>> {
        anyhow::bail!("World agent only supported on Linux inside VMs")
    }

    /// Execute a command with budget tracking.
    pub async fn execute(&self, req: ExecuteRequest) -> Result<ExecuteResponse> {
        // Validate agent_id
        if req.agent_id.is_empty() {
            anyhow::bail!("agent_id is required for API calls");
        }

        // Apply and track budget
        if let Some(budget) = req.budget {
            {
                let mut budgets = self.budgets.write().unwrap();
                let tracker = budgets
                    .entry(req.agent_id.clone())
                    .or_insert_with(|| AgentBudgetTracker::new(&req.agent_id, budget));

                // Check budget before execution
                if !tracker.can_execute() {
                    anyhow::bail!("Budget exceeded: max executions reached");
                }

                tracker.decrement_exec();
            }
        }

        // Create world spec from request
        let spec = WorldSpec {
            reuse_session: true,
            isolate_network: true,
            limits: world_api::ResourceLimits::default(),
            enable_preload: false,
            allowed_domains: vec![], // TODO: Get from policy
            project_dir: req
                .cwd
                .clone()
                .map(|p| std::path::PathBuf::from(p))
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default()),
        };

        // Ensure world exists
        let world = self
            .backend
            .ensure_session(&spec)
            .context("Failed to ensure session world")?;

        // Prepare execution request
        let exec_req = world_api::ExecRequest {
            cmd: req.cmd,
            cwd: req
                .cwd
                .map(|p| std::path::PathBuf::from(p))
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default()),
            env: req.env.unwrap_or_default(),
            pty: req.pty,
            span_id: None,
        };

        // Execute command
        let result = self
            .backend
            .exec(&world, exec_req)
            .context("Command execution failed")?;

        // Generate span ID
        let span_id = format!("spn_{}", uuid::Uuid::now_v7());

        Ok(ExecuteResponse {
            exit: result.exit,
            span_id,
            stdout_b64: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result.stdout),
            stderr_b64: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result.stderr),
            scopes_used: result.scopes_used,
        })
    }

    /// Get trace information for a span.
    pub async fn get_trace(&self, span_id: &str) -> Result<serde_json::Value> {
        // TODO: Implement trace retrieval
        Ok(serde_json::json!({
            "span_id": span_id,
            "status": "not_implemented"
        }))
    }

    /// Request additional scopes.
    pub async fn request_scopes(&self, _scopes: Vec<String>) -> Result<serde_json::Value> {
        // TODO: Implement scope request handling
        Ok(serde_json::json!({
            "status": "not_implemented"
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_tracker() {
        let budget = Budget {
            max_execs: Some(5),
            max_runtime_ms: Some(10000),
            max_egress_bytes: Some(1000000),
        };

        let tracker = AgentBudgetTracker::new("test-agent", budget);

        assert!(tracker.can_execute());

        // Use up all executions
        for _ in 0..5 {
            tracker.decrement_exec();
        }

        assert!(!tracker.can_execute());
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn test_service_creation() {
        let service = WorldAgentService::new().unwrap();
        assert_eq!(service.worlds.read().unwrap().len(), 0);
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn test_service_creation_fails_on_non_linux() {
        let result = WorldAgentService::new();
        assert!(result.is_err());
    }
}
