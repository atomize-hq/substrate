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
                    if chunk.windows(4).any(|w| w == b"\x1b[6n") {
                        if let Some(writer) = writer_for_thread.upgrade() {
                            if let Ok(mut w) = writer.lock() {
                                let _ = w.write_all(b"\x1b[1;1R");
                                let _ = w.flush();
                            }
                        }
                    }
                    if chunk.windows(5).any(|w| w == b"\x1b[18t") {
                        if let Some(writer) = writer_for_thread.upgrade() {
                            if let Ok(mut w) = writer.lock() {
                                let _ = w.write_all(b"\x1b[8;24;80t");
                                let _ = w.flush();
                            }
                        }
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
        writer.write_all(b"\r").expect("write CR");
        writer.flush().expect("flush");
    }

    fn wait_for_output(&self, needle: &str, timeout: Duration) -> Option<usize> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            let out = self.output.lock().expect("output lock");
            if let Ok(text) = std::str::from_utf8(&out) {
                if let Some(pos) = text.find(needle) {
                    return Some(pos);
                }
            }
            drop(out);
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
