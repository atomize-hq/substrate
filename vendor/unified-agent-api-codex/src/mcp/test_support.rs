use super::*;

pub(super) mod prelude {
    pub(crate) use serde_json::Value;
    #[cfg(unix)]
    pub(crate) use std::os::unix::fs::PermissionsExt;
    pub(crate) use std::{
        collections::{BTreeMap, HashMap},
        env,
        ffi::OsString,
        fs,
        path::PathBuf,
        sync::Arc,
        time::Duration,
    };
    pub(crate) use tokio::{
        io::{AsyncBufReadExt, BufReader},
        time,
    };
    pub(crate) use toml::Value as TomlValue;
}

use prelude::*;

pub(super) fn temp_config_manager() -> (tempfile::TempDir, McpConfigManager) {
    let dir = tempfile::tempdir().expect("tempdir");
    let manager = McpConfigManager::from_code_home(dir.path());
    (dir, manager)
}

pub(super) fn stdio_definition(command: &str) -> McpServerDefinition {
    McpServerDefinition {
        transport: McpTransport::Stdio(StdioServerDefinition {
            command: command.to_string(),
            args: Vec::new(),
            env: BTreeMap::new(),
            timeout_ms: Some(1500),
        }),
        description: None,
        tags: Vec::new(),
        tools: None,
    }
}

pub(super) fn streamable_definition(url: &str, bearer_var: &str) -> McpServerDefinition {
    McpServerDefinition {
        transport: McpTransport::StreamableHttp(StreamableHttpDefinition {
            url: url.to_string(),
            headers: BTreeMap::new(),
            bearer_env_var: Some(bearer_var.to_string()),
            connect_timeout_ms: Some(5000),
            request_timeout_ms: Some(5000),
        }),
        description: None,
        tags: Vec::new(),
        tools: Some(McpToolConfig {
            enabled: vec![],
            disabled: vec![],
        }),
    }
}

pub(super) fn write_fake_mcp_server() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let script_path = dir.path().join("fake-codex");
    let script = r#"#!/usr/bin/env python3
import json
import sys
import threading
import time

pending = {}

def send(payload):
    sys.stdout.write(json.dumps(payload) + "\n")
    sys.stdout.flush()

def mark_cancelled(target, reason="cancelled"):
    if target is None:
        return
    state = pending.get(str(target)) or {}
    conv_id = state.get("conversation_id")
    pending[str(target)] = {"status": "cancelled", "conversation_id": conv_id}
    if conv_id:
        send({"jsonrpc": "2.0", "method": "codex/event", "params": {"type": "cancelled", "conversation_id": conv_id, "reason": reason}})
    send({"jsonrpc": "2.0", "id": target, "error": {"code": -32800, "message": reason}})

def handle_codex(req_id, params):
    conversation_id = params.get("conversation_id") or params.get("conversationId") or f"conv-{req_id}"
    pending[str(req_id)] = {"status": "pending", "conversation_id": conversation_id}
    def worker():
        time.sleep(0.05)
        state = pending.get(str(req_id))
        if not state or state.get("status") == "cancelled":
            return
        send({"jsonrpc": "2.0", "method": "codex/event", "params": {"type": "approval_required", "approval_id": f"ap-{req_id}", "kind": "exec"}})
        time.sleep(0.05)
        state = pending.get(str(req_id))
        if not state or state.get("status") == "cancelled":
            return
        send({"jsonrpc": "2.0", "method": "codex/event", "params": {"type": "task_complete", "conversation_id": conversation_id, "result": {"ok": True}}})
        send({"jsonrpc": "2.0", "id": req_id, "result": {"conversation_id": conversation_id, "output": {"ok": True}}})
        pending.pop(str(req_id), None)
    threading.Thread(target=worker, daemon=True).start()

for line in sys.stdin:
    if not line.strip():
        continue
    msg = json.loads(line)
    method = msg.get("method")
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": msg.get("id"), "result": {"ready": True}})
    elif method == "tools/call":
        params = msg.get("params", {})
        tool = params.get("name")
        args = params.get("arguments", {})
        if tool in ["codex", "codex-reply"]:
            handle_codex(msg.get("id"), args)
    elif method == "$/cancelRequest":
        target = msg.get("params", {}).get("id")
        mark_cancelled(target, reason="client_cancel")
    elif method == "shutdown":
        send({"jsonrpc": "2.0", "id": msg.get("id"), "result": {"ok": True}})
        break
    elif method == "exit":
        break
"#;

    fs::write(&script_path, script).expect("write script");
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&script_path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("chmod");
    }
    (dir, script_path)
}

pub(super) fn write_fake_app_server() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let script_path = dir.path().join("fake-codex-app");
    let script = r#"#!/usr/bin/env python3
import json
import os
import sys
import threading
import time

pending = {}
turn_lookup = {}

