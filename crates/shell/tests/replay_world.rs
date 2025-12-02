#![cfg(all(unix, target_os = "linux"))]

#[path = "support/mod.rs"]
mod support;

use serde_json::json;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use support::{substrate_command_for_home, ShellEnvFixture};

fn write_trace(fixture: &ShellEnvFixture, span_id: &str, cmd: &str, cwd: &Path) -> PathBuf {
    let trace = fixture.home().join("trace.jsonl");
    let entry = json!({
        "ts": "2025-01-01T00:00:00Z",
        "event_type": "command_complete",
        "span_id": span_id,
        "session_id": "session-r1a",
        "component": "shell",
        "cmd": cmd,
        "cwd": cwd.to_string_lossy(),
        "exit_code": 0
    });
    fs::create_dir_all(trace.parent().unwrap()).expect("failed to create trace dir");
    fs::write(&trace, format!("{}\n", entry)).expect("failed to write trace entry");
    trace
}

fn configure_nft_stub(fixture: &ShellEnvFixture, script: &str) -> String {
    let bin_dir = fixture.home().join("bin");
    fs::create_dir_all(&bin_dir).expect("failed to create stub bin dir");
    let nft_path = bin_dir.join("nft");
    fs::write(&nft_path, script).expect("failed to write nft stub");
    let mut perms = fs::metadata(&nft_path)
        .expect("failed to stat nft stub")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&nft_path, perms).expect("failed to chmod nft stub");
    let current = env::var("PATH").unwrap_or_default();
    if current.is_empty() {
        bin_dir.to_string_lossy().into_owned()
    } else {
        format!("{}:{}", bin_dir.display(), current)
    }
}

#[test]
fn replay_warns_when_nft_unavailable() {
    let fixture = ShellEnvFixture::new();
    let cwd = fixture.home().join("workspace");
    fs::create_dir_all(&cwd).expect("failed to create replay cwd");
    let span_id = "span-nft-missing";
    let trace = write_trace(&fixture, span_id, "printf fallback-nft > replay.log", &cwd);
    let path_override = configure_nft_stub(&fixture, "#!/bin/sh\nexit 1\n");

    let mut cmd = substrate_command_for_home(&fixture);
    cmd.arg("--replay")
        .arg(span_id)
        .arg("--replay-verbose")
        .env("SHIM_TRACE_LOG", &trace)
        .env("SUBSTRATE_REPLAY_VERBOSE", "1")
        .env("SUBSTRATE_REPLAY_USE_WORLD", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("PATH", path_override);

    let assert = cmd.assert().success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("[replay] warn: nft not available; netfilter scoping/logging disabled"),
        "expected nft fallback warning in stderr, got:\n{}",
        stderr
    );
}

#[test]
fn replay_keeps_standard_path_when_nft_succeeds() {
    let fixture = ShellEnvFixture::new();
    let cwd = fixture.home().join("workspace-ok");
    fs::create_dir_all(&cwd).expect("failed to create replay cwd");
    let span_id = "span-nft-ok";
    let trace = write_trace(&fixture, span_id, "printf nft-ok > replay.log", &cwd);
    let path_override = configure_nft_stub(&fixture, "#!/bin/sh\necho \"nft (test) v1\"\nexit 0\n");

    let mut cmd = substrate_command_for_home(&fixture);
    cmd.arg("--replay")
        .arg(span_id)
        .arg("--replay-verbose")
        .env("SHIM_TRACE_LOG", &trace)
        .env("SUBSTRATE_REPLAY_VERBOSE", "1")
        .env("SUBSTRATE_REPLAY_USE_WORLD", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("PATH", path_override);

    let assert = cmd.assert().success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        !stderr.contains("[replay] warn: nft not available; netfilter scoping/logging disabled"),
        "nft available case should not warn, stderr:\n{}",
        stderr
    );
}
