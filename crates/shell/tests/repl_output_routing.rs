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
use support::ReplWorldAgentStub;
use support::{binary_path, ensure_substrate_built, temp_dir, PersistentExecStdoutOverride};
use tempfile::TempDir;

const PTY_MARKER: &str = "OR1_PTY_MARKER";
const PTY_START: &str = "__OR1_PTY_START__";
const PTY_END: &str = "__OR1_PTY_END__";
const DEMO_BURST_ACK: &str = "scheduled burst: agents=1, events_per_agent=3, delay_ms=1000";
const DEMO_BURST_EVENT_1: &str = "chunk #00001";
const DEMO_BURST_EVENT_2: &str = "chunk #00002";

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

fn write_workspace_marker(workspace_root: &Path, extra_workspace_yaml: Option<&str>) {
    let dir = workspace_root.join(".substrate");
    fs::create_dir_all(&dir).expect("create .substrate");
    let base = r#"world:
  enabled: true
  anchor_mode: workspace
  anchor_path: ''
  caged: false
"#;
    let contents = match extra_workspace_yaml {
        Some(extra) => format!("{base}{extra}"),
        None => base.to_string(),
    };
    fs::write(dir.join("workspace.yaml"), contents).expect("write workspace.yaml");
}

fn short_socket_dir(prefix: &str) -> TempDir {
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in("/tmp")
        .expect("create short socket tempdir in /tmp")
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
        socket_path: &Path,
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
        cmd.arg("--async-repl");
        cmd.arg("--world");
        cmd.arg("--shim-skip");

        let child = pair.slave.spawn_command(cmd).expect("spawn substrate");
        let writer: Arc<Mutex<Box<dyn Write + Send>>> =
            Arc::new(Mutex::new(master.take_writer().expect("take writer")));

        let output = Arc::new(Mutex::new(Vec::new()));
        let stop_reader = Arc::new(AtomicBool::new(false));
        let output_for_thread = output.clone();
        let writer_for_thread = Arc::downgrade(&writer);
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
        if let Ok(mut w) = self.writer.lock() {
            let _ = w.write_all(line.as_bytes());
            let _ = w.write_all(b"\r");
            let _ = w.flush();
        }
    }

    fn output_string(&self) -> String {
        let guard = self.output.lock().expect("lock output");
        String::from_utf8_lossy(&guard).into_owned()
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

    fn wait_for_output_or_exit(&mut self, needle: &str, timeout: Duration) -> anyhow::Result<()> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if self.output_string().contains(needle) {
                return Ok(());
            }
            if self.try_wait()? {
                anyhow::bail!(
                    "process exited while waiting for `{needle}`; output so far:\n{}",
                    self.output_string()
                );
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        anyhow::bail!(
            "timed out waiting for output containing `{needle}`; output so far:\n{}",
            self.output_string()
        )
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

fn read_trace(trace_path: &Path) -> Vec<Value> {
    fs::read_to_string(trace_path)
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("parse trace line"))
        .collect()
}

#[test]
#[serial]
fn structured_events_are_deferred_until_after_pty_passthrough() {
    let temp = temp_dir("substrate-or1-output-routing-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(substrate_home.join("shims")).expect("create shims dir");
    fs::create_dir_all(&project).expect("create project dir");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(substrate_home.join("trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy(&substrate_home);
    write_workspace_marker(&project, None);

    let sock_temp = short_socket_dir("sub-or1-routing-");
    let sock = sock_temp.path().join("world.sock");
    let _server = ReplWorldAgentStub::start_with_persistent_exec_script(
        &sock,
        PersistentExecStdoutOverride {
            marker: PTY_MARKER.to_string(),
            bytes: format!("{PTY_START}\n").into_bytes(),
            suffix_bytes: Some(format!("{PTY_END}\n").into_bytes()),
            delay_before_suffix_ms: Some(2500),
            out_of_band_after_complete: None,
        },
    );

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock);
    repl.wait_for_output_or_exit("Substrate v", Duration::from_secs(3))
        .expect("banner");

    repl.send_line(":demo-burst 1 3 1000");
    repl.wait_for_output_or_exit(DEMO_BURST_ACK, Duration::from_secs(3))
        .expect("demo burst scheduled");
    repl.send_line(&format!(":pty echo {PTY_MARKER}"));

    repl.wait_for_output_or_exit(PTY_START, Duration::from_secs(2))
        .expect("pty start marker");
    repl.wait_for_output_or_exit(PTY_END, Duration::from_secs(5))
        .expect("pty end marker");
    repl.wait_for_output_or_exit(DEMO_BURST_EVENT_1, Duration::from_secs(3))
        .expect("deferred agent output");

    repl.send_line("exit");
    let (code, out) = repl.shutdown_graceful(Duration::from_secs(3));
    assert_eq!(code, 0, "expected clean exit; output:\n{out}");

    let start_idx = out.find(PTY_START).expect("pty start marker index");
    let end_idx = out.find(PTY_END).expect("pty end marker index");
    let first_event_idx = out.find(DEMO_BURST_EVENT_1).expect("agent event output");
    assert!(
        first_event_idx > end_idx,
        "expected structured agent output after PTY passthrough end; output:\n{out}"
    );

    assert!(
        !out[start_idx..end_idx].contains("chunk #"),
        "structured agent output must not be printed during PTY passthrough; output:\n{out}"
    );
}

#[test]
#[serial]
fn max_pty_buffered_lines_zero_drops_structured_lines_and_emits_one_warning_record() {
    let temp = temp_dir("substrate-or1-output-routing-cap0-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(substrate_home.join("shims")).expect("create shims dir");
    fs::create_dir_all(&project).expect("create project dir");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(substrate_home.join("trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy(&substrate_home);
    write_workspace_marker(
        &project,
        Some(
            r#"repl:
  max_pty_buffered_lines: 0
"#,
        ),
    );

    let sock_temp = short_socket_dir("sub-or1-routing-cap0-");
    let sock = sock_temp.path().join("world.sock");
    let _server = ReplWorldAgentStub::start_with_persistent_exec_script(
        &sock,
        PersistentExecStdoutOverride {
            marker: PTY_MARKER.to_string(),
            bytes: format!("{PTY_START}\n").into_bytes(),
            suffix_bytes: Some(format!("{PTY_END}\n").into_bytes()),
            delay_before_suffix_ms: Some(2500),
            out_of_band_after_complete: None,
        },
    );

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock);
    repl.wait_for_output_or_exit("Substrate v", Duration::from_secs(3))
        .expect("banner");

    repl.send_line(":demo-burst 1 3 1000");
    repl.wait_for_output_or_exit(DEMO_BURST_ACK, Duration::from_secs(3))
        .expect("demo burst scheduled");
    repl.send_line(&format!(":pty echo {PTY_MARKER}"));

    repl.wait_for_output_or_exit(PTY_END, Duration::from_secs(6))
        .expect("pty end marker");
    std::thread::sleep(Duration::from_millis(200));
    repl.send_line("exit");

    let (code, out) = repl.shutdown_graceful(Duration::from_secs(3));
    assert_eq!(code, 0, "expected clean exit; output:\n{out}");

    let start_idx = out.find(PTY_START).expect("pty start marker index");
    let end_idx = out.find(PTY_END).expect("pty end marker index");
    assert!(
        !out[start_idx..end_idx].contains("chunk #"),
        "structured agent output must not be printed during PTY passthrough; output:\n{out}"
    );

    assert!(
        !out.contains(DEMO_BURST_EVENT_1) && !out.contains(DEMO_BURST_EVENT_2),
        "expected no buffered structured lines when cap=0; output:\n{out}"
    );

    let trace_path = substrate_home.join("trace.jsonl");
    let events = read_trace(&trace_path);
    let burst_events: Vec<&Value> = events
        .iter()
        .filter(|event| event.get("event_type").and_then(Value::as_str) == Some("agent_event"))
        .filter(|event| event.get("agent_id").and_then(Value::as_str) == Some("burst-00"))
        .collect();
    assert!(
        burst_events.len() >= 3,
        "burst-00 should emit at least 3 agent_event records even when display is suppressed; got {} records: {burst_events:?}",
        burst_events.len()
    );

    let warnings: Vec<&Value> = events
        .iter()
        .filter(|event| event.get("event_type").and_then(Value::as_str) == Some("warning"))
        .filter(|event| {
            event.get("code").and_then(Value::as_str) == Some("pty_structured_event_drops")
        })
        .collect();
    assert_eq!(
        warnings.len(),
        1,
        "expected exactly one pty_structured_event_drops warning record; got {warnings:?}"
    );

    let warning = warnings[0];
    assert_eq!(
        warning.get("component").and_then(Value::as_str),
        Some("shell"),
        "warning record must have component=shell: {warning:?}"
    );
    assert_eq!(
        warning
            .get("max_pty_buffered_lines")
            .and_then(Value::as_i64),
        Some(0),
        "warning record must report effective max_pty_buffered_lines=0: {warning:?}"
    );
    assert_eq!(
        warning
            .get("dropped_structured_event_lines")
            .and_then(Value::as_i64),
        Some(2),
        "warning record must report dropped_structured_event_lines=2 (chunk #00001/#00002): {warning:?}"
    );
}
