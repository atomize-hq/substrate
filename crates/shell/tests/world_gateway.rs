#![cfg(unix)]

#[path = "common.rs"]
mod common;

use clap::Parser;
use common::substrate_shell_driver;
use predicates::prelude::*;
use serde_json::json;
use serde_json::Value as JsonValue;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use substrate_shell::execution::{Cli, SubCommands, WorldAction, WorldGatewayAction};
use tempfile::TempDir;

#[path = "support/socket.rs"]
mod socket;

use socket::{AgentSocket, SocketResponse};

fn short_socket_tempdir(prefix: &str) -> TempDir {
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in("/tmp")
        .expect("create short socket tempdir")
}

struct GatewayAuthFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    workspace_root: PathBuf,
}

impl GatewayAuthFixture {
    fn new() -> Self {
        let temp = common::temp_dir("substrate-world-gateway-auth-");
        let home = temp.path().join("home");
        fs::create_dir_all(&home).expect("create HOME fixture");
        let substrate_home = temp.path().join("substrate-home");
        fs::create_dir_all(&substrate_home).expect("create SUBSTRATE_HOME fixture");
        let workspace_root = temp.path().join("workspace");
        fs::create_dir_all(&workspace_root).expect("create workspace root");
        Self {
            _temp: temp,
            home,
            substrate_home,
            workspace_root,
        }
    }

    fn command(&self) -> assert_cmd::Command {
        let mut cmd = substrate_shell_driver();
        cmd.current_dir(&self.workspace_root)
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_HOME", &self.substrate_home);
        cmd
    }

    fn write_global_config(&self, contents: &str) {
        fs::write(self.substrate_home.join("config.yaml"), contents).expect("write config.yaml");
    }

    fn write_global_policy(&self, contents: &str) {
        fs::write(self.substrate_home.join("policy.yaml"), contents).expect("write policy.yaml");
    }

    fn write_codex_auth_state(&self, contents: &str) {
        let auth_dir = self.home.join(".codex");
        fs::create_dir_all(&auth_dir).expect("create .codex auth dir");
        fs::write(auth_dir.join("auth.json"), contents).expect("write auth.json");
    }

    fn write_global_agent_inventory(&self, name: &str, contents: &str) {
        let agents_dir = self.substrate_home.join("agents");
        fs::create_dir_all(&agents_dir).expect("create global agent inventory dir");
        fs::write(agents_dir.join(name), contents).expect("write agent inventory file");
    }
}

struct RecordedGatewayRequestSocket {
    _temp: TempDir,
    socket_path: PathBuf,
    recorded_request: Arc<Mutex<Option<JsonValue>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl RecordedGatewayRequestSocket {
    fn start(response: JsonValue) -> Self {
        let temp = short_socket_tempdir("sub-gwr-");
        let socket_path = temp.path().join("agent.sock");
        let listener = UnixListener::bind(&socket_path).expect("bind gateway capture socket");
        let recorded_request = Arc::new(Mutex::new(None));
        let recorded_request_for_thread = recorded_request.clone();

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept gateway capture request");
            let request = read_http_request(&mut stream).expect("read gateway HTTP request");
            let json: JsonValue =
                serde_json::from_slice(&request.body).expect("parse gateway request JSON");
            *recorded_request_for_thread
                .lock()
                .expect("lock recorded gateway request") = Some(json);

            write_http_json_response(&mut stream, &response.to_string())
                .expect("write gateway capture response");
        });

        Self {
            _temp: temp,
            socket_path,
            recorded_request,
            handle: Some(handle),
        }
    }

    fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    fn recorded_request(&mut self) -> JsonValue {
        if let Some(handle) = self.handle.take() {
            handle.join().expect("join gateway capture thread");
        }
        self.recorded_request
            .lock()
            .expect("lock recorded gateway request")
            .clone()
            .expect("gateway request should be recorded")
    }
}

