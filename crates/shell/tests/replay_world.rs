#![cfg(all(unix, target_os = "linux"))]

#[path = "support/mod.rs"]
mod support;

use serde_json::{json, Value};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use support::{substrate_command_for_home, AgentSocket, ShellEnvFixture, SocketResponse};
use tempfile::{Builder, TempDir};

fn trace_path(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join("trace.jsonl")
}

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

fn replay_strategy_entries(trace_path: &Path) -> Vec<Value> {
    let file = fs::File::open(trace_path).expect("missing trace log");
    let reader = BufReader::new(file);
    reader
        .lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| serde_json::from_str::<Value>(&line).ok())
        .filter(|value| {
            value.get("event_type").and_then(|event| event.as_str()) == Some("replay_strategy")
        })
        .collect()
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

const ENOSPC_SHIM_SOURCE: &str = r#"
#define _GNU_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

typedef int (*mkdir_fn)(const char *, mode_t);
typedef int (*mkdirat_fn)(int, const char *, mode_t);

static const char *prefix(void) {
    return getenv("SUBSTRATE_ENOSPC_PREFIX");
}

int mkdir(const char *path, mode_t mode) {
    static mkdir_fn real_mkdir = NULL;
    if (!real_mkdir) {
        real_mkdir = (mkdir_fn)dlsym(RTLD_NEXT, "mkdir");
    }
    const char *p = prefix();
    if (p && strncmp(path, p, strlen(p)) == 0) {
        errno = ENOSPC;
        return -1;
    }
    return real_mkdir(path, mode);
}

int mkdirat(int dirfd, const char *path, mode_t mode) {
    static mkdirat_fn real_mkdirat = NULL;
    if (!real_mkdirat) {
        real_mkdirat = (mkdirat_fn)dlsym(RTLD_NEXT, "mkdirat");
    }
    const char *p = prefix();
    if (p && strncmp(path, p, strlen(p)) == 0) {
        errno = ENOSPC;
        return -1;
    }
    return real_mkdirat(dirfd, path, mode);
}
"#;

struct EnospcShim {
    _temp: TempDir,
    path: PathBuf,
}

impl EnospcShim {
    fn build() -> Self {
        let temp = tempfile::Builder::new()
            .prefix("substrate-enospc-shim-")
            .tempdir()
            .expect("failed to create shim tempdir");
        let c_path = temp.path().join("shim.c");
        fs::write(&c_path, ENOSPC_SHIM_SOURCE).expect("failed to write shim source");
        let so_path = temp.path().join("libenospc_shim.so");
        let status = Command::new("cc")
            .arg("-shared")
            .arg("-fPIC")
            .arg(&c_path)
            .arg("-o")
            .arg(&so_path)
            .status()
            .expect("failed to invoke cc for ENOSPC shim");
        assert!(
            status.success(),
            "failed to compile ENOSPC shim (status: {status})"
        );
        Self {
            _temp: temp,
            path: so_path,
        }
    }
}

fn replay_command(
    fixture: &ShellEnvFixture,
    span_id: &str,
    command: &str,
    cwd: &Path,
    path_override: &str,
) -> assert_cmd::Command {
    let trace = write_trace(fixture, span_id, command, cwd);
    let mut cmd = substrate_command_for_home(fixture);
    let xdg_runtime = fixture.home().join("xdg-runtime");
    fs::create_dir_all(&xdg_runtime).expect("failed to create xdg runtime dir");
    cmd.arg("--replay")
        .arg(span_id)
        .arg("--replay-verbose")
        .env("SHIM_TRACE_LOG", &trace)
        .env("SUBSTRATE_REPLAY_VERBOSE", "1")
        .env("SUBSTRATE_REPLAY_USE_WORLD", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("XDG_RUNTIME_DIR", &xdg_runtime)
        .env("PATH", path_override);
    cmd
}

fn copydiff_unavailable(stderr: &str) -> bool {
    stderr.contains("copy-diff failed in")
        && stderr.contains("failed spawning command under copydiff work dir")
}

#[test]
fn replay_warns_when_nft_unavailable() {
    let fixture = ShellEnvFixture::new();
    let cwd = fixture.home().join("workspace");
    fs::create_dir_all(&cwd).expect("failed to create replay cwd");
    let span_id = "span-nft-missing";
    let path_override = configure_nft_stub(&fixture, "#!/bin/sh\nexit 1\n");

    let mut cmd = replay_command(
        &fixture,
        span_id,
        "printf fallback-nft > replay.log",
        &cwd,
        &path_override,
    );

    let output = cmd.output().expect("failed to run replay command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        if copydiff_unavailable(&stderr) {
            eprintln!("skipping nft unavailable replay test: copy-diff unavailable\n{stderr}");
            return;
        }
        panic!("replay command failed unexpectedly: {stderr}");
    }
    assert!(
        stderr.contains("[replay] warn: nft not available; netfilter scoping/logging disabled"),
        "expected nft fallback warning in stderr, got:\n{}",
        stderr
    );
    assert!(
        stderr.contains("[replay] scopes: []"),
        "scopes line missing when nft unavailable, stderr:\n{}",
        stderr
    );
}

