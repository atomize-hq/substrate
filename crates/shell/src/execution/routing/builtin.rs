//! Builtin command handling for shell routing.

use super::path_env::{canonicalize_cd_target, enforce_caged_destination, ok_status};
use crate::execution::ShellConfig;
use anyhow::Result;
use serde_json::json;
use std::env;
use uuid::Uuid;

pub(crate) fn handle_builtin(
    config: &ShellConfig,
    command: &str,
    parent_cmd_id: &str,
) -> Result<Option<std::process::ExitStatus>> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(None);
    }

    let builtin_result = match parts[0] {
        "cd" => {
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
            let (destination, warning) =
                enforce_caged_destination(&config.world_root, &prev, requested);
            if let Some(message) = warning {
                eprintln!("{message}");
            }
            env::set_current_dir(&destination)?;
            env::set_var("OLDPWD", prev);
            env::set_var("PWD", env::current_dir()?.display().to_string());
            Some(ok_status()?)
        }
        "pwd" => {
            println!("{}", env::current_dir()?.display());
            Some(ok_status()?)
        }
        "unset" => {
            for k in &parts[1..] {
                env::remove_var(k);
            }
            Some(ok_status()?)
        }
        "export" => {
            let mut handled = true;
            for part in &parts[1..] {
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
                Some(ok_status()?)
            } else {
                // Defer complex cases to the external shell
                None
            }
        }
        _ => None,
    };

    // Log builtin command if we handled it
    if builtin_result.is_some() {
        let builtin_cmd_id = Uuid::now_v7().to_string();
        let extra = json!({ "parent_cmd_id": parent_cmd_id });

        // Apply redaction to builtin commands
        let redacted_command = {
            let tokens = shell_words::split(command).unwrap_or_else(|_| vec![command.to_string()]);
            let mut out = Vec::new();
            let mut i = 0;

            while i < tokens.len() {
                let t = &tokens[i];

                // Check for environment variable exports with sensitive names
                if tokens.len() > 1 && tokens[0] == "export" && t.contains('=') {
                    if let Some((k, _)) = t.split_once('=') {
                        let kl = k.to_lowercase();
                        if kl.contains("token")
                            || kl.contains("password")
                            || kl.contains("secret")
                            || kl.contains("apikey")
                            || kl.contains("api_key")
                        {
                            out.push(format!("{k}=***"));
                            i += 1;
                            continue;
                        }
                    }
                }

                out.push(t.clone());
                i += 1;
            }
            out.join(" ")
        };

        super::log_command_event(
            config,
            "builtin_command",
            &redacted_command,
            &builtin_cmd_id,
            Some(extra),
        )?;
    }

    Ok(builtin_result)
}
