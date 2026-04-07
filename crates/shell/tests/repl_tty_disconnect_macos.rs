#![cfg(target_os = "macos")]

#[path = "support/mod.rs"]
mod support;

use serial_test::serial;
use std::ffi::CStr;
use std::fs;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
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

fn slave_tty_path(master_fd: i32) -> anyhow::Result<PathBuf> {
    let raw = unsafe { libc::ptsname(master_fd) };
    if raw.is_null() {
        anyhow::bail!(
            "failed to resolve slave tty path from master fd {master_fd}: {}",
            std::io::Error::last_os_error()
        );
    }

    let c_path = unsafe { CStr::from_ptr(raw) };
    Ok(PathBuf::from(c_path.to_string_lossy().into_owned()))
}

struct PtyRepl {
    child: Box<dyn portable_pty::Child + Send>,
    master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    waited: Option<portable_pty::ExitStatus>,
    _writer: Arc<Mutex<Box<dyn Write + Send>>>,
    output: Arc<Mutex<Vec<u8>>>,
    reader_handle: Option<std::thread::JoinHandle<()>>,
    stop_reader: Arc<AtomicBool>,
    saw_reedline_query: Arc<AtomicBool>,
    slave_tty_path: PathBuf,
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn revoke(path: *const libc::c_char) -> libc::c_int;
}

impl Drop for PtyRepl {
    fn drop(&mut self) {
        self.stop_reader.store(true, Ordering::Relaxed);
        self.master.take();

        if self.waited.is_none() {
            let _ = self.child.kill();
            let _ = self.child.try_wait().ok().flatten().map(|status| {
                self.waited = Some(status);
            });
        }

        if let Some(handle) = self.reader_handle.take() {
            let _ = handle.join();
        }
    }
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

        #[cfg(unix)]
        let slave_tty_path =
            slave_tty_path(master_fd.expect("master fd")).expect("resolve slave tty path");

        let mut cmd = CommandBuilder::new(binary_path());
        cmd.args(["--async-repl", "--no-world"]);
        cmd.cwd(project_dir);
        cmd.env("HOME", home_dir);
        cmd.env("USERPROFILE", home_dir);
        cmd.env("SUBSTRATE_HOME", substrate_home);
        cmd.env("SUBSTRATE_MANAGER_MANIFEST", manager_manifest_path());
        cmd.env("SUBSTRATE_CAGED", "0");
        cmd.arg("--uncaged");
        cmd.env("SHIM_TRACE_LOG", home_dir.join(".substrate/trace.jsonl"));
        cmd.env_remove("SUBSTRATE_WORLD_SOCKET");
        cmd.env("SUBSTRATE_OVERRIDE_WORLD", "disabled");
        cmd.env_remove("SUBSTRATE_WORLD");
        cmd.env_remove("SUBSTRATE_WORLD_ENABLED");
        cmd.env_remove("SUBSTRATE_WORLD_ID");
        cmd.env_remove("CI");
        cmd.env_remove("GITHUB_ACTIONS");
        cmd.env("SHELL", "/bin/bash");
        cmd.arg("--shim-skip");

        let child = pair.slave.spawn_command(cmd).expect("spawn substrate");
        let writer: Arc<Mutex<Box<dyn Write + Send>>> =
            Arc::new(Mutex::new(master.take_writer().expect("take writer")));

        let output = Arc::new(Mutex::new(Vec::new()));
        let stop_reader = Arc::new(AtomicBool::new(false));
        let saw_reedline_query = Arc::new(AtomicBool::new(false));
        let output_for_thread = output.clone();
        let writer_for_thread = writer.clone();
        let stop_for_thread = stop_reader.clone();
        let saw_reedline_query_for_thread = saw_reedline_query.clone();

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

                    if probe.windows(4).any(|w| w == b"\x1b[6n")
                        || probe.windows(5).any(|w| w == b"\x1b[18t")
                    {
                        saw_reedline_query_for_thread.store(true, Ordering::SeqCst);
                    }

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
            _writer: writer,
            output,
            reader_handle: Some(reader_handle),
            stop_reader,
            saw_reedline_query,
            slave_tty_path,
        }
    }

    fn output_string(&self) -> String {
        let guard = self.output.lock().expect("lock output");
        String::from_utf8_lossy(&guard).into_owned()
    }

    fn wait_for_reedline_probe(&self, timeout: Duration) -> anyhow::Result<()> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if self.saw_reedline_query.load(Ordering::SeqCst) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(25));
        }

        anyhow::bail!(
            "timed out waiting for Reedline cursor/query traffic; output so far:\n{}",
            self.output_string()
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

    fn wait_for_exit(&mut self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while self.waited.is_none() && Instant::now() < deadline {
            match self.try_wait() {
                Ok(true) => return true,
                Ok(false) => std::thread::sleep(Duration::from_millis(25)),
                Err(_) => break,
            }
        }
        self.try_wait().unwrap_or(false)
    }

    fn shutdown(mut self) -> (i32, String, bool) {
        let exited_in_time = self.wait_for_exit(Duration::from_secs(5));

        self.stop_reader.store(true, Ordering::Relaxed);
        self.master.take();

        if self.waited.is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait().expect("wait child after kill");
            let _ = self.try_wait();
        }

        if let Some(handle) = self.reader_handle.take() {
            let _ = handle.join();
        }

        let code = self
            .waited
            .as_ref()
            .map(|s| s.exit_code() as i32)
            .unwrap_or(-1);

        (code, self.output_string(), exited_in_time)
    }
}

fn revoke_tty(path: &Path) -> anyhow::Result<()> {
    use std::ffi::CString;

    let c_path = CString::new(path.as_os_str().as_bytes().to_vec())?;
    let rc = unsafe { revoke(c_path.as_ptr()) };
    if rc != 0 {
        anyhow::bail!(
            "revoke({}) failed: {}",
            path.display(),
            std::io::Error::last_os_error()
        );
    }
    Ok(())
}

#[test]
#[serial]
fn repl_revoke_disconnect_exits_1_on_reedline_path() {
    let temp = temp_dir("substrate-repl-tty-revoke-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");

    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
    write_profile(&project);
    write_policy(&substrate_home);

    let repl = PtyRepl::spawn(&project, &home, &substrate_home);
    repl.wait_for_reedline_probe(Duration::from_secs(3))
        .expect("expected Reedline prompt traffic before revoke");

    revoke_tty(&repl.slave_tty_path).expect("revoke slave tty");

    let (code, out, exited_in_time) = repl.shutdown();

    assert!(
        exited_in_time,
        "async REPL did not exit within the bounded timeout after revoke; output:\n{out}"
    );
    assert_eq!(
        code, 1,
        "expected abnormal terminal-loss exit code 1; output:\n{out}"
    );

    let diagnostic_lines: Vec<&str> = out
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("substrate: error:")
                || trimmed.starts_with("substrate: warning:")
                || trimmed.starts_with("prompt error:")
        })
        .collect();
    assert!(
        diagnostic_lines.len() <= 1,
        "expected at most one bounded abnormal-terminal-loss diagnostic line; output:\n{out}"
    );
    if let Some(line) = diagnostic_lines.first() {
        let lower = line.to_ascii_lowercase();
        assert!(
            lower.contains("abnormal terminal loss")
                || lower.contains("prompt error")
                || lower.contains("terminal"),
            "unexpected diagnostic line for revoke path: {line}"
        );
    }

    assert!(
        out.to_ascii_lowercase().contains("substrate"),
        "expected substrate output to be captured around the revoke path; output:\n{out}"
    );
}
