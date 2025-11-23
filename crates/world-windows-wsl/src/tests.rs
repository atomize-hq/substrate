use super::backend::AgentApiMock;
use super::warm::WarmCmd;
use super::WindowsWslBackend;
use agent_api_types::{ExecuteRequest, ExecuteResponse, FsDiff as AgentFsDiff};
use anyhow::anyhow;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use world_api::WorldSpec;

struct MockAgent {
    capabilities: Mutex<VecDeque<anyhow::Result<serde_json::Value>>>,
    execute: Mutex<VecDeque<anyhow::Result<ExecuteResponse>>>,
    traces: Mutex<HashMap<String, serde_json::Value>>,
    requests: Mutex<Vec<ExecuteRequest>>,
    capability_calls: AtomicUsize,
}

impl MockAgent {
    fn new() -> Self {
        Self {
            capabilities: Mutex::new(VecDeque::new()),
            execute: Mutex::new(VecDeque::new()),
            traces: Mutex::new(HashMap::new()),
            requests: Mutex::new(Vec::new()),
            capability_calls: AtomicUsize::new(0),
        }
    }

    fn push_capabilities(&self, value: anyhow::Result<serde_json::Value>) {
        self.capabilities.lock().unwrap().push_back(value);
    }

    fn push_execute(&self, value: anyhow::Result<ExecuteResponse>) {
        self.execute.lock().unwrap().push_back(value);
    }

    fn insert_trace(&self, span_id: &str, value: serde_json::Value) {
        self.traces
            .lock()
            .unwrap()
            .insert(span_id.to_string(), value);
    }

    fn take_requests(&self) -> Vec<ExecuteRequest> {
        self.requests.lock().unwrap().clone()
    }

    fn capability_calls(&self) -> usize {
        self.capability_calls.load(Ordering::SeqCst)
    }
}

impl AgentApiMock for MockAgent {
    fn capabilities(&self) -> anyhow::Result<serde_json::Value> {
        self.capability_calls.fetch_add(1, Ordering::SeqCst);
        self.capabilities
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| Ok(json!({})))
    }

    fn execute(&self, request: ExecuteRequest) -> anyhow::Result<ExecuteResponse> {
        self.requests.lock().unwrap().push(request);
        self.execute.lock().unwrap().pop_front().unwrap_or_else(|| {
            Ok(ExecuteResponse {
                exit: 0,
                span_id: "span".to_string(),
                stdout_b64: BASE64_STANDARD.encode(b""),
                stderr_b64: BASE64_STANDARD.encode(b""),
                scopes_used: vec![],
                fs_diff: None,
            })
        })
    }

    fn get_trace(&self, span_id: &str) -> anyhow::Result<serde_json::Value> {
        self.traces
            .lock()
            .unwrap()
            .get(span_id)
            .cloned()
            .ok_or_else(|| anyhow!("missing trace for span {span_id}"))
    }
}

fn test_backend_with_agent() -> (
    WindowsWslBackend,
    Arc<MockAgent>,
    Arc<std::sync::atomic::AtomicUsize>,
) {
    let agent = Arc::new(MockAgent::new());
    let (warm_cmd, invocations) =
        WarmCmd::disabled("test-distro".to_string(), PathBuf::from("C:/repo"));
    let backend = WindowsWslBackend::with_mock_agent(
        "test-distro".to_string(),
        PathBuf::from("C:/repo"),
        warm_cmd,
        agent.clone(),
    )
    .expect("backend init");
    (backend, agent, invocations)
}

#[test]
fn ensure_session_reuses_handle() {
    let (backend, agent, warm_invocations) = test_backend_with_agent();
    agent.push_capabilities(Ok(json!({"v": 1})));
    agent.push_capabilities(Ok(json!({"v": 1})));

    let spec = WorldSpec::default();
    let first = backend.ensure_session(&spec).expect("session");
    let second = backend.ensure_session(&spec).expect("session");

    assert_eq!(first.id, second.id);
    assert_eq!(agent.capability_calls(), 2);
    assert_eq!(warm_invocations.load(Ordering::SeqCst), 0);
}

#[test]
fn ensure_session_runs_warm_on_failure() {
    let (backend, agent, warm_invocations) = test_backend_with_agent();
    agent.push_capabilities(Err(anyhow!("pipe missing")));
    agent.push_capabilities(Ok(json!({"ok": true})));

    let spec = WorldSpec::default();
    let handle = backend.ensure_session(&spec).expect("session after warm");
    assert!(handle.id.starts_with("wsl:test-distro:"));
    assert_eq!(warm_invocations.load(Ordering::SeqCst), 1);
}

#[test]
fn exec_routes_to_agent() {
    let (backend, agent, _) = test_backend_with_agent();
    agent.push_capabilities(Ok(json!({})));
    agent.push_execute(Ok(ExecuteResponse {
        exit: 0,
        span_id: "span-123".to_string(),
        stdout_b64: BASE64_STANDARD.encode(b"hello"),
        stderr_b64: BASE64_STANDARD.encode(b""),
        scopes_used: vec!["fs.write:/project".to_string()],
        fs_diff: Some(AgentFsDiff {
            writes: vec![PathBuf::from("/mnt/c/repo/new.txt")],
            ..Default::default()
        }),
    }));

    let spec = WorldSpec::default();
    let world = backend.ensure_session(&spec).expect("session");
    let req = ExecRequest {
        cmd: "echo hello".to_string(),
        cwd: PathBuf::from("C:/repo"),
        env: std::iter::once(("KEY".to_string(), "VALUE".to_string())).collect(),
        pty: false,
        span_id: Some("span-123".to_string()),
    };

    let result = backend.exec(&world, req.clone()).expect("exec result");
    assert_eq!(result.exit, 0);
    assert_eq!(result.stdout, b"hello");
    assert_eq!(result.stderr, b"");
    let diff = result.fs_diff.expect("fs diff");
    let map = diff.display_path.expect("display map");
    assert_eq!(
        map.get("/mnt/c/repo/new.txt"),
        Some(&"C:\\repo\\new.txt".to_string())
    );

    let recorded = agent.take_requests();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].cmd, req.cmd);
    assert_eq!(recorded[0].cwd.as_deref().unwrap(), "/mnt/c/repo");
}

#[test]
fn fs_diff_deserializes() {
    let (backend, agent, _) = test_backend_with_agent();
    agent.push_capabilities(Ok(json!({})));
    let world = backend
        .ensure_session(&WorldSpec::default())
        .expect("session");

    agent.insert_trace(
        "span",
        json!({
            "fs_diff": {
                "writes": ["/mnt/c/repo/new.txt"],
                "mods": [],
                "deletes": []
            }
        }),
    );

    let diff = backend.fs_diff(&world, "span").expect("fs diff");
    assert_eq!(diff.writes.len(), 1);
    assert_eq!(diff.writes[0], PathBuf::from("/mnt/c/repo/new.txt"));
    let display = diff.display_path.expect("display map");
    assert_eq!(
        display.get("/mnt/c/repo/new.txt"),
        Some(&"C:\\repo\\new.txt".to_string())
    );
}
