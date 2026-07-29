mod bootstrap;
mod logging;
mod policy;

use self::bootstrap::{
    execute_command, execute_real_binary_bypass, handle_bypass_mode, persist_original_path,
    resolve_command_binary,
};
use self::logging::{collect_world_telemetry, hint_payload, log_spawn_failure, ManagerHintEngine};
use self::policy::{evaluate_policy, PolicyResult};
use crate::context::{
    default_platform_bootstrap_mapping_v1, resolve_install_bootstrap_context_from_invocation,
    resolve_invoked_path, world_features_enabled, ShimContext, SUBSTRATE_WORLD_ID_VAR,
    SUBSTRATE_WORLD_PROJECT_DIR_VAR,
};
use crate::logger::{log_execution, ExecutionLogMetadata};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::{Instant, SystemTime};
use std::{env, fs};
use substrate_broker::{
    detect_profile, policy_mode, resolve_effective_policy_with_explain_from_global_source,
    set_global_broker, BrokerHandle, PolicyMode,
};
use substrate_trace::{set_global_trace_context, TraceContext};
use uuid::Uuid;

/// Main shim execution function
pub fn run_shim() -> Result<i32> {
    let ctx = ShimContext::from_current_exe()?;
    let running_executable = env::current_exe()?;
    let invoked = env::args_os()
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("shim invocation witness is missing"))?;
    let invoked_path = resolve_invoked_path(&invoked, &running_executable)?;
    let install_bootstrap_context =
        resolve_install_bootstrap_context_from_invocation(&invoked_path, &running_executable)?;
    let selected_host_prefix =
        PathBuf::from(&install_bootstrap_context.context.selected_host_prefix);
    let trace_context = TraceContext::explicit_product(selected_host_prefix.as_path())?;

    let _ = set_global_broker(BrokerHandle::new());
    let _ = set_global_trace_context(trace_context.clone());

    persist_original_path(&ctx);

    if ShimContext::is_bypass_enabled() {
        return handle_bypass_mode();
    }

    if ctx.should_skip_shimming() {
        return execute_real_binary_bypass(&ctx);
    }

    ctx.setup_execution_env();

    let mut hint_engine = ManagerHintEngine::new(&install_bootstrap_context);
    let capture_stderr = hint_engine
        .as_ref()
        .map(|engine| engine.is_active())
        .unwrap_or(false);

    let real_binary = resolve_command_binary(&ctx)?;
    let args: Vec<_> = env::args_os().skip(1).collect();
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let argv: Vec<String> = std::iter::once(ctx.command_name.clone())
        .chain(args.iter().map(|s| s.to_string_lossy().to_string()))
        .collect();
    let command_str = argv.join(" ");

    let mode = policy_mode();
    if mode != PolicyMode::Disabled {
        let selected_host_prefix_canonical = selected_host_prefix
            .canonicalize()
            .unwrap_or_else(|_| selected_host_prefix.clone());
        let cwd_relative_to_selected_prefix = cwd.canonicalize().ok().and_then(|canonical_cwd| {
            canonical_cwd
                .strip_prefix(&selected_host_prefix_canonical)
                .ok()
                .map(PathBuf::from)
        });
        let workspace_dir = selected_host_prefix.join(".substrate");
        let workspace_marker = workspace_dir.join("workspace.yaml");
        let workspace_disabled = workspace_dir.join("workspace.disabled");
        let workspace_policy = workspace_dir.join("policy.yaml");
        let global_policy = selected_host_prefix.join("policy.yaml");
        let global_policy_bytes = global_policy
            .is_file()
            .then(|| {
                fs::read(&global_policy)
                    .with_context(|| format!("failed to read {}", global_policy.display()))
            })
            .transpose()?;
        let workspace_policy_bytes =
            if workspace_marker.is_file() && !workspace_disabled.exists() {
                if workspace_policy.is_file() {
                    Some(fs::read(&workspace_policy).with_context(|| {
                        format!("failed to read {}", workspace_policy.display())
                    })?)
                } else {
                    None
                }
            } else {
                global_policy_bytes.clone()
            };

        let isolated_root = env::temp_dir().join(format!(
            "substrate-shim-bound-policy-root-{}",
            Uuid::now_v7()
        ));
        let bound_cwd = match cwd_relative_to_selected_prefix
            .as_ref()
            .filter(|relative| !relative.as_os_str().is_empty())
        {
            Some(relative) => isolated_root.join(relative),
            None => isolated_root.clone(),
        };
        let isolated_workspace = isolated_root.join(".substrate");
        fs::create_dir_all(&isolated_workspace).with_context(|| {
            format!(
                "failed to create isolated policy resolution root {}",
                isolated_workspace.display()
            )
        })?;
        fs::create_dir_all(&bound_cwd)
            .with_context(|| format!("failed to create bound cwd {}", bound_cwd.display()))?;
        fs::write(isolated_workspace.join("workspace.yaml"), "version: 1\n").with_context(
            || {
                format!(
                    "failed to create isolated workspace marker under {}",
                    isolated_root.display()
                )
            },
        )?;
        if let Some(bytes) = global_policy_bytes.as_ref() {
            fs::write(isolated_root.join("policy.yaml"), bytes).with_context(|| {
                format!(
                    "failed to materialize bound global policy under {}",
                    isolated_root.display()
                )
            })?;
        }
        if let Some(bytes) = workspace_policy_bytes.as_ref() {
            fs::write(isolated_workspace.join("policy.yaml"), bytes).with_context(|| {
                format!(
                    "failed to materialize bound workspace policy under {}",
                    isolated_root.display()
                )
            })?;
        }
        if let Some(relative) = cwd_relative_to_selected_prefix
            .as_ref()
            .filter(|relative| !relative.as_os_str().is_empty())
        {
            let mut source_dir = selected_host_prefix.clone();
            let mut dest_dir = isolated_root.clone();
            for component in relative.components() {
                source_dir = source_dir.join(component.as_os_str());
                dest_dir = dest_dir.join(component.as_os_str());
                fs::create_dir_all(&dest_dir).with_context(|| {
                    format!(
                        "failed to materialize bound policy directory {}",
                        dest_dir.display()
                    )
                })?;

                let source_workspace_dir = source_dir.join(".substrate");
                let source_workspace_marker = source_workspace_dir.join("workspace.yaml");
                let source_workspace_disabled = source_workspace_dir.join("workspace.disabled");
                if source_workspace_marker.is_file() && !source_workspace_disabled.exists() {
                    let dest_workspace_dir = dest_dir.join(".substrate");
                    fs::create_dir_all(&dest_workspace_dir).with_context(|| {
                        format!(
                            "failed to create bound workspace mirror {}",
                            dest_workspace_dir.display()
                        )
                    })?;
                    fs::write(
                        dest_workspace_dir.join("workspace.yaml"),
                        fs::read(&source_workspace_marker).with_context(|| {
                            format!("failed to read {}", source_workspace_marker.display())
                        })?,
                    )
                    .with_context(|| {
                        format!(
                            "failed to materialize workspace marker under {}",
                            dest_workspace_dir.display()
                        )
                    })?;

                    let source_workspace_policy = source_workspace_dir.join("policy.yaml");
                    if source_workspace_policy.is_file() {
                        fs::write(
                            dest_workspace_dir.join("policy.yaml"),
                            fs::read(&source_workspace_policy).with_context(|| {
                                format!("failed to read {}", source_workspace_policy.display())
                            })?,
                        )
                        .with_context(|| {
                            format!(
                                "failed to materialize workspace policy under {}",
                                dest_workspace_dir.display()
                            )
                        })?;
                    }
                }
            }
        }

        let resolve_result = resolve_effective_policy_with_explain_from_global_source(
            bound_cwd.as_path(),
            &isolated_root.join("policy.yaml"),
            global_policy_bytes.as_deref(),
            false,
        )
        .with_context(|| {
            format!(
                "failed to resolve bound effective policy from {}",
                selected_host_prefix.display()
            )
        });
        let policy = match resolve_result {
            Ok((policy, _)) => policy,
            Err(err) => {
                let _ = fs::remove_dir_all(&isolated_root);
                return Err(err);
            }
        };
        trace_context.set_policy_id(&policy.id);

        let previous_substrate_home = env::var_os("SUBSTRATE_HOME");
        env::set_var("SUBSTRATE_HOME", &isolated_root);
        let detect_result = detect_profile(bound_cwd.as_path());
        match previous_substrate_home {
            Some(value) => env::set_var("SUBSTRATE_HOME", value),
            None => env::remove_var("SUBSTRATE_HOME"),
        }
        let _ = fs::remove_dir_all(&isolated_root);
        detect_result.with_context(|| {
            format!(
                "failed to load bound effective policy from {}",
                selected_host_prefix.display()
            )
        })?;
    }
    let active_span = match evaluate_policy(&trace_context, &command_str, &cwd, &argv)? {
        PolicyResult::Proceed(context) => context.span,
        PolicyResult::Deny(exit_code) => return Ok(exit_code),
    };

    let start_time = Instant::now();
    let timestamp = SystemTime::now();

    let outcome = match execute_command(&real_binary, &args, &ctx.command_name, capture_stderr) {
        Ok(outcome) => outcome,
        Err(err) => {
            log_spawn_failure(&ctx, &real_binary, timestamp, &err);
            return Err(err);
        }
    };

    let mut manager_hint_payload = None;
    if let Some(engine) = hint_engine.as_mut() {
        if engine.is_active()
            && !outcome.status.success()
            && capture_stderr
            && outcome.captured_stderr.is_some()
        {
            if let Some(match_info) = engine.evaluate(outcome.captured_stderr.as_deref().unwrap()) {
                eprintln!(
                    "substrate: {} hint matched (pattern: {})\n{}",
                    match_info.manager_name,
                    match_info.pattern,
                    match_info.hint.trim_end()
                );
                manager_hint_payload = Some(hint_payload(&match_info));
            }
        }
    }

    let status = outcome.status;
    let duration = start_time.elapsed();

    if let Some(log_path) = &ctx.log_file {
        let metadata = ExecutionLogMetadata {
            duration,
            timestamp,
            resolved_path: &real_binary,
            manager_hint: manager_hint_payload.as_ref(),
        };
        if let Err(e) = log_execution(&trace_context, log_path, &ctx, &args, &status, &metadata) {
            eprintln!("Warning: Failed to log execution: {e}");
        }
    }

    if let Some(span) = active_span {
        let exit_code = status.code().unwrap_or(-1);
        let world_id = env::var(SUBSTRATE_WORLD_ID_VAR).ok();
        let project_path = env::var_os(SUBSTRATE_WORLD_PROJECT_DIR_VAR).map(PathBuf::from);
        let platform_bootstrap_mapping = if world_features_enabled() && world_id.is_some() {
            match default_platform_bootstrap_mapping_v1(&install_bootstrap_context) {
                Ok(mapping) => mapping,
                Err(err) => {
                    if cfg!(any(target_os = "macos", windows)) {
                        eprintln!(
                            "Warning: Failed to derive authenticated platform telemetry mapping: {err}"
                        );
                    }
                    None
                }
            }
        } else {
            None
        };
        let (scopes_used, fs_diff) = if world_features_enabled() {
            if let Some(world_id) = world_id.as_deref() {
                collect_world_telemetry(
                    span.get_span_id(),
                    world_id,
                    &install_bootstrap_context,
                    platform_bootstrap_mapping.as_ref(),
                    project_path.as_deref(),
                )
            } else {
                (vec![], None)
            }
        } else {
            (vec![], None)
        };

        let _ = span.finish(exit_code, scopes_used, fs_diff);
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return Ok(128 + signal);
        }
    }

    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::bootstrap::is_executable;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_executable_bit_check() {
        let temp = TempDir::new().unwrap();
        let non_executable = temp.path().join("not_exec");
        fs::write(&non_executable, "content").unwrap();

        assert!(!is_executable(&non_executable));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let executable = temp.path().join("exec");
            fs::write(&executable, "#!/bin/bash\necho test").unwrap();
            let mut perms = fs::metadata(&executable).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&executable, perms).unwrap();

            assert!(is_executable(&executable));
        }
    }
}
