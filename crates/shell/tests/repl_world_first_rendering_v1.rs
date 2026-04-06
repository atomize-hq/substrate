#![cfg(any(target_os = "linux", target_os = "macos"))]

#[path = "support/mod.rs"]
mod support;

use serial_test::serial;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use support::{binary_path, ensure_substrate_built, temp_dir, ReplWorldAgentStub};
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

                // If we can't get an FD for non-blocking reads (unexpected on unix), don't
                // busy-loop. We'll rely on teardown to kill the child and let the thread exit.
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
            let _ = w.write_all(b"\n");
            let _ = w.flush();
        }
    }

    fn send_bytes(&mut self, bytes: &[u8]) {
        if let Ok(mut w) = self.writer.lock() {
            let _ = w.write_all(bytes);
            let _ = w.flush();
        }
    }

    fn output_bytes(&self) -> Vec<u8> {
        self.output.lock().expect("lock output").clone()
    }

    fn output_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.output_bytes()).into_owned()
    }

    fn wait_for_output(&self, needle: &str, timeout: Duration) -> anyhow::Result<()> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            let out = self.output_string_lossy();
            if out.contains(needle) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        anyhow::bail!(
            "timed out waiting for output containing `{}`; output so far:\n{}",
            needle,
            self.output_string_lossy()
        )
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

    fn shutdown_graceful(mut self, timeout: Duration) -> (i32, Vec<u8>) {
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

    fn shutdown(mut self) -> (i32, Vec<u8>) {
        self.stop_reader.store(true, Ordering::Relaxed);
        // Dropping the master PTY handle helps ensure any in-flight read unblocks cleanly.
        self.master.take();
        if self.waited.is_none() {
            let _ = self.child.kill();
            let deadline = Instant::now() + Duration::from_secs(2);
            while Instant::now() < deadline {
                match self.child.try_wait() {
                    Ok(Some(status)) => {
                        self.waited = Some(status);
                        break;
                    }
                    Ok(None) => std::thread::sleep(Duration::from_millis(25)),
                    Err(_) => break,
                }
            }
        }
        if let Some(handle) = self.reader_handle.take() {
            // Never spawn a detached "joiner" thread here: if the join blocks, it will keep the
            // whole test binary alive and appear as a CI hang after the last `... ok`.
            let _ = handle.join();
        }
        let code = self
            .waited
            .as_ref()
            .map(|s| s.exit_code() as i32)
            .unwrap_or(-1);
        (code, self.output_bytes())
    }
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}

fn find_substring(haystack: &str, needle: &str) -> Option<usize> {
    haystack.find(needle)
}

#[test]
#[serial]
fn c4_repl_renders_non_utf8_bytes_and_continues() {
    let temp = temp_dir("substrate-c4-rendering-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy(&substrate_home);

    let sock_temp = short_socket_dir("sub-c4ws-render-");
    let sock = sock_temp.path().join("world.sock");

    // Include invalid UTF-8 bytes. The REPL must forward them to the PTY without panicking,
    // then continue accepting commands.
    let marker = "__C4_RENDER__";
    let mut payload = Vec::new();
    payload.extend_from_slice(marker.as_bytes());
    payload.extend_from_slice(b" bin=");
    payload.extend_from_slice(&[0xFF, 0xFE, 0xFD]);
    payload.extend_from_slice(b"\n");

    let _server =
        ReplWorldAgentStub::start_with_persistent_exec_stdout_override(&sock, marker, payload);

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &["--world"]);

    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");

    repl.send_line(&format!("echo {marker}"));
    // Avoid matching the echoed input line (`echo __C4_RENDER__`) and instead wait for the actual
    // command output.
    repl.wait_for_output(&format!("{marker} bin="), Duration::from_secs(3))
        .expect("marker output");

    repl.send_line("echo after");
    repl.wait_for_output("after", Duration::from_secs(3))
        .expect("followup output");

    repl.send_line("exit");

    let (code, out) = repl.shutdown_graceful(Duration::from_secs(3));
    assert_eq!(
        code,
        0,
        "expected clean exit; output:\n{}",
        String::from_utf8_lossy(&out)
    );

    assert!(
        contains_subslice(&out, &[0xFF, 0xFE, 0xFD]),
        "expected raw non-UTF8 bytes to reach PTY; output (lossy):\n{}",
        String::from_utf8_lossy(&out)
    );
}

