use crate::execution::policy_snapshot::{
    request_world_network_routing, resolve_world_network_policy_for_cwd,
};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::execution::pw;
use crate::execution::{WorldGatewayAction, WorldGatewayCmd, WorldGatewayStatusArgs};
use agent_api_client::AgentClient;
use agent_api_types::{GatewayLifecycleRequestV1, GatewayLifecycleResponseV1, GatewayStatusV1};

#[cfg(target_os = "linux")]
const DEFAULT_WORLD_SOCKET_PATH: &str = "/run/substrate.sock";
const EXIT_COMPONENT_UNAVAILABLE: i32 = 4;

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
            Err(err) => return Err(err),
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
            Err(err) => return Err(err),
        }
    };

    if response.status == GatewayStatusV1::Unavailable {
        return Ok(emit_unavailable(command));
    }

    println!("{command}: available");
    Ok(0)
}

fn call_gateway_action(action: GatewayAction) -> anyhow::Result<GatewayLifecycleResponseV1> {
    let client = build_gateway_client()?;
    let request = build_gateway_request()?;

    match action {
        GatewayAction::Status => client.gateway_status(request).await_result(),
        GatewayAction::Sync => client.gateway_sync(request).await_result(),
        GatewayAction::Restart => client.gateway_restart(request).await_result(),
    }
}

fn build_gateway_request() -> anyhow::Result<GatewayLifecycleRequestV1> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let network_policy = resolve_world_network_policy_for_cwd(&cwd)?;
    let world_network = request_world_network_routing(&network_policy);

    Ok(GatewayLifecycleRequestV1 {
        profile: None,
        cwd: Some(cwd.display().to_string()),
        env: None,
        agent_id: std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string()),
        policy_snapshot: network_policy.snapshot,
        world_network: Some(world_network),
    })
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
            .is_some_and(|io_err| {
                matches!(
                    io_err.kind(),
                    ErrorKind::NotFound
                        | ErrorKind::ConnectionRefused
                        | ErrorKind::AddrNotAvailable
                        | ErrorKind::TimedOut
                )
            })
    }) {
        return true;
    }

    err.chain().any(|cause| {
        let msg = cause.to_string().to_ascii_lowercase();
        msg.contains("world backend unavailable")
            || msg.contains("listener missing")
            || msg.contains("no such file or directory")
            || msg.contains("connection refused")
            || msg.contains("failed to open named pipe")
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

#[cfg(target_os = "linux")]
fn build_gateway_client() -> anyhow::Result<AgentClient> {
    let socket_path = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(DEFAULT_WORLD_SOCKET_PATH));
    AgentClient::unix_socket(socket_path)
}

#[cfg(target_os = "macos")]
fn build_gateway_client() -> anyhow::Result<AgentClient> {
    if let Some(socket_path) = std::env::var_os("SUBSTRATE_WORLD_SOCKET") {
        return AgentClient::unix_socket(std::path::PathBuf::from(socket_path));
    }

    let ctx = pw::detect()?;
    match &ctx.transport {
        pw::WorldTransport::Unix(path) => AgentClient::unix_socket(path),
        pw::WorldTransport::Tcp { host, port } => AgentClient::tcp(host, *port),
        pw::WorldTransport::Vsock { port } => AgentClient::tcp("127.0.0.1", *port),
    }
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
