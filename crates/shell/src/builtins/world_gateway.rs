use crate::execution::agent_inventory;
use crate::execution::agent_runtime::dispatch_contract::retired_exact_backend_selector_guidance;
#[cfg(unix)]
use crate::execution::config_model::CliConfigOverrides;
use crate::execution::config_model::{self, LlmGatewayMode};
#[cfg(unix)]
use crate::execution::policy_snapshot::{
    request_world_network_routing, resolve_world_network_policy_for_bootstrap_home,
};
#[cfg(target_os = "windows")]
use crate::execution::pw;
#[cfg(target_os = "macos")]
use crate::execution::pw;
#[cfg(unix)]
use crate::execution::{
    agent_runtime::host_session_authority::trusted_fs::TrustedAuthorityRoot,
    agent_runtime::HostSessionAuthority,
};
use crate::execution::{WorldGatewayAction, WorldGatewayCmd, WorldGatewayStatusArgs};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
#[cfg(target_os = "macos")]
use std::io::{Read, Write};
#[cfg(target_os = "macos")]
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
#[cfg(target_os = "macos")]
use std::time::Duration;
use substrate_common::identity::normalize_identity_tuple_client_id;
use transport_api_client::AgentClient;
#[cfg(unix)]
use transport_api_types::InstallBootstrapContextCarrierV1;
use transport_api_types::{
    GatewayApiEnvIntegratedAuthV1, GatewayCliCodexIntegratedAuthV1, GatewayIntegratedAuthPayloadV1,
    GatewayLifecycleRequestV1, GatewayLifecycleResponseV1, GatewayStatusV1, IdentityTuple,
    PlacementExecution, PlacementPosture,
};
#[cfg(target_os = "macos")]
use transport_api_types::{PlatformBootstrapMappingV1, PlatformTransportIdentityV1};

#[cfg(target_os = "linux")]
const DEFAULT_WORLD_SOCKET_PATH: &str = "/run/substrate.sock";
const EXIT_INVALID_INTEGRATION: i32 = 2;
const EXIT_TRANSIENT_FAILURE: i32 = 3;
const EXIT_COMPONENT_UNAVAILABLE: i32 = 4;
const EXIT_POLICY_FAILURE: i32 = 5;
const CLI_CLAUDE_CODE_HOST_BACKEND: &str = "cli:claude_code-host";
const CLI_CLAUDE_CODE_WORLD_BACKEND: &str = "cli:claude_code-world";
const CLI_CODEX_HOST_BACKEND: &str = "cli:codex-host";
const CLI_CODEX_WORLD_BACKEND: &str = "cli:codex-world";
const API_OPENAI_BACKEND: &str = "api:openai";
const API_ANTHROPIC_BACKEND: &str = "api:anthropic";
const SUBSTRATE_GATEWAY_ROUTER: &str = "substrate_gateway";
#[cfg(target_os = "macos")]
const WORLD_PROJECT_DIR_OVERRIDE_ENV: &str = "SUBSTRATE_WORLD_PROJECT_DIR";
const CODEX_ACCOUNT_ID_ENV: &str = "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID";
const CODEX_ACCESS_TOKEN_ENV: &str = "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN";
const OPENAI_API_KEY_ENV: &str = "OPENAI_API_KEY";
const ANTHROPIC_API_KEY_ENV: &str = "ANTHROPIC_API_KEY";
#[cfg(target_os = "macos")]
const MACOS_STAGED_WORKSPACE_CURRENT: &str = "/var/lib/substrate/staged-workspace/current";

struct GatewayLifecycleRequestContext {
    request: GatewayLifecycleRequestV1,
    identity_tuple: Option<IdentityTuple>,
    placement_posture: Option<PlacementPosture>,
    world_enabled: bool,
    #[cfg(test)]
    codex_auth_state_path: PathBuf,
}

