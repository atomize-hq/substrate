use crate::execution::config_model::{self, CliConfigOverrides, LlmGatewayMode};
use crate::execution::policy_snapshot::{
    request_world_network_routing, resolve_world_network_policy_for_cwd,
};
#[cfg(target_os = "windows")]
use crate::execution::pw;
use crate::execution::{WorldGatewayAction, WorldGatewayCmd, WorldGatewayStatusArgs};
use agent_api_client::AgentClient;
use agent_api_types::{
    GatewayCliCodexIntegratedAuthV1, GatewayIntegratedAuthPayloadV1, GatewayLifecycleRequestV1,
    GatewayLifecycleResponseV1, GatewayStatusV1,
};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
#[cfg(target_os = "macos")]
use std::io::{Read, Write};
#[cfg(target_os = "macos")]
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::time::Duration;

#[cfg(target_os = "linux")]
const DEFAULT_WORLD_SOCKET_PATH: &str = "/run/substrate.sock";
const EXIT_INVALID_INTEGRATION: i32 = 2;
const EXIT_TRANSIENT_FAILURE: i32 = 3;
const EXIT_COMPONENT_UNAVAILABLE: i32 = 4;
const EXIT_POLICY_FAILURE: i32 = 5;
const CLI_CODEX_BACKEND: &str = "cli:codex";
const CODEX_ACCOUNT_ID_ENV: &str = "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID";
const CODEX_ACCESS_TOKEN_ENV: &str = "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN";

pub fn run(cmd: &WorldGatewayCmd) -> i32 {
    match run_inner(cmd) {
        Ok(exit_code) => exit_code,
        Err(err) => {
            eprintln!("substrate world gateway: {err:#}");
            1
        }
    }
}

fn run_inner(cmd: &WorldGatewayCmd) -> anyhow::Result<i32> {
    match &cmd.action {
        WorldGatewayAction::Sync => {
            run_typed_action("substrate world gateway sync", GatewayAction::Sync)
        }
        WorldGatewayAction::Status(args) => {
            run_typed_action_with_status_args("substrate world gateway status", args)
        }
        WorldGatewayAction::Restart => {
            run_typed_action("substrate world gateway restart", GatewayAction::Restart)
        }
    }
}

fn run_typed_action_with_status_args(
    command: &str,
    args: &WorldGatewayStatusArgs,
) -> anyhow::Result<i32> {
    let response = if world_routing_disabled() {
        synthesized_unavailable_response()
    } else {
        match call_gateway_action(GatewayAction::Status) {
            Ok(response) => response,
            Err(err) if error_is_component_unavailable(&err) => synthesized_unavailable_response(),
            Err(err) => return Ok(classify_and_print_gateway_error(command, err)),
        }
    };

    if response.status == GatewayStatusV1::Unavailable {
        if args.json {
            println!("{}", serde_json::to_string(&response)?);
        } else {
            eprintln!(
                "{}: unavailable (required gateway/world component unavailable)",
                command_for_status(command, args)
            );
        }
        return Ok(EXIT_COMPONENT_UNAVAILABLE);
    }

    if args.json {
        println!("{}", serde_json::to_string(&response)?);
    } else {
        println!("{command}: available");
    }

    Ok(0)
}

fn run_typed_action(command: &str, action: GatewayAction) -> anyhow::Result<i32> {
    let response = if world_routing_disabled() {
        synthesized_unavailable_response()
    } else {
        match call_gateway_action(action) {
            Ok(response) => response,
            Err(err) if error_is_component_unavailable(&err) => synthesized_unavailable_response(),
            Err(err) => return Ok(classify_and_print_gateway_error(command, err)),
        }
    };

    if response.status == GatewayStatusV1::Unavailable {
        return Ok(emit_unavailable(command));
    }

    println!("{command}: available");
    Ok(0)
}

