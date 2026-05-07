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
use substrate_common::agent_events::{AgentEvent, MessageEventKind};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use support::{binary_path, ensure_substrate_built, temp_dir};

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

fn write_host_only_profile(project_dir: &Path) {
    let profile = r#"id: test-policy
name: Test Policy
world_fs:
  host_visible: true
  fail_closed:
    routing: false
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
    fs::write(project_dir.join(".substrate-profile"), profile).expect("write host-only profile");
}

#[allow(dead_code)]
fn write_policy(home_substrate: &Path) {
    fs::create_dir_all(home_substrate).expect("create SUBSTRATE_HOME");
    let policy = r#"id: test-global-policy
name: Test Global Policy
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
    fs::write(home_substrate.join("policy.yaml"), policy).expect("write policy.yaml");
}

fn write_host_only_policy(home_substrate: &Path) {
    fs::create_dir_all(home_substrate).expect("create SUBSTRATE_HOME");
    let policy = r#"id: test-global-policy
name: Test Global Policy
world_fs:
  host_visible: true
  fail_closed:
    routing: false
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
    fs::write(home_substrate.join("policy.yaml"), policy).expect("write host-only policy.yaml");
}

fn write_orchestrator_runtime_config(home_substrate: &Path, fake_codex: &Path) {
    fs::create_dir_all(home_substrate.join("agents")).expect("create agents dir");
    fs::write(
        home_substrate.join("config.yaml"),
        "agents:\n  enabled: true\n  hub:\n    orchestrator_agent_id: codex\n",
    )
    .expect("write config.yaml");
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

#[cfg(unix)]
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

fn extract_session_id(output: &[u8]) -> String {
    let text = String::from_utf8_lossy(output);
    text.lines()
        .find_map(|line| line.strip_prefix("Session ID: ").map(str::to_string))
        .expect("session id line")
}

fn read_trace_events(trace_path: &Path) -> Vec<Value> {
    fs::read_to_string(trace_path)
        .expect("read trace")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("parse trace line"))
        .collect()
}

fn wait_for_trace_output(trace_path: &Path, needle: &str, timeout: Duration) -> Option<usize> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Ok(contents) = fs::read_to_string(trace_path) {
            if let Some(index) = contents.find(needle) {
                return Some(index);
            }
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    None
}

fn sessions_dir(substrate_home: &Path) -> PathBuf {
    substrate_home.join("run/agent-hub/sessions")
}

fn canonical_session_path(substrate_home: &Path, orchestration_session_id: &str) -> PathBuf {
    sessions_dir(substrate_home)
        .join(orchestration_session_id)
        .join("session.json")
}

fn canonical_participant_path(
    substrate_home: &Path,
    orchestration_session_id: &str,
    participant_id: &str,
) -> PathBuf {
    sessions_dir(substrate_home)
        .join(orchestration_session_id)
        .join("participants")
        .join(format!("{participant_id}.json"))
}

fn canonical_lease_path(
    substrate_home: &Path,
    orchestration_session_id: &str,
    participant_id: &str,
) -> PathBuf {
    sessions_dir(substrate_home)
        .join(orchestration_session_id)
        .join("leases")
        .join(format!("{participant_id}.lease"))
}

fn load_single_session_record(substrate_home: &Path) -> Value {
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
        .expect("parse canonical session file");
    }

    let sessions_dir = substrate_home.join("run/agent-hub/sessions");
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
    serde_json::from_str::<Value>(
        &fs::read_to_string(session_path).expect("read flat session file"),
    )
    .expect("parse flat session file")
}

fn load_single_orchestration_session_id(substrate_home: &Path) -> String {
    load_single_session_record(substrate_home)
        .get("orchestration_session_id")
        .and_then(Value::as_str)
        .expect("session orchestration_session_id")
        .to_string()
}

