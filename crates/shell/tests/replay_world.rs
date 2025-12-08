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
use substrate_trace::ExecutionOrigin;
use support::{substrate_command_for_home, AgentSocket, ShellEnvFixture, SocketResponse};
use tempfile::{Builder, TempDir};

fn trace_path(fixture: &ShellEnvFixture) -> PathBuf {
    fixture.home().join("trace.jsonl")
}

struct TraceOptions {
    execution_origin: ExecutionOrigin,
    transport_endpoint: Option<PathBuf>,
    transport_socket_activation: Option<bool>,
    replay_context: Option<Value>,
}

impl Default for TraceOptions {
    fn default() -> Self {
        Self {
            execution_origin: ExecutionOrigin::World,
            transport_endpoint: None,
            transport_socket_activation: None,
            replay_context: None,
        }
    }
}

fn write_trace_with_options(
    fixture: &ShellEnvFixture,
    span_id: &str,
    cmd: &str,
    cwd: &Path,
    options: TraceOptions,
) -> PathBuf {
    let trace = fixture.home().join("trace.jsonl");
    let mut entry = json!({
        "ts": "2025-01-01T00:00:00Z",
        "event_type": "command_complete",
        "span_id": span_id,
        "session_id": "session-r1a",
        "component": "shell",
        "cmd": cmd,
        "cwd": cwd.to_string_lossy(),
        "exit_code": 0,
        "execution_origin": options.execution_origin,
    });
    if let Some(context) = options.replay_context {
        entry["replay_context"] = context;
    }
    if let Some(endpoint) = options.transport_endpoint {
        let mut transport = json!({
            "mode": "unix",
            "endpoint": endpoint.to_string_lossy(),
        });
        if let Some(activated) = options.transport_socket_activation {
            transport["socket_activation"] = json!(activated);
        }
        entry["transport"] = transport;
    }
    fs::create_dir_all(trace.parent().unwrap()).expect("failed to create trace dir");
    fs::write(&trace, format!("{}\n", entry)).expect("failed to write trace entry");
    trace
}

fn replay_strategy_entries(trace_path: &Path) -> Vec<Value> {
    let file = fs::File::open(trace_path).expect("missing trace log");
    let reader = BufReader::new(file);
    reader
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| serde_json::from_str::<Value>(&line).ok())
        .filter(|value| {
            value.get("event_type").and_then(|event| event.as_str()) == Some("replay_strategy")
        })
        .collect()
}

fn latest_replay_strategy(trace_path: &Path) -> Option<Value> {
    replay_strategy_entries(trace_path).last().cloned()
}

fn command_complete_entries(trace_path: &Path) -> Vec<Value> {
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
                    event
                        .get("event_type")
                        .and_then(|value| value.as_str())
                        == Some("command_complete")
                })
                .collect()
        })
        .unwrap_or_default()
}