pub fn run(
    cmd: &WorldGatewayCmd,
    no_world: bool,
    world: bool,
    launch_cwd: &Path,
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> i32 {
    match run_inner(
        cmd,
        no_world,
        world,
        launch_cwd,
        #[cfg(unix)]
        install_context,
    ) {
        Ok(exit_code) => exit_code,
        Err(err) => {
            eprintln!("substrate world gateway: {err:#}");
            1
        }
    }
}

fn run_inner(
    cmd: &WorldGatewayCmd,
    no_world: bool,
    world: bool,
    launch_cwd: &Path,
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> anyhow::Result<i32> {
    match &cmd.action {
        WorldGatewayAction::Sync => run_typed_action(
            "substrate world gateway sync",
            GatewayAction::Sync,
            no_world,
            world,
            launch_cwd,
            #[cfg(unix)]
            install_context,
        ),
        WorldGatewayAction::Status(args) => run_typed_action_with_status_args(
            "substrate world gateway status",
            args,
            no_world,
            world,
            launch_cwd,
            #[cfg(unix)]
            install_context,
        ),
        WorldGatewayAction::Restart => run_typed_action(
            "substrate world gateway restart",
            GatewayAction::Restart,
            no_world,
            world,
            launch_cwd,
            #[cfg(unix)]
            install_context,
        ),
    }
}

fn run_typed_action_with_status_args(
    command: &str,
    args: &WorldGatewayStatusArgs,
    no_world: bool,
    world: bool,
    launch_cwd: &Path,
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> anyhow::Result<i32> {
    let request_context = match build_gateway_request_context(
        no_world,
        world,
        launch_cwd,
        #[cfg(unix)]
        install_context,
    ) {
        Ok(context) => context,
        Err(err) => return Ok(classify_and_print_gateway_error(command, err)),
    };
    let response = if world_routing_disabled(&request_context) {
        synthesized_unavailable_response(&request_context)
    } else {
        match call_gateway_action(GatewayAction::Status, &request_context) {
            Ok(response) => response,
            Err(err) if error_is_component_unavailable(&err) => {
                synthesized_unavailable_response(&request_context)
            }
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
            print_status_identity_metadata_to_stderr(&response);
        }
        return Ok(EXIT_COMPONENT_UNAVAILABLE);
    }

    if args.json {
        println!("{}", serde_json::to_string(&response)?);
    } else {
        println!("{command}: available");
        print_status_identity_metadata(&response);
    }

    Ok(0)
}

fn run_typed_action(
    command: &str,
    action: GatewayAction,
    no_world: bool,
    world: bool,
    launch_cwd: &Path,
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> anyhow::Result<i32> {
    let request_context = match build_gateway_request_context(
        no_world,
        world,
        launch_cwd,
        #[cfg(unix)]
        install_context,
    ) {
        Ok(context) => context,
        Err(err) => return Ok(classify_and_print_gateway_error(command, err)),
    };
    let response = if world_routing_disabled(&request_context) {
        synthesized_unavailable_response(&request_context)
    } else {
        match call_gateway_action(action, &request_context) {
            Ok(response) => response,
            Err(err) if error_is_component_unavailable(&err) => {
                synthesized_unavailable_response(&request_context)
            }
            Err(err) => return Ok(classify_and_print_gateway_error(command, err)),
        }
    };

    if response.status == GatewayStatusV1::Unavailable {
        return Ok(emit_unavailable(command, &response));
    }

    println!("{command}: available");
    print_status_identity_metadata(&response);
    Ok(0)
}

fn call_gateway_action(
    action: GatewayAction,
    request_context: &GatewayLifecycleRequestContext,
) -> anyhow::Result<GatewayLifecycleResponseV1> {
    #[cfg(target_os = "macos")]
    {
        if cfg!(target_os = "macos") {
            return Err(gateway_invalid_integration_error(
                "authenticated macOS gateway endpoint selection is unavailable until R2-3",
            ));
        }
        let client = build_macos_gateway_client()?;

        let response = match action {
            GatewayAction::Status => client
                .client
                .gateway_status(request_context.request.clone())
                .await_result(),
            GatewayAction::Sync => client
                .client
                .gateway_sync(request_context.request.clone())
                .await_result(),
            GatewayAction::Restart => client
                .client
                .gateway_restart(request_context.request.clone())
                .await_result(),
        }?;

        augment_gateway_response(response, request_context)
    }

    #[cfg(not(target_os = "macos"))]
    {
        if !cfg!(target_os = "linux") {
            return Err(gateway_invalid_integration_error(
                "authenticated gateway endpoint selection is unavailable on this platform until R2-3",
            ));
        }
        let client = build_gateway_client()?;

        let response = match action {
            GatewayAction::Status => client
                .gateway_status(request_context.request.clone())
                .await_result(),
            GatewayAction::Sync => client
                .gateway_sync(request_context.request.clone())
                .await_result(),
            GatewayAction::Restart => client
                .gateway_restart(request_context.request.clone())
                .await_result(),
        }?;

        augment_gateway_response(response, request_context)
    }
}

#[cfg(target_os = "macos")]
struct MacosGatewayClient {
    client: AgentClient,
    _forwarding: Option<world_mac_lima::ForwardingHandle>,
}

#[cfg(target_os = "macos")]
fn macos_platform_world_context(
) -> anyhow::Result<std::sync::Arc<crate::execution::pw::PlatformWorldContext>> {
    if let Some(ctx) = pw::get_context() {
        return Ok(ctx);
    }

    let detected = pw::detect().map_err(|err| {
        gateway_invalid_integration_error(format!(
            "authenticated macOS platform mapping is unavailable: {err:#}"
        ))
    })?;
    pw::store_context_globally(detected);
    pw::get_context().ok_or_else(|| {
        gateway_invalid_integration_error("authenticated macOS platform mapping did not persist")
    })
}

#[cfg(target_os = "macos")]
fn macos_authenticated_host_socket() -> anyhow::Result<PathBuf> {
    #[cfg(test)]
    {
        let mapping = TEST_MACOS_GATEWAY_MAPPING.with(|slot| slot.borrow().clone());
        let mapping = mapping.ok_or_else(|| {
            gateway_invalid_integration_error(
                "authenticated macOS platform mapping is unavailable in unit-test builds",
            )
        })?;
        return macos_host_socket_from_mapping(&mapping);
    }

    let ctx = macos_platform_world_context()?;
    let mapping = ctx.bootstrap_mapping.as_ref().ok_or_else(|| {
        gateway_invalid_integration_error("authenticated macOS platform mapping is unavailable")
    })?;
    macos_host_socket_from_mapping(mapping)
}

#[cfg(target_os = "macos")]
fn macos_host_socket_from_mapping(mapping: &PlatformBootstrapMappingV1) -> anyhow::Result<PathBuf> {
    let PlatformTransportIdentityV1::Lima { host_socket, .. } = &mapping.realized_transport else {
        return Err(gateway_invalid_integration_error(
            "authenticated macOS platform mapping does not carry a Lima socket transport",
        ));
    };
    Ok(PathBuf::from(host_socket))
}

#[cfg(all(test, target_os = "macos"))]
thread_local! {
    static TEST_MACOS_GATEWAY_MAPPING: std::cell::RefCell<Option<PlatformBootstrapMappingV1>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(all(test, target_os = "macos"))]
fn with_test_authenticated_macos_mapping<T>(
    mapping: PlatformBootstrapMappingV1,
    f: impl FnOnce() -> T,
) -> T {
    TEST_MACOS_GATEWAY_MAPPING.with(|slot| {
        let previous = slot.replace(Some(mapping));
        let result = f();
        slot.replace(previous);
        result
    })
}

#[cfg(target_os = "macos")]
fn build_macos_gateway_client() -> anyhow::Result<MacosGatewayClient> {
    let socket = resolve_macos_host_gateway_socket().ok_or_else(|| {
        gateway_invalid_integration_error(
            "No forwarding transport available. Run scripts/mac/lima-doctor.sh",
        )
    })?;
    let client = AgentClient::unix_socket(socket)?;

    Ok(MacosGatewayClient {
        client,
        _forwarding: None,
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
    MacosGatewayClientEndpoint::Unix(
        resolve_macos_host_gateway_socket().unwrap_or_else(macos_default_world_socket_path),
    )
}

#[cfg(target_os = "macos")]
fn resolve_macos_host_gateway_socket() -> Option<PathBuf> {
    let default_sock = macos_authenticated_host_socket().ok()?;
    if default_sock.exists() && probe_gateway_caps_uds(&default_sock) {
        Some(default_sock)
    } else {
        None
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

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn macos_default_world_socket_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        macos_authenticated_host_socket().unwrap_or_default()
    }

    #[cfg(not(target_os = "macos"))]
    {
        substrate_common::paths::substrate_home()
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".substrate")
            })
            .join("sock/agent.sock")
    }
}

fn build_gateway_request_context(
    no_world: bool,
    world: bool,
    launch_cwd: &Path,
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> anyhow::Result<GatewayLifecycleRequestContext> {
    #[cfg(not(unix))]
    {
        let _ = (no_world, world, launch_cwd);
        // Keep the frozen projection closure type-checked on non-Unix targets while
        // this authenticated entrypoint fails before invoking any of it. R2-3 owns
        // native authority mapping and may make these helpers reachable there.
        let _compile_only_projection_closure = (
            validate_gateway_lifecycle_config,
            validate_gateway_backend_selection,
            derive_gateway_identity_tuple,
            derive_gateway_auth_authority,
            derive_gateway_placement_posture,
            resolve_integrated_auth_payload,
            codex_auth_state_path,
        );
        return Err(gateway_invalid_integration_error(
            "authenticated install bootstrap context is unavailable on this platform until R2-3",
        ));
    }

    #[cfg(unix)]
    {
        crate::execution::install_bootstrap::bind_unix_install_bootstrap_context(install_context)
            .map_err(|err| gateway_invalid_integration_error(err.to_string()))?;
        let root =
            TrustedAuthorityRoot::open(Path::new(&install_context.context.selected_host_prefix))
                .map_err(|err| gateway_invalid_integration_error(err.to_string()))?;
        let authority = HostSessionAuthority::from_trusted_root(root)
            .map_err(|err| gateway_invalid_integration_error(err.to_string()))?;
        let bootstrap_home = authority.bootstrap_home();
        let account_home = crate::execution::install_bootstrap::unix_account_home_for_principal(
            &install_context.context.intended_host_principal,
        )
        .map_err(|err| gateway_invalid_integration_error(err.to_string()))?;
        let committed_codex_auth_state_path = codex_auth_state_path(&account_home);
        let cwd = launch_cwd.to_path_buf();
        let cli_world_enabled = if world {
            Some(true)
        } else if no_world {
            Some(false)
        } else {
            None
        };
        let (effective_config, config_explain) =
            config_model::resolve_effective_config_with_explain_for_bootstrap_home(
                &cwd,
                &CliConfigOverrides {
                    world_enabled: cli_world_enabled,
                    ..Default::default()
                },
                &bootstrap_home,
                true,
            )?;
        validate_gateway_lifecycle_config(&effective_config, config_explain.as_ref())?;
        let effective_policy =
            crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
                &cwd,
                &bootstrap_home,
            )?;
        let selected_backend = effective_config
            .llm
            .routing
            .default_backend
            .trim()
            .to_string();
        let backend_entry = validate_gateway_backend_selection(
            &cwd,
            &effective_policy,
            &selected_backend,
            &bootstrap_home,
        )?;
        let network_policy = resolve_world_network_policy_for_bootstrap_home(
            &cwd,
            &bootstrap_home,
            &effective_config,
        )?;
        let world_network = request_world_network_routing(&network_policy);
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
        #[cfg(target_os = "macos")]
        if effective_config.llm.gateway.mode == LlmGatewayMode::InWorld {
            env.insert(
                WORLD_PROJECT_DIR_OVERRIDE_ENV.to_string(),
                MACOS_STAGED_WORKSPACE_CURRENT.to_string(),
            );
        }

        let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
        let mut identity_tuple =
            derive_gateway_identity_tuple(&agent_id, &effective_policy, &selected_backend)?;
        let integrated_auth = resolve_integrated_auth_payload(
            &effective_config,
            &effective_policy,
            &backend_entry,
            &committed_codex_auth_state_path,
        )?;
        identity_tuple.auth_authority = derive_gateway_auth_authority(integrated_auth.as_ref());
        enforce_identity_constraint(
            "llm.constraints.auth_authorities",
            "auth authority",
            identity_tuple.auth_authority.as_deref(),
            &effective_policy.llm_constraints_auth_authorities,
        )?;
        identity_tuple
            .validate()
            .map_err(gateway_invalid_integration_error)?;
        let identity_tuple = Some(identity_tuple);
        let placement_posture = Some(derive_gateway_placement_posture(&effective_config)?);

        let request = GatewayLifecycleRequestV1 {
            profile: None,
            cwd: Some(cwd.display().to_string()),
            env: Some(env),
            agent_id,
            policy_snapshot: network_policy.snapshot,
            world_network: Some(world_network),
            integrated_auth,
            identity_tuple,
            placement_posture,
        };
        request
            .validate_identity_contract()
            .map_err(gateway_invalid_integration_error)?;

        Ok(GatewayLifecycleRequestContext {
            identity_tuple: request.identity_tuple.clone(),
            placement_posture: request.placement_posture.clone(),
            world_enabled: effective_config.world.enabled,
            #[cfg(test)]
            codex_auth_state_path: committed_codex_auth_state_path,
            request,
        })
    }
}

fn derive_gateway_identity_tuple(
    agent_id: &str,
    effective_policy: &substrate_broker::Policy,
    selected_backend: &str,
) -> anyhow::Result<IdentityTuple> {
    let protocol = match selected_backend {
        backend if is_cli_codex_backend(backend) || backend == API_OPENAI_BACKEND => {
            "openai.responses"
        }
        CLI_CLAUDE_CODE_HOST_BACKEND | CLI_CLAUDE_CODE_WORLD_BACKEND | API_ANTHROPIC_BACKEND => {
            "anthropic.messages"
        }
        other => {
            return Err(gateway_invalid_integration_error(format!(
                "unsupported backend '{}' for gateway identity tuple publication",
                other
            )));
        }
    };
    let provider = match selected_backend {
        backend if is_cli_codex_backend(backend) || backend == API_OPENAI_BACKEND => {
            Some("openai".to_string())
        }
        CLI_CLAUDE_CODE_HOST_BACKEND | CLI_CLAUDE_CODE_WORLD_BACKEND | API_ANTHROPIC_BACKEND => {
            Some("anthropic".to_string())
        }
        _ => None,
    };
    let tuple = IdentityTuple {
        client: resolve_originating_client(agent_id),
        router: SUBSTRATE_GATEWAY_ROUTER.to_string(),
        protocol: protocol.to_string(),
        provider,
        auth_authority: None,
    };

    enforce_identity_constraint(
        "llm.constraints.routers",
        "routing authority",
        Some(tuple.router.as_str()),
        &effective_policy.llm_constraints_routers,
    )?;
    enforce_identity_constraint(
        "llm.constraints.protocols",
        "protocol",
        Some(tuple.protocol.as_str()),
        &effective_policy.llm_constraints_protocols,
    )?;
    enforce_identity_constraint(
        "llm.constraints.providers",
        "provider",
        tuple.provider.as_deref(),
        &effective_policy.llm_constraints_providers,
    )?;
    Ok(tuple)
}

fn derive_gateway_auth_authority(
    integrated_auth: Option<&GatewayIntegratedAuthPayloadV1>,
) -> Option<String> {
    integrated_auth.and_then(|auth| {
        if auth.cli_codex.is_some() {
            Some("codex_subscription".to_string())
        } else if let Some(api_env) = auth.api_env.as_ref() {
            if api_env.env.contains_key(OPENAI_API_KEY_ENV) {
                Some("openai_api_key".to_string())
            } else if api_env.env.contains_key(ANTHROPIC_API_KEY_ENV) {
                Some("anthropic_api_key".to_string())
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn is_cli_codex_backend(backend_id: &str) -> bool {
    matches!(backend_id, CLI_CODEX_HOST_BACKEND | CLI_CODEX_WORLD_BACKEND)
}

fn derive_gateway_placement_posture(
    effective_config: &config_model::SubstrateConfig,
) -> anyhow::Result<PlacementPosture> {
    let posture = PlacementPosture {
        execution: match effective_config.llm.gateway.mode {
            LlmGatewayMode::InWorld => PlacementExecution::InWorld,
            LlmGatewayMode::HostOnly => PlacementExecution::HostOnly,
        },
        host_to_world_bridge: None,
    };
    posture
        .validate()
        .map_err(gateway_invalid_integration_error)?;
    Ok(posture)
}

fn resolve_originating_client(agent_id: &str) -> String {
    normalize_identity_tuple_client_id(agent_id).unwrap_or_else(|| "human".to_string())
}

fn enforce_identity_constraint(
    policy_key: &str,
    label: &str,
    value: Option<&str>,
    allowed: &[String],
) -> anyhow::Result<()> {
    if allowed.is_empty() {
        return Ok(());
    }

    let Some(value) = value else {
        return Err(gateway_policy_blocked_error(format!(
            "effective gateway {label} is unresolved while {policy_key} is constrained"
        )));
    };

    if allowed.iter().any(|candidate| candidate == value) {
        Ok(())
    } else {
        Err(gateway_policy_blocked_error(format!(
            "effective gateway {label} '{}' is not allowlisted by {}",
            value, policy_key
        )))
    }
}

fn augment_gateway_response(
    mut response: GatewayLifecycleResponseV1,
    request_context: &GatewayLifecycleRequestContext,
) -> anyhow::Result<GatewayLifecycleResponseV1> {
    if response.identity_tuple.is_none() {
        response.identity_tuple = request_context.identity_tuple.clone();
    }
    if response.placement_posture.is_none() {
        response.placement_posture = request_context.placement_posture.clone();
    }
    validate_gateway_response(response)
}

fn print_status_identity_metadata(response: &GatewayLifecycleResponseV1) {
    print_status_identity_metadata_impl(response, false);
}

fn print_status_identity_metadata_to_stderr(response: &GatewayLifecycleResponseV1) {
    print_status_identity_metadata_impl(response, true);
}

fn print_status_identity_metadata_impl(response: &GatewayLifecycleResponseV1, stderr: bool) {
    let emit = |line: &str, stderr: bool| {
        if stderr {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
    };

    if let Some(identity_tuple) = response.identity_tuple.as_ref() {
        emit(
            &format!("originating client: {}", identity_tuple.client),
            stderr,
        );
        emit(
            &format!("routing authority: {}", identity_tuple.router),
            stderr,
        );
        if let Some(provider) = identity_tuple.provider.as_deref() {
            emit(&format!("fulfillment provider: {provider}"), stderr);
        }
        if let Some(auth_authority) = identity_tuple.auth_authority.as_deref() {
            emit(&format!("auth authority: {auth_authority}"), stderr);
        }
        emit(&format!("protocol: {}", identity_tuple.protocol), stderr);
    }

    if let Some(placement_posture) = response.placement_posture.as_ref() {
        let execution = match placement_posture.execution {
            PlacementExecution::InWorld => "in_world",
            PlacementExecution::HostOnly => "host_only",
        };
        emit(&format!("deployment posture: {execution}"), stderr);
        if placement_posture.host_to_world_bridge == Some(true) {
            emit("bridge transport: host_to_world_bridge", stderr);
        }
    }
}

fn validate_gateway_backend_selection(
    cwd: &std::path::Path,
    effective_policy: &substrate_broker::Policy,
    selected_backend: &str,
    bootstrap_home: &crate::execution::agent_runtime::OpenedBootstrapHomeV1<'_>,
) -> anyhow::Result<agent_inventory::AgentInventoryEntryV1> {
    if let Some(reason) = retired_exact_backend_selector_guidance(selected_backend) {
        return Err(gateway_invalid_integration_error(reason));
    }

    let entry = agent_inventory::resolve_gateway_backend_inventory_entry_for_bootstrap_home(
        cwd,
        selected_backend,
        effective_policy,
        bootstrap_home,
    )
    .map_err(|err| gateway_invalid_integration_error(err.to_string()))?;
    ensure_backend_allowed(
        &effective_policy.llm_allowed_backends,
        "llm.allowed_backends",
        selected_backend,
    )?;
    Ok(entry)
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

/// Resolve fresh integrated auth for the E3 preparation endpoint.
#[cfg(target_os = "linux")]
#[allow(dead_code)] // The retained dispatch caller is admitted separately in E3-F.
pub(crate) fn resolve_e3_integrated_auth_payload(
    effective_config: &config_model::SubstrateConfig,
    effective_policy: &substrate_broker::Policy,
    selected: &config_projection::LogicalAgentConfigProjectionV1,
    account_home: &Path,
) -> anyhow::Result<GatewayIntegratedAuthPayloadV1> {
    if !effective_config.llm.gateway.enabled
        || effective_config.llm.gateway.mode != LlmGatewayMode::InWorld
        || effective_config.llm.routing.default_backend != CLI_CODEX_WORLD_BACKEND
        || selected.backend_id != effective_config.llm.routing.default_backend
        || selected.kind != "cli"
        || selected.runtime_family != "codex"
        || selected.placement != "world"
        || selected.execution_scope != "world"
    {
        anyhow::bail!("UnsupportedConfiguration");
    }
    let payload = GatewayIntegratedAuthPayloadV1 {
        backend_id: selected.backend_id.clone(),
        cli_codex: Some(
            resolve_cli_codex_integrated_auth(
                effective_policy,
                &selected.backend_id,
                &codex_auth_state_path(account_home),
            )
            .map_err(|_| anyhow::anyhow!("E3 integrated authentication unavailable"))?,
        ),
        api_env: None,
    };
    payload
        .validate()
        .map_err(|_| anyhow::anyhow!("Malformed E3 integrated authentication"))?;
    Ok(payload)
}

fn resolve_integrated_auth_payload(
    effective_config: &config_model::SubstrateConfig,
    effective_policy: &substrate_broker::Policy,
    backend_entry: &agent_inventory::AgentInventoryEntryV1,
    codex_auth_state_path: &Path,
) -> anyhow::Result<Option<GatewayIntegratedAuthPayloadV1>> {
    if !effective_config.llm.gateway.enabled
        || effective_config.llm.gateway.mode != LlmGatewayMode::InWorld
    {
        return Ok(None);
    }

    let selected_backend = effective_config.llm.routing.default_backend.trim();
    match backend_entry.file.config.kind {
        agent_inventory::AgentConfigKind::Cli if is_cli_codex_backend(selected_backend) => {
            Ok(Some(GatewayIntegratedAuthPayloadV1 {
                backend_id: selected_backend.to_string(),
                cli_codex: Some(resolve_cli_codex_integrated_auth(
                    effective_policy,
                    selected_backend,
                    codex_auth_state_path,
                )?),
                api_env: None,
            }))
        }
        agent_inventory::AgentConfigKind::Cli
            if matches!(
                selected_backend,
                CLI_CLAUDE_CODE_HOST_BACKEND | CLI_CLAUDE_CODE_WORLD_BACKEND
            ) =>
        {
            resolve_claude_code_integrated_auth(selected_backend, effective_policy)
        }
        agent_inventory::AgentConfigKind::Cli => Ok(None),
        agent_inventory::AgentConfigKind::Api => {
            resolve_api_env_integrated_auth(selected_backend, backend_entry, effective_policy)
        }
    }
}

fn resolve_claude_code_integrated_auth(
    selected_backend: &str,
    effective_policy: &substrate_broker::Policy,
) -> anyhow::Result<Option<GatewayIntegratedAuthPayloadV1>> {
    let Some(api_key) = read_trimmed_env(ANTHROPIC_API_KEY_ENV)
        .map_err(|err| gateway_invalid_integration_error(err.to_string()))?
    else {
        return Ok(None);
    };

    ensure_env_name_allowed(effective_policy, ANTHROPIC_API_KEY_ENV)?;

    Ok(Some(GatewayIntegratedAuthPayloadV1 {
        backend_id: selected_backend.to_string(),
        cli_codex: None,
        api_env: Some(GatewayApiEnvIntegratedAuthV1 {
            env: HashMap::from([(ANTHROPIC_API_KEY_ENV.to_string(), api_key)]),
        }),
    }))
}

fn resolve_cli_codex_integrated_auth(
    effective_policy: &substrate_broker::Policy,
    backend_id: &str,
    auth_path: &Path,
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
        backend_id,
    )?;

    let content = fs::read_to_string(auth_path).map_err(|err| {
        gateway_invalid_integration_error(format!(
            "failed to read committed Codex auth state: {err}"
        ))
    })?;
    let json: Value = serde_json::from_str(&content).map_err(|err| {
        gateway_invalid_integration_error(format!(
            "failed to parse committed Codex auth state: {err}"
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

fn resolve_api_env_integrated_auth(
    selected_backend: &str,
    backend_entry: &agent_inventory::AgentInventoryEntryV1,
    effective_policy: &substrate_broker::Policy,
) -> anyhow::Result<Option<GatewayIntegratedAuthPayloadV1>> {
    let Some(api_config) = backend_entry.file.config.api.as_ref() else {
        return Ok(None);
    };

    let mut present_env = HashMap::new();
    let mut missing_env = Vec::new();
    for env_name in &api_config.auth.env {
        match read_trimmed_env(env_name)
            .map_err(|err| gateway_invalid_integration_error(err.to_string()))?
        {
            Some(value) => {
                ensure_env_name_allowed(effective_policy, env_name)?;
                present_env.insert(env_name.clone(), value);
            }
            None => missing_env.push(env_name.clone()),
        }
    }

    if present_env.is_empty() {
        return Ok(None);
    }

    if !missing_env.is_empty() {
        return Err(gateway_invalid_integration_error(format!(
            "integrated API env auth for {selected_backend} is incomplete: missing {}",
            missing_env.join(", ")
        )));
    }

    Ok(Some(GatewayIntegratedAuthPayloadV1 {
        backend_id: selected_backend.to_string(),
        cli_codex: None,
        api_env: Some(GatewayApiEnvIntegratedAuthV1 { env: present_env }),
    }))
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

fn codex_auth_state_path(account_home: &Path) -> PathBuf {
    account_home.join(".codex").join("auth.json")
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

#[cfg(target_os = "linux")]
fn build_gateway_client() -> anyhow::Result<AgentClient> {
    AgentClient::unix_socket(std::path::PathBuf::from(DEFAULT_WORLD_SOCKET_PATH))
}

#[cfg(target_os = "windows")]
fn build_gateway_client() -> anyhow::Result<AgentClient> {
    pw::windows::build_agent_client()
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn build_gateway_client() -> anyhow::Result<AgentClient> {
    anyhow::bail!("gateway runtime client is unsupported on this platform")
}

fn gateway_invalid_integration_error(message: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("gateway_invalid_integration: {}", message.into())
}

fn gateway_policy_blocked_error(message: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("gateway_policy_blocked: {}", message.into())
}

fn world_routing_disabled(request_context: &GatewayLifecycleRequestContext) -> bool {
    !request_context.world_enabled
}

fn synthesized_unavailable_response(
    request_context: &GatewayLifecycleRequestContext,
) -> GatewayLifecycleResponseV1 {
    GatewayLifecycleResponseV1 {
        status: GatewayStatusV1::Unavailable,
        client_wiring: None,
        identity_tuple: request_context.identity_tuple.clone(),
        placement_posture: request_context.placement_posture.clone(),
    }
}

fn validate_gateway_response(
    response: GatewayLifecycleResponseV1,
) -> anyhow::Result<GatewayLifecycleResponseV1> {
    response
        .validate_identity_contract()
        .map_err(gateway_invalid_integration_error)?;
    Ok(response)
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
            || msg.contains("no forwarding transport available")
            || msg.contains("lima ssh config not found")
            || msg.contains("limactl not found")
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

fn emit_unavailable(command: &str, response: &GatewayLifecycleResponseV1) -> i32 {
    eprintln!("{command}: unavailable (required gateway/world component unavailable)");
    print_status_identity_metadata_to_stderr(response);
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
    use crate::execution::world_env_guard;
    use serial_test::serial;
    use std::{
        fs,
        io::{Read, Write},
        path::{Path, PathBuf},
    };

    struct EnvVarGuard {
        key: String,
        prev: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &str, value: Option<&std::ffi::OsStr>) -> Self {
            let prev = std::env::var_os(key);
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
            Self {
                key: key.to_string(),
                prev,
            }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.prev {
                Some(value) => std::env::set_var(&self.key, value),
                None => std::env::remove_var(&self.key),
            }
        }
    }

    fn with_env_var<T>(key: &str, value: Option<&std::ffi::OsStr>, f: impl FnOnce() -> T) -> T {
        let _guard = world_env_guard();
        let _env_guard = EnvVarGuard::set(key, value);
        f()
    }

    struct CurrentDirGuard {
        _process_cwd: crate::execution::ProcessCwdTestGuard,
    }

    impl CurrentDirGuard {
        fn set(path: &Path) -> Self {
            Self {
                _process_cwd: crate::execution::ProcessCwdTestGuard::change_to(path),
            }
        }
    }

    fn with_current_dir<T>(path: &Path, f: impl FnOnce() -> T) -> T {
        let _guard = CurrentDirGuard::set(path);
        f()
    }

    fn write_gateway_test_config(substrate_home: &Path) {
        use std::os::unix::fs::PermissionsExt;

        fs::create_dir_all(substrate_home).expect("create substrate home");
        fs::set_permissions(substrate_home, fs::Permissions::from_mode(0o700))
            .expect("secure substrate home");
        fs::write(
            substrate_home.join("config.yaml"),
            r#"world:
  enabled: true
policy:
  mode: observe
llm:
  enabled: true
  gateway:
    enabled: true
    mode: in_world
  routing:
    default_backend: cli:codex-world
agents:
  enabled: true
"#,
        )
        .expect("write config");
        fs::set_permissions(
            substrate_home.join("config.yaml"),
            fs::Permissions::from_mode(0o600),
        )
        .expect("secure config");
        fs::write(
            substrate_home.join("policy.yaml"),
            format!(
                r#"llm:
  allowed_backends:
    - cli:codex-world
  secrets:
    env_allowed:
      - {CODEX_ACCOUNT_ID_ENV}
      - {CODEX_ACCESS_TOKEN_ENV}
"#
            ),
        )
        .expect("write policy");
        fs::set_permissions(
            substrate_home.join("policy.yaml"),
            fs::Permissions::from_mode(0o600),
        )
        .expect("secure policy");
        let agents_dir = substrate_home.join("agents");
        fs::create_dir_all(&agents_dir).expect("create agents dir");
        fs::set_permissions(&agents_dir, fs::Permissions::from_mode(0o700))
            .expect("secure agents dir");
        let manifest_src =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../config/agents/codex.yaml");
        fs::copy(manifest_src, agents_dir.join("codex.yaml")).expect("copy codex manifest");
        fs::set_permissions(
            agents_dir.join("codex.yaml"),
            fs::Permissions::from_mode(0o600),
        )
        .expect("secure codex manifest");
    }

    fn test_authenticated_macos_mapping(
        selected_prefix: &Path,
        host_socket: &Path,
    ) -> PlatformBootstrapMappingV1 {
        let (principal, _) = crate::execution::install_bootstrap::current_unix_principal_and_home()
            .expect("current Unix principal");
        let transport_api_types::PlatformPrincipalV1::Unix { account, uid } = principal else {
            panic!("expected Unix principal")
        };
        let carrier = transport_api_types::InstallBootstrapContextCarrierV1::from_context(
            transport_api_types::InstallBootstrapContextV1::new_unix(
                selected_prefix.to_str().expect("UTF-8 selected prefix"),
                &account,
                uid,
            )
            .expect("install context"),
        )
        .expect("install carrier");

        transport_api_types::PlatformBootstrapMappingV1::new_lima(
            &carrier,
            "substrate-test",
            "0123456789abcdef0123456789abcdef",
            "/Users/test/.lima",
            "/home/substrate/.substrate",
            "substrate",
            1000,
            host_socket.to_str().expect("UTF-8 host socket"),
            "/run/substrate.sock",
        )
        .expect("typed Lima mapping")
    }

    #[test]
    #[serial]
    fn macos_gateway_client_endpoint_stays_unix_when_host_socket_exists() {
        let temp = tempfile::tempdir().expect("tempdir");
        let selected_prefix = temp.path().join("selected-prefix");
        let socket = selected_prefix.join("sock/agent.sock");
        std::fs::create_dir_all(socket.parent().expect("sock parent")).expect("create sock dir");
        let listener = std::os::unix::net::UnixListener::bind(&socket).expect("bind socket");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept capabilities probe");
            let mut request = [0_u8; 256];
            let count = stream.read(&mut request).expect("read capabilities probe");
            assert!(String::from_utf8_lossy(&request[..count]).contains("GET /v1/capabilities"));
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                .expect("write capabilities response");
        });

        with_test_authenticated_macos_mapping(
            test_authenticated_macos_mapping(&selected_prefix, &socket),
            || match resolve_macos_gateway_client_endpoint() {
                MacosGatewayClientEndpoint::Unix(path) => assert_eq!(path, socket),
                MacosGatewayClientEndpoint::Tcp { .. } => {
                    panic!("expected unix endpoint when host socket exists")
                }
            },
        );
        server.join().expect("join socket server");
    }

    #[test]
    #[serial]
    fn macos_gateway_client_endpoint_does_not_fall_back_to_tcp_when_host_socket_missing() {
        let temp = tempfile::tempdir().expect("tempdir");
        let selected_prefix = temp.path().join("selected-prefix");
        let socket = selected_prefix.join("sock/agent.sock");

        with_test_authenticated_macos_mapping(
            test_authenticated_macos_mapping(&selected_prefix, &socket),
            || match resolve_macos_gateway_client_endpoint() {
                MacosGatewayClientEndpoint::Unix(path) => assert_eq!(path, socket),
                MacosGatewayClientEndpoint::Tcp { .. } => {
                    panic!("expected typed unix endpoint when socket is missing")
                }
            },
        );
    }

    #[test]
    #[serial]
    fn macos_gateway_client_endpoint_does_not_fall_back_to_tcp_when_explicit_socket_is_missing() {
        let temp = tempfile::tempdir().expect("tempdir");
        let substrate_home = temp.path().join("isolated-substrate-home");
        let selected_prefix = temp.path().join("selected-prefix");
        let socket = selected_prefix.join("sock/agent.sock");

        with_env_var("SUBSTRATE_HOME", Some(substrate_home.as_os_str()), || {
            with_test_authenticated_macos_mapping(
                test_authenticated_macos_mapping(&selected_prefix, &socket),
                || match resolve_macos_gateway_client_endpoint() {
                    MacosGatewayClientEndpoint::Unix(path) => assert_eq!(path, socket),
                    MacosGatewayClientEndpoint::Tcp { .. } => {
                        panic!("expected typed unix endpoint when explicit socket is missing")
                    }
                },
            )
        });
    }

    #[test]
    #[serial]
    fn macos_gateway_client_endpoint_does_not_fall_back_to_tcp_when_host_socket_is_stale() {
        let temp = tempfile::tempdir().expect("tempdir");
        let selected_prefix = temp.path().join("selected-prefix");
        let socket = selected_prefix.join("sock/agent.sock");
        std::fs::create_dir_all(socket.parent().expect("sock parent")).expect("create sock dir");
        std::fs::write(&socket, "").expect("create placeholder socket path");

        with_test_authenticated_macos_mapping(
            test_authenticated_macos_mapping(&selected_prefix, &socket),
            || match resolve_macos_gateway_client_endpoint() {
                MacosGatewayClientEndpoint::Unix(path) => assert_eq!(path, socket),
                MacosGatewayClientEndpoint::Tcp { .. } => {
                    panic!("expected typed unix endpoint when socket is stale")
                }
            },
        );
    }

    #[test]
    #[serial]
    fn macos_default_world_socket_path_uses_supplied_authenticated_mapping() {
        let temp = tempfile::tempdir().expect("tempdir");
        let selected_prefix = temp.path().join("selected-prefix");
        let socket = selected_prefix.join("sock/agent.sock");

        with_test_authenticated_macos_mapping(
            test_authenticated_macos_mapping(&selected_prefix, &socket),
            || assert_eq!(macos_default_world_socket_path(), socket),
        );
    }

    #[test]
    #[serial]
    fn build_gateway_request_context_uses_staged_workspace_override_for_macos_in_world_gateway() {
        let temp = tempfile::tempdir().expect("tempdir");
        let home = temp.path().join("home");
        let substrate_home = home.join(".substrate");
        let workspace_root = temp.path().join("workspace");
        let nested_cwd = workspace_root.join("nested");
        fs::create_dir_all(&nested_cwd).expect("create nested cwd");
        write_gateway_test_config(&substrate_home);
        let (principal, _) = crate::execution::install_bootstrap::current_unix_principal_and_home()
            .expect("current Unix principal");
        let transport_api_types::PlatformPrincipalV1::Unix { account, uid } = principal else {
            panic!("expected Unix principal")
        };
        let install_context = transport_api_types::InstallBootstrapContextCarrierV1::from_context(
            transport_api_types::InstallBootstrapContextV1::new_unix(
                substrate_home.to_str().expect("UTF-8 substrate home"),
                &account,
                uid,
            )
            .expect("install context"),
        )
        .expect("install carrier");

        with_env_var("HOME", Some(home.as_os_str()), || {
            with_env_var("SUBSTRATE_HOME", Some(substrate_home.as_os_str()), || {
                with_env_var(
                    CODEX_ACCOUNT_ID_ENV,
                    Some(std::ffi::OsStr::new("acct_test")),
                    || {
                        with_env_var(
                            CODEX_ACCESS_TOKEN_ENV,
                            Some(std::ffi::OsStr::new("token_test")),
                            || {
                                with_current_dir(&nested_cwd, || {
                                    let context = build_gateway_request_context(
                                        false,
                                        false,
                                        &nested_cwd,
                                        &install_context,
                                    )
                                    .expect("gateway request context");
                                    let expected_cwd = std::env::current_dir()
                                        .expect("current dir during request build")
                                        .display()
                                        .to_string();
                                    assert_eq!(
                                        context
                                            .request
                                            .env
                                            .as_ref()
                                            .and_then(|env| env.get(WORLD_PROJECT_DIR_OVERRIDE_ENV))
                                            .map(String::as_str),
                                        Some(MACOS_STAGED_WORKSPACE_CURRENT)
                                    );
                                    assert_eq!(
                                        context.request.cwd.as_deref(),
                                        Some(expected_cwd.as_str())
                                    );
                                })
                            },
                        )
                    },
                )
            })
        });
    }
}

#[cfg(test)]
mod classification_tests {
    #[cfg(target_os = "linux")]
    mod e3f_auth_tests {
        use super::super::*;
        use super::AmbientSelectionGuard;
        use crate::execution::agent_runtime::host_session_authority::{
            facade::HostSessionAuthority,
            trusted_fs::{
                HeldE3AgentInventoryRootV1, HeldE3AgentInventorySourceV1, TrustedAuthorityRoot,
                TrustedWorkspaceRoot,
            },
        };
        use config_projection::{
            CanonicalDirectoryV1, E3EffectiveConfigInputV1, EffectiveSubstrateConfigSourceV1,
            LogicalAgentConfigProjectionV1,
        };
        use serial_test::serial;
        use std::os::{
            fd::AsFd,
            unix::{ffi::OsStrExt, fs::PermissionsExt},
        };

        // Resolve actual V3 bytes through the existing strict inventory resolver. The
        // known source is fixture input, not a new production inventory selector.
        fn with_selection(
            test: impl FnOnce(&config_model::SubstrateConfig, &LogicalAgentConfigProjectionV1, &Path),
        ) {
            let _env = crate::execution::AuthorityEnvTestGuard::preserve();
            let home = tempfile::tempdir().unwrap();
            let workspace = tempfile::tempdir().unwrap();
            fs::set_permissions(home.path(), fs::Permissions::from_mode(0o700)).unwrap();
            fs::set_permissions(workspace.path(), fs::Permissions::from_mode(0o700)).unwrap();
            fs::create_dir(home.path().join("agents")).unwrap();
            fs::set_permissions(
                home.path().join("agents"),
                fs::Permissions::from_mode(0o700),
            )
            .unwrap();
            fs::write(home.path().join("policy.yaml"), "{}\n").unwrap();
            fs::set_permissions(
                home.path().join("policy.yaml"),
                fs::Permissions::from_mode(0o600),
            )
            .unwrap();
            fs::write(
                home.path().join("agents/codex.yaml"),
                r#"version: 3
id: codex
config:
  enabled: true
  kind: cli
  protocol: substrate.agent.session
  placements:
    world:
      enabled: true
      cli:
        binary: /var/lib/substrate/world-deps/codex-runtime/bin/codex
        mode: persistent
        runtime_family: codex
      api: null
      capabilities:
        session_start: true
        session_resume: true
        session_fork: true
        session_stop: true
        status_snapshot: true
        event_stream: true
        llm: true
        mcp_client: false
      runtime_projection:
        model: codex
        mcp_servers: []
        features: []
"#,
            )
            .unwrap();
            fs::set_permissions(
                home.path().join("agents/codex.yaml"),
                fs::Permissions::from_mode(0o600),
            )
            .unwrap();
            let root = TrustedAuthorityRoot::open(home.path()).unwrap();
            let authority = HostSessionAuthority::open(home.path()).unwrap();
            let workspace_identity = CanonicalDirectoryV1::capture_linux_from_fd(
                fs::File::open(workspace.path()).unwrap().as_fd(),
            )
            .unwrap();
            let config_projection::DirectoryPhysicalIdentityV1::Linux { device_id, inode } =
                workspace_identity.physical_identity;
            let trusted_workspace = TrustedWorkspaceRoot::open_exact(
                &crate::execution::agent_runtime::host_session_authority::schema::CanonicalDirectoryV1 {
                    physical_path: workspace_identity.physical_path.clone(),
                    physical_identity: crate::execution::agent_runtime::host_session_authority::schema::DirectoryPhysicalIdentityV1::Linux { device_id, inode },
                },
            ).unwrap();
            let global = HeldE3AgentInventoryRootV1::from_global(&root).unwrap();
            let local = HeldE3AgentInventoryRootV1::from_workspace(&trusted_workspace).unwrap();
            let selected =
                HeldE3AgentInventorySourceV1::open(&global, "agents/codex.yaml").unwrap();
            let mut config = config_model::SubstrateConfig::default();
            config.llm.gateway.enabled = true;
            config.llm.gateway.mode = LlmGatewayMode::InWorld;
            config.llm.routing.default_backend = CLI_CODEX_WORLD_BACKEND.into();
            let source = EffectiveSubstrateConfigSourceV1 {
                schema_version: 1,
                authority_store_id: "as_018f0f3a-9b2c-7def-8abc-0123456789ab".into(),
                accepted_home: CanonicalDirectoryV1::capture_linux_from_fd(
                    fs::File::open(home.path()).unwrap().as_fd(),
                )
                .unwrap(),
                workspace_root: workspace_identity,
                values: E3EffectiveConfigInputV1 {
                    llm_enabled: true,
                    agents_enabled: true,
                    world_enabled: true,
                    default_execution_scope: "world".into(),
                    default_cli_mode: "persistent".into(),
                    managed_gateway_enabled: config.llm.gateway.enabled,
                    managed_gateway_mode: "in_world".into(),
                    default_backend_id: config.llm.routing.default_backend.clone(),
                },
                ordered_explain_origins: vec![],
                source_revision: "1".into(),
                source_hash: "a".repeat(64),
            };
            let logical = agent_inventory::resolve_e3_selected_inventory_projection_v1(
                &authority.bootstrap_home(),
                &global,
                &local,
                &source,
                selected.source_material(),
            )
            .unwrap();
            test(&config, &logical, home.path());
            selected.revalidate().unwrap();
            global.revalidate().unwrap();
            local.revalidate().unwrap();
        }

        fn policy() -> substrate_broker::Policy {
            substrate_broker::Policy {
                llm_secrets_env_allowed: vec![
                    CODEX_ACCESS_TOKEN_ENV.into(),
                    CODEX_ACCOUNT_ID_ENV.into(),
                ],
                agents_host_credentials_read_allowed_backends: vec![CLI_CODEX_WORLD_BACKEND.into()],
                ..Default::default()
            }
        }

        fn write_auth(home: &Path, bytes: &str) {
            fs::create_dir_all(home.join(".codex")).unwrap();
            fs::write(codex_auth_state_path(home), bytes).unwrap();
        }

        #[test]
        #[serial]
        fn test_e3f_auth_selected_v3_and_environment_precedence() {
            with_selection(|config, selected, home| {
                write_auth(
                    home,
                    r#"{"access_token":"synthetic-file-token","account_id":"synthetic-file-account"}"#,
                );
                let _env = AmbientSelectionGuard::set(&[
                    (
                        CODEX_ACCESS_TOKEN_ENV,
                        Some(std::ffi::OsStr::new(" synthetic-env-token ")),
                    ),
                    (
                        CODEX_ACCOUNT_ID_ENV,
                        Some(std::ffi::OsStr::new(" synthetic-env-account ")),
                    ),
                ]);
                let payload =
                    resolve_e3_integrated_auth_payload(config, &policy(), selected, home).unwrap();
                assert_eq!(payload.backend_id, CLI_CODEX_WORLD_BACKEND);
                assert!(payload.api_env.is_none());
                let auth = payload.cli_codex.unwrap();
                assert!(auth.access_token == "synthetic-env-token");
                assert!(auth.account_id.as_deref() == Some("synthetic-env-account"));
            });
        }

        #[test]
        #[serial]
        fn test_e3f_auth_file_fallback_and_optional_account() {
            with_selection(|config, selected, home| {
                let _env = AmbientSelectionGuard::set(&[
                    (CODEX_ACCESS_TOKEN_ENV, None),
                    (CODEX_ACCOUNT_ID_ENV, None),
                ]);
                for account in [None, Some("synthetic-file-account")] {
                    write_auth(home, &serde_json::json!({"access_token":"synthetic-file-token", "account_id":account}).to_string());
                    let payload =
                        resolve_e3_integrated_auth_payload(config, &policy(), selected, home)
                            .unwrap();
                    payload.validate().unwrap();
                    assert_eq!(payload.backend_id, CLI_CODEX_WORLD_BACKEND);
                    assert!(payload.api_env.is_none());
                    let auth = payload.cli_codex.unwrap();
                    assert!(auth.access_token == "synthetic-file-token");
                    assert!(auth.account_id.as_deref() == account);
                }
            });
        }

        #[test]
        #[serial]
        fn test_e3f_auth_selection_rejects_before_credential_lookup() {
            with_selection(|config, selected, home| {
                // An attempted environment lookup would return the sanitized resolver
                // error. Every selection rejection must happen before that lookup.
                let _env = AmbientSelectionGuard::set(&[
                    (
                        CODEX_ACCESS_TOKEN_ENV,
                        Some(std::ffi::OsStr::from_bytes(b"\xff")),
                    ),
                    (CODEX_ACCOUNT_ID_ENV, None),
                ]);
                for case in 0..9 {
                    let mut config = config.clone();
                    let mut selected = selected.clone();
                    match case {
                        0 => selected.backend_id = "cli:other-world".into(),
                        1 => config.llm.routing.default_backend = "cli:other-world".into(),
                        2 => selected.kind = "api".into(),
                        3 => selected.runtime_family = "claude_code".into(),
                        4 => selected.placement = "host".into(),
                        5 => selected.execution_scope = "host".into(),
                        6 => config.llm.gateway.enabled = false,
                        7 => config.llm.gateway.mode = LlmGatewayMode::HostOnly,
                        8 => {
                            config.llm.routing.default_backend = "cli:other-world".into();
                            selected.backend_id = config.llm.routing.default_backend.clone();
                        }
                        _ => unreachable!(),
                    }
                    let error =
                        resolve_e3_integrated_auth_payload(&config, &policy(), &selected, home)
                            .err()
                            .expect("E3 authentication must reject");
                    assert_eq!(
                        format!("{error:#}"),
                        "UnsupportedConfiguration",
                        "case {case}"
                    );
                }
                let error = resolve_e3_integrated_auth_payload(config, &policy(), selected, home)
                    .err()
                    .expect("E3 authentication must reject");
                assert_eq!(
                    format!("{error:#}"),
                    "E3 integrated authentication unavailable"
                );
            });
        }

        #[test]
        #[serial]
        fn test_e3f_auth_policy_denial_never_falls_back() {
            with_selection(|config, selected, home| {
                write_auth(home, r#"{"access_token":"synthetic-file-token"}"#);
                let _env = AmbientSelectionGuard::set(&[
                    (
                        CODEX_ACCESS_TOKEN_ENV,
                        Some(std::ffi::OsStr::new("synthetic-env-token")),
                    ),
                    (
                        CODEX_ACCOUNT_ID_ENV,
                        Some(std::ffi::OsStr::new("synthetic-env-account")),
                    ),
                ]);
                for denied in [CODEX_ACCESS_TOKEN_ENV, CODEX_ACCOUNT_ID_ENV] {
                    let mut denied_policy = policy();
                    denied_policy
                        .llm_secrets_env_allowed
                        .retain(|name| name != denied);
                    let error =
                        resolve_e3_integrated_auth_payload(config, &denied_policy, selected, home)
                            .err()
                            .expect("E3 authentication must reject");
                    assert_eq!(
                        format!("{error:#}"),
                        "E3 integrated authentication unavailable"
                    );
                }
                let _no_env = AmbientSelectionGuard::set(&[
                    (CODEX_ACCESS_TOKEN_ENV, None),
                    (CODEX_ACCOUNT_ID_ENV, None),
                ]);
                let mut denied_policy = policy();
                denied_policy
                    .agents_host_credentials_read_allowed_backends
                    .clear();
                let error =
                    resolve_e3_integrated_auth_payload(config, &denied_policy, selected, home)
                        .err()
                        .expect("E3 authentication must reject");
                assert_eq!(
                    format!("{error:#}"),
                    "E3 integrated authentication unavailable"
                );
            });
        }

        #[test]
        #[serial]
        fn test_e3f_auth_errors_are_sanitized() {
            with_selection(|config, selected, home| {
                let _env = AmbientSelectionGuard::set(&[
                    (CODEX_ACCESS_TOKEN_ENV, None),
                    (CODEX_ACCOUNT_ID_ENV, None),
                ]);
                // Missing file, malformed JSON, and missing/empty token all stay opaque.
                for contents in [
                    None,
                    Some("secret-sentinel-invalid-json"),
                    Some(r#"{"account_id":"synthetic-account"}"#),
                    Some(r#"{"access_token":"  "}"#),
                ] {
                    if let Some(bytes) = contents {
                        write_auth(home, bytes);
                    }
                    let error =
                        resolve_e3_integrated_auth_payload(config, &policy(), selected, home)
                            .err()
                            .expect("E3 authentication must reject");
                    assert_eq!(
                        format!("{error:#}"),
                        "E3 integrated authentication unavailable"
                    );
                }
                let _incomplete = AmbientSelectionGuard::set(&[(
                    CODEX_ACCOUNT_ID_ENV,
                    Some(std::ffi::OsStr::new("synthetic-orphan-account")),
                )]);
                let error = resolve_e3_integrated_auth_payload(config, &policy(), selected, home)
                    .err()
                    .expect("E3 authentication must reject");
                assert_eq!(
                    format!("{error:#}"),
                    "E3 integrated authentication unavailable"
                );
            });
        }
    }

    #[cfg(target_os = "linux")]
    use super::build_gateway_client;
    #[cfg(unix)]
    use super::{build_gateway_request_context, world_routing_disabled};
    use super::{
        codex_auth_state_path, error_is_component_unavailable, resolve_cli_codex_integrated_auth,
        CLI_CODEX_WORLD_BACKEND, CODEX_ACCESS_TOKEN_ENV, CODEX_ACCOUNT_ID_ENV,
    };
    use crate::execution::{AuthorityEnvTestGuard, WorldSocketTestGuard};
    use serial_test::serial;

    fn with_env_var<T>(key: &str, value: Option<&std::ffi::OsStr>, f: impl FnOnce() -> T) -> T {
        if key == "SUBSTRATE_HOME" {
            let _guard = match value {
                Some(value) => AuthorityEnvTestGuard::set_home(value),
                None => AuthorityEnvTestGuard::remove_home(),
            };
            return f();
        }
        if key == "SUBSTRATE_WORLD_SOCKET" {
            let _guard = WorldSocketTestGuard::set_optional(value);
            return f();
        }
        let _guard = AuthorityEnvTestGuard::preserve();
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
        f()
    }

    #[cfg(unix)]
    struct AmbientSelectionGuard {
        previous: Vec<(String, Option<std::ffi::OsString>)>,
        _authority_env: AuthorityEnvTestGuard,
    }

    #[cfg(unix)]
    impl AmbientSelectionGuard {
        fn set(entries: &[(&str, Option<&std::ffi::OsStr>)]) -> Self {
            let authority_env = AuthorityEnvTestGuard::preserve();
            let mut previous = Vec::with_capacity(entries.len());
            for (key, value) in entries {
                previous.push(((*key).to_string(), std::env::var_os(key)));
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
            Self {
                previous,
                _authority_env: authority_env,
            }
        }
    }

    #[cfg(unix)]
    impl Drop for AmbientSelectionGuard {
        fn drop(&mut self) {
            for (key, value) in self.previous.iter().rev() {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }

    #[cfg(unix)]
    struct GatewayCurrentDirGuard {
        _process_cwd: crate::execution::ProcessCwdTestGuard,
    }

    #[cfg(unix)]
    impl GatewayCurrentDirGuard {
        fn set(path: &std::path::Path) -> Self {
            Self {
                _process_cwd: crate::execution::ProcessCwdTestGuard::change_to(path),
            }
        }
    }

    #[test]
    #[serial]
    fn component_unavailable_includes_macos_forwarding_bootstrap_failures() {
        for message in [
            "No forwarding transport available. Run scripts/mac/lima-doctor.sh",
            "Lima SSH config not found at: /Users/test/.lima/substrate/ssh.config",
            "limactl not found. Install Lima with: brew install lima",
        ] {
            let err = anyhow::anyhow!(message);
            assert!(
                error_is_component_unavailable(&err),
                "expected macOS bootstrap error to classify as component unavailable: {message}"
            );
        }
    }

    #[cfg(unix)]
    fn authenticated_gateway_fixture() -> (
        tempfile::TempDir,
        transport_api_types::InstallBootstrapContextCarrierV1,
    ) {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use transport_api_types::{InstallBootstrapContextCarrierV1, InstallBootstrapContextV1};

        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                    .join(".cache")
            });
        fs::create_dir_all(&safe_parent).expect("create safe parent");
        let parent = tempfile::tempdir_in(safe_parent).expect("private fixture parent");
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700))
            .expect("secure fixture parent");
        let selected = parent.path().join("selected");
        let agents = selected.join("agents");
        fs::create_dir_all(&agents).expect("create selected agents");
        fs::set_permissions(&selected, fs::Permissions::from_mode(0o700))
            .expect("secure selected home");
        fs::set_permissions(&agents, fs::Permissions::from_mode(0o700))
            .expect("secure selected agents");
        fs::write(
            selected.join("config.yaml"),
            b"world:\n  enabled: true\n  net:\n    filter: true\nllm:\n  enabled: true\n  gateway:\n    enabled: true\n    mode: in_world\n  routing:\n    default_backend: api:openai\n",
        )
        .expect("write selected config");
        fs::write(
            selected.join("policy.yaml"),
            b"id: selected-policy\nllm:\n  allowed_backends: [api:openai]\nnet_allowed: [selected.example]\n",
        )
        .expect("write selected policy");
        fs::write(
            agents.join("openai.yaml"),
            b"version: 1\nid: openai\nconfig:\n  enabled: true\n  kind: api\n  api:\n    base_url: https://api.openai.com/v1\n    auth:\n      env: [OPENAI_API_KEY]\n  capabilities:\n    llm: true\n",
        )
        .expect("write selected inventory");
        for path in [
            selected.join("config.yaml"),
            selected.join("policy.yaml"),
            agents.join("openai.yaml"),
        ] {
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))
                .expect("secure selected file");
        }

        let (principal, _) = crate::execution::install_bootstrap::current_unix_principal_and_home()
            .expect("current Unix principal");
        let transport_api_types::PlatformPrincipalV1::Unix { account, uid } = principal else {
            panic!("expected Unix principal")
        };
        let context = InstallBootstrapContextV1::new_unix(
            selected.to_str().expect("UTF-8 selected path"),
            &account,
            uid,
        )
        .expect("install context");
        let carrier = InstallBootstrapContextCarrierV1::from_context(context).expect("carrier");
        (parent, carrier)
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn authenticated_gateway_context_uses_a_for_config_policy_inventory_and_network() {
        let _authority_env = AuthorityEnvTestGuard::preserve();
        let (fixture, carrier) = authenticated_gateway_fixture();
        let conflicting = fixture.path().join("conflicting");
        std::fs::create_dir(&conflicting).expect("conflicting home");
        std::fs::write(conflicting.join("config.yaml"), "not: [valid")
            .expect("poison ambient config");

        with_env_var("SUBSTRATE_HOME", Some(conflicting.as_os_str()), || {
            with_env_var("HOME", Some(conflicting.as_os_str()), || {
                with_env_var("CODEX_HOME", Some(conflicting.as_os_str()), || {
                    with_env_var("OPENAI_API_KEY", None, || {
                        let context =
                            build_gateway_request_context(false, false, fixture.path(), &carrier)
                                .expect("authenticated gateway context");
                        assert_eq!(
                            context
                                .request
                                .env
                                .as_ref()
                                .and_then(|env| env.get("SUBSTRATE_LLM_DEFAULT_BACKEND"))
                                .map(String::as_str),
                            Some("api:openai")
                        );
                        assert_eq!(
                            context.request.policy_snapshot.net_allowed,
                            vec!["selected.example".to_string()]
                        );
                        let network = context
                            .request
                            .world_network
                            .as_ref()
                            .expect("world network projection");
                        assert!(network.isolate_network);
                        assert_eq!(
                            network.allowed_domains,
                            vec!["selected.example".to_string()]
                        );
                        assert!(!world_routing_disabled(&context));
                    })
                })
            })
        });
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn authenticated_gateway_projection_ignores_named_ambient_roots_and_uses_account_db_codex_home()
    {
        let _authority_env = AuthorityEnvTestGuard::preserve();
        let (fixture, carrier) = authenticated_gateway_fixture();
        let conflicting = fixture.path().join("conflicting");
        let ambient_codex = conflicting.join(".codex");
        let ambient_xdg = conflicting.join("xdg");
        let ambient_workspace = conflicting.join(".substrate");
        std::fs::create_dir_all(&ambient_codex).expect("create ambient Codex home");
        std::fs::create_dir_all(&ambient_xdg).expect("create ambient XDG root");
        std::fs::create_dir_all(ambient_workspace.join("agents"))
            .expect("create ambient workspace roots");
        let ambient_secret = "ambient-credential-payload-must-not-project";
        std::fs::write(
            ambient_codex.join("auth.json"),
            format!(r#"{{"access_token":"{ambient_secret}"}}"#),
        )
        .expect("write ambient auth state");
        std::fs::write(
            conflicting.join("config.toml"),
            format!("api_key = \"{ambient_secret}\"\n"),
        )
        .expect("write ambient native config");
        std::fs::write(ambient_workspace.join("workspace.yaml"), "not: [valid")
            .expect("poison ambient workspace config");
        std::fs::write(ambient_workspace.join("policy.yaml"), "not: [valid")
            .expect("poison ambient workspace policy");
        std::fs::write(ambient_workspace.join("agents/openai.yaml"), "not: [valid")
            .expect("poison ambient workspace inventory");

        let expected_account_home =
            crate::execution::install_bootstrap::unix_account_home_for_principal(
                &carrier.context.intended_host_principal,
            )
            .expect("committed principal account home");
        let expected_codex_auth = expected_account_home.join(".codex/auth.json");
        let _cwd = GatewayCurrentDirGuard::set(&conflicting);
        let _ambient = AmbientSelectionGuard::set(&[
            ("HOME", Some(conflicting.as_os_str())),
            ("USERPROFILE", Some(conflicting.as_os_str())),
            ("SUBSTRATE_HOME", Some(conflicting.as_os_str())),
            ("SUBSTRATE_ROOT", Some(conflicting.as_os_str())),
            ("CODEX_HOME", Some(ambient_codex.as_os_str())),
            ("XDG_CONFIG_HOME", Some(ambient_xdg.as_os_str())),
            ("XDG_DATA_HOME", Some(ambient_xdg.as_os_str())),
            ("XDG_STATE_HOME", Some(ambient_xdg.as_os_str())),
            ("OPENAI_API_KEY", None),
        ]);

        let explicit_launch_cwd = fixture.path().to_path_buf();
        let context = build_gateway_request_context(false, false, &explicit_launch_cwd, &carrier)
            .expect("authenticated gateway context");
        assert_eq!(context.codex_auth_state_path, expected_codex_auth);
        assert_eq!(
            context
                .request
                .env
                .as_ref()
                .and_then(|env| env.get("SUBSTRATE_LLM_DEFAULT_BACKEND"))
                .map(String::as_str),
            Some("api:openai")
        );
        assert!(context.request.integrated_auth.is_none());
        assert_eq!(
            context.request.cwd.as_deref(),
            Some(explicit_launch_cwd.to_string_lossy().as_ref())
        );
        let projection = serde_json::to_string(&context.request).expect("serialize request");
        assert!(!projection.contains(ambient_secret));
        assert!(!projection.contains(".codex/auth.json"));
        assert!(!projection.contains("config.toml"));
        assert!(!projection.contains(&ambient_xdg.to_string_lossy().to_string()));
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn tampered_gateway_context_rejects_before_ambient_selection() {
        let _authority_env = AuthorityEnvTestGuard::preserve();
        let (fixture, mut carrier) = authenticated_gateway_fixture();
        let conflicting = fixture.path().join("conflicting");
        std::fs::create_dir(&conflicting).expect("conflicting home");
        std::fs::write(conflicting.join("config.yaml"), "not: [valid")
            .expect("poison ambient config");
        let replacement = if carrier.host_context_commitment.starts_with('0') {
            "1"
        } else {
            "0"
        };
        carrier
            .host_context_commitment
            .replace_range(..1, replacement);

        with_env_var("SUBSTRATE_HOME", Some(conflicting.as_os_str()), || {
            let err = match build_gateway_request_context(false, false, fixture.path(), &carrier) {
                Ok(_) => panic!("tampered carrier must fail"),
                Err(err) => err,
            };
            assert!(
                err.to_string()
                    .contains("invalid install bootstrap carrier"),
                "unexpected error: {err:#}"
            );
            assert!(!err.to_string().contains("config.yaml"));
        });
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn malformed_gateway_context_rejects_before_ambient_selection() {
        let _authority_env = AuthorityEnvTestGuard::preserve();
        let (fixture, mut carrier) = authenticated_gateway_fixture();
        carrier.host_context_commitment.replace_range(..1, "g");
        let malformed_commitment = carrier.host_context_commitment.clone();
        let conflicting = fixture.path().join("conflicting");
        std::fs::create_dir(&conflicting).expect("conflicting home");
        std::fs::write(conflicting.join("policy.yaml"), "not: [valid")
            .expect("poison ambient policy");

        with_env_var("SUBSTRATE_HOME", Some(conflicting.as_os_str()), || {
            let err = match build_gateway_request_context(false, false, fixture.path(), &carrier) {
                Ok(_) => panic!("malformed carrier must fail"),
                Err(err) => err,
            };
            assert!(
                err.to_string()
                    .contains("invalid install bootstrap carrier"),
                "unexpected error: {err:#}"
            );
            assert!(!err.to_string().contains("policy.yaml"));
            assert!(!err.to_string().contains(&malformed_commitment));
        });
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn mismatched_gateway_principal_rejects_before_ambient_selection() {
        let _authority_env = AuthorityEnvTestGuard::preserve();
        let (fixture, carrier) = authenticated_gateway_fixture();
        let transport_api_types::PlatformPrincipalV1::Unix { uid, .. } =
            &carrier.context.intended_host_principal
        else {
            panic!("expected Unix principal")
        };
        let mismatched = transport_api_types::InstallBootstrapContextCarrierV1::from_context(
            transport_api_types::InstallBootstrapContextV1::new_unix(
                &carrier.context.selected_host_prefix,
                "mismatched-principal",
                *uid,
            )
            .expect("mismatched context"),
        )
        .expect("mismatched carrier");
        let conflicting = fixture.path().join("conflicting");
        std::fs::create_dir(&conflicting).expect("conflicting home");
        std::fs::write(conflicting.join("config.yaml"), "not: [valid")
            .expect("poison ambient config");

        with_env_var("SUBSTRATE_HOME", Some(conflicting.as_os_str()), || {
            let err = match build_gateway_request_context(false, false, fixture.path(), &mismatched)
            {
                Ok(_) => panic!("mismatched principal must fail"),
                Err(err) => err,
            };
            assert!(
                err.to_string()
                    .contains("install bootstrap principal does not match current Unix principal"),
                "unexpected error: {err:#}"
            );
            assert!(!err.to_string().contains("config.yaml"));
        });
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn ambient_world_disable_toggles_apply_only_after_authenticated_a() {
        let (fixture, carrier) = authenticated_gateway_fixture();
        with_env_var(
            "SUBSTRATE_WORLD_ENABLED",
            Some(std::ffi::OsStr::new("0")),
            || {
                with_env_var(
                    "SUBSTRATE_WORLD",
                    Some(std::ffi::OsStr::new("disabled")),
                    || {
                        with_env_var(
                            "SUBSTRATE_OVERRIDE_WORLD",
                            Some(std::ffi::OsStr::new("disabled")),
                            || {
                                with_env_var("OPENAI_API_KEY", None, || {
                                    let context = build_gateway_request_context(
                                        false,
                                        false,
                                        fixture.path(),
                                        &carrier,
                                    )
                                    .expect("authenticated gateway context");
                                    assert!(world_routing_disabled(&context));
                                })
                            },
                        )
                    },
                )
            },
        );
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn ambient_world_disable_toggles_cannot_bypass_tampered_a() {
        let (fixture, mut carrier) = authenticated_gateway_fixture();
        carrier.host_context_commitment.replace_range(..1, "g");
        with_env_var(
            "SUBSTRATE_WORLD_ENABLED",
            Some(std::ffi::OsStr::new("0")),
            || {
                with_env_var(
                    "SUBSTRATE_WORLD",
                    Some(std::ffi::OsStr::new("disabled")),
                    || {
                        let err = match build_gateway_request_context(
                            false,
                            false,
                            fixture.path(),
                            &carrier,
                        ) {
                            Ok(_) => panic!("disabled routing must not bypass carrier validation"),
                            Err(err) => err,
                        };
                        assert!(
                            err.to_string()
                                .contains("invalid install bootstrap carrier"),
                            "unexpected error: {err:#}"
                        );
                    },
                )
            },
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial]
    fn linux_gateway_client_uses_fixed_service_socket_under_ambient_override() {
        with_env_var(
            "SUBSTRATE_WORLD_SOCKET",
            Some(std::ffi::OsStr::new("/tmp/ambient-must-not-select.sock")),
            || {
                let client = build_gateway_client().expect("build Linux gateway client");
                match client.transport() {
                    transport_api_client::Transport::UnixSocket { path } => {
                        assert_eq!(path, std::path::Path::new("/run/substrate.sock"));
                    }
                    other => panic!("unexpected Linux gateway transport: {other:?}"),
                }
            },
        );
    }

    #[test]
    #[serial]
    fn codex_auth_projection_uses_committed_principal_home_not_ambient_home() {
        let selected = std::path::Path::new("/account-database/home");
        with_env_var("HOME", Some(std::ffi::OsStr::new("/ambient/home")), || {
            assert_eq!(
                codex_auth_state_path(selected),
                selected.join(".codex/auth.json")
            );
        });
    }

    #[test]
    #[serial]
    fn codex_auth_projection_errors_redact_committed_account_home() {
        let temp = tempfile::tempdir().expect("tempdir");
        let sentinel_root = temp.path().join("principal-home-must-not-disclose");
        let missing_auth = sentinel_root.join(".codex/auth.json");
        let policy = substrate_broker::Policy {
            agents_host_credentials_read_allowed_backends: vec![CLI_CODEX_WORLD_BACKEND.to_string()],
            ..Default::default()
        };

        with_env_var(CODEX_ACCESS_TOKEN_ENV, None, || {
            with_env_var(CODEX_ACCOUNT_ID_ENV, None, || {
                let read_err = resolve_cli_codex_integrated_auth(
                    &policy,
                    CLI_CODEX_WORLD_BACKEND,
                    &missing_auth,
                )
                .expect_err("missing committed auth state must fail");
                assert!(read_err
                    .to_string()
                    .contains("failed to read committed Codex auth state"));
                assert!(!read_err
                    .to_string()
                    .contains("principal-home-must-not-disclose"));
                assert!(!read_err
                    .to_string()
                    .contains(&sentinel_root.to_string_lossy().to_string()));

                std::fs::create_dir_all(missing_auth.parent().expect("auth parent"))
                    .expect("create sentinel auth parent");
                std::fs::write(&missing_auth, "not-json").expect("write malformed auth state");
                let parse_err = resolve_cli_codex_integrated_auth(
                    &policy,
                    CLI_CODEX_WORLD_BACKEND,
                    &missing_auth,
                )
                .expect_err("malformed committed auth state must fail");
                assert!(parse_err
                    .to_string()
                    .contains("failed to parse committed Codex auth state"));
                assert!(!parse_err
                    .to_string()
                    .contains("principal-home-must-not-disclose"));
                assert!(!parse_err
                    .to_string()
                    .contains(&sentinel_root.to_string_lossy().to_string()));
            })
        });
    }

    #[test]
    fn non_linux_gateway_clients_are_contained_before_compatibility_selection() {
        let source = include_str!("world_gateway.rs");
        let call_start = source
            .find("fn call_gateway_action(")
            .expect("call boundary");
        let call_end = source[call_start..]
            .find("#[cfg(target_os = \"macos\")]\nstruct MacosGatewayClient")
            .map(|offset| call_start + offset)
            .expect("call boundary end");
        let call = &source[call_start..call_end];

        let mac_reject = call
            .find("authenticated macOS gateway endpoint selection is unavailable until R2-3")
            .expect("macOS authenticated rejection");
        let mac_client = call
            .find("let client = build_macos_gateway_client()")
            .expect("frozen macOS compatibility call");
        assert!(mac_reject < mac_client);

        let other_reject = call
            .find("authenticated gateway endpoint selection is unavailable on this platform until R2-3")
            .expect("other-platform authenticated rejection");
        let other_client = call[other_reject..]
            .find("let client = build_gateway_client()")
            .expect("frozen other-platform compatibility call");
        assert!(other_client > 0);
        let production = source
            .split("#[cfg(test)]\nmod classification_tests")
            .next()
            .expect("production source prefix");
        assert!(!production.contains("synthesized_unavailable_response_without_context"));
    }
}
