#![cfg(any(target_os = "linux", target_os = "macos"))]

#[path = "support/mod.rs"]
mod support;

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde_json::{json, Value};
use serial_test::serial;
use std::fs;
use std::io::{BufRead, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use support::{
    binary_path, ensure_substrate_built, persist_runtime_alert_for_substrate_home,
    substrate_shell_driver,
};
#[cfg(target_os = "linux")]
use support::{MemberDispatchStreamScript, ReplWorldAgentStub, StreamBehavior};
use tempfile::TempDir;

const PURE_AGENT_PROTOCOL: &str = "substrate.agent.session";
#[cfg(unix)]
const PRIVATE_STOP_UNIX_PATH_MAX: usize = 100;

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

struct AgentControlFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    workspace_root: PathBuf,
    fake_codex: PathBuf,
}

impl AgentControlFixture {
    fn new() -> Self {
        Self::new_with_fake_codex(write_fake_codex_script)
    }

    fn new_with_fake_codex(script_writer: fn(&Path) -> PathBuf) -> Self {
        let temp = tempfile::Builder::new()
            .prefix("sac-")
            .tempdir_in("/tmp")
            .expect("allocate short temp dir");
        let home = temp.path().join("h");
        let substrate_home = temp.path().join("s");
        let workspace_root = temp.path().join("w");
        fs::create_dir_all(&home).expect("create HOME");
        fs::create_dir_all(&substrate_home).expect("create SUBSTRATE_HOME");
        fs::create_dir_all(substrate_home.join("shims")).expect("create shims dir");
        fs::create_dir_all(&workspace_root).expect("create workspace root");
        fs::write(substrate_home.join("trace.jsonl"), "").expect("seed trace");
        let fake_codex = script_writer(temp.path());
        Self {
            _temp: temp,
            home,
            substrate_home,
            workspace_root,
            fake_codex,
        }
    }

    fn command(&self) -> assert_cmd::Command {
        let mut cmd = substrate_shell_driver();
        cmd.env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_HOME", &self.substrate_home)
            .env("SUBSTRATE_MANAGER_MANIFEST", manager_manifest_path())
            .env("SHIM_TRACE_LOG", self.trace_path());
        cmd
    }

    fn init_workspace(&self) {
        let output = self
            .command()
            .arg("workspace")
            .arg("init")
            .arg(&self.workspace_root)
            .arg("--force")
            .output()
            .expect("run workspace init");
        assert!(
            output.status.success(),
            "workspace init should succeed: {output:?}"
        );
    }

    fn write_runtime_inventory(&self, include_world_backend: bool) {
        fs::create_dir_all(self.substrate_home.join("agents")).expect("create agents dir");
        let allowed_backends = if include_world_backend {
            "    - cli:codex\n    - cli:claude_code\n"
        } else {
            "    - cli:codex\n"
        };
        fs::write(
            self.substrate_home.join("config.yaml"),
            "agents:\n  enabled: true\n  hub:\n    orchestrator_agent_id: codex\n  toolbox:\n    enabled: true\n    bind:\n      transport: uds\n",
        )
        .expect("write config.yaml");
        fs::write(
            self.substrate_home.join("policy.yaml"),
            format!(
                "id: test-global-policy\nname: Test Global Policy\nworld_fs:\n  host_visible: true\n  fail_closed:\n    routing: true\n  write:\n    enabled: true\nnet_allowed: []\ncmd_allowed: []\ncmd_denied: []\ncmd_isolated: []\nrequire_approval: false\nallow_shell_operators: true\nlimits:\n  max_memory_mb: null\n  max_cpu_percent: null\n  max_runtime_ms: null\n  max_egress_bytes: null\nmetadata: {{}}\nagents:\n  allowed_backends:\n{allowed_backends}",
            ),
        )
        .expect("write policy.yaml");
        fs::write(
            self.workspace_root.join(".substrate-profile"),
            "id: test-policy\nname: Test Policy\nworld_fs:\n  host_visible: true\n  fail_closed:\n    routing: true\n  write:\n    enabled: true\nnet_allowed: []\ncmd_allowed: []\ncmd_denied: []\ncmd_isolated: []\nrequire_approval: false\nallow_shell_operators: true\nlimits:\n  max_memory_mb: null\n  max_cpu_percent: null\n  max_runtime_ms: null\n  max_egress_bytes: null\nmetadata: {}\n",
        )
        .expect("write .substrate-profile");
        fs::write(
            self.substrate_home.join("agents/codex.yaml"),
            cli_agent_file("codex", "host", &self.fake_codex),
        )
        .expect("write codex agent file");
        if include_world_backend {
            fs::write(
                self.substrate_home.join("agents/claude_code.yaml"),
                cli_agent_file("claude_code", "world", &self.fake_codex),
            )
            .expect("write claude_code agent file");
        }
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command()
            .current_dir(&self.workspace_root)
            .args(args)
            .output()
            .expect("run substrate command")
    }

    fn spawn(&self, args: &[&str]) -> Child {
        ensure_substrate_built();

        std::process::Command::new(binary_path())
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_HOME", &self.substrate_home)
            .env("SUBSTRATE_MANAGER_MANIFEST", manager_manifest_path())
            .env("SHIM_TRACE_LOG", self.trace_path())
            .current_dir(&self.workspace_root)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn substrate command")
    }

    fn trace_path(&self) -> PathBuf {
        self.substrate_home.join("trace.jsonl")
    }

    fn load_orchestration_session(&self, orchestration_session_id: &str) -> Value {
        read_json_file(&canonical_orchestration_session_path(
            &self.substrate_home,
            orchestration_session_id,
        ))
    }

    fn load_participant(&self, orchestration_session_id: &str, participant_id: &str) -> Value {
        read_json_file(&canonical_participant_manifest_path(
            &self.substrate_home,
            orchestration_session_id,
            participant_id,
        ))
    }

    fn reset_fake_codex_state(&self) {
        let state_path = self
            .fake_codex
            .parent()
            .expect("fake codex parent")
            .join("fake-codex.count");
        let _ = fs::remove_file(state_path);
    }

    fn fake_codex_args_path(&self, invocation: u32) -> PathBuf {
        self.fake_codex
            .parent()
            .expect("fake codex parent")
            .join(format!("fake-codex-{invocation}.args"))
    }

    fn fake_codex_stdin_path(&self, invocation: u32) -> PathBuf {
        self.fake_codex
            .parent()
            .expect("fake codex parent")
            .join(format!("fake-codex-{invocation}.stdin"))
    }

    fn read_fake_codex_args(&self, invocation: u32) -> Vec<String> {
        fs::read_to_string(self.fake_codex_args_path(invocation))
            .unwrap_or_else(|err| panic!("read fake codex args {invocation}: {err}"))
            .lines()
            .map(|line| line.to_string())
            .collect()
    }

    fn read_fake_codex_stdin(&self, invocation: u32) -> String {
        fs::read_to_string(self.fake_codex_stdin_path(invocation))
            .unwrap_or_else(|err| panic!("read fake codex stdin {invocation}: {err}"))
    }
}

fn cli_agent_file(agent_id: &str, scope: &str, binary: &Path) -> String {
    format!(
        "version: 1\nid: {agent_id}\nconfig:\n  kind: cli\n  enabled: true\n  protocol: {PURE_AGENT_PROTOCOL}\n  execution:\n    scope: {scope}\n  cli:\n    binary: {}\n    mode: persistent\n  capabilities:\n    session_start: true\n    session_resume: true\n    session_fork: true\n    session_stop: true\n    status_snapshot: true\n    event_stream: true\n    llm: true\n    mcp_client: false\n",
        binary.display()
    )
}

fn write_fake_codex_script(dir: &Path) -> PathBuf {
    let path = dir.join("fake-codex.sh");
    let count_path = dir.join("fake-codex.count");
    let body = format!(
        "#!/bin/sh\nSTATE_FILE='{}'\nSCRIPT_DIR='{}'\ncount=0\nif [ -f \"$STATE_FILE\" ]; then\n  count=$(cat \"$STATE_FILE\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$STATE_FILE\"\nprintf '%s\\n' \"$@\" > \"$SCRIPT_DIR/fake-codex-$count.args\"\ncat > \"$SCRIPT_DIR/fake-codex-$count.stdin\"\nif [ \"$count\" -eq 1 ]; then\n  trap 'exit 0' INT TERM\n  printf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-test\"}}\\r\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\"}}\\r\\n'\n  printf '{{\"type\":\"item.completed\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\",\"item_id\":\"msg-1\",\"status\":\"completed\",\"item_type\":\"agent_message\",\"content\":{{\"text\":\"startup prompt success\"}}}}\\r\\n'\n  printf '{{\"type\":\"turn.completed\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\"}}\\r\\n'\n  while :; do sleep 1; done\nfi\nprintf '{{\"type\":\"thread.resumed\",\"thread_id\":\"thread-test\"}}\\r\\n'\nprintf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-%s\"}}\\r\\n' \"$count\"\nprintf '{{\"type\":\"item.completed\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-%s\",\"item_id\":\"msg-%s\",\"status\":\"completed\",\"item_type\":\"agent_message\",\"content\":{{\"text\":\"follow-up prompt success\"}}}}\\r\\n' \"$count\" \"$count\"\nprintf '{{\"type\":\"turn.completed\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-%s\"}}\\r\\n' \"$count\"\nexit 0\n",
        count_path.display()
        ,
        dir.display()
    );
    fs::write(&path, body).expect("write fake codex script");
    let mut perms = fs::metadata(&path)
        .expect("fake codex metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set fake codex permissions");
    path
}

fn write_fake_codex_script_exit_after_startup_prompt(dir: &Path) -> PathBuf {
    let path = dir.join("fake-codex-exit-after-startup-prompt.sh");
    let count_path = dir.join("fake-codex-exit-after-startup-prompt.count");
    let body = format!(
        "#!/bin/sh\nSTATE_FILE='{}'\nSCRIPT_DIR='{}'\ncount=0\nif [ -f \"$STATE_FILE\" ]; then\n  count=$(cat \"$STATE_FILE\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$STATE_FILE\"\nprintf '%s\\n' \"$@\" > \"$SCRIPT_DIR/fake-codex-$count.args\"\ncat > \"$SCRIPT_DIR/fake-codex-$count.stdin\"\nif [ \"$count\" -eq 1 ]; then\n  printf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-test\"}}\\r\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\"}}\\r\\n'\n  printf '{{\"type\":\"item.completed\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\",\"item_id\":\"msg-1\",\"status\":\"completed\",\"item_type\":\"agent_message\",\"content\":{{\"text\":\"startup prompt success\"}}}}\\r\\n'\n  printf '{{\"type\":\"turn.completed\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\"}}\\r\\n'\n  exit 0\nfi\nprintf '{{\"type\":\"thread.resumed\",\"thread_id\":\"thread-test\"}}\\r\\n'\nprintf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-%s\"}}\\r\\n' \"$count\"\nprintf '{{\"type\":\"item.completed\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-%s\",\"item_id\":\"msg-%s\",\"status\":\"completed\",\"item_type\":\"agent_message\",\"content\":{{\"text\":\"follow-up prompt success\"}}}}\\r\\n' \"$count\" \"$count\"\nprintf '{{\"type\":\"turn.completed\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-%s\"}}\\r\\n' \"$count\"\nexit 0\n",
        count_path.display(),
        dir.display()
    );
    fs::write(&path, body).expect("write exit-after-startup fake codex script");
    let mut perms = fs::metadata(&path)
        .expect("exit-after-startup fake codex metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set exit-after-startup fake codex permissions");
    path
}

fn write_fake_codex_script_turn_requires_helper_cancel_after_prompt(dir: &Path) -> PathBuf {
    let path = dir.join("fake-codex-turn-requires-helper-cancel.sh");
    let count_path = dir.join("fake-codex-turn-requires-helper-cancel.count");
    let body = format!(
        "#!/bin/sh\nSTATE_FILE='{}'\nSCRIPT_DIR='{}'\ncount=0\nif [ -f \"$STATE_FILE\" ]; then\n  count=$(cat \"$STATE_FILE\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$STATE_FILE\"\nprintf '%s\\n' \"$@\" > \"$SCRIPT_DIR/fake-codex-$count.args\"\ncat > \"$SCRIPT_DIR/fake-codex-$count.stdin\"\nif [ \"$count\" -eq 1 ]; then\n  trap 'exit 0' INT TERM\n  printf '{{\"type\":\"thread.resumed\",\"thread_id\":\"thread-test\"}}\\r\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-bootstrap\"}}\\r\\n'\n  while :; do sleep 1; done\nfi\nif [ \"$count\" -eq 2 ]; then\n  printf '{{\"type\":\"thread.resumed\",\"thread_id\":\"thread-test\"}}\\r\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-%s\"}}\\r\\n' \"$count\"\n  printf '{{\"type\":\"item.completed\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-%s\",\"item_id\":\"msg-%s\",\"status\":\"completed\",\"item_type\":\"agent_message\",\"content\":{{\"text\":\"follow-up prompt success\"}}}}\\r\\n' \"$count\" \"$count\"\n  printf '{{\"type\":\"turn.completed\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-%s\"}}\\r\\n' \"$count\"\n  exit 0\nfi\ntrap 'exit 0' INT TERM\nprintf '{{\"type\":\"thread.resumed\",\"thread_id\":\"thread-test\"}}\\r\\n'\nprintf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-%s\"}}\\r\\n' \"$count\"\nwhile :; do sleep 1; done\n",
        count_path.display(),
        dir.display(),
    );
    fs::write(&path, body).expect("write helper-cancel fake codex script");
    let mut perms = fs::metadata(&path)
        .expect("helper-cancel fake codex metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set helper-cancel fake codex permissions");
    path
}

