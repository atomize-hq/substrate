use anyhow::Result;

use crate::execution::cli::{Cli, SyncDirectionArg, WorkspaceSyncArgs};
use crate::execution::config_model;
use crate::execution::config_model::SubstrateConfig;
use crate::execution::workspace_cmd;
use crate::execution::ShellConfig;
use substrate_common::WorldRootMode;

fn first_non_empty_line(value: &str) -> Option<&str> {
    value.lines().map(str::trim).find(|l| !l.is_empty())
}

fn cli_for_auto_sync(config: &ShellConfig) -> Cli {
    let (caged, uncaged) = match config.cli_caged {
        Some(true) => (true, false),
        Some(false) => (false, true),
        None => (false, false),
    };

    let anchor_mode = config.cli_anchor_mode.map(|mode| match mode {
        WorldRootMode::Project => crate::execution::cli::AnchorModeArg::Workspace,
        WorldRootMode::FollowCwd => crate::execution::cli::AnchorModeArg::FollowCwd,
        WorldRootMode::Custom => crate::execution::cli::AnchorModeArg::Custom,
    });

    Cli {
        command: None,
        script: None,
        ci_mode: config.ci_mode,
        no_exit_on_error: config.no_exit_on_error,
        use_pty: false,
        shell: Some(config.shell_path.clone()),
        version_json: false,
        shim_status: false,
        shim_status_json: false,
        shim_skip: config.skip_shims,
        shim_deploy: false,
        shim_remove: false,
        async_repl: config.async_repl,
        legacy_repl: !config.async_repl,
        repl_host_escape: config.repl_host_escape,
        trace: None,
        replay: None,
        replay_verbose: false,
        flip_world: false,
        caged,
        uncaged,
        anchor_mode,
        anchor_path: config.cli_anchor_path.clone(),
        world: config.cli_world,
        no_world: config.cli_no_world,
        sub: None,
    }
}

pub(crate) fn run_auto_sync_if_enabled(
    config: &ShellConfig,
    effective_config: &SubstrateConfig,
) -> Result<i32> {
    if !effective_config.sync.auto_sync {
        return Ok(0);
    }

    let direction = match effective_config.sync.direction {
        config_model::SyncDirection::FromHost => {
            // Spec: direction=from_host is a no-op for auto-sync (WS5 will implement).
            return Ok(0);
        }
        config_model::SyncDirection::FromWorld => SyncDirectionArg::FromWorld,
        config_model::SyncDirection::Both => SyncDirectionArg::Both,
    };

    let cli = cli_for_auto_sync(config);
    let sync_args = WorkspaceSyncArgs {
        dry_run: false,
        direction: Some(direction),
        conflict_policy: None,
        exclude: Vec::new(),
        verbose: false,
    };

    match workspace_cmd::run_workspace_sync_for_auto_sync(&sync_args, &cli) {
        Ok((code, failure_reason)) => {
            if code != 0 {
                let reason = failure_reason
                    .as_deref()
                    .and_then(first_non_empty_line)
                    .unwrap_or("workspace sync failed");
                eprintln!("auto-sync failed: {reason}");
            }
            Ok(code)
        }
        Err(err) => {
            let reason = format!("{:#}", err);
            let reason = first_non_empty_line(&reason).unwrap_or("workspace sync failed");
            eprintln!("auto-sync failed: {reason}");
            Ok(workspace_cmd::workspace_cmd_exit_code_for_error(&err))
        }
    }
}
