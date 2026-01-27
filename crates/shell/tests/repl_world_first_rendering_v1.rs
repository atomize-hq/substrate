#![cfg(any(target_os = "linux", target_os = "macos"))]

#[path = "support/mod.rs"]
mod support;

use serial_test::serial;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use support::{binary_path, ensure_substrate_built, temp_dir, ReplWorldAgentStub};
use tempfile::TempDir;

fn manager_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/manager_hooks.yaml")
}

fn write_profile(project_dir: &Path) {
    let profile = r#"id: test-policy
name: Test Policy
world_fs:
  mode: writable
  isolation: workspace
  require_world: true
  read_allowlist:
    - "*"
  write_allowlist: []
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
  mode: writable
  isolation: workspace
  require_world: true
  read_allowlist:
    - "*"
  write_allowlist: []
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
    waited: Option<portable_pty::ExitStatus>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    output: Arc<Mutex<Vec<u8>>>,
    reader_handle: Option<std::thread::JoinHandle<()>>,
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
        let mut reader = pair.master.try_clone_reader().expect("clone reader");
        let writer: Arc<Mutex<Box<dyn Write + Send>>> =
            Arc::new(Mutex::new(pair.master.take_writer().expect("take writer")));

        let output = Arc::new(Mutex::new(Vec::new()));
        let output_for_thread = output.clone();
        let writer_for_thread = writer.clone();
        let reader_handle = std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = &buf[..n];
                        if chunk.windows(4).any(|w| w == b"\x1b[6n") {
                            if let Ok(mut w) = writer_for_thread.lock() {
                                let _ = w.write_all(b"\x1b[1;1R");
                                let _ = w.flush();
                            }
                        }
                        if chunk.windows(5).any(|w| w == b"\x1b[18t") {
                            if let Ok(mut w) = writer_for_thread.lock() {
                                let _ = w.write_all(b"\x1b[8;24;80t");
                                let _ = w.flush();
                            }
                        }

                        if let Ok(mut guard) = output_for_thread.lock() {
                            guard.extend_from_slice(&buf[..n]);
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            child,
            waited: None,
            writer,
            output,
            reader_handle: Some(reader_handle),
        }
    }

    fn send_line(&mut self, line: &str) {
        if let Ok(mut w) = self.writer.lock() {
            let _ = w.write_all(line.as_bytes());
            let _ = w.write_all(b"\r");
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
        (code, self.output_bytes())
    }
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
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
    repl.wait_for_output(marker, Duration::from_secs(3))
        .expect("marker output");

    repl.send_line("echo after");
    repl.wait_for_output("after", Duration::from_secs(3))
        .expect("followup output");

    repl.send_line("exit");

    let (code, out) = repl.shutdown_graceful(Duration::from_secs(3));
    assert_eq!(code, 0, "expected clean exit; output:\n{}", String::from_utf8_lossy(&out));

    assert!(
        contains_subslice(&out, &[0xFF, 0xFE, 0xFD]),
        "expected raw non-UTF8 bytes to reach PTY; output (lossy):\n{}",
        String::from_utf8_lossy(&out)
    );
}

