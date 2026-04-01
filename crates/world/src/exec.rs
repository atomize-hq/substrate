//! Helpers for executing commands with incremental streaming callbacks.

use crate::stream::{emit_chunk, StreamKind};
use anyhow::{anyhow, Context, Result};
#[cfg(target_os = "linux")]
use chrono::{SecondsFormat, Utc};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread;
use substrate_common::ProcessTelemetry;
use tracing::warn;
use world_api::WorldFsMode;

#[cfg(target_os = "linux")]
use std::sync::{Mutex, OnceLock};
#[cfg(target_os = "linux")]
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
#[cfg(target_os = "linux")]
use substrate_common::{ProcessEvent, ProcessEventType, ProcessEventsStatus};

#[cfg(target_os = "linux")]
fn active_exec_registry() -> &'static Mutex<HashMap<String, i32>> {
    static ACTIVE_EXEC_REGISTRY: OnceLock<Mutex<HashMap<String, i32>>> = OnceLock::new();
    ACTIVE_EXEC_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

const PROCESS_EVENTS_DEFAULT_MAX: usize = 10_000;

#[cfg(target_os = "linux")]
const PROCESS_EVENTS_BACKEND: &str = "world-linux-ptrace";

const SUBSTRATE_PARENT_SPAN_ENV: &str = "SUBSTRATE_PARENT_SPAN_ID";

#[derive(Clone, Debug)]
pub struct ProcessCaptureSpec {
    pub session_id: String,
    pub world_id: String,
    pub parent_span: String,
    pub parent_cmd_id: Option<String>,
    pub max_events: usize,
}

impl ProcessCaptureSpec {
    pub fn from_env(world_id: &str, env: &HashMap<String, String>, span_id: Option<&str>) -> Self {
        let session_id = env
            .get("SHIM_SESSION_ID")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let parent_span = env
            .get(SUBSTRATE_PARENT_SPAN_ENV)
            .cloned()
            .or_else(|| span_id.map(ToOwned::to_owned))
            .unwrap_or_else(|| "unknown".to_string());

        Self {
            session_id,
            world_id: world_id.to_string(),
            parent_span,
            parent_cmd_id: env.get("SHIM_PARENT_CMD_ID").cloned(),
            max_events: PROCESS_EVENTS_DEFAULT_MAX,
        }
    }
}

pub struct CapturedCommandOutput {
    pub output: Output,
    pub process_telemetry: ProcessTelemetry,
}

#[derive(Clone, Copy)]
pub(crate) struct CommandCapture<'a> {
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    span_id: Option<&'a str>,
    process_capture: Option<&'a ProcessCaptureSpec>,
}

impl<'a> CommandCapture<'a> {
    pub(crate) const fn new(
        span_id: Option<&'a str>,
        process_capture: Option<&'a ProcessCaptureSpec>,
    ) -> Self {
        Self {
            span_id,
            process_capture,
        }
    }
}

#[cfg(target_os = "linux")]
pub struct TracedProcessTreeResult {
    pub raw_exit_status: i32,
    pub process_telemetry: ProcessTelemetry,
}

#[cfg(target_os = "linux")]
fn parse_signal_name(sig: &str) -> Option<libc::c_int> {
    match sig.trim().to_ascii_uppercase().as_str() {
        "INT" | "SIGINT" => Some(libc::SIGINT),
        "TERM" | "SIGTERM" => Some(libc::SIGTERM),
        "HUP" | "SIGHUP" => Some(libc::SIGHUP),
        "QUIT" | "SIGQUIT" => Some(libc::SIGQUIT),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
pub fn note_pending_exec(span_id: &str) {
    let mut registry = active_exec_registry()
        .lock()
        .expect("active exec registry lock poisoned");
    registry.entry(span_id.to_string()).or_insert(0);
}

#[cfg(target_os = "linux")]
pub fn clear_registered_exec(span_id: &str) {
    if let Ok(mut registry) = active_exec_registry().lock() {
        registry.remove(span_id);
    }
}

#[cfg(target_os = "linux")]
fn register_active_exec(span_id: Option<&str>, pgid: i32) -> Option<ActiveExecRegistration> {
    let span_id = span_id?;
    let mut registry = active_exec_registry()
        .lock()
        .expect("active exec registry lock poisoned");
    registry.insert(span_id.to_string(), pgid);
    Some(ActiveExecRegistration {
        span_id: span_id.to_string(),
    })
}

#[cfg(target_os = "linux")]
struct ActiveExecRegistration {
    span_id: String,
}

#[cfg(target_os = "linux")]
impl Drop for ActiveExecRegistration {
    fn drop(&mut self) {
        if let Ok(mut registry) = active_exec_registry().lock() {
            registry.remove(&self.span_id);
        }
    }
}

#[cfg(target_os = "linux")]
pub fn signal_registered_exec(span_id: &str, sig: &str) -> Result<bool> {
    let signo = parse_signal_name(sig)
        .ok_or_else(|| anyhow!("unsupported execute cancellation signal: {sig}"))?;
    let pgid = {
        let registry = active_exec_registry()
            .lock()
            .expect("active exec registry lock poisoned");
        registry.get(span_id).copied()
    };

    let Some(pgid) = pgid else {
        return Ok(false);
    };

    if pgid <= 0 {
        return Ok(false);
    };

    let rc = unsafe { libc::kill(-(pgid as libc::pid_t), signo) };
    if rc != 0 {
        return Err(anyhow!(std::io::Error::last_os_error()))
            .context(format!("failed to signal active execute span {span_id}"));
    }

    Ok(true)
}

#[cfg(not(target_os = "linux"))]
pub fn signal_registered_exec(_span_id: &str, _sig: &str) -> Result<bool> {
    Ok(false)
}

#[cfg(unix)]
fn configure_child_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

#[cfg(not(unix))]
fn configure_child_process_group(_command: &mut Command) {}

#[cfg(target_os = "linux")]
fn configure_traced_child_process(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }

            if libc::ptrace(libc::PTRACE_TRACEME, 0, 0, 0) == -1 {
                return Err(std::io::Error::last_os_error());
            }

            Ok(())
        });
    }
}

#[cfg(target_os = "linux")]
pub fn process_capture_capability_available() -> bool {
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return false;
    };
    let Some(raw_caps) = status
        .lines()
        .find_map(|line| line.strip_prefix("CapEff:\t"))
        .map(str::trim)
    else {
        return false;
    };
    let Ok(bits) = u64::from_str_radix(raw_caps, 16) else {
        return false;
    };

    bits & (1u64 << 19) != 0
}

#[cfg(target_os = "linux")]
pub fn unavailable_process_telemetry(reason: &str) -> ProcessTelemetry {
    let mut telemetry = ProcessTelemetry::unavailable(reason);
    telemetry.process_events_backend = Some(PROCESS_EVENTS_BACKEND.to_string());
    telemetry
}

#[cfg(target_os = "linux")]
fn error_process_telemetry(error: String) -> ProcessTelemetry {
    ProcessTelemetry {
        process_events: Vec::new(),
        process_events_status: ProcessEventsStatus::Error,
        process_events_reason: Some("internal_error".to_string()),
        process_events_dropped: None,
        process_events_max: None,
        process_events_backend: Some(PROCESS_EVENTS_BACKEND.to_string()),
        process_events_error: Some(error),
    }
}

#[cfg(target_os = "linux")]
fn ptrace_continue(pid: libc::pid_t, sig: libc::c_int) {
    let deliver = if sig == libc::SIGTRAP { 0 } else { sig };
    unsafe {
        libc::ptrace(libc::PTRACE_CONT, pid, 0, deliver);
    }
}

#[cfg(target_os = "linux")]
fn ptrace_setoptions(pid: libc::pid_t) -> std::io::Result<()> {
    let options = (libc::PTRACE_O_TRACEFORK
        | libc::PTRACE_O_TRACEVFORK
        | libc::PTRACE_O_TRACEEXEC
        | libc::PTRACE_O_TRACEEXIT) as libc::c_long;
    let rc = unsafe { libc::ptrace(libc::PTRACE_SETOPTIONS, pid, 0, options) };
    if rc == -1 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn ptrace_geteventmsg(pid: libc::pid_t) -> Option<u64> {
    let mut value: libc::c_ulong = 0;
    let rc = unsafe {
        libc::ptrace(
            libc::PTRACE_GETEVENTMSG,
            pid,
            0,
            &mut value as *mut libc::c_ulong,
        )
    };
    if rc == -1 {
        return None;
    }
    Some(value as u64)
}

#[cfg(target_os = "linux")]
fn proc_link(pid: libc::pid_t, kind: &str) -> Option<String> {
    std::fs::read_link(format!("/proc/{pid}/{kind}"))
        .ok()
        .map(|path| path.display().to_string())
}

#[cfg(target_os = "linux")]
fn proc_ppid(pid: libc::pid_t) -> Option<u32> {
    let status = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    status.lines().find_map(|line| {
        let value = line.strip_prefix("PPid:\t")?;
        value.trim().parse::<u32>().ok()
    })
}

#[cfg(target_os = "linux")]
fn now_ts_parts() -> (String, u64) {
    let now = SystemTime::now();
    let unix = now
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0));
    (
        Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true),
        unix.as_nanos().min(u128::from(u64::MAX)) as u64,
    )
}

#[cfg(target_os = "linux")]
#[derive(Clone)]
struct TrackedProcess {
    ppid: u32,
    cwd: String,
    exe: Option<String>,
    started_at: Instant,
    start_ts: String,
    start_unix_ns: u64,
    exit_cwd: Option<String>,
    options_applied: bool,
    start_emitted: bool,
}