fn write_fake_codex_script_clean_bootstrap_then_prompt_resume(dir: &Path) -> PathBuf {
    let path = dir.join("fake-codex-clean-bootstrap-then-prompt-resume.sh");
    let count_path = dir.join("fake-codex-clean-bootstrap-then-prompt-resume.count");
    let start_ready_path = dir.join("fake-codex-clean-bootstrap-start.ready");
    let turn_ready_path = dir.join("fake-codex-clean-bootstrap-turn.ready");
    let body = format!(
        "#!/bin/sh\nSTATE_FILE='{}'\nSTART_READY='{}'\nTURN_READY='{}'\ncount=0\nif [ -f \"$STATE_FILE\" ]; then\n  count=$(cat \"$STATE_FILE\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$STATE_FILE\"\nif [ \"$count\" -eq 1 ]; then\n  printf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-test\"}}\\r\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\"}}\\r\\n'\n  exit 0\nfi\nif [ \"$count\" -eq 2 ]; then\n  trap 'exit 0' INT TERM\n  rm -f \"$START_READY\"\n  printf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-test\"}}\\r\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-2\"}}\\r\\n'\n  while [ ! -f \"$START_READY\" ]; do sleep 0.1; done\n  exit 0\nfi\nif [ \"$count\" -eq 3 ]; then\n  : > \"$START_READY\"\n  printf 'start prompt success\\n'\n  exit 0\nfi\nif [ \"$count\" -eq 4 ]; then\n  trap 'exit 0' INT TERM\n  rm -f \"$TURN_READY\"\n  printf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-test\"}}\\r\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-4\"}}\\r\\n'\n  while [ ! -f \"$TURN_READY\" ]; do sleep 0.1; done\n  exit 0\nfi\nif [ \"$count\" -eq 5 ]; then\n  : > \"$TURN_READY\"\n  printf 'turn prompt success\\n'\n  exit 0\nfi\ntrap 'exit 0' INT TERM\nprintf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-test\"}}\\r\\n'\nprintf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-%s\"}}\\r\\n' \"$count\"\nwhile :; do sleep 1; done\n",
        count_path.display(),
        start_ready_path.display(),
        turn_ready_path.display()
    );
    fs::write(&path, body).expect("write clean bootstrap fake codex script");
    let mut perms = fs::metadata(&path)
        .expect("clean bootstrap fake codex metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set clean bootstrap fake codex permissions");
    path
}

fn write_fake_codex_script_without_session_handle(dir: &Path) -> PathBuf {
    let path = dir.join("fake-codex-no-session-handle.sh");
    let body = "#!/bin/sh\nprintf 'bootstrap-without-session-handle\\n'\n";
    fs::write(&path, body).expect("write fake codex script without session handle");
    let mut perms = fs::metadata(&path)
        .expect("fake codex metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set fake codex permissions");
    path
}

fn write_fake_codex_script_with_delayed_prompt_completion(dir: &Path) -> PathBuf {
    let path = dir.join("fake-codex-delayed-prompt-completion.sh");
    let count_path = dir.join("fake-codex-delayed-prompt-completion.count");
    let body = format!(
        "#!/bin/sh\nSTATE_FILE='{}'\ncount=0\nif [ -f \"$STATE_FILE\" ]; then\n  count=$(cat \"$STATE_FILE\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$STATE_FILE\"\nif [ \"$count\" -eq 1 ]; then\n  trap 'exit 0' INT TERM\n  printf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-test\"}}\\r\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\"}}\\r\\n'\n  while :; do sleep 1; done\nfi\nsleep 5\nexit 0\n",
        count_path.display()
    );
    fs::write(&path, body).expect("write delayed prompt fake codex script");
    let mut perms = fs::metadata(&path)
        .expect("delayed prompt fake codex metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set delayed prompt fake codex permissions");
    path
}

fn write_fake_codex_script_with_slow_startup_prompt_completion(dir: &Path) -> PathBuf {
    let path = dir.join("fake-codex-slow-startup-prompt-completion.sh");
    let count_path = dir.join("fake-codex-slow-startup-prompt-completion.count");
    let body = format!(
        "#!/bin/sh\nSTATE_FILE='{}'\nSCRIPT_DIR='{}'\ncount=0\nif [ -f \"$STATE_FILE\" ]; then\n  count=$(cat \"$STATE_FILE\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$STATE_FILE\"\nprintf '%s\\n' \"$@\" > \"$SCRIPT_DIR/fake-codex-$count.args\"\ncat > \"$SCRIPT_DIR/fake-codex-$count.stdin\"\nif [ \"$count\" -eq 1 ]; then\n  printf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-test\"}}\\r\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\"}}\\r\\n'\n  sleep 11\n  printf '{{\"type\":\"item.completed\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\",\"item_id\":\"msg-1\",\"status\":\"completed\",\"item_type\":\"agent_message\",\"content\":{{\"text\":\"slow startup prompt success\"}}}}\\r\\n'\n  printf '{{\"type\":\"turn.completed\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-1\"}}\\r\\n'\n  exit 0\nfi\nprintf '{{\"type\":\"thread.resumed\",\"thread_id\":\"thread-test\"}}\\r\\n'\nprintf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-%s\"}}\\r\\n' \"$count\"\nprintf '{{\"type\":\"item.completed\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-%s\",\"item_id\":\"msg-%s\",\"status\":\"completed\",\"item_type\":\"agent_message\",\"content\":{{\"text\":\"follow-up prompt success\"}}}}\\r\\n' \"$count\" \"$count\"\nprintf '{{\"type\":\"turn.completed\",\"thread_id\":\"thread-test\",\"turn_id\":\"turn-%s\"}}\\r\\n' \"$count\"\nexit 0\n",
        count_path.display(),
        dir.display()
    );
    fs::write(&path, body).expect("write slow-startup fake codex script");
    let mut perms = fs::metadata(&path)
        .expect("slow-startup fake codex metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set slow-startup fake codex permissions");
    path
}

fn parse_json_output(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

fn parse_ndjson_output(output: &Output) -> Vec<Value> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("stdout should be valid NDJSON"))
        .collect()
}

fn find_ndjson_record<'a>(records: &'a [Value], kind: &str) -> &'a Value {
    records
        .iter()
        .find(|record| record.get("kind").and_then(Value::as_str) == Some(kind))
        .unwrap_or_else(|| panic!("missing NDJSON record kind={kind}: {records:?}"))
}

fn status_sessions(json: &Value) -> &[Value] {
    json["sessions"]
        .as_array()
        .map(Vec::as_slice)
        .expect("sessions should be an array")
}

fn find_status_session_by_orchestration_session_id<'a>(
    sessions: &'a [Value],
    orchestration_session_id: &str,
) -> &'a Value {
    sessions
        .iter()
        .find(|session| {
            session
                .get("orchestration_session_id")
                .and_then(Value::as_str)
                == Some(orchestration_session_id)
        })
        .unwrap_or_else(|| {
            panic!(
                "missing status row orchestration_session_id={orchestration_session_id}: {sessions:?}"
            )
        })
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn assert_empty_warnings(json: &Value) {
    assert_eq!(
        json.get("warnings").and_then(Value::as_array).map(Vec::len),
        Some(0),
        "control success output must keep warnings empty: {json}"
    );
}

fn canonical_orchestration_session_path(
    substrate_home: &Path,
    orchestration_session_id: &str,
) -> PathBuf {
    substrate_home
        .join("run/agent-hub/sessions")
        .join(orchestration_session_id)
        .join("session.json")
}

fn canonical_participant_manifest_path(
    substrate_home: &Path,
    orchestration_session_id: &str,
    participant_id: &str,
) -> PathBuf {
    substrate_home
        .join("run/agent-hub/sessions")
        .join(orchestration_session_id)
        .join("participants")
        .join(format!("{participant_id}.json"))
}

#[cfg(target_os = "linux")]
fn canonical_participants_dir(substrate_home: &Path, orchestration_session_id: &str) -> PathBuf {
    substrate_home
        .join("run/agent-hub/sessions")
        .join(orchestration_session_id)
        .join("participants")
}

fn read_json_file(path: &Path) -> Value {
    serde_json::from_str(
        &fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display())),
    )
    .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()))
}

fn compact_stop_transport_fragment(id: &str) -> String {
    let normalized = id
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>();
    if normalized.len() <= 12 {
        return normalized;
    }
    format!(
        "{}{}",
        &normalized[..6],
        &normalized[normalized.len() - 6..]
    )
}

fn stop_transport_path(
    fixture: &AgentControlFixture,
    orchestration_session_id: &str,
    participant_id: &str,
) -> PathBuf {
    let socket_name = format!(
        "{}-{}.sock",
        compact_stop_transport_fragment(orchestration_session_id),
        compact_stop_transport_fragment(participant_id)
    );
    let preferred = fixture
        .substrate_home
        .join("run/agent-hub/handles/stop")
        .join(&socket_name);
    if preferred.as_os_str().len() > PRIVATE_STOP_UNIX_PATH_MAX {
        return PathBuf::from("/tmp")
            .join("substrate-agent-hub-stop")
            .join(socket_name);
    }
    preferred
}

fn wait_for_path(path: &Path, timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if path.exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    false
}

#[cfg(unix)]
fn pid_is_alive(pid: u32) -> bool {
    let rc = unsafe { libc::kill(pid as libc::pid_t, 0) };
    rc == 0
        || matches!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::EPERM)
        )
}