impl Drop for RecordedGatewayRequestSocket {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

struct HttpRequest {
    body: Vec<u8>,
}

fn read_http_request(stream: &mut UnixStream) -> std::io::Result<HttpRequest> {
    let mut header_bytes = Vec::new();
    let mut buf = [0u8; 1];
    loop {
        stream.read_exact(&mut buf)?;
        header_bytes.push(buf[0]);
        if header_bytes.ends_with(b"\r\n\r\n") {
            break;
        }
    }

    let header = String::from_utf8_lossy(&header_bytes);
    let content_length = header
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        stream.read_exact(&mut body)?;
    }

    Ok(HttpRequest { body })
}

fn write_http_json_response(stream: &mut UnixStream, body: &str) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

fn gateway_config_with_codex_backend() -> &'static str {
    "llm:\n  enabled: true\n  gateway:\n    enabled: true\n  routing:\n    default_backend: cli:codex\n"
}

fn gateway_config_with_generic_backend() -> &'static str {
    "llm:\n  enabled: true\n  gateway:\n    enabled: true\n  routing:\n    default_backend: api:openai\n"
}

fn gateway_config_with_gateway_disabled() -> &'static str {
    "llm:\n  enabled: true\n  gateway:\n    enabled: false\n  routing:\n    default_backend: cli:codex\n"
}

fn gateway_config_with_empty_backend() -> &'static str {
    "llm:\n  enabled: true\n  gateway:\n    enabled: true\n  routing:\n    default_backend: \"\"\n"
}

fn gateway_inventory_for_codex() -> &'static str {
    r#"version: 1
id: codex
config:
  enabled: true
  kind: cli
  cli:
    binary: codex
  capabilities:
    llm: true
    mcp_client: false
"#
}

fn gateway_inventory_for_openai() -> &'static str {
    r#"version: 1
id: openai
config:
  enabled: true
  kind: api
  api:
    base_url: https://api.openai.com/v1
    auth:
      env:
        - OPENAI_API_KEY
  capabilities:
    llm: true
    mcp_client: false
"#
}

fn gateway_inventory_with_id_mismatch() -> &'static str {
    r#"version: 1
id: openai-wrong
config:
  enabled: true
  kind: api
  api:
    base_url: https://api.openai.com/v1
    auth:
      env:
        - OPENAI_API_KEY
  capabilities:
    llm: true
    mcp_client: false
"#
}

fn gateway_policy_with_codex_host_credentials() -> &'static str {
    r#"id: "gateway-policy"
name: "gateway-policy"

world_fs:
  host_visible: true
  fail_closed:
    routing: false
  write:
    enabled: true

llm:
  allowed_backends:
    - "cli:codex"

agents:
  host_credentials:
    read:
      allowed_backends:
        - "cli:codex"

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
"#
}

fn gateway_policy_with_openai_backend() -> &'static str {
    r#"id: "gateway-policy"
name: "gateway-policy"

world_fs:
  host_visible: true
  fail_closed:
    routing: false
  write:
    enabled: true

llm:
  allowed_backends:
    - "api:openai"

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
"#
}

fn gateway_policy_with_codex_env_override() -> &'static str {
    r#"id: "gateway-policy"
name: "gateway-policy"

world_fs:
  host_visible: true
  fail_closed:
    routing: false
  write:
    enabled: true

llm:
  allowed_backends:
    - "cli:codex"
  secrets:
    env_allowed:
      - "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID"
      - "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN"

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
"#
}

fn gateway_policy_missing_host_credentials_gate() -> &'static str {
    r#"id: "gateway-policy"
name: "gateway-policy"

world_fs:
  host_visible: true
  fail_closed:
    routing: false
  write:
    enabled: true

llm:
  allowed_backends:
    - "cli:codex"

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
"#
}

fn parse_world_gateway_status_json() -> Cli {
    Cli::try_parse_from(["substrate", "world", "gateway", "status", "--json"])
        .expect("gateway status --json should parse")
}

fn assert_gateway_unavailable(args: &[&str], expected_fragment: &str) {
    let mut cmd = substrate_shell_driver();
    cmd.args(args)
        .assert()
        .code(4)
        .stderr(predicate::str::contains(expected_fragment));
}

fn assert_gateway_unavailable_json(args: &[&str]) {
    let mut cmd = substrate_shell_driver();
    cmd.args(args)
        .assert()
        .code(4)
        .stdout("{\"status\":\"unavailable\"}\n")
        .stderr(predicate::str::is_empty());
}

