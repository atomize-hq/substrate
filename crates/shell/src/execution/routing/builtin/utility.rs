//! Core builtin handling and utility operations.

use super::super::path_env::ok_status;
use super::shim_actions::log_builtin_command;
use super::world_deps::handle_cd;
use crate::execution::ShellConfig;
use anyhow::Result;
use std::env;
use std::process::ExitStatus;

pub(crate) fn handle_builtin(
    config: &ShellConfig,
    command: &str,
    parent_cmd_id: &str,
) -> Result<Option<ExitStatus>> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(None);
    }

    let builtin_result = match parts[0] {
        "cd" => handle_cd(config, &parts)?,
        "pwd" => Some(handle_pwd()?),
        "unset" => Some(handle_unset(&parts[1..])?),
        "export" => handle_export(&parts[1..])?,
        _ => None,
    };

    if builtin_result.is_some() {
        log_builtin_command(config, command, parent_cmd_id)?;
    }

    Ok(builtin_result)
}

fn handle_pwd() -> Result<ExitStatus> {
    println!("{}", env::current_dir()?.display());
    ok_status()
}

fn handle_unset(vars: &[&str]) -> Result<ExitStatus> {
    for key in vars {
        env::remove_var(key);
    }
    ok_status()
}

fn handle_export(vars: &[&str]) -> Result<Option<ExitStatus>> {
    let mut handled = true;
    for part in vars {
        if let Some((k, v)) = part.split_once('=') {
            // Reject quotes or variable refs to avoid wrong semantics
            if v.contains('"') || v.contains('\'') || v.contains('$') {
                handled = false;
                break;
            }
            env::set_var(k, v);
        } else {
            handled = false;
            break;
        }
    }

    if handled {
        Ok(Some(ok_status()?))
    } else {
        // Defer complex cases to the external shell
        Ok(None)
    }
}