#[cfg(unix)]
fn terminate_pid(pid: u32) {
    let pid = pid as libc::pid_t;
    unsafe {
        let _ = libc::kill(pid, libc::SIGTERM);
    }
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        if !pid_is_alive(pid as u32) {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    unsafe {
        let _ = libc::kill(pid, libc::SIGKILL);
    }
}

fn wait_for_pid_exit(pid: u32, timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if !pid_is_alive(pid) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    false
}

fn wait_for_single_active_session(
    fixture: &AgentControlFixture,
    timeout: Duration,
) -> (String, String) {
    let start = Instant::now();
    while start.elapsed() < timeout {
        let sessions_dir = fixture.substrate_home.join("run/agent-hub/sessions");
        if let Ok(entries) = fs::read_dir(&sessions_dir) {
            let mut dirs = entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect::<Vec<_>>();
            dirs.sort();
            if let Some(dir) = dirs.into_iter().next() {
                let session = read_json_file(&dir.join("session.json"));
                if session.get("state").and_then(Value::as_str) == Some("active") {
                    if let Some(participant_id) = session
                        .get("active_session_handle_id")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                    {
                        if read_json_file(
                            &dir.join("participants")
                                .join(format!("{participant_id}.json")),
                        )
                        .get("state")
                        .and_then(Value::as_str)
                        .is_some_and(|state| matches!(state, "ready" | "running"))
                        {
                            return (
                                session["orchestration_session_id"]
                                    .as_str()
                                    .expect("session id")
                                    .to_string(),
                                participant_id,
                            );
                        }
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("timed out waiting for a single active orchestration session");
}

fn wait_for_session_posture(
    fixture: &AgentControlFixture,
    orchestration_session_id: &str,
    expected_posture: &str,
    timeout: Duration,
) -> Value {
    let start = Instant::now();
    while start.elapsed() < timeout {
        let session = fixture.load_orchestration_session(orchestration_session_id);
        if session.get("posture").and_then(Value::as_str) == Some(expected_posture) {
            return session;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!(
        "timed out waiting for session {} posture {}",
        orchestration_session_id, expected_posture
    );
}

fn wait_for_detached_resume_eligible_participant(
    fixture: &AgentControlFixture,
    orchestration_session_id: &str,
    participant_id: &str,
    timeout: Duration,
) -> Value {
    let start = Instant::now();
    while start.elapsed() < timeout {
        let participant = fixture.load_participant(orchestration_session_id, participant_id);
        let resume_eligible = participant
            .pointer("/internal/resume_eligible")
            .and_then(Value::as_bool);
        let attached_client_present = participant
            .pointer("/internal/attached_client_present")
            .and_then(Value::as_bool);
        if resume_eligible == Some(true) && attached_client_present == Some(false) {
            return participant;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!(
        "timed out waiting for participant {} in session {} to become detached and resume-eligible",
        participant_id, orchestration_session_id
    );
}

fn wait_for_successor_owner_pid(
    fixture: &AgentControlFixture,
    orchestration_session_id: &str,
    previous_participant_id: &str,
    timeout: Duration,
) -> u32 {
    let start = Instant::now();
    while start.elapsed() < timeout {
        let session = fixture.load_orchestration_session(orchestration_session_id);
        let participant_changed = session
            .get("active_session_handle_id")
            .and_then(Value::as_str)
            .is_some_and(|participant_id| participant_id != previous_participant_id);
        let shell_owner_pid = session
            .get("shell_owner_pid")
            .and_then(Value::as_u64)
            .map(|pid| pid as u32);
        if participant_changed && shell_owner_pid.is_some_and(pid_is_alive) {
            return shell_owner_pid.expect("owner pid");
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!(
        "timed out waiting for successor owner pid in orchestration session {}",
        orchestration_session_id
    );
}

#[cfg(target_os = "linux")]
fn session_participant_manifests(
    substrate_home: &Path,
    orchestration_session_id: &str,
) -> Vec<Value> {
    let participants_dir = canonical_participants_dir(substrate_home, orchestration_session_id);
    let Ok(entries) = fs::read_dir(participants_dir) else {
        return Vec::new();
    };
    let mut manifests = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("json"))
        .map(|entry| read_json_file(&entry.path()))
        .collect::<Vec<_>>();
    manifests.sort_by(|left, right| {
        left.get("participant_id")
            .and_then(Value::as_str)
            .cmp(&right.get("participant_id").and_then(Value::as_str))
    });
    manifests
}

#[cfg(target_os = "linux")]
fn participant_is_authoritative_live(manifest: &Value) -> bool {
    let Some(state) = manifest.get("state").and_then(Value::as_str) else {
        return false;
    };
    let live_state = matches!(
        state,
        "allocating" | "ready" | "running" | "restarting" | "stopping"
    );
    live_state
        && manifest
            .pointer("/internal/uaa_session_id")
            .and_then(Value::as_str)
            .is_some()
        && manifest
            .pointer("/internal/control_owner_retained")
            .and_then(Value::as_bool)
            == Some(true)
        && manifest
            .pointer("/internal/event_stream_active")
            .and_then(Value::as_bool)
            == Some(true)
        && manifest
            .pointer("/internal/completion_observer_retained")
            .and_then(Value::as_bool)
            == Some(true)
        && manifest
            .pointer("/internal/ownership_valid")
            .and_then(Value::as_bool)
            == Some(true)
        && manifest
            .pointer("/internal/terminal_observed_at")
            .is_none_or(Value::is_null)
}

#[cfg(target_os = "linux")]
fn authoritative_live_world_member_manifests_for_session(
    substrate_home: &Path,
    orchestration_session_id: &str,
) -> Vec<Value> {
    session_participant_manifests(substrate_home, orchestration_session_id)
        .into_iter()
        .filter(|manifest| manifest.get("role").and_then(Value::as_str) == Some("member"))
        .filter(|manifest| {
            manifest.pointer("/execution/scope").and_then(Value::as_str) == Some("world")
        })
        .filter(participant_is_authoritative_live)
        .collect()
}

#[cfg(target_os = "linux")]
fn wait_for_live_world_member_count(
    fixture: &AgentControlFixture,
    orchestration_session_id: &str,
    expected_count: usize,
    timeout: Duration,
) -> Vec<Value> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let members = authoritative_live_world_member_manifests_for_session(
            &fixture.substrate_home,
            orchestration_session_id,
        );
        if members.len() == expected_count {
            return members;
        }
        std::thread::sleep(Duration::from_millis(25));
    }

    panic!(
        "timed out waiting for authoritative live world member count == {expected_count}; got {:?}",
        authoritative_live_world_member_manifests_for_session(
            &fixture.substrate_home,
            orchestration_session_id
        ),
    );
}

fn write_json_file(path: &Path, value: &Value) {
    let parent = path.parent().expect("fixture json path should have parent");
    fs::create_dir_all(parent)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", parent.display()));
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("serialize fixture json"),
    )
    .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
}

fn write_active_orchestration_session(
    fixture: &AgentControlFixture,
    agent_id: &str,
    orchestration_session_id: &str,
    active_session_handle_id: &str,
    ts: &str,
) {
    write_json_file(
        &canonical_orchestration_session_path(&fixture.substrate_home, orchestration_session_id),
        &json!({
            "orchestration_session_id": orchestration_session_id,
            "shell_trace_session_id": "ses_agent_control",
            "workspace_root": fixture.workspace_root.display().to_string(),
            "shell_owner_pid": std::process::id(),
            "state": "active",
            "posture": "active_attached",
            "posture_changed_at": ts,
            "opened_at": ts,
            "last_active_at": ts,
            "orchestrator_agent_id": agent_id,
            "orchestrator_backend_id": format!("cli:{agent_id}"),
            "orchestrator_protocol": PURE_AGENT_PROTOCOL,
            "active_session_handle_id": active_session_handle_id,
            "attached_participant_id": active_session_handle_id,
            "pending_inbox_count": 0,
            "last_parked_at": Value::Null,
            "last_attention_at": Value::Null,
            "parked_reason": Value::Null,
            "latest_run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6fab",
            "world_id": Value::Null,
            "world_generation": Value::Null,
            "invalidation_reason": Value::Null,
            "closed_at": Value::Null
        }),
    );
}

fn write_parked_orchestration_session(
    fixture: &AgentControlFixture,
    agent_id: &str,
    orchestration_session_id: &str,
    active_session_handle_id: &str,
    ts: &str,
) {
    write_json_file(
        &canonical_orchestration_session_path(&fixture.substrate_home, orchestration_session_id),
        &json!({
            "orchestration_session_id": orchestration_session_id,
            "shell_trace_session_id": "ses_agent_control",
            "workspace_root": fixture.workspace_root.display().to_string(),
            "shell_owner_pid": std::process::id(),
            "state": "active",
            "posture": "parked_resumable",
            "posture_changed_at": ts,
            "opened_at": ts,
            "last_active_at": ts,
            "orchestrator_agent_id": agent_id,
            "orchestrator_backend_id": format!("cli:{agent_id}"),
            "orchestrator_protocol": PURE_AGENT_PROTOCOL,
            "active_session_handle_id": active_session_handle_id,
            "attached_participant_id": Value::Null,
            "pending_inbox_count": 0,
            "last_parked_at": ts,
            "last_attention_at": Value::Null,
            "parked_reason": "owner detached cleanly",
            "latest_run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6fab",
            "world_id": Value::Null,
            "world_generation": Value::Null,
            "invalidation_reason": Value::Null,
            "closed_at": Value::Null
        }),
    );
}

#[cfg(target_os = "linux")]
fn write_parked_world_orchestration_session(
    fixture: &AgentControlFixture,
    agent_id: &str,
    orchestration_session_id: &str,
    active_session_handle_id: &str,
    world_id: &str,
    world_generation: u64,
    ts: &str,
) {
    write_json_file(
        &canonical_orchestration_session_path(&fixture.substrate_home, orchestration_session_id),
        &json!({
            "orchestration_session_id": orchestration_session_id,
            "shell_trace_session_id": "ses_agent_control",
            "workspace_root": fixture.workspace_root.display().to_string(),
            "shell_owner_pid": std::process::id(),
            "state": "active",
            "posture": "parked_resumable",
            "posture_changed_at": ts,
            "opened_at": ts,
            "last_active_at": ts,
            "orchestrator_agent_id": agent_id,
            "orchestrator_backend_id": format!("cli:{agent_id}"),
            "orchestrator_protocol": PURE_AGENT_PROTOCOL,
            "active_session_handle_id": active_session_handle_id,
            "attached_participant_id": Value::Null,
            "pending_inbox_count": 0,
            "last_parked_at": ts,
            "last_attention_at": Value::Null,
            "parked_reason": "owner detached cleanly",
            "latest_run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6fab",
            "world_id": world_id,
            "world_generation": world_generation,
            "invalidation_reason": Value::Null,
            "closed_at": Value::Null
        }),
    );
}

#[allow(clippy::too_many_arguments)]
fn write_orchestration_session(
    fixture: &AgentControlFixture,
    agent_id: &str,
    orchestration_session_id: &str,
    active_session_handle_id: Option<&str>,
    state: &str,
    world_id: Option<&str>,
    world_generation: Option<u64>,
    ts: &str,
) {
    let (posture, attached_participant_id, last_parked_at, parked_reason, closed_at) = match state {
        "active" => (
            "active_attached",
            active_session_handle_id
                .map(Value::from)
                .unwrap_or(Value::Null),
            Value::Null,
            Value::Null,
            Value::Null,
        ),
        "stopped" | "failed" | "invalidated" => (
            "terminal",
            Value::Null,
            Value::Null,
            Value::Null,
            Value::String(ts.to_string()),
        ),
        _ => (
            "active_attached",
            active_session_handle_id
                .map(Value::from)
                .unwrap_or(Value::Null),
            Value::Null,
            Value::Null,
            Value::Null,
        ),
    };
    write_json_file(
        &canonical_orchestration_session_path(&fixture.substrate_home, orchestration_session_id),
        &json!({
            "orchestration_session_id": orchestration_session_id,
            "shell_trace_session_id": "ses_agent_control",
            "workspace_root": fixture.workspace_root.display().to_string(),
            "shell_owner_pid": std::process::id(),
            "state": state,
            "posture": posture,
            "posture_changed_at": ts,
            "opened_at": ts,
            "last_active_at": ts,
            "orchestrator_agent_id": agent_id,
            "orchestrator_backend_id": format!("cli:{agent_id}"),
            "orchestrator_protocol": PURE_AGENT_PROTOCOL,
            "active_session_handle_id": active_session_handle_id,
            "attached_participant_id": attached_participant_id,
            "pending_inbox_count": 0,
            "last_parked_at": last_parked_at,
            "last_attention_at": Value::Null,
            "parked_reason": parked_reason,
            "latest_run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6fab",
            "world_id": world_id,
            "world_generation": world_generation,
            "invalidation_reason": Value::Null,
            "closed_at": closed_at
        }),
    );
}

#[allow(clippy::too_many_arguments)]
fn write_runtime_participant(
    fixture: &AgentControlFixture,
    participant_id: &str,
    agent_id: &str,
    orchestration_session_id: &str,
    state: &str,
    ownership_valid: bool,
    uaa_session_id: Option<&str>,
    resumed_from_participant_id: Option<&str>,
    ts: &str,
) {
    let attached_client_present = ownership_valid;
    let resume_eligible = uaa_session_id.is_some() && state != "invalidated";
    let last_attached_at = if resume_eligible {
        Value::String(ts.to_string())
    } else {
        Value::Null
    };
    let last_detached_at = if !attached_client_present && resume_eligible {
        Value::String(ts.to_string())
    } else {
        Value::Null
    };
    let detach_reason = if !attached_client_present && resume_eligible {
        Value::String("owner detached cleanly".to_string())
    } else {
        Value::Null
    };
    write_json_file(
        &canonical_participant_manifest_path(
            &fixture.substrate_home,
            orchestration_session_id,
            participant_id,
        ),
        &json!({
            "participant_id": participant_id,
            "orchestration_session_id": orchestration_session_id,
            "agent_id": agent_id,
            "backend_id": format!("cli:{agent_id}"),
            "role": "orchestrator",
            "protocol": PURE_AGENT_PROTOCOL,
            "execution": { "scope": "host" },
            "state": state,
            "opened_at": ts,
            "last_transition_at": ts,
            "resumed_from_participant_id": resumed_from_participant_id,
            "internal": {
                "resolved_agent_kind": agent_id,
                "resolved_binary_path": fixture.fake_codex.display().to_string(),
                "shell_owner_pid": std::process::id(),
                "lease_token": format!("lease-{participant_id}"),
                "uaa_session_id": uaa_session_id,
                "latest_run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6fab",
                "cancel_supported": true,
                "control_owner_retained": ownership_valid,
                "event_stream_active": ownership_valid,
                "completion_observer_retained": ownership_valid,
                "ownership_mode": "attached_control",
                "ownership_valid": ownership_valid,
                "ownership_verified_at": ts,
                "last_heartbeat_at": ts,
                "last_event_at": ts,
                "attached_client_present": attached_client_present,
                "last_attached_at": last_attached_at,
                "last_detached_at": last_detached_at,
                "detach_reason": detach_reason,
                "resume_eligible": resume_eligible,
                "terminal_observed_at": Value::Null,
                "termination_reason": Value::Null,
                "last_error_bucket": Value::Null,
                "last_error_message": Value::Null
            }
        }),
    );
}

#[cfg(target_os = "linux")]
#[allow(clippy::too_many_arguments)]
fn write_world_member_participant(
    fixture: &AgentControlFixture,
    participant_id: &str,
    agent_id: &str,
    orchestration_session_id: &str,
    orchestrator_participant_id: &str,
    world_id: &str,
    world_generation: u64,
    state: &str,
    ownership_valid: bool,
    uaa_session_id: Option<&str>,
    ts: &str,
) {
    write_json_file(
        &canonical_participant_manifest_path(
            &fixture.substrate_home,
            orchestration_session_id,
            participant_id,
        ),
        &json!({
            "participant_id": participant_id,
            "orchestration_session_id": orchestration_session_id,
            "agent_id": agent_id,
            "backend_id": format!("cli:{agent_id}"),
            "role": "member",
            "protocol": PURE_AGENT_PROTOCOL,
            "execution": { "scope": "world" },
            "state": state,
            "opened_at": ts,
            "last_transition_at": ts,
            "parent_session_handle_id": Value::Null,
            "resumed_from_session_handle_id": Value::Null,
            "world_id": world_id,
            "world_generation": world_generation,
            "orchestrator_participant_id": orchestrator_participant_id,
            "internal": {
                "resolved_agent_kind": agent_id,
                "resolved_binary_path": fixture.fake_codex.display().to_string(),
                "shell_owner_pid": std::process::id(),
                "lease_token": format!("lease-{participant_id}"),
                "uaa_session_id": uaa_session_id,
                "latest_run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6fab",
                "cancel_supported": true,
                "control_owner_retained": ownership_valid,
                "event_stream_active": ownership_valid,
                "completion_observer_retained": ownership_valid,
                "ownership_mode": "member_runtime",
                "ownership_valid": ownership_valid,
                "ownership_verified_at": ts,
                "last_heartbeat_at": ts,
                "last_event_at": ts,
                "terminal_observed_at": Value::Null,
                "termination_reason": Value::Null,
                "last_error_bucket": Value::Null,
                "last_error_message": Value::Null
            }
        }),
    );
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
    fn spawn(fixture: &AgentControlFixture) -> Self {
        Self::spawn_with_options(fixture, None)
    }

    #[cfg(target_os = "linux")]
    fn spawn_with_world_socket(fixture: &AgentControlFixture, socket_path: &Path) -> Self {
        Self::spawn_with_options(fixture, Some(socket_path))
    }

    fn spawn_with_options(fixture: &AgentControlFixture, socket_path: Option<&Path>) -> Self {
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
        cmd.cwd(&fixture.workspace_root);
        cmd.env("HOME", &fixture.home);
        cmd.env("USERPROFILE", &fixture.home);
        cmd.env("SUBSTRATE_HOME", &fixture.substrate_home);
        cmd.env("SUBSTRATE_MANAGER_MANIFEST", manager_manifest_path());
        cmd.env("SHIM_TRACE_LOG", fixture.trace_path());
        cmd.env_remove("SHIM_ORIGINAL_PATH");
        cmd.env_remove("SUBSTRATE_WORLD");
        cmd.env_remove("SUBSTRATE_WORLD_ENABLED");
        cmd.env_remove("SUBSTRATE_WORLD_ID");
        match socket_path {
            Some(socket_path) => {
                cmd.env("SUBSTRATE_WORLD_SOCKET", socket_path);
                cmd.env("SUBSTRATE_OVERRIDE_WORLD", "enabled");
                cmd.arg("--world");
            }
            None => {
                cmd.env("SUBSTRATE_OVERRIDE_WORLD", "disabled");
            }
        }
        cmd.env("SHELL", "/bin/bash");
        cmd.arg("--async-repl");
        cmd.arg("--shim-skip");

        let child = pair.slave.spawn_command(cmd).expect("spawn substrate repl");
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
                        let err = std::io::Error::last_os_error();
                        if err.kind() == std::io::ErrorKind::WouldBlock {
                            std::thread::sleep(Duration::from_millis(25));
                            continue;
                        }
                        break;
                    }

                    let n = rc as usize;
                    let chunk = &buf[..n];
                    if chunk.windows(4).any(|window| window == b"\x1b[6n") {
                        if let Some(writer) = writer_for_thread.upgrade() {
                            if let Ok(mut handle) = writer.lock() {
                                let _ = handle.write_all(b"\x1b[1;1R");
                                let _ = handle.flush();
                            }
                        }
                    }
                    if chunk.windows(5).any(|window| window == b"\x1b[18t") {
                        if let Some(writer) = writer_for_thread.upgrade() {
                            if let Ok(mut handle) = writer.lock() {
                                let _ = handle.write_all(b"\x1b[8;24;80t");
                                let _ = handle.flush();
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
        writer.write_all(b"\n").expect("write newline");
        writer.flush().expect("flush line");
    }

    fn wait_for_output(&self, needle: &str, timeout: Duration) -> Option<usize> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            let output = self.output.lock().expect("output lock");
            if let Ok(text) = std::str::from_utf8(&output) {
                if let Some(pos) = text.find(needle) {
                    return Some(pos);
                }
            }
            drop(output);
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
            .map(|status| status.exit_code() as i32)
            .unwrap_or(-1);
        let output = self.output.lock().expect("output lock").clone();
        (code, output)
    }
}

#[test]
#[serial]
fn public_start_turn_and_stop_emit_streaming_ndjson_and_authoritative_state() {
    let fixture = AgentControlFixture::new();
    fixture.init_workspace();
    fixture.write_runtime_inventory(false);

    let start_output = fixture.run(&[
        "agent",
        "start",
        "--backend",
        "cli:codex",
        "--prompt",
        "hello from start",
        "--json",
    ]);
    assert!(
        start_output.status.success(),
        "public start should succeed: {start_output:?}"
    );
    let start_records = parse_ndjson_output(&start_output);
    let start_accepted = find_ndjson_record(&start_records, "accepted");
    assert_eq!(
        start_records
            .first()
            .and_then(|record| record.get("kind"))
            .and_then(Value::as_str),
        Some("accepted"),
        "public start must stream acceptance before terminal completion: {start_records:?}"
    );
    let start_json = find_ndjson_record(&start_records, "completed");
    assert_eq!(
        start_json.get("action").and_then(Value::as_str),
        Some("start")
    );
    assert_eq!(
        start_json.get("backend_id").and_then(Value::as_str),
        Some("cli:codex")
    );
    assert_eq!(
        start_accepted.get("scope").and_then(Value::as_str),
        Some("host")
    );
    assert_eq!(
        start_json.get("turn_outcome").and_then(Value::as_str),
        Some("success")
    );
    assert_eq!(
        start_json.get("session_posture").and_then(Value::as_str),
        Some("active")
    );
    assert_eq!(
        start_json.get("state").and_then(Value::as_str),
        Some("active")
    );
    assert!(
        start_json.get("source_orchestration_session_id").is_none(),
        "start must not surface source_orchestration_session_id: {start_json}"
    );
    assert_empty_warnings(start_json);

    let orchestration_session_id = start_json["orchestration_session_id"]
        .as_str()
        .expect("start session id")
        .to_string();
    let participant_id = start_json["participant_id"]
        .as_str()
        .expect("start participant id")
        .to_string();
    let persisted_session = fixture.load_orchestration_session(&orchestration_session_id);
    assert_eq!(
        persisted_session
            .get("active_session_handle_id")
            .and_then(Value::as_str),
        Some(participant_id.as_str()),
        "public start must not report readiness before the state store points at the active participant"
    );
    assert_eq!(
        persisted_session
            .pointer("/startup_prompt/state")
            .and_then(Value::as_str),
        Some("completed"),
        "public start must persist startup prompt completion for replay gating"
    );
    let owner_pid = persisted_session["shell_owner_pid"]
        .as_u64()
        .expect("shell owner pid") as u32;
    assert!(
        pid_is_alive(owner_pid),
        "public start must leave the hidden owner-helper process alive"
    );
    assert!(
        wait_for_path(
            &stop_transport_path(&fixture, &orchestration_session_id, &participant_id),
            Duration::from_secs(5),
        ),
        "public start must materialize a per-session private stop transport"
    );
    let persisted_participant =
        fixture.load_participant(&orchestration_session_id, &participant_id);
    assert_eq!(
        persisted_participant
            .pointer("/internal/uaa_session_id")
            .and_then(Value::as_str),
        Some("thread-test"),
        "public start must persist the retained internal.uaa_session_id from the single startup exec"
    );
    let start_args = fixture.read_fake_codex_args(1);
    assert!(
        start_args.iter().any(|arg| arg == "exec"),
        "startup invocation must use codex exec: {start_args:?}"
    );
    assert!(
        start_args.iter().any(|arg| arg == "--json"),
        "startup invocation must use codex exec --json: {start_args:?}"
    );
    assert!(
        !start_args.iter().any(|arg| arg == "resume"),
        "start prompt must not use resume on the initial retained exec: {start_args:?}"
    );
    let start_stdin = fixture.read_fake_codex_stdin(1);
    assert!(
        start_stdin.contains("hello from start"),
        "the first visible start prompt must ride the startup exec stdin payload: {start_stdin:?}"
    );
    assert!(
        !start_stdin.contains("Enter persistent Substrate host orchestrator mode."),
        "start prompt must not be wrapped in hidden host bootstrap instructions: {start_stdin:?}"
    );
    assert!(
        !start_stdin.contains("First visible operator request:"),
        "start prompt must not be rewritten into a bootstrap+visible composed payload: {start_stdin:?}"
    );
    assert!(
        !fixture.fake_codex_args_path(2).exists(),
        "agent start must cause exactly one Codex exec launch before any follow-up turn"
    );

    let turn_output = fixture.run(&[
        "agent",
        "turn",
        "--session",
        &orchestration_session_id,
        "--backend",
        "cli:codex",
        "--prompt",
        "hello from turn",
        "--json",
    ]);
    assert!(
        turn_output.status.success(),
        "public turn should succeed: {turn_output:?}"
    );
    let turn_records = parse_ndjson_output(&turn_output);
    let turn_accepted = find_ndjson_record(&turn_records, "accepted");
    assert_eq!(
        turn_records
            .first()
            .and_then(|record| record.get("kind"))
            .and_then(Value::as_str),
        Some("accepted"),
        "public turn must stream acceptance before terminal completion: {turn_records:?}"
    );
    let turn_json = find_ndjson_record(&turn_records, "completed");
    assert_eq!(
        turn_json.get("action").and_then(Value::as_str),
        Some("turn")
    );
    assert_eq!(
        turn_json
            .get("orchestration_session_id")
            .and_then(Value::as_str),
        Some(orchestration_session_id.as_str())
    );
    assert_eq!(
        turn_json.get("backend_id").and_then(Value::as_str),
        Some("cli:codex")
    );
    assert_eq!(
        turn_accepted.get("scope").and_then(Value::as_str),
        Some("host")
    );
    assert_eq!(
        turn_json.get("turn_outcome").and_then(Value::as_str),
        Some("success")
    );
    assert_eq!(
        turn_json.get("session_posture").and_then(Value::as_str),
        Some("active")
    );
    assert_eq!(
        turn_json.get("state").and_then(Value::as_str),
        Some("active")
    );
    assert_empty_warnings(turn_json);
    let turn_args = fixture.read_fake_codex_args(2);
    assert!(
        turn_args.iter().any(|arg| arg == "resume"),
        "the first follow-up turn must be the first resume-backed invocation: {turn_args:?}"
    );
    let turn_stdin = fixture.read_fake_codex_stdin(2);
    assert!(
        turn_stdin.contains("hello from turn"),
        "resume-backed follow-up turns must continue to send the prompt on stdin: {turn_stdin:?}"
    );

    let stop_output = fixture.run(&[
        "agent",
        "stop",
        "--session",
        &orchestration_session_id,
        "--json",
    ]);
    assert!(
        stop_output.status.success(),
        "public stop should succeed: {stop_output:?}"
    );
    let stop_json = parse_json_output(&stop_output);
    assert_eq!(
        stop_json.get("action").and_then(Value::as_str),
        Some("stop")
    );
    assert_eq!(
        stop_json
            .get("orchestration_session_id")
            .and_then(Value::as_str),
        Some(orchestration_session_id.as_str())
    );
    assert_eq!(
        stop_json.get("participant_id").and_then(Value::as_str),
        Some(participant_id.as_str())
    );
    assert_eq!(
        stop_json.get("backend_id").and_then(Value::as_str),
        Some("cli:codex")
    );
    assert!(
        matches!(
            stop_json.get("state").and_then(Value::as_str),
            Some("stopped") | Some("invalidated")
        ),
        "public stop must wait for a terminal parent state: {stop_json}"
    );
    assert_empty_warnings(&stop_json);

    let final_session = fixture.load_orchestration_session(&orchestration_session_id);
    assert_eq!(
        final_session.get("state").and_then(Value::as_str),
        Some("stopped"),
        "host-scoped public stop should persist a stopped parent session on clean shutdown"
    );
}

#[test]
#[serial]
fn public_reattach_and_fork_preserve_exact_session_and_lineage_contracts() {
    let fixture = AgentControlFixture::new();
    fixture.init_workspace();
    fixture.write_runtime_inventory(false);

    let ts = "2026-05-05T00:00:00Z";
    write_parked_orchestration_session(&fixture, "codex", "sess_resume_source", "ash_source", ts);
    write_runtime_participant(
        &fixture,
        "ash_source",
        "codex",
        "sess_resume_source",
        "running",
        false,
        Some("uaa-detached-1"),
        None,
        ts,
    );

    let reattach_output = fixture.run(&[
        "agent",
        "reattach",
        "--session",
        "sess_resume_source",
        "--json",
    ]);
    assert!(
        reattach_output.status.success(),
        "public reattach should succeed for an orphaned authoritative session: {reattach_output:?}"
    );
    let resume_json = parse_json_output(&reattach_output);
    assert_eq!(
        resume_json.get("action").and_then(Value::as_str),
        Some("reattach")
    );
    assert_eq!(
        resume_json
            .get("orchestration_session_id")
            .and_then(Value::as_str),
        Some("sess_resume_source")
    );
    assert_eq!(
        resume_json.get("backend_id").and_then(Value::as_str),
        Some("cli:codex")
    );
    assert_eq!(
        resume_json.get("scope").and_then(Value::as_str),
        Some("host")
    );
    assert_eq!(
        resume_json.get("state").and_then(Value::as_str),
        Some("active")
    );
    assert!(
        resume_json.get("source_orchestration_session_id").is_none(),
        "reattach must stay inside the source orchestration session: {resume_json}"
    );
    assert_empty_warnings(&resume_json);

    let resumed_participant_id = resume_json["participant_id"]
        .as_str()
        .expect("resume participant id")
        .to_string();
    assert_ne!(
        resumed_participant_id, "ash_source",
        "reattach must allocate a successor participant"
    );
    let resumed_participant =
        fixture.load_participant("sess_resume_source", &resumed_participant_id);
    assert_eq!(
        resumed_participant
            .get("resumed_from_participant_id")
            .and_then(Value::as_str),
        Some("ash_source"),
        "reattach successor persistence must retain resumed_from_participant_id lineage"
    );
    let resumed_owner_pid = fixture.load_orchestration_session("sess_resume_source")
        ["shell_owner_pid"]
        .as_u64()
        .expect("resume owner pid") as u32;
    assert!(
        pid_is_alive(resumed_owner_pid),
        "reattach must leave a live owner loop"
    );

    fixture.reset_fake_codex_state();
    let fork_output = fixture.run(&["agent", "fork", "--session", "sess_resume_source", "--json"]);
    assert!(
        fork_output.status.success(),
        "public fork should succeed from the active resumed session: {fork_output:?}"
    );
    let fork_json = parse_json_output(&fork_output);
    assert_eq!(
        fork_json.get("action").and_then(Value::as_str),
        Some("fork")
    );
    assert_eq!(
        fork_json
            .get("source_orchestration_session_id")
            .and_then(Value::as_str),
        Some("sess_resume_source")
    );
    assert_eq!(
        fork_json.get("backend_id").and_then(Value::as_str),
        Some("cli:codex")
    );
    assert_eq!(fork_json.get("scope").and_then(Value::as_str), Some("host"));
    assert_eq!(
        fork_json.get("state").and_then(Value::as_str),
        Some("active")
    );
    assert_empty_warnings(&fork_json);

    let fork_session_id = fork_json["orchestration_session_id"]
        .as_str()
        .expect("fork session id")
        .to_string();
    assert_ne!(
        fork_session_id, "sess_resume_source",
        "fork must allocate a new orchestration session id"
    );
    let fork_participant_id = fork_json["participant_id"]
        .as_str()
        .expect("fork participant id")
        .to_string();
    let fork_participant = fixture.load_participant(&fork_session_id, &fork_participant_id);
    assert_eq!(
        fork_participant
            .get("resumed_from_participant_id")
            .and_then(Value::as_str),
        Some(resumed_participant_id.as_str()),
        "fork successor persistence must point lineage at the exact live source participant"
    );
    let fork_owner_pid = fixture.load_orchestration_session(&fork_session_id)["shell_owner_pid"]
        .as_u64()
        .expect("fork owner pid") as u32;
    assert!(
        pid_is_alive(fork_owner_pid),
        "fork must leave a live owner loop"
    );

    terminate_pid(resumed_owner_pid);
    terminate_pid(fork_owner_pid);
}

#[test]
#[serial]
fn public_stop_cleanly_closes_same_durable_session_after_reattach() {
    let fixture = AgentControlFixture::new();
    fixture.init_workspace();
    fixture.write_runtime_inventory(false);

    let ts = "2026-05-05T00:00:00Z";
    write_parked_orchestration_session(&fixture, "codex", "sess_resume_stop", "ash_source", ts);
    write_runtime_participant(
        &fixture,
        "ash_source",
        "codex",
        "sess_resume_stop",
        "running",
        false,
        Some("uaa-detached-stop-1"),
        None,
        ts,
    );

    let reattach_output = fixture.run(&[
        "agent",
        "reattach",
        "--session",
        "sess_resume_stop",
        "--json",
    ]);
    assert!(
        reattach_output.status.success(),
        "public reattach should succeed for a parked durable session: {reattach_output:?}"
    );
    let reattach_json = parse_json_output(&reattach_output);
    let resumed_participant_id = reattach_json["participant_id"]
        .as_str()
        .expect("reattach participant id")
        .to_string();
    let resumed_owner_pid = wait_for_successor_owner_pid(
        &fixture,
        "sess_resume_stop",
        "ash_source",
        Duration::from_secs(5),
    );
    let reattached_session = wait_for_session_posture(
        &fixture,
        "sess_resume_stop",
        "active_attached",
        Duration::from_secs(5),
    );
    assert_eq!(
        reattached_session
            .get("attached_participant_id")
            .and_then(Value::as_str),
        Some(resumed_participant_id.as_str())
    );
    assert!(pid_is_alive(resumed_owner_pid));
    assert!(
        wait_for_path(
            &stop_transport_path(&fixture, "sess_resume_stop", &resumed_participant_id),
            Duration::from_secs(5),
        ),
        "reattach must materialize the per-session private stop transport before stop is exercised"
    );

    let stop_output = fixture.run(&["agent", "stop", "--session", "sess_resume_stop", "--json"]);
    assert!(
        stop_output.status.success(),
        "public stop should close the same durable session after reattach: {stop_output:?}"
    );
    let stop_json = parse_json_output(&stop_output);
    assert_eq!(
        stop_json.get("action").and_then(Value::as_str),
        Some("stop")
    );
    assert_eq!(
        stop_json
            .get("orchestration_session_id")
            .and_then(Value::as_str),
        Some("sess_resume_stop")
    );
    assert_eq!(
        stop_json.get("state").and_then(Value::as_str),
        Some("stopped")
    );

    let stopped_session = fixture.load_orchestration_session("sess_resume_stop");
    assert_eq!(
        stopped_session.get("state").and_then(Value::as_str),
        Some("stopped")
    );
    assert_eq!(
        stopped_session.get("posture").and_then(Value::as_str),
        Some("terminal")
    );
    assert!(stopped_session
        .get("attached_participant_id")
        .is_none_or(Value::is_null));
    assert!(stopped_session
        .get("closed_at")
        .and_then(Value::as_str)
        .is_some());
    assert!(
        wait_for_pid_exit(resumed_owner_pid, Duration::from_secs(5)),
        "reattach stop should leave no live hidden owner-helper"
    );

    let repeated_stop_output =
        fixture.run(&["agent", "stop", "--session", "sess_resume_stop", "--json"]);
    assert_eq!(
        repeated_stop_output.status.code(),
        Some(2),
        "repeated stop must no longer see a routable active parent after terminal convergence: {repeated_stop_output:?}"
    );
    assert!(
        stderr_text(&repeated_stop_output).contains("missing_active_parent"),
        "repeated stop must fail closed as non-active after terminal convergence: {repeated_stop_output:?}"
    );
}

#[test]
#[serial]
fn public_turn_resumes_parked_host_session_and_preserves_exact_session_selector_contracts() {
    let fixture = AgentControlFixture::new();
    fixture.init_workspace();
    fixture.write_runtime_inventory(false);

    let ts = "2026-05-05T00:00:00Z";
    write_parked_orchestration_session(&fixture, "codex", "sess_turn_parked", "ash_parked", ts);
    write_runtime_participant(
        &fixture,
        "ash_parked",
        "codex",
        "sess_turn_parked",
        "running",
        false,
        Some("uaa-detached-turn"),
        None,
        ts,
    );

    let turn_output = fixture.run(&[
        "agent",
        "turn",
        "--session",
        "sess_turn_parked",
        "--backend",
        "cli:codex",
        "--prompt",
        "resume parked host turn",
        "--json",
    ]);
    assert!(
        turn_output.status.success(),
        "public turn should resume a parked authoritative session: {turn_output:?}"
    );
    let turn_records = parse_ndjson_output(&turn_output);
    let turn_accepted = find_ndjson_record(&turn_records, "accepted");
    let turn_json = find_ndjson_record(&turn_records, "completed");
    assert_eq!(
        turn_records
            .first()
            .and_then(|record| record.get("kind"))
            .and_then(Value::as_str),
        Some("accepted"),
        "parked host turn must still emit acceptance before terminal completion: {turn_records:?}"
    );
    assert_eq!(
        turn_accepted.get("scope").and_then(Value::as_str),
        Some("host")
    );
    assert_eq!(
        turn_json.get("action").and_then(Value::as_str),
        Some("turn")
    );
    assert_eq!(
        turn_json
            .get("orchestration_session_id")
            .and_then(Value::as_str),
        Some("sess_turn_parked")
    );
    assert_eq!(
        turn_json.get("backend_id").and_then(Value::as_str),
        Some("cli:codex")
    );
    assert_eq!(
        turn_json.get("session_posture").and_then(Value::as_str),
        Some("active")
    );
    assert_eq!(
        turn_json.get("state").and_then(Value::as_str),
        Some("active")
    );
    assert!(
        turn_json.get("source_orchestration_session_id").is_none(),
        "parked host turn must stay inside the selected orchestration session: {turn_json}"
    );
    assert_empty_warnings(turn_json);

    let resumed_participant_id = turn_json["participant_id"]
        .as_str()
        .expect("resumed participant id")
        .to_string();
    let resumed_participant = fixture.load_participant("sess_turn_parked", &resumed_participant_id);
    assert_eq!(
        resumed_participant
            .get("resumed_from_participant_id")
            .and_then(Value::as_str),
        Some("ash_parked")
    );
}

#[test]
#[serial]
fn public_same_session_parked_status_turn_reattach_and_stop_stay_on_one_orchestration_session_id() {
    let fixture = AgentControlFixture::new_with_fake_codex(
        write_fake_codex_script_turn_requires_helper_cancel_after_prompt,
    );
    fixture.init_workspace();
    fixture.write_runtime_inventory(false);

    let ts = "2026-05-05T00:00:00Z";
    let orchestration_session_id = "sess_turn_reattach_stop";
    write_parked_orchestration_session(
        &fixture,
        "codex",
        orchestration_session_id,
        "ash_turn_source",
        ts,
    );
    write_runtime_participant(
        &fixture,
        "ash_turn_source",
        "codex",
        orchestration_session_id,
        "running",
        false,
        Some("uaa-turn-reattach-1"),
        None,
        ts,
    );

    let parked_status_output = fixture.run(&["agent", "status", "--json"]);
    assert!(
        parked_status_output.status.success(),
        "parked status must succeed before same-session follow-up control flow: {parked_status_output:?}"
    );
    let parked_status_json = parse_json_output(&parked_status_output);
    let parked_status_row = find_status_session_by_orchestration_session_id(
        status_sessions(&parked_status_json),
        orchestration_session_id,
    );
    assert_eq!(
        parked_status_row.get("source_kind").and_then(Value::as_str),
        Some("live_runtime")
    );
    assert_eq!(
        parked_status_row.get("posture").and_then(Value::as_str),
        Some("parked_resumable")
    );
    assert!(
        parked_status_row
            .get("attached_participant_id")
            .is_some_and(Value::is_null),
        "parked status rows must preserve detached ownership as explicit null: {parked_status_row}"
    );
    assert_eq!(
        parked_status_row
            .get("pending_inbox_count")
            .and_then(Value::as_u64),
        Some(0)
    );

    let turn_output = fixture.run(&[
        "agent",
        "turn",
        "--session",
        orchestration_session_id,
        "--backend",
        "cli:codex",
        "--prompt",
        "resume parked host turn and exit",
        "--json",
    ]);
    assert!(
        turn_output.status.success(),
        "parked turn should succeed before reattach on the same durable session: {turn_output:?}"
    );
    let turn_records = parse_ndjson_output(&turn_output);
    let turn_accepted = find_ndjson_record(&turn_records, "accepted");
    let turn_json = find_ndjson_record(&turn_records, "completed");
    assert_eq!(
        turn_records
            .first()
            .and_then(|record| record.get("kind"))
            .and_then(Value::as_str),
        Some("accepted"),
        "parked same-session turn must stream acceptance before completion: {turn_records:?}"
    );
    assert_eq!(
        turn_records
            .iter()
            .filter(|record| {
                matches!(
                    record.get("kind").and_then(Value::as_str),
                    Some("completed" | "failed")
                )
            })
            .count(),
        1,
        "same-session public turn must emit exactly one terminal envelope after accepted: {turn_records:?}"
    );
    assert_eq!(
        turn_accepted.get("scope").and_then(Value::as_str),
        Some("host")
    );
    assert_eq!(
        turn_json.get("action").and_then(Value::as_str),
        Some("turn")
    );
    assert_eq!(
        turn_json
            .get("orchestration_session_id")
            .and_then(Value::as_str),
        Some(orchestration_session_id)
    );
    assert_eq!(
        turn_json.get("backend_id").and_then(Value::as_str),
        Some("cli:codex")
    );
    assert_eq!(
        turn_json.get("session_posture").and_then(Value::as_str),
        Some("active")
    );
    assert_eq!(
        turn_json.get("state").and_then(Value::as_str),
        Some("active")
    );
    assert!(
        turn_json.get("source_orchestration_session_id").is_none(),
        "turn must remain on the selected orchestration session id: {turn_json}"
    );
    assert_empty_warnings(turn_json);

    let turned_participant_id = turn_json["participant_id"]
        .as_str()
        .expect("turned participant id")
        .to_string();
    let turned_participant =
        fixture.load_participant(orchestration_session_id, &turned_participant_id);
    assert_eq!(
        turned_participant
            .get("resumed_from_participant_id")
            .and_then(Value::as_str),
        Some("ash_turn_source")
    );

    let reparked_session = wait_for_session_posture(
        &fixture,
        orchestration_session_id,
        "parked_resumable",
        Duration::from_secs(5),
    );
    assert_eq!(
        reparked_session
            .get("active_session_handle_id")
            .and_then(Value::as_str),
        Some(turned_participant_id.as_str())
    );
    assert!(reparked_session
        .get("attached_participant_id")
        .is_none_or(Value::is_null));
    assert_eq!(
        reparked_session
            .get("pending_inbox_count")
            .and_then(Value::as_u64),
        Some(0)
    );
    let reparked_status_output = fixture.run(&["agent", "status", "--json"]);
    assert!(
        reparked_status_output.status.success(),
        "status should reflect detached durable truth immediately after same-session turn repark: {reparked_status_output:?}"
    );
    let reparked_status_json = parse_json_output(&reparked_status_output);
    let reparked_status_row = find_status_session_by_orchestration_session_id(
        status_sessions(&reparked_status_json),
        orchestration_session_id,
    );
    assert_eq!(
        reparked_status_row.get("posture").and_then(Value::as_str),
        Some("parked_resumable")
    );
    assert!(
        reparked_status_row
            .get("attached_participant_id")
            .is_some_and(Value::is_null),
        "status must report the detached parked participant as explicit null after turn: {reparked_status_row}"
    );
    assert_eq!(
        reparked_status_row
            .get("pending_inbox_count")
            .and_then(Value::as_u64),
        Some(0)
    );

    let reattach_output = fixture.run(&[
        "agent",
        "reattach",
        "--session",
        orchestration_session_id,
        "--json",
    ]);
    assert!(
        reattach_output.status.success(),
        "reattach should recover the same durable session after the follow-up turn exits: {reattach_output:?}"
    );
    let reattach_json = parse_json_output(&reattach_output);
    let reattached_participant_id = reattach_json["participant_id"]
        .as_str()
        .expect("reattach participant id")
        .to_string();
    let resumed_owner_pid = wait_for_successor_owner_pid(
        &fixture,
        orchestration_session_id,
        &turned_participant_id,
        Duration::from_secs(5),
    );
    let reattached_session = wait_for_session_posture(
        &fixture,
        orchestration_session_id,
        "active_attached",
        Duration::from_secs(5),
    );
    assert_eq!(
        reattached_session
            .get("attached_participant_id")
            .and_then(Value::as_str),
        Some(reattached_participant_id.as_str())
    );
    assert!(pid_is_alive(resumed_owner_pid));
    assert!(
        wait_for_path(
            &stop_transport_path(
                &fixture,
                orchestration_session_id,
                &reattached_participant_id,
            ),
            Duration::from_secs(5),
        ),
        "reattach must restore the private stop transport before the same-session stop"
    );

    let stop_output = fixture.run(&[
        "agent",
        "stop",
        "--session",
        orchestration_session_id,
        "--json",
    ]);
    assert!(
        stop_output.status.success(),
        "stop should close the same orchestration session after parked turn plus reattach: {stop_output:?}"
    );
    let stop_json = parse_json_output(&stop_output);
    assert_eq!(
        stop_json.get("action").and_then(Value::as_str),
        Some("stop")
    );
    assert_eq!(
        stop_json
            .get("orchestration_session_id")
            .and_then(Value::as_str),
        Some(orchestration_session_id)
    );
    assert_eq!(
        stop_json.get("state").and_then(Value::as_str),
        Some("stopped")
    );

    let stopped_session = fixture.load_orchestration_session(orchestration_session_id);
    assert_eq!(
        stopped_session.get("state").and_then(Value::as_str),
        Some("stopped")
    );
    assert_eq!(
        stopped_session.get("posture").and_then(Value::as_str),
        Some("terminal")
    );
    assert!(
        wait_for_pid_exit(resumed_owner_pid, Duration::from_secs(5)),
        "same-session stop should leave no live reattached owner helper"
    );
}

#[test]
#[serial]
fn public_start_split_bootstrap_retry_timeout_emits_single_terminal_failure() {
    let fixture = AgentControlFixture::new_with_fake_codex(
        write_fake_codex_script_clean_bootstrap_then_prompt_resume,
    );
    fixture.init_workspace();
    fixture.write_runtime_inventory(false);

    let start_output = fixture.run(&[
        "agent",
        "start",
        "--backend",
        "cli:codex",
        "--prompt",
        "hello from clean bootstrap",
        "--json",
    ]);
    assert_eq!(
        start_output.status.code(),
        Some(1),
        "split bootstrap start must surface the accepted-then-terminal failure contract when readiness reconciliation times out: {start_output:?}"
    );
    let records = parse_ndjson_output(&start_output);
    assert_eq!(
        records
            .first()
            .and_then(|record| record.get("kind"))
            .and_then(Value::as_str),
        Some("accepted"),
        "split bootstrap retry timeout must preserve the accepted envelope before terminal failure: {records:?}"
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| {
                matches!(
                    record.get("kind").and_then(Value::as_str),
                    Some("completed" | "failed")
                )
            })
            .count(),
        1,
        "split bootstrap retry timeout must emit exactly one terminal envelope after accepted: {records:?}"
    );
    let failed = find_ndjson_record(&records, "failed");
    assert_eq!(failed.get("terminal").and_then(Value::as_bool), Some(true));
    assert_eq!(failed.get("stage").and_then(Value::as_str), Some("runtime"));
    assert!(
        failed
            .get("error_code")
            .and_then(Value::as_str)
            .is_some_and(|code| code == "owner_unreachable"),
        "split bootstrap retry timeout must keep the startup-stream owner_unreachable classifier: {failed}"
    );
    assert!(
        failed
            .get("message")
            .and_then(Value::as_str)
            .is_some_and(|message| {
                message.contains("startup prompt stream ended before terminal completion")
            }),
        "split bootstrap retry timeout must explain the terminalized startup prompt stream failure: {failed}"
    );
    assert!(
        records
            .iter()
            .all(|record| record.get("kind").and_then(Value::as_str) != Some("completed")),
        "split bootstrap retry timeout must not also emit completed: {records:?}"
    );
    let orchestration_session_id = records
        .iter()
        .find_map(|record| {
            record
                .get("orchestration_session_id")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .expect("split bootstrap accepted session id");
    let reconciled_session = wait_for_session_posture(
        &fixture,
        &orchestration_session_id,
        "parked_resumable",
        Duration::from_secs(5),
    );
    assert_eq!(
        reconciled_session.get("state").and_then(Value::as_str),
        Some("active"),
        "readiness timeout must reconcile the failed start onto detached live session truth"
    );
    assert_ne!(
        reconciled_session.get("posture").and_then(Value::as_str),
        Some("active_attached"),
        "failed start must not leave behind stale active_attached posture"
    );
    assert_eq!(
        reconciled_session
            .pointer("/startup_prompt/state")
            .and_then(Value::as_str),
        Some("failed"),
        "readiness-timeout fallback failure must terminalize startup prompt tracking"
    );
    assert_eq!(
        reconciled_session
            .pointer("/startup_prompt/participant_id")
            .and_then(Value::as_str),
        reconciled_session
            .get("active_session_handle_id")
            .and_then(Value::as_str),
        "terminalized startup prompt must remain pinned to the failed authoritative participant"
    );
    assert!(
        reconciled_session
            .get("attached_participant_id")
            .is_none_or(Value::is_null),
        "readiness-timeout reconciliation must clear attached ownership once the helper detaches: {reconciled_session}"
    );
    let stderr = stderr_text(&start_output);
    assert!(
        stderr.contains("startup prompt stream ended before terminal completion"),
        "split bootstrap retry timeout stderr should match the terminal failed envelope message: {start_output:?}"
    );
}

#[test]
#[serial]
fn public_start_persists_detached_session_when_hidden_owner_helper_exits() {
    let fixture =
        AgentControlFixture::new_with_fake_codex(write_fake_codex_script_exit_after_startup_prompt);
    fixture.init_workspace();
    fixture.write_runtime_inventory(false);

    let start_output = fixture.run(&[
        "agent",
        "start",
        "--backend",
        "cli:codex",
        "--prompt",
        "hello from parked start",
        "--json",
    ]);
    assert!(
        start_output.status.success(),
        "public start should succeed even if the helper exits immediately after the startup prompt: {start_output:?}"
    );
    let start_records = parse_ndjson_output(&start_output);
    let start_json = find_ndjson_record(&start_records, "completed");
    assert_empty_warnings(start_json);

    let orchestration_session_id = start_json["orchestration_session_id"]
        .as_str()
        .expect("start session id")
        .to_string();
    let participant_id = start_json["participant_id"]
        .as_str()
        .expect("start participant id")
        .to_string();

    let persisted_session = fixture.load_orchestration_session(&orchestration_session_id);
    assert_eq!(
        persisted_session.get("state").and_then(Value::as_str),
        Some("active")
    );
    assert_eq!(
        persisted_session.get("posture").and_then(Value::as_str),
        Some("parked_resumable"),
        "stored session must already be detached-normalized before any later reattach"
    );
    assert_eq!(
        persisted_session
            .get("active_session_handle_id")
            .and_then(Value::as_str),
        Some(participant_id.as_str())
    );
    assert!(persisted_session
        .get("attached_participant_id")
        .is_none_or(Value::is_null));
    assert_eq!(
        persisted_session
            .get("pending_inbox_count")
            .and_then(Value::as_u64),
        Some(0)
    );
    assert_eq!(
        persisted_session
            .get("shell_owner_pid")
            .and_then(Value::as_u64),
        Some(0),
        "detached normalization must clear the dead helper pid from persisted session truth"
    );
    assert!(persisted_session
        .get("closed_at")
        .is_none_or(Value::is_null));

    let persisted_participant =
        fixture.load_participant(&orchestration_session_id, &participant_id);
    assert_eq!(
        persisted_participant
            .pointer("/internal/uaa_session_id")
            .and_then(Value::as_str),
        Some("thread-test"),
        "detached normalization must preserve the durable internal.uaa_session_id"
    );
    assert_eq!(
        persisted_participant
            .pointer("/internal/shell_owner_pid")
            .and_then(Value::as_u64),
        Some(0),
        "detached normalization must clear the dead helper pid from participant truth"
    );
    assert_eq!(
        persisted_participant
            .pointer("/internal/attached_client_present")
            .and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        persisted_participant
            .pointer("/internal/resume_eligible")
            .and_then(Value::as_bool),
        Some(true)
    );
}

#[test]
#[serial]
fn detached_pending_inbox_normalizes_to_awaiting_attention() {
    let fixture = AgentControlFixture::new();
    fixture.init_workspace();
    fixture.write_runtime_inventory(false);

    let ts = "2026-05-05T00:00:00Z";
    let orchestration_session_id = "sess_attention".to_string();
    let original_participant_id = "ash_attention".to_string();
    write_parked_orchestration_session(
        &fixture,
        "codex",
        &orchestration_session_id,
        &original_participant_id,
        ts,
    );
    write_runtime_participant(
        &fixture,
        &original_participant_id,
        "codex",
        &orchestration_session_id,
        "running",
        false,
        Some("uaa-attention-1"),
        None,
        ts,
    );

    let parked_session = wait_for_session_posture(
        &fixture,
        &orchestration_session_id,
        "parked_resumable",
        Duration::from_secs(5),
    );
    let parked_handle_id = parked_session
        .get("active_session_handle_id")
        .and_then(Value::as_str)
        .expect("parked authoritative handle id")
        .to_string();
    assert_eq!(parked_handle_id, original_participant_id);
    assert!(parked_session
        .get("attached_participant_id")
        .is_none_or(Value::is_null));

    persist_runtime_alert_for_substrate_home(
        &fixture.substrate_home,
        &orchestration_session_id,
        "item_attention",
        Some("attention needed for item_attention".to_string()),
    );
    let attention_session = wait_for_session_posture(
        &fixture,
        &orchestration_session_id,
        "awaiting_attention",
        Duration::from_secs(5),
    );
    assert_eq!(
        attention_session
            .get("orchestration_session_id")
            .and_then(Value::as_str),
        Some(orchestration_session_id.as_str())
    );
    assert_eq!(
        attention_session.get("state").and_then(Value::as_str),
        Some("active")
    );
    assert_eq!(
        attention_session.get("posture").and_then(Value::as_str),
        Some("awaiting_attention")
    );
    assert!(attention_session
        .get("attached_participant_id")
        .is_none_or(Value::is_null));
    assert_eq!(
        attention_session
            .get("active_session_handle_id")
            .and_then(Value::as_str),
        Some(parked_handle_id.as_str())
    );
    assert_eq!(
        attention_session
            .get("pending_inbox_count")
            .and_then(Value::as_u64),
        Some(1)
    );
    let detached_participant = wait_for_detached_resume_eligible_participant(
        &fixture,
        &orchestration_session_id,
        &original_participant_id,
        Duration::from_secs(5),
    );
    assert_eq!(
        detached_participant
            .pointer("/internal/resume_eligible")
            .and_then(Value::as_bool),
        Some(true),
        "detached awaiting-attention normalization must keep the authoritative host participant resumable"
    );
    assert_eq!(
        detached_participant
            .pointer("/internal/attached_client_present")
            .and_then(Value::as_bool),
        Some(false),
        "detached awaiting-attention normalization must keep the authoritative host participant detached"
    );

    let status_output = fixture.run(&["agent", "status", "--json"]);
    assert!(
        status_output.status.success(),
        "status must keep awaiting-attention durable rows visible after detached normalization: {status_output:?}"
    );
    let status_json = parse_json_output(&status_output);
    let attention_status_row = find_status_session_by_orchestration_session_id(
        status_sessions(&status_json),
        &orchestration_session_id,
    );
    assert_eq!(
        attention_status_row
            .get("source_kind")
            .and_then(Value::as_str),
        Some("live_runtime")
    );
    assert_eq!(
        attention_status_row.get("posture").and_then(Value::as_str),
        Some("awaiting_attention")
    );
    assert!(
        attention_status_row
            .get("attached_participant_id")
            .is_some_and(Value::is_null),
        "awaiting-attention rows must preserve detached ownership as explicit null: {attention_status_row}"
    );
    assert_eq!(
        attention_status_row
            .get("pending_inbox_count")
            .and_then(Value::as_u64),
        Some(1)
    );
}

#[test]
#[serial]
fn public_start_reports_runtime_start_failed_for_missing_bootstrap_handle() {
    let fixture =
        AgentControlFixture::new_with_fake_codex(write_fake_codex_script_without_session_handle);
    fixture.init_workspace();
    fixture.write_runtime_inventory(false);

    let start_output = fixture.run(&[
        "agent",
        "start",
        "--backend",
        "cli:codex",
        "--prompt",
        "hello from broken bootstrap",
        "--json",
    ]);
    assert_eq!(
        start_output.status.code(),
        Some(2),
        "broken bootstrap must fail closed instead of reporting a successful parked session: {start_output:?}"
    );
    assert!(
        start_output.stdout.is_empty(),
        "broken bootstrap must not emit public prompt envelopes when startup ownership never becomes authoritative: {start_output:?}"
    );
    let stderr = stderr_text(&start_output);
    assert!(
        stderr.contains("runtime_start_failed"),
        "broken bootstrap must keep the runtime_start_failed taxonomy: {start_output:?}"
    );
    assert!(
        stderr.contains("failed to establish attached control ownership")
            || stderr.contains("timed out waiting for authoritative owner-helper readiness")
            || stderr.contains("missing an active participant")
            || stderr.contains("owner_unreachable"),
        "broken bootstrap failure should explain the missing ownership/bootstrap continuity: {start_output:?}"
    );
}

#[test]
#[serial]
fn public_start_survives_slow_startup_prompt_completion_after_bootstrap_readiness() {
    let fixture = AgentControlFixture::new_with_fake_codex(
        write_fake_codex_script_with_slow_startup_prompt_completion,
    );
    fixture.init_workspace();
    fixture.write_runtime_inventory(false);

    let start_output = fixture.run(&[
        "agent",
        "start",
        "--backend",
        "cli:codex",
        "--prompt",
        "hello from slow startup prompt",
        "--json",
    ]);
    assert!(
        start_output.status.success(),
        "slow startup prompt completion must not trip the helper readiness timeout once bootstrap continuity is established: {start_output:?}"
    );
    let records = parse_ndjson_output(&start_output);
    assert_eq!(
        records
            .iter()
            .filter(|record| record.get("kind").and_then(Value::as_str) == Some("accepted"))
            .count(),
        1,
        "slow startup prompt completion must still emit exactly one accepted envelope: {records:?}"
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| {
                matches!(
                    record.get("kind").and_then(Value::as_str),
                    Some("completed" | "failed")
                )
            })
            .count(),
        1,
        "slow startup prompt completion must still emit exactly one terminal envelope: {records:?}"
    );
    let completed = find_ndjson_record(&records, "completed");
    let orchestration_session_id = completed["orchestration_session_id"]
        .as_str()
        .expect("slow start session id");
    let persisted_session = fixture.load_orchestration_session(orchestration_session_id);
    assert_eq!(
        persisted_session
            .pointer("/startup_prompt/state")
            .and_then(Value::as_str),
        Some("completed"),
        "slow startup prompt completion must persist completed startup prompt state"
    );
    assert_eq!(
        persisted_session.get("state").and_then(Value::as_str),
        Some("active"),
        "slow startup prompt completion must still produce a live durable session"
    );
    assert_eq!(
        persisted_session.get("posture").and_then(Value::as_str),
        Some("parked_resumable"),
        "slow startup prompt completion must still return a parked host session"
    );
}

#[test]
#[serial]
fn public_turn_emits_explicit_failed_after_accepted_owner_drop() {
    let fixture = AgentControlFixture::new_with_fake_codex(
        write_fake_codex_script_with_delayed_prompt_completion,
    );
    fixture.init_workspace();
    fixture.write_runtime_inventory(false);

    let ts = "2026-05-05T00:00:00Z";
    write_parked_orchestration_session(
        &fixture,
        "codex",
        "sess_late_drop",
        "ash_late_drop_source",
        ts,
    );
    write_runtime_participant(
        &fixture,
        "ash_late_drop_source",
        "codex",
        "sess_late_drop",
        "running",
        false,
        Some("uaa-late-drop-1"),
        None,
        ts,
    );

    let mut child = fixture.spawn(&[
        "agent",
        "turn",
        "--session",
        "sess_late_drop",
        "--backend",
        "cli:codex",
        "--prompt",
        "force late owner loss",
        "--json",
    ]);
    let stdout = child.stdout.take().expect("stdout pipe");
    let mut reader = std::io::BufReader::new(stdout);
    let mut line = String::new();
    let mut records = Vec::new();
    let mut killed_owner = false;
    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).expect("read stdout line");
        if bytes_read == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let record: Value = serde_json::from_str(trimmed).expect("stdout should be valid NDJSON");
        if !killed_owner && record.get("kind").and_then(Value::as_str) == Some("accepted") {
            let owner_pid = wait_for_successor_owner_pid(
                &fixture,
                "sess_late_drop",
                "ash_late_drop_source",
                Duration::from_secs(5),
            );
            terminate_pid(owner_pid);
            killed_owner = true;
        }
        records.push(record);
    }

    let status = child.wait().expect("wait for child");
    let stderr = {
        let mut stderr = String::new();
        let mut handle = child.stderr.take().expect("stderr pipe");
        std::io::Read::read_to_string(&mut handle, &mut stderr).expect("read stderr");
        stderr
    };

    assert_eq!(
        status.code(),
        Some(1),
        "owner loss after accepted must fail the public turn command: {records:?}\nstderr={stderr}"
    );
    assert_eq!(
        records
            .first()
            .and_then(|record| record.get("kind"))
            .and_then(Value::as_str),
        Some("accepted"),
        "the late-drop prompt path must emit accepted before the owner is terminated: {records:?}"
    );
    let failed = find_ndjson_record(&records, "failed");
    assert_eq!(failed.get("terminal").and_then(Value::as_bool), Some(true));
    assert_eq!(
        failed.get("error_code").and_then(Value::as_str),
        Some("owner_unreachable")
    );
    assert!(
        records
            .iter()
            .all(|record| record.get("kind").and_then(Value::as_str) != Some("completed")),
        "owner drop after accepted must not silently produce completed: {records:?}"
    );
}

#[test]
#[serial]
fn public_control_rejects_non_orchestration_session_selectors() {
    let fixture = AgentControlFixture::new();
    fixture.init_workspace();
    fixture.write_runtime_inventory(false);

    let ts = "2026-05-05T00:00:00Z";
    write_parked_orchestration_session(&fixture, "codex", "sess_selector", "ash_live", ts);
    write_runtime_participant(
        &fixture,
        "ash_live",
        "codex",
        "sess_selector",
        "running",
        false,
        Some("uaa-live-1"),
        None,
        ts,
    );
    write_runtime_participant(
        &fixture,
        "ash_previous",
        "codex",
        "sess_selector",
        "invalidated",
        false,
        Some("uaa-old-1"),
        Some("ash_older"),
        ts,
    );

    let active_handle_output =
        fixture.run(&["agent", "reattach", "--session", "ash_live", "--json"]);
    assert_eq!(
        active_handle_output.status.code(),
        Some(2),
        "active participant selectors must fail closed: {active_handle_output:?}"
    );
    assert!(
        stderr_text(&active_handle_output).contains("matched active_session_handle_id"),
        "public control must explain active_session_handle_id rejection: {active_handle_output:?}"
    );

    let participant_output =
        fixture.run(&["agent", "reattach", "--session", "ash_previous", "--json"]);
    assert_eq!(
        participant_output.status.code(),
        Some(2),
        "non-canonical participant selectors must fail closed: {participant_output:?}"
    );
    assert!(
        stderr_text(&participant_output).contains("matched participant_id/session_handle_id"),
        "public control must explain participant/session-handle rejection: {participant_output:?}"
    );

    let internal_output = fixture.run(&["agent", "reattach", "--session", "uaa-live-1", "--json"]);
    assert_eq!(
        internal_output.status.code(),
        Some(2),
        "internal uaa session ids must fail closed as public selectors: {internal_output:?}"
    );
    assert!(
        stderr_text(&internal_output).contains("matched internal.uaa_session_id"),
        "public control must explain internal.uaa_session_id rejection: {internal_output:?}"
    );

    let turn_selector_output = fixture.run(&[
        "agent",
        "turn",
        "--session",
        "ash_live",
        "--backend",
        "cli:codex",
        "--prompt",
        "next",
        "--json",
    ]);
    assert_eq!(
        turn_selector_output.status.code(),
        Some(2),
        "public turn must reject non-canonical active handle selectors: {turn_selector_output:?}"
    );
    assert!(
        stderr_text(&turn_selector_output).contains("matched active_session_handle_id"),
        "public turn must explain active_session_handle_id rejection: {turn_selector_output:?}"
    );

    let turn_participant_output = fixture.run(&[
        "agent",
        "turn",
        "--session",
        "ash_previous",
        "--backend",
        "cli:codex",
        "--prompt",
        "next",
        "--json",
    ]);
    assert_eq!(
        turn_participant_output.status.code(),
        Some(2),
        "public turn must reject participant/session-handle selectors: {turn_participant_output:?}"
    );
    assert!(
        stderr_text(&turn_participant_output).contains("matched participant_id/session_handle_id"),
        "public turn must explain participant/session-handle rejection: {turn_participant_output:?}"
    );

    let turn_internal_output = fixture.run(&[
        "agent",
        "turn",
        "--session",
        "uaa-live-1",
        "--backend",
        "cli:codex",
        "--prompt",
        "next",
        "--json",
    ]);
    assert_eq!(
        turn_internal_output.status.code(),
        Some(2),
        "public turn must reject internal uaa session selectors: {turn_internal_output:?}"
    );
    assert!(
        stderr_text(&turn_internal_output).contains("matched internal.uaa_session_id"),
        "public turn must explain internal uaa selector rejection: {turn_internal_output:?}"
    );
}

#[test]
#[serial]
fn public_turn_fail_closed_taxonomy_is_explicit_for_missing_backend_unknown_session_and_parent_slot_errors(
) {
    let fixture = AgentControlFixture::new();
    fixture.init_workspace();
    fixture.write_runtime_inventory(true);

    let ts = "2026-05-05T00:00:00Z";
    write_active_orchestration_session(&fixture, "codex", "sess_host_only", "ash_live", ts);
    write_runtime_participant(
        &fixture,
        "ash_live",
        "codex",
        "sess_host_only",
        "running",
        true,
        Some("uaa-live-1"),
        None,
        ts,
    );

    let missing_backend_output = fixture.run(&[
        "agent",
        "turn",
        "--session",
        "sess_host_only",
        "--backend",
        "",
        "--prompt",
        "next",
        "--json",
    ]);
    assert_eq!(
        missing_backend_output.status.code(),
        Some(2),
        "public turn must fail closed when --backend is missing: {missing_backend_output:?}"
    );
    assert!(
        stderr_text(&missing_backend_output).contains("missing_backend"),
        "missing backend must stay classified explicitly: {missing_backend_output:?}"
    );

    let unknown_session_output = fixture.run(&[
        "agent",
        "turn",
        "--session",
        "sess_missing",
        "--backend",
        "cli:codex",
        "--prompt",
        "next",
        "--json",
    ]);
    assert_eq!(
        unknown_session_output.status.code(),
        Some(2),
        "unknown orchestration sessions must fail closed: {unknown_session_output:?}"
    );
    assert!(
        stderr_text(&unknown_session_output).contains("unknown_session"),
        "unknown orchestration sessions must keep the frozen classifier: {unknown_session_output:?}"
    );

    write_orchestration_session(
        &fixture,
        "codex",
        "sess_stopped",
        Some("ash_stopped"),
        "stopped",
        None,
        None,
        ts,
    );
    write_runtime_participant(
        &fixture,
        "ash_stopped",
        "codex",
        "sess_stopped",
        "running",
        true,
        Some("uaa-live-stopped"),
        None,
        ts,
    );
    let missing_parent_output = fixture.run(&[
        "agent",
        "turn",
        "--session",
        "sess_stopped",
        "--backend",
        "cli:codex",
        "--prompt",
        "next",
        "--json",
    ]);
    assert_eq!(
        missing_parent_output.status.code(),
        Some(2),
        "inactive parents must fail closed for follow-up turns: {missing_parent_output:?}"
    );
    assert!(
        stderr_text(&missing_parent_output).contains("missing_active_parent"),
        "inactive parents must keep the missing_active_parent classifier: {missing_parent_output:?}"
    );

    let backend_not_in_session_output = fixture.run(&[
        "agent",
        "turn",
        "--session",
        "sess_host_only",
        "--backend",
        "cli:claude_code",
        "--prompt",
        "next",
        "--json",
    ]);
    assert_eq!(
        backend_not_in_session_output.status.code(),
        Some(2),
        "public turn must fail closed when the backend is not present in the orchestration session: {backend_not_in_session_output:?}"
    );
    assert!(
        stderr_text(&backend_not_in_session_output).contains("backend_not_in_session"),
        "backend-not-in-session must keep the frozen classifier: {backend_not_in_session_output:?}"
    );
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn public_turn_fail_closed_taxonomy_is_explicit_for_world_linkage_ambiguity_and_detached_rejection()
{
    let fixture = AgentControlFixture::new();
    fixture.init_workspace();
    fixture.write_runtime_inventory(true);

    let ts = "2026-05-05T00:00:00Z";

    write_orchestration_session(
        &fixture,
        "codex",
        "sess_world_stale",
        Some("ash_owner_stale"),
        "active",
        Some("world-17"),
        Some(2),
        ts,
    );
    write_runtime_participant(
        &fixture,
        "ash_owner_stale",
        "codex",
        "sess_world_stale",
        "running",
        true,
        Some("uaa-owner-1"),
        None,
        ts,
    );
    write_world_member_participant(
        &fixture,
        "ash_member_stale",
        "claude_code",
        "sess_world_stale",
        "ash_stale_owner",
        "world-17",
        2,
        "ready",
        true,
        Some("uaa-member-stale"),
        ts,
    );

    let stale_linkage_output = fixture.run(&[
        "agent",
        "turn",
        "--session",
        "sess_world_stale",
        "--backend",
        "cli:claude_code",
        "--prompt",
        "next",
        "--json",
    ]);
    assert_eq!(
        stale_linkage_output.status.code(),
        Some(2),
        "world follow-up must fail closed when retained linkage drifts: {stale_linkage_output:?}"
    );
    assert!(
        stderr_text(&stale_linkage_output).contains("stale_linkage"),
        "world linkage drift must keep the stale_linkage classifier: {stale_linkage_output:?}"
    );

    write_orchestration_session(
        &fixture,
        "codex",
        "sess_world_ambiguous",
        Some("ash_owner_ambiguous"),
        "active",
        Some("world-18"),
        Some(3),
        ts,
    );
    write_runtime_participant(
        &fixture,
        "ash_owner_ambiguous",
        "codex",
        "sess_world_ambiguous",
        "running",
        true,
        Some("uaa-owner-2"),
        None,
        ts,
    );
    write_world_member_participant(
        &fixture,
        "ash_member_a",
        "claude_code",
        "sess_world_ambiguous",
        "ash_owner_ambiguous",
        "world-18",
        3,
        "ready",
        true,
        Some("uaa-member-a"),
        ts,
    );
    write_world_member_participant(
        &fixture,
        "ash_member_b",
        "claude_code",
        "sess_world_ambiguous",
        "ash_owner_ambiguous",
        "world-18",
        3,
        "ready",
        true,
        Some("uaa-member-b"),
        ts,
    );

    let ambiguous_output = fixture.run(&[
        "agent",
        "turn",
        "--session",
        "sess_world_ambiguous",
        "--backend",
        "cli:claude_code",
        "--prompt",
        "next",
        "--json",
    ]);
    assert_eq!(
        ambiguous_output.status.code(),
        Some(2),
        "world follow-up must fail closed when multiple authoritative slots match the same backend: {ambiguous_output:?}"
    );
    assert!(
        stderr_text(&ambiguous_output).contains("ambiguous_backend_slot"),
        "ambiguous world member slots must keep the frozen classifier: {ambiguous_output:?}"
    );

    write_parked_world_orchestration_session(
        &fixture,
        "codex",
        "sess_world_detached",
        "ash_owner_detached",
        "world-19",
        4,
        ts,
    );
    write_runtime_participant(
        &fixture,
        "ash_owner_detached",
        "codex",
        "sess_world_detached",
        "running",
        false,
        Some("uaa-owner-3"),
        None,
        ts,
    );
    write_world_member_participant(
        &fixture,
        "ash_member_detached",
        "claude_code",
        "sess_world_detached",
        "ash_owner_detached",
        "world-19",
        4,
        "ready",
        false,
        Some("uaa-member-detached"),
        ts,
    );

    let detached_world_output = fixture.run(&[
        "agent",
        "turn",
        "--session",
        "sess_world_detached",
        "--backend",
        "cli:claude_code",
        "--prompt",
        "next",
        "--json",
    ]);
    assert_eq!(
        detached_world_output.status.code(),
        Some(2),
        "detached world follow-up must fail closed until the parent owner is reattached: {detached_world_output:?}"
    );
    let detached_world_stderr = stderr_text(&detached_world_output);
    assert!(
        detached_world_stderr.contains("unsupported_platform_or_posture"),
        "detached world follow-up must keep the detached-world posture classifier: {detached_world_output:?}"
    );
    assert!(
        detached_world_stderr.contains("substrate agent reattach --session sess_world_detached"),
        "detached world rejection must direct callers through reattach before world follow-up resumes: {detached_world_output:?}"
    );
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn public_turn_routes_linux_world_member_follow_up_through_typed_submit_path() {
    let fixture = AgentControlFixture::new();
    fixture.init_workspace();
    fixture.write_runtime_inventory(true);

    let socket_home = tempfile::Builder::new()
        .prefix("sac-world-submit-")
        .tempdir_in("/tmp")
        .expect("socket tempdir");
    let socket_path = socket_home.path().join("world.sock");
    let server = ReplWorldAgentStub::start_with_member_dispatch_scripts(
        &socket_path,
        StreamBehavior::Normal,
        vec![MemberDispatchStreamScript::ReadyAndHoldUntilCancel {
            session_handle_id: "session-public-world-turn".to_string(),
            exit_code_on_cancel: 130,
        }],
    );
    let records = server.records();
    let mut repl = PtyRepl::spawn_with_world_socket(&fixture, &socket_path);
    repl.wait_for_output("Substrate v", Duration::from_secs(6))
        .expect("banner");
    repl.wait_for_output("substrate>", Duration::from_secs(2))
        .expect("prompt");

    repl.send_line("::cli:codex start retained host runtime");
    repl.wait_for_output(
        "shell-owned orchestrator session is ready via retained attached control ownership",
        Duration::from_secs(5),
    )
    .expect("host runtime ready");

    repl.send_line("::cli:claude_code member targeted first turn");
    repl.wait_for_output("substrate>", Duration::from_secs(5))
        .expect("prompt after initial world turn");

    let (orchestration_session_id, owner_participant_id) =
        wait_for_single_active_session(&fixture, Duration::from_secs(5));
    let owner_pid = fixture.load_orchestration_session(&orchestration_session_id)["shell_owner_pid"]
        .as_u64()
        .expect("owner pid") as u32;
    let live_members = wait_for_live_world_member_count(
        &fixture,
        &orchestration_session_id,
        1,
        Duration::from_secs(5),
    );
    let member = &live_members[0];
    let member_participant_id = member
        .get("participant_id")
        .and_then(Value::as_str)
        .expect("member participant_id")
        .to_string();
    let member_orchestrator_participant_id = member
        .get("orchestrator_participant_id")
        .and_then(Value::as_str)
        .expect("member orchestrator_participant_id")
        .to_string();
    assert_eq!(
        member_orchestrator_participant_id, owner_participant_id,
        "the retained world member must stay linked to the exact authoritative owner participant"
    );
    let world_id = member
        .get("world_id")
        .and_then(Value::as_str)
        .expect("member world_id")
        .to_string();
    let world_generation = member
        .get("world_generation")
        .and_then(Value::as_u64)
        .expect("member world_generation");

    let turn_output = fixture
        .command()
        .current_dir(&fixture.workspace_root)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args([
            "agent",
            "turn",
            "--session",
            &orchestration_session_id,
            "--backend",
            "cli:claude_code",
            "--prompt",
            "continue in world",
            "--json",
        ])
        .output()
        .expect("run public world turn");
    assert!(
        turn_output.status.success(),
        "public world turn should succeed on Linux: {turn_output:?}"
    );
    let turn_records = parse_ndjson_output(&turn_output);
    let turn_json = find_ndjson_record(&turn_records, "completed");
    assert_eq!(
        turn_json.get("action").and_then(Value::as_str),
        Some("turn")
    );
    assert_eq!(
        turn_json
            .get("orchestration_session_id")
            .and_then(Value::as_str),
        Some(orchestration_session_id.as_str())
    );
    assert_eq!(
        turn_json.get("backend_id").and_then(Value::as_str),
        Some("cli:claude_code")
    );
    assert_eq!(
        turn_json.get("turn_outcome").and_then(Value::as_str),
        Some("success")
    );
    assert_eq!(
        turn_json.get("session_posture").and_then(Value::as_str),
        Some("active")
    );

    let guard = records.lock().expect("lock world-service records");
    assert_eq!(
        guard.member_turn_submit_requests.len(),
        1,
        "public world follow-up must submit exactly one typed member turn request: {guard:#?}"
    );
    let submit = &guard.member_turn_submit_requests[0];
    assert_eq!(submit.orchestration_session_id, orchestration_session_id);
    assert_eq!(submit.participant_id, member_participant_id);
    assert_eq!(submit.orchestrator_participant_id, owner_participant_id);
    assert_eq!(submit.backend_id, "cli:claude_code");
    assert_eq!(submit.world_id, world_id);
    assert_eq!(submit.world_generation, world_generation);
    assert_eq!(submit.prompt, "continue in world");
    drop(guard);

    assert!(
        String::from_utf8_lossy(&turn_output.stdout)
            .contains("__MEMBER_TURN_SUBMIT_STUB__ continue in world"),
        "public world follow-up must surface typed submit output: {turn_output:?}"
    );

    repl.send_line("exit");
    let (_code, _out) = repl.shutdown_graceful(Duration::from_secs(3));
    terminate_pid(owner_pid);
}

#[test]
#[serial]
fn public_root_start_rejects_world_scoped_backends_in_v1() {
    let fixture = AgentControlFixture::new();
    fixture.init_workspace();
    fixture.write_runtime_inventory(true);

    let output = fixture.run(&[
        "agent",
        "start",
        "--backend",
        "cli:claude_code",
        "--prompt",
        "hello",
        "--json",
    ]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "world-scoped root start must fail closed: {output:?}"
    );
    let stderr = stderr_text(&output);
    assert!(
        stderr.contains("unsupported_platform_or_posture"),
        "world-scoped root start must classify the posture failure exactly: {stderr}"
    );
    assert!(
        stderr.contains("public root start is host-only in v1"),
        "world-scoped root start failure must explain the Linux-first host-only contract: {stderr}"
    );
}

#[test]
#[serial]
fn public_command_mode_remains_shell_wrap_not_agent_prompt() {
    let fixture = AgentControlFixture::new();
    fixture.init_workspace();
    fs::write(
        fixture.workspace_root.join(".substrate/workspace.yaml"),
        "world:\n  enabled: false\n",
    )
    .expect("disable world in workspace config");
    fixture.write_runtime_inventory(false);
    fs::write(
        fixture.workspace_root.join(".substrate-profile"),
        "id: test-policy\nname: Test Policy\nworld_fs:\n  host_visible: true\n  fail_closed:\n    routing: false\n  write:\n    enabled: true\nnet_allowed: []\ncmd_allowed: []\ncmd_denied: []\ncmd_isolated: []\nrequire_approval: false\nallow_shell_operators: true\nlimits:\n  max_memory_mb: null\n  max_cpu_percent: null\n  max_runtime_ms: null\n  max_egress_bytes: null\nmetadata: {}\n",
    )
    .expect("write host-only profile");
    fs::write(
        fixture.substrate_home.join("policy.yaml"),
        "id: test-global-policy\nname: Test Global Policy\nworld_fs:\n  host_visible: true\n  fail_closed:\n    routing: false\n  write:\n    enabled: true\nnet_allowed: []\ncmd_allowed: []\ncmd_denied: []\ncmd_isolated: []\nrequire_approval: false\nallow_shell_operators: true\nlimits:\n  max_memory_mb: null\n  max_cpu_percent: null\n  max_runtime_ms: null\n  max_egress_bytes: null\nmetadata: {}\nagents:\n  allowed_backends:\n    - cli:codex\n",
    )
    .expect("write host-only policy");

    let output = fixture.run(&["-c", "printf shell-wrap"]);
    assert!(
        output.status.success(),
        "substrate -c must remain ordinary shell wrap mode: {output:?}"
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "shell-wrap");
    assert!(
        !stderr_text(&output).contains("missing_prompt_source"),
        "substrate -c must not be reinterpreted as an agent prompt surface: {output:?}"
    );
}

#[test]
#[serial]
fn public_stop_reaches_repl_owned_sessions_through_the_same_private_owner_plane() {
    let fixture = AgentControlFixture::new();
    fixture.write_runtime_inventory(false);

    let mut repl = PtyRepl::spawn(&fixture);
    repl.wait_for_output("Substrate v", Duration::from_secs(2))
        .expect("repl banner");
    repl.wait_for_output("substrate>", Duration::from_secs(2))
        .expect("initial prompt");
    repl.send_line("::cli:codex start retained host runtime");
    repl.wait_for_output(
        "shell-owned orchestrator session is ready via retained attached control ownership",
        Duration::from_secs(5),
    )
    .expect("runtime ready");
    let (orchestration_session_id, participant_id) =
        wait_for_single_active_session(&fixture, Duration::from_secs(5));
    assert!(
        wait_for_path(
            &stop_transport_path(&fixture, &orchestration_session_id, &participant_id),
            Duration::from_secs(5),
        ),
        "repl-owned sessions must publish the same per-session private stop transport"
    );
    let stop_output = fixture.run(&[
        "agent",
        "stop",
        "--session",
        &orchestration_session_id,
        "--json",
    ]);
    assert!(
        stop_output.status.success(),
        "public stop should succeed against a REPL-owned owner plane: {stop_output:?}"
    );
    let stop_json = parse_json_output(&stop_output);
    assert_eq!(
        stop_json.get("action").and_then(Value::as_str),
        Some("stop")
    );
    assert_eq!(
        stop_json
            .get("orchestration_session_id")
            .and_then(Value::as_str),
        Some(orchestration_session_id.as_str())
    );
    assert_eq!(
        fixture
            .load_orchestration_session(&orchestration_session_id)
            .get("state")
            .and_then(Value::as_str),
        Some("stopped"),
        "REPL-owned public stop must drive the same authoritative terminal session state"
    );

    repl.send_line("exit");
    let (code, output) = repl.shutdown_graceful(Duration::from_secs(5));
    assert_eq!(
        code,
        0,
        "repl should still exit cleanly after public stop:\n{}",
        String::from_utf8_lossy(&output)
    );
}