fn gateway_socket_fixture() -> (TempDir, AgentSocket, std::path::PathBuf) {
    let temp = short_socket_tempdir("sub-gw-");
    let socket_path = temp.path().join("agent.sock");
    let socket = AgentSocket::start(
        &socket_path,
        SocketResponse::GatewayLifecycle {
            status: json!({
                "status": "available",
                "client_wiring": {
                    "openai_base_url": "http://gateway.test/openai",
                    "anthropic_base_url": "http://gateway.test/anthropic"
                }
            }),
            sync: json!({
                "status": "available",
                "client_wiring": {
                    "openai_base_url": "http://gateway.test/openai",
                    "anthropic_base_url": "http://gateway.test/anthropic"
                }
            }),
            restart: json!({
                "status": "available",
                "client_wiring": {
                    "openai_base_url": "http://gateway.test/openai",
                    "anthropic_base_url": "http://gateway.test/anthropic"
                }
            }),
        },
    );

    (temp, socket, socket_path)
}

fn gateway_unavailable_socket_fixture() -> (TempDir, AgentSocket, std::path::PathBuf) {
    let temp = short_socket_tempdir("sub-gwu-");
    let socket_path = temp.path().join("agent.sock");
    let socket = AgentSocket::start(
        &socket_path,
        SocketResponse::GatewayLifecycle {
            status: json!({
                "status": "unavailable"
            }),
            sync: json!({
                "status": "unavailable"
            }),
            restart: json!({
                "status": "unavailable"
            }),
        },
    );

    (temp, socket, socket_path)
}

fn stale_gateway_socket_path() -> (TempDir, std::path::PathBuf) {
    let temp = short_socket_tempdir("sub-gws-");
    let socket_path = temp.path().join("agent.sock");
    let listener = UnixListener::bind(&socket_path).expect("bind stale gateway socket");
    drop(listener);
    (temp, socket_path)
}

#[test]
fn world_gateway_status_accepts_json_leaf() {
    let cli = parse_world_gateway_status_json();

    match cli.sub {
        Some(SubCommands::World(world)) => match world.action {
            WorldAction::Gateway(gateway) => match gateway.action {
                WorldGatewayAction::Status(args) => {
                    assert!(args.json, "status --json leaf should set the json flag");
                }
                _ => panic!("expected world gateway status action"),
            },
            _ => panic!("expected world gateway command family"),
        },
        _ => panic!("expected world subcommand"),
    }
}

#[test]
fn world_gateway_rejects_archived_command_ordering() {
    for args in [
        ["substrate", "world", "status", "gateway"],
        ["substrate", "world", "sync", "gateway"],
        ["substrate", "world", "restart", "gateway"],
    ] {
        assert!(
            Cli::try_parse_from(args).is_err(),
            "archived ordering should not parse: {args:?}"
        );
    }
}

#[test]
fn world_gateway_absent_state_is_explicit_for_status_sync_and_restart() {
    assert_gateway_unavailable(
        &["world", "gateway", "status"],
        "substrate world gateway status: unavailable (required gateway/world component unavailable)",
    );
    assert_gateway_unavailable_json(&["world", "gateway", "status", "--json"]);
    assert_gateway_unavailable(
        &["world", "gateway", "sync"],
        "substrate world gateway sync: unavailable (required gateway/world component unavailable)",
    );
    assert_gateway_unavailable(
        &["world", "gateway", "restart"],
        "substrate world gateway restart: unavailable (required gateway/world component unavailable)",
    );
}