#[cfg(target_os = "linux")]
struct TelemetryCollector {
    spec: ProcessCaptureSpec,
    events: Vec<ProcessEvent>,
    dropped: u64,
}

#[cfg(target_os = "linux")]
impl TelemetryCollector {
    fn new(spec: ProcessCaptureSpec) -> Self {
        Self {
            spec,
            events: Vec::new(),
            dropped: 0,
        }
    }

    fn push(&mut self, event: ProcessEvent) {
        if self.events.len() < self.spec.max_events {
            self.events.push(event);
        } else {
            self.dropped = self.dropped.saturating_add(1);
        }
    }

    fn finish(mut self) -> ProcessTelemetry {
        self.events.sort_by(|a, b| {
            a.ts_unix_ns
                .cmp(&b.ts_unix_ns)
                .then_with(|| a.pid.cmp(&b.pid))
                .then_with(|| {
                    let lhs = match a.event_type {
                        ProcessEventType::WorldProcessStart => 0,
                        ProcessEventType::WorldProcessExit => 1,
                    };
                    let rhs = match b.event_type {
                        ProcessEventType::WorldProcessStart => 0,
                        ProcessEventType::WorldProcessExit => 1,
                    };
                    lhs.cmp(&rhs)
                })
        });

        if self.dropped > 0 {
            ProcessTelemetry {
                process_events: self.events,
                process_events_status: ProcessEventsStatus::Truncated,
                process_events_reason: Some("capture_overflow".to_string()),
                process_events_dropped: Some(self.dropped),
                process_events_max: Some(self.spec.max_events as u64),
                process_events_backend: Some(PROCESS_EVENTS_BACKEND.to_string()),
                process_events_error: None,
            }
        } else {
            ProcessTelemetry {
                process_events: self.events,
                process_events_status: ProcessEventsStatus::Ok,
                process_events_reason: None,
                process_events_dropped: None,
                process_events_max: None,
                process_events_backend: Some(PROCESS_EVENTS_BACKEND.to_string()),
                process_events_error: None,
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn emit_start_event(collector: &mut TelemetryCollector, pid: libc::pid_t, state: &TrackedProcess) {
    collector.push(ProcessEvent {
        event_type: ProcessEventType::WorldProcessStart,
        ts: state.start_ts.clone(),
        ts_unix_ns: state.start_unix_ns,
        session_id: collector.spec.session_id.clone(),
        world_id: collector.spec.world_id.clone(),
        pid: pid as u32,
        ppid: state.ppid,
        cwd: state.cwd.clone(),
        parent_span: collector.spec.parent_span.clone(),
        parent_cmd_id: collector.spec.parent_cmd_id.clone(),
        argv: None,
        argv_omitted: Some(true),
        exe: state.exe.clone(),
        exit_code: None,
        signal: None,
        duration_ms: None,
        env: None,
    });
}

#[cfg(target_os = "linux")]
fn build_tracked_process(pid: libc::pid_t, parent_pid: Option<u32>) -> TrackedProcess {
    let (start_ts, start_unix_ns) = now_ts_parts();
    TrackedProcess {
        ppid: parent_pid.or_else(|| proc_ppid(pid)).unwrap_or(0),
        cwd: proc_link(pid, "cwd").unwrap_or_else(|| "/".to_string()),
        exe: proc_link(pid, "exe"),
        started_at: Instant::now(),
        start_ts,
        start_unix_ns,
        exit_cwd: None,
        options_applied: false,
        start_emitted: false,
    }
}

#[cfg(target_os = "linux")]
fn finish_tracked_process(
    collector: &mut TelemetryCollector,
    pid: libc::pid_t,
    state: TrackedProcess,
    raw_status: libc::c_int,
) {
    let (ts, ts_unix_ns) = now_ts_parts();
    let duration_ms = state
        .started_at
        .elapsed()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64;
    collector.push(ProcessEvent {
        event_type: ProcessEventType::WorldProcessExit,
        ts,
        ts_unix_ns,
        session_id: collector.spec.session_id.clone(),
        world_id: collector.spec.world_id.clone(),
        pid: pid as u32,
        ppid: state.ppid,
        cwd: state.exit_cwd.unwrap_or(state.cwd),
        parent_span: collector.spec.parent_span.clone(),
        parent_cmd_id: collector.spec.parent_cmd_id.clone(),
        argv: None,
        argv_omitted: Some(true),
        exe: state.exe,
        exit_code: if libc::WIFEXITED(raw_status) {
            Some(libc::WEXITSTATUS(raw_status))
        } else {
            None
        },
        signal: if libc::WIFSIGNALED(raw_status) {
            Some(libc::WTERMSIG(raw_status))
        } else {
            None
        },
        duration_ms: Some(duration_ms),
        env: None,
    });
}

#[cfg(target_os = "linux")]
fn wait_for_ptrace_exit(pid: libc::pid_t) -> libc::c_int {
    loop {
        let mut status = 0;
        let rc = unsafe { libc::waitpid(pid, &mut status as *mut libc::c_int, 0) };
        if rc < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return 1 << 8;
        }
        if libc::WIFSTOPPED(status) {
            ptrace_continue(pid, libc::WSTOPSIG(status));
            continue;
        }
        return status;
    }
}

#[cfg(target_os = "linux")]
pub fn trace_spawned_process_tree(
    root_pid: libc::pid_t,
    spec: &ProcessCaptureSpec,
) -> TracedProcessTreeResult {
    let mut collector = TelemetryCollector::new(spec.clone());
    let mut tracked: HashMap<libc::pid_t, TrackedProcess> = HashMap::new();
    let mut final_root_status: Option<libc::c_int> = None;

    let mut status = 0;
    let rc = unsafe { libc::waitpid(root_pid, &mut status as *mut libc::c_int, 0) };
    if rc < 0 {
        return TracedProcessTreeResult {
            raw_exit_status: 1 << 8,
            process_telemetry: error_process_telemetry(format!(
                "waitpid initial ptrace stop failed: {}",
                std::io::Error::last_os_error()
            )),
        };
    }

    if !libc::WIFSTOPPED(status) {
        return TracedProcessTreeResult {
            raw_exit_status: status,
            process_telemetry: error_process_telemetry(
                "child did not enter ptrace stop as expected".to_string(),
            ),
        };
    }

    if let Err(err) = ptrace_setoptions(root_pid) {
        ptrace_continue(root_pid, libc::WSTOPSIG(status));
        return TracedProcessTreeResult {
            raw_exit_status: wait_for_ptrace_exit(root_pid),
            process_telemetry: if matches!(
                err.raw_os_error(),
                Some(libc::EPERM) | Some(libc::EACCES)
            ) {
                unavailable_process_telemetry("ptrace_not_permitted")
            } else {
                error_process_telemetry(format!("ptrace SETOPTIONS failed: {err}"))
            },
        };
    }

    let mut root_state = build_tracked_process(root_pid, None);
    root_state.options_applied = true;
    emit_start_event(&mut collector, root_pid, &root_state);
    root_state.start_emitted = true;
    tracked.insert(root_pid, root_state);
    ptrace_continue(root_pid, libc::WSTOPSIG(status));

    while !tracked.is_empty() {
        status = 0;
        let rc = unsafe {
            libc::waitpid(
                -1,
                &mut status as *mut libc::c_int,
                libc::__WALL | libc::WUNTRACED,
            )
        };
        if rc < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return TracedProcessTreeResult {
                raw_exit_status: final_root_status.unwrap_or(1 << 8),
                process_telemetry: error_process_telemetry(format!("ptrace waitpid failed: {err}")),
            };
        }

        let pid = rc;

        if libc::WIFSTOPPED(status) {
            let sig = libc::WSTOPSIG(status);
            let event = (status >> 16) & 0xffff;
            let state = tracked
                .entry(pid)
                .or_insert_with(|| build_tracked_process(pid, None));

            if !state.options_applied {
                if let Err(err) = ptrace_setoptions(pid) {
                    ptrace_continue(pid, sig);
                    return TracedProcessTreeResult {
                        raw_exit_status: final_root_status.unwrap_or(1 << 8),
                        process_telemetry: if matches!(
                            err.raw_os_error(),
                            Some(libc::EPERM) | Some(libc::EACCES)
                        ) {
                            unavailable_process_telemetry("ptrace_not_permitted")
                        } else {
                            error_process_telemetry(format!(
                                "ptrace SETOPTIONS failed for pid {pid}: {err}"
                            ))
                        },
                    };
                }
                state.options_applied = true;
            }

            if !state.start_emitted {
                emit_start_event(&mut collector, pid, state);
                state.start_emitted = true;
            }

            if event == libc::PTRACE_EVENT_EXIT {
                state.exit_cwd = proc_link(pid, "cwd");
            } else if matches!(event, libc::PTRACE_EVENT_FORK | libc::PTRACE_EVENT_VFORK) {
                if let Some(child_pid) = ptrace_geteventmsg(pid) {
                    tracked.entry(child_pid as libc::pid_t).or_insert_with(|| {
                        build_tracked_process(child_pid as libc::pid_t, Some(pid as u32))
                    });
                }
            }

            ptrace_continue(pid, sig);
            continue;
        }

        if libc::WIFEXITED(status) || libc::WIFSIGNALED(status) {
            if let Some(state) = tracked.remove(&pid) {
                finish_tracked_process(&mut collector, pid, state, status);
            }
            if pid == root_pid {
                final_root_status = Some(status);
            }
        }
    }

    TracedProcessTreeResult {
        raw_exit_status: final_root_status.unwrap_or(1 << 8),
        process_telemetry: collector.finish(),
    }
}

/// Execute `cmd` via `sh -lc` in the provided directory/environment, pushing
/// stdout/stderr chunks through the global stream sink while accumulating the
/// full output for the caller.
pub fn execute_shell_command(
    cmd: &str,
    cwd: &Path,
    env: &HashMap<String, String>,
    login_shell: bool,
    _span_id: Option<&str>,
) -> Result<Output> {
    Ok(execute_shell_command_with_capture(
        cmd,
        cwd,
        env,
        login_shell,
        CommandCapture::new(_span_id, None),
    )?
    .output)
}

#[cfg(target_os = "linux")]
pub(crate) fn execute_shell_command_with_capture(
    cmd: &str,
    cwd: &Path,
    env: &HashMap<String, String>,
    login_shell: bool,
    capture: CommandCapture<'_>,
) -> Result<CapturedCommandOutput> {
    fn build_command(
        cmd: &str,
        cwd: &Path,
        env: &HashMap<String, String>,
        login_shell: bool,
        enable_ptrace: bool,
    ) -> Command {
        let mut command = Command::new("sh");
        if login_shell {
            command.arg("-lc");
        } else {
            command.arg("-c");
        }
        command.arg(cmd);
        command.current_dir(cwd);
        command.env_clear();
        command.envs(env);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        if enable_ptrace {
            configure_traced_child_process(&mut command);
        } else {
            configure_child_process_group(&mut command);
        }
        command
    }

    let tracing_requested = capture.process_capture.is_some();
    let tracing_permitted = tracing_requested && process_capture_capability_available();

    let mut telemetry = if tracing_requested && !tracing_permitted {
        unavailable_process_telemetry("ptrace_not_permitted")
    } else {
        ProcessTelemetry::default()
    };

    let mut child = build_command(cmd, cwd, env, login_shell, tracing_permitted)
        .spawn()
        .with_context(|| format!("Failed to spawn command: {cmd}"))?;
    let _active_exec = register_active_exec(capture.span_id, child.id() as i32);

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("Failed to capture stdout pipe"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("Failed to capture stderr pipe"))?;

    let stdout_handle = spawn_reader(stdout, StreamKind::Stdout);
    let stderr_handle = spawn_reader(stderr, StreamKind::Stderr);

    let status = if tracing_permitted {
        let trace = trace_spawned_process_tree(
            child.id() as libc::pid_t,
            capture.process_capture.expect("capture spec"),
        );
        telemetry = trace.process_telemetry;
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            std::process::ExitStatus::from_raw(trace.raw_exit_status)
        }
    } else {
        child
            .wait()
            .context("Failed to wait for child process completion")?
    };

    let stdout_buf = join_reader(stdout_handle, "stdout");
    let stderr_buf = join_reader(stderr_handle, "stderr");

    Ok(CapturedCommandOutput {
        output: Output {
            status,
            stdout: stdout_buf,
            stderr: stderr_buf,
        },
        process_telemetry: telemetry,
    })
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn execute_shell_command_with_capture(
    cmd: &str,
    cwd: &Path,
    env: &HashMap<String, String>,
    login_shell: bool,
    capture: CommandCapture<'_>,
) -> Result<CapturedCommandOutput> {
    let mut command = Command::new("sh");
    if login_shell {
        command.arg("-lc");
    } else {
        command.arg("-c");
    }
    command.arg(cmd);
    command.current_dir(cwd);
    command.env_clear();
    command.envs(env);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    configure_child_process_group(&mut command);

    let mut child = command
        .spawn()
        .with_context(|| format!("Failed to spawn command: {cmd}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("Failed to capture stdout pipe"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("Failed to capture stderr pipe"))?;

    let stdout_handle = spawn_reader(stdout, StreamKind::Stdout);
    let stderr_handle = spawn_reader(stderr, StreamKind::Stderr);

    let status = child
        .wait()
        .context("Failed to wait for child process completion")?;

    let stdout_buf = join_reader(stdout_handle, "stdout");
    let stderr_buf = join_reader(stderr_handle, "stderr");

    let telemetry = if capture.process_capture.is_some() {
        ProcessTelemetry::not_supported_platform()
    } else {
        ProcessTelemetry::default()
    };

    Ok(CapturedCommandOutput {
        output: Output {
            status,
            stdout: stdout_buf,
            stderr: stderr_buf,
        },
        process_telemetry: telemetry,
    })
}

pub struct ProjectBindMount<'a> {
    pub merged_dir: &'a Path,
    pub project_dir: &'a Path,
    pub desired_cwd: &'a Path,
    pub fs_mode: WorldFsMode,
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
#[derive(Clone)]
pub(crate) struct CgroupAttachPolicy<'a> {
    required: bool,
    cgroup_procs_path: Option<PathBuf>,
    helper_label: &'static str,
    _marker: std::marker::PhantomData<&'a Path>,
}

impl<'a> CgroupAttachPolicy<'a> {
    pub(crate) fn optional(helper_label: &'static str) -> Self {
        Self {
            required: false,
            cgroup_procs_path: None,
            helper_label,
            _marker: std::marker::PhantomData,
        }
    }

    pub(crate) fn required(helper_label: &'static str, cgroup_path: &'a Path) -> Self {
        Self {
            required: true,
            cgroup_procs_path: Some(cgroup_path.join("cgroup.procs")),
            helper_label,
            _marker: std::marker::PhantomData,
        }
    }

    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    fn apply_env(self, env_map: &mut HashMap<String, String>) {
        if !self.required {
            return;
        }

        env_map.insert(
            "SUBSTRATE_CGROUP_ATTACH_REQUIRED".to_string(),
            "1".to_string(),
        );
        env_map.insert(
            "SUBSTRATE_CGROUP_ATTACH_HELPER_LABEL".to_string(),
            self.helper_label.to_string(),
        );
        if let Some(path) = self.cgroup_procs_path {
            env_map.insert(
                "SUBSTRATE_CGROUP_PROCS_PATH".to_string(),
                path.display().to_string(),
            );
        }
    }
}

pub const PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT: &str = r#"set -eu
set -f

mount --make-rprivate / 2>/dev/null || mount --make-private / 2>/dev/null || true

attach_to_cgroup_or_fail() {
  if [ "${SUBSTRATE_CGROUP_ATTACH_REQUIRED:-0}" != "1" ]; then
    return 0
  fi

  helper_label="${SUBSTRATE_CGROUP_ATTACH_HELPER_LABEL:-project_bind_mount}"
  cgroup_procs_path="${SUBSTRATE_CGROUP_PROCS_PATH:-}"
  if [ -z "$cgroup_procs_path" ]; then
    echo "substrate: error: ${helper_label}: cgroup attach target missing" >&2
    exit 4
  fi
  if [ ! -e "$cgroup_procs_path" ]; then
    echo "substrate: error: ${helper_label}: cgroup attach target does not exist: $cgroup_procs_path" >&2
    exit 4
  fi
  if ! printf '%s\n' "$$" > "$cgroup_procs_path"; then
    echo "substrate: error: ${helper_label}: cgroup attach failed: $cgroup_procs_path" >&2
    exit 4
  fi
}

attach_to_cgroup_or_fail

world_deps_host_root="${SUBSTRATE_WORLD_DEPS_HOST_ROOT:-/var/lib/substrate/world-deps}"
mkdir -p "$world_deps_host_root"

if [ "${SUBSTRATE_WORLD_FS_ISOLATION:-workspace}" = "full" ]; then
  new_root="$(mktemp -d /tmp/substrate-full-isolation.XXXXXX)"

  # Ensure new_root is a mountpoint (required by pivot_root).
  mount --bind "$new_root" "$new_root"
  mkdir -p "$new_root/old_root"

  bind_ro() {
    src="$1"
    dst="$2"
    if [ -e "$src" ]; then
      mkdir -p "$dst"
      mount --rbind "$src" "$dst"
      mount -o remount,bind,ro "$dst"
    fi
  }

  # Minimal system mounts.
  bind_ro /usr "$new_root/usr"
  bind_ro /bin "$new_root/bin"
  bind_ro /lib "$new_root/lib"
  bind_ro /lib64 "$new_root/lib64"
  bind_ro /etc "$new_root/etc"

  # /dev: bind-mounted read-only.
  mkdir -p "$new_root/dev"
  mount --rbind /dev "$new_root/dev"
  mount -o remount,bind,ro "$new_root/dev"

  # /var/lib/substrate/world-deps: bind-mounted read-write.
  mkdir -p "$new_root/var/lib/substrate/world-deps"
  mount --rbind "$world_deps_host_root" "$new_root/var/lib/substrate/world-deps"

  # Fresh /proc and writable /tmp.
  #
  # Note: /tmp is a tmpfs in full-isolation mode. This must be mounted before binding the project
  # into its host-absolute path. Otherwise, when the host project lives under /tmp, the tmpfs
  # mount would cover that project bind mount and `cd $SUBSTRATE_MOUNT_CWD` would fail.
  mkdir -p "$new_root/proc"
  mount -t proc proc "$new_root/proc"

  mkdir -p "$new_root/tmp"
  mount -t tmpfs tmpfs "$new_root/tmp"
  chmod 1777 "$new_root/tmp" || true

  # Project mount points: stable (/project) and host-absolute ($SUBSTRATE_MOUNT_PROJECT_DIR).
  #
  # ADR-0004: place the overlay mount at the project path via mount --move (not mount --bind).
  mkdir -p "$new_root$SUBSTRATE_MOUNT_PROJECT_DIR"
  mount --move "$SUBSTRATE_MOUNT_MERGED_DIR" "$new_root$SUBSTRATE_MOUNT_PROJECT_DIR"

  mkdir -p "$new_root/project"
  mount --bind "$new_root$SUBSTRATE_MOUNT_PROJECT_DIR" "$new_root/project"

  # Ensure overlayfs backing dirs (upper/work) are visible after pivot_root.
  #
  # Overlayfs keeps upper/workdir paths as absolute host paths. In full-isolation mode we pivot_root
  # into a minimal rootfs (and mount tmpfs on /tmp), so we must bind-mount the backing storage into
  # the new root before pivot_root or overlayfs writes can fail with EPERM.
  mount_point="$new_root$SUBSTRATE_MOUNT_PROJECT_DIR"
  opts="$(awk -v mp="$mount_point" '$2==mp {print $4; exit}' /proc/mounts 2>/dev/null || true)"
  upperdir=""
  workdir=""
  if [ -n "$opts" ]; then
    upperdir="$(printf '%s' "$opts" | tr ',' '\n' | sed -n 's/^upperdir=//p' | head -n 1)"
    workdir="$(printf '%s' "$opts" | tr ',' '\n' | sed -n 's/^workdir=//p' | head -n 1)"
  fi
  if [ -n "$upperdir" ] || [ -n "$workdir" ]; then
    state_dir="$upperdir"
    case "$state_dir" in
      */upper) state_dir="${state_dir%/upper}" ;;
      */work) state_dir="${state_dir%/work}" ;;
      '') state_dir="$workdir" ;;
    esac
    if [ -n "$state_dir" ]; then
      case "$state_dir" in
        */upper) state_dir="${state_dir%/upper}" ;;
        */work) state_dir="${state_dir%/work}" ;;
        *) state_dir="$(dirname "$state_dir")" ;;
      esac
    fi

    if [ -n "$state_dir" ] && [ -d "$state_dir" ]; then
      upper_sig=""
      work_sig=""
      if [ -n "$upperdir" ] && [ -d "$upperdir" ]; then
        upper_sig="$(stat -c '%d:%i' "$upperdir" 2>/dev/null || true)"
      fi
      if [ -n "$workdir" ] && [ -d "$workdir" ]; then
        work_sig="$(stat -c '%d:%i' "$workdir" 2>/dev/null || true)"
      fi

      mkdir -p "$new_root$state_dir"
      if ! mount --rbind "$state_dir" "$new_root$state_dir"; then
        echo "substrate: error: failed to bind-mount overlay backing dir into isolated root: $state_dir" >&2
        exit 4
      fi

      if [ -n "$upper_sig" ] && [ -n "$upperdir" ]; then
        new_upper_sig="$(stat -c '%d:%i' "$new_root$upperdir" 2>/dev/null || true)"
        if [ -z "$new_upper_sig" ] || [ "$new_upper_sig" != "$upper_sig" ]; then
          echo "substrate: error: overlay upperdir is not visible inside isolated root (expected $upperdir)" >&2
          exit 4
        fi
      fi
      if [ -n "$work_sig" ] && [ -n "$workdir" ]; then
        new_work_sig="$(stat -c '%d:%i' "$new_root$workdir" 2>/dev/null || true)"
        if [ -z "$new_work_sig" ] || [ "$new_work_sig" != "$work_sig" ]; then
          echo "substrate: error: overlay workdir is not visible inside isolated root (expected $workdir)" >&2
          exit 4
        fi
      fi
    fi
  fi

  landlock_requested=0
  if [ -n "${SUBSTRATE_WORLD_FS_LANDLOCK_READ_ALLOWLIST:-}" ] || [ -n "${SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST:-}" ]; then
    landlock_requested=1
  fi

  # Ensure allowlisted writable prefixes exist before we remount the project read-only.
  if [ "${SUBSTRATE_MOUNT_FS_MODE:-writable}" != "read_only" ] && [ -n "${SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST:-}" ]; then
    oldIFS=$IFS
    IFS='
