#![cfg(any(target_os = "linux", target_os = "macos"))]

#[path = "support/mod.rs"]
mod support;

use serde_json::Value;
use serial_test::serial;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use support::MemberDispatchStreamScript;
use support::{binary_path, ensure_substrate_built, temp_dir, ReplWorldAgentStub, StreamBehavior};
use tempfile::TempDir;

#[cfg(unix)]
fn set_fd_nonblocking(fd: i32) {
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL);
        if flags < 0 {
            return;
        }
        let _ = libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
    }
}

fn manager_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/manager_hooks.yaml")
}

fn write_profile(project_dir: &Path) {
    let profile = r#"id: test-policy
name: Test Policy
world_fs:
  host_visible: true
  fail_closed:
    routing: true
  write:
    enabled: true
net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null
metadata: {}
"#;
    fs::write(project_dir.join(".substrate-profile"), profile).expect("write .substrate-profile");
}

fn write_policy(home_substrate: &Path, require_world: bool) {
    write_policy_with_net_allowed(home_substrate, require_world, "[]");
}

fn write_policy_with_net_allowed(
    home_substrate: &Path,
    require_world: bool,
    net_allowed_yaml: &str,
) {
    fs::create_dir_all(home_substrate).expect("create SUBSTRATE_HOME");
    let require_world = if require_world { "true" } else { "false" };
    let policy = format!(
        r#"id: test-global-policy
name: Test Global Policy
world_fs:
  host_visible: true
  fail_closed:
    routing: {require_world}
  write:
    enabled: true
net_allowed: {net_allowed_yaml}
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null
metadata: {{}}
"#
    );
    fs::write(home_substrate.join("policy.yaml"), policy).expect("write policy.yaml");
}

fn write_config(home_substrate: &Path, world_net_filter: bool) {
    write_config_with_world_restart_on_drift(home_substrate, world_net_filter, "auto_restart");
}

fn write_config_with_world_restart_on_drift(
    home_substrate: &Path,
    world_net_filter: bool,
    on_drift: &str,
) {
    fs::create_dir_all(home_substrate).expect("create SUBSTRATE_HOME");
    let config = format!(
        r#"world:
  enabled: true
  anchor_mode: workspace
  anchor_path: ''
  caged: false
  net:
    filter: {}
policy:
  mode: observe
sync:
  auto_sync: false
  direction: from_world
  conflict_policy: prefer_host
  exclude: []
agents:
  hub:
    world_restart:
      on_drift: {}
"#,
        if world_net_filter { "true" } else { "false" },
        on_drift,
    );
    fs::write(home_substrate.join("config.yaml"), config).expect("write config.yaml");
}

fn short_socket_dir(prefix: &str) -> TempDir {
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in("/tmp")
        .expect("create short socket tempdir in /tmp")
}

#[derive(Debug)]
struct ReplRoutingCase<'a> {
    name: &'a str,
    net_allowed_yaml: &'a str,
    world_net_filter: bool,
    expected_net_allowed: &'a [&'a str],
    expected_isolate_network: bool,
    expected_allowed_domains: &'a [&'a str],
}

fn wait_for_min_start_sessions(
    records: &Arc<Mutex<support::ReplWorldAgentRecords>>,
    min_starts: usize,
    timeout: Duration,
) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let guard = records.lock().expect("lock records");
        if guard.persistent_start_sessions.len() >= min_starts {
            return;
        }
        drop(guard);
        std::thread::sleep(Duration::from_millis(25));
    }

    let guard = records.lock().expect("lock records");
    panic!(
        "timed out waiting for persistent start sessions >= {min_starts}; got {}; records: {guard:#?}",
        guard.persistent_start_sessions.len(),
    );
}

fn wait_for_min_start_sessions_with_output(
    repl: &PtyRepl,
    records: &Arc<Mutex<support::ReplWorldAgentRecords>>,
    min_starts: usize,
    timeout: Duration,
) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let guard = records.lock().expect("lock records");
        if guard.persistent_start_sessions.len() >= min_starts {
            return;
        }
        drop(guard);
        std::thread::sleep(Duration::from_millis(25));
    }

    let guard = records.lock().expect("lock records");
    panic!(
        "timed out waiting for persistent start sessions >= {min_starts}; got {}; output:\n{}\nrecords: {guard:#?}",
        guard.persistent_start_sessions.len(),
        repl.output_string(),
    );
}

fn wait_for_min_records(
    records: &Arc<Mutex<support::ReplWorldAgentRecords>>,
    min_execs: usize,
    min_starts: usize,
    timeout: Duration,
) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let guard = records.lock().expect("lock records");
        if guard.persistent_execs.len() >= min_execs
            && guard.persistent_start_sessions.len() >= min_starts
        {
            return;
        }
        drop(guard);
        std::thread::sleep(Duration::from_millis(25));
    }

    let guard = records.lock().expect("lock records");
    panic!(
        "timed out waiting for records execs>={min_execs} starts>={min_starts}; got execs={} starts={}; records: {guard:#?}",
        guard.persistent_execs.len(),
        guard.persistent_start_sessions.len(),
    );
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn wait_for_min_member_dispatch_requests(
    records: &Arc<Mutex<support::ReplWorldAgentRecords>>,
    min_requests: usize,
    timeout: Duration,
) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let guard = records.lock().expect("lock records");
        if guard.member_dispatch_requests.len() >= min_requests {
            return;
        }
        drop(guard);
        std::thread::sleep(Duration::from_millis(25));
    }

    let guard = records.lock().expect("lock records");
    panic!(
        "timed out waiting for member_dispatch requests >= {min_requests}; got {}; records: {guard:#?}",
        guard.member_dispatch_requests.len(),
    );
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn wait_for_min_member_turn_submit_requests(
    records: &Arc<Mutex<support::ReplWorldAgentRecords>>,
    min_requests: usize,
    timeout: Duration,
) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let guard = records.lock().expect("lock records");
        if guard.member_turn_submit_requests.len() >= min_requests {
            return;
        }
        drop(guard);
        std::thread::sleep(Duration::from_millis(25));
    }

    let guard = records.lock().expect("lock records");
    panic!(
        "timed out waiting for member_turn_submit requests >= {min_requests}; got {}; records: {guard:#?}",
        guard.member_turn_submit_requests.len(),
    );
}

fn wait_for_min_execute_cancel_requests(
    records: &Arc<Mutex<support::ReplWorldAgentRecords>>,
    min_requests: usize,
    timeout: Duration,
) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let guard = records.lock().expect("lock records");
        if guard.execute_cancel_requests.len() >= min_requests {
            return;
        }
        drop(guard);
        std::thread::sleep(Duration::from_millis(25));
    }

    let guard = records.lock().expect("lock records");
    panic!(
        "timed out waiting for execute/cancel requests >= {min_requests}; got {}; records: {guard:#?}",
        guard.execute_cancel_requests.len(),
    );
}