#[test]
#[serial]
fn c4_out_of_band_stdout_renders_while_idle_and_input_submission_survives() {
    let temp = temp_dir("substrate-c4-oob-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy(&substrate_home);

    let sock_temp = short_socket_dir("sub-c4ws-oob-");
    let sock = sock_temp.path().join("world.sock");

    let trigger = "__C4_OOB_TRIGGER__";
    let oob = "__C4_OOB_BYTES__\n";
    let script = support::PersistentExecStdoutOverride {
        marker: trigger.to_string(),
        bytes: format!("{trigger}\n").into_bytes(),
        suffix_bytes: None,
        delay_before_suffix_ms: None,
        out_of_band_after_complete: Some((800, oob.as_bytes().to_vec())),
    };

    let server = ReplWorldAgentStub::start_with_persistent_exec_script(&sock, script);
    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &["--world"]);

    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");

    repl.send_line(&format!("echo {trigger}"));
    repl.wait_for_output(trigger, Duration::from_secs(3))
        .expect("trigger output");

    // Out-of-band bytes should render while idle (Reedline active) and the REPL should remain usable.
    repl.wait_for_output("__C4_OOB_BYTES__", Duration::from_secs(3))
        .expect("oob stdout rendered");

    repl.send_line("echo __C4_INPUT_OK__");
    repl.wait_for_output("__C4_INPUT_OK__", Duration::from_secs(3))
        .expect("input submission survived");

    // Also assert the stub saw the intact submission.
    let records = server.records();
    let guard = records.lock().expect("lock records");
    assert!(
        guard
            .persistent_execs
            .iter()
            .any(|e| e.program_utf8.trim() == "echo __C4_INPUT_OK__"),
        "expected intact submission in exec records; execs: {:?}",
        guard
            .persistent_execs
            .iter()
            .map(|e| e.program_utf8.trim().to_string())
            .collect::<Vec<_>>()
    );

    repl.send_line("exit");
    let (code, out) = repl.shutdown_graceful(Duration::from_secs(3));
    assert_eq!(
        code,
        0,
        "expected clean exit; output:\n{}",
        String::from_utf8_lossy(&out)
    );
}

#[test]
#[serial]
fn c4_pty_passthrough_forwards_raw_bytes_and_buffers_structured_events() {
    let temp = temp_dir("substrate-c4-pty-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy(&substrate_home);

    let sock_temp = short_socket_dir("sub-c4ws-pty-");
    let sock = sock_temp.path().join("world.sock");

    let marker = "__C4_PTY__";
    let prefix = format!("BEGIN {marker}\n");
    let suffix = format!("END {marker}\n");
    let script = support::PersistentExecStdoutOverride {
        marker: marker.to_string(),
        bytes: prefix.into_bytes(),
        suffix_bytes: Some(suffix.clone().into_bytes()),
        delay_before_suffix_ms: Some(3000),
        out_of_band_after_complete: None,
    };

    let server = ReplWorldAgentStub::start_with_persistent_exec_script(&sock, script);
    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &["--world"]);

    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("banner");

    // Schedule structured agent events that fire while PTY passthrough is active. Use
    // `:demo-burst` here because it prints an acknowledgement synchronously, which gives the
    // test a deterministic readiness point before we submit the PTY command.
    repl.send_line(":demo-burst 1 3 1500");
    repl.wait_for_output(
        "scheduled burst: agents=1, events_per_agent=3, delay_ms=1500",
        Duration::from_secs(3),
    )
    .expect("demo burst scheduled");

    repl.send_line(&format!(":pty echo {marker}"));
    repl.wait_for_output(&format!("BEGIN {marker}"), Duration::from_secs(8))
        .expect("pty prefix");

    // While PTY passthrough is active, typed Ctrl+C must be forwarded as byte 0x03 (not as a signal).
    repl.send_bytes(b"x");
    repl.send_bytes(&[0x03]);

    repl.wait_for_output(&format!("END {marker}"), Duration::from_secs(8))
        .expect("pty suffix");

    // Structured agent output should be buffered during passthrough and appear after the PTY command ends.
    repl.wait_for_output("chunk #00001", Duration::from_secs(5))
        .expect("burst event");

    // Give any additional buffered output a moment to flush, then exit.
    std::thread::sleep(Duration::from_millis(150));
    repl.send_line("exit");

    let (_code, out_bytes) = repl.shutdown_graceful(Duration::from_secs(3));
    let out = String::from_utf8_lossy(&out_bytes).into_owned();

    let end_idx =
        find_substring(&out, &format!("END {marker}")).expect("expected END marker in output");
    let ev1 = find_substring(&out, "chunk #00001");
    assert!(
        ev1.is_some(),
        "expected burst event to appear somewhere in output (buffered or not); output:\n{out}"
    );

    // C4 contract: during PTY passthrough, structured host output SHOULD be buffered and rendered
    // only after the foreground command completes.
    assert!(
        ev1.unwrap() > end_idx,
        "expected demo events to render after PTY passthrough; output:\n{out}"
    );

    // Assert raw stdin bytes were forwarded (including 0x03) and were not translated into a signal frame.
    let records = server.records();
    let guard = records.lock().expect("lock records");
    let stdin_concat: Vec<u8> = guard.persistent_stdin.concat();
    assert!(
        stdin_concat.contains(&0x03),
        "expected forwarded stdin to contain 0x03; got: {stdin_concat:?}"
    );
    assert!(
        guard.persistent_signals.is_empty(),
        "expected no signal frames from typed 0x03; signals: {:?}",
        guard.persistent_signals
    );
}