fn find_command_span(entries: &[Value], needle: &str) -> Option<Value> {
    entries
        .iter()
        .find(|event| {
            event
                .get("cmd")
                .and_then(|value| value.as_str())
                .map(|cmd| cmd.contains(needle))
                .unwrap_or(false)
                || event
                    .get("command")
                    .and_then(|value| value.as_str())
                    .map(|cmd| cmd.contains(needle))
                    .unwrap_or(false)
        })
        .cloned()
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

fn replay_command_with_options(
    fixture: &ShellEnvFixture,
    span_id: &str,
    command: &str,
    cwd: &Path,
    path_override: &str,
    options: TraceOptions,
) -> assert_cmd::Command {
    let trace = write_trace_with_options(fixture, span_id, command, cwd, options);
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

fn replay_command(
    fixture: &ShellEnvFixture,
    span_id: &str,
    command: &str,
    cwd: &Path,
    path_override: &str,
) -> assert_cmd::Command {
    replay_command_with_options(
        fixture,
        span_id,
        command,
        cwd,
        path_override,
        TraceOptions::default(),
    )
}

fn copydiff_unavailable(stderr: &str) -> bool {
    stderr.contains("copy-diff failed in")
        && stderr.contains("failed spawning command under copydiff work dir")
}

#[test]
fn host_commands_emit_replayable_spans() {
    let fixture = ShellEnvFixture::new();
    let trace = trace_path(&fixture);
    if let Some(parent) = trace.parent() {
        fs::create_dir_all(parent).expect("failed to create trace dir");
    }
    fs::write(&trace, "").expect("failed to reset trace file");

    let workspace = fixture.home().join("host-span-workspace");
    fs::create_dir_all(&workspace).expect("failed to prepare workspace");

    let mut no_world_cmd = substrate_command_for_home(&fixture);
    no_world_cmd
        .env("SHIM_TRACE_LOG", &trace)
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .arg("--no-world")
        .arg("-c")
        .arg("printf host-no-world > host-no-world.log")
        .current_dir(&workspace)
        .assert()
        .success();

    let mut env_disable_cmd = substrate_command_for_home(&fixture);
    env_disable_cmd
        .env("SHIM_TRACE_LOG", &trace)
        .env("SUBSTRATE_WORLD", "disabled")
        .env("SUBSTRATE_WORLD_ENABLED", "0")
        .env("SUBSTRATE_REPLAY_USE_WORLD", "disabled")
        .arg("-c")
        .arg("printf host-env-opt-out > host-env-opt-out.log")
        .current_dir(&workspace)
        .assert()
        .success();

    let entries = command_complete_entries(&trace);
    assert!(
        !entries.is_empty(),
        "expected command_complete spans in trace at {:?}",
        trace
    );

    for marker in ["host-no-world", "host-env-opt-out"] {
        let Some(span) = find_command_span(&entries, marker) else {
            eprintln!(
                "skipping host span assertions: missing span for {marker} in {:?}",
                entries
            );
            return;
        };
        let Some(_span_id) = span.get("span_id").and_then(|value| value.as_str()) else {
            eprintln!(
                "skipping host span assertions: span_id missing for {marker}: {:?}",
                span
            );
            return;
        };

        let Some(replay_ctx) = span
            .get("replay_context")
            .and_then(|value| value.as_object()) else {
                eprintln!(
                    "skipping host span assertions: replay_context missing for {marker}: {:?}",
                    span
                );
                return;
            };

        assert_eq!(
            replay_ctx
                .get("execution_origin")
                .and_then(|value| value.as_str()),
            Some("host"),
            "replay_context should mark host origin for {marker}: {:?}",
            span
        );
        assert_eq!(
            span.get("execution_origin")
                .and_then(|value| value.as_str()),
            Some("host"),
            "span should record host execution origin for {marker}: {:?}",
            span
        );
    }
}

#[test]
fn replay_host_span_respects_env_opt_out_without_agent_probe() {
    let fixture = ShellEnvFixture::new();
    let trace = trace_path(&fixture);
    if let Some(parent) = trace.parent() {
        fs::create_dir_all(parent).expect("failed to create trace dir");
    }
    fs::write(&trace, "").expect("failed to reset trace file");

    let workspace = fixture.home().join("workspace-replay-host");
    fs::create_dir_all(&workspace).expect("failed to create replay workspace");

    let mut record_cmd = substrate_command_for_home(&fixture);
    record_cmd
        .env("SHIM_TRACE_LOG", &trace)
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .arg("--no-world")
        .arg("-c")
        .arg("printf host-replay > host-replay.log")
        .current_dir(&workspace)
        .assert()
        .success();

    let entries = command_complete_entries(&trace);
    let Some(span) = find_command_span(&entries, "host-replay") else {
        eprintln!(
            "skipping host replay env opt-out assertions: missing recorded span in {:?}",
            entries
        );
        return;
    };
    let Some(span_id) = span.get("span_id").and_then(|value| value.as_str()) else {
        eprintln!(
            "skipping host replay env opt-out assertions: span_id missing: {:?}",
            span
        );
        return;
    };
    let Some(replay_ctx) = span
        .get("replay_context")
        .and_then(|value| value.as_object()) else {
            eprintln!(
                "skipping host replay env opt-out assertions: replay_context missing: {:?}",
                span
            );
            return;
        };
    assert_eq!(
        replay_ctx
            .get("execution_origin")
            .and_then(|value| value.as_str()),
        Some("host"),
        "host span replay_context should mark host execution"
    );
    assert_eq!(
        span.get("execution_origin")
            .and_then(|value| value.as_str()),
        Some("host"),
        "host span should record host execution origin"
    );
    let span_id = span_id.to_string();

    let replay_log = workspace.join("host-replay.log");
    let _ = fs::remove_file(&replay_log);

    let path_override =
        configure_nft_stub(&fixture, "#!/bin/sh\necho \"nft (test) v1\"\nexit 0\n");
    let xdg_runtime = fixture.home().join("xdg-runtime-host-replay");
    fs::create_dir_all(&xdg_runtime).expect("failed to create xdg runtime dir");
    let missing_socket = fixture.home().join("missing-agent-host-replay.sock");

    let assert = substrate_command_for_home(&fixture)
        .env("SHIM_TRACE_LOG", &trace)
        .env("SUBSTRATE_REPLAY_VERBOSE", "1")
        .env("SUBSTRATE_REPLAY_USE_WORLD", "disabled")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
        .env("XDG_RUNTIME_DIR", &xdg_runtime)
        .env("PATH", &path_override)
        .current_dir(&workspace)
        .arg("--replay")
        .arg(&span_id)
        .arg("--replay-verbose")
        .assert()
        .success();

    let output = assert.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("SUBSTRATE_REPLAY_USE_WORLD=disabled"),
        "env opt-out reason should be reported: {stderr}"
    );
    assert!(
        stderr.contains("[replay] origin: host"),
        "host origin summary should be present: {stderr}"
    );
    let host_warns = stderr.matches("warn: running on host").count();
    assert_eq!(
        host_warns, 1,
        "host opt-out warning should be deduped: {stderr}"
    );
    assert!(
        !stderr.contains("agent replay unavailable"),
        "host replay should skip agent probes even when socket env is set: {stderr}"
    );
    assert!(
        !stderr.contains("[replay] world strategy:"),
        "host replay should not log world strategies: {stderr}"
    );

    let content = fs::read_to_string(&replay_log)
        .expect("replayed host command should write output in workspace");
    assert_eq!(
        content, "host-replay",
        "host replay should run command in workspace: {}",
        replay_log.display()
    );

    if let Some(strategy) = latest_replay_strategy(&trace) {
        assert_eq!(
            strategy.get("strategy").and_then(|value| value.as_str()),
            Some("host"),
            "replay_strategy should record host execution: {strategy:?}"
        );
        assert_eq!(
            strategy
                .get("recorded_origin")
                .and_then(|value| value.as_str()),
            Some("host"),
            "replay_strategy recorded_origin should be host: {strategy:?}"
        );
        assert_eq!(
            strategy
                .get("target_origin")
                .and_then(|value| value.as_str()),
            Some("host"),
            "replay_strategy target_origin should stay host: {strategy:?}"
        );
        assert_eq!(
            strategy
                .get("origin_reason_code")
                .and_then(|value| value.as_str()),
            Some("env_disabled"),
            "replay_strategy should note env opt-out reason: {strategy:?}"
        );
    } else {
        eprintln!("skipping host replay strategy assertions: no replay_strategy entries written");
    }
}

#[test]
fn replay_host_span_warns_once_when_forced_to_world() {
    let fixture = ShellEnvFixture::new();
    let trace = trace_path(&fixture);
    if let Some(parent) = trace.parent() {
        fs::create_dir_all(parent).expect("failed to create trace dir");
    }
    fs::write(&trace, "").expect("failed to reset trace file");

    let workspace = fixture.home().join("workspace-force-world");
    fs::create_dir_all(&workspace).expect("failed to create replay workspace");

    let mut record_cmd = substrate_command_for_home(&fixture);
    record_cmd
        .env("SHIM_TRACE_LOG", &trace)
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .arg("--no-world")
        .arg("-c")
        .arg("printf host-force-world > host-force-world.log")
        .current_dir(&workspace)
        .assert()
        .success();

    let entries = command_complete_entries(&trace);
    let Some(span) = find_command_span(&entries, "host-force-world") else {
        eprintln!(
            "skipping forced-world host replay assertions: missing recorded span in {:?}",
            entries
        );
        return;
    };
    let Some(span_id) = span.get("span_id").and_then(|value| value.as_str()) else {
        eprintln!(
            "skipping forced-world host replay assertions: span_id missing: {:?}",
            span
        );
        return;
    };
    let Some(replay_ctx) = span
        .get("replay_context")
        .and_then(|value| value.as_object()) else {
            eprintln!(
                "skipping forced-world host replay assertions: replay_context missing: {:?}",
                span
            );
            return;
        };
    assert_eq!(
        replay_ctx
            .get("execution_origin")
            .and_then(|value| value.as_str()),
        Some("host"),
        "host span replay_context should mark host execution"
    );
    assert_eq!(
        span.get("execution_origin")
            .and_then(|value| value.as_str()),
        Some("host"),
        "host span should record host execution origin"
    );
    let span_id = span_id.to_string();

    let path_override =
        configure_nft_stub(&fixture, "#!/bin/sh\necho \"nft (test) v1\"\nexit 0\n");
    let xdg_runtime = fixture.home().join("xdg-runtime-force-world");
    fs::create_dir_all(&xdg_runtime).expect("failed to create xdg runtime dir");

    let socket_dir = Builder::new()
        .prefix("substrate-agent-force-world-")
        .tempdir_in("/tmp")
        .expect("failed to create socket dir");
    let socket_path = socket_dir.path().join("substrate.sock");
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndExecute {
            stdout: "agent-host-forced\n".to_string(),
            stderr: String::new(),
            exit: 0,
            scopes: vec!["agent:uds:host-force".to_string()],
        },
    );

    let assert = substrate_command_for_home(&fixture)
        .env("SHIM_TRACE_LOG", &trace)
        .env("SUBSTRATE_REPLAY_VERBOSE", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("XDG_RUNTIME_DIR", &xdg_runtime)
        .env("PATH", &path_override)
        .current_dir(&workspace)
        .arg("--replay")
        .arg(&span_id)
        .arg("--replay-verbose")
        .arg("--world")
        .assert()
        .success();

    let output = assert.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[replay] origin: host -> world (--world flag)"),
        "world override should surface in origin summary: {stderr}"
    );
    let world_warns = stderr.matches("warn: running on world").count();
    assert_eq!(
        world_warns, 1,
        "world override should emit a single warning: {stderr}"
    );
    assert!(
        stderr.contains(&format!(
            "[replay] world strategy: agent (socket={}, project_dir={})",
            socket_path.display(),
            workspace.display()
        )),
        "world strategy should route through stub agent: {stderr}"
    );
    assert!(
        stderr.contains("[replay] scopes: [agent:uds:host-force]"),
        "scopes should reflect stub agent execution: {stderr}"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("agent-host-forced"),
        "forced world replay should return agent stdout payload: {stdout}"
    );

    if let Some(strategy) = latest_replay_strategy(&trace) {
        assert_eq!(
            strategy.get("strategy").and_then(|value| value.as_str()),
            Some("agent"),
            "replay_strategy should record agent execution: {strategy:?}"
        );
        assert_eq!(
            strategy
                .get("recorded_origin")
                .and_then(|value| value.as_str()),
            Some("host"),
            "replay_strategy recorded_origin should stay host: {strategy:?}"
        );
        assert_eq!(
            strategy
                .get("target_origin")
                .and_then(|value| value.as_str()),
            Some("world"),
            "replay_strategy target_origin should be world after override: {strategy:?}"
        );
        assert_eq!(
            strategy
                .get("origin_reason_code")
                .and_then(|value| value.as_str()),
            Some("flag_world"),
            "replay_strategy should capture override reason code: {strategy:?}"
        );
    } else {
        eprintln!("skipping forced-world strategy assertions: no replay_strategy entries written");
    }
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
    let missing_socket = fixture.home().join("missing-agent-no-world.sock");

    let mut cmd = replay_command(
        &fixture,
        span_id,
        "printf flag-mode > replay-no-world.log",
        &cwd,
        &path_override,
    );
    cmd.arg("--no-world");
    cmd.env("SUBSTRATE_WORLD_SOCKET", &missing_socket);

    let assert = cmd.assert().success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("[replay] origin: world -> host (--no-world flag)"),
        "no-world flag should emit an origin summary, stderr:\n{}",
        stderr
    );
    assert!(
        stderr.contains("[replay] warn: running on host (--no-world flag)"),
        "no-world flag should document the opt-out reason, stderr:\n{}",
        stderr
    );
    assert!(
        !stderr.contains("[replay] world strategy:"),
        "no-world flag should skip world strategy line, stderr:\n{}",
        stderr
    );
    assert!(
        !stderr.contains("agent replay unavailable"),
        "no-world flag should skip agent socket warnings even when a socket path is set: {stderr}"
    );
    assert!(
        !stderr.contains("copy-diff root"),
        "no-world flag should skip copy-diff fallback lines: {stderr}"
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
    let missing_socket = fixture.home().join("missing-agent-no-world-env.sock");

    let mut cmd = replay_command(
        &fixture,
        span_id,
        "printf env-mode > replay-env.log",
        &cwd,
        &path_override,
    );
    cmd.env("SUBSTRATE_REPLAY_USE_WORLD", "disabled");
    cmd.env("SUBSTRATE_WORLD_SOCKET", &missing_socket);

    let assert = cmd.assert().success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("[replay] origin: world -> host (SUBSTRATE_REPLAY_USE_WORLD=disabled)"),
        "env disable should emit the origin summary, stderr:\n{}",
        stderr
    );
    assert!(
        stderr.contains("[replay] warn: running on host (SUBSTRATE_REPLAY_USE_WORLD=disabled)"),
        "env disable should document the opt-out reason, stderr:\n{}",
        stderr
    );
    assert!(
        !stderr.contains("[replay] world strategy:"),
        "env disable should skip world strategy line, stderr:\n{}",
        stderr
    );
    assert!(
        !stderr.contains("agent replay unavailable"),
        "env disable should skip agent socket warnings even when a socket path is set: {stderr}"
    );
    assert!(
        !stderr.contains("copy-diff root"),
        "env disable should skip copy-diff fallback lines: {stderr}"
    );
    assert!(
        stderr.contains("[replay] scopes: []"),
        "scopes line missing when env disables world, stderr:\n{}",
        stderr
    );
}