fn read_trace(trace_path: &Path) -> Vec<Value> {
    fs::read_to_string(trace_path)
        .expect("read trace")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("parse trace line"))
        .collect()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn write_orchestrator_runtime_world_config(
    home_substrate: &Path,
    fake_codex: &Path,
    on_drift: &str,
) {
    fs::create_dir_all(home_substrate.join("agents")).expect("create agents dir");
    let config = format!(
        r#"world:
  enabled: true
  anchor_mode: workspace
  anchor_path: ''
  caged: false
  net:
    filter: false
policy:
  mode: observe
sync:
  auto_sync: false
  direction: from_world
  conflict_policy: prefer_host
  exclude: []
agents:
  enabled: true
  hub:
    orchestrator_agent_id: codex
    world_restart:
      on_drift: {on_drift}
"#
    );
    fs::write(home_substrate.join("config.yaml"), config).expect("write config.yaml");
    fs::write(
        home_substrate.join("policy.yaml"),
        "id: test-global-policy\nname: Test Global Policy\nworld_fs:\n  host_visible: true\n  fail_closed:\n    routing: true\n  write:\n    enabled: true\nnet_allowed: []\ncmd_allowed: []\ncmd_denied: []\ncmd_isolated: []\nrequire_approval: false\nallow_shell_operators: true\nlimits:\n  max_memory_mb: null\n  max_cpu_percent: null\n  max_runtime_ms: null\n  max_egress_bytes: null\nmetadata: {}\nagents:\n  allowed_backends:\n    - cli:codex\n",
    )
    .expect("write agent runtime policy");
    fs::write(
        home_substrate.join("agents/codex.yaml"),
        format!(
            "version: 1\nid: codex\nconfig:\n  kind: cli\n  enabled: true\n  protocol: uaa.agent.session\n  execution:\n    scope: host\n  cli:\n    binary: {}\n    mode: persistent\n  capabilities:\n    session_start: true\n    session_resume: true\n    session_fork: true\n    session_stop: true\n    status_snapshot: true\n    event_stream: true\n    llm: true\n    mcp_client: false\n",
            fake_codex.display()
        ),
    )
    .expect("write codex agent file");
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn write_orchestrator_and_world_member_runtime_world_config(
    home_substrate: &Path,
    fake_orchestrator: &Path,
    fake_member: &Path,
    on_drift: &str,
) {
    fs::create_dir_all(home_substrate.join("agents")).expect("create agents dir");
    let config = format!(
        r#"world:
  enabled: true
  anchor_mode: workspace
  anchor_path: ''
  caged: false
  net:
    filter: false
policy:
  mode: observe
sync:
  auto_sync: false
  direction: from_world
  conflict_policy: prefer_host
  exclude: []
agents:
  enabled: true
  hub:
    orchestrator_agent_id: claude_code
    world_restart:
      on_drift: {on_drift}
"#
    );
    fs::write(home_substrate.join("config.yaml"), config).expect("write config.yaml");
    write_member_runtime_policy(home_substrate, true);
    fs::write(
        home_substrate.join("agents/claude_code.yaml"),
        format!(
            "version: 1\nid: claude_code\nconfig:\n  kind: cli\n  enabled: true\n  protocol: uaa.agent.session\n  execution:\n    scope: host\n  cli:\n    binary: {}\n    mode: persistent\n  capabilities:\n    session_start: true\n    session_resume: true\n    session_fork: true\n    session_stop: true\n    status_snapshot: true\n    event_stream: true\n    llm: true\n    mcp_client: false\n",
            fake_orchestrator.display()
        ),
    )
    .expect("write claude_code agent file");
    fs::write(
        home_substrate.join("agents/codex.yaml"),
        format!(
            "version: 1\nid: codex\nconfig:\n  kind: cli\n  enabled: true\n  protocol: uaa.agent.session\n  execution:\n    scope: world\n  cli:\n    binary: {}\n    mode: persistent\n  capabilities:\n    session_start: true\n    session_resume: true\n    session_fork: true\n    session_stop: true\n    status_snapshot: true\n    event_stream: true\n    llm: true\n    mcp_client: false\n",
            fake_member.display()
        ),
    )
    .expect("write codex agent file");
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn write_dual_host_runtime_world_config(
    home_substrate: &Path,
    fake_orchestrator: &Path,
    fake_secondary_host: &Path,
    on_drift: &str,
) {
    fs::create_dir_all(home_substrate.join("agents")).expect("create agents dir");
    let config = format!(
        r#"world:
  enabled: true
  anchor_mode: workspace
  anchor_path: ''
  caged: false
  net:
    filter: false
policy:
  mode: observe
sync:
  auto_sync: false
  direction: from_world
  conflict_policy: prefer_host
  exclude: []
agents:
  enabled: true
  hub:
    orchestrator_agent_id: claude_code
    world_restart:
      on_drift: {on_drift}
"#
    );
    fs::write(home_substrate.join("config.yaml"), config).expect("write config.yaml");
    write_member_runtime_policy(home_substrate, true);
    fs::write(
        home_substrate.join("agents/claude_code.yaml"),
        format!(
            "version: 1\nid: claude_code\nconfig:\n  kind: cli\n  enabled: true\n  protocol: uaa.agent.session\n  execution:\n    scope: host\n  cli:\n    binary: {}\n    mode: persistent\n  capabilities:\n    session_start: true\n    session_resume: true\n    session_fork: true\n    session_stop: true\n    status_snapshot: true\n    event_stream: true\n    llm: true\n    mcp_client: false\n",
            fake_orchestrator.display()
        ),
    )
    .expect("write claude_code agent file");
    fs::write(
        home_substrate.join("agents/codex.yaml"),
        format!(
            "version: 1\nid: codex\nconfig:\n  kind: cli\n  enabled: true\n  protocol: uaa.agent.session\n  execution:\n    scope: host\n  cli:\n    binary: {}\n    mode: persistent\n  capabilities:\n    session_start: true\n    session_resume: true\n    session_fork: true\n    session_stop: true\n    status_snapshot: true\n    event_stream: true\n    llm: true\n    mcp_client: false\n",
            fake_secondary_host.display()
        ),
    )
    .expect("write codex agent file");
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn write_member_runtime_policy(home_substrate: &Path, require_world: bool) {
    fs::create_dir_all(home_substrate).expect("create SUBSTRATE_HOME");
    let require_world = if require_world { "true" } else { "false" };
    let policy = format!(
        r#"id: test-global-policy
name: Test Global Policy
world_fs:
  host_visible: true
  fail_closed:
    routing: {require_world}
  write:
    enabled: true
net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null
metadata: {{}}
agents:
  allowed_backends:
    - cli:claude_code
    - cli:codex
"#
    );
    fs::write(home_substrate.join("policy.yaml"), policy).expect("write policy.yaml");
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn write_fake_codex_script(temp: &Path) -> PathBuf {
    let path = temp.join("fake-codex.sh");
    let body = "#!/bin/sh\ntrap 'exit 0' INT TERM\nprintf '{\"type\":\"thread.started\",\"thread_id\":\"thread-test\"}\\r\\n'\nprintf '{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\"}\\r\\n'\nwhile :; do sleep 1; done\n";
    fs::write(&path, body).expect("write fake codex script");
    let mut perms = fs::metadata(&path)
        .expect("fake codex metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set fake codex permissions");
    path
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn write_fake_codex_script_with_invocation_log_and_output(temp: &Path) -> (PathBuf, PathBuf) {
    let path = temp.join("fake-codex-with-log.sh");
    let count_path = temp.join("fake-codex-with-log.count");
    let body = format!(
        "#!/bin/sh\nSTATE_FILE='{}'\ncount=0\nif [ -f \"$STATE_FILE\" ]; then\n  count=$(cat \"$STATE_FILE\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$STATE_FILE\"\ntrap 'exit 0' INT TERM\nprintf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-test\"}}\\r\\n'\nprintf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\"}}\\r\\n'\nwhile :; do sleep 1; done\n",
        count_path.display()
    );
    fs::write(&path, body).expect("write fake codex script with invocation log");
    let mut perms = fs::metadata(&path)
        .expect("fake codex metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set fake codex permissions");
    (path, count_path)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn write_fake_claude_script(temp: &Path) -> PathBuf {
    let path = temp.join("fake-claude.sh");
    let body = "#!/bin/sh\ntrap 'exit 0' INT TERM\nprintf '{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"session-test\"}\\r\\n'\nwhile :; do sleep 1; done\n";
    fs::write(&path, body).expect("write fake claude script");
    let mut perms = fs::metadata(&path)
        .expect("fake claude metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set fake claude permissions");
    path
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn write_fake_claude_script_first_session_then_exit(temp: &Path) -> PathBuf {
    let path = temp.join("fake-claude-first-session-then-exit.sh");
    let state_path = temp.join("fake-claude-first-session-then-exit.count");
    let body = format!(
        "#!/bin/sh\nSTATE_FILE='{}'\ncount=0\nif [ -f \"$STATE_FILE\" ]; then\n  count=$(cat \"$STATE_FILE\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$STATE_FILE\"\nif [ \"$count\" -eq 1 ]; then\n  trap 'exit 0' INT TERM\n  printf '{{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"session-test\"}}\\r\\n'\n  while :; do sleep 1; done\nfi\nexit 0\n",
        state_path.display()
    );
    fs::write(&path, body).expect("write fake claude first-session-then-exit script");
    let mut perms = fs::metadata(&path)
        .expect("fake claude first-session-then-exit metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set fake claude first-session-then-exit permissions");
    path
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn write_fake_codex_script_first_success_then_fail_without_session_handle(temp: &Path) -> PathBuf {
    let path = temp.join("fake-codex-first-success-then-fail.sh");
    let state_path = temp.join("fake-codex-first-success-then-fail.count");
    let body = format!(
        "#!/bin/sh\nSTATE_FILE='{}'\ncount=0\nif [ -f \"$STATE_FILE\" ]; then\n  count=$(cat \"$STATE_FILE\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$STATE_FILE\"\nif [ \"$count\" -eq 1 ]; then\n  trap 'exit 0' INT TERM\n  printf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-test\"}}\\r\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\"}}\\r\\n'\n  while :; do sleep 1; done\nfi\nprintf 'bootstrap-without-session-handle\\n'\n",
        state_path.display()
    );
    fs::write(&path, body).expect("write fake codex replacement failure script");
    let mut perms = fs::metadata(&path)
        .expect("fake codex metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set fake codex permissions");
    path
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn write_fake_codex_script_without_session_handle(temp: &Path) -> PathBuf {
    let path = temp.join("fake-codex-no-session-handle.sh");
    let body = "#!/bin/sh\nprintf 'bootstrap-without-session-handle\\n'\n";
    fs::write(&path, body).expect("write fake codex script without session handle");
    let mut perms = fs::metadata(&path)
        .expect("fake codex metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set fake codex permissions");
    path
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn read_invocation_count(path: &Path) -> usize {
    fs::read_to_string(path)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(0)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn load_single_orchestration_session_id(substrate_home: &Path) -> String {
    let sessions_dir = sessions_dir(substrate_home);
    let mut canonical_entries = fs::read_dir(&sessions_dir)
        .expect("read orchestration session dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .map(|path| path.join("session.json"))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    canonical_entries.sort();
    if let Some(session_path) = canonical_entries.into_iter().next() {
        return serde_json::from_str::<Value>(
            &fs::read_to_string(session_path).expect("read canonical session file"),
        )
        .expect("parse canonical session file")
        .get("orchestration_session_id")
        .and_then(Value::as_str)
        .expect("session orchestration_session_id")
        .to_string();
    }

    let mut entries = fs::read_dir(&sessions_dir)
        .expect("read orchestration session dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    entries.sort();
    let session_path = entries
        .into_iter()
        .next()
        .expect("orchestration session file");
    serde_json::from_str::<Value>(&fs::read_to_string(session_path).expect("read session file"))
        .expect("parse session file")
        .get("orchestration_session_id")
        .and_then(Value::as_str)
        .expect("session orchestration_session_id")
        .to_string()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn sessions_dir(substrate_home: &Path) -> PathBuf {
    substrate_home.join("run/agent-hub/sessions")
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn orchestration_session_path(substrate_home: &Path, orchestration_session_id: &str) -> PathBuf {
    sessions_dir(substrate_home)
        .join(orchestration_session_id)
        .join("session.json")
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn canonical_participants_dir(substrate_home: &Path, orchestration_session_id: &str) -> PathBuf {
    sessions_dir(substrate_home)
        .join(orchestration_session_id)
        .join("participants")
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn canonical_participant_path(
    substrate_home: &Path,
    orchestration_session_id: &str,
    participant_id: &str,
) -> PathBuf {
    canonical_participants_dir(substrate_home, orchestration_session_id)
        .join(format!("{participant_id}.json"))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn read_orchestration_session(session_path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(session_path).expect("read orchestration session"))
        .expect("parse orchestration session")
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn assert_session_world_binding(
    session: &Value,
    expected_world_id: Option<&str>,
    expected_world_generation: Option<u64>,
) {
    assert_eq!(
        session.pointer("/world_id").and_then(Value::as_str),
        expected_world_id,
        "unexpected persisted parent world_id: {session:?}"
    );
    assert_eq!(
        session.pointer("/world_generation").and_then(Value::as_u64),
        expected_world_generation,
        "unexpected persisted parent world_generation: {session:?}"
    );
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn flat_participants_dir(substrate_home: &Path) -> PathBuf {
    substrate_home.join("run/agent-hub/participants")
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn flat_participant_manifest_path(substrate_home: &Path, participant_id: &str) -> PathBuf {
    flat_participants_dir(substrate_home).join(format!("{participant_id}.json"))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn read_trace_lenient(trace_path: &Path) -> Vec<Value> {
    fs::read_to_string(trace_path)
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn read_participant_manifest(substrate_home: &Path, participant_id: &str) -> Value {
    let canonical_path = fs::read_dir(sessions_dir(substrate_home))
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .map(|path| {
            path.join("participants")
                .join(format!("{participant_id}.json"))
        })
        .find(|path| path.is_file());
    let path = canonical_path
        .unwrap_or_else(|| flat_participant_manifest_path(substrate_home, participant_id));
    serde_json::from_str(&fs::read_to_string(path).expect("read participant manifest"))
        .expect("parse participant manifest")
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn write_live_world_member_manifest(
    substrate_home: &Path,
    orchestration_session_id: &str,
    participant_id: &str,
    orchestrator_participant_id: &str,
    world_id: &str,
    world_generation: u64,
) {
    fs::create_dir_all(canonical_participants_dir(
        substrate_home,
        orchestration_session_id,
    ))
    .expect("create canonical participants dir");
    let ts = "2026-04-30T12:00:00Z";
    let payload = serde_json::json!({
        "participant_id": participant_id,
        "orchestration_session_id": orchestration_session_id,
        "agent_id": "codex",
        "backend_id": "cli:codex",
        "role": "member",
        "protocol": "uaa.agent.session",
        "execution": { "scope": "world" },
        "state": "ready",
        "opened_at": ts,
        "last_transition_at": ts,
        "world_id": world_id,
        "world_generation": world_generation,
        "parent_session_handle_id": Value::Null,
        "resumed_from_session_handle_id": Value::Null,
        "orchestrator_participant_id": orchestrator_participant_id,
        "internal": {
            "resolved_agent_kind": "codex",
            "resolved_binary_path": "/usr/bin/codex",
            "shell_owner_pid": std::process::id(),
            "lease_token": format!("lease_{participant_id}"),
            "uaa_session_id": "uaa_session",
            "latest_run_id": Value::Null,
            "cancel_supported": true,
            "control_owner_retained": true,
            "event_stream_active": true,
            "completion_observer_retained": true,
            "ownership_mode": "member_runtime",
            "ownership_valid": true,
            "ownership_verified_at": ts,
            "last_heartbeat_at": ts,
            "last_event_at": ts,
            "terminal_observed_at": Value::Null,
            "termination_reason": Value::Null,
            "last_error_bucket": Value::Null,
            "last_error_message": Value::Null
        }
    });
    fs::write(
        canonical_participant_path(substrate_home, orchestration_session_id, participant_id),
        serde_json::to_vec_pretty(&payload).expect("serialize participant manifest"),
    )
    .expect("write participant manifest");
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn session_participant_manifests(
    substrate_home: &Path,
    orchestration_session_id: &str,
) -> Vec<Value> {
    use std::collections::BTreeMap;

    let mut manifests = BTreeMap::new();

    let canonical_dir = canonical_participants_dir(substrate_home, orchestration_session_id);
    for path in fs::read_dir(canonical_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
    {
        let manifest = serde_json::from_str::<Value>(
            &fs::read_to_string(path).expect("read canonical participant file"),
        )
        .expect("parse canonical participant file");
        let Some(participant_id) = manifest
            .get("participant_id")
            .and_then(Value::as_str)
            .map(str::to_string)
        else {
            continue;
        };
        manifests.insert(participant_id, manifest);
    }

    for manifest in fs::read_dir(flat_participants_dir(substrate_home))
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .map(|path| {
            serde_json::from_str::<Value>(
                &fs::read_to_string(path).expect("read flat participant file"),
            )
            .expect("parse flat participant file")
        })
        .filter(|manifest| {
            manifest
                .get("orchestration_session_id")
                .and_then(Value::as_str)
                == Some(orchestration_session_id)
        })
    {
        let Some(participant_id) = manifest
            .get("participant_id")
            .and_then(Value::as_str)
            .map(str::to_string)
        else {
            continue;
        };
        manifests.entry(participant_id).or_insert(manifest);
    }

    manifests.into_values().collect()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn participant_is_authoritative_live(manifest: &Value) -> bool {
    let Some(state) = manifest.get("state").and_then(Value::as_str) else {
        return false;
    };
    let live_state = matches!(
        state,
        "allocating" | "ready" | "running" | "restarting" | "stopping"
    );
    live_state
        && manifest
            .pointer("/internal/uaa_session_id")
            .and_then(Value::as_str)
            .is_some()
        && manifest
            .pointer("/internal/control_owner_retained")
            .and_then(Value::as_bool)
            == Some(true)
        && manifest
            .pointer("/internal/event_stream_active")
            .and_then(Value::as_bool)
            == Some(true)
        && manifest
            .pointer("/internal/completion_observer_retained")
            .and_then(Value::as_bool)
            == Some(true)
        && manifest
            .pointer("/internal/ownership_valid")
            .and_then(Value::as_bool)
            == Some(true)
        && manifest
            .pointer("/internal/terminal_observed_at")
            .is_none_or(Value::is_null)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn live_world_member_generations_for_session(
    substrate_home: &Path,
    orchestration_session_id: &str,
) -> Vec<(String, u64)> {
    let mut live = session_participant_manifests(substrate_home, orchestration_session_id)
        .into_iter()
        .filter(|manifest| manifest.get("role").and_then(Value::as_str) == Some("member"))
        .filter(|manifest| {
            manifest.pointer("/execution/scope").and_then(Value::as_str) == Some("world")
        })
        .filter(participant_is_authoritative_live)
        .filter_map(|manifest| {
            Some((
                manifest.get("participant_id")?.as_str()?.to_string(),
                manifest.get("world_generation")?.as_u64()?,
            ))
        })
        .collect::<Vec<_>>();
    live.sort();
    live
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn world_member_manifests_for_session(
    substrate_home: &Path,
    orchestration_session_id: &str,
) -> Vec<Value> {
    let mut manifests = session_participant_manifests(substrate_home, orchestration_session_id)
        .into_iter()
        .filter(|manifest| manifest.get("role").and_then(Value::as_str) == Some("member"))
        .filter(|manifest| {
            manifest.pointer("/execution/scope").and_then(Value::as_str) == Some("world")
        })
        .collect::<Vec<_>>();
    manifests.sort_by(|left, right| {
        left.get("participant_id")
            .and_then(Value::as_str)
            .cmp(&right.get("participant_id").and_then(Value::as_str))
    });
    manifests
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn authoritative_live_participant_manifests_for_session(
    substrate_home: &Path,
    orchestration_session_id: &str,
) -> Vec<Value> {
    let mut manifests = session_participant_manifests(substrate_home, orchestration_session_id)
        .into_iter()
        .filter(participant_is_authoritative_live)
        .collect::<Vec<_>>();
    manifests.sort_by(|left, right| {
        left.get("backend_id")
            .and_then(Value::as_str)
            .cmp(&right.get("backend_id").and_then(Value::as_str))
    });
    manifests
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn authoritative_live_participant_manifest_for_backend<'a>(
    manifests: &'a [Value],
    backend_id: &str,
) -> &'a Value {
    manifests
        .iter()
        .find(|manifest| manifest.get("backend_id").and_then(Value::as_str) == Some(backend_id))
        .unwrap_or_else(|| panic!("missing authoritative-live participant for {backend_id}"))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn authoritative_live_world_member_manifests_for_session(
    substrate_home: &Path,
    orchestration_session_id: &str,
) -> Vec<Value> {
    world_member_manifests_for_session(substrate_home, orchestration_session_id)
        .into_iter()
        .filter(participant_is_authoritative_live)
        .collect()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn assert_world_member_absent_for_interval(
    substrate_home: &Path,
    orchestration_session_id: &str,
    interval: Duration,
) {
    let deadline = Instant::now() + interval;
    while Instant::now() < deadline {
        let members = authoritative_live_world_member_manifests_for_session(
            substrate_home,
            orchestration_session_id,
        );
        assert!(
            members.is_empty(),
            "world member must remain absent before the first world-backed command: {members:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn wait_for_live_world_member_count(
    substrate_home: &Path,
    orchestration_session_id: &str,
    expected_count: usize,
    timeout: Duration,
) -> Vec<Value> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let members = authoritative_live_world_member_manifests_for_session(
            substrate_home,
            orchestration_session_id,
        );
        if members.len() == expected_count {
            return members;
        }
        std::thread::sleep(Duration::from_millis(25));
    }

    panic!(
        "timed out waiting for authoritative live world member count == {expected_count}; got {:?}",
        authoritative_live_world_member_manifests_for_session(
            substrate_home,
            orchestration_session_id
        ),
    );
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn wait_for_world_restarted_alert_without_stale_liveness(
    trace_path: &Path,
    substrate_home: &Path,
    orchestration_session_id: &str,
    stale_generation: u64,
    timeout: Duration,
) -> Value {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let events = read_trace_lenient(trace_path);
        let alert = events.into_iter().find(|event| {
            event.get("event_type").and_then(Value::as_str) == Some("agent_event")
                && event.get("kind").and_then(Value::as_str) == Some("alert")
                && event.pointer("/data/code").and_then(Value::as_str) == Some("world_restarted")
                && event
                    .get("orchestration_session_id")
                    .and_then(Value::as_str)
                    == Some(orchestration_session_id)
        });
        let stale_live =
            live_world_member_generations_for_session(substrate_home, orchestration_session_id)
                .into_iter()
                .filter(|(_, generation)| *generation == stale_generation)
                .collect::<Vec<_>>();
        if let Some(alert) = alert {
            assert!(
                stale_live.is_empty(),
                "world_restarted published before stale generation {stale_generation} invalidated: alert={alert:?} stale_live={stale_live:?}"
            );
            return alert;
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    panic!(
        "timed out waiting for world_restarted without stale live generation {stale_generation}; live={:?}; trace={:?}",
        live_world_member_generations_for_session(substrate_home, orchestration_session_id),
        read_trace_lenient(trace_path),
    );
}

fn alert_rows_by_code<'a>(events: &'a [Value], code: &str) -> Vec<&'a Value> {
    events
        .iter()
        .filter(|event| event.get("event_type").and_then(Value::as_str) == Some("agent_event"))
        .filter(|event| event.get("kind").and_then(Value::as_str) == Some("alert"))
        .filter(|event| event.pointer("/data/code").and_then(Value::as_str) == Some(code))
        .collect()
}

fn assert_no_alert_rows_by_code(events: &[Value], code: &str, scenario: &str) {
    let alerts = alert_rows_by_code(events, code);
    assert!(
        alerts.is_empty(),
        "{scenario}: explicit-context-only suppression must prevent `{code}` agent_event rows when no shared-world context exists: {alerts:?}"
    );
}

fn assert_start_sessions_have_no_shared_world_context(
    records: &support::ReplWorldAgentRecords,
    scenario: &str,
) {
    assert!(
        records
            .persistent_start_sessions
            .iter()
            .all(|start| start.shared_world.is_none()),
        "{scenario}: harness only exposes explicit orchestration context through start_session.shared_world; unexpected shared-world requests: {:#?}",
        records.persistent_start_sessions
    );
}

fn assert_world_network_payload(
    start: &support::PersistentStartSessionRecord,
    expected_isolate_network: bool,
    expected_allowed_domains: &[&str],
) {
    let isolate_network = start
        .world_network
        .get("isolate_network")
        .and_then(|value| value.as_bool())
        .expect("world_network.isolate_network bool");
    assert_eq!(
        isolate_network, expected_isolate_network,
        "unexpected world_network.isolate_network"
    );

    let allowed_domains = start
        .world_network
        .get("allowed_domains")
        .and_then(|value| value.as_array())
        .expect("world_network.allowed_domains array");
    let allowed_domains: Vec<&str> = allowed_domains
        .iter()
        .map(|value| value.as_str().expect("allowed_domains string"))
        .collect();
    assert_eq!(
        allowed_domains, expected_allowed_domains,
        "unexpected world_network.allowed_domains"
    );
}

fn run_repl_routing_case(case: &ReplRoutingCase<'_>) {
    let temp = temp_dir("substrate-repl-net-allowed-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy_with_net_allowed(&substrate_home, true, case.net_allowed_yaml);
    write_config(&substrate_home, case.world_net_filter);

    let sock_temp = short_socket_dir("sub-c3ws-routing-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    wait_for_min_start_sessions_with_output(&repl, &records, 1, Duration::from_secs(3));
    repl.send_line("exit");

    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(2));

    let guard = records.lock().expect("lock records");
    assert_start_sessions_have_no_shared_world_context(&guard, case.name);
    let start = guard
        .persistent_start_sessions
        .first()
        .expect("persistent start session");
    let net_allowed = start
        .policy_snapshot
        .get("net_allowed")
        .and_then(|value| value.as_array())
        .expect("policy_snapshot.net_allowed array");
    let net_allowed: Vec<&str> = net_allowed
        .iter()
        .map(|value| value.as_str().expect("net_allowed string"))
        .collect();
    assert_eq!(
        net_allowed, case.expected_net_allowed,
        "{}: unexpected canonical policy_snapshot.net_allowed",
        case.name
    );
    assert_world_network_payload(
        start,
        case.expected_isolate_network,
        case.expected_allowed_domains,
    );
}

fn write_workspace_marker(workspace_root: &Path) {
    let dir = workspace_root.join(".substrate");
    fs::create_dir_all(&dir).expect("create .substrate");
    let cfg = r#"world:
  enabled: true
  anchor_mode: workspace
  anchor_path: ''
  caged: false
policy:
  mode: observe
sync:
  auto_sync: false
  direction: from_world
  conflict_policy: prefer_host
  exclude: []
"#;
    fs::write(dir.join("workspace.yaml"), cfg).expect("write workspace.yaml");
}

struct PtyRepl {
    child: Box<dyn portable_pty::Child + Send>,
    master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    waited: Option<portable_pty::ExitStatus>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    output: Arc<Mutex<Vec<u8>>>,
    reader_handle: Option<std::thread::JoinHandle<()>>,
    stop_reader: Arc<AtomicBool>,
}

impl PtyRepl {
    fn spawn(
        project_dir: &Path,
        home_dir: &Path,
        substrate_home: &Path,
        socket_path: &Path,
        extra_env: &[(&str, &str)],
        args: &[&str],
    ) -> Self {
        Self::spawn_inner(
            project_dir,
            home_dir,
            substrate_home,
            socket_path,
            extra_env,
            args,
            None,
        )
    }

    fn spawn_inner(
        project_dir: &Path,
        home_dir: &Path,
        substrate_home: &Path,
        socket_path: &Path,
        extra_env: &[(&str, &str)],
        args: &[&str],
        startup_delay: Option<Duration>,
    ) -> Self {
        ensure_substrate_built();

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("openpty");

        let master = pair.master;

        #[cfg(unix)]
        let master_fd = master.as_raw_fd();

        #[cfg(unix)]
        if let Some(fd) = master_fd {
            set_fd_nonblocking(fd);
        }

        let substrate_binary = binary_path();
        let mut cmd = if let Some(startup_delay) = startup_delay {
            let mut cmd = CommandBuilder::new("bash");
            cmd.arg("-lc");
            cmd.arg(format!(
                "sleep {:.3}; exec \"$0\" \"$@\"",
                startup_delay.as_secs_f64()
            ));
            cmd.arg(substrate_binary.clone());
            cmd
        } else {
            CommandBuilder::new(substrate_binary)
        };
        cmd.args(args);
        cmd.cwd(project_dir);
        cmd.env("HOME", home_dir);
        cmd.env("USERPROFILE", home_dir);
        cmd.env("SUBSTRATE_HOME", substrate_home);
        cmd.env("SUBSTRATE_MANAGER_MANIFEST", manager_manifest_path());
        cmd.env("SUBSTRATE_CAGED", "0");
        cmd.arg("--uncaged");
        cmd.env("SHIM_TRACE_LOG", home_dir.join(".substrate/trace.jsonl"));
        cmd.env("SUBSTRATE_WORLD_SOCKET", socket_path);
        cmd.env("SUBSTRATE_OVERRIDE_WORLD", "enabled");
        cmd.env_remove("SHIM_ORIGINAL_PATH");
        cmd.env_remove("SUBSTRATE_WORLD");
        cmd.env_remove("SUBSTRATE_WORLD_ENABLED");
        cmd.env_remove("SUBSTRATE_WORLD_ID");
        cmd.env("SHELL", "/bin/bash");
        cmd.arg("--shim-skip");
        for (k, v) in extra_env {
            cmd.env(k, v);
        }

        let child = pair.slave.spawn_command(cmd).expect("spawn substrate");
        let writer: Arc<Mutex<Box<dyn Write + Send>>> =
            Arc::new(Mutex::new(master.take_writer().expect("take writer")));

        let output = Arc::new(Mutex::new(Vec::new()));
        let stop_reader = Arc::new(AtomicBool::new(false));
        let output_for_thread = output.clone();
        let writer_for_thread = writer.clone();
        let stop_for_thread = stop_reader.clone();
        let reader_handle = std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            let mut carry = Vec::<u8>::new();
            loop {
                if stop_for_thread.load(Ordering::Relaxed) {
                    break;
                }

                #[cfg(unix)]
                if let Some(fd) = master_fd {
                    let rc = unsafe { libc::read(fd, buf.as_mut_ptr().cast(), buf.len()) };
                    if rc == 0 {
                        break;
                    }
                    if rc < 0 {
                        let e = std::io::Error::last_os_error();
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            std::thread::sleep(Duration::from_millis(25));
                            continue;
                        }
                        break;
                    }

                    let n: usize = rc as usize;
                    // Reedline/crossterm may emit terminal queries which require a response
                    // from the terminal emulator. When running under a raw PTY in tests,
                    // provide minimal responses so the REPL can make progress.
                    //
                    // - DSR (cursor position): ESC [ 6 n  →  ESC [ 1 ; 1 R
                    // - Window size request:  ESC [ 18 t →  ESC [ 8 ; rows ; cols t
                    let chunk = &buf[..n];
                    // Some terminals split these query bytes across reads. Use a small
                    // rolling carry buffer to detect queries across chunk boundaries.
                    let mut probe = carry.clone();
                    probe.extend_from_slice(chunk);

                    if probe.windows(4).any(|w| w == b"\x1b[6n") {
                        if let Ok(mut w) = writer_for_thread.lock() {
                            let _ = w.write_all(b"\x1b[1;1R");
                            let _ = w.flush();
                        }
                    }
                    if probe.windows(5).any(|w| w == b"\x1b[18t") {
                        if let Ok(mut w) = writer_for_thread.lock() {
                            let _ = w.write_all(b"\x1b[8;24;80t");
                            let _ = w.flush();
                        }
                    }

                    carry.clear();
                    let keep = 8usize;
                    if probe.len() > keep {
                        carry.extend_from_slice(&probe[probe.len() - keep..]);
                    } else {
                        carry.extend_from_slice(&probe);
                    }

                    if let Ok(mut guard) = output_for_thread.lock() {
                        guard.extend_from_slice(chunk);
                    }
                    continue;
                }

                std::thread::sleep(Duration::from_millis(25));
            }
        });

        Self {
            child,
            master: Some(master),
            waited: None,
            writer,
            output,
            reader_handle: Some(reader_handle),
            stop_reader,
        }
    }

    fn send_line(&mut self, line: &str) {
        if let Ok(mut w) = self.writer.lock() {
            let _ = w.write_all(line.as_bytes());
            // Use LF for Enter so both the stdio prompt worker (read_line) and
            // Reedline-backed prompt worker reliably consume the line under PTY
            // harnesses in CI.
            let _ = w.write_all(b"\n");
            let _ = w.flush();
        }
    }

    fn output_string(&self) -> String {
        let guard = self.output.lock().expect("lock output");
        String::from_utf8_lossy(&guard).into_owned()
    }

    fn wait_for_output(&self, needle: &str, timeout: Duration) -> anyhow::Result<()> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            let out = self.output_string();
            if out.contains(needle) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        anyhow::bail!(
            "timed out waiting for output containing `{}`; output so far:\n{}",
            needle,
            self.output_string()
        )
    }

    fn wait_for_prompt(&self, timeout: Duration) -> anyhow::Result<()> {
        self.wait_for_output("substrate> ", timeout)
    }

    fn try_wait(&mut self) -> anyhow::Result<bool> {
        if self.waited.is_some() {
            return Ok(true);
        }
        if let Some(status) = self.child.try_wait()? {
            self.waited = Some(status);
            return Ok(true);
        }
        Ok(false)
    }

    fn shutdown_graceful(mut self, timeout: Duration) -> (i32, String) {
        let deadline = Instant::now() + timeout;
        while self.waited.is_none() && Instant::now() < deadline {
            match self.try_wait() {
                Ok(true) => break,
                Ok(false) => std::thread::sleep(Duration::from_millis(25)),
                Err(_) => break,
            }
        }
        self.shutdown()
    }

    fn shutdown(mut self) -> (i32, String) {
        self.stop_reader.store(true, Ordering::Relaxed);
        self.master.take();
        if self.waited.is_none() {
            let _ = self.child.kill();
            let status = self.child.wait().expect("wait child");
            self.waited = Some(status);
        }
        if let Some(handle) = self.reader_handle.take() {
            let _ = handle.join();
        }
        let code = self
            .waited
            .as_ref()
            .map(|s| s.exit_code() as i32)
            .unwrap_or(-1);
        (code, self.output_string())
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn launch_host_runtime_via_targeted_turn(repl: &mut PtyRepl, backend_id: &str) {
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("prompt before targeted host launch");
    repl.send_line(&format!("::{backend_id} start retained host runtime"));
    repl.wait_for_output(
        "shell-owned orchestrator session is ready via retained attached control ownership",
        Duration::from_secs(5),
    )
    .expect("runtime ready event");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("prompt after targeted host launch");
}

#[test]
#[serial]
fn c3_host_directive_is_gated_disabled_by_default() {
    let temp = temp_dir("substrate-c3-host-gate-off-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy(&substrate_home, true);

    let sock_temp = short_socket_dir("sub-c3ws-host-off-");
    let sock = sock_temp.path().join("world.sock");
    let _server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);

    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("prompt");

    repl.send_line(":host pwd");
    repl.wait_for_output("host escape", Duration::from_secs(2))
        .expect("host-escape gating message");
    repl.send_line("exit");

    let (_code, out) = repl.shutdown_graceful(Duration::from_secs(2));

    assert!(
        out.to_ascii_lowercase().contains("host escape"),
        "expected a host-escape gating error, got output:\n{out}"
    );
}

#[test]
#[serial]
fn c3_host_directive_executes_on_host_when_enabled() {
    let temp = temp_dir("substrate-c3-host-gate-on-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy(&substrate_home, true);

    let sock_temp = short_socket_dir("sub-c3ws-host-on-");
    let sock = sock_temp.path().join("world.sock");
    let _server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);

    let mut repl = PtyRepl::spawn(
        &project,
        &home,
        &substrate_home,
        &sock,
        &[("SUBSTRATE_REPL_HOST_ESCAPE", "1")],
        &["--world"],
    );

    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("prompt");

    let project = fs::canonicalize(&project).unwrap_or(project);
    let project_str = project.to_string_lossy().into_owned();
    repl.send_line(":host pwd");
    repl.wait_for_output(project_str.as_ref(), Duration::from_secs(2))
        .expect("host pwd output");
    repl.send_line("exit");

    let (_code, out) = repl.shutdown_graceful(Duration::from_secs(2));
    assert!(
        out.contains(project_str.as_str()),
        "expected :host pwd to print the host cwd ({project_str}), got output:\n{out}"
    );
}

#[test]
#[serial]
fn c3_pty_directive_routes_to_persistent_session_when_world_enabled() {
    let temp = temp_dir("substrate-c3-pty-world-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy(&substrate_home, true);

    let sock_temp = short_socket_dir("sub-c3ws-pty-world-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);

    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");

    repl.send_line(":pty echo hello");
    repl.send_line("exit");

    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(2));

    let guard = records.lock().expect("lock records");
    assert!(
        guard
            .persistent_execs
            .iter()
            .any(|rec| rec.stdin_mode == "passthrough" && rec.program_utf8.trim() == "echo hello"),
        "expected :pty to use persistent-session exec passthrough with stripped prefix; records: {guard:#?}"
    );
}

#[test]
#[serial]
fn c3_persistent_session_start_carries_canonical_net_allowed_snapshot() {
    let temp = temp_dir("substrate-c3-net-allowed-pty-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy_with_net_allowed(
        &substrate_home,
        true,
        "[\" Example.COM. \", \"example.com\", \"Api.Example.com.\"]",
    );
    write_config(&substrate_home, true);

    let sock_temp = short_socket_dir("sub-c3ws-net-allowed-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);

    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.send_line("echo hello");
    repl.wait_for_output("hello", Duration::from_secs(3))
        .expect("command output");
    repl.send_line("exit");

    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(2));

    let guard = records.lock().expect("lock records");
    let start = guard
        .persistent_start_sessions
        .first()
        .expect("persistent start session");
    let net_allowed = start
        .policy_snapshot
        .get("net_allowed")
        .and_then(|value| value.as_array())
        .expect("policy_snapshot.net_allowed array");
    let net_allowed: Vec<&str> = net_allowed
        .iter()
        .map(|value| value.as_str().expect("net_allowed string"))
        .collect();
    assert_eq!(net_allowed, vec!["example.com", "api.example.com"]);
    assert_world_network_payload(start, true, &["example.com", "api.example.com"]);
}

#[test]
#[serial]
fn c3_persistent_session_start_obeys_net_allowed_routing_matrix() {
    let cases = [
        ReplRoutingCase {
            name: "gate off plus restrictive policy stays allow-all at routing layer",
            net_allowed_yaml: "[\" Example.COM. \", \"example.com\", \"Api.Example.com.\"]",
            world_net_filter: false,
            expected_net_allowed: &["example.com", "api.example.com"],
            expected_isolate_network: false,
            expected_allowed_domains: &[],
        },
        ReplRoutingCase {
            name: "gate on plus allow-all singleton does not request isolation",
            net_allowed_yaml: "[\" * \"]",
            world_net_filter: true,
            expected_net_allowed: &["*"],
            expected_isolate_network: false,
            expected_allowed_domains: &[],
        },
        ReplRoutingCase {
            name: "gate on plus deny-all requests isolation with empty allowlist",
            net_allowed_yaml: "[]",
            world_net_filter: true,
            expected_net_allowed: &[],
            expected_isolate_network: true,
            expected_allowed_domains: &[],
        },
        ReplRoutingCase {
            name: "gate on plus restrictive allowlist requests isolation with canonical domains",
            net_allowed_yaml: "[\" Example.COM. \", \"example.com\", \"Api.Example.com.\"]",
            world_net_filter: true,
            expected_net_allowed: &["example.com", "api.example.com"],
            expected_isolate_network: true,
            expected_allowed_domains: &["example.com", "api.example.com"],
        },
    ];

    for case in &cases {
        run_repl_routing_case(case);
    }
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn c3_first_start_shared_world_attach_create_is_owner_bound() {
    let temp = temp_dir("substrate-c3-world-owner-bound-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    let fake_codex = write_fake_codex_script(temp.path());
    write_orchestrator_runtime_world_config(&substrate_home, &fake_codex, "auto_restart");

    let sock_temp = short_socket_dir("sub-c3ws-owner-bound-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);

    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    launch_host_runtime_via_targeted_turn(&mut repl, "cli:codex");
    repl.send_line("exit");

    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(2));
    let orchestration_session_id = load_single_orchestration_session_id(&substrate_home);

    let guard = records.lock().expect("lock records");
    assert_eq!(
        guard.persistent_start_sessions.len(),
        1,
        "lazy host launch must reuse the existing REPL-owned world session instead of creating a second shared-world start: {guard:#?}"
    );
    let session = read_orchestration_session(&orchestration_session_path(
        &substrate_home,
        &orchestration_session_id,
    ));
    assert_eq!(
        session
            .get("orchestration_session_id")
            .and_then(Value::as_str),
        Some(orchestration_session_id.as_str())
    );
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn c3_host_orchestrator_remains_dormant_until_first_targeted_turn() {
    let temp = temp_dir("substrate-c3-host-dormant-until-target-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    let (fake_codex, invocation_count_path) =
        write_fake_codex_script_with_invocation_log_and_output(temp.path());
    write_orchestrator_runtime_world_config(&substrate_home, &fake_codex, "auto_restart");

    let sock_temp = short_socket_dir("sub-c3ws-host-dormant-target-");
    let sock = sock_temp.path().join("world.sock");
    let _server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);

    let mut repl = PtyRepl::spawn(
        &project,
        &home,
        &substrate_home,
        &sock,
        &[],
        &["--no-world"],
    );
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    std::thread::sleep(Duration::from_millis(150));

    let out_before_target = repl.output_string();
    assert_eq!(
        read_invocation_count(&invocation_count_path),
        0,
        "orchestrator backend must not be invoked before the first explicit targeted turn; output:\n{out_before_target}"
    );

    repl.send_line("::cli:codex launch on demand");
    repl.wait_for_output(
        "shell-owned orchestrator session is ready via retained attached control ownership",
        Duration::from_secs(3),
    )
    .expect("runtime ready after explicit targeted turn");
    assert_eq!(
        read_invocation_count(&invocation_count_path),
        1,
        "first explicit targeted turn should launch the retained host runtime exactly once"
    );

    repl.send_line("exit");
    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
#[serial]
fn c3_first_targeted_world_turn_uses_initial_prompt_in_member_dispatch() {
    let temp = temp_dir("substrate-c3-targeted-world-initial-prompt-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    let fake_orchestrator = write_fake_claude_script(temp.path());
    let fake_member = write_fake_codex_script(temp.path());
    write_orchestrator_and_world_member_runtime_world_config(
        &substrate_home,
        &fake_orchestrator,
        &fake_member,
        "auto_restart",
    );

    let sock_temp = short_socket_dir("sub-c3ws-targeted-world-initial-prompt-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start_with_member_dispatch_scripts(
        &sock,
        StreamBehavior::Normal,
        vec![MemberDispatchStreamScript::ReadyAndHoldUntilCancel {
            session_handle_id: "session-targeted-world-first-turn".to_string(),
            exit_code_on_cancel: 130,
        }],
    );
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");

    repl.send_line("::cli:claude_code start host runtime");
    repl.wait_for_output(
        "shell-owned orchestrator session is ready via retained attached control ownership",
        Duration::from_secs(3),
    )
    .expect("host runtime ready");

    repl.send_line("::cli:codex member targeted first turn");
    wait_for_min_member_dispatch_requests(&records, 1, Duration::from_secs(3));
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("prompt after first targeted world turn");
    std::thread::sleep(Duration::from_millis(100));

    let guard = records.lock().expect("lock records");
    assert_eq!(
        guard.member_turn_submit_requests.len(),
        0,
        "first targeted world turn must ride launch-time initial_prompt instead of the typed submit route: {guard:#?}"
    );
    let dispatch = guard
        .member_dispatch_requests
        .first()
        .and_then(|request| request.member_dispatch.as_ref())
        .expect("captured member dispatch request");
    assert_eq!(
        dispatch.initial_prompt.as_deref(),
        Some("member targeted first turn")
    );
    drop(guard);

    repl.send_line("exit");
    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));
    wait_for_min_execute_cancel_requests(&records, 1, Duration::from_secs(3));

    let guard = records.lock().expect("lock records");
    assert_eq!(
        guard.execute_cancel_requests.len(),
        1,
        "member launch shutdown must route retained member cancel through /v1/execute/cancel: {guard:#?}"
    );
    assert!(
        guard.execute_cancel_requests[0]
            .span_id
            .starts_with("member-span-"),
        "member shutdown cancel must target the guest member_dispatch span: {guard:#?}"
    );
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
#[serial]
fn c3_first_world_backed_command_lazily_launches_member_runtime() {
    let temp = temp_dir("substrate-c3-first-world-command-lazy-member-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    let fake_orchestrator = write_fake_claude_script(temp.path());
    let fake_member = write_fake_codex_script(temp.path());
    write_orchestrator_and_world_member_runtime_world_config(
        &substrate_home,
        &fake_orchestrator,
        &fake_member,
        "auto_restart",
    );

    let sock_temp = short_socket_dir("sub-c3ws-lazy-member-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start_with_member_dispatch_scripts(
        &sock,
        StreamBehavior::Normal,
        vec![MemberDispatchStreamScript::ReadyAndHoldUntilCancel {
            session_handle_id: "session-lazy-member".to_string(),
            exit_code_on_cancel: 130,
        }],
    );
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    launch_host_runtime_via_targeted_turn(&mut repl, "cli:claude_code");

    let orchestration_session_id = load_single_orchestration_session_id(&substrate_home);
    let initial_session = read_orchestration_session(&orchestration_session_path(
        &substrate_home,
        &orchestration_session_id,
    ));
    assert_world_member_absent_for_interval(
        &substrate_home,
        &orchestration_session_id,
        Duration::from_millis(250),
    );

    repl.send_line("echo first");
    wait_for_min_records(&records, 1, 1, Duration::from_secs(3));
    repl.wait_for_output("first", Duration::from_secs(3))
        .expect("first command output");

    let live_members = wait_for_live_world_member_count(
        &substrate_home,
        &orchestration_session_id,
        1,
        Duration::from_secs(5),
    );
    let member = &live_members[0];
    assert_eq!(
        member.get("agent_id").and_then(Value::as_str),
        Some("codex")
    );
    assert!(
        matches!(
            member.get("state").and_then(Value::as_str),
            Some("ready" | "running")
        ),
        "lazy member launch must become authoritative-live: {member:?}"
    );
    assert_eq!(
        member.get("world_generation").and_then(Value::as_u64),
        Some(0)
    );
    assert_eq!(
        member.get("world_id").and_then(Value::as_str),
        initial_session.get("world_id").and_then(Value::as_str),
        "lazy member launch must bind to the current authoritative world"
    );
    assert_eq!(
        member
            .get("orchestrator_participant_id")
            .and_then(Value::as_str),
        initial_session
            .get("active_session_handle_id")
            .and_then(Value::as_str),
        "lazy member launch must retain the live orchestrator seam"
    );
    assert!(
        member
            .get("resumed_from_participant_id")
            .is_none_or(Value::is_null),
        "first member launch must not claim replacement lineage: {member:?}"
    );

    repl.send_line("exit");
    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn c3_targeted_turn_requires_exact_double_colon_grammar_before_shell_fallback() {
    let temp = temp_dir("substrate-c3-targeted-turn-grammar-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    let fake_orchestrator = write_fake_claude_script(temp.path());
    let fake_member = write_fake_codex_script(temp.path());
    write_orchestrator_and_world_member_runtime_world_config(
        &substrate_home,
        &fake_orchestrator,
        &fake_member,
        "auto_restart",
    );

    let sock_temp = short_socket_dir("sub-c3ws-targeted-turn-grammar-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");

    repl.send_line("::cli:codex");
    repl.wait_for_output(
        "substrate: error: targeted follow-up turns require exact syntax '::<backend_id> <prompt>' on a single line",
        Duration::from_secs(3),
    )
    .expect("malformed syntax rejection");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("prompt after malformed syntax");
    std::thread::sleep(Duration::from_millis(100));

    let guard = records.lock().expect("lock records");
    assert!(
        guard.persistent_execs.is_empty(),
        "malformed targeted syntax must fail before shell fallback: {guard:#?}"
    );
    assert!(
        guard.member_dispatch_requests.is_empty(),
        "malformed targeted syntax must not lazy-launch a member runtime: {guard:#?}"
    );
    assert!(
        guard.member_turn_submit_requests.is_empty(),
        "malformed targeted syntax must not hit the typed submit route: {guard:#?}"
    );
    drop(guard);

    repl.send_line("exit");
    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
#[serial]
fn c3_targeted_world_turn_uses_typed_submit_route_without_relaunching_member() {
    let temp = temp_dir("substrate-c3-targeted-world-submit-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    let fake_orchestrator = write_fake_claude_script(temp.path());
    let fake_member = write_fake_codex_script(temp.path());
    write_orchestrator_and_world_member_runtime_world_config(
        &substrate_home,
        &fake_orchestrator,
        &fake_member,
        "auto_restart",
    );

    let sock_temp = short_socket_dir("sub-c3ws-targeted-world-submit-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start_with_member_dispatch_scripts(
        &sock,
        StreamBehavior::Normal,
        vec![MemberDispatchStreamScript::ReadyAndHoldUntilCancel {
            session_handle_id: "session-targeted-world".to_string(),
            exit_code_on_cancel: 130,
        }],
    );
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    launch_host_runtime_via_targeted_turn(&mut repl, "cli:claude_code");

    let orchestration_session_id = load_single_orchestration_session_id(&substrate_home);

    repl.send_line("echo first");
    wait_for_min_records(&records, 1, 1, Duration::from_secs(3));
    wait_for_min_member_dispatch_requests(&records, 1, Duration::from_secs(3));
    repl.wait_for_output("first", Duration::from_secs(3))
        .expect("first command output");

    let orchestration_session = read_orchestration_session(&orchestration_session_path(
        &substrate_home,
        &orchestration_session_id,
    ));
    let live_participants = authoritative_live_participant_manifests_for_session(
        &substrate_home,
        &orchestration_session_id,
    );
    assert_eq!(
        live_participants.len(),
        2,
        "first world-backed command must leave both the host orchestrator and world member authoritative-live: {live_participants:?}"
    );
    assert_eq!(
        live_participants
            .iter()
            .map(|manifest| manifest.get("backend_id").and_then(Value::as_str))
            .collect::<Vec<_>>(),
        vec![Some("cli:claude_code"), Some("cli:codex")],
        "first world-backed command must establish authoritative-live coexistence for exactly cli:claude_code and cli:codex"
    );
    let orchestrator =
        authoritative_live_participant_manifest_for_backend(&live_participants, "cli:claude_code");
    let orchestrator_participant_id = orchestrator
        .get("participant_id")
        .and_then(Value::as_str)
        .expect("orchestrator participant_id")
        .to_string();
    assert_eq!(
        orchestrator.get("role").and_then(Value::as_str),
        Some("orchestrator")
    );
    assert_eq!(
        orchestrator
            .pointer("/execution/scope")
            .and_then(Value::as_str),
        Some("host")
    );
    assert_eq!(
        Some(orchestrator_participant_id.as_str()),
        orchestration_session
            .get("active_session_handle_id")
            .and_then(Value::as_str),
        "the active orchestration seam must remain owned by cli:claude_code after cli:codex becomes live"
    );

    let live_members = wait_for_live_world_member_count(
        &substrate_home,
        &orchestration_session_id,
        1,
        Duration::from_secs(5),
    );
    let member = &live_members[0];
    let member_participant_id = member
        .get("participant_id")
        .and_then(Value::as_str)
        .expect("member participant_id")
        .to_string();
    let member_orchestrator_participant_id = member
        .get("orchestrator_participant_id")
        .and_then(Value::as_str)
        .expect("member orchestrator_participant_id")
        .to_string();
    assert_eq!(
        member_orchestrator_participant_id,
        orchestrator_participant_id,
        "the retained cli:codex world member must stay linked to the authoritative cli:claude_code orchestrator participant"
    );
    let world_id = member
        .get("world_id")
        .and_then(Value::as_str)
        .expect("member world_id")
        .to_string();
    let world_generation = member
        .get("world_generation")
        .and_then(Value::as_u64)
        .expect("member world_generation");

    repl.send_line("::cli:codex second");
    wait_for_min_member_turn_submit_requests(&records, 1, Duration::from_secs(3));
    repl.wait_for_output("__MEMBER_TURN_SUBMIT_STUB__ second", Duration::from_secs(3))
        .expect("typed submit route output");

    let guard = records.lock().expect("lock records");
    assert_eq!(
        guard.member_dispatch_requests.len(),
        1,
        "targeted follow-up turn must reuse the retained member instead of relaunching it: {guard:#?}"
    );
    let submit = guard
        .member_turn_submit_requests
        .first()
        .expect("member turn submit request");
    assert_eq!(submit.orchestration_session_id, orchestration_session_id);
    assert_eq!(submit.participant_id, member_participant_id);
    assert_eq!(
        submit.orchestrator_participant_id,
        member_orchestrator_participant_id
    );
    assert_eq!(submit.backend_id, "cli:codex");
    assert_eq!(submit.world_id, world_id);
    assert_eq!(submit.world_generation, world_generation);
    assert_eq!(submit.prompt, "second");
    drop(guard);

    let live_members_after = wait_for_live_world_member_count(
        &substrate_home,
        &orchestration_session_id,
        1,
        Duration::from_secs(3),
    );
    assert_eq!(
        live_members_after[0]
            .get("participant_id")
            .and_then(Value::as_str),
        Some(member_participant_id.as_str()),
        "targeted submit must keep one retained world member rather than swap or duplicate it"
    );
    let live_participants_after = authoritative_live_participant_manifests_for_session(
        &substrate_home,
        &orchestration_session_id,
    );
    assert_eq!(
        live_participants_after.len(),
        2,
        "targeted follow-up turns must keep the same two authoritative-live backends after coexistence is established: {live_participants_after:?}"
    );
    assert_eq!(
        live_participants_after
            .iter()
            .map(|manifest| manifest.get("backend_id").and_then(Value::as_str))
            .collect::<Vec<_>>(),
        vec![Some("cli:claude_code"), Some("cli:codex")],
        "targeted follow-up turns must preserve authoritative-live coexistence for exactly cli:claude_code and cli:codex"
    );
    let orchestrator_after = authoritative_live_participant_manifest_for_backend(
        &live_participants_after,
        "cli:claude_code",
    );
    assert_eq!(
        orchestrator_after
            .get("participant_id")
            .and_then(Value::as_str),
        Some(orchestrator_participant_id.as_str()),
        "cli:claude_code targeted coexistence must reuse the original orchestrator participant"
    );
    let member_after =
        authoritative_live_participant_manifest_for_backend(&live_participants_after, "cli:codex");
    assert_eq!(
        member_after.get("participant_id").and_then(Value::as_str),
        Some(member_participant_id.as_str()),
        "cli:codex targeted coexistence must reuse the original world member participant"
    );

    repl.send_line("exit");
    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn c3_targeted_world_turn_relaunches_exact_backend_after_world_restart() {
    let temp = temp_dir("substrate-c3-targeted-world-relaunch-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    let trace_path = home.join(".substrate/trace.jsonl");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(&trace_path, "").expect("seed trace");
    write_profile(&project);
    let fake_orchestrator = write_fake_claude_script(temp.path());
    let fake_member = write_fake_codex_script(temp.path());
    write_orchestrator_and_world_member_runtime_world_config(
        &substrate_home,
        &fake_orchestrator,
        &fake_member,
        "auto_restart",
    );

    let sock_temp = short_socket_dir("sub-c3ws-targeted-world-relaunch-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start_with_member_dispatch_scripts(
        &sock,
        StreamBehavior::Normal,
        vec![
            MemberDispatchStreamScript::ReadyAndHoldUntilCancel {
                session_handle_id: "session-first-member".to_string(),
                exit_code_on_cancel: 130,
            },
            MemberDispatchStreamScript::ReadyAndHoldUntilCancel {
                session_handle_id: "session-replacement-member".to_string(),
                exit_code_on_cancel: 130,
            },
        ],
    );
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    launch_host_runtime_via_targeted_turn(&mut repl, "cli:claude_code");
    let orchestration_session_id = load_single_orchestration_session_id(&substrate_home);
    let session_path = orchestration_session_path(&substrate_home, &orchestration_session_id);

    repl.send_line("echo first");
    wait_for_min_records(&records, 1, 1, Duration::from_secs(3));
    repl.wait_for_output("first", Duration::from_secs(3))
        .expect("first command output");
    let first_live_members = wait_for_live_world_member_count(
        &substrate_home,
        &orchestration_session_id,
        1,
        Duration::from_secs(5),
    );
    let first_member = &first_live_members[0];
    let first_member_id = first_member
        .get("participant_id")
        .and_then(Value::as_str)
        .expect("first member participant_id")
        .to_string();
    let orchestrator_participant_id = first_member
        .get("orchestrator_participant_id")
        .and_then(Value::as_str)
        .expect("member orchestrator_participant_id")
        .to_string();

    write_member_runtime_policy(&substrate_home, false);
    std::thread::sleep(Duration::from_millis(25));

    repl.send_line("::cli:codex second");
    let alert = wait_for_world_restarted_alert_without_stale_liveness(
        &trace_path,
        &substrate_home,
        &orchestration_session_id,
        0,
        Duration::from_secs(15),
    );
    wait_for_min_member_dispatch_requests(&records, 2, Duration::from_secs(3));
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("prompt after replacement launch turn");

    let replacement_live_members = wait_for_live_world_member_count(
        &substrate_home,
        &orchestration_session_id,
        1,
        Duration::from_secs(5),
    );
    let replacement = &replacement_live_members[0];
    let replacement_id = replacement
        .get("participant_id")
        .and_then(Value::as_str)
        .expect("replacement participant_id")
        .to_string();
    assert_ne!(
        replacement_id, first_member_id,
        "targeted follow-up after restart must relaunch a distinct exact-backend member runtime"
    );
    assert_eq!(
        replacement.get("world_generation").and_then(Value::as_u64),
        Some(1),
        "replacement member must bind to the new world generation"
    );
    assert_eq!(
        replacement.get("world_id").and_then(Value::as_str),
        alert.get("world_id").and_then(Value::as_str),
        "replacement member must bind to the replacement world id"
    );
    assert_eq!(
        replacement
            .get("orchestrator_participant_id")
            .and_then(Value::as_str),
        Some(orchestrator_participant_id.as_str()),
        "replacement member must preserve the retained-control seam"
    );
    assert_eq!(
        replacement
            .get("resumed_from_participant_id")
            .and_then(Value::as_str),
        Some(first_member_id.as_str()),
        "replacement member must preserve lineage to the stale exact-backend runtime"
    );

    let guard = records.lock().expect("lock records");
    assert_eq!(
        guard.persistent_execs.len(),
        1,
        "targeted world follow-up must not fall back to a second shell/persistent exec after restart: {guard:#?}"
    );
    assert_eq!(
        guard.member_dispatch_requests.len(),
        2,
        "targeted world follow-up after restart must relaunch the exact backend slot once: {guard:#?}"
    );
    assert!(
        guard.member_turn_submit_requests.is_empty(),
        "replacement targeted turn should ride launch-time initial_prompt instead of a second typed submit: {guard:#?}"
    );
    let replacement_dispatch = guard
        .member_dispatch_requests
        .get(1)
        .and_then(|request| request.member_dispatch.as_ref())
        .expect("replacement member dispatch request");
    assert_eq!(
        replacement_dispatch.initial_prompt.as_deref(),
        Some("second")
    );
    drop(guard);

    let stale = read_participant_manifest(&substrate_home, &first_member_id);
    assert_eq!(
        stale.get("state").and_then(Value::as_str),
        Some("invalidated"),
        "stale generation member must remain invalidated after targeted relaunch submit: {stale:?}"
    );
    let persisted = read_orchestration_session(&session_path);
    assert_session_world_binding(&persisted, Some("wld_stub_0002"), Some(1));

    repl.send_line("exit");
    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn c3_targeted_host_turn_resumes_active_orchestrator_backend() {
    let temp = temp_dir("substrate-c3-targeted-host-active-resume-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    let fake_orchestrator = write_fake_claude_script_first_session_then_exit(temp.path());
    let fake_secondary_host = write_fake_codex_script(temp.path());
    write_dual_host_runtime_world_config(
        &substrate_home,
        &fake_orchestrator,
        &fake_secondary_host,
        "auto_restart",
    );

    let sock_temp = short_socket_dir("sub-c3ws-targeted-host-active-resume-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    launch_host_runtime_via_targeted_turn(&mut repl, "cli:claude_code");

    repl.send_line("::cli:claude_code resume active host");
    repl.wait_for_output(
        "submitted targeted follow-up turn to cli:claude_code",
        Duration::from_secs(3),
    )
    .expect("targeted host submit started");
    repl.wait_for_output(
        "targeted follow-up turn completed for cli:claude_code",
        Duration::from_secs(3),
    )
    .expect("targeted host submit completion");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("prompt after targeted host submit");

    let guard = records.lock().expect("lock records");
    assert!(
        guard.member_dispatch_requests.is_empty(),
        "targeted host follow-up must not launch a world member runtime: {guard:#?}"
    );
    assert!(
        guard.member_turn_submit_requests.is_empty(),
        "targeted host follow-up must not hit the typed world submit route: {guard:#?}"
    );
    drop(guard);

    repl.send_line("exit");
    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
#[serial]
fn c3_targeted_host_turn_rejects_non_active_orchestrator_backend() {
    let temp = temp_dir("substrate-c3-targeted-host-rejects-nonactive-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    let fake_orchestrator = write_fake_claude_script(temp.path());
    let fake_secondary_host = write_fake_codex_script(temp.path());
    write_dual_host_runtime_world_config(
        &substrate_home,
        &fake_orchestrator,
        &fake_secondary_host,
        "auto_restart",
    );

    let sock_temp = short_socket_dir("sub-c3ws-targeted-host-rejects-nonactive-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    launch_host_runtime_via_targeted_turn(&mut repl, "cli:claude_code");

    repl.send_line("::cli:codex should fail");
    repl.wait_for_output(
        "substrate: error: targeted host follow-up turns may only target the active orchestrator backend for this REPL session",
        Duration::from_secs(3),
    )
    .expect("non-active host backend rejection");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("prompt after non-active host backend rejection");
    std::thread::sleep(Duration::from_millis(100));

    let guard = records.lock().expect("lock records");
    assert!(
        guard.persistent_execs.is_empty(),
        "rejected host targeted turn must not fall back to shell execution: {guard:#?}"
    );
    assert!(
        guard.member_dispatch_requests.is_empty(),
        "rejected host targeted turn must not launch a world member runtime: {guard:#?}"
    );
    assert!(
        guard.member_turn_submit_requests.is_empty(),
        "rejected host targeted turn must not hit the typed world submit route: {guard:#?}"
    );
    drop(guard);

    repl.send_line("exit");
    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn c3_same_generation_world_command_reuses_live_member_runtime() {
    let temp = temp_dir("substrate-c3-same-generation-member-reuse-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    let fake_orchestrator = write_fake_claude_script(temp.path());
    let fake_member = write_fake_codex_script(temp.path());
    write_orchestrator_and_world_member_runtime_world_config(
        &substrate_home,
        &fake_orchestrator,
        &fake_member,
        "auto_restart",
    );

    let sock_temp = short_socket_dir("sub-c3ws-member-reuse-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start_with_member_dispatch_scripts(
        &sock,
        StreamBehavior::Normal,
        vec![
            MemberDispatchStreamScript::ReadyAndHoldUntilCancel {
                session_handle_id: "session-first-member".to_string(),
                exit_code_on_cancel: 130,
            },
            MemberDispatchStreamScript::ExitWithoutReady { exit_code: 1 },
        ],
    );
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    launch_host_runtime_via_targeted_turn(&mut repl, "cli:claude_code");
    let orchestration_session_id = load_single_orchestration_session_id(&substrate_home);
    repl.send_line("echo first");
    wait_for_min_records(&records, 1, 1, Duration::from_secs(3));
    repl.wait_for_output("first", Duration::from_secs(3))
        .expect("first command output");
    let first_live_members = wait_for_live_world_member_count(
        &substrate_home,
        &orchestration_session_id,
        1,
        Duration::from_secs(5),
    );
    let first_member_id = first_live_members[0]
        .get("participant_id")
        .and_then(Value::as_str)
        .expect("first member participant_id")
        .to_string();

    repl.send_line("echo second");
    wait_for_min_records(&records, 2, 1, Duration::from_secs(3));
    repl.wait_for_output("second", Duration::from_secs(3))
        .expect("second command output");
    std::thread::sleep(Duration::from_millis(100));

    let members = world_member_manifests_for_session(&substrate_home, &orchestration_session_id);
    assert_eq!(
        members.len(),
        1,
        "same-generation second command must reuse the existing member instead of creating a sibling: {members:?}"
    );
    let live_members = wait_for_live_world_member_count(
        &substrate_home,
        &orchestration_session_id,
        1,
        Duration::from_secs(2),
    );
    let member = &live_members[0];
    assert_eq!(
        member.get("participant_id").and_then(Value::as_str),
        Some(first_member_id.as_str()),
        "same-generation second command must reuse the live member"
    );
    assert_eq!(
        member.get("world_generation").and_then(Value::as_u64),
        Some(0)
    );

    repl.send_line("exit");
    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn c3_startup_drift_before_first_command_retains_persisted_startup_context() {
    let temp = temp_dir("substrate-c3-startup-drift-context-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let child = project.join("child");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&child).expect("create project child");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_workspace_marker(&project);
    write_profile(&project);
    let fake_codex = write_fake_codex_script(temp.path());
    write_orchestrator_runtime_world_config(&substrate_home, &fake_codex, "auto_restart");

    let parent_cwd = temp.path().to_string_lossy().into_owned();

    let sock_temp = short_socket_dir("sub-c3ws-startup-context-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start_with_first_ready_cwd_override(
        &sock,
        StreamBehavior::Normal,
        parent_cwd,
    );
    let records = server.records();

    let mut repl = PtyRepl::spawn(&child, &home, &substrate_home, &sock, &[], &["--world"]);

    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    launch_host_runtime_via_targeted_turn(&mut repl, "cli:codex");
    wait_for_min_start_sessions_with_output(&repl, &records, 2, Duration::from_secs(3));
    repl.send_line("exit");

    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));
    let guard = records.lock().expect("lock records");
    assert_eq!(
        guard.persistent_start_sessions.len(),
        2,
        "startup drift before the first command should only restart the REPL-owned world session once: {guard:#?}"
    );
    assert_start_sessions_have_no_shared_world_context(&guard, "workspace root drift");
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn c3_parent_binding_persists_before_world_restarted_publishes() {
    let temp = temp_dir("substrate-c3-world-restarted-binding-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    let trace_path = home.join(".substrate/trace.jsonl");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(&trace_path, "").expect("seed trace");
    write_profile(&project);
    let fake_codex = write_fake_codex_script(temp.path());
    write_orchestrator_runtime_world_config(&substrate_home, &fake_codex, "auto_restart");

    let sock_temp = short_socket_dir("sub-c3ws-world-restarted-binding-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);

    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    launch_host_runtime_via_targeted_turn(&mut repl, "cli:codex");
    let orchestration_session_id = load_single_orchestration_session_id(&substrate_home);
    let session_path = orchestration_session_path(&substrate_home, &orchestration_session_id);
    repl.send_line("echo first");
    wait_for_min_records(&records, 1, 1, Duration::from_secs(3));
    repl.wait_for_output("first", Duration::from_secs(3))
        .expect("first command output");

    write_policy(&substrate_home, false);
    std::thread::sleep(Duration::from_millis(25));

    repl.send_line("echo second");
    repl.wait_for_output(
        "[shell] world restarted due to policy snapshot drift",
        Duration::from_secs(5),
    )
    .expect("world_restarted alert output");

    let persisted = read_orchestration_session(&session_path);
    assert_session_world_binding(&persisted, Some("wld_stub_0002"), Some(1));

    repl.wait_for_output("second", Duration::from_secs(3))
        .expect("second command output");
    repl.send_line("exit");
    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));

    let events = read_trace(&trace_path);
    let alerts = alert_rows_by_code(&events, "world_restarted");
    assert_eq!(
        alerts.len(),
        1,
        "expected one world_restarted alert: {alerts:?}"
    );
    let alert = alerts[0];
    assert_eq!(
        alert
            .get("orchestration_session_id")
            .and_then(Value::as_str),
        Some(orchestration_session_id.as_str()),
        "world_restarted alert must retain the authoritative orchestration_session_id: {alert:?}"
    );
    assert_eq!(
        alert.get("world_id").and_then(Value::as_str),
        Some("wld_stub_0002"),
        "world_restarted alert must publish the replacement world binding: {alert:?}"
    );
    assert_eq!(
        alert.get("world_generation").and_then(Value::as_u64),
        Some(1),
        "world_restarted alert must publish the replacement world generation: {alert:?}"
    );
    assert_eq!(
        alert
            .pointer("/data/previous_world_id")
            .and_then(Value::as_str),
        Some("wld_stub_0001"),
        "world_restarted alert must retain the previous world_id: {alert:?}"
    );
    assert_eq!(
        alert.pointer("/data/new_world_id").and_then(Value::as_str),
        Some("wld_stub_0002"),
        "world_restarted alert must retain the replacement world_id: {alert:?}"
    );
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn c3_fail_closed_drift_repersists_binding_before_world_restart_required_publishes() {
    let temp = temp_dir("substrate-c3-world-restart-required-binding-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    let trace_path = home.join(".substrate/trace.jsonl");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(&trace_path, "").expect("seed trace");
    write_profile(&project);
    let fake_codex = write_fake_codex_script(temp.path());
    write_orchestrator_runtime_world_config(&substrate_home, &fake_codex, "fail_closed");

    let sock_temp = short_socket_dir("sub-c3ws-world-restart-required-binding-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);

    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    launch_host_runtime_via_targeted_turn(&mut repl, "cli:codex");
    let orchestration_session_id = load_single_orchestration_session_id(&substrate_home);
    let session_path = orchestration_session_path(&substrate_home, &orchestration_session_id);
    repl.send_line("echo first");
    wait_for_min_records(&records, 1, 1, Duration::from_secs(3));
    repl.wait_for_output("first", Duration::from_secs(3))
        .expect("first command output");

    write_policy(&substrate_home, false);
    std::thread::sleep(Duration::from_millis(25));

    repl.send_line("echo second");
    repl.wait_for_output(
        "[shell] world restart required due to policy snapshot drift",
        Duration::from_secs(5),
    )
    .expect("world_restart_required alert output");

    let persisted = read_orchestration_session(&session_path);
    assert_session_world_binding(&persisted, Some("wld_stub_0001"), Some(0));

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        if repl.try_wait().expect("try_wait") {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let (code, _out) = repl.shutdown();
    assert_eq!(code, 3, "expected fail-closed drift exit code 3");

    let events = read_trace(&trace_path);
    let alerts = alert_rows_by_code(&events, "world_restart_required");
    assert_eq!(
        alerts.len(),
        1,
        "expected one world_restart_required alert: {alerts:?}"
    );
    let alert = alerts[0];
    assert_eq!(
        alert.get("orchestration_session_id").and_then(Value::as_str),
        Some(orchestration_session_id.as_str()),
        "world_restart_required alert must retain the authoritative orchestration_session_id: {alert:?}"
    );
    assert_eq!(
        alert.get("world_id").and_then(Value::as_str),
        Some("wld_stub_0001"),
        "world_restart_required alert must publish the current authoritative world binding: {alert:?}"
    );
    assert_eq!(
        alert.get("world_generation").and_then(Value::as_u64),
        Some(0),
        "world_restart_required alert must publish the current authoritative world generation: {alert:?}"
    );
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn c3_world_restart_invalidates_stale_member_generation_before_publish() {
    let temp = temp_dir("substrate-c3-world-restart-invalidates-stale-members-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    let trace_path = home.join(".substrate/trace.jsonl");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(&trace_path, "").expect("seed trace");
    write_profile(&project);
    let fake_codex = write_fake_codex_script(temp.path());
    write_orchestrator_runtime_world_config(&substrate_home, &fake_codex, "auto_restart");

    let sock_temp = short_socket_dir("sub-c3ws-world-restart-invalidates-stale-members-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    launch_host_runtime_via_targeted_turn(&mut repl, "cli:codex");
    repl.send_line("echo first");
    wait_for_min_records(&records, 1, 1, Duration::from_secs(3));
    repl.wait_for_output("first", Duration::from_secs(3))
        .expect("first command output");

    let orchestration_session_id = load_single_orchestration_session_id(&substrate_home);
    let session_path = orchestration_session_path(&substrate_home, &orchestration_session_id);
    let initial_session = read_orchestration_session(&session_path);
    let world_id = initial_session
        .get("world_id")
        .and_then(Value::as_str)
        .expect("initial world_id")
        .to_string();
    let orchestrator_participant_id = initial_session
        .get("active_session_handle_id")
        .and_then(Value::as_str)
        .expect("active_session_handle_id")
        .to_string();

    let stale_member_ids = (0..256)
        .map(|idx| format!("ash_member_stale_{idx:03}"))
        .collect::<Vec<_>>();
    for participant_id in &stale_member_ids {
        write_live_world_member_manifest(
            &substrate_home,
            &orchestration_session_id,
            participant_id,
            &orchestrator_participant_id,
            &world_id,
            0,
        );
    }

    write_policy(&substrate_home, false);
    std::thread::sleep(Duration::from_millis(25));

    repl.send_line("echo second");
    let alert = wait_for_world_restarted_alert_without_stale_liveness(
        &trace_path,
        &substrate_home,
        &orchestration_session_id,
        0,
        Duration::from_secs(15),
    );
    assert_eq!(
        alert.get("world_generation").and_then(Value::as_u64),
        Some(1),
        "world_restarted alert must publish replacement generation: {alert:?}"
    );

    repl.wait_for_output("second", Duration::from_secs(3))
        .expect("second command output");
    repl.send_line("exit");
    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));

    let persisted = read_orchestration_session(&session_path);
    assert_session_world_binding(&persisted, Some("wld_stub_0002"), Some(1));
    assert!(
        live_world_member_generations_for_session(&substrate_home, &orchestration_session_id)
            .is_empty(),
        "stale generation members must not remain authoritative-live after restart"
    );
    for participant_id in &stale_member_ids {
        let manifest = read_participant_manifest(&substrate_home, participant_id);
        assert_eq!(
            manifest.get("state").and_then(Value::as_str),
            Some("invalidated"),
            "stale member must be invalidated: {manifest:?}"
        );
        assert_eq!(
            manifest
                .pointer("/internal/termination_reason")
                .and_then(Value::as_str),
            Some("world generation invalidated by replacement binding"),
            "stale member invalidation reason must explain generation rollover: {manifest:?}"
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn c3_world_restart_launches_live_member_replacement_on_new_generation() {
    let temp = temp_dir("substrate-c3-world-restart-live-member-replacement-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    let trace_path = home.join(".substrate/trace.jsonl");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(&trace_path, "").expect("seed trace");
    write_profile(&project);
    let fake_orchestrator = write_fake_claude_script(temp.path());
    let fake_member = write_fake_codex_script(temp.path());
    write_orchestrator_and_world_member_runtime_world_config(
        &substrate_home,
        &fake_orchestrator,
        &fake_member,
        "auto_restart",
    );

    let sock_temp = short_socket_dir("sub-c3ws-live-member-replacement-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start_with_member_dispatch_scripts(
        &sock,
        StreamBehavior::Normal,
        vec![
            MemberDispatchStreamScript::ReadyAndHoldUntilCancel {
                session_handle_id: "session-first-member".to_string(),
                exit_code_on_cancel: 130,
            },
            MemberDispatchStreamScript::ReadyAndHoldUntilCancel {
                session_handle_id: "session-replacement-member".to_string(),
                exit_code_on_cancel: 130,
            },
        ],
    );
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    launch_host_runtime_via_targeted_turn(&mut repl, "cli:claude_code");
    let orchestration_session_id = load_single_orchestration_session_id(&substrate_home);
    let session_path = orchestration_session_path(&substrate_home, &orchestration_session_id);

    repl.send_line("echo first");
    wait_for_min_records(&records, 1, 1, Duration::from_secs(3));
    repl.wait_for_output("first", Duration::from_secs(3))
        .expect("first command output");
    let first_live_members = wait_for_live_world_member_count(
        &substrate_home,
        &orchestration_session_id,
        1,
        Duration::from_secs(5),
    );
    let first_member = &first_live_members[0];
    let first_member_id = first_member
        .get("participant_id")
        .and_then(Value::as_str)
        .expect("first member participant_id")
        .to_string();
    let orchestrator_participant_id = first_member
        .get("orchestrator_participant_id")
        .and_then(Value::as_str)
        .expect("member orchestrator_participant_id")
        .to_string();

    write_member_runtime_policy(&substrate_home, false);
    std::thread::sleep(Duration::from_millis(25));

    repl.send_line("echo second");
    let alert = wait_for_world_restarted_alert_without_stale_liveness(
        &trace_path,
        &substrate_home,
        &orchestration_session_id,
        0,
        Duration::from_secs(15),
    );
    wait_for_min_records(&records, 2, 1, Duration::from_secs(3));
    repl.wait_for_output("second", Duration::from_secs(3))
        .expect("second command output");

    let replacement_live_members = wait_for_live_world_member_count(
        &substrate_home,
        &orchestration_session_id,
        1,
        Duration::from_secs(5),
    );
    let replacement = &replacement_live_members[0];
    let replacement_id = replacement
        .get("participant_id")
        .and_then(Value::as_str)
        .expect("replacement participant_id");
    assert_ne!(
        replacement_id, first_member_id,
        "restart must create a distinct replacement member runtime"
    );
    assert_eq!(
        replacement.get("world_generation").and_then(Value::as_u64),
        Some(1),
        "replacement member must bind to the new world generation"
    );
    assert_eq!(
        replacement.get("world_id").and_then(Value::as_str),
        alert.get("world_id").and_then(Value::as_str),
        "replacement member must bind to the replacement world id"
    );
    assert_eq!(
        replacement
            .get("orchestrator_participant_id")
            .and_then(Value::as_str),
        Some(orchestrator_participant_id.as_str()),
        "replacement member must preserve the retained-control seam"
    );
    assert_eq!(
        replacement
            .get("resumed_from_participant_id")
            .and_then(Value::as_str),
        Some(first_member_id.as_str()),
        "replacement member must retain explicit lineage to the previous generation"
    );

    let stale = read_participant_manifest(&substrate_home, &first_member_id);
    assert_eq!(
        stale.get("state").and_then(Value::as_str),
        Some("invalidated"),
        "previous generation member must be invalidated after replacement launch: {stale:?}"
    );

    let persisted = read_orchestration_session(&session_path);
    assert_session_world_binding(&persisted, Some("wld_stub_0002"), Some(1));

    repl.send_line("exit");
    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn c3_world_restart_failed_member_replacement_leaves_honest_absence() {
    let temp = temp_dir("substrate-c3-world-restart-failed-member-replacement-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    let trace_path = home.join(".substrate/trace.jsonl");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(&trace_path, "").expect("seed trace");
    write_profile(&project);
    let fake_orchestrator = write_fake_claude_script(temp.path());
    let fake_member =
        write_fake_codex_script_first_success_then_fail_without_session_handle(temp.path());
    write_orchestrator_and_world_member_runtime_world_config(
        &substrate_home,
        &fake_orchestrator,
        &fake_member,
        "auto_restart",
    );

    let sock_temp = short_socket_dir("sub-c3ws-failed-member-replacement-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start_with_member_dispatch_scripts(
        &sock,
        StreamBehavior::Normal,
        vec![
            MemberDispatchStreamScript::ReadyAndHoldUntilCancel {
                session_handle_id: "session-first-member".to_string(),
                exit_code_on_cancel: 130,
            },
            MemberDispatchStreamScript::ExitWithoutReady { exit_code: 1 },
        ],
    );
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    launch_host_runtime_via_targeted_turn(&mut repl, "cli:claude_code");
    let orchestration_session_id = load_single_orchestration_session_id(&substrate_home);
    let session_path = orchestration_session_path(&substrate_home, &orchestration_session_id);

    repl.send_line("echo first");
    wait_for_min_records(&records, 1, 1, Duration::from_secs(3));
    repl.wait_for_output("first", Duration::from_secs(3))
        .expect("first command output");
    let first_live_members = wait_for_live_world_member_count(
        &substrate_home,
        &orchestration_session_id,
        1,
        Duration::from_secs(5),
    );
    let first_member_id = first_live_members[0]
        .get("participant_id")
        .and_then(Value::as_str)
        .expect("first member participant_id")
        .to_string();

    write_member_runtime_policy(&substrate_home, false);
    std::thread::sleep(Duration::from_millis(25));

    repl.send_line("echo second");
    let _alert = wait_for_world_restarted_alert_without_stale_liveness(
        &trace_path,
        &substrate_home,
        &orchestration_session_id,
        0,
        Duration::from_secs(15),
    );
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        if repl.try_wait().expect("try_wait") {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let (code, out) = repl.shutdown();
    assert_eq!(
        code, 1,
        "replacement startup failure must fail closed instead of continuing; output:\n{out}"
    );
    assert!(
        out.contains("world-scoped member runtime exited with status 1 before ownership could be established"),
        "replacement startup failure must surface the member bootstrap error; output:\n{out}"
    );
    assert!(
        !out.contains("__PERSISTENT_EXEC_STUB__ eof echo second"),
        "replacement startup failure must block the second command from executing; output:\n{out}"
    );

    let guard = records.lock().expect("lock records");
    assert_eq!(
        guard.persistent_execs.len(),
        1,
        "fail-closed member startup must not fall through to a host-local or restarted-world second exec; records: {guard:#?}"
    );
    assert!(
        guard.persistent_start_sessions.len() >= 2,
        "world restart should still allocate the replacement world before the member bootstrap failure; records: {guard:#?}"
    );
    drop(guard);

    let live_members = wait_for_live_world_member_count(
        &substrate_home,
        &orchestration_session_id,
        0,
        Duration::from_secs(5),
    );
    assert!(
        live_members.is_empty(),
        "replacement failure must leave no authoritative-live world member"
    );
    let stale = read_participant_manifest(&substrate_home, &first_member_id);
    assert_eq!(
        stale.get("state").and_then(Value::as_str),
        Some("invalidated"),
        "replacement failure must not resurrect the stale member: {stale:?}"
    );
    assert!(
        world_member_manifests_for_session(&substrate_home, &orchestration_session_id)
            .into_iter()
            .all(|manifest| !participant_is_authoritative_live(&manifest)),
        "replacement failure must leave honest absence rather than any authoritative-live member"
    );
    let persisted = read_orchestration_session(&session_path);
    assert_session_world_binding(&persisted, Some("wld_stub_0002"), Some(1));
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn c3_world_restart_missing_member_replacement_leaves_absence() {
    let temp = temp_dir("substrate-c3-world-restart-member-absence-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    let trace_path = home.join(".substrate/trace.jsonl");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(&trace_path, "").expect("seed trace");
    write_profile(&project);
    let fake_codex = write_fake_codex_script(temp.path());
    write_orchestrator_runtime_world_config(&substrate_home, &fake_codex, "auto_restart");

    let sock_temp = short_socket_dir("sub-c3ws-world-restart-member-absence-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    launch_host_runtime_via_targeted_turn(&mut repl, "cli:codex");
    repl.send_line("echo first");
    wait_for_min_records(&records, 1, 1, Duration::from_secs(3));
    repl.wait_for_output("first", Duration::from_secs(3))
        .expect("first command output");

    let orchestration_session_id = load_single_orchestration_session_id(&substrate_home);
    let session_path = orchestration_session_path(&substrate_home, &orchestration_session_id);
    let initial_session = read_orchestration_session(&session_path);
    write_live_world_member_manifest(
        &substrate_home,
        &orchestration_session_id,
        "ash_member_stale_only",
        initial_session
            .get("active_session_handle_id")
            .and_then(Value::as_str)
            .expect("active_session_handle_id"),
        initial_session
            .get("world_id")
            .and_then(Value::as_str)
            .expect("initial world_id"),
        0,
    );

    write_policy(&substrate_home, false);
    std::thread::sleep(Duration::from_millis(25));

    repl.send_line("echo second");
    let _alert = wait_for_world_restarted_alert_without_stale_liveness(
        &trace_path,
        &substrate_home,
        &orchestration_session_id,
        0,
        Duration::from_secs(5),
    );
    assert!(
        live_world_member_generations_for_session(&substrate_home, &orchestration_session_id)
            .is_empty(),
        "missing member replacement must leave absence rather than stale liveness"
    );

    repl.wait_for_output("second", Duration::from_secs(3))
        .expect("second command output");
    repl.send_line("exit");
    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));

    let stale = read_participant_manifest(&substrate_home, "ash_member_stale_only");
    assert_eq!(
        stale.get("state").and_then(Value::as_str),
        Some("invalidated"),
        "stale member must remain invalidated when no replacement is created: {stale:?}"
    );
    let persisted = read_orchestration_session(&session_path);
    assert_session_world_binding(&persisted, Some("wld_stub_0002"), Some(1));
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn c3_world_restart_replacement_generation_becomes_only_live_generation() {
    let temp = temp_dir("substrate-c3-world-restart-member-replacement-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    let trace_path = home.join(".substrate/trace.jsonl");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(&trace_path, "").expect("seed trace");
    write_profile(&project);
    let fake_codex = write_fake_codex_script(temp.path());
    write_orchestrator_runtime_world_config(&substrate_home, &fake_codex, "auto_restart");

    let sock_temp = short_socket_dir("sub-c3ws-world-restart-member-replacement-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    launch_host_runtime_via_targeted_turn(&mut repl, "cli:codex");
    repl.send_line("echo first");
    wait_for_min_records(&records, 1, 1, Duration::from_secs(3));
    repl.wait_for_output("first", Duration::from_secs(3))
        .expect("first command output");

    let orchestration_session_id = load_single_orchestration_session_id(&substrate_home);
    let session_path = orchestration_session_path(&substrate_home, &orchestration_session_id);
    let initial_session = read_orchestration_session(&session_path);
    let orchestrator_participant_id = initial_session
        .get("active_session_handle_id")
        .and_then(Value::as_str)
        .expect("active_session_handle_id")
        .to_string();
    write_live_world_member_manifest(
        &substrate_home,
        &orchestration_session_id,
        "ash_member_old_generation",
        &orchestrator_participant_id,
        initial_session
            .get("world_id")
            .and_then(Value::as_str)
            .expect("initial world_id"),
        0,
    );

    write_policy(&substrate_home, false);
    std::thread::sleep(Duration::from_millis(25));

    repl.send_line("echo second");
    let alert = wait_for_world_restarted_alert_without_stale_liveness(
        &trace_path,
        &substrate_home,
        &orchestration_session_id,
        0,
        Duration::from_secs(5),
    );
    let replacement_world_id = alert
        .get("world_id")
        .and_then(Value::as_str)
        .expect("replacement world_id")
        .to_string();
    let replacement_generation = alert
        .get("world_generation")
        .and_then(Value::as_u64)
        .expect("replacement world_generation");
    write_live_world_member_manifest(
        &substrate_home,
        &orchestration_session_id,
        "ash_member_new_generation",
        &orchestrator_participant_id,
        &replacement_world_id,
        replacement_generation,
    );
    assert_eq!(
        live_world_member_generations_for_session(&substrate_home, &orchestration_session_id),
        vec![("ash_member_new_generation".to_string(), 1)],
        "replacement generation must become the only authoritative-live world member generation"
    );

    repl.wait_for_output("second", Duration::from_secs(3))
        .expect("second command output");
    repl.send_line("exit");
    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));

    let stale = read_participant_manifest(&substrate_home, "ash_member_old_generation");
    assert_eq!(
        stale.get("state").and_then(Value::as_str),
        Some("invalidated"),
        "stale generation must not remain live after replacement is present: {stale:?}"
    );
    let persisted = read_orchestration_session(&session_path);
    assert_session_world_binding(
        &persisted,
        Some(replacement_world_id.as_str()),
        Some(replacement_generation),
    );
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn c3_world_restart_keeps_same_agent_members_in_other_sessions_isolated() {
    let temp = temp_dir("substrate-c3-world-restart-cross-session-isolation-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    let trace_path = home.join(".substrate/trace.jsonl");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(&trace_path, "").expect("seed trace");
    write_profile(&project);
    let fake_codex = write_fake_codex_script(temp.path());
    write_orchestrator_runtime_world_config(&substrate_home, &fake_codex, "auto_restart");

    let sock_temp = short_socket_dir("sub-c3ws-cross-session-isolation-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    launch_host_runtime_via_targeted_turn(&mut repl, "cli:codex");
    repl.send_line("echo first");
    wait_for_min_records(&records, 1, 1, Duration::from_secs(3));
    repl.wait_for_output("first", Duration::from_secs(3))
        .expect("first command output");

    let orchestration_session_id = load_single_orchestration_session_id(&substrate_home);
    let session_path = orchestration_session_path(&substrate_home, &orchestration_session_id);
    let initial_session = read_orchestration_session(&session_path);
    let orchestrator_participant_id = initial_session
        .get("active_session_handle_id")
        .and_then(Value::as_str)
        .expect("active_session_handle_id")
        .to_string();
    let initial_world_id = initial_session
        .get("world_id")
        .and_then(Value::as_str)
        .expect("initial world_id")
        .to_string();

    write_live_world_member_manifest(
        &substrate_home,
        &orchestration_session_id,
        "ash_member_current_session",
        &orchestrator_participant_id,
        &initial_world_id,
        0,
    );

    let other_orchestration_session_id = "orch_other_same_agent";
    write_live_world_member_manifest(
        &substrate_home,
        other_orchestration_session_id,
        "ash_member_other_session",
        "ash_other_orchestrator",
        "wld_other_0001",
        0,
    );
    assert_eq!(
        live_world_member_generations_for_session(&substrate_home, other_orchestration_session_id),
        vec![("ash_member_other_session".to_string(), 0)],
        "fixture must start with an independently live same-agent member in the other session"
    );

    write_policy(&substrate_home, false);
    std::thread::sleep(Duration::from_millis(25));

    repl.send_line("echo second");
    let alert = wait_for_world_restarted_alert_without_stale_liveness(
        &trace_path,
        &substrate_home,
        &orchestration_session_id,
        0,
        Duration::from_secs(5),
    );
    assert_eq!(
        alert.get("world_generation").and_then(Value::as_u64),
        Some(1),
        "restart must still advance the active session generation: {alert:?}"
    );

    assert_eq!(
        live_world_member_generations_for_session(&substrate_home, &orchestration_session_id),
        Vec::<(String, u64)>::new(),
        "current-session stale members must be suppressed after restart"
    );
    assert_eq!(
        live_world_member_generations_for_session(&substrate_home, other_orchestration_session_id),
        vec![("ash_member_other_session".to_string(), 0)],
        "same-agent members in other orchestration sessions must stay isolated from current-session invalidation"
    );

    repl.wait_for_output("second", Duration::from_secs(3))
        .expect("second command output");
    repl.send_line("exit");
    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));

    let current = read_participant_manifest(&substrate_home, "ash_member_current_session");
    assert_eq!(
        current.get("state").and_then(Value::as_str),
        Some("invalidated"),
        "current-session stale member must be invalidated by the restart: {current:?}"
    );

    let other = read_participant_manifest(&substrate_home, "ash_member_other_session");
    assert_eq!(
        other.get("state").and_then(Value::as_str),
        Some("ready"),
        "same-agent member from another orchestration session must stay live: {other:?}"
    );
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn c3_bootstrap_failure_after_attach_cleans_up_world_and_parent_session_state() {
    let temp = temp_dir("substrate-c3-startup-bootstrap-cleanup-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    let fake_codex = write_fake_codex_script_without_session_handle(temp.path());
    write_orchestrator_runtime_world_config(&substrate_home, &fake_codex, "fail_closed");

    let sock_temp = short_socket_dir("sub-c3ws-startup-bootstrap-cleanup-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);

    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_prompt(Duration::from_secs(2))
        .expect("initial prompt");
    repl.send_line("::cli:codex bootstrap should fail");
    wait_for_min_start_sessions_with_output(&repl, &records, 1, Duration::from_secs(3));

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        if repl.try_wait().expect("try_wait") {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let (code, out) = repl.shutdown();
    assert_eq!(
        code, 1,
        "expected targeted orchestrator bootstrap failure exit code 1, got output:\n{out}"
    );
    assert!(
        out.contains("attached control turn ended before ownership could be established")
            || out.contains("failed to establish attached control ownership"),
        "expected bootstrap failure output after world attach; output:\n{out}"
    );

    let session_count = fs::read_dir(sessions_dir(&substrate_home))
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .count();
    if session_count == 0 {
        return;
    }

    let orchestration_session_id = load_single_orchestration_session_id(&substrate_home);
    let session_path = orchestration_session_path(&substrate_home, &orchestration_session_id);
    let persisted = read_orchestration_session(&session_path);
    assert_session_world_binding(&persisted, None, None);
}

#[test]
#[serial]
fn c3_drift_restart_restarts_session_and_suppresses_world_restarted_agent_event_without_explicit_context(
) {
    let temp = temp_dir("substrate-c3-drift-restart-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    let trace_path = home.join(".substrate/trace.jsonl");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(&trace_path, "").expect("seed trace");
    write_profile(&project);
    write_policy(&substrate_home, true);

    let sock_temp = short_socket_dir("sub-c3ws-drift-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);

    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");

    repl.send_line("echo first");
    wait_for_min_records(&records, 1, 1, Duration::from_secs(3));
    repl.wait_for_output("first", Duration::from_secs(3))
        .expect("first command output");

    // Drift: rewrite global policy (changes snapshot hash).
    write_policy(&substrate_home, false);
    // Ensure filesystem mtime changes are observable.
    std::thread::sleep(Duration::from_millis(25));

    repl.send_line("echo second");
    repl.send_line("exit");

    let (_code, out) = repl.shutdown_graceful(Duration::from_secs(3));

    let guard = records.lock().expect("lock records");
    assert!(
        guard.persistent_start_sessions.len() >= 2,
        "expected a session restart on snapshot/workspace drift; output:\n{out}\nrecords: {guard:#?}"
    );

    let lower = out.to_ascii_lowercase();
    assert!(
        lower.contains("restart") && (lower.contains("drift") || lower.contains("snapshot")),
        "expected an operator-visible drift-restart message, got output:\n{out}"
    );

    assert_start_sessions_have_no_shared_world_context(
        &guard,
        "drift restart without explicit orchestration context",
    );
    drop(guard);

    let events = read_trace(&trace_path);
    assert_no_alert_rows_by_code(
        &events,
        "world_restarted",
        "drift restart without explicit orchestration context",
    );
}

#[test]
#[serial]
fn c3_startup_drift_restart_suppresses_world_restarted_agent_event_without_explicit_context() {
    let temp = temp_dir("substrate-c3-startup-drift-restart-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let child = project.join("child");
    let substrate_home = home.join(".substrate");
    let trace_path = home.join(".substrate/trace.jsonl");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&child).expect("create project child");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(&trace_path, "").expect("seed trace");
    write_workspace_marker(&project);
    write_profile(&project);
    write_policy(&substrate_home, true);

    let parent_cwd = temp.path().to_string_lossy().into_owned();

    let sock_temp = short_socket_dir("sub-c3ws-startup-drift-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start_with_first_ready_cwd_override(
        &sock,
        StreamBehavior::Normal,
        parent_cwd,
    );
    let records = server.records();

    let mut repl = PtyRepl::spawn(&child, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    wait_for_min_start_sessions(&records, 2, Duration::from_secs(3));
    repl.send_line("exit");

    let (_code, out) = repl.shutdown_graceful(Duration::from_secs(3));

    let lower = out.to_ascii_lowercase();
    assert!(
        lower.contains("restarting due to snapshot/workspace drift before first command"),
        "expected startup drift restart note, got output:\n{out}"
    );

    let guard = records.lock().expect("lock records");
    assert!(
        guard.persistent_start_sessions.len() >= 2,
        "expected startup drift to restart the session before prompt; records: {guard:#?}"
    );
    assert_start_sessions_have_no_shared_world_context(
        &guard,
        "startup drift restart without explicit orchestration context",
    );
    drop(guard);

    let events = read_trace(&trace_path);
    assert_no_alert_rows_by_code(
        &events,
        "world_restarted",
        "startup drift restart without explicit orchestration context",
    );
}

#[test]
#[serial]
fn c3_drift_fail_closed_suppresses_world_restart_required_agent_event_without_explicit_context() {
    let temp = temp_dir("substrate-c3-drift-fail-closed-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    let trace_path = home.join(".substrate/trace.jsonl");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(&trace_path, "").expect("seed trace");
    write_profile(&project);
    write_policy(&substrate_home, true);
    write_config_with_world_restart_on_drift(&substrate_home, false, "fail_closed");

    let sock_temp = short_socket_dir("sub-c3ws-drift-fail-closed-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");

    repl.send_line("echo first");
    wait_for_min_records(&records, 1, 1, Duration::from_secs(3));
    repl.wait_for_output("first", Duration::from_secs(3))
        .expect("first command output");

    write_policy(&substrate_home, false);
    std::thread::sleep(Duration::from_millis(25));

    repl.send_line("echo second");

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        if repl.try_wait().expect("try_wait") {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let (code, out) = repl.shutdown();
    assert_eq!(
        code, 3,
        "expected fail-closed drift exit code 3, got output:\n{out}"
    );

    let lower = out.to_ascii_lowercase();
    assert!(
        lower.contains("restart required"),
        "expected operator-visible restart-required output, got:\n{out}"
    );
    assert!(
        !lower.contains("second\n"),
        "drift fail-closed must stop before executing the second command; output:\n{out}"
    );

    let guard = records.lock().expect("lock records");
    assert_eq!(
        guard.persistent_start_sessions.len(),
        1,
        "fail-closed drift must not auto-restart the world session; records: {guard:#?}"
    );
    assert_eq!(
        guard.persistent_execs.len(),
        1,
        "fail-closed drift must stop before the second exec reaches world-agent; records: {guard:#?}"
    );
    assert_start_sessions_have_no_shared_world_context(
        &guard,
        "fail-closed drift without explicit orchestration context",
    );
    drop(guard);

    let events = read_trace(&trace_path);
    assert_no_alert_rows_by_code(
        &events,
        "world_restart_required",
        "fail-closed drift without explicit orchestration context",
    );
}

#[test]
#[serial]
fn c3_startup_drift_fail_closed_suppresses_world_restart_required_agent_event_without_explicit_context(
) {
    let temp = temp_dir("substrate-c3-startup-drift-fail-closed-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let child = project.join("child");
    let substrate_home = home.join(".substrate");
    let trace_path = home.join(".substrate/trace.jsonl");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&child).expect("create project child");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(&trace_path, "").expect("seed trace");
    write_workspace_marker(&project);
    write_profile(&project);
    write_policy(&substrate_home, true);
    write_config_with_world_restart_on_drift(&substrate_home, false, "fail_closed");

    let parent_cwd = temp.path().to_string_lossy().into_owned();

    let sock_temp = short_socket_dir("sub-c3ws-startup-drift-fail-closed-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start_with_first_ready_cwd_override(
        &sock,
        StreamBehavior::Normal,
        parent_cwd,
    );
    let records = server.records();

    let mut repl = PtyRepl::spawn(&child, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        if repl.try_wait().expect("try_wait") {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let (code, out) = repl.shutdown();
    assert_eq!(
        code, 3,
        "expected startup fail-closed drift exit code 3, got output:\n{out}"
    );

    let lower = out.to_ascii_lowercase();
    assert!(
        lower.contains("restart required"),
        "expected startup restart-required output, got:\n{out}"
    );

    let guard = records.lock().expect("lock records");
    assert_eq!(
        guard.persistent_start_sessions.len(),
        1,
        "startup fail-closed drift must not auto-restart the world session; records: {guard:#?}"
    );
    assert_start_sessions_have_no_shared_world_context(
        &guard,
        "startup fail-closed drift without explicit orchestration context",
    );
    drop(guard);

    let events = read_trace(&trace_path);
    assert_no_alert_rows_by_code(
        &events,
        "world_restart_required",
        "startup fail-closed drift without explicit orchestration context",
    );
}

#[test]
#[serial]
fn c3_startup_fail_closed_when_persistent_session_cannot_reach_ready() {
    let temp = temp_dir("substrate-c3-startup-fail-closed-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy(&substrate_home, true);

    let sock_temp = short_socket_dir("sub-c3ws-startup-");
    let sock = sock_temp.path().join("world.sock");
    let _server = ReplWorldAgentStub::start(&sock, StreamBehavior::CloseBeforeReady);

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);

    // C3: world-first REPL must start session before accepting input; if it can't reach `ready`,
    // it should terminate (fail closed).
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        if repl.try_wait().expect("try_wait") {
            let (_code, out) = repl.shutdown();
            assert!(
                out.to_ascii_lowercase().contains("fail")
                    || out.to_ascii_lowercase().contains("protocol")
                    || out.to_ascii_lowercase().contains("world"),
                "expected fail-closed startup error output, got:\n{out}"
            );
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let (_code, out) = repl.shutdown();
    panic!("expected REPL to exit fail-closed on startup, but it kept running; output:\n{out}");
}

#[test]
#[serial]
fn c3_startup_surfaces_fatal_persistent_session_error_before_ready() {
    let temp = temp_dir("substrate-c3-startup-fatal-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy(&substrate_home, true);

    let sock_temp = short_socket_dir("sub-c3ws-startup-fatal-");
    let sock = sock_temp.path().join("world.sock");
    let _server = ReplWorldAgentStub::start(&sock, StreamBehavior::FatalBeforeReady);

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        if repl.try_wait().expect("try_wait") {
            let (_code, out) = repl.shutdown();
            assert!(
                out.contains("simulated persistent start failure"),
                "expected fatal startup message to be surfaced, got:\n{out}"
            );
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let (_code, out) = repl.shutdown();
    panic!(
        "expected REPL to exit on fatal startup error before ready, but it kept running; output:\n{out}"
    );
}

#[test]
#[serial]
fn c3_drift_restart_refreshes_anchor_env_for_new_cwd() {
    let temp = tempfile::Builder::new()
        .prefix("substrate-c3-anchor-drift-")
        .tempdir_in("/tmp")
        .expect("create anchor drift tempdir in /tmp");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let child = project.join("child");
    let substrate_home = home.join(".substrate");

    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&child).expect("create project child");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");

    write_workspace_marker(&project);
    write_profile(&project);
    write_policy(&substrate_home, true);

    let sock_temp = short_socket_dir("sub-c3ws-anchor-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&child, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");

    let project_canon = project.canonicalize().unwrap_or(project.clone());
    let parent_canon = temp
        .path()
        .canonicalize()
        .unwrap_or_else(|_| temp.path().to_path_buf());
    let project_str = project_canon.to_string_lossy().into_owned();
    let parent_str = parent_canon.to_string_lossy().into_owned();

    // Move up to project root (still in workspace), then move to parent (workspace_root=None).
    // Drift restart should happen immediately after the second `cd` (the REPL checks drift both
    // before and after each world command). Wait on server-side records to avoid PTY timing
    // differences across platforms.
    repl.send_line("cd ..");
    wait_for_min_records(&records, 1, 1, Duration::from_secs(3));
    repl.wait_for_output("__PERSISTENT_EXEC_STUB__ eof cd ..", Duration::from_secs(2))
        .expect("cd .. executed in persistent session");

    // Use a distinct spelling so we can wait on the second exec deterministically.
    repl.send_line("cd ../");
    wait_for_min_records(&records, 2, 1, Duration::from_secs(3));
    repl.wait_for_output(
        "__PERSISTENT_EXEC_STUB__ eof cd ../",
        Duration::from_secs(2),
    )
    .expect("cd ../ executed in persistent session");

    // Ensure the drift restart has actually occurred (i.e., the client re-sent a new StartSession)
    // before terminating the REPL. Otherwise, a fast `exit` can race the restart and flake.
    wait_for_min_records(&records, 2, 2, Duration::from_secs(5));
    repl.send_line("exit");
    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));

    let guard = records.lock().expect("lock records");
    assert!(
        guard.persistent_start_sessions.len() >= 2,
        "expected a session restart when leaving workspace root; records: {guard:#?}"
    );

    let first = &guard.persistent_start_sessions[0];
    let second = &guard.persistent_start_sessions[1];

    assert_eq!(
        first.env.get("SUBSTRATE_ANCHOR_PATH").map(String::as_str),
        Some(project_str.as_str()),
        "expected initial anchor path to be workspace root"
    );
    assert_eq!(
        second.env.get("SUBSTRATE_ANCHOR_PATH").map(String::as_str),
        Some(parent_str.as_str()),
        "expected drift restart to refresh anchor path for new cwd"
    );
}

#[test]
#[serial]
fn c3_drift_restart_refreshes_world_network_routing() {
    let temp = temp_dir("substrate-c3-net-drift-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    let trace_path = home.join(".substrate/trace.jsonl");

    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(&trace_path, "").expect("seed trace");

    write_profile(&project);
    write_policy_with_net_allowed(&substrate_home, true, "[\"*\"]");
    write_config(&substrate_home, true);

    let sock_temp = short_socket_dir("sub-c3ws-net-drift-");
    let sock = sock_temp.path().join("world.sock");
    let server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);
    let records = server.records();

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");

    repl.send_line("echo first");
    wait_for_min_records(&records, 1, 1, Duration::from_secs(3));
    repl.wait_for_output("first", Duration::from_secs(3))
        .expect("first command output");

    write_policy_with_net_allowed(
        &substrate_home,
        true,
        "[\" Example.COM. \", \"example.com\", \"Api.Example.com.\"]",
    );

    repl.send_line("echo second");
    wait_for_min_records(&records, 2, 2, Duration::from_secs(5));
    repl.wait_for_output("second", Duration::from_secs(3))
        .expect("second command output");
    repl.send_line("exit");

    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));

    let guard = records.lock().expect("lock records");
    assert!(
        guard.persistent_start_sessions.len() >= 2,
        "expected a session restart after policy drift; records: {guard:#?}"
    );

    let first = &guard.persistent_start_sessions[0];
    let second = &guard.persistent_start_sessions[1];

    assert_world_network_payload(first, false, &[]);
    assert_world_network_payload(second, true, &["example.com", "api.example.com"]);
    assert_start_sessions_have_no_shared_world_context(
        &guard,
        "network-routing drift restart without explicit orchestration context",
    );

    let second_net_allowed = second
        .policy_snapshot
        .get("net_allowed")
        .and_then(|value| value.as_array())
        .expect("policy_snapshot.net_allowed array");
    let second_net_allowed: Vec<&str> = second_net_allowed
        .iter()
        .map(|value| value.as_str().expect("net_allowed string"))
        .collect();
    assert_eq!(second_net_allowed, vec!["example.com", "api.example.com"]);

    drop(guard);

    let events = read_trace(&trace_path);
    assert_no_alert_rows_by_code(
        &events,
        "world_restarted",
        "network-routing drift restart without explicit orchestration context",
    );
}
