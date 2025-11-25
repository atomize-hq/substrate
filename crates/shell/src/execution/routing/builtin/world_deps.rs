//! Builtin helpers tied to world root and dependency-aware navigation.

use super::super::path_env::{canonicalize_cd_target, enforce_caged_destination, ok_status};
use crate::execution::ShellConfig;
use anyhow::Result;
use std::env;
use std::process::ExitStatus;

pub(super) fn handle_cd(config: &ShellConfig, parts: &[&str]) -> Result<Option<ExitStatus>> {
    let target = match parts.get(1).copied() {
        None => "~".to_string(),
        Some("-") => {
            if let Ok(oldpwd) = env::var("OLDPWD") {
                println!("{oldpwd}");
                oldpwd
            } else {
                "~".to_string()
            }
        }
        Some(p) => p.to_string(),
    };

    let expanded = shellexpand::tilde(&target);
    let prev = env::current_dir()?;
    let requested = canonicalize_cd_target(&prev, expanded.as_ref())?;
    let (destination, warning) = enforce_caged_destination(&config.world_root, &prev, requested);
    if let Some(message) = warning {
        eprintln!("{message}");
    }
    env::set_current_dir(&destination)?;
    env::set_var("OLDPWD", prev);
    env::set_var("PWD", env::current_dir()?.display().to_string());
    Ok(Some(ok_status()?))
}