#[test]
fn replay_defaults_to_recorded_host_origin() {
    let fixture = ShellEnvFixture::new();
    let cwd = fixture.home().join("workspace-recorded-host");
    fs::create_dir_all(&cwd).expect("failed to create replay cwd");
    let span_id = "span-recorded-host-default";
    let path_override = configure_nft_stub(&fixture, "#!/bin/sh\necho \"nft (test) v1\"\nexit 0\n");

    let mut cmd = replay_command_with_options(
        &fixture,
        span_id,
        "printf recorded-host > recorded-host.log",
        &cwd,
        &path_override,
        TraceOptions {
            execution_origin: ExecutionOrigin::Host,
            ..TraceOptions::default()
        },
    );

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[replay] origin: host (recorded)"),
        "recorded host origin should be reported when no flip/override is set: {stderr}"
    );
    assert!(
        !stderr.contains("[replay] world strategy:"),
        "host replay should skip world strategy output: {stderr}"
    );
    assert!(
        !stderr.contains("warn: running on host"),
        "recorded host origin should not emit a host warning: {stderr}"
    );
    let host_log = cwd.join("recorded-host.log");
    let content = fs::read_to_string(&host_log).expect("host replay should write output in cwd");
    assert!(
        content.contains("recorded-host"),
        "expected host replay to write to {:?}",
        host_log
    );

    let Some(strategy) = latest_replay_strategy(&trace_path(&fixture)) else {
        eprintln!("skipping recorded-host strategy assertions: no replay_strategy entries written");
        return;
    };
    assert_eq!(
        strategy.get("strategy").and_then(|value| value.as_str()),
        Some("host"),
        "strategy should record host execution: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("recorded_origin")
            .and_then(|value| value.as_str()),
        Some("host"),
        "recorded_origin should reflect host span metadata: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("target_origin")
            .and_then(|value| value.as_str()),
        Some("host"),
        "target_origin should remain host without flips/overrides: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("recorded_origin_source")
            .and_then(|value| value.as_str()),
        Some("span"),
        "recorded_origin_source should track span metadata: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("origin_reason_code")
            .and_then(|value| value.as_str()),
        Some("recorded_origin"),
        "origin_reason_code should note the recorded origin path: {strategy:?}"
    );
}

