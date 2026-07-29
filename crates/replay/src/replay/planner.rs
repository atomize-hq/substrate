use anyhow::Result;
use substrate_broker::{set_global_broker, BrokerHandle};

use super::{ExecutionResult, ExecutionState};
use crate::replay::executor::{
    execute_direct, execute_with_world_backends, record_replay_strategy,
};
use crate::replay::helpers::{replay_verbose, world_isolation_available};
use serde_json::json;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReplayPlannerPlatform {
    Linux,
    Macos,
    Windows,
    Unsupported,
}

impl ReplayPlannerPlatform {
    fn current() -> Self {
        if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Unsupported
        }
    }
}

fn should_short_circuit_to_direct(
    state: &ExecutionState,
    platform: ReplayPlannerPlatform,
    world_available: bool,
) -> bool {
    if matches!(
        platform,
        ReplayPlannerPlatform::Macos | ReplayPlannerPlatform::Windows
    ) && state.target_origin == substrate_trace::ExecutionOrigin::World
    {
        return false;
    }

    !world_available
}

/// Execute a command in an isolated world when possible.
pub async fn execute_in_world(
    state: &ExecutionState,
    timeout_secs: u64,
) -> Result<ExecutionResult> {
    let _ = set_global_broker(BrokerHandle::new());

    if should_short_circuit_to_direct(
        state,
        ReplayPlannerPlatform::current(),
        world_isolation_available(),
    ) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn world_state() -> ExecutionState {
        ExecutionState {
            raw_cmd: "echo hi".to_string(),
            command: "echo".to_string(),
            args: vec!["hi".to_string()],
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            env: HashMap::new(),
            stdin: None,
            session_id: "session".to_string(),
            span_id: "span".to_string(),
            recorded_origin: substrate_trace::ExecutionOrigin::World,
            recorded_origin_source: None,
            recorded_transport: None,
            target_origin: substrate_trace::ExecutionOrigin::World,
            origin_reason: None,
            origin_reason_code: None,
            world_disable_source: None,
            platform_bootstrap_mapping: None,
        }
    }

    #[test]
    fn windows_world_replay_does_not_short_circuit_before_authority_validation() {
        let state = world_state();

        assert!(
            !should_short_circuit_to_direct(&state, ReplayPlannerPlatform::Windows, false),
            "Windows world replay must reach backend authority validation even when ambient world availability probes report false"
        );
    }

    #[test]
    fn macos_world_replay_does_not_short_circuit_before_authority_validation() {
        let state = world_state();

        assert!(
            !should_short_circuit_to_direct(&state, ReplayPlannerPlatform::Macos, false),
            "macOS world replay must reach backend authority validation even when ambient world availability probes report false"
        );
    }

    #[test]
    fn linux_world_replay_keeps_direct_short_circuit_when_world_is_unavailable() {
        let state = world_state();

        assert!(
            should_short_circuit_to_direct(&state, ReplayPlannerPlatform::Linux, false),
            "Linux replay should preserve the existing direct fallback when world isolation is unavailable"
        );
    }
}