log_path = os.environ.get("ARGV_LOG")
if log_path:
    with open(log_path, "w", encoding="utf-8") as fh:
        fh.write(json.dumps(sys.argv[1:]) + "\n")

def send(payload):
    sys.stdout.write(json.dumps(payload) + "\n")
    sys.stdout.flush()

def mark_cancelled(req_id, reason="cancelled"):
    if req_id is None:
        return
    state = pending.get(str(req_id)) or {}
    thread_id = state.get("thread_id") or "thread-unknown"
    turn_id = state.get("turn_id")
    pending[str(req_id)] = {"status": "cancelled", "thread_id": thread_id, "turn_id": turn_id}
    if turn_id:
        send({"jsonrpc": "2.0", "method": "task/notification", "params": {"type": "task_complete", "thread_id": thread_id, "turn_id": turn_id, "result": {"cancelled": True, "reason": reason}}})
    send({"jsonrpc": "2.0", "id": req_id, "error": {"code": -32800, "message": reason}})

def handle_turn(req_id, params):
    thread_id = params.get("threadId") or params.get("thread_id") or "thread-unknown"
    turn_id = params.get("turnId") or params.get("turn_id") or f"turn-{req_id}"
    pending[str(req_id)] = {"status": "pending", "thread_id": thread_id, "turn_id": turn_id}
    turn_lookup[turn_id] = req_id

    def worker():
        time.sleep(0.05)
        state = pending.get(str(req_id))
        if not state or state.get("status") == "cancelled":
            return
        send({"jsonrpc": "2.0", "method": "task/notification", "params": {"type": "item", "thread_id": thread_id, "turn_id": turn_id, "item": {"message": "processing"}}})
        time.sleep(0.05)
        state = pending.get(str(req_id))
        if not state or state.get("status") == "cancelled":
            return
        send({"jsonrpc": "2.0", "method": "task/notification", "params": {"type": "task_complete", "thread_id": thread_id, "turn_id": turn_id, "result": {"ok": True}}})
        send({"jsonrpc": "2.0", "id": req_id, "result": {"turn_id": turn_id, "accepted": True}})
        pending.pop(str(req_id), None)
        turn_lookup.pop(turn_id, None)

    threading.Thread(target=worker, daemon=True).start()

for line in sys.stdin:
    if not line.strip():
        continue
    msg = json.loads(line)
    method = msg.get("method")
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": msg.get("id"), "result": {"ready": True}})
    elif method == "thread/start":
        params = msg.get("params", {})
        thread_id = params.get("thread_id") or f"thread-{msg.get('id')}"
        send({"jsonrpc": "2.0", "id": msg.get("id"), "result": {"thread_id": thread_id}})
    elif method == "thread/resume":
        params = msg.get("params", {})
        thread_id = params.get("threadId") or params.get("thread_id")
        send({"jsonrpc": "2.0", "id": msg.get("id"), "result": {"thread_id": thread_id, "resumed": True}})
    elif method == "turn/start":
        handle_turn(msg.get("id"), msg.get("params", {}))
    elif method == "turn/interrupt":
        params = msg.get("params", {})
        turn_id = params.get("turnId") or params.get("turn_id")
        req_id = turn_lookup.get(turn_id)
        if req_id:
            mark_cancelled(req_id, reason="interrupted")
            turn_lookup.pop(turn_id, None)
            pending.pop(str(req_id), None)
        send({"jsonrpc": "2.0", "id": msg.get("id"), "result": {"interrupted": True}})
    elif method == "$/cancelRequest":
        target = msg.get("params", {}).get("id")
        mark_cancelled(target, reason="client_cancel")
    elif method == "shutdown":
        send({"jsonrpc": "2.0", "id": msg.get("id"), "result": {"ok": True}})
        break
    elif method == "exit":
        break
"#;

    fs::write(&script_path, script).expect("write script");
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&script_path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("chmod");
    }
    (dir, script_path)
}

pub(super) fn write_fake_app_server_fork_v1() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let script_path = dir.path().join("fake-codex-app-fork-v1");
    let rpc_log_path = dir.path().join("rpc-log.jsonl");

    fs::write(&rpc_log_path, "").expect("write rpc log");

    let script = r#"#!/usr/bin/env python3
import json
import os
import sys
import threading
import time

pending = {}

rpc_log_path = os.environ.get("RPC_LOG")
rpc_log_fh = None
if rpc_log_path:
    rpc_log_fh = open(rpc_log_path, "a", encoding="utf-8")

def log(payload):
    if rpc_log_fh is None:
        return
    rpc_log_fh.write(json.dumps(payload) + "\n")
    rpc_log_fh.flush()

def send(payload):
    sys.stdout.write(json.dumps(payload) + "\n")
    sys.stdout.flush()