fn call_gateway_action(action: GatewayAction) -> anyhow::Result<GatewayLifecycleResponseV1> {
    #[cfg(target_os = "macos")]
    {
        let request = build_gateway_request()?;
        let client = build_macos_gateway_client()?;

        match action {
            GatewayAction::Status => client.client.gateway_status(request).await_result(),
            GatewayAction::Sync => client.client.gateway_sync(request).await_result(),
            GatewayAction::Restart => client.client.gateway_restart(request).await_result(),
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let client = build_gateway_client()?;
        let request = build_gateway_request()?;

        match action {
            GatewayAction::Status => client.gateway_status(request).await_result(),
            GatewayAction::Sync => client.gateway_sync(request).await_result(),
            GatewayAction::Restart => client.gateway_restart(request).await_result(),
        }
    }
}

#[cfg(target_os = "macos")]
struct MacosGatewayClient {
    client: AgentClient,
    _forwarding: Option<world_mac_lima::ForwardingHandle>,
}

#[cfg(target_os = "macos")]
fn build_macos_gateway_client() -> anyhow::Result<MacosGatewayClient> {
    if let Some(socket_path) = std::env::var_os("SUBSTRATE_WORLD_SOCKET") {
        return Ok(MacosGatewayClient {
            client: AgentClient::unix_socket(std::path::PathBuf::from(socket_path))?,
            _forwarding: None,
        });
    }

    let default_sock = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".substrate/sock/agent.sock");
    if default_sock.exists() && probe_gateway_caps_uds(&default_sock) {
        return Ok(MacosGatewayClient {
            client: AgentClient::unix_socket(default_sock)?,
            _forwarding: None,
        });
    }

    let vm_name = std::env::var("SUBSTRATE_LIMA_VM_NAME")
        .or_else(|_| std::env::var("LIMA_VM_NAME"))
        .unwrap_or_else(|_| "substrate".to_string());
    let forwarding = world_mac_lima::forwarding::auto_select(&vm_name)?;
    let client = match forwarding.kind() {
        world_mac_lima::ForwardingKind::SshUds { path } => AgentClient::unix_socket(path.clone())?,
        world_mac_lima::ForwardingKind::SshTcp { port }
        | world_mac_lima::ForwardingKind::Vsock { port } => AgentClient::tcp("127.0.0.1", *port)?,
    };

    Ok(MacosGatewayClient {
        client,
        _forwarding: Some(forwarding),
    })
}

#[cfg_attr(not(test), allow(dead_code))]
#[cfg(target_os = "macos")]
enum MacosGatewayClientEndpoint {
    Unix(std::path::PathBuf),
    Tcp { host: String, port: u16 },
}

#[cfg_attr(not(test), allow(dead_code))]
#[cfg(target_os = "macos")]
fn resolve_macos_gateway_client_endpoint() -> MacosGatewayClientEndpoint {
    if let Some(socket_path) = std::env::var_os("SUBSTRATE_WORLD_SOCKET") {
        return MacosGatewayClientEndpoint::Unix(std::path::PathBuf::from(socket_path));
    }

    let default_sock = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".substrate/sock/agent.sock");

    if default_sock.exists() && probe_gateway_caps_uds(&default_sock) {
        MacosGatewayClientEndpoint::Unix(default_sock)
    } else {
        MacosGatewayClientEndpoint::Tcp {
            host: "127.0.0.1".to_string(),
            port: 17788,
        }
    }
}

#[cfg(target_os = "macos")]
fn probe_gateway_caps_uds(path: &std::path::Path) -> bool {
    let Ok(mut stream) = UnixStream::connect(path) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let request = b"GET /v1/capabilities HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    if stream.write_all(request).is_err() {
        return false;
    }

    let mut buf = [0u8; 512];
    match stream.read(&mut buf) {
        Ok(n) if n > 0 => std::str::from_utf8(&buf[..n])
            .unwrap_or("")
            .contains(" 200 "),
        _ => false,
    }
}

