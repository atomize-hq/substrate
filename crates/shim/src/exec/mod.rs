mod bootstrap;
mod logging;
mod policy;

use self::bootstrap::{
    execute_command, execute_real_binary_bypass, handle_bypass_mode, persist_original_path,
    resolve_command_binary,
};
use self::logging::{collect_world_telemetry, hint_payload, log_spawn_failure, ManagerHintEngine};
use self::policy::{evaluate_policy, PolicyResult};
use crate::context::{world_features_enabled, ShimContext};
use crate::logger::{log_execution, ExecutionLogMetadata};
use anyhow::Result;
use std::env;
use std::path::PathBuf;
use std::time::{Instant, SystemTime};
use substrate_broker::{set_global_broker, BrokerHandle};
use substrate_trace::{create_span_builder, init_trace, set_global_trace_context, TraceContext};

/// Main shim execution function
pub fn run_shim() -> Result<i32> {
    if ShimContext::is_bypass_enabled() {
        return handle_bypass_mode();
    }

    let ctx = ShimContext::from_current_exe()?;

    let _ = set_global_broker(BrokerHandle::new());
    let _ = set_global_trace_context(TraceContext::default());

    persist_original_path(&ctx);

    if ctx.should_skip_shimming() {
        return execute_real_binary_bypass(&ctx);
    }

    ctx.setup_execution_env();

    let mut hint_engine = ManagerHintEngine::new();
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

    let mut active_span = None;

    if world_features_enabled() {
        match evaluate_policy(&command_str, &cwd, &argv)? {
            PolicyResult::Proceed(context) => {
                active_span = context.span;
            }
            PolicyResult::Deny(exit_code) => {
                return Ok(exit_code);
            }
        }
    } else {
        let _ = init_trace(None);
        if let Ok(mut builder) = create_span_builder() {
            builder = builder
                .with_command(&command_str)
                .with_cwd(cwd.to_str().unwrap_or("."));

            match builder.start() {
                Ok(span) => {
                    env::set_var("SHIM_PARENT_SPAN", span.get_span_id());
                    active_span = Some(span);
                }
                Err(e) => {
                    eprintln!("substrate: failed to create span: {}", e);
                }
            }
        } else {
            eprintln!("substrate: failed to create span builder");
        }
    }

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
        if let Err(e) = log_execution(log_path, &ctx, &args, &status, &metadata) {
            eprintln!("Warning: Failed to log execution: {e}");
        }
    }

    if let Some(span) = active_span {
        let exit_code = status.code().unwrap_or(-1);
        let (scopes_used, fs_diff) = if world_features_enabled() {
            collect_world_telemetry(span.get_span_id())
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