#[test]
fn replay_keeps_standard_path_when_nft_succeeds() {
    let fixture = ShellEnvFixture::new();
    let cwd = fixture.home().join("workspace-ok");
    fs::create_dir_all(&cwd).expect("failed to create replay cwd");
    let span_id = "span-nft-ok";
    let path_override = configure_nft_stub(&fixture, "#!/bin/sh\necho \"nft (test) v1\"\nexit 0\n");

    let mut cmd = replay_command(
        &fixture,
        span_id,
        "printf nft-ok > replay.log",
        &cwd,
        &path_override,
    );

    let output = cmd.output().expect("failed to run replay command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        if copydiff_unavailable(&stderr) {
            eprintln!("skipping nft available replay test: copy-diff unavailable\n{stderr}");
            return;
        }
        panic!("replay command failed unexpectedly: {stderr}");
    }
    assert!(
        !stderr.contains("[replay] warn: nft not available; netfilter scoping/logging disabled"),
        "nft available case should not warn, stderr:\n{}",
        stderr
    );
    assert!(
        stderr.contains("[replay] scopes: []"),
        "scopes line missing when nft succeeds, stderr:\n{}",
        stderr
    );
}

#[test]
fn replay_no_world_flag_reports_world_toggle() {
    let fixture = ShellEnvFixture::new();
    let cwd = fixture.home().join("workspace-no-world-flag");
    fs::create_dir_all(&cwd).expect("failed to create replay cwd");
    let span_id = "span-no-world-flag";
    let path_override = configure_nft_stub(&fixture, "#!/bin/sh\nexit 1\n");

    let mut cmd = replay_command(
        &fixture,
        span_id,
        "printf flag-mode > replay-no-world.log",
        &cwd,
        &path_override,
    );
    cmd.arg("--no-world");

    let assert = cmd.assert().success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("[replay] world toggle: disabled (--no-world flag)"),
        "no-world flag should emit a world toggle summary, stderr:\n{}",
        stderr
    );
    assert!(
        stderr.contains("[replay] warn: running without world isolation (--no-world flag)"),
        "no-world flag should document the opt-out reason, stderr:\n{}",
        stderr
    );
    assert!(
        !stderr.contains("[replay] world strategy:"),
        "no-world flag should skip world strategy line, stderr:\n{}",
        stderr
    );
    assert!(
        stderr.contains("[replay] scopes: []"),
        "scopes line missing for no-world flag run, stderr:\n{}",
        stderr
    );
}

#[test]
fn replay_env_override_reports_world_toggle() {
    let fixture = ShellEnvFixture::new();
    let cwd = fixture.home().join("workspace-no-world-env");
    fs::create_dir_all(&cwd).expect("failed to create replay cwd");
    let span_id = "span-no-world-env";
    let path_override = configure_nft_stub(&fixture, "#!/bin/sh\nexit 1\n");

    let mut cmd = replay_command(
        &fixture,
        span_id,
        "printf env-mode > replay-env.log",
        &cwd,
        &path_override,
    );
    cmd.env("SUBSTRATE_REPLAY_USE_WORLD", "disabled");

    let assert = cmd.assert().success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("[replay] world toggle: disabled (SUBSTRATE_REPLAY_USE_WORLD override)"),
        "env disable should emit the world toggle summary, stderr:\n{}",
        stderr
    );
    assert!(
        stderr.contains(
            "[replay] warn: running without world isolation (SUBSTRATE_REPLAY_USE_WORLD=disabled)"
        ),
        "env disable should document the opt-out reason, stderr:\n{}",
        stderr
    );
    assert!(
        !stderr.contains("[replay] world strategy:"),
        "env disable should skip world strategy line, stderr:\n{}",
        stderr
    );
    assert!(
        stderr.contains("[replay] scopes: []"),
        "scopes line missing when env disables world, stderr:\n{}",
        stderr
    );
}