fn build_gateway_request() -> anyhow::Result<GatewayLifecycleRequestV1> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let (effective_config, config_explain) = config_model::resolve_effective_config_with_explain(
        &cwd,
        &CliConfigOverrides::default(),
        true,
    )?;
    validate_gateway_lifecycle_config(&effective_config, config_explain.as_ref())?;
    let (effective_policy, _) =
        substrate_broker::resolve_effective_policy_with_explain(&cwd, false)
            .map_err(|err| config_model::user_error(err.to_string()))?;
    let network_policy = resolve_world_network_policy_for_cwd(&cwd)?;
    let world_network = request_world_network_routing(&network_policy);
    let integrated_auth = resolve_integrated_auth_payload(&effective_config, &effective_policy)?;
    let gateway_mode = match effective_config.llm.gateway.mode {
        LlmGatewayMode::InWorld => "in_world",
        LlmGatewayMode::HostOnly => "host_only",
    };
    let mut env = HashMap::new();
    env.insert(
        "SUBSTRATE_LLM_GATEWAY_ENABLED".to_string(),
        if effective_config.llm.gateway.enabled {
            "1".to_string()
        } else {
            "0".to_string()
        },
    );
    env.insert(
        "SUBSTRATE_LLM_GATEWAY_MODE".to_string(),
        gateway_mode.to_string(),
    );
    env.insert(
        "SUBSTRATE_LLM_DEFAULT_BACKEND".to_string(),
        effective_config.llm.routing.default_backend.clone(),
    );

    Ok(GatewayLifecycleRequestV1 {
        profile: None,
        cwd: Some(cwd.display().to_string()),
        env: Some(env),
        agent_id: std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string()),
        policy_snapshot: network_policy.snapshot,
        world_network: Some(world_network),
        integrated_auth,
    })
}

fn validate_gateway_lifecycle_config(
    effective_config: &config_model::SubstrateConfig,
    config_explain: Option<&config_model::ConfigExplainV1>,
) -> anyhow::Result<()> {
    if !effective_config.llm.gateway.enabled
        && config_key_is_explicit(config_explain, "llm.gateway.enabled")
    {
        return Err(gateway_policy_blocked_error(
            "gateway lifecycle is disabled by effective config",
        ));
    }

    if effective_config.llm.gateway.mode == LlmGatewayMode::HostOnly {
        return Err(gateway_policy_blocked_error(
            "gateway lifecycle is unavailable while llm.gateway.mode=host_only",
        ));
    }

    if effective_config
        .llm
        .routing
        .default_backend
        .trim()
        .is_empty()
    {
        return Err(gateway_invalid_integration_error(
            "llm.routing.default_backend must be set before using gateway lifecycle commands",
        ));
    }

    Ok(())
}