#[test]
fn replay_flip_world_to_host_reports_reason() {
    let fixture = ShellEnvFixture::new();
    let cwd = fixture.home().join("workspace-flip-world");
    fs::create_dir_all(&cwd).expect("failed to create replay cwd");
    let span_id = "span-flip-world-to-host";
    let path_override = configure_nft_stub(&fixture, "#!/bin/sh\necho \"nft (test) v1\"\nexit 0\n");

    let mut cmd = replay_command_with_options(
        &fixture,
        span_id,
        "printf flipped-world > flipped-world.log",
        &cwd,
        &path_override,
        TraceOptions {
            execution_origin: ExecutionOrigin::World,
            ..TraceOptions::default()
        },
    );
    cmd.arg("--flip-world");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[replay] origin: world -> host (--flip-world)"),
        "flip flag should invert recorded world origin: {stderr}"
    );
    assert!(
        stderr.contains("[replay] warn: running on host (--flip-world)"),
        "flip from world to host should emit a host warning: {stderr}"
    );
    assert!(
        !stderr.contains("[replay] world strategy:"),
        "flipped host run should skip world strategy output: {stderr}"
    );
    let flipped_log = cwd.join("flipped-world.log");
    let content =
        fs::read_to_string(&flipped_log).expect("flipped host replay should write output in cwd");
    assert!(
        content.contains("flipped-world"),
        "expected flipped host replay to write to {:?}",
        flipped_log
    );

    let Some(strategy) = latest_replay_strategy(&trace_path(&fixture)) else {
        eprintln!("skipping flip-world strategy assertions: no replay_strategy entries written");
        return;
    };
    assert_eq!(
        strategy.get("strategy").and_then(|value| value.as_str()),
        Some("host"),
        "strategy should record host execution after flip: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("recorded_origin")
            .and_then(|value| value.as_str()),
        Some("world"),
        "recorded_origin should stay world even when flipped: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("target_origin")
            .and_then(|value| value.as_str()),
        Some("host"),
        "target_origin should reflect flip to host: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("origin_reason_code")
            .and_then(|value| value.as_str()),
        Some("flip_world"),
        "origin_reason_code should capture flip flag: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("fallback_reason")
            .and_then(|value| value.as_str()),
        Some("--flip-world"),
        "fallback_reason should explain the host selection when flipped: {strategy:?}"
    );
}

