use super::backend::{
    set_test_windows_host_observation, set_test_wsl_command_outputs, set_test_wsl_observation,
    validate_wsl_mapping_v1, AgentApiMock,
};
use super::warm::WarmCmd;
use super::WindowsWslBackend;
use anyhow::anyhow;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use transport_api_client::Transport;
use transport_api_types::{
    ExecuteRequest, ExecuteResponse, FsDiff as AgentFsDiff, InstallBootstrapContextCarrierV1,
    InstallBootstrapContextV1, PlatformBootstrapMappingV1, ProcessTelemetry,
    WindowsForwarderScopeV1,
};
use world_api::{ExecRequest, WorldBackend, WorldSpec};

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
                shared_world: None,
                process_telemetry: ProcessTelemetry::not_supported_platform(),
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

fn test_host_carrier() -> InstallBootstrapContextCarrierV1 {
    InstallBootstrapContextCarrierV1::from_context(
        InstallBootstrapContextV1::new_windows(r"C:\Substrate", r"ACME\Alice", "S-1-5-21-1000")
            .expect("test Windows install bootstrap context"),
    )
    .expect("test Windows install bootstrap carrier")
}

fn test_platform_mapping(distro: &str, pipe_path: &str) -> PlatformBootstrapMappingV1 {
    let host = test_host_carrier();
    let scope = WindowsForwarderScopeV1::derive(
        "S-1-5-21-1000",
        distro,
        "0123456789abcdef0123456789abcdef",
        pipe_path,
    )
    .expect("test scope");
    PlatformBootstrapMappingV1::new_wsl(
        &host,
        distro,
        "0123456789abcdef0123456789abcdef",
        &format!(
            r"C:\Users\Alice\AppData\Local\Substrate\forwarder\{}",
            scope.0
        ),
        "/home/substrate/.substrate",
        "substrate",
        1000,
        pipe_path,
        "/run/substrate.sock",
    )
    .expect("test WSL platform mapping")
}

fn install_valid_mapping_observation(distro: &str) {
    set_test_windows_host_observation(Ok((
        r"ACME\Alice".to_string(),
        "S-1-5-21-1000".to_string(),
        r"C:\Users\Alice\AppData\Local".to_string(),
    )));
    set_test_wsl_observation(Ok((
        distro.to_string(),
        "0123456789abcdef0123456789abcdef".to_string(),
        "substrate".to_string(),
        1000,
        "/home/substrate".to_string(),
    )));
}

fn test_backend_with_agent() -> (
    WindowsWslBackend,
    Arc<MockAgent>,
    Arc<std::sync::atomic::AtomicUsize>,
    Arc<Mutex<Option<super::warm::WarmInvocation>>>,
) {
    let agent = Arc::new(MockAgent::new());
    let (warm_cmd, invocations, last_invocation) = WarmCmd::disabled(
        "test-distro".to_string(),
        PathBuf::from("C:/repo"),
        r"\\.\pipe\substrate-agent".to_string(),
        r"C:\Substrate".to_string(),
        "host-carrier".to_string(),
        "platform-mapping".to_string(),
    );
    let backend = WindowsWslBackend::with_mock_agent(
        "test-distro".to_string(),
        PathBuf::from("C:/repo"),
        warm_cmd,
        agent.clone(),
    )
    .expect("backend init");
    install_valid_mapping_observation("test-distro");
    (backend, agent, invocations, last_invocation)
}

#[test]
fn ensure_session_reuses_handle() {
    let (backend, agent, warm_invocations, _) = test_backend_with_agent();
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
    let (backend, agent, warm_invocations, _) = test_backend_with_agent();
    agent.push_capabilities(Err(anyhow!("pipe missing")));
    agent.push_capabilities(Ok(json!({"ok": true})));

    let spec = WorldSpec::default();
    let handle = backend.ensure_session(&spec).expect("session after warm");
    assert!(handle.id.starts_with("wsl:test-distro:"));
    assert_eq!(warm_invocations.load(Ordering::SeqCst), 1);
}

#[test]
fn ensure_ready_runs_warm_on_failure() {
    let (backend, agent, warm_invocations, _) = test_backend_with_agent();
    agent.push_capabilities(Err(anyhow!("pipe missing")));
    agent.push_capabilities(Ok(json!({"ok": true})));

    backend.ensure_ready().expect("ready after warm");
    assert_eq!(warm_invocations.load(Ordering::SeqCst), 1);
}

#[test]
fn ensure_persistent_session_ready_runs_warm_on_failure() {
    let (backend, agent, warm_invocations, _) = test_backend_with_agent();
    agent.push_capabilities(Err(anyhow!("pipe missing")));
    agent.push_capabilities(Ok(json!({"ok": true})));

    backend
        .ensure_persistent_session_ready()
        .expect("persistent session ready after warm");
    assert_eq!(warm_invocations.load(Ordering::SeqCst), 1);
}