fn config_key_is_explicit(
    config_explain: Option<&config_model::ConfigExplainV1>,
    key: &str,
) -> bool {
    config_explain
        .and_then(|explain| serde_json::to_value(explain).ok())
        .and_then(|value| {
            value
                .pointer(&format!("/keys/{key}/sources/0/layer"))
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .is_some_and(|layer| layer != "default")
}

fn resolve_integrated_auth_payload(
    effective_config: &config_model::SubstrateConfig,
    effective_policy: &substrate_broker::Policy,
) -> anyhow::Result<Option<GatewayIntegratedAuthPayloadV1>> {
    if !effective_config.llm.gateway.enabled
        || effective_config.llm.gateway.mode != LlmGatewayMode::InWorld
    {
        return Ok(None);
    }

    if effective_config.llm.routing.default_backend.trim() != CLI_CODEX_BACKEND {
        return Ok(None);
    }

    ensure_backend_allowed(
        &effective_policy.llm_allowed_backends,
        "llm.allowed_backends",
        CLI_CODEX_BACKEND,
    )?;

    Ok(Some(GatewayIntegratedAuthPayloadV1 {
        cli_codex: Some(resolve_cli_codex_integrated_auth(effective_policy)?),
    }))
}

fn resolve_cli_codex_integrated_auth(
    effective_policy: &substrate_broker::Policy,
) -> anyhow::Result<GatewayCliCodexIntegratedAuthV1> {
    let env_access_token = read_trimmed_env(CODEX_ACCESS_TOKEN_ENV)
        .map_err(|err| gateway_invalid_integration_error(err.to_string()))?;
    let env_account_id = read_trimmed_env(CODEX_ACCOUNT_ID_ENV)
        .map_err(|err| gateway_invalid_integration_error(err.to_string()))?;

    if let Some(access_token) = env_access_token {
        ensure_env_name_allowed(effective_policy, CODEX_ACCESS_TOKEN_ENV)?;
        if env_account_id.is_some() {
            ensure_env_name_allowed(effective_policy, CODEX_ACCOUNT_ID_ENV)?;
        }

        return Ok(GatewayCliCodexIntegratedAuthV1 {
            account_id: env_account_id,
            access_token,
        });
    }

    if env_account_id.is_some() {
        return Err(gateway_invalid_integration_error(format!(
            "integrated Codex auth handoff is incomplete: {} is set without {}",
            CODEX_ACCOUNT_ID_ENV, CODEX_ACCESS_TOKEN_ENV
        )));
    }

    ensure_backend_allowed(
        &effective_policy.agents_host_credentials_read_allowed_backends,
        "agents.host_credentials.read.allowed_backends",
        CLI_CODEX_BACKEND,
    )?;

    let auth_path = codex_auth_state_path();
    let content = fs::read_to_string(&auth_path).map_err(|err| {
        gateway_invalid_integration_error(format!(
            "failed to read Codex auth state from {}: {}",
            auth_path.display(),
            err
        ))
    })?;
    let json: Value = serde_json::from_str(&content).map_err(|err| {
        gateway_invalid_integration_error(format!(
            "failed to parse Codex auth state at {}: {}",
            auth_path.display(),
            err
        ))
    })?;

    let access_token = find_json_string(&json, &["access_token"]).ok_or_else(|| {
        gateway_invalid_integration_error("Codex auth state is missing access_token")
    })?;
    let account_id = find_json_string(&json, &["account_id"]);

    Ok(GatewayCliCodexIntegratedAuthV1 {
        account_id,
        access_token,
    })
}

fn ensure_backend_allowed(
    allowed_backends: &[String],
    policy_path: &str,
    backend_id: &str,
) -> anyhow::Result<()> {
    if allowed_backends.iter().any(|value| value == backend_id) {
        return Ok(());
    }

    Err(gateway_policy_blocked_error(format!(
        "{backend_id} is not allowlisted by effective policy {policy_path}"
    )))
}

fn ensure_env_name_allowed(
    effective_policy: &substrate_broker::Policy,
    env_name: &str,
) -> anyhow::Result<()> {
    if effective_policy
        .llm_secrets_env_allowed
        .iter()
        .any(|value| value == env_name)
    {
        return Ok(());
    }

    Err(gateway_policy_blocked_error(format!(
        "{env_name} is not allowlisted by effective policy llm.secrets.env_allowed"
    )))
}

fn codex_auth_state_path() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codex")
        .join("auth.json")
}

fn find_json_string(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for key in keys {
                if let Some(raw) = map.get(*key).and_then(Value::as_str) {
                    let trimmed = raw.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
            map.values().find_map(|value| find_json_string(value, keys))
        }
        Value::Array(items) => items.iter().find_map(|value| find_json_string(value, keys)),
        _ => None,
    }
}

fn read_trimmed_env(key: &str) -> anyhow::Result<Option<String>> {
    match std::env::var(key) {
        Ok(value) => {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed))
            }
        }
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(anyhow::anyhow!("failed to read {key}: {err}")),
    }
}