#[test]
fn replay_flip_host_to_world_prefers_agent_and_reports_origin() {
    let fixture = ShellEnvFixture::new();
    let socket_dir = tempfile::Builder::new()
        .prefix("substrate-agent-flip-")
        .tempdir_in("/tmp")
        .expect("failed to create socket tempdir");
    let socket_path = socket_dir.path().join("substrate.sock");
    let socket_str = socket_path.display().to_string();
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndExecute {
            stdout: "agent-flip\n".to_string(),
            stderr: String::new(),
            exit: 0,
            scopes: vec!["agent:uds:flip".to_string()],
        },
    );

    let cwd = fixture.home().join("workspace-flip-to-world");
    fs::create_dir_all(&cwd).expect("failed to create replay cwd");
    let span_id = "span-flip-host-to-world";
    let path_override = configure_nft_stub(&fixture, "#!/bin/sh\necho \"nft (test) v1\"\nexit 0\n");

    let mut cmd = replay_command_with_options(
        &fixture,
        span_id,
        "printf flipped-agent > flipped-agent.log",
        &cwd,
        &path_override,
        TraceOptions {
            execution_origin: ExecutionOrigin::Host,
            transport_endpoint: Some(socket_path.clone()),
            transport_socket_activation: Some(true),
            ..TraceOptions::default()
        },
    );
    cmd.env("SUBSTRATE_ANCHOR_MODE", "custom");
    cmd.env("SUBSTRATE_WORLD_ROOT_MODE", "custom");
    cmd.env("SUBSTRATE_ANCHOR_PATH", &cwd);
    cmd.env("SUBSTRATE_WORLD_ROOT_PATH", &cwd);
    cmd.env("SUBSTRATE_CAGED", "0");
    cmd.arg("--flip");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[replay] origin: host -> world (--flip-world)"),
        "flip alias should invert recorded host origin: {stderr}"
    );
    assert!(
        stderr.contains("[replay] world strategy: agent"),
        "world strategy should route through the recorded agent endpoint after flip: {stderr}"
    );
    assert!(
        stderr.contains(&socket_path.display().to_string()),
        "agent strategy should mention the recorded socket path: {stderr}"
    );
    assert!(
        stderr.contains(&cwd.display().to_string()),
        "agent strategy should mention the project_dir: {stderr}"
    );
    assert!(
        stderr.contains("[replay] scopes: [agent:uds:flip]"),
        "scopes should reflect agent execution after flip: {stderr}"
    );
    assert!(
        !stderr.contains("agent replay unavailable"),
        "agent path should not emit fallback warning when flip succeeds: {stderr}"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("agent-flip"),
        "flipped agent replay should return stubbed stdout payload: {stdout}"
    );

    let Some(strategy) = latest_replay_strategy(&trace_path(&fixture)) else {
        eprintln!("skipping flip-agent strategy assertions: no replay_strategy entries written");
        return;
    };
    assert_eq!(
        strategy.get("strategy").and_then(|value| value.as_str()),
        Some("agent"),
        "strategy should record agent execution after flip: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("recorded_origin")
            .and_then(|value| value.as_str()),
        Some("host"),
        "recorded_origin should reflect the original host span: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("target_origin")
            .and_then(|value| value.as_str()),
        Some("world"),
        "target_origin should reflect flipped world execution: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("recorded_origin_source")
            .and_then(|value| value.as_str()),
        Some("span"),
        "recorded_origin_source should track span metadata: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("origin_reason_code")
            .and_then(|value| value.as_str()),
        Some("flip_world"),
        "origin_reason_code should capture flip flag: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("recorded_transport")
            .and_then(|value| value.get("endpoint"))
            .and_then(|value| value.as_str()),
        Some(socket_str.as_str()),
        "recorded transport endpoint should be captured for flipped agent replay: {strategy:?}"
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
    let socket_str = socket_path.display().to_string();
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndExecute {
            stdout: "agent-ok\n".to_string(),
            stderr: String::new(),
            exit: 0,
            scopes: vec!["agent:uds:test".to_string()],
        },
    );

    let anchor_root = fixture.home().join("caged-root");
    let cwd = anchor_root.join("workspace-agent");
    fs::create_dir_all(&cwd).expect("failed to create replay cwd");
    let span_id = "span-agent-healthy";
    let path_override = configure_nft_stub(&fixture, "#!/bin/sh\necho \"nft (test) v1\"\nexit 0\n");
    let anchor_root_str = anchor_root.display().to_string();
    let options = TraceOptions {
        execution_origin: ExecutionOrigin::World,
        transport_endpoint: Some(socket_path.clone()),
        transport_socket_activation: Some(true),
        ..TraceOptions::default()
    };

    let mut cmd = replay_command_with_options(
        &fixture,
        span_id,
        "printf agent-ok > replay-agent.log",
        &cwd,
        &path_override,
        options,
    );
    cmd.env("SUBSTRATE_ANCHOR_MODE", "custom");
    cmd.env("SUBSTRATE_WORLD_ROOT_MODE", "custom");
    cmd.env("SUBSTRATE_ANCHOR_PATH", &anchor_root);
    cmd.env("SUBSTRATE_WORLD_ROOT_PATH", &anchor_root);
    cmd.env("SUBSTRATE_CAGED", "1");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[replay] origin: world"),
        "world origin should be reported in verbose output: {stderr}"
    );
    assert!(
        stderr.contains("[replay] world strategy: agent"),
        "expected agent strategy in stderr: {stderr}"
    );
    assert!(
        stderr.contains(&socket_path.display().to_string()),
        "agent world strategy should mention socket path: {stderr}"
    );
    assert!(
        stderr.contains(&anchor_root.display().to_string()),
        "agent world strategy should mention project_dir: {stderr}"
    );
    assert!(
        stderr.contains("[replay] scopes: [agent:uds:test]"),
        "scopes should reflect agent response in verbose output: {stderr}"
    );
    assert!(
        !stderr.contains("agent replay unavailable"),
        "agent path should not emit fallback warning: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("agent-ok"),
        "stdout should include agent response payload: {stdout}"
    );

    let Some(strategy) = latest_replay_strategy(&trace_path(&fixture)) else {
        eprintln!("skipping agent strategy assertions: no replay_strategy entries written");
        return;
    };
    assert_eq!(
        strategy.get("strategy").and_then(|value| value.as_str()),
        Some("agent"),
        "expected agent replay_strategy entry: {strategy:?}"
    );
    assert_eq!(
        strategy.get("project_dir").and_then(|value| value.as_str()),
        Some(anchor_root_str.as_str()),
        "project_dir should reflect caged anchor: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("agent_socket")
            .and_then(|value| value.as_str()),
        Some(socket_str.as_str()),
        "agent socket should be recorded in replay_strategy: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("recorded_origin")
            .and_then(|value| value.as_str()),
        Some("world"),
        "recorded_origin should be captured on the strategy entry: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("target_origin")
            .and_then(|value| value.as_str()),
        Some("world"),
        "target_origin should stay world when no flip/overrides are set: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("recorded_origin_source")
            .and_then(|value| value.as_str()),
        Some("span"),
        "recorded_origin_source should reflect span metadata: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("origin_reason_code")
            .and_then(|value| value.as_str()),
        Some("recorded_origin"),
        "origin_reason_code should reflect the recorded origin path: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("recorded_transport")
            .and_then(|value| value.get("endpoint"))
            .and_then(|value| value.as_str()),
        Some(socket_str.as_str()),
        "recorded transport endpoint should be captured alongside the agent socket: {strategy:?}"
    );
    assert!(
        strategy.get("fallback_reason").is_none(),
        "agent path should not record a fallback_reason: {strategy:?}"
    );
}