'
    for rel in $SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST; do
      [ -z "$rel" ] && continue
      case "$rel" in
        /*) continue ;;
      esac
      case "/$rel/" in
        */../*) continue ;;
      esac
      [ "$rel" = "." ] && continue
      mkdir -p "$new_root/project/$rel"
      mkdir -p "$new_root$SUBSTRATE_MOUNT_PROJECT_DIR/$rel"
    done
    IFS=$oldIFS
  fi

  if [ "$landlock_requested" != "1" ]; then
    # Project is read-only by default; remount allowlisted prefixes writable.
    #
    # Note: on some kernels overlayfs may still deny writes under a bind-mounted subpath when the
    # overlay superblock is mounted read-only, even if the sub-mount shows `rw` in /proc/mounts.
    # When Landlock allowlists are in use, we skip this mount-based enforcement and rely on
    # Landlock instead.
    mount -o remount,bind,ro "$new_root/project"
    mount -o remount,bind,ro "$new_root$SUBSTRATE_MOUNT_PROJECT_DIR"

    if [ "${SUBSTRATE_MOUNT_FS_MODE:-writable}" != "read_only" ] && [ -n "${SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST:-}" ]; then
      oldIFS=$IFS
      IFS='
'
      for rel in $SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST; do
        [ -z "$rel" ] && continue
        case "$rel" in
          /*) continue ;;
        esac
        case "/$rel/" in
          */../*) continue ;;
        esac
        if [ "$rel" = "." ]; then
          mount -o remount,bind,rw "$new_root/project"
          mount -o remount,bind,rw "$new_root$SUBSTRATE_MOUNT_PROJECT_DIR"
          continue
        fi
        mount --bind "$new_root/project/$rel" "$new_root/project/$rel"
        mount -o remount,bind,rw "$new_root/project/$rel"
        mount --bind "$new_root$SUBSTRATE_MOUNT_PROJECT_DIR/$rel" "$new_root$SUBSTRATE_MOUNT_PROJECT_DIR/$rel"
        mount -o remount,bind,rw "$new_root$SUBSTRATE_MOUNT_PROJECT_DIR/$rel"
      done
      IFS=$oldIFS
    fi
  fi

  # Optional: bind-mount the host world-agent binary into the isolated rootfs so it can apply Landlock
  # restrictions before executing the command.
  if [ "$landlock_requested" = "1" ]; then
    if [ -z "${SUBSTRATE_LANDLOCK_HELPER_SRC:-}" ] || [ ! -e "${SUBSTRATE_LANDLOCK_HELPER_SRC:-}" ]; then
      echo "substrate: error: landlock allowlists set but SUBSTRATE_LANDLOCK_HELPER_SRC was not available" >&2
      exit 4
    fi
  fi
  if [ -n "${SUBSTRATE_LANDLOCK_HELPER_SRC:-}" ] && [ -e "${SUBSTRATE_LANDLOCK_HELPER_SRC:-}" ]; then
    touch "$new_root/substrate-landlock-helper" 2>/dev/null || true
    mount --bind "$SUBSTRATE_LANDLOCK_HELPER_SRC" "$new_root/substrate-landlock-helper" 2>/dev/null || true
    mount -o remount,bind,ro "$new_root/substrate-landlock-helper" 2>/dev/null || true
    export SUBSTRATE_LANDLOCK_HELPER_PATH="/substrate-landlock-helper"
  fi

  pivot_root "$new_root" "$new_root/old_root"
  cd /
  umount -l /old_root 2>/dev/null || true
  rmdir /old_root 2>/dev/null || true

  # WDH0 contract uses HOME=/root. In hardened worlds, the base / can be remounted read-only,
  # so ensure /root is writable (tmpfs-backed) before executing the inner command.
  mkdir -p /root 2>/dev/null || true
  if ! touch /root/.substrate_write_probe 2>/dev/null; then
    mount -t tmpfs tmpfs /root 2>/dev/null || true
    chmod 0700 /root 2>/dev/null || true
  else
    rm -f /root/.substrate_write_probe 2>/dev/null || true
  fi
  mkdir -p /root/.npm /root/.cache /root/.config /root/.local/share 2>/dev/null || true

else
  # ADR-0004: place the overlay mount at the project path via mount --move (not mount --bind).
  mount --move "$SUBSTRATE_MOUNT_MERGED_DIR" "$SUBSTRATE_MOUNT_PROJECT_DIR"
  # Preserve the world-deps host root before mounting tmpfs on /var/lib. When the host root lives
  # under /var/lib (the default), mounting tmpfs would otherwise hide it and we would end up
  # bind-mounting an empty directory into the isolated /var/lib.
  world_deps_hold="${XDG_RUNTIME_DIR:-/tmp}/substrate-world-deps-host.$$"
  mkdir -p "$world_deps_hold"
  mount --rbind "$world_deps_host_root" "$world_deps_hold"
  mount -t tmpfs tmpfs /var/lib
  mkdir -p /var/lib/substrate/world-deps
  mount --rbind "$world_deps_hold" /var/lib/substrate/world-deps
  umount -l "$world_deps_hold" 2>/dev/null || true
  rmdir "$world_deps_hold" 2>/dev/null || true
  if [ "${SUBSTRATE_MOUNT_FS_MODE:-writable}" = "read_only" ]; then
    mount -o remount,bind,ro "$SUBSTRATE_MOUNT_PROJECT_DIR"
  fi
  if [ -n "${SUBSTRATE_LANDLOCK_HELPER_SRC:-}" ] && [ -x "${SUBSTRATE_LANDLOCK_HELPER_SRC:-}" ]; then
    export SUBSTRATE_LANDLOCK_HELPER_PATH="${SUBSTRATE_LANDLOCK_HELPER_SRC}"
  fi
  # WDH0 contract uses HOME=/root. In hardened worlds, the base / can be remounted read-only,
  # so ensure /root is writable (tmpfs-backed) before executing the inner command.
  mkdir -p /root 2>/dev/null || true
  if ! touch /root/.substrate_write_probe 2>/dev/null; then
    mount -t tmpfs tmpfs /root 2>/dev/null || true
    chmod 0700 /root 2>/dev/null || true
  else
    rm -f /root/.substrate_write_probe 2>/dev/null || true
  fi
  mkdir -p /root/.npm /root/.cache /root/.config /root/.local/share 2>/dev/null || true
  mkdir -p "${XDG_CACHE_HOME:-/tmp/substrate-xdg/cache}" 2>/dev/null || true
  mkdir -p "${XDG_CONFIG_HOME:-/tmp/substrate-xdg/config}" 2>/dev/null || true
  mkdir -p "${XDG_DATA_HOME:-/tmp/substrate-xdg/data}" 2>/dev/null || true
fi

cd "$SUBSTRATE_MOUNT_CWD"
if [ -n "${SUBSTRATE_LANDLOCK_HELPER_PATH:-}" ] && [ -x "${SUBSTRATE_LANDLOCK_HELPER_PATH}" ]; then
  exec "$SUBSTRATE_LANDLOCK_HELPER_PATH" "__substrate_world_landlock_exec"
fi
if [ "${SUBSTRATE_INNER_LOGIN_SHELL:-0}" = "1" ]; then
  exec sh -lc "$SUBSTRATE_INNER_CMD"
else
  exec sh -c "$SUBSTRATE_INNER_CMD"
fi
"#;

pub const WORLD_DEPS_BIND_MOUNT_FALLBACK_SCRIPT: &str = r#"set -eu
set -f

mount --make-rprivate / 2>/dev/null || mount --make-private / 2>/dev/null || true

attach_to_cgroup_or_fail() {
  if [ "${SUBSTRATE_CGROUP_ATTACH_REQUIRED:-0}" != "1" ]; then
    return 0
  fi

  helper_label="${SUBSTRATE_CGROUP_ATTACH_HELPER_LABEL:-world_deps_fallback}"
  cgroup_procs_path="${SUBSTRATE_CGROUP_PROCS_PATH:-}"
  if [ -z "$cgroup_procs_path" ]; then
    echo "substrate: error: ${helper_label}: cgroup attach target missing" >&2
    exit 4
  fi
  if [ ! -e "$cgroup_procs_path" ]; then
    echo "substrate: error: ${helper_label}: cgroup attach target does not exist: $cgroup_procs_path" >&2
    exit 4
  fi
  if ! printf '%s\n' "$$" > "$cgroup_procs_path"; then
    echo "substrate: error: ${helper_label}: cgroup attach failed: $cgroup_procs_path" >&2
    exit 4
  fi
}

attach_to_cgroup_or_fail

mkdir -p "$SUBSTRATE_WORLD_DEPS_HOST_ROOT"
world_deps_hold="${XDG_RUNTIME_DIR:-/tmp}/substrate-world-deps-host.$$"
mkdir -p "$world_deps_hold"
mount --rbind "$SUBSTRATE_WORLD_DEPS_HOST_ROOT" "$world_deps_hold"
mount -t tmpfs tmpfs /var/lib
mkdir -p /var/lib/substrate/world-deps
mount --rbind "$world_deps_hold" /var/lib/substrate/world-deps
umount -l "$world_deps_hold" 2>/dev/null || true
rmdir "$world_deps_hold" 2>/dev/null || true

# WDH0 contract uses HOME=/root. In hardened worlds, the base / can be remounted read-only,
# so ensure /root is writable (tmpfs-backed) before executing the inner command.
mkdir -p /root 2>/dev/null || true
if ! touch /root/.substrate_write_probe 2>/dev/null; then
  mount -t tmpfs tmpfs /root 2>/dev/null || true
  chmod 0700 /root 2>/dev/null || true
else
  rm -f /root/.substrate_write_probe 2>/dev/null || true
fi
mkdir -p /root/.npm /root/.cache /root/.config /root/.local/share 2>/dev/null || true
mkdir -p "${XDG_CACHE_HOME:-/tmp/substrate-xdg/cache}" 2>/dev/null || true
mkdir -p "${XDG_CONFIG_HOME:-/tmp/substrate-xdg/config}" 2>/dev/null || true
mkdir -p "${XDG_DATA_HOME:-/tmp/substrate-xdg/data}" 2>/dev/null || true

cd "$SUBSTRATE_FALLBACK_CWD"
if [ "${SUBSTRATE_INNER_LOGIN_SHELL:-0}" = "1" ]; then
  exec sh -lc "$SUBSTRATE_INNER_CMD"
else
  exec sh -c "$SUBSTRATE_INNER_CMD"
fi
"#;

pub(crate) fn execute_shell_command_with_project_bind_mount(
    cmd: &str,
    mount: ProjectBindMount<'_>,
    env: &HashMap<String, String>,
    login_shell: bool,
    _attach_policy: CgroupAttachPolicy<'_>,
    _span_id: Option<&str>,
) -> Result<Output> {
    Ok(execute_shell_command_with_project_bind_mount_capture(
        cmd,
        mount,
        env,
        login_shell,
        _attach_policy,
        CommandCapture::new(_span_id, None),
    )?
    .output)
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn execute_shell_command_with_project_bind_mount_capture(
    cmd: &str,
    mount: ProjectBindMount<'_>,
    env: &HashMap<String, String>,
    login_shell: bool,
    _attach_policy: CgroupAttachPolicy<'_>,
    _capture: CommandCapture<'_>,
) -> Result<CapturedCommandOutput> {
    let _ = cmd;
    let _ = mount;
    let _ = env;
    let _ = login_shell;
    Err(anyhow!(
        "project bind mount enforcement is only supported on Linux"
    ))
}

#[cfg(target_os = "linux")]
pub(crate) fn execute_shell_command_with_project_bind_mount_capture(
    cmd: &str,
    mount: ProjectBindMount<'_>,
    env: &HashMap<String, String>,
    login_shell: bool,
    _attach_policy: CgroupAttachPolicy<'_>,
    capture: CommandCapture<'_>,
) -> Result<CapturedCommandOutput> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = cmd;
        let _ = mount;
        let _ = env;
        let _ = login_shell;
        Err(anyhow!(
            "project bind mount enforcement is only supported on Linux"
        ))
    }

    #[cfg(target_os = "linux")]
    {
        use nix::unistd::Uid;
        #[cfg(unix)]
        use std::os::unix::process::ExitStatusExt;

        // Outer script: establish a private mount namespace, then either:
        // - isolation=workspace: bind the overlay merged root onto the host project path to prevent
        //   absolute-path escapes back into the host project, or
        // - isolation=full: build a minimal rootfs, bind-mount only the allowed paths, then pivot_root
        //   so host paths are no longer nameable.
        //
        // We avoid setting the child's cwd via Command::current_dir() because holding an inode
        // reference into the host project dir would bypass the bind mount (absolute-path escape).
        let script = PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT;

        let env_map = project_bind_mount_env_map(cmd, &mount, env, login_shell, _attach_policy);
        let isolation = env_map
            .get("SUBSTRATE_WORLD_FS_ISOLATION")
            .map(|raw| raw.trim().to_ascii_lowercase())
            .unwrap_or_else(|| "workspace".to_string());
        let isolation_full = isolation == "full";
        let tracing_requested = capture.process_capture.is_some();
        let tracing_permitted = tracing_requested && process_capture_capability_available();
        let mut telemetry = if tracing_requested && !tracing_permitted {
            unavailable_process_telemetry("ptrace_not_permitted")
        } else {
            ProcessTelemetry::default()
        };

        let mut command = Command::new("unshare");
        command.arg("--mount");
        command.arg("--fork");
        if !Uid::effective().is_root() {
            // Best-effort: try to acquire mount privileges via user namespaces when unprivileged.
            // If user namespaces are disabled on the host, the command will fail and the caller
            // should fall back to the non-caged path.
            command.arg("--user");
            command.arg("--map-root-user");
        }
        command.arg("--");
        command.arg("sh");
        command.arg("-c");
        command.arg(script);
        command.current_dir("/");
        // Ensure the unshare wrapper and its child workload do not inherit the host/world-agent
        // service environment. The caller must fully specify the desired environment.
        command.env_clear();
        command.envs(env_map);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        if tracing_permitted {
            configure_traced_child_process(&mut command);
        } else {
            configure_child_process_group(&mut command);
        }

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(err) => {
                if isolation_full {
                    let message = format!(
                        "substrate: error: world_fs.isolation=full requested but failed to spawn unshare wrapper: {err}\n"
                    );
                    return Ok(CapturedCommandOutput {
                        output: Output {
                            status: std::process::ExitStatus::from_raw(126 << 8),
                            stdout: Vec::new(),
                            stderr: message.into_bytes(),
                        },
                        process_telemetry: telemetry,
                    });
                }
                return Err(err).with_context(|| format!("Failed to spawn command: {cmd}"));
            }
        };
        let _active_exec = register_active_exec(capture.span_id, child.id() as i32);

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to capture stdout pipe"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("Failed to capture stderr pipe"))?;

        let stdout_handle = spawn_reader(stdout, StreamKind::Stdout);
        let stderr_handle = spawn_reader(stderr, StreamKind::Stderr);

        let status = if tracing_permitted {
            let trace = trace_spawned_process_tree(
                child.id() as libc::pid_t,
                capture.process_capture.expect("capture spec"),
            );
            telemetry = trace.process_telemetry;
            std::process::ExitStatus::from_raw(trace.raw_exit_status)
        } else {
            child
                .wait()
                .context("Failed to wait for child process completion")?
        };

        let stdout_buf = join_reader(stdout_handle, "stdout");
        let mut stderr_buf = join_reader(stderr_handle, "stderr");

        if isolation_full && !status.success() {
            if let Ok(stderr_str) = std::str::from_utf8(&stderr_buf) {
                if stderr_str.starts_with("unshare:") {
                    let mut wrapped = Vec::new();
                    wrapped.extend_from_slice(b"substrate: error: world_fs.isolation=full requested but failed to enter a mount namespace (unshare).\n");
                    wrapped.extend_from_slice(b"substrate: hint: run with CAP_SYS_ADMIN (root) or enable unprivileged user namespaces (kernel.unprivileged_userns_clone=1).\n");
                    wrapped.extend_from_slice(stderr_buf.as_slice());
                    stderr_buf = wrapped;
                }
            }
        }

        if !isolation_full && mount.fs_mode != WorldFsMode::ReadOnly && !status.success() {
            if let Some(trimmed) = mount_namespace_setup_failure(&stderr_buf) {
                anyhow::bail!("project bind mount setup failed: {trimmed}");
            }
        }

        Ok(CapturedCommandOutput {
            output: Output {
                status,
                stdout: stdout_buf,
                stderr: stderr_buf,
            },
            process_telemetry: telemetry,
        })
    }
}

#[cfg(target_os = "linux")]
fn project_bind_mount_env_map(
    cmd: &str,
    mount: &ProjectBindMount<'_>,
    env: &HashMap<String, String>,
    login_shell: bool,
    attach_policy: CgroupAttachPolicy<'_>,
) -> HashMap<String, String> {
    let mut env_map = env.clone();
    env_map.insert(
        "SUBSTRATE_MOUNT_MERGED_DIR".to_string(),
        mount.merged_dir.display().to_string(),
    );
    env_map.insert(
        "SUBSTRATE_MOUNT_PROJECT_DIR".to_string(),
        mount.project_dir.display().to_string(),
    );
    env_map.insert(
        "SUBSTRATE_MOUNT_CWD".to_string(),
        mount.desired_cwd.display().to_string(),
    );
    env_map.insert(
        "SUBSTRATE_MOUNT_FS_MODE".to_string(),
        mount.fs_mode.as_str().to_string(),
    );
    env_map.insert("SUBSTRATE_INNER_CMD".to_string(), cmd.to_string());
    env_map.insert(
        "SUBSTRATE_INNER_LOGIN_SHELL".to_string(),
        if login_shell {
            "1".to_string()
        } else {
            "0".to_string()
        },
    );

    let isolation = env_map
        .get("SUBSTRATE_WORLD_FS_ISOLATION")
        .map(|raw| raw.trim().to_ascii_lowercase())
        .unwrap_or_else(|| "workspace".to_string());
    let isolation_enabled = matches!(isolation.as_str(), "full" | "workspace" | "project");
    if isolation_enabled {
        env_map
            .entry("HOME".to_string())
            .or_insert_with(|| "/tmp/substrate-home".to_string());
        env_map
            .entry("XDG_CACHE_HOME".to_string())
            .or_insert_with(|| "/tmp/substrate-xdg/cache".to_string());
        env_map
            .entry("XDG_CONFIG_HOME".to_string())
            .or_insert_with(|| "/tmp/substrate-xdg/config".to_string());
        env_map
            .entry("XDG_DATA_HOME".to_string())
            .or_insert_with(|| "/tmp/substrate-xdg/data".to_string());
    }

    attach_policy.apply_env(&mut env_map);

    env_map
}

#[cfg(target_os = "linux")]
fn mount_namespace_setup_failure(stderr: &[u8]) -> Option<&str> {
    let stderr_str = std::str::from_utf8(stderr).ok()?;
    let trimmed = stderr_str.trim();
    if trimmed.starts_with("mount:") || trimmed.starts_with("unshare:") {
        Some(trimmed)
    } else {
        None
    }
}

pub(crate) fn execute_shell_command_with_world_deps_bind_mount(
    cmd: &str,
    cwd: &Path,
    env: &HashMap<String, String>,
    login_shell: bool,
    world_deps_root: &Path,
    _attach_policy: CgroupAttachPolicy<'_>,
    _span_id: Option<&str>,
) -> Result<Output> {
    Ok(execute_shell_command_with_world_deps_bind_mount_capture(
        cmd,
        cwd,
        env,
        login_shell,
        world_deps_root,
        _attach_policy,
        CommandCapture::new(_span_id, None),
    )?
    .output)
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn execute_shell_command_with_world_deps_bind_mount_capture(
    cmd: &str,
    cwd: &Path,
    env: &HashMap<String, String>,
    login_shell: bool,
    world_deps_root: &Path,
    _attach_policy: CgroupAttachPolicy<'_>,
    _capture: CommandCapture<'_>,
) -> Result<CapturedCommandOutput> {
    let _ = cmd;
    let _ = cwd;
    let _ = env;
    let _ = login_shell;
    let _ = world_deps_root;
    Err(anyhow!(
        "world-deps bind mount fallback is only supported on Linux"
    ))
}

#[cfg(target_os = "linux")]
pub(crate) fn execute_shell_command_with_world_deps_bind_mount_capture(
    cmd: &str,
    cwd: &Path,
    env: &HashMap<String, String>,
    login_shell: bool,
    world_deps_root: &Path,
    _attach_policy: CgroupAttachPolicy<'_>,
    capture: CommandCapture<'_>,
) -> Result<CapturedCommandOutput> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = cmd;
        let _ = cwd;
        let _ = env;
        let _ = login_shell;
        let _ = world_deps_root;
        Err(anyhow!(
            "world-deps bind mount fallback is only supported on Linux"
        ))
    }

    #[cfg(target_os = "linux")]
    {
        use nix::unistd::Uid;

        std::fs::create_dir_all(world_deps_root).with_context(|| {
            format!(
                "failed to create world-deps fallback root {}",
                world_deps_root.display()
            )
        })?;

        let env_map = world_deps_bind_mount_env_map(
            cmd,
            cwd,
            env,
            login_shell,
            world_deps_root,
            _attach_policy,
        );
        let tracing_requested = capture.process_capture.is_some();
        let tracing_permitted = tracing_requested && process_capture_capability_available();
        let mut telemetry = if tracing_requested && !tracing_permitted {
            unavailable_process_telemetry("ptrace_not_permitted")
        } else {
            ProcessTelemetry::default()
        };

        let mut command = Command::new("unshare");
        command.arg("--mount");
        command.arg("--fork");
        if !Uid::effective().is_root() {
            command.arg("--user");
            command.arg("--map-root-user");
        }
        command.arg("--");
        command.arg("sh");
        command.arg("-c");
        command.arg(WORLD_DEPS_BIND_MOUNT_FALLBACK_SCRIPT);
        command.current_dir("/");
        command.env_clear();
        command.envs(env_map);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        if tracing_permitted {
            configure_traced_child_process(&mut command);
        } else {
            configure_child_process_group(&mut command);
        }

        let mut child = command
            .spawn()
            .with_context(|| format!("Failed to spawn command: {cmd}"))?;
        let _active_exec = register_active_exec(capture.span_id, child.id() as i32);

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to capture stdout pipe"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("Failed to capture stderr pipe"))?;

        let stdout_handle = spawn_reader(stdout, StreamKind::Stdout);
        let stderr_handle = spawn_reader(stderr, StreamKind::Stderr);

        let status = if tracing_permitted {
            use std::os::unix::process::ExitStatusExt;
            let trace = trace_spawned_process_tree(
                child.id() as libc::pid_t,
                capture.process_capture.expect("capture spec"),
            );
            telemetry = trace.process_telemetry;
            std::process::ExitStatus::from_raw(trace.raw_exit_status)
        } else {
            child
                .wait()
                .context("Failed to wait for child process completion")?
        };

        let stdout_buf = join_reader(stdout_handle, "stdout");
        let stderr_buf = join_reader(stderr_handle, "stderr");

        if !status.success() {
            if let Some(trimmed) = mount_namespace_setup_failure(&stderr_buf) {
                anyhow::bail!("world-deps bind mount fallback setup failed: {trimmed}");
            }
        }

        Ok(CapturedCommandOutput {
            output: Output {
                status,
                stdout: stdout_buf,
                stderr: stderr_buf,
            },
            process_telemetry: telemetry,
        })
    }
}

#[cfg(target_os = "linux")]
fn world_deps_bind_mount_env_map(
    cmd: &str,
    cwd: &Path,
    env: &HashMap<String, String>,
    login_shell: bool,
    world_deps_root: &Path,
    attach_policy: CgroupAttachPolicy<'_>,
) -> HashMap<String, String> {
    let mut env_map = env.clone();
    env_map.insert(
        "SUBSTRATE_WORLD_DEPS_HOST_ROOT".to_string(),
        world_deps_root.display().to_string(),
    );
    env_map.insert(
        "SUBSTRATE_FALLBACK_CWD".to_string(),
        cwd.display().to_string(),
    );
    env_map.insert("SUBSTRATE_INNER_CMD".to_string(), cmd.to_string());
    env_map.insert(
        "SUBSTRATE_INNER_LOGIN_SHELL".to_string(),
        if login_shell {
            "1".to_string()
        } else {
            "0".to_string()
        },
    );
    env_map
        .entry("HOME".to_string())
        .or_insert_with(|| "/tmp/substrate-home".to_string());
    env_map
        .entry("XDG_CACHE_HOME".to_string())
        .or_insert_with(|| "/tmp/substrate-xdg/cache".to_string());
    env_map
        .entry("XDG_CONFIG_HOME".to_string())
        .or_insert_with(|| "/tmp/substrate-xdg/config".to_string());
    env_map
        .entry("XDG_DATA_HOME".to_string())
        .or_insert_with(|| "/tmp/substrate-xdg/data".to_string());
    attach_policy.apply_env(&mut env_map);
    env_map
}

pub(crate) fn is_cgroup_attach_wrapper_failure(stderr: &[u8]) -> bool {
    let Ok(stderr_str) = std::str::from_utf8(stderr) else {
        return false;
    };
    stderr_str.contains(": cgroup attach ")
}

pub fn stable_world_deps_fallback_root(project_dir: &Path) -> std::path::PathBuf {
    let uid = current_uid();
    let mut hasher = Sha256::new();
    hasher.update(project_dir.to_string_lossy().as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    std::env::temp_dir()
        .join(format!("substrate-{uid}-world-deps"))
        .join(digest)
}

#[cfg(unix)]
fn current_uid() -> u32 {
    unsafe { libc::geteuid() as u32 }
}

#[cfg(not(unix))]
fn current_uid() -> u32 {
    0
}

fn spawn_reader<R>(mut reader: R, kind: StreamKind) -> thread::JoinHandle<Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            match reader.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    let slice = &chunk[..n];
                    buf.extend_from_slice(slice);
                    emit_chunk(kind, slice);
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    return Err(anyhow!("pipe read failed: {e}"));
                }
            }
        }
        Ok(buf)
    })
}

fn join_reader(handle: thread::JoinHandle<Result<Vec<u8>>>, label: &str) -> Vec<u8> {
    match handle.join() {
        Ok(Ok(buf)) => buf,
        Ok(Err(e)) => {
            warn!(stream = label, error = %e, "stream reader error");
            Vec::new()
        }
        Err(e) => {
            warn!(stream = label, error = ?e, "stream reader panicked");
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "linux")]
    use substrate_common::ProcessEventsStatus;

    #[test]
    fn workspace_isolation_preserves_world_deps_root_before_tmpfs() {
        // Regression test: when workspace isolation mounts tmpfs on /var/lib, the default world-deps
        // host root lives under /var/lib and would be hidden unless we bind it somewhere stable
        // (outside /var/lib) first.
        assert!(PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT
            .contains("world_deps_hold=\"${XDG_RUNTIME_DIR:-/tmp}/substrate-world-deps-host.$$\""));
        assert!(PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT
            .contains("mount --rbind \"$world_deps_host_root\" \"$world_deps_hold\""));
        assert!(PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT
            .contains("mount --rbind \"$world_deps_hold\" /var/lib/substrate/world-deps"));

        assert!(WORLD_DEPS_BIND_MOUNT_FALLBACK_SCRIPT
            .contains("world_deps_hold=\"${XDG_RUNTIME_DIR:-/tmp}/substrate-world-deps-host.$$\""));
        assert!(WORLD_DEPS_BIND_MOUNT_FALLBACK_SCRIPT
            .contains("mount --rbind \"$SUBSTRATE_WORLD_DEPS_HOST_ROOT\" \"$world_deps_hold\""));
        assert!(WORLD_DEPS_BIND_MOUNT_FALLBACK_SCRIPT
            .contains("mount --rbind \"$world_deps_hold\" /var/lib/substrate/world-deps"));

        // HOME (/root) must be made writable even when / is remounted read-only by strict deny.
        assert!(PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT.contains("mount -t tmpfs tmpfs /root"));
        assert!(WORLD_DEPS_BIND_MOUNT_FALLBACK_SCRIPT.contains("mount -t tmpfs tmpfs /root"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn process_capture_spec_reads_trace_metadata_from_env() {
        let env = HashMap::from([
            ("SHIM_SESSION_ID".to_string(), "ses_test".to_string()),
            (
                SUBSTRATE_PARENT_SPAN_ENV.to_string(),
                "spn_parent".to_string(),
            ),
            ("SHIM_PARENT_CMD_ID".to_string(), "cmd_test".to_string()),
        ]);

        let spec = ProcessCaptureSpec::from_env("wld_test", &env, None);
        assert_eq!(spec.session_id, "ses_test");
        assert_eq!(spec.world_id, "wld_test");
        assert_eq!(spec.parent_span, "spn_parent");
        assert_eq!(spec.parent_cmd_id.as_deref(), Some("cmd_test"));
        assert_eq!(spec.max_events, PROCESS_EVENTS_DEFAULT_MAX);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn unavailable_process_telemetry_reports_ptrace_backend() {
        let telemetry = unavailable_process_telemetry("ptrace_not_permitted");
        assert_eq!(
            telemetry.process_events_status,
            ProcessEventsStatus::Unavailable
        );
        assert_eq!(
            telemetry.process_events_reason.as_deref(),
            Some("ptrace_not_permitted")
        );
        assert_eq!(
            telemetry.process_events_backend.as_deref(),
            Some(PROCESS_EVENTS_BACKEND)
        );
    }

    #[test]
    fn helper_scripts_attach_before_execing_inner_command() {
        let project_attach = PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT
            .find("attach_to_cgroup_or_fail")
            .expect("project helper should define attach preamble");
        let project_exec = PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT
            .rfind("exec sh -c \"$SUBSTRATE_INNER_CMD\"")
            .expect("project helper should exec inner command");
        assert!(
            project_attach < project_exec,
            "project helper must attach to the cgroup before execing the inner command"
        );

        let fallback_attach = WORLD_DEPS_BIND_MOUNT_FALLBACK_SCRIPT
            .find("attach_to_cgroup_or_fail")
            .expect("fallback helper should define attach preamble");
        let fallback_exec = WORLD_DEPS_BIND_MOUNT_FALLBACK_SCRIPT
            .rfind("exec sh -c \"$SUBSTRATE_INNER_CMD\"")
            .expect("fallback helper should exec inner command");
        assert!(
            fallback_attach < fallback_exec,
            "fallback helper must attach to the cgroup before execing the inner command"
        );
    }

    #[cfg(target_os = "linux")]
    mod linux {
        use super::super::*;
        use std::collections::HashMap;
        use tempfile::tempdir;

        #[test]
        fn read_only_bind_mount_blocks_absolute_project_writes() {
            let merged = tempdir().expect("merged tempdir");
            let project = tempdir().expect("project tempdir");

            let mount = ProjectBindMount {
                merged_dir: merged.path(),
                project_dir: project.path(),
                desired_cwd: project.path(),
                fs_mode: WorldFsMode::ReadOnly,
            };

            let env: HashMap<String, String> = HashMap::new();
            let cmd = r#"touch "$SUBSTRATE_MOUNT_PROJECT_DIR/abs_escape.txt""#;

            let output = match execute_shell_command_with_project_bind_mount(
                cmd,
                mount,
                &env,
                true,
                CgroupAttachPolicy::optional("project_bind_mount"),
                None,
            ) {
                Ok(output) => output,
                Err(err) => {
                    let message = err.to_string();
                    if message.contains("Operation not permitted")
                        || message.contains("EPERM")
                        || message.contains("unshare")
                    {
                        println!("Skipping bind-mount caging test: {}", message);
                        return;
                    }
                    panic!("unexpected error running bind-mount wrapper: {:#}", err);
                }
            };

            assert!(
                !output.status.success(),
                "expected read-only bind mount to reject writes via absolute project path"
            );
            assert!(
                !project.path().join("abs_escape.txt").exists(),
                "file should not appear in host project dir"
            );
        }

        #[test]
        fn project_bind_mount_env_map_preserves_world_deps_host_root() {
            let merged = tempdir().expect("merged tempdir");
            let project = tempdir().expect("project tempdir");
            let shared_world_deps_root = tempdir().expect("shared world-deps tempdir");

            let mount = ProjectBindMount {
                merged_dir: merged.path(),
                project_dir: project.path(),
                desired_cwd: project.path(),
                fs_mode: WorldFsMode::Writable,
            };

            let mut env: HashMap<String, String> = HashMap::new();
            env.insert(
                "SUBSTRATE_WORLD_DEPS_HOST_ROOT".to_string(),
                shared_world_deps_root.path().display().to_string(),
            );
            let env_map = project_bind_mount_env_map(
                "true",
                &mount,
                &env,
                false,
                CgroupAttachPolicy::optional("project_bind_mount"),
            );

            assert_eq!(
                env_map.get("SUBSTRATE_WORLD_DEPS_HOST_ROOT"),
                Some(&shared_world_deps_root.path().display().to_string()),
                "expected primary bind mount env setup to preserve the configured shared world-deps root"
            );
        }

        #[test]
        fn project_bind_mount_env_map_only_injects_cgroup_target_when_required() {
            let merged = tempdir().expect("merged tempdir");
            let project = tempdir().expect("project tempdir");
            let cgroup = tempdir().expect("cgroup tempdir");

            let mount = ProjectBindMount {
                merged_dir: merged.path(),
                project_dir: project.path(),
                desired_cwd: project.path(),
                fs_mode: WorldFsMode::Writable,
            };

            let optional_env_map = project_bind_mount_env_map(
                "true",
                &mount,
                &HashMap::new(),
                false,
                CgroupAttachPolicy::optional("project_bind_mount"),
            );
            let required_env_map = project_bind_mount_env_map(
                "true",
                &mount,
                &HashMap::new(),
                false,
                CgroupAttachPolicy::required("project_bind_mount", cgroup.path()),
            );

            assert!(
                !optional_env_map.contains_key("SUBSTRATE_CGROUP_PROCS_PATH"),
                "expected primary bind mount env setup to omit cgroup attach config when attach is optional"
            );
            assert!(
                !optional_env_map.contains_key("SUBSTRATE_CGROUP_ATTACH_REQUIRED"),
                "expected primary bind mount env setup to omit attach-required marker when attach is optional"
            );
            assert_eq!(
                required_env_map.get("SUBSTRATE_CGROUP_ATTACH_REQUIRED"),
                Some(&"1".to_string())
            );
            assert_eq!(
                required_env_map.get("SUBSTRATE_CGROUP_PROCS_PATH"),
                Some(&cgroup.path().join("cgroup.procs").display().to_string()),
                "expected primary bind mount env setup to inject the cgroup.procs target when attach is required"
            );
            assert!(
                !required_env_map.contains_key("SUBSTRATE_WORLD_DEPS_HOST_ROOT"),
                "expected primary bind mount env setup to keep relying on the shared script default world-deps root"
            );
        }

        #[test]
        fn mount_namespace_setup_failure_matches_unshare_errors() {
            assert_eq!(
                mount_namespace_setup_failure(
                    b"unshare: unshare failed: Operation not permitted\n"
                ),
                Some("unshare: unshare failed: Operation not permitted")
            );
        }

        #[test]
        fn mount_namespace_setup_failure_matches_mount_errors() {
            assert_eq!(
                mount_namespace_setup_failure(
                    b"mount: /tmp/foo: wrong fs type, bad option, bad superblock\n"
                ),
                Some("mount: /tmp/foo: wrong fs type, bad option, bad superblock")
            );
        }

        #[test]
        fn mount_namespace_setup_failure_ignores_command_failures() {
            assert_eq!(
                mount_namespace_setup_failure(b"sh: smoke-hello: not found\n"),
                None
            );
        }

        #[test]
        fn world_deps_bind_mount_fallback_exposes_default_guest_path() {
            let cwd = tempdir().expect("cwd tempdir");
            let world_deps_root = tempdir().expect("world-deps tempdir");
            let env: HashMap<String, String> = HashMap::new();
            let cmd = r#"mkdir -p /var/lib/substrate/world-deps/bin && printf '#!/bin/sh\nexit 0\n' > /var/lib/substrate/world-deps/bin/probe && chmod +x /var/lib/substrate/world-deps/bin/probe"#;

            let output = match execute_shell_command_with_world_deps_bind_mount(
                cmd,
                cwd.path(),
                &env,
                false,
                world_deps_root.path(),
                CgroupAttachPolicy::optional("world_deps_fallback"),
                None,
            ) {
                Ok(output) => output,
                Err(err) => {
                    let message = err.to_string();
                    if message.contains("Operation not permitted")
                        || message.contains("EPERM")
                        || message.contains("unshare")
                    {
                        println!("Skipping world-deps bind fallback test: {}", message);
                        return;
                    }
                    panic!(
                        "unexpected error running world-deps fallback wrapper: {:#}",
                        err
                    );
                }
            };

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("Operation not permitted")
                    || stderr.contains("EPERM")
                    || stderr.contains("unshare:")
                {
                    println!("Skipping world-deps bind fallback test: {stderr}");
                    return;
                }
            }

            assert!(
                output.status.success(),
                "expected world-deps fallback wrapper to succeed, stdout={}, stderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                world_deps_root.path().join("bin/probe").exists(),
                "expected helper to bind guest world-deps path to fallback root"
            );
        }

        #[test]
        fn stable_world_deps_fallback_root_is_deterministic_per_project() {
            let project = tempdir().expect("project tempdir");
            let first = stable_world_deps_fallback_root(project.path());
            let second = stable_world_deps_fallback_root(project.path());
            assert_eq!(first, second);
            assert!(
                first.starts_with(std::env::temp_dir()),
                "expected fallback root under temp dir, got {}",
                first.display()
            );
        }

        #[test]
        fn world_deps_bind_mount_fallback_persists_across_commands() {
            let project = tempdir().expect("project tempdir");
            let cwd = tempdir().expect("cwd tempdir");
            let world_deps_root = stable_world_deps_fallback_root(project.path());
            let mut env: HashMap<String, String> = HashMap::new();
            env.insert(
                "SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR".to_string(),
                "/var/lib/substrate/world-deps/bin".to_string(),
            );
            env.insert(
                "PATH".to_string(),
                "/var/lib/substrate/world-deps/bin:/usr/bin:/bin".to_string(),
            );

            let install = match execute_shell_command_with_world_deps_bind_mount(
                r#"mkdir -p /var/lib/substrate/world-deps/bin && printf '#!/bin/sh\necho smoke-hello\n' > /var/lib/substrate/world-deps/bin/smoke-hello && chmod +x /var/lib/substrate/world-deps/bin/smoke-hello"#,
                cwd.path(),
                &env,
                false,
                &world_deps_root,
                CgroupAttachPolicy::optional("world_deps_fallback"),
                None,
            ) {
                Ok(output) => output,
                Err(err) => {
                    let message = err.to_string();
                    if message.contains("Operation not permitted")
                        || message.contains("EPERM")
                        || message.contains("unshare")
                    {
                        println!("Skipping persistent world-deps fallback test: {}", message);
                        return;
                    }
                    panic!("unexpected error running install command: {:#}", err);
                }
            };

            if !install.status.success() {
                let stderr = String::from_utf8_lossy(&install.stderr);
                if stderr.contains("Operation not permitted")
                    || stderr.contains("EPERM")
                    || stderr.contains("unshare:")
                {
                    println!("Skipping persistent world-deps fallback test: {stderr}");
                    return;
                }
            }
            assert!(
                install.status.success(),
                "expected install command to succeed, stdout={}, stderr={}",
                String::from_utf8_lossy(&install.stdout),
                String::from_utf8_lossy(&install.stderr)
            );

            let probe = execute_shell_command_with_world_deps_bind_mount(
                "smoke-hello",
                cwd.path(),
                &env,
                false,
                &world_deps_root,
                CgroupAttachPolicy::optional("world_deps_fallback"),
                None,
            )
            .expect("probe command should run");

            assert!(
                probe.status.success(),
                "expected probe command to succeed, stdout={}, stderr={}",
                String::from_utf8_lossy(&probe.stdout),
                String::from_utf8_lossy(&probe.stderr)
            );
            assert_eq!(String::from_utf8_lossy(&probe.stdout).trim(), "smoke-hello");
        }

        #[test]
        fn world_deps_bind_mount_env_map_only_injects_cgroup_target_when_required() {
            let cwd = tempdir().expect("cwd tempdir");
            let world_deps_root = tempdir().expect("world-deps root");
            let cgroup = tempdir().expect("cgroup tempdir");

            let optional_env_map = world_deps_bind_mount_env_map(
                "true",
                cwd.path(),
                &HashMap::new(),
                false,
                world_deps_root.path(),
                CgroupAttachPolicy::optional("world_deps_fallback"),
            );
            let required_env_map = world_deps_bind_mount_env_map(
                "true",
                cwd.path(),
                &HashMap::new(),
                false,
                world_deps_root.path(),
                CgroupAttachPolicy::required("world_deps_fallback", cgroup.path()),
            );

            assert!(
                !optional_env_map.contains_key("SUBSTRATE_CGROUP_PROCS_PATH"),
                "expected fallback env setup to omit cgroup attach config when attach is optional"
            );
            assert_eq!(
                required_env_map.get("SUBSTRATE_CGROUP_ATTACH_REQUIRED"),
                Some(&"1".to_string())
            );
            assert_eq!(
                required_env_map.get("SUBSTRATE_CGROUP_PROCS_PATH"),
                Some(&cgroup.path().join("cgroup.procs").display().to_string()),
                "expected fallback env setup to inject the cgroup.procs target when attach is required"
            );
        }

        #[test]
        fn project_bind_mount_fails_closed_when_cgroup_attach_target_is_missing() {
            let merged = tempdir().expect("merged tempdir");
            let project = tempdir().expect("project tempdir");
            let cgroup = tempdir().expect("cgroup tempdir");
            let mount = ProjectBindMount {
                merged_dir: merged.path(),
                project_dir: project.path(),
                desired_cwd: project.path(),
                fs_mode: WorldFsMode::Writable,
            };

            let output = match execute_shell_command_with_project_bind_mount(
                "true",
                mount,
                &HashMap::new(),
                false,
                CgroupAttachPolicy::required("project_bind_mount", cgroup.path()),
                None,
            ) {
                Ok(output) => output,
                Err(err) => {
                    let message = err.to_string();
                    if message.contains("Operation not permitted")
                        || message.contains("EPERM")
                        || message.contains("unshare")
                    {
                        println!("Skipping cgroup attach project helper test: {}", message);
                        return;
                    }
                    panic!("unexpected error running project helper: {:#}", err);
                }
            };

            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success(),
                "expected missing cgroup target to fail before command execution"
            );
            assert!(
                stderr.contains(
                    "substrate: error: project_bind_mount: cgroup attach target does not exist"
                ),
                "expected deterministic project helper attach failure, got stderr={stderr}"
            );
        }

        #[test]
        fn world_deps_bind_mount_fails_closed_when_cgroup_attach_target_is_missing() {
            let cwd = tempdir().expect("cwd tempdir");
            let world_deps_root = tempdir().expect("world-deps root");
            let cgroup = tempdir().expect("cgroup tempdir");

            let output = match execute_shell_command_with_world_deps_bind_mount(
                "true",
                cwd.path(),
                &HashMap::new(),
                false,
                world_deps_root.path(),
                CgroupAttachPolicy::required("world_deps_fallback", cgroup.path()),
                None,
            ) {
                Ok(output) => output,
                Err(err) => {
                    let message = err.to_string();
                    if message.contains("Operation not permitted")
                        || message.contains("EPERM")
                        || message.contains("unshare")
                    {
                        println!("Skipping cgroup attach fallback helper test: {}", message);
                        return;
                    }
                    panic!("unexpected error running fallback helper: {:#}", err);
                }
            };

            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success(),
                "expected missing cgroup target to fail before command execution"
            );
            assert!(
                stderr.contains(
                    "substrate: error: world_deps_fallback: cgroup attach target does not exist"
                ),
                "expected deterministic fallback helper attach failure, got stderr={stderr}"
            );
        }
    }
}