#[test]
fn world_gateway_status_json_uses_typed_runtime_contract() {
    let (_temp, _socket, socket_path) = gateway_socket_fixture();
    let fixture = GatewayAuthFixture::new();
    fixture.write_global_config(gateway_config_with_generic_backend());
    fixture.write_global_agent_inventory("openai.yaml", gateway_inventory_for_openai());
    fixture.write_global_policy(gateway_policy_with_openai_backend());

    let mut cmd = fixture.command();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args(["world", "gateway", "status", "--json"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains(
            "{\"status\":\"available\",\"client_wiring\":{\"openai_base_url\":\"http://gateway.test/openai\",\"anthropic_base_url\":\"http://gateway.test/anthropic\"}}",
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
fn world_gateway_status_json_preserves_unavailable_shape_from_runtime() {
    let (_temp, _socket, socket_path) = gateway_unavailable_socket_fixture();
    let fixture = GatewayAuthFixture::new();
    fixture.write_global_config(gateway_config_with_generic_backend());
    fixture.write_global_agent_inventory("openai.yaml", gateway_inventory_for_openai());
    fixture.write_global_policy(gateway_policy_with_openai_backend());

    let mut cmd = fixture.command();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args(["world", "gateway", "status", "--json"])
        .assert()
        .code(4)
        .stdout("{\"status\":\"unavailable\"}\n")
        .stderr(predicate::str::is_empty());
}

#[test]
fn world_gateway_disabled_state_skips_typed_runtime_bootstrap() {
    let (_temp, _socket, socket_path) = gateway_socket_fixture();

    let mut cmd = substrate_shell_driver();
    cmd.env("SUBSTRATE_WORLD_ENABLED", "0")
        .env("SUBSTRATE_WORLD", "disabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args(["world", "gateway", "status", "--json"])
        .assert()
        .code(4)
        .stdout("{\"status\":\"unavailable\"}\n")
        .stderr(predicate::str::is_empty());
}

#[test]
fn world_gateway_missing_socket_is_classified_as_absent_state() {
    let temp = short_socket_tempdir("sub-gwm-");
    let missing_socket_path = temp.path().join("missing.sock");
    let fixture = GatewayAuthFixture::new();
    fixture.write_global_config(gateway_config_with_generic_backend());
    fixture.write_global_agent_inventory("openai.yaml", gateway_inventory_for_openai());
    fixture.write_global_policy(gateway_policy_with_openai_backend());

    let mut cmd = fixture.command();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket_path)
        .args(["world", "gateway", "status"])
        .assert()
        .code(4)
        .stderr(predicate::str::contains(
            "substrate world gateway status: unavailable (required gateway/world component unavailable)",
        ));
}

#[test]
fn world_gateway_connection_refused_is_classified_as_transient_runtime_failure() {
    let (_temp, socket_path) = stale_gateway_socket_path();
    let fixture = GatewayAuthFixture::new();
    fixture.write_global_config(gateway_config_with_generic_backend());
    fixture.write_global_agent_inventory("openai.yaml", gateway_inventory_for_openai());
    fixture.write_global_policy(gateway_policy_with_openai_backend());

    let mut cmd = fixture.command();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args(["world", "gateway", "status"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains(
            "substrate world gateway status: transient runtime failure",
        ));
}

#[test]
fn world_gateway_http_failures_bubble_as_errors() {
    let temp = short_socket_tempdir("sub-gwe-");
    let socket_path = temp.path().join("agent.sock");
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::GatewayLifecycleHttpError {
            status: 500,
            body: "{\"error\":\"internal\"}".to_string(),
        },
    );
    let fixture = GatewayAuthFixture::new();
    fixture.write_global_config(gateway_config_with_generic_backend());
    fixture.write_global_agent_inventory("openai.yaml", gateway_inventory_for_openai());
    fixture.write_global_policy(gateway_policy_with_openai_backend());

    let mut cmd = fixture.command();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args(["world", "gateway", "status"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("substrate world gateway:"));
}

#[test]
fn world_gateway_invalid_integration_uses_exit_code_2() {
    let temp = short_socket_tempdir("sub-gwi-");
    let socket_path = temp.path().join("agent.sock");
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::GatewayLifecycleHttpError {
            status: 500,
            body: "{\"error\":\"internal: gateway_invalid_integration: unsupported integrated backend\"}".to_string(),
        },
    );
    let fixture = GatewayAuthFixture::new();
    fixture.write_global_config(gateway_config_with_generic_backend());
    fixture.write_global_agent_inventory("openai.yaml", gateway_inventory_for_openai());
    fixture.write_global_policy(gateway_policy_with_openai_backend());

    let mut cmd = fixture.command();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args(["world", "gateway", "sync"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "substrate world gateway sync: invalid integration",
        ));
}

#[test]
fn world_gateway_transient_runtime_failures_use_exit_code_3() {
    let temp = short_socket_tempdir("sub-gwt-");
    let socket_path = temp.path().join("agent.sock");
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::GatewayLifecycleHttpError {
            status: 500,
            body: "{\"error\":\"internal: gateway_transient_failure: gateway did not become ready before timeout\"}".to_string(),
        },
    );
    let fixture = GatewayAuthFixture::new();
    fixture.write_global_config(gateway_config_with_generic_backend());
    fixture.write_global_agent_inventory("openai.yaml", gateway_inventory_for_openai());
    fixture.write_global_policy(gateway_policy_with_openai_backend());

    let mut cmd = fixture.command();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args(["world", "gateway", "restart"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains(
            "substrate world gateway restart: transient runtime failure",
        ));
}

#[test]
fn world_gateway_policy_failures_use_exit_code_5() {
    let temp = short_socket_tempdir("sub-gwp-");
    let socket_path = temp.path().join("agent.sock");
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::GatewayLifecycleHttpError {
            status: 500,
            body: "{\"error\":\"internal: gateway_policy_blocked: gateway lifecycle is disabled by effective config\"}".to_string(),
        },
    );
    let fixture = GatewayAuthFixture::new();
    fixture.write_global_config(gateway_config_with_generic_backend());
    fixture.write_global_agent_inventory("openai.yaml", gateway_inventory_for_openai());
    fixture.write_global_policy(gateway_policy_with_openai_backend());

    let mut cmd = fixture.command();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args(["world", "gateway", "status"])
        .assert()
        .code(5)
        .stderr(predicate::str::contains(
            "substrate world gateway status: policy or safety failure",
        ));
}

#[test]
fn world_gateway_missing_inventory_uses_exit_code_2_before_socket_dispatch() {
    let temp = short_socket_tempdir("sub-gwmiss-");
    let missing_socket_path = temp.path().join("missing.sock");
    let fixture = GatewayAuthFixture::new();
    fixture.write_global_config(gateway_config_with_generic_backend());

    let mut cmd = fixture.command();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket_path)
        .args(["world", "gateway", "status"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "substrate world gateway status: invalid integration",
        ));
}

#[test]
fn world_gateway_inventory_filename_id_mismatch_uses_exit_code_2() {
    let fixture = GatewayAuthFixture::new();
    fixture.write_global_config(gateway_config_with_generic_backend());
    fixture.write_global_agent_inventory("openai.yaml", gateway_inventory_with_id_mismatch());
    fixture.write_global_policy(gateway_policy_with_openai_backend());

    let mut cmd = fixture.command();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .args(["world", "gateway", "status"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "substrate world gateway status: invalid integration",
        ));
}

#[test]
fn world_gateway_allowlist_denial_uses_exit_code_5() {
    let fixture = GatewayAuthFixture::new();
    fixture.write_global_config(gateway_config_with_generic_backend());
    fixture.write_global_agent_inventory("openai.yaml", gateway_inventory_for_openai());
    fixture.write_global_policy(gateway_policy_with_codex_host_credentials());

    let mut cmd = fixture.command();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .args(["world", "gateway", "status"])
        .assert()
        .code(5)
        .stderr(predicate::str::contains(
            "substrate world gateway status: policy or safety failure",
        ));
}

#[test]
fn world_gateway_sync_builds_integrated_auth_payload_from_host_auth_file() {
    let fixture = GatewayAuthFixture::new();
    fixture.write_global_config(gateway_config_with_codex_backend());
    fixture.write_global_agent_inventory("codex.yaml", gateway_inventory_for_codex());
    fixture.write_global_policy(gateway_policy_with_codex_host_credentials());
    fixture.write_codex_auth_state(
        r#"{
  "account_id": "acct_file_explicit",
  "access_token": "token-from-file"
}"#,
    );

    let mut socket = RecordedGatewayRequestSocket::start(json!({
        "status": "available",
        "client_wiring": {
            "openai_base_url": "http://gateway.test/openai",
            "anthropic_base_url": "http://gateway.test/anthropic"
        }
    }));

    let mut cmd = fixture.command();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", socket.socket_path())
        .args(["world", "gateway", "sync"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains(
            "substrate world gateway sync: available",
        ));

    let request = socket.recorded_request();
    assert_eq!(
        request.pointer("/integrated_auth/cli_codex/account_id"),
        Some(&json!("acct_file_explicit"))
    );
    assert_eq!(
        request.pointer("/integrated_auth/cli_codex/access_token"),
        Some(&json!("token-from-file"))
    );
}

#[test]
fn world_gateway_status_builds_integrated_auth_payload_from_allowed_env_override() {
    let fixture = GatewayAuthFixture::new();
    fixture.write_global_config(gateway_config_with_codex_backend());
    fixture.write_global_agent_inventory("codex.yaml", gateway_inventory_for_codex());
    fixture.write_global_policy(gateway_policy_with_codex_env_override());

    let mut socket = RecordedGatewayRequestSocket::start(json!({
        "status": "available",
        "client_wiring": {
            "openai_base_url": "http://gateway.test/openai",
            "anthropic_base_url": "http://gateway.test/anthropic"
        }
    }));

    let mut cmd = fixture.command();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", socket.socket_path())
        .env(
            "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID",
            "acct_env_explicit",
        )
        .env(
            "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN",
            "token-from-env",
        )
        .args(["world", "gateway", "status", "--json"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"status\":\"available\""));

    let request = socket.recorded_request();
    assert_eq!(
        request.pointer("/integrated_auth/cli_codex/account_id"),
        Some(&json!("acct_env_explicit"))
    );
    assert_eq!(
        request.pointer("/integrated_auth/cli_codex/access_token"),
        Some(&json!("token-from-env"))
    );
}

#[test]
fn world_gateway_host_credential_policy_denials_use_exit_code_5() {
    let fixture = GatewayAuthFixture::new();
    fixture.write_global_config(gateway_config_with_codex_backend());
    fixture.write_global_agent_inventory("codex.yaml", gateway_inventory_for_codex());
    fixture.write_global_policy(gateway_policy_missing_host_credentials_gate());
    fixture.write_codex_auth_state(
        r#"{
  "account_id": "acct_file_explicit",
  "access_token": "token-from-file"
}"#,
    );

    let mut cmd = fixture.command();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .args(["world", "gateway", "status"])
        .assert()
        .code(5)
        .stderr(predicate::str::contains(
            "substrate world gateway status: policy or safety failure",
        ));
}

#[test]
fn world_gateway_config_disabled_stays_policy_blocked() {
    let fixture = GatewayAuthFixture::new();
    fixture.write_global_config(gateway_config_with_gateway_disabled());
    fixture.write_global_policy(gateway_policy_with_codex_host_credentials());

    let mut cmd = fixture.command();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .args(["world", "gateway", "status"])
        .assert()
        .code(5)
        .stderr(predicate::str::contains(
            "substrate world gateway status: policy or safety failure",
        ));
}

#[test]
fn world_gateway_incomplete_env_override_uses_exit_code_2() {
    let fixture = GatewayAuthFixture::new();
    fixture.write_global_config(gateway_config_with_codex_backend());
    fixture.write_global_agent_inventory("codex.yaml", gateway_inventory_for_codex());
    fixture.write_global_policy(gateway_policy_with_codex_env_override());

    let mut cmd = fixture.command();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env(
            "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID",
            "acct_env_explicit",
        )
        .args(["world", "gateway", "status"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "substrate world gateway status: invalid integration",
        ));
}

#[test]
fn world_gateway_empty_default_backend_uses_exit_code_2() {
    let fixture = GatewayAuthFixture::new();
    fixture.write_global_config(gateway_config_with_empty_backend());
    fixture.write_global_policy(gateway_policy_with_codex_host_credentials());

    let mut cmd = fixture.command();
    cmd.env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .args(["world", "gateway", "status"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "substrate world gateway status: invalid integration",
        ));
}