fn gateway_invalid_integration_error(message: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("gateway_invalid_integration: {}", message.into())
}

fn gateway_policy_blocked_error(message: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("gateway_policy_blocked: {}", message.into())
}

fn world_routing_disabled() -> bool {
    matches!(std::env::var("SUBSTRATE_WORLD_ENABLED").as_deref(), Ok("0"))
        || matches!(std::env::var("SUBSTRATE_WORLD").as_deref(), Ok("disabled"))
        || matches!(
            std::env::var("SUBSTRATE_OVERRIDE_WORLD").as_deref(),
            Ok("disabled")
        )
}

fn synthesized_unavailable_response() -> GatewayLifecycleResponseV1 {
    GatewayLifecycleResponseV1 {
        status: GatewayStatusV1::Unavailable,
        client_wiring: None,
    }
}

fn error_is_component_unavailable(err: &anyhow::Error) -> bool {
    use std::io::ErrorKind;

    if err.chain().any(|cause| {
        cause
            .downcast_ref::<std::io::Error>()
            .is_some_and(|io_err| matches!(io_err.kind(), ErrorKind::NotFound))
    }) {
        return true;
    }

    err.chain().any(|cause| {
        let msg = cause.to_string().to_ascii_lowercase();
        msg.contains("world backend unavailable")
            || msg.contains("listener missing")
            || msg.contains("no such file or directory")
            || msg.contains("failed to open named pipe")
    })
}

fn error_is_transient_runtime_failure(err: &anyhow::Error) -> bool {
    use std::io::ErrorKind;

    if err.chain().any(|cause| {
        cause
            .downcast_ref::<std::io::Error>()
            .is_some_and(|io_err| {
                matches!(
                    io_err.kind(),
                    ErrorKind::ConnectionRefused
                        | ErrorKind::AddrNotAvailable
                        | ErrorKind::TimedOut
                        | ErrorKind::ConnectionReset
                        | ErrorKind::BrokenPipe
                )
            })
    }) {
        return true;
    }

    err.chain().any(|cause| {
        let msg = cause.to_string().to_ascii_lowercase();
        msg.contains("connection refused")
            || msg.contains("timed out")
            || msg.contains("timeout")
            || msg.contains("connection reset")
            || msg.contains("broken pipe")
            || msg.contains("failed to connect")
    })
}

fn command_for_status<'a>(command: &'a str, args: &WorldGatewayStatusArgs) -> &'a str {
    if args.json {
        "substrate world gateway status --json"
    } else {
        command
    }
}

fn emit_unavailable(command: &str) -> i32 {
    eprintln!("{command}: unavailable (required gateway/world component unavailable)");
    EXIT_COMPONENT_UNAVAILABLE
}

fn classify_and_print_gateway_error(command: &str, err: anyhow::Error) -> i32 {
    let (exit_code, label) = if error_has_marker(&err, "gateway_invalid_integration:") {
        (EXIT_INVALID_INTEGRATION, "invalid integration")
    } else if error_has_marker(&err, "gateway_policy_blocked:") {
        (EXIT_POLICY_FAILURE, "policy or safety failure")
    } else if error_has_marker(&err, "gateway_transient_failure:")
        || error_is_transient_runtime_failure(&err)
    {
        (EXIT_TRANSIENT_FAILURE, "transient runtime failure")
    } else {
        eprintln!("substrate world gateway: {err:#}");
        return 1;
    };

    eprintln!("{command}: {label}");
    eprintln!("substrate world gateway: {err:#}");
    exit_code
}

fn error_has_marker(err: &anyhow::Error, marker: &str) -> bool {
    err.chain().any(|cause| cause.to_string().contains(marker))
}

#[cfg(target_os = "linux")]
fn build_gateway_client() -> anyhow::Result<AgentClient> {
    let socket_path = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(DEFAULT_WORLD_SOCKET_PATH));
    AgentClient::unix_socket(socket_path)
}

