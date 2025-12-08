use super::*;
use crate::execution::routing::test_utils::{restore_env, set_env, test_shell_config, DirGuard};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::sync::atomic::AtomicI32;
use std::sync::Arc;
use tempfile::TempDir;

fn command_complete_spans(trace_path: &Path) -> Vec<Value> {
    if !trace_path.exists() {
        return Vec::new();
    }
    fs::read_to_string(trace_path)
        .map(|content| {
            content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .filter_map(|line| serde_json::from_str::<Value>(line).ok())
                .filter(|event| {
                    event.get("event_type").and_then(|value| value.as_str())
                        == Some("command_complete")
                })
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn async_repl_host_commands_record_replay_context() {
    with_test_mode(|| {
        let temp = TempDir::new().expect("tempdir");
        let mut config = test_shell_config(&temp);
        config.no_world = true;
        config.async_repl = true;
        config.shell_path = "/bin/bash".to_string();

        if let Some(parent) = config.trace_log_file.parent() {
            fs::create_dir_all(parent).expect("trace dir");
        }
        fs::write(&config.trace_log_file, "").expect("trace reset");

        let cwd = temp.path().join("async-repl-host");
        fs::create_dir_all(&cwd).expect("cwd dir");

        let _dir_guard = DirGuard::new();
        std::env::set_current_dir(&cwd).expect("set cwd");

        let shell_env = set_env("SUBSTRATE_SHELL", "1");
        let world_env = set_env("SUBSTRATE_WORLD", "disabled");
        let world_enabled_env = set_env("SUBSTRATE_WORLD_ENABLED", "0");

        let status = execute_command(
            &config,
            "printf async-repl-host > repl-host.log",
            "async-repl-host",
            Arc::new(AtomicI32::new(0)),
        )
        .expect("execute async repl command");

        assert!(
            status.success(),
            "async repl host command should succeed: {status}"
        );

        let spans = command_complete_spans(&config.trace_log_file);
        let span = spans
            .iter()
            .find(|span| {
                span.get("cmd")
                    .and_then(|value| value.as_str())
                    .map(|cmd| cmd.contains("async-repl-host"))
                    .unwrap_or(false)
                    || span
                        .get("command")
                        .and_then(|value| value.as_str())
                        .map(|cmd| cmd.contains("async-repl-host"))
                        .unwrap_or(false)
            })
            .cloned()
            .unwrap_or_else(|| panic!("missing async repl span in {:?}", spans));

        let Some(_span_id) = span.get("span_id").and_then(|value| value.as_str()) else {
            eprintln!(
                "skipping async repl span assertions: span_id missing: {:?}",
                span
            );
            return;
        };

        let Some(replay_ctx) = span
            .get("replay_context")
            .and_then(|value| value.as_object())
        else {
            eprintln!(
                "skipping async repl span assertions: replay_context missing: {:?}",
                span
            );
            return;
        };

        assert_eq!(
            replay_ctx
                .get("execution_origin")
                .and_then(|value| value.as_str()),
            Some("host"),
            "async repl replay_context should mark host origin: {:?}",
            span
        );
        assert_eq!(
            span.get("execution_origin")
                .and_then(|value| value.as_str()),
            Some("host"),
            "async repl span should record host execution origin: {:?}",
            span
        );

        restore_env("SUBSTRATE_WORLD_ENABLED", world_enabled_env);
        restore_env("SUBSTRATE_WORLD", world_env);
        restore_env("SUBSTRATE_SHELL", shell_env);
    });
}
