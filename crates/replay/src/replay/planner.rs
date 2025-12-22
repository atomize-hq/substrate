use anyhow::Result;
use substrate_broker::{set_global_broker, BrokerHandle};

use super::{ExecutionResult, ExecutionState};
use crate::replay::executor::{
    execute_direct, execute_with_world_backends, record_replay_strategy,
};
use crate::replay::helpers::{replay_verbose, world_isolation_available};
use serde_json::json;

/// Execute a command in an isolated world when possible.
pub async fn execute_in_world(
    state: &ExecutionState,
    timeout_secs: u64,
) -> Result<ExecutionResult> {
    let _ = set_global_broker(BrokerHandle::new());

    if !world_isolation_available() {
        if replay_verbose() {
            eprintln!("[replay] world strategy: direct (world isolation unavailable)");
        }
        record_replay_strategy(
            state,
            "direct",
            None,
            Some("world isolation unavailable"),
            json!({"requested_origin": state.target_origin.as_str()}),
        );
        return execute_direct(state, timeout_secs).await;
    }

    execute_with_world_backends(state, timeout_secs).await
}

/// Replay a command sequence (multiple related commands)
pub async fn replay_sequence(
    states: Vec<ExecutionState>,
    timeout_secs: u64,
    use_world: bool,
) -> Result<Vec<ExecutionResult>> {
    let mut results = Vec::new();

    for state in states {
        let result = if use_world {
            execute_in_world(&state, timeout_secs).await?
        } else {
            execute_direct(&state, timeout_secs).await?
        };

        if result.exit_code != 0 {
            tracing::warn!(
                "Command failed with exit code {}: {}",
                result.exit_code,
                state.command
            );
        }

        results.push(result);
    }

    Ok(results)
}