#[test]
fn exec_routes_to_agent() {
    let (backend, agent, _, _) = test_backend_with_agent();
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
        shared_world: None,
        process_telemetry: ProcessTelemetry::not_supported_platform(),
    }));

    let spec = WorldSpec::default();
    let world = backend.ensure_session(&spec).expect("session");
    let req = ExecRequest {
        cmd: "echo hello".to_string(),
        cwd: PathBuf::from("C:/repo"),
        env: std::iter::once(("KEY".to_string(), "VALUE".to_string())).collect(),
        pty: false,
        span_id: Some("span-123".to_string()),
        shared_world: None,
        member_dispatch: None,
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
    let (backend, agent, _, _) = test_backend_with_agent();
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

#[test]
fn new_requires_mapping() {
    let err = match WindowsWslBackend::new() {
        Ok(_) => panic!("contextless constructor must fail"),
        Err(err) => err,
    };
    assert!(
        err.to_string()
            .contains("requires an authenticated install bootstrap carrier"),
        "unexpected error: {err:#}"
    );
}

#[test]
fn new_with_mapping_accepts_canonical_mapping_and_uses_named_pipe() {
    install_valid_mapping_observation("test-distro");
    let backend = WindowsWslBackend::new_with_mapping(
        test_host_carrier(),
        test_platform_mapping("test-distro", r"\\.\pipe\Substrate-Agent"),
        PathBuf::from("C:/repo"),
    )
    .expect("backend with mapping");

    assert_eq!(backend.distro, "test-distro");
    assert_eq!(
        backend.agent_pipe,
        PathBuf::from(r"\\.\pipe\substrate-agent")
    );
    match backend.agent_transport() {
        Transport::NamedPipe { path } => {
            assert_eq!(path, PathBuf::from(r"\\.\pipe\substrate-agent"));
        }
        other => panic!("expected named pipe transport, got {other:?}"),
    }
}

#[test]
fn new_with_mapping_projects_canonical_mapping_into_warm_cmd() {
    install_valid_mapping_observation("test-distro");
    let host = test_host_carrier();
    let mapping = test_platform_mapping("test-distro", r"\\.\pipe\Substrate-Agent");
    let expected_host = host.encode().expect("encoded host carrier");
    let expected_mapping = mapping
        .encode(&host)
        .expect("encoded platform bootstrap mapping");

    let backend = WindowsWslBackend::new_with_mapping(host, mapping, PathBuf::from("C:/repo"))
        .expect("backend with warm projection");

    assert_eq!(backend.warm_cmd.pipe_path, r"\\.\pipe\substrate-agent");
    assert_eq!(backend.warm_cmd.install_prefix, r"C:\Substrate");
    assert_eq!(backend.warm_cmd.install_bootstrap_context_v1, expected_host);
    assert_eq!(
        backend.warm_cmd.platform_bootstrap_mapping_v1,
        expected_mapping
    );
}

#[test]
fn new_with_mapping_rejects_non_windows_host_carrier_before_os_observation() {
    let unix_host = InstallBootstrapContextCarrierV1::from_context(
        InstallBootstrapContextV1::new_unix("/opt/substrate", "alice", 1000)
            .expect("unix install bootstrap context"),
    )
    .expect("unix install bootstrap carrier");
    let mapping = PlatformBootstrapMappingV1::new_wsl(
        &unix_host,
        "test-distro",
        "0123456789abcdef0123456789abcdef",
        r"C:\Users\Alice\AppData\Local\Substrate\forwarder\scope",
        "/home/substrate/.substrate",
        "substrate",
        1000,
        r"\\.\pipe\substrate-agent",
        "/run/substrate.sock",
    )
    .expect("mapping tied to unix carrier");

    let err =
        match WindowsWslBackend::new_with_mapping(unix_host, mapping, PathBuf::from("C:/repo")) {
            Ok(_) => panic!("non-Windows host carrier must fail"),
            Err(err) => err,
        };
    assert!(
        err.to_string()
            .contains("requires a Windows install bootstrap carrier"),
        "unexpected error: {err:#}"
    );
}

#[test]
fn validate_rejects_control_root_mismatch() {
    install_valid_mapping_observation("test-distro");
    let host = test_host_carrier();
    let mut mapping = test_platform_mapping("test-distro", r"\\.\pipe\substrate-agent");
    mapping.host_platform_control_root =
        r"C:\Users\Alice\AppData\Local\Substrate\forwarder\wrong".to_string();

    let err = validate_wsl_mapping_v1(&host, &mapping).expect_err("control root mismatch");
    assert!(
        err.to_string().contains("platform control root"),
        "unexpected error: {err:#}"
    );
}

#[test]
fn validate_rejects_live_wsl_identity_mismatch() {
    set_test_windows_host_observation(Ok((
        r"ACME\Alice".to_string(),
        "S-1-5-21-1000".to_string(),
        r"C:\Users\Alice\AppData\Local".to_string(),
    )));
    set_test_wsl_observation(Ok((
        "test-distro".to_string(),
        "ffffffffffffffffffffffffffffffff".to_string(),
        "substrate".to_string(),
        1000,
        "/home/substrate".to_string(),
    )));

    let err = validate_wsl_mapping_v1(
        &test_host_carrier(),
        &test_platform_mapping("test-distro", r"\\.\pipe\substrate-agent"),
    )
    .expect_err("machine ID mismatch");
    assert!(
        err.to_string().contains("guest machine ID"),
        "unexpected error: {err:#}"
    );
}

#[test]
fn observe_wsl_mapping_preserves_registered_names_with_repeated_spaces() {
    set_test_wsl_command_outputs(
        "Substrate  WSL\nUbuntu-24.04\n".to_string(),
        "  NAME              STATE           VERSION\n* Substrate  WSL    Running         2\n  Ubuntu-24.04      Stopped         2\n".to_string(),
        "machine_id=0123456789abcdef0123456789abcdef\naccount=substrate\nuid=1000\npasswd_by_name=substrate:x:1000:1000:Substrate:/home/substrate:/bin/bash\npasswd_by_uid=substrate:x:1000:1000:Substrate:/home/substrate:/bin/bash\n".to_string(),
    );

    let observation = super::backend::observe_wsl_mapping_v1("Substrate  WSL")
        .expect("exact registered distro name should parse");
    assert_eq!(observation.0, "Substrate  WSL");
    assert_eq!(observation.1, "0123456789abcdef0123456789abcdef");
}

#[test]
fn ensure_ready_fails_before_capabilities_when_mapping_revalidation_fails() {
    let (backend, agent, warm_invocations, _) = test_backend_with_agent();
    set_test_windows_host_observation(Ok((
        r"ACME\Bob".to_string(),
        "S-1-5-21-1000".to_string(),
        r"C:\Users\Alice\AppData\Local".to_string(),
    )));
    set_test_wsl_observation(Ok((
        "test-distro".to_string(),
        "0123456789abcdef0123456789abcdef".to_string(),
        "substrate".to_string(),
        1000,
        "/home/substrate".to_string(),
    )));
    agent.push_capabilities(Ok(json!({"ok": true})));

    let err = backend
        .ensure_ready()
        .expect_err("mapping mismatch must fail");
    assert!(
        err.to_string().contains("current Windows principal"),
        "unexpected error: {err:#}"
    );
    assert_eq!(agent.capability_calls(), 0);
    assert_eq!(warm_invocations.load(Ordering::SeqCst), 0);
}

#[test]
fn warm_invocation_projects_exact_mapping_arguments() {
    let (backend, agent, warm_invocations, last_invocation) = test_backend_with_agent();
    agent.push_capabilities(Err(anyhow!("pipe missing")));
    agent.push_capabilities(Ok(json!({"ok": true})));

    backend.ensure_ready().expect("ready after warm");
    assert_eq!(warm_invocations.load(Ordering::SeqCst), 1);

    let invocation = last_invocation
        .lock()
        .expect("warm invocation mutex")
        .clone()
        .expect("warm invocation");
    assert_eq!(
        invocation.args,
        vec![
            "-NoProfile".to_string(),
            "-NoLogo".to_string(),
            "-File".to_string(),
            "C:/repo/scripts/windows/wsl-warm.ps1".to_string(),
            "-DistroName".to_string(),
            "test-distro".to_string(),
            "-ProjectPath".to_string(),
            "C:/repo".to_string(),
            "-PipePath".to_string(),
            r"\\.\pipe\substrate-agent".to_string(),
            "-InstallPrefix".to_string(),
            r"C:\Substrate".to_string(),
            "-InstallBootstrapContextV1".to_string(),
            "host-carrier".to_string(),
            "-PlatformBootstrapMappingV1".to_string(),
            "platform-mapping".to_string(),
        ]
    );
    assert!(invocation
        .removed_env
        .contains(&"SUBSTRATE_FORWARDER_PIPE".to_string()));
    assert!(invocation
        .removed_env
        .contains(&"SUBSTRATE_FORWARDER_TCP".to_string()));
    assert!(invocation.removed_env.contains(&"LOCALAPPDATA".to_string()));
    assert!(invocation.removed_env.contains(&"USERPROFILE".to_string()));
    assert!(invocation.set_env.is_empty());
}
