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

fn read_trace(trace_path: &Path) -> Vec<Value> {
    fs::read_to_string(trace_path)
        .expect("read trace")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("parse trace line"))
        .collect()
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
    wait_for_min_start_sessions(&records, 1, Duration::from_secs(3));
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

        let mut cmd = CommandBuilder::new(binary_path());
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

#[test]
#[serial]
fn c3_drift_restart_restarts_session_and_emits_message() {
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

    drop(guard);

    let events = read_trace(&trace_path);
    let alerts: Vec<&Value> = events
        .iter()
        .filter(|event| event.get("event_type").and_then(Value::as_str) == Some("agent_event"))
        .filter(|event| event.get("kind").and_then(Value::as_str) == Some("alert"))
        .filter(|event| {
            event.pointer("/data/code").and_then(Value::as_str) == Some("world_restarted")
        })
        .collect();
    assert_eq!(
        alerts.len(),
        1,
        "expected exactly one world_restarted alert record, got: {alerts:?}"
    );

    let alert = alerts[0];
    assert_eq!(
        alert.get("agent_id").and_then(Value::as_str),
        Some("shell"),
        "restart alert must attribute to shell: {alert:?}"
    );
    assert_eq!(
        alert.get("backend_id").and_then(Value::as_str),
        Some("shell:repl"),
        "restart alert must preserve backend attribution: {alert:?}"
    );
    assert_eq!(
        alert.get("role").and_then(Value::as_str),
        Some("orchestrator"),
        "restart alert must carry role=orchestrator: {alert:?}"
    );
    assert_eq!(
        alert.get("world_id").and_then(Value::as_str),
        Some("wld_stub_0002"),
        "top-level world_id must point at the active replacement world: {alert:?}"
    );
    assert_eq!(
        alert.get("world_generation").and_then(Value::as_u64),
        Some(1),
        "top-level world_generation must point at the active replacement generation: {alert:?}"
    );
    assert_eq!(
        alert.pointer("/data/reason").and_then(Value::as_str),
        Some("policy_snapshot_changed"),
        "policy drift restart should classify as policy_snapshot_changed: {alert:?}"
    );
    assert_eq!(
        alert.pointer("/data/on_drift").and_then(Value::as_str),
        Some("auto_restart"),
        "restart alert must record on_drift=auto_restart: {alert:?}"
    );
    assert_eq!(
        alert
            .pointer("/data/previous_world_id")
            .and_then(Value::as_str),
        Some("wld_stub_0001"),
        "restart alert must capture previous world_id: {alert:?}"
    );
    assert_eq!(
        alert.pointer("/data/new_world_id").and_then(Value::as_str),
        Some("wld_stub_0002"),
        "restart alert must capture new world_id: {alert:?}"
    );
    assert_eq!(
        alert
            .pointer("/data/previous_world_generation")
            .and_then(Value::as_u64),
        Some(0),
        "restart alert must capture previous generation 0: {alert:?}"
    );
    assert_eq!(
        alert
            .pointer("/data/new_world_generation")
            .and_then(Value::as_u64),
        Some(1),
        "restart alert must capture new generation 1: {alert:?}"
    );
    assert!(
        alert.pointer("/data/world_id").is_none() && alert.pointer("/data/world_generation").is_none(),
        "restart alerts must keep active world identity at the top level, not duplicate it under data: {alert:?}"
    );
}

#[test]
#[serial]
fn c3_startup_drift_restart_emits_world_restarted_alert() {
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

    let events = read_trace(&trace_path);
    let alerts: Vec<&Value> = events
        .iter()
        .filter(|event| event.get("event_type").and_then(Value::as_str) == Some("agent_event"))
        .filter(|event| event.get("kind").and_then(Value::as_str) == Some("alert"))
        .filter(|event| {
            event.pointer("/data/code").and_then(Value::as_str) == Some("world_restarted")
        })
        .collect();
    assert_eq!(
        alerts.len(),
        1,
        "expected exactly one startup world_restarted alert record, got: {alerts:?}"
    );

    let alert = alerts[0];
    assert_eq!(
        alert.get("world_id").and_then(Value::as_str),
        Some("wld_stub_0002"),
        "startup restart alert must publish the active replacement world at the top level: {alert:?}"
    );
    assert_eq!(
        alert.get("world_generation").and_then(Value::as_u64),
        Some(1),
        "startup restart alert must publish the replacement generation at the top level: {alert:?}"
    );
    assert_eq!(
        alert.pointer("/data/reason").and_then(Value::as_str),
        Some("workspace_root_changed"),
        "startup drift restart should classify as workspace_root_changed: {alert:?}"
    );
    assert_eq!(
        alert
            .pointer("/data/previous_world_id")
            .and_then(Value::as_str),
        Some("wld_stub_0001"),
        "startup drift alert must capture the first world id: {alert:?}"
    );
    assert_eq!(
        alert.pointer("/data/new_world_id").and_then(Value::as_str),
        Some("wld_stub_0002"),
        "startup drift alert must capture the restarted world id: {alert:?}"
    );
    assert_eq!(
        alert
            .pointer("/data/previous_world_generation")
            .and_then(Value::as_u64),
        Some(0),
        "startup drift alert must start from generation 0: {alert:?}"
    );
    assert_eq!(
        alert
            .pointer("/data/new_world_generation")
            .and_then(Value::as_u64),
        Some(1),
        "startup drift alert must increment to generation 1: {alert:?}"
    );
    assert!(
        alert.pointer("/data/world_id").is_none() && alert.pointer("/data/world_generation").is_none(),
        "startup restart alerts must keep active world identity at the top level, not under data: {alert:?}"
    );
}