#[cfg(target_os = "windows")]
fn build_gateway_client() -> anyhow::Result<AgentClient> {
    pw::windows::build_agent_client()
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn build_gateway_client() -> anyhow::Result<AgentClient> {
    anyhow::bail!("gateway runtime client is unsupported on this platform")
}

#[derive(Clone, Copy)]
enum GatewayAction {
    Status,
    Sync,
    Restart,
}

trait AwaitGatewayResult {
    fn await_result(self) -> anyhow::Result<GatewayLifecycleResponseV1>;
}

impl<F> AwaitGatewayResult for F
where
    F: std::future::Future<Output = anyhow::Result<GatewayLifecycleResponseV1>>,
{
    fn await_result(self) -> anyhow::Result<GatewayLifecycleResponseV1> {
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(self)
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    fn with_env_var<T>(key: &str, value: Option<&std::ffi::OsStr>, f: impl FnOnce() -> T) -> T {
        let prev = std::env::var_os(key);
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
        let result = f();
        match prev {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
        result
    }

    #[test]
    fn macos_gateway_client_endpoint_prefers_existing_host_socket() {
        let temp = tempfile::tempdir().expect("tempdir");
        let home = temp.path();
        let sock = home.join(".substrate/sock/agent.sock");
        std::fs::create_dir_all(sock.parent().expect("sock parent")).expect("create sock dir");
        let listener = std::os::unix::net::UnixListener::bind(&sock).expect("bind listener");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buf = [0u8; 256];
            let _ = stream.read(&mut buf);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}")
                .expect("write response");
        });

        with_env_var("HOME", Some(home.as_os_str()), || {
            with_env_var("SUBSTRATE_WORLD_SOCKET", None, || {
                match resolve_macos_gateway_client_endpoint() {
                    MacosGatewayClientEndpoint::Unix(path) => assert_eq!(path, sock),
                    MacosGatewayClientEndpoint::Tcp { .. } => {
                        panic!("expected unix endpoint when host socket exists")
                    }
                }
            })
        });

        server.join().expect("join server");
    }

    #[test]
    fn macos_gateway_client_endpoint_falls_back_to_tcp_when_host_socket_missing() {
        let temp = tempfile::tempdir().expect("tempdir");
        let home = temp.path();

        with_env_var("HOME", Some(home.as_os_str()), || {
            with_env_var("SUBSTRATE_WORLD_SOCKET", None, || {
                match resolve_macos_gateway_client_endpoint() {
                    MacosGatewayClientEndpoint::Tcp { host, port } => {
                        assert_eq!(host, "127.0.0.1");
                        assert_eq!(port, 17788);
                    }
                    MacosGatewayClientEndpoint::Unix(path) => {
                        panic!("expected tcp fallback when socket is missing, got {path:?}")
                    }
                }
            })
        });
    }

    #[test]
    fn macos_gateway_client_endpoint_falls_back_to_tcp_when_host_socket_is_stale() {
        let temp = tempfile::tempdir().expect("tempdir");
        let home = temp.path();
        let sock = home.join(".substrate/sock/agent.sock");
        std::fs::create_dir_all(sock.parent().expect("sock parent")).expect("create sock dir");
        std::fs::write(&sock, "").expect("create placeholder socket path");

        with_env_var("HOME", Some(home.as_os_str()), || {
            with_env_var("SUBSTRATE_WORLD_SOCKET", None, || {
                match resolve_macos_gateway_client_endpoint() {
                    MacosGatewayClientEndpoint::Tcp { host, port } => {
                        assert_eq!(host, "127.0.0.1");
                        assert_eq!(port, 17788);
                    }
                    MacosGatewayClientEndpoint::Unix(path) => {
                        panic!("expected tcp fallback when socket is stale, got {path:?}")
                    }
                }
            })
        });
    }
}
