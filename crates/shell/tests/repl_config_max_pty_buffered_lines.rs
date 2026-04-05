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

impl PtyRepl {
    fn spawn(project_dir: &Path, home_dir: &Path, substrate_home: &Path) -> Self {
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
        cmd.env("SHIM_TRACE_LOG", substrate_home.join("trace.jsonl"));
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
                    let chunk = &buf[..n];
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

struct ReplConfigClampRun {
    _temp: tempfile::TempDir,
    code: i32,
    out: String,
    trace_path: PathBuf,
}

fn run_repl_with_global_config(config_yaml: &str) -> ReplConfigClampRun {
    let temp = temp_dir("substrate-or1-config-clamp-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(substrate_home.join("shims")).expect("create shims dir");
    fs::create_dir_all(&project).expect("create project dir");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(substrate_home.join("trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy(&substrate_home);
    fs::write(substrate_home.join("config.yaml"), config_yaml).expect("write config.yaml");

    let trace_path = substrate_home.join("trace.jsonl");
    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home);
    repl.wait_for_output_or_exit("Substrate v", Duration::from_secs(3))
        .expect("banner");
    repl.send_line("exit");
    let (code, out) = repl.shutdown_graceful(Duration::from_secs(3));
    ReplConfigClampRun {
        _temp: temp,
        code,
        out,
        trace_path,
    }
}

#[test]
#[serial]
fn out_of_range_config_is_clamped_and_emits_one_warning_record_high() {
    let run = run_repl_with_global_config(
        r#"world:
  enabled: true
policy:
  mode: observe
sync:
  auto_sync: false
repl:
  max_pty_buffered_lines: 99999
"#,
    );
    assert_eq!(run.code, 0, "expected clean exit; output:\n{}", run.out);

    let events = read_trace(&run.trace_path);
    let warnings: Vec<&Value> = events
        .iter()
        .filter(|event| event.get("event_type").and_then(Value::as_str) == Some("warning"))
        .filter(|event| event.get("code").and_then(Value::as_str) == Some("config_value_clamped"))
        .collect();
    assert_eq!(
        warnings.len(),
        1,
        "expected exactly one config_value_clamped warning record; got: {warnings:?}"
    );

    let warning = warnings[0];
    assert_eq!(
        warning.get("component").and_then(Value::as_str),
        Some("shell"),
        "warning record must have component=shell: {warning:?}"
    );
    assert_eq!(
        warning.get("key").and_then(Value::as_str),
        Some("repl.max_pty_buffered_lines"),
        "warning record must have key=repl.max_pty_buffered_lines: {warning:?}"
    );
    assert_eq!(
        warning.get("provided").and_then(Value::as_i64),
        Some(99999),
        "warning record must have provided=99999: {warning:?}"
    );
    assert_eq!(
        warning.get("effective").and_then(Value::as_i64),
        Some(16384),
        "warning record must have effective=16384: {warning:?}"
    );
    assert_eq!(
        warning.get("min").and_then(Value::as_i64),
        Some(0),
        "warning record must have min=0: {warning:?}"
    );
    assert_eq!(
        warning.get("max").and_then(Value::as_i64),
        Some(16384),
        "warning record must have max=16384: {warning:?}"
    );
}

#[test]
#[serial]
fn out_of_range_config_is_clamped_and_emits_one_warning_record_low() {
    let run = run_repl_with_global_config(
        r#"world:
  enabled: true
policy:
  mode: observe
sync:
  auto_sync: false
repl:
  max_pty_buffered_lines: -5
"#,
    );
    assert_eq!(run.code, 0, "expected clean exit; output:\n{}", run.out);

    let events = read_trace(&run.trace_path);
    let warnings: Vec<&Value> = events
        .iter()
        .filter(|event| event.get("event_type").and_then(Value::as_str) == Some("warning"))
        .filter(|event| event.get("code").and_then(Value::as_str) == Some("config_value_clamped"))
        .collect();
    assert_eq!(
        warnings.len(),
        1,
        "expected exactly one config_value_clamped warning record; got: {warnings:?}"
    );

    let warning = warnings[0];
    assert_eq!(
        warning.get("provided").and_then(Value::as_i64),
        Some(-5),
        "warning record must have provided=-5: {warning:?}"
    );
    assert_eq!(
        warning.get("effective").and_then(Value::as_i64),
        Some(0),
        "warning record must have effective=0: {warning:?}"
    );
}