#[test]
#[serial]
fn c3_drift_fail_closed_emits_world_restart_required_alert() {
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
    drop(guard);

    let events = read_trace(&trace_path);
    let alerts: Vec<&Value> = events
        .iter()
        .filter(|event| event.get("event_type").and_then(Value::as_str) == Some("agent_event"))
        .filter(|event| event.get("kind").and_then(Value::as_str) == Some("alert"))
        .filter(|event| {
            event.pointer("/data/code").and_then(Value::as_str) == Some("world_restart_required")
        })
        .collect();
    assert_eq!(
        alerts.len(),
        1,
        "expected exactly one world_restart_required alert record, got: {alerts:?}"
    );

    let alert = alerts[0];
    assert_eq!(
        alert.get("agent_id").and_then(Value::as_str),
        Some("shell"),
        "restart-required alert must attribute to shell: {alert:?}"
    );
    assert_eq!(
        alert.get("backend_id").and_then(Value::as_str),
        Some("shell:repl"),
        "restart-required alert must preserve backend attribution: {alert:?}"
    );
    assert_eq!(
        alert.get("role").and_then(Value::as_str),
        Some("orchestrator"),
        "restart-required alert must carry role=orchestrator: {alert:?}"
    );
    assert_eq!(
        alert.get("world_id").and_then(Value::as_str),
        Some("wld_stub_0001"),
        "top-level world_id must point at the current world: {alert:?}"
    );
    assert_eq!(
        alert.get("world_generation").and_then(Value::as_u64),
        Some(0),
        "top-level world_generation must publish the current generation for fail-closed restart-required alerts: {alert:?}"
    );
    assert_eq!(
        alert.pointer("/data/reason").and_then(Value::as_str),
        Some("policy_snapshot_changed"),
        "policy drift fail-closed alert should classify as policy_snapshot_changed: {alert:?}"
    );
    assert_eq!(
        alert.pointer("/data/on_drift").and_then(Value::as_str),
        Some("fail_closed"),
        "restart-required alert must record on_drift=fail_closed: {alert:?}"
    );
    assert_eq!(
        alert
            .pointer("/data/required_action")
            .and_then(Value::as_str),
        Some("restart_world"),
        "restart-required alert must record required_action=restart_world: {alert:?}"
    );
    assert!(
        alert.pointer("/data/world_id").is_none() && alert.pointer("/data/world_generation").is_none(),
        "restart-required alerts must keep current world identity at the top level when known: {alert:?}"
    );
}

#[test]
#[serial]
fn c3_startup_drift_fail_closed_emits_world_restart_required_alert() {
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
    drop(guard);

    let events = read_trace(&trace_path);
    let alerts: Vec<&Value> = events
        .iter()
        .filter(|event| event.get("event_type").and_then(Value::as_str) == Some("agent_event"))
        .filter(|event| event.get("kind").and_then(Value::as_str) == Some("alert"))
        .filter(|event| {
            event.pointer("/data/code").and_then(Value::as_str) == Some("world_restart_required")
        })
        .collect();
    assert_eq!(
        alerts.len(),
        1,
        "expected exactly one startup world_restart_required alert record, got: {alerts:?}"
    );

    let alert = alerts[0];
    assert_eq!(
        alert.get("world_id").and_then(Value::as_str),
        Some("wld_stub_0001"),
        "startup fail-closed alert must publish the current world id at the top level: {alert:?}"
    );
    assert_eq!(
        alert.get("world_generation").and_then(Value::as_u64),
        Some(0),
        "startup fail-closed alert must publish the current generation at the top level: {alert:?}"
    );
    assert_eq!(
        alert.pointer("/data/reason").and_then(Value::as_str),
        Some("workspace_root_changed"),
        "startup fail-closed drift should classify as workspace_root_changed: {alert:?}"
    );
    assert_eq!(
        alert.pointer("/data/on_drift").and_then(Value::as_str),
        Some("fail_closed"),
        "startup fail-closed alert must record on_drift=fail_closed: {alert:?}"
    );
    assert_eq!(
        alert
            .pointer("/data/required_action")
            .and_then(Value::as_str),
        Some("restart_world"),
        "startup fail-closed alert must record required_action=restart_world: {alert:?}"
    );
    assert!(
        alert.pointer("/data/world_id").is_none() && alert.pointer("/data/world_generation").is_none(),
        "startup fail-closed alerts must keep current world identity at the top level when known: {alert:?}"
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
    let temp = temp_dir("substrate-c3-anchor-drift-");
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

    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");

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
}