#[test]
fn replay_emits_single_agent_warning_and_retries_copydiff_on_enospc() {
    let fixture = ShellEnvFixture::new();
    let trace = trace_path(&fixture);
    let cwd = fixture.home().join("workspace-agent-missing");
    fs::create_dir_all(&cwd).expect("failed to create replay cwd");
    let cwd_str = cwd.display().to_string();
    let span_id = "span-agent-missing";
    let path_override = configure_nft_stub(&fixture, "#!/bin/sh\nexit 1\n");

    let shim = EnospcShim::build();
    let xdg_runtime = fixture.home().join("xdg-runtime");
    let enospc_prefix = xdg_runtime.join("substrate/copydiff");
    let missing_socket = PathBuf::from("/tmp/substrate-missing-agent.sock");
    let _ = fs::remove_file(&missing_socket);
    let missing_socket_str = missing_socket.display().to_string();
    let enospc_prefix_str = enospc_prefix.display().to_string();
    let options = TraceOptions {
        execution_origin: ExecutionOrigin::World,
        transport_endpoint: Some(missing_socket.clone()),
        transport_socket_activation: Some(false),
        ..TraceOptions::default()
    };

    let mut cmd = replay_command_with_options(
        &fixture,
        span_id,
        "printf fallback > replay-fallback.log",
        &cwd,
        &path_override,
        options,
    );
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
    assert!(
        stderr.contains("[replay] origin: world"),
        "world origin should be surfaced before agent fallback: {stderr}"
    );
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
        strategy.get("project_dir").and_then(|value| value.as_str()),
        Some(cwd_str.as_str()),
        "project_dir should match uncaged cwd: {strategy:?}"
    );
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
fn replay_agent_fallback_uses_caged_project_dir() {
    let fixture = ShellEnvFixture::new();
    let anchor_root = fixture.home().join("caged-root");
    let cwd = anchor_root.join("workspace-caged");
    fs::create_dir_all(&cwd).expect("failed to create replay cwd");
    let span_id = "span-agent-missing-caged";
    let path_override = configure_nft_stub(&fixture, "#!/bin/sh\nexit 1\n");
    let missing_socket = PathBuf::from("/tmp/substrate-missing-agent.sock");
    let _ = fs::remove_file(&missing_socket);
    let missing_socket_str = missing_socket.display().to_string();
    let anchor_root_str = anchor_root.display().to_string();
    let options = TraceOptions {
        execution_origin: ExecutionOrigin::World,
        transport_endpoint: Some(missing_socket.clone()),
        transport_socket_activation: Some(false),
        ..TraceOptions::default()
    };

    let mut cmd = replay_command_with_options(
        &fixture,
        span_id,
        "printf fallback-caged > replay-fallback-caged.log",
        &cwd,
        &path_override,
        options,
    );
    cmd.env("SUBSTRATE_ANCHOR_MODE", "custom");
    cmd.env("SUBSTRATE_WORLD_ROOT_MODE", "custom");
    cmd.env("SUBSTRATE_ANCHOR_PATH", &anchor_root);
    cmd.env("SUBSTRATE_WORLD_ROOT_PATH", &anchor_root);
    cmd.env("SUBSTRATE_CAGED", "1");

    let output = cmd.output().expect("failed to run replay command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        if copydiff_unavailable(&stderr) {
            eprintln!("skipping caged agent fallback replay test: copy-diff unavailable\n{stderr}");
            return;
        }
        panic!("replay command failed unexpectedly: {stderr}");
    }

    assert!(
        stderr.contains("[replay] origin: world"),
        "world origin should be logged before agent fallback: {stderr}"
    );
    let agent_warns = stderr.matches("agent replay unavailable").count();
    assert_eq!(
        agent_warns, 1,
        "agent fallback warning should be emitted once: {stderr}"
    );
    assert!(
        stderr.contains("[replay] copy-diff root:"),
        "copy-diff root should be printed after agent fallback: {stderr}"
    );

    let Some(strategy) = latest_replay_strategy(&trace_path(&fixture)) else {
        eprintln!("skipping caged agent strategy assertions: no replay_strategy entries written");
        return;
    };
    let fallback_reason = strategy
        .get("fallback_reason")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    assert!(
        fallback_reason.contains("agent socket missing"),
        "fallback reason should mention missing socket: {strategy:?}"
    );
    assert!(
        fallback_reason.contains(&missing_socket_str),
        "fallback reason should include missing socket path: {strategy:?}"
    );
    assert_eq!(
        strategy
            .get("agent_socket")
            .and_then(|value| value.as_str()),
        Some(missing_socket_str.as_str()),
        "agent socket path should be recorded: {strategy:?}"
    );
    assert_eq!(
        strategy.get("project_dir").and_then(|value| value.as_str()),
        Some(anchor_root_str.as_str()),
        "project_dir should reflect the caged anchor root: {strategy:?}"
    );
}