def send_cancelled(req_id, reason="cancelled"):
    if req_id is None:
        return
    send({"jsonrpc": "2.0", "id": req_id, "error": {"code": -32800, "message": reason}})

def handle_turn_start(req_id, params):
    pending[str(req_id)] = {"status": "pending"}
    def worker():
        time.sleep(0.2)
        state = pending.get(str(req_id))
        if not state or state.get("status") == "cancelled":
            return
        send({"jsonrpc": "2.0", "id": req_id, "result": {"accepted": True}})
        pending.pop(str(req_id), None)
    threading.Thread(target=worker, daemon=True).start()

for line in sys.stdin:
    if not line.strip():
        continue
    msg = json.loads(line)
    log(msg)
    method = msg.get("method")

    if method == "initialize":
        send({"jsonrpc": "2.0", "id": msg.get("id"), "result": {"ready": True}})
    elif method == "thread/list":
        params = msg.get("params") or {}
        cursor = params.get("cursor")
        cwd = params.get("cwd")
        if cursor is None:
            data = [
                {"id": "t-a", "createdAt": 100, "updatedAt": 200, "cwd": cwd},
                {"id": "t-b", "createdAt": 101, "updatedAt": 200, "cwd": cwd},
            ]
            send({"jsonrpc": "2.0", "id": msg.get("id"), "result": {"data": data, "nextCursor": "cursor-1"}})
        elif cursor == "cursor-1":
            data = [
                {"id": "t-c", "createdAt": 101, "updatedAt": 200, "cwd": cwd},
                {"id": "t-x", "createdAt": 99, "updatedAt": 199, "cwd": cwd},
            ]
            send({"jsonrpc": "2.0", "id": msg.get("id"), "result": {"data": data, "nextCursor": None}})
        else:
            send({"jsonrpc": "2.0", "id": msg.get("id"), "result": {"data": [], "nextCursor": None}})
    elif method == "thread/fork":
        params = msg.get("params") or {}
        thread_id = params.get("threadId") or ""
        send({"jsonrpc": "2.0", "id": msg.get("id"), "result": {"thread": {"id": "forked-" + thread_id}}})
    elif method == "turn/start":
        handle_turn_start(msg.get("id"), msg.get("params") or {})
    elif method == "$/cancelRequest":
        target = (msg.get("params") or {}).get("id")
        if target is not None:
            pending[str(target)] = {"status": "cancelled"}
        send_cancelled(target, reason="cancelled")
    elif method == "shutdown":
        send({"jsonrpc": "2.0", "id": msg.get("id"), "result": {"ok": True}})
        break
    elif method == "exit":
        break
"#;

    fs::write(&script_path, script).expect("write script");
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&script_path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("chmod");
    }

    (dir, script_path, rpc_log_path)
}

pub(super) fn write_env_probe_server(var: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let script_path = dir.path().join("env-probe-server");
    let script = format!(
        r#"#!/usr/bin/env python3
import os
import sys
import time

sys.stdout.write(os.environ.get("{var}", "") + "\n")
sys.stdout.flush()
time.sleep(30)
"#
    );

    fs::write(&script_path, script).expect("write script");
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&script_path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("chmod");
    }
    (dir, script_path)
}

pub(super) fn test_config(binary: PathBuf) -> StdioServerConfig {
    StdioServerConfig {
        binary,
        code_home: None,
        current_dir: None,
        env: Vec::new(),
        app_server_analytics_default_enabled: false,
        mirror_stdio: false,
        startup_timeout: Duration::from_secs(5),
    }
}

pub(super) fn test_client() -> ClientInfo {
    ClientInfo {
        name: "tests".to_string(),
        version: "0.0.0".to_string(),
    }
}

pub(super) async fn start_fake_mcp_server() -> (tempfile::TempDir, CodexMcpServer) {
    let (dir, script) = write_fake_mcp_server();
    let config = test_config(script);
    let client = test_client();
    let server = CodexMcpServer::start(config, client)
        .await
        .expect("spawn mcp server");
    (dir, server)
}

pub(super) async fn start_fake_app_server() -> (tempfile::TempDir, CodexAppServer) {
    let (dir, script) = write_fake_app_server();
    let config = test_config(script);
    let client = test_client();
    let server = CodexAppServer::start(config, client)
        .await
        .expect("spawn app server");
    (dir, server)
}

pub(super) async fn start_fake_app_server_fork_v1() -> (tempfile::TempDir, CodexAppServer, PathBuf)
{
    let (dir, script, rpc_log) = write_fake_app_server_fork_v1();
    let mut config = test_config(script);
    config.env.push((
        OsString::from("RPC_LOG"),
        OsString::from(rpc_log.as_os_str()),
    ));
    let client = test_client();
    let server = CodexAppServer::start_experimental(config, client)
        .await
        .expect("spawn app server");
    (dir, server, rpc_log)
}