#[test]
fn replay_prefers_agent_when_socket_healthy() {
    let fixture = ShellEnvFixture::new();
    let socket_dir = Builder::new()
        .prefix("substrate-agent-")
        .tempdir_in("/tmp")
        .expect("failed to create socket tempdir");
    let socket_path = socket_dir.path().join("substrate.sock");
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndExecute {
            stdout: "agent-ok\n".to_string(),
            stderr: String::new(),
            exit: 0,
            scopes: vec!["agent:uds:test".to_string()],
        },
    );

    let cwd = fixture.home().join("workspace-agent");
    fs::create_dir_all(&cwd).expect("failed to create replay cwd");
    let span_id = "span-agent-healthy";
    let path_override = configure_nft_stub(&fixture, "#!/bin/sh\necho \"nft (test) v1\"\nexit 0\n");

    let mut cmd = replay_command(
        &fixture,
        span_id,
        "printf agent-ok > replay-agent.log",
        &cwd,
        &path_override,
    );
    cmd.env("SUBSTRATE_WORLD_SOCKET", &socket_path);

    let assert = cmd.assert().success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("[replay] world strategy: agent"),
        "expected agent strategy in stderr: {stderr}"
    );
    assert!(
        stderr.contains("agent:uds:test"),
        "scopes should reflect agent response: {stderr}"
    );
    assert!(
        !stderr.contains("agent replay unavailable"),
        "agent path should not emit fallback warning: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("agent-ok"),
        "stdout should include agent response payload: {stdout}"
    );
}

