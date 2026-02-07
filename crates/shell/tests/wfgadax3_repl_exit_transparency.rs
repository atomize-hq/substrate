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

fn write_workspace_patch(workspace_root: &Path, patch: &str) {
    let dir = workspace_root.join(".substrate");
    fs::create_dir_all(&dir).expect("create .substrate");
    fs::write(dir.join("workspace.yaml"), patch).expect("write workspace.yaml");
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
                    "process exited before seeing `{needle}` (code={}); output so far:\n{}",
                    self.waited
                        .as_ref()
                        .map(|s| s.exit_code() as i32)
                        .unwrap_or(-1),
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

fn find_exit_note_path(output: &str) -> Option<String> {
    let prefix = "substrate: note: returning to host cwd: ";
    output.lines().find_map(|line| {
        let trimmed = line.trim_end_matches('\r');
        trimmed
            .strip_prefix(prefix)
            .map(|rest| rest.trim().to_string())
    })
}

#[test]
#[serial]
fn wfgadax3_prints_exit_note_when_world_cwd_differs_default_target_is_entered_cwd() {
    let temp = temp_dir("substrate-wfgadax3-entered-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    let exit_target = project.join("exit-target");

    fs::create_dir_all(home.join(".substrate")).expect("create home/.substrate");
    fs::create_dir_all(&exit_target).expect("create exit-target");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    write_profile(&project);
    write_policy(&substrate_home);

    let sock_temp = short_socket_dir("sub-wfgadax3-entered-");
    let sock = sock_temp.path().join("world.sock");
    let _server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output_or_exit("Substrate v", Duration::from_secs(2))
        .expect("banner");

    repl.send_line("cd exit-target");
    repl.wait_for_output_or_exit(
        "__PERSISTENT_EXEC_STUB__ eof cd exit-target",
        Duration::from_secs(3),
    )
    .expect("cd executed");

    repl.send_line("exit");
    let (code, out) = repl.shutdown_graceful(Duration::from_secs(3));
    assert_eq!(code, 0, "expected clean exit; output:\n{out}");

    let note = find_exit_note_path(&out).unwrap_or_else(|| {
        panic!("expected REPL to print exit note when world_cwd != entered_cwd; output:\n{out}")
    });

    let expected = project.canonicalize().unwrap_or(project);
    let actual = PathBuf::from(note)
        .canonicalize()
        .expect("note path should exist");
    assert_eq!(
        actual, expected,
        "expected exit note path to be entered_cwd"
    );
}

#[test]
#[serial]
fn wfgadax3_repl_exit_cwd_last_world_selects_world_cwd_as_exit_target_when_representable() {
    let temp = temp_dir("substrate-wfgadax3-last-world-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    let exit_target = project.join("exit-target");

    fs::create_dir_all(home.join(".substrate")).expect("create home/.substrate");
    fs::create_dir_all(&exit_target).expect("create exit-target");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    write_profile(&project);
    write_policy(&substrate_home);
    write_workspace_patch(&project, "repl:\n  exit_cwd: last_world\n");

    let sock_temp = short_socket_dir("sub-wfgadax3-last-world-");
    let sock = sock_temp.path().join("world.sock");
    let _server = ReplWorldAgentStub::start(&sock, StreamBehavior::Normal);

    let mut repl = PtyRepl::spawn(&project, &home, &substrate_home, &sock, &[], &["--world"]);
    repl.wait_for_output_or_exit("Substrate v", Duration::from_secs(2))
        .expect("banner");

    repl.send_line("cd exit-target");
    repl.wait_for_output_or_exit(
        "__PERSISTENT_EXEC_STUB__ eof cd exit-target",
        Duration::from_secs(3),
    )
    .expect("cd executed");

    repl.send_line("exit");
    let (code, out) = repl.shutdown_graceful(Duration::from_secs(3));
    assert_eq!(code, 0, "expected clean exit; output:\n{out}");

    let note = find_exit_note_path(&out).unwrap_or_else(|| {
        panic!("expected REPL to print exit note when world_cwd != entered_cwd; output:\n{out}")
    });

    let expected = exit_target.canonicalize().unwrap_or(exit_target);
    let actual = PathBuf::from(note)
        .canonicalize()
        .expect("note path should exist");
    assert_eq!(
        actual, expected,
        "expected exit note path to be last_world cwd"
    );
}

#[test]
fn wfgadax3_repl_exit_cwd_rejects_invalid_modes_as_hard_error_exit_2() {
    ensure_substrate_built();

    let temp = temp_dir("substrate-wfgadax3-invalid-mode-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = temp.path().join("substrate-home");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(temp.path().join("trace.jsonl"), "").expect("seed trace");

    write_policy(&substrate_home);
    fs::write(
        substrate_home.join("config.yaml"),
        "repl:\n  exit_cwd: not-a-mode\n",
    )
    .expect("write config.yaml");

    let marker = "__WFGADAX3_INVALID_MODE_MARKER__";
    let mut cmd = assert_cmd::Command::new(binary_path());
    let assert = cmd
        .current_dir(&project)
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", &substrate_home)
        .env("SUBSTRATE_MANAGER_MANIFEST", manager_manifest_path())
        .env("SHIM_TRACE_LOG", temp.path().join("trace.jsonl"))
        .env("SUBSTRATE_OVERRIDE_WORLD", "disabled")
        .env("SUBSTRATE_CAGED", "0")
        .arg("--uncaged")
        .arg("--shim-skip")
        .arg("-c")
        .arg(format!("printf {marker}"))
        .assert()
        .code(2);

    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains(marker),
        "expected hard error to occur before execution; stdout:\n{stdout}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("repl.exit_cwd")
            && stderr.contains("entered")
            && stderr.contains("last_world"),
        "expected stderr to mention repl.exit_cwd and supported modes; stderr:\n{stderr}"
    );
}
