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
use support::{binary_path, ensure_substrate_built, temp_dir, ReplWorldAgentStub, StreamBehavior};
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

fn write_policy(home_substrate: &Path, require_world: bool) {
    fs::create_dir_all(home_substrate).expect("create SUBSTRATE_HOME");
    let require_world = if require_world { "true" } else { "false" };
    let policy = format!(
        r#"id: test-global-policy
name: Test Global Policy
world_fs:
  mode: writable
  isolation: workspace
  require_world: {require_world}
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
metadata: {{}}
"#
    );
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
                        // Reedline/crossterm may emit terminal queries which require a response
                        // from the terminal emulator. When running under a raw PTY in tests,
                        // provide minimal responses so the REPL can make progress.
                        //
                        // - DSR (cursor position): ESC [ 6 n  →  ESC [ 1 ; 1 R
                        // - Window size request:  ESC [ 18 t →  ESC [ 8 ; rows ; cols t
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
            // Use CR for Enter; many line editors expect this in terminal mode.
            let _ = w.write_all(b"\r");
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

    repl.send_line(":host pwd");
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

    repl.send_line(":host pwd");
    repl.send_line("exit");

    let (_code, out) = repl.shutdown_graceful(Duration::from_secs(2));
    let project = fs::canonicalize(&project).unwrap_or(project);
    let project_str = project.to_string_lossy();
    assert!(
        out.contains(project_str.as_ref()),
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
fn c3_drift_restart_restarts_session_and_emits_message() {
    let temp = temp_dir("substrate-c3-drift-restart-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&substrate_home).expect("create substrate home");
    fs::write(home.join(".substrate/trace.jsonl"), "").expect("seed trace");
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
        "expected a session restart on snapshot/workspace drift; records: {guard:#?}"
    );

    let lower = out.to_ascii_lowercase();
    assert!(
        lower.contains("restart") && (lower.contains("drift") || lower.contains("snapshot")),
        "expected an operator-visible drift-restart message, got output:\n{out}"
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