#[test]
fn replay_emits_single_agent_warning_and_retries_copydiff_on_enospc() {
    let fixture = ShellEnvFixture::new();
    let trace = trace_path(&fixture);
    let cwd = fixture.home().join("workspace-agent-missing");
    fs::create_dir_all(&cwd).expect("failed to create replay cwd");
    let span_id = "span-agent-missing";
    let path_override = configure_nft_stub(&fixture, "#!/bin/sh\nexit 1\n");

    let shim = EnospcShim::build();
    let xdg_runtime = fixture.home().join("xdg-runtime");
    let enospc_prefix = xdg_runtime.join("substrate/copydiff");
    let missing_socket = fixture.home().join("missing-agent.sock");
    let missing_socket_str = missing_socket.display().to_string();
    let enospc_prefix_str = enospc_prefix.display().to_string();

    let mut cmd = replay_command(
        &fixture,
        span_id,
        "printf fallback > replay-fallback.log",
        &cwd,
        &path_override,
    );
    cmd.env("SUBSTRATE_WORLD_SOCKET", &missing_socket);
    cmd.env("XDG_RUNTIME_DIR", &xdg_runtime);
    cmd.env("LD_PRELOAD", &shim.path);
    cmd.env("SUBSTRATE_ENOSPC_PREFIX", &enospc_prefix);

    let output = cmd.output().expect("failed to run replay command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        if copydiff_unavailable(&stderr) {
            eprintln!("skipping agent fallback replay test: copy-diff unavailable\n{stderr}");
            return;
        }
        panic!("replay command failed unexpectedly: {stderr}");
    }
    let agent_warns = stderr.matches("agent replay unavailable").count();
    assert_eq!(
        agent_warns, 1,
        "agent fallback warning should be emitted once: {stderr}"
    );
    assert!(
        stderr.contains("substrate world doctor --json"),
        "agent warning should include doctor guidance: {stderr}"
    );
    assert!(
        stderr.contains("SUBSTRATE_WORLD_SOCKET"),
        "agent warning should mention SUBSTRATE_WORLD_SOCKET override: {stderr}"
    );
    assert!(
        stderr.contains("copy-diff storage"),
        "copy-diff ENOSPC warning missing: {stderr}"
    );
    assert!(
        stderr.contains("ran out of space; retrying fallback location"),
        "copy-diff ENOSPC retry message missing: {stderr}"
    );
    assert!(
        stderr.contains("[replay] copy-diff root:"),
        "copy-diff root should be printed in verbose output: {stderr}"
    );

    let strategies = replay_strategy_entries(&trace);
    assert!(
        !strategies.is_empty(),
        "expected replay_strategy entry in trace log"
    );
    let strategy = strategies.last().unwrap();
    assert_eq!(
        strategy.get("strategy").and_then(|value| value.as_str()),
        Some("copy-diff"),
        "expected copy-diff strategy when agent fallback triggered: {strategy:?}"
    );
    let fallback_reason = strategy
        .get("fallback_reason")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    assert!(
        fallback_reason.contains("agent socket missing"),
        "fallback_reason should mention missing agent socket: {fallback_reason}"
    );
    assert!(
        fallback_reason.contains(&missing_socket_str),
        "fallback_reason should include missing socket path ({fallback_reason})"
    );
    assert_eq!(
        strategy
            .get("agent_socket")
            .and_then(|value| value.as_str()),
        Some(missing_socket_str.as_str()),
        "agent_socket should record missing socket path"
    );
    let copydiff_root = strategy
        .get("copydiff_root")
        .and_then(|value| value.as_str())
        .expect("copydiff_root missing from replay_strategy");
    let copydiff_root_source = strategy
        .get("copydiff_root_source")
        .and_then(|value| value.as_str())
        .expect("copydiff_root_source missing from replay_strategy");
    assert!(
        stderr.contains(&format!("[replay] copy-diff root: {}", copydiff_root)),
        "stderr should include the copy-diff root from telemetry ({copydiff_root}): {stderr}"
    );
    assert!(
        stderr.contains(copydiff_root_source),
        "stderr should include the copy-diff root source ({copydiff_root_source}): {stderr}"
    );
    assert!(
        !copydiff_root.starts_with(&enospc_prefix_str),
        "copy-diff should retry a different root after ENOSPC (got {copydiff_root})"
    );
}
#[test]
fn replay_logs_copydiff_override_root_and_telemetry() {
    let fixture = ShellEnvFixture::new();
    let trace = trace_path(&fixture);
    let cwd = fixture.home().join("workspace-copydiff-override-with-env");
    fs::create_dir_all(&cwd).expect("failed to create replay cwd");
    let span_id = "span-copydiff-override";
    let path_override = configure_nft_stub(&fixture, "#!/bin/sh\necho \"nft (test) v1\"\nexit 0\n");
    let override_root = fixture.home().join("custom copydiff root");
    let override_root_str = override_root.display().to_string();

    let mut cmd = replay_command(
        &fixture,
        span_id,
        "printf override-root > replay-copydiff-override.log",
        &cwd,
        &path_override,
    );
    cmd.env("SUBSTRATE_COPYDIFF_ROOT", &override_root);

    let output = cmd.output().expect("failed to run replay command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        if copydiff_unavailable(&stderr) {
            eprintln!("skipping copy-diff override replay test: copy-diff unavailable\n{stderr}");
            return;
        }
        panic!("replay command failed unexpectedly: {stderr}");
    }

    assert!(
        stderr.contains(&format!(
            "[replay] copy-diff root: {} (env:SUBSTRATE_COPYDIFF_ROOT)",
            override_root.display()
        )),
        "copy-diff root line should reflect override path and source: {stderr}"
    );
    assert!(
        !stderr.contains("copy-diff storage"),
        "override root should not trigger copy-diff storage retries: {stderr}"
    );

    let strategies = replay_strategy_entries(&trace);
    assert!(
        !strategies.is_empty(),
        "expected replay_strategy entry in trace log"
    );
    let strategy = strategies.last().unwrap();
    assert_eq!(
        strategy.get("strategy").and_then(|value| value.as_str()),
        Some("copy-diff"),
        "expected copy-diff strategy when override is provided: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("copydiff_root")
            .and_then(|value| value.as_str()),
        Some(override_root_str.as_str()),
        "trace should record the override copy-diff root"
    );
    assert_eq!(
        strategy
            .get("copydiff_root_source")
            .and_then(|value| value.as_str()),
        Some("env:SUBSTRATE_COPYDIFF_ROOT"),
        "trace should record the copy-diff root source"
    );
}