#[test]
fn replay_retries_copydiff_roots_and_dedupes_warnings() {
    let fixture = ShellEnvFixture::new();
    let trace = trace_path(&fixture);
    let cwd = fixture.home().join("workspace-copydiff-retries");
    fs::create_dir_all(&cwd).expect("failed to create replay cwd");
    let span_id = "span-copydiff-retries";
    let path_override = configure_nft_stub(&fixture, "#!/bin/sh\nexit 1\n");

    let uid = unsafe { libc::getuid() } as u32;
    let tmp_root = PathBuf::from(format!("/tmp/substrate-{}-copydiff", uid));
    let _ = fs::remove_dir_all(&tmp_root);
    let _ = fs::remove_file(&tmp_root);
    fs::write(&tmp_root, b"block tmp copydiff root")
        .expect("failed to block default tmp copydiff root");

    let shim = EnospcShim::build();
    let xdg_runtime = fixture.home().join("xdg-runtime");
    let enospc_prefix = xdg_runtime.join("substrate/copydiff");
    let missing_socket = fixture.home().join("missing-agent-copydiff.sock");
    let _ = fs::remove_file(&missing_socket);

    let mut cmd = replay_command_with_options(
        &fixture,
        span_id,
        "printf retry-roots > replay-copydiff-retries.log",
        &cwd,
        &path_override,
        TraceOptions {
            execution_origin: ExecutionOrigin::World,
            transport_endpoint: Some(missing_socket.clone()),
            transport_socket_activation: Some(false),
            ..TraceOptions::default()
        },
    );
    cmd.env("SUBSTRATE_WORLD_SOCKET", &missing_socket);
    cmd.env("XDG_RUNTIME_DIR", &xdg_runtime);
    cmd.env("LD_PRELOAD", &shim.path);
    cmd.env("SUBSTRATE_ENOSPC_PREFIX", &enospc_prefix);

    let output = cmd.output().expect("failed to run replay command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        if copydiff_unavailable(&stderr) {
            eprintln!("skipping copy-diff retry test: copy-diff unavailable\n{stderr}");
            let _ = fs::remove_file(&tmp_root);
            return;
        }
        panic!("replay command failed unexpectedly: {stderr}");
    }

    let retry_warnings = stderr.matches("retrying fallback location").count();
    assert!(
        retry_warnings >= 2,
        "expected multiple copy-diff retry warnings after /run and /tmp failures: {stderr}"
    );
    let tmp_root_str = tmp_root.display().to_string();
    assert_eq!(
        stderr.matches(tmp_root_str.as_str()).count(),
        1,
        "tmp root failure should be logged once: {stderr}"
    );
    let enospc_prefix_str = enospc_prefix.display().to_string();
    assert_eq!(
        stderr.matches(enospc_prefix_str.as_str()).count(),
        1,
        "xdg-runtime ENOSPC warning should be logged once: {stderr}"
    );
    assert!(
        stderr.contains("/run"),
        "expected copy-diff retry warnings to mention /run roots: {stderr}"
    );
    assert!(
        stderr.contains("[replay] copy-diff root:"),
        "final copy-diff root should be reported in verbose output: {stderr}"
    );

    let strategies = replay_strategy_entries(&trace);
    if strategies.is_empty() {
        eprintln!("skipping copy-diff retry assertions: no replay_strategy entries written");
        let _ = fs::remove_file(&tmp_root);
        return;
    }
    let strategy = strategies.last().unwrap();
    if strategy.get("strategy").and_then(|value| value.as_str()) != Some("copy-diff") {
        eprintln!(
            "skipping copy-diff retry assertions: expected copy-diff strategy, got {:?}",
            strategy.get("strategy")
        );
        let _ = fs::remove_file(&tmp_root);
        return;
    }
    let copydiff_root = strategy
        .get("copydiff_root")
        .and_then(|value| value.as_str())
        .expect("copydiff_root missing from replay_strategy");
    let copydiff_root_source = strategy
        .get("copydiff_root_source")
        .and_then(|value| value.as_str())
        .expect("copydiff_root_source missing from replay_strategy");
    assert!(
        copydiff_root.starts_with("/var/tmp"),
        "copy-diff should fall back to /var/tmp after /run and /tmp failures (got {copydiff_root})"
    );
    assert_eq!(
        copydiff_root_source, "/var/tmp",
        "copy-diff root source should reflect the /var/tmp fallback"
    );
    assert!(
        stderr.contains(&format!(
            "[replay] copy-diff root: {} ({})",
            copydiff_root, copydiff_root_source
        )),
        "verbose output should include the final copy-diff root and source: {stderr}"
    );
    let _ = fs::remove_file(&tmp_root);
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
