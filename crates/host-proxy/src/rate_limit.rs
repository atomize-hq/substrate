//! Rate limiting for the host proxy.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::Result;
use tracing::debug;

use crate::RateLimitConfig;

/// Rate limiter for tracking request rates per agent.
pub struct RateLimiter {
    config: RateLimitConfig,
    agents: HashMap<String, AgentRateInfo>,
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new(config: &RateLimitConfig) -> Self {
        Self {
            config: config.clone(),
            agents: HashMap::new(),
        }
    }

    /// Check and update rate limit for an agent.
    pub fn check_and_update(&mut self, agent_id: &str) -> Result<()> {
        let now = Instant::now();
        let info = self
            .agents
            .entry(agent_id.to_string())
            .or_insert_with(|| AgentRateInfo::new(now));

        // Clean up old requests
        info.cleanup_old_requests(now);

        // Check rate limit
        let window = Duration::from_secs(60);
        let max_requests = if self.config.burst_enabled {
            (self.config.requests_per_minute as f32 * self.config.burst_multiplier) as u32
        } else {
            self.config.requests_per_minute
        };

        if info.request_count_in_window(now, window) >= max_requests {
            return Err(anyhow::anyhow!(
                "Rate limit exceeded: {} requests per minute",
                max_requests
            ));
        }

        // Check concurrent executions
        if info.concurrent_executions >= self.config.max_concurrent {
            return Err(anyhow::anyhow!(
                "Concurrent execution limit exceeded: {}",
                self.config.max_concurrent
            ));
        }

        // Record the request
        info.record_request(now);
        debug!(
            "Agent {} now has {} requests in window",
            agent_id,
            info.requests.len()
        );

        Ok(())
    }

    /// Record completion of an execution.
    pub fn complete_execution(&mut self, agent_id: &str) {
        if let Some(info) = self.agents.get_mut(agent_id) {
            if info.concurrent_executions > 0 {
                info.concurrent_executions -= 1;
            }
        }
    }

    /// Get current stats for an agent.
    pub fn get_agent_stats(&self, agent_id: &str) -> Option<AgentRateStats> {
        self.agents.get(agent_id).map(|info| {
            let now = Instant::now();
            AgentRateStats {
                requests_per_minute: info.request_count_in_window(now, Duration::from_secs(60)),
                concurrent_executions: info.concurrent_executions,
            }
        })
    }

    /// Clean up inactive agents.
    pub fn cleanup_inactive(&mut self, inactive_threshold: Duration) {
        let now = Instant::now();
        self.agents.retain(|_, info| {
            info.last_request
                .is_some_and(|last| now.duration_since(last) < inactive_threshold)
        });
    }
}

/// Rate limiting information for a single agent.
struct AgentRateInfo {
    requests: Vec<Instant>,
    concurrent_executions: u32,
    last_request: Option<Instant>,
}

impl AgentRateInfo {
    fn new(now: Instant) -> Self {
        Self {
            requests: Vec::new(),
            concurrent_executions: 0,
            last_request: Some(now),
        }
    }

    fn record_request(&mut self, now: Instant) {
        self.requests.push(now);
        self.last_request = Some(now);
        self.concurrent_executions += 1;
    }

    fn cleanup_old_requests(&mut self, now: Instant) {
        let window = Duration::from_secs(60);
        self.requests
            .retain(|&req_time| now.duration_since(req_time) < window);
    }

    fn request_count_in_window(&self, now: Instant, window: Duration) -> u32 {
        self.requests
            .iter()
            .filter(|&&req_time| now.duration_since(req_time) < window)
            .count() as u32
    }
}

/// Rate limiting statistics for an agent.
pub struct AgentRateStats {
    pub requests_per_minute: u32,
    pub concurrent_executions: u32,
}