fn load_single_active_participant_id(substrate_home: &Path) -> String {
    load_single_session_record(substrate_home)
        .get("active_session_handle_id")
        .and_then(Value::as_str)
        .expect("session active_session_handle_id")
        .to_string()
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

impl Drop for PtyRepl {
    fn drop(&mut self) {
        self.stop_reader.store(true, Ordering::Relaxed);
        self.master.take();

        if self.waited.is_none() {
            let _ = self.child.kill();
            let _ = self.child.try_wait().ok().flatten().map(|s| {
                self.waited = Some(s);
            });
        }

        if let Some(handle) = self.reader_handle.take() {
            let _ = handle.join();
        }
    }
}

impl PtyRepl {
    fn spawn(
        project_dir: &Path,
        home_dir: &Path,
        substrate_home: &Path,
        trace_path: &Path,
    ) -> Self {
        Self::spawn_with_env(project_dir, home_dir, substrate_home, trace_path, &[])
    }

    fn spawn_with_env(
        project_dir: &Path,
        home_dir: &Path,
        substrate_home: &Path,
        trace_path: &Path,
        extra_env: &[(&str, &str)],
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
        cmd.cwd(project_dir);
        cmd.env("HOME", home_dir);
        cmd.env("USERPROFILE", home_dir);
        cmd.env("SUBSTRATE_HOME", substrate_home);
        cmd.env("SUBSTRATE_MANAGER_MANIFEST", manager_manifest_path());
        cmd.env("SHIM_TRACE_LOG", trace_path);
        cmd.env("SUBSTRATE_OVERRIDE_WORLD", "disabled");
        cmd.env_remove("SHIM_ORIGINAL_PATH");
        cmd.env_remove("SUBSTRATE_WORLD");
        cmd.env_remove("SUBSTRATE_WORLD_ENABLED");
        cmd.env_remove("SUBSTRATE_WORLD_ID");
        cmd.env("SHELL", "/bin/bash");
        for (key, value) in extra_env {
            cmd.env(key, value);
        }
        cmd.arg("--async-repl");
        cmd.arg("--shim-skip");

        let child = pair.slave.spawn_command(cmd).expect("spawn substrate");
        let writer: Arc<Mutex<Box<dyn Write + Send>>> =
            Arc::new(Mutex::new(master.take_writer().expect("take writer")));

        let output = Arc::new(Mutex::new(Vec::new()));
        let stop_reader = Arc::new(AtomicBool::new(false));
        let output_for_thread = output.clone();
        let stop_for_thread = stop_reader.clone();
        let writer_for_thread = Arc::downgrade(&writer);
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
                    let chunk = &buf[..n];
                    let mut probe = carry.clone();
                    probe.extend_from_slice(chunk);

                    if probe.windows(4).any(|w| w == b"\x1b[6n") {
                        if let Some(writer) = writer_for_thread.upgrade() {
                            if let Ok(mut w) = writer.lock() {
                                let _ = w.write_all(b"\x1b[1;1R");
                                let _ = w.flush();
                            }
                        }
                    }
                    if probe.windows(5).any(|w| w == b"\x1b[18t") {
                        if let Some(writer) = writer_for_thread.upgrade() {
                            if let Ok(mut w) = writer.lock() {
                                let _ = w.write_all(b"\x1b[8;24;80t");
                                let _ = w.flush();
                            }
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
        let mut writer = self.writer.lock().expect("pty writer");
        writer.write_all(line.as_bytes()).expect("write line");
        writer.write_all(b"\n").expect("write LF");
        writer.flush().expect("flush");
    }

    fn output_string(&self) -> String {
        let out = self.output.lock().expect("output lock");
        String::from_utf8_lossy(&out).into_owned()
    }

    fn wait_for_output(&self, needle: &str, timeout: Duration) -> Option<usize> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            let text = self.output_string();
            if let Some(pos) = text.find(needle) {
                return Some(pos);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        None
    }

    fn shutdown_graceful(mut self, timeout: Duration) -> (i32, Vec<u8>) {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if let Ok(Some(status)) = self.child.try_wait() {
                self.waited = Some(status);
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        if self.waited.is_none() {
            let _ = self.child.kill();
            if let Ok(Some(status)) = self.child.try_wait() {
                self.waited = Some(status);
            }
        }

        self.stop_reader.store(true, Ordering::Relaxed);
        self.master.take();
        if let Some(handle) = self.reader_handle.take() {
            let _ = handle.join();
        }

        let code = self
            .waited
            .as_ref()
            .map(|s| s.exit_code() as i32)
            .unwrap_or(-1);
        let out = self.output.lock().expect("output lock").clone();
        (code, out)
    }
}

#[test]
#[serial]
fn agent_events_append_flattened_agent_event_records_with_join_keys() {
    let temp = temp_dir("substrate-agent-hub-trace-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(substrate_home.join("shims")).expect("create shims dir");
    fs::create_dir_all(&project).expect("create project dir");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(substrate_home.join("trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy(&substrate_home);

    let trace_path = substrate_home.join("trace.jsonl");
    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &trace_path);

    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");

    repl.send_line(":demo-burst 1 1 0");
    repl.wait_for_output(
        "scheduled burst: agents=1, events_per_agent=1, delay_ms=0",
        Duration::from_secs(2),
    )
    .expect("demo burst ack");
    repl.wait_for_output("chunk #00000", Duration::from_secs(3))
        .expect("burst event rendered");

    repl.send_line("exit");
    let (code, out) = repl.shutdown_graceful(Duration::from_secs(3));
    assert_eq!(
        code,
        0,
        "expected clean exit; output:\n{}",
        String::from_utf8_lossy(&out)
    );

    let events: Vec<Value> = fs::read_to_string(&trace_path)
        .expect("read trace")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("parse trace line"))
        .collect();

    let agent_events: Vec<&Value> = events
        .iter()
        .filter(|event| event.get("event_type").and_then(Value::as_str) == Some("agent_event"))
        .collect();

    assert!(
        !agent_events.is_empty(),
        "expected at least one event_type=agent_event record; trace had {} records",
        events.len()
    );

    let burst_records: Vec<&&Value> = agent_events
        .iter()
        .filter(|event| event.get("agent_id").and_then(Value::as_str) == Some("burst-00"))
        .collect();
    assert!(
        !burst_records.is_empty(),
        "expected at least one agent_event record for burst-00; got: {agent_events:?}"
    );

    for record in burst_records {
        assert_eq!(
            record.get("component").and_then(Value::as_str),
            Some("agent-hub"),
            "agent_event record must have component=agent-hub; got: {record:?}"
        );

        for key in [
            "ts",
            "session_id",
            "kind",
            "agent_id",
            "orchestration_session_id",
            "run_id",
            "data",
        ] {
            assert!(
                record.get(key).is_some(),
                "expected flattened top-level join/envelope key `{key}`; got: {record:?}"
            );
        }

        assert!(
            record.get("envelope").is_none(),
            "agent_event record must be flattened (no nested envelope object); got: {record:?}"
        );
        assert!(
            record.get("payload").is_none(),
            "agent_event record must be flattened (no payload wrapper); got: {record:?}"
        );
    }
}

#[test]
#[serial]
fn runtime_owned_agent_event_rows_retain_shell_session_and_real_orchestration_session() {
    let temp = temp_dir("substrate-runtime-agent-hub-trace-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(substrate_home.join("shims")).expect("create shims dir");
    fs::create_dir_all(&project).expect("create project dir");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(substrate_home.join("trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    let fake_codex = write_fake_codex_script(temp.path());
    write_orchestrator_runtime_config(&substrate_home, &fake_codex);

    let trace_path = substrate_home.join("trace.jsonl");
    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &trace_path);

    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.wait_for_output("substrate>", Duration::from_secs(2))
        .expect("initial prompt");
    repl.send_line("::cli:codex trace owned runtime");
    let runtime_ready = repl.wait_for_output(
        "shell-owned orchestrator session is ready via retained attached control ownership",
        Duration::from_secs(5),
    );
    if runtime_ready.is_none() {
        let trace_ready = wait_for_trace_output(
            &trace_path,
            "shell-owned orchestrator session is ready via retained attached control ownership",
            Duration::from_secs(5),
        );
        assert!(
            trace_ready.is_some(),
            "runtime ready trace event; output:\n{}\ntrace:\n{}",
            repl.output_string(),
            fs::read_to_string(&trace_path).unwrap_or_default()
        );
    }

    repl.send_line("exit");
    let (code, out) = repl.shutdown_graceful(Duration::from_secs(5));
    assert_eq!(
        code,
        0,
        "expected clean exit; output:\n{}",
        String::from_utf8_lossy(&out)
    );

    let shell_session_id = extract_session_id(&out);
    let orchestration_session_id = load_single_orchestration_session_id(&substrate_home);
    assert_ne!(
        shell_session_id, orchestration_session_id,
        "shell trace session_id and orchestration_session_id must remain distinct identities"
    );
    let participant_id = load_single_active_participant_id(&substrate_home);
    let canonical_session = canonical_session_path(&substrate_home, &orchestration_session_id);
    let canonical_participant =
        canonical_participant_path(&substrate_home, &orchestration_session_id, &participant_id);
    let canonical_lease =
        canonical_lease_path(&substrate_home, &orchestration_session_id, &participant_id);
    let legacy_handle = substrate_home
        .join("run/agent-hub/handles")
        .join(format!("{participant_id}.json"));
    let events = read_trace_events(&trace_path);
    let runtime_records = events
        .iter()
        .filter(|event| event.get("event_type").and_then(Value::as_str) == Some("agent_event"))
        .filter(|event| event.get("agent_id").and_then(Value::as_str) == Some("codex"))
        .collect::<Vec<_>>();

    assert!(
        !runtime_records.is_empty(),
        "expected runtime-owned codex agent_event rows in trace; got: {events:?}"
    );
    assert!(
        canonical_session.is_file(),
        "runtime session persistence must write canonical session roots: {}",
        canonical_session.display()
    );
    assert!(
        canonical_participant.is_file(),
        "runtime session persistence must write canonical participant roots: {}",
        canonical_participant.display()
    );
    assert!(
        canonical_lease.is_file(),
        "runtime session persistence must write canonical lease roots: {}",
        canonical_lease.display()
    );
    assert!(
        !legacy_handle.exists(),
        "runtime session persistence must not resurrect handles/*.json authority: {}",
        legacy_handle.display()
    );

    for record in runtime_records {
        assert_eq!(
            record.get("session_id").and_then(Value::as_str),
            Some(shell_session_id.as_str()),
            "runtime-owned agent_event row must retain the shell trace session_id: {record:?}"
        );
        assert_eq!(
            record
                .get("orchestration_session_id")
                .and_then(Value::as_str),
            Some(orchestration_session_id.as_str()),
            "runtime-owned agent_event row must retain the authoritative orchestration_session_id: {record:?}"
        );
        assert_ne!(
            record.get("orchestration_session_id").and_then(Value::as_str),
            record.get("session_id").and_then(Value::as_str),
            "runtime-owned agent_event row must not synthesize orchestration_session_id from session_id: {record:?}"
        );
        assert_eq!(
            record.get("participant_id").and_then(Value::as_str),
            Some(participant_id.as_str()),
            "runtime-owned agent_event row must retain participant_id from the live runtime manifest: {record:?}"
        );
        assert!(
            record.get("parent_participant_id").is_none(),
            "runtime-owned orchestrator rows with no parent lineage must omit parent_participant_id: {record:?}"
        );
        assert!(
            record.get("resumed_from_participant_id").is_none(),
            "runtime-owned orchestrator rows with no resume lineage must omit resumed_from_participant_id: {record:?}"
        );
    }
}

#[test]
#[serial]
fn no_context_shell_command_completion_does_not_synthesize_agent_event_trace_row() {
    let temp = temp_dir("substrate-no-context-command-trace-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(substrate_home.join("shims")).expect("create shims dir");
    fs::create_dir_all(&project).expect("create project dir");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(substrate_home.join("trace.jsonl"), "").expect("seed trace");
    write_host_only_profile(&project);
    write_host_only_policy(&substrate_home);

    let trace_path = substrate_home.join("trace.jsonl");
    let mut repl = PtyRepl::spawn_with_env(
        &project,
        &home,
        &substrate_home,
        &trace_path,
        &[("SUBSTRATE_REPL_HOST_ESCAPE", "1")],
    );

    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");
    repl.send_line(":host false");
    repl.wait_for_output("Command failed with status: 1", Duration::from_secs(3))
        .expect("host failure output");
    repl.send_line("exit");
    let (code, out) = repl.shutdown_graceful(Duration::from_secs(5));
    assert_eq!(
        code,
        0,
        "expected clean exit; output:\n{}",
        String::from_utf8_lossy(&out)
    );

    let shell_session_id = extract_session_id(&out);
    let events = read_trace_events(&trace_path);
    let repl_status_stop_records = events
        .iter()
        .filter(|event| event.get("event_type").and_then(Value::as_str) == Some("repl_status"))
        .filter(|event| event.get("component").and_then(Value::as_str) == Some("shell"))
        .filter(|event| {
            event.get("session_id").and_then(Value::as_str) == Some(shell_session_id.as_str())
        })
        .filter(|event| event.get("stage").and_then(Value::as_str) == Some("stop"))
        .collect::<Vec<_>>();
    let shell_completion_records = events
        .iter()
        .filter(|event| event.get("event_type").and_then(Value::as_str) == Some("agent_event"))
        .filter(|event| event.get("agent_id").and_then(Value::as_str) == Some("shell"))
        .filter(|event| event.get("kind").and_then(Value::as_str) == Some("task_end"))
        .collect::<Vec<_>>();

    assert!(
        !repl_status_stop_records.is_empty(),
        "suppressing a shell-owned orchestration agent_event row must remain additive; expected non-agent shell trace records to continue: {events:?}"
    );
    for record in repl_status_stop_records {
        assert_eq!(
            record
                .get("metrics")
                .and_then(|metrics| metrics.get("commands_executed"))
                .and_then(Value::as_u64),
            Some(1),
            "no-context shell completion should still contribute to persisted shell metrics even when the orchestration-scoped agent_event row is suppressed: {record:?}"
        );
        assert!(
            record.get("orchestration_session_id").is_none(),
            "shell-owned non-agent trace records without orchestration context must not grow heuristic orchestration_session_id fields: {record:?}"
        );
    }
    assert!(
        shell_completion_records.is_empty(),
        "no-context shell completion must not synthesize an orchestration-scoped agent_event row: {shell_completion_records:?}"
    );
}

#[test]
fn flattened_agent_event_records_retain_parent_run_id_when_present() {
    let mut event = AgentEvent::message(
        "nested-agent",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
        MessageEventKind::Status,
        "nested gateway request completed",
    );
    event.parent_run_id = Some("0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13".to_string());

    let mut record = event.to_trace_record().expect("flatten agent event");
    let obj = record
        .as_object_mut()
        .expect("agent event trace record should be an object");
    obj.insert(
        "event_type".to_string(),
        Value::String("agent_event".to_string()),
    );
    obj.insert(
        "session_id".to_string(),
        Value::String("ses_agent_hub".to_string()),
    );
    obj.insert(
        "component".to_string(),
        Value::String("agent-hub".to_string()),
    );

    assert_eq!(
        record.get("parent_run_id").and_then(Value::as_str),
        Some("0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13"),
        "flattened agent_event records must retain parent_run_id when present: {record:?}"
    );
}

#[test]
fn flattened_agent_event_records_retain_participant_lineage_when_present() {
    let mut event = AgentEvent::message(
        "runtime-agent",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
        MessageEventKind::Status,
        "runtime participant event",
    );
    event.participant_id = Some("ash_live".to_string());
    event.parent_participant_id = Some("ash_parent".to_string());
    event.resumed_from_participant_id = Some("ash_previous".to_string());

    let mut record = event.to_trace_record().expect("flatten agent event");
    let obj = record
        .as_object_mut()
        .expect("agent event trace record should be an object");
    obj.insert(
        "event_type".to_string(),
        Value::String("agent_event".to_string()),
    );
    obj.insert(
        "session_id".to_string(),
        Value::String("ses_agent_hub".to_string()),
    );
    obj.insert(
        "component".to_string(),
        Value::String("agent-hub".to_string()),
    );

    assert_eq!(
        record.get("participant_id").and_then(Value::as_str),
        Some("ash_live"),
        "flattened agent_event records must retain participant_id when present: {record:?}"
    );
    assert_eq!(
        record.get("parent_participant_id").and_then(Value::as_str),
        Some("ash_parent"),
        "flattened agent_event records must retain parent_participant_id when present: {record:?}"
    );
    assert_eq!(
        record
            .get("resumed_from_participant_id")
            .and_then(Value::as_str),
        Some("ash_previous"),
        "flattened agent_event records must retain resumed_from_participant_id when present: {record:?}"
    );
}

#[test]
fn world_member_registered_status_and_terminal_trace_rows_preserve_top_level_world_identity() {
    let mut registered = AgentEvent::message(
        "runtime-agent",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f15",
        MessageEventKind::Registered,
        "world member registered",
    );
    registered.participant_id = Some("ash_member_live".to_string());
    registered.parent_participant_id = Some("ash_orchestrator_live".to_string());
    registered.role = Some("member".to_string());
    registered.world_id = Some("wld_trace_member_0001".to_string());
    registered.world_generation = Some(8);

    let mut status = AgentEvent::message(
        "runtime-agent",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f16",
        MessageEventKind::Status,
        "world member still live",
    );
    status.participant_id = Some("ash_member_live".to_string());
    status.parent_participant_id = Some("ash_orchestrator_live".to_string());
    status.role = Some("member".to_string());
    status.world_id = Some("wld_trace_member_0001".to_string());
    status.world_generation = Some(8);

    let mut terminal = AgentEvent::message(
        "runtime-agent",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f17",
        MessageEventKind::TaskEnd,
        "world member exited cleanly",
    );
    terminal.participant_id = Some("ash_member_live".to_string());
    terminal.parent_participant_id = Some("ash_orchestrator_live".to_string());
    terminal.role = Some("member".to_string());
    terminal.world_id = Some("wld_trace_member_0001".to_string());
    terminal.world_generation = Some(8);

    for (event, expected_kind) in [
        (registered, "registered"),
        (status, "status"),
        (terminal, "task_end"),
    ] {
        let mut record = event.to_trace_record().expect("flatten agent event");
        let obj = record
            .as_object_mut()
            .expect("agent event trace record should be an object");
        obj.insert(
            "event_type".to_string(),
            Value::String("agent_event".to_string()),
        );
        obj.insert(
            "session_id".to_string(),
            Value::String("ses_agent_hub".to_string()),
        );
        obj.insert(
            "component".to_string(),
            Value::String("agent-hub".to_string()),
        );

        assert_eq!(
            record.get("kind").and_then(Value::as_str),
            Some(expected_kind),
            "member trace rows must preserve their event kind: {record:?}"
        );
        assert_eq!(
            record.get("participant_id").and_then(Value::as_str),
            Some("ash_member_live")
        );
        assert_eq!(
            record.get("parent_participant_id").and_then(Value::as_str),
            Some("ash_orchestrator_live")
        );
        assert_eq!(record.get("role").and_then(Value::as_str), Some("member"));
        assert_eq!(
            record.get("world_id").and_then(Value::as_str),
            Some("wld_trace_member_0001"),
            "world-scoped member trace rows must keep world_id at the top level: {record:?}"
        );
        assert_eq!(
            record.get("world_generation").and_then(Value::as_u64),
            Some(8),
            "world-scoped member trace rows must keep world_generation at the top level: {record:?}"
        );
    }
}

#[test]
fn replacement_member_trace_rows_preserve_lineage_while_terminal_predecessors_stay_auditable() {
    let mut predecessor_terminal = AgentEvent::message(
        "runtime-agent",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f18",
        MessageEventKind::TaskEnd,
        "old world member terminated before replacement",
    );
    predecessor_terminal.participant_id = Some("ash_member_old".to_string());
    predecessor_terminal.parent_participant_id = Some("ash_orchestrator_live".to_string());
    predecessor_terminal.role = Some("member".to_string());
    predecessor_terminal.world_id = Some("wld_trace_member_old".to_string());
    predecessor_terminal.world_generation = Some(7);

    let mut replacement_registered = AgentEvent::message(
        "runtime-agent",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f19",
        MessageEventKind::Registered,
        "replacement world member registered",
    );
    replacement_registered.participant_id = Some("ash_member_new".to_string());
    replacement_registered.parent_participant_id = Some("ash_orchestrator_live".to_string());
    replacement_registered.resumed_from_participant_id = Some("ash_member_old".to_string());
    replacement_registered.role = Some("member".to_string());
    replacement_registered.world_id = Some("wld_trace_member_new".to_string());
    replacement_registered.world_generation = Some(8);

    let mut terminal_record = predecessor_terminal
        .to_trace_record()
        .expect("flatten predecessor terminal event");
    let terminal_obj = terminal_record
        .as_object_mut()
        .expect("terminal trace record should be an object");
    terminal_obj.insert(
        "event_type".to_string(),
        Value::String("agent_event".to_string()),
    );
    terminal_obj.insert(
        "session_id".to_string(),
        Value::String("ses_agent_hub".to_string()),
    );
    terminal_obj.insert(
        "component".to_string(),
        Value::String("agent-hub".to_string()),
    );

    let mut replacement_record = replacement_registered
        .to_trace_record()
        .expect("flatten replacement registered event");
    let replacement_obj = replacement_record
        .as_object_mut()
        .expect("replacement trace record should be an object");
    replacement_obj.insert(
        "event_type".to_string(),
        Value::String("agent_event".to_string()),
    );
    replacement_obj.insert(
        "session_id".to_string(),
        Value::String("ses_agent_hub".to_string()),
    );
    replacement_obj.insert(
        "component".to_string(),
        Value::String("agent-hub".to_string()),
    );

    assert_eq!(
        terminal_record.get("kind").and_then(Value::as_str),
        Some("task_end"),
        "stale terminal rows must remain auditable as terminal events: {terminal_record:?}"
    );
    assert_eq!(
        terminal_record
            .get("participant_id")
            .and_then(Value::as_str),
        Some("ash_member_old")
    );
    assert_eq!(
        terminal_record.get("world_id").and_then(Value::as_str),
        Some("wld_trace_member_old")
    );
    assert!(
        terminal_record.get("resumed_from_participant_id").is_none(),
        "terminal predecessor rows must not be rewritten with replacement lineage: {terminal_record:?}"
    );

    assert_eq!(
        replacement_record.get("kind").and_then(Value::as_str),
        Some("registered")
    );
    assert_eq!(
        replacement_record
            .get("participant_id")
            .and_then(Value::as_str),
        Some("ash_member_new")
    );
    assert_eq!(
        replacement_record
            .get("resumed_from_participant_id")
            .and_then(Value::as_str),
        Some("ash_member_old"),
        "replacement trace rows must preserve resumed_from_participant_id lineage at the top level: {replacement_record:?}"
    );
    assert_eq!(
        replacement_record.get("world_id").and_then(Value::as_str),
        Some("wld_trace_member_new")
    );
    assert_eq!(
        replacement_record
            .get("world_generation")
            .and_then(Value::as_u64),
        Some(8)
    );
}

#[test]
fn world_alert_rows_omit_synthesized_participant_lineage_without_runtime_context() {
    let event = AgentEvent::alert(
        "shell",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
        "world_restart_required",
        "world restart required before continuing",
    );

    let mut record = event.to_trace_record().expect("flatten agent event");
    let obj = record
        .as_object_mut()
        .expect("agent event trace record should be an object");
    obj.insert(
        "event_type".to_string(),
        Value::String("agent_event".to_string()),
    );
    obj.insert(
        "session_id".to_string(),
        Value::String("ses_agent_hub".to_string()),
    );
    obj.insert(
        "component".to_string(),
        Value::String("agent-hub".to_string()),
    );

    assert!(
        record.get("participant_id").is_none(),
        "world alert rows without runtime context must not synthesize participant_id: {record:?}"
    );
    assert!(
        record.get("parent_participant_id").is_none(),
        "world alert rows without runtime context must not synthesize parent_participant_id: {record:?}"
    );
    assert!(
        record.get("resumed_from_participant_id").is_none(),
        "world alert rows without runtime context must not synthesize resumed_from_participant_id: {record:?}"
    );
}
