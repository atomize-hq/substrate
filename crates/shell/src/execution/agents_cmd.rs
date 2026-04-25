use crate::execution::agent_inventory::{
    discover_agent_files, load_effective_agent_inventory, validate_agent_file,
    AgentInventoryEntryV1,
};
use crate::execution::cli::{
    AgentAction, AgentCmd, AgentDoctorArgs, AgentScopeArg, AgentViewArgs, AgentsAction, AgentsCmd,
    Cli,
};
use crate::execution::config_model::{
    self, AgentExecutionScope, CliConfigOverrides, SubstrateConfig,
};
use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use substrate_broker::Policy;
use substrate_common::paths as substrate_paths;
use substrate_common::{AgentEvent, PlacementExecution};

const PURE_AGENT_PROTOCOL: &str = "uaa.agent.session";
const PURE_AGENT_ROUTER: &str = "agent_hub";
const NESTED_ROUTER: &str = "substrate_gateway";
const ORCHESTRATOR_ROLE: &str = "orchestrator";

pub(crate) fn handle_agent_command(cmd: &AgentCmd, cli: &Cli) -> i32 {
    match &cmd.action {
        AgentAction::List(args) => match run_list(args, cli) {
            Ok(()) => 0,
            Err(err) if config_model::is_user_error(&err) => {
                eprintln!("{err}");
                2
            }
            Err(err) => {
                eprintln!("{err:#}");
                1
            }
        },
        AgentAction::Status(args) => match run_status(args, cli) {
            Ok(()) => 0,
            Err(err) if config_model::is_user_error(&err) => {
                eprintln!("{err}");
                2
            }
            Err(err) => {
                eprintln!("{err:#}");
                1
            }
        },
        AgentAction::Doctor(args) => match run_doctor(args, cli) {
            Ok(code) => code,
            Err(err) if config_model::is_user_error(&err) => {
                eprintln!("{err}");
                2
            }
            Err(err) => {
                eprintln!("{err:#}");
                1
            }
        },
    }
}

pub(crate) fn handle_agents_command(cmd: &AgentsCmd, _cli: &Cli) -> i32 {
    let result = match &cmd.action {
        AgentsAction::Validate => run_validate(),
    };

    match result {
        Ok(()) => 0,
        Err(err) if config_model::is_user_error(&err) => {
            eprintln!("{err}");
            2
        }
        Err(err) => {
            eprintln!("{:#}", err);
            1
        }
    }
}

fn run_validate() -> Result<()> {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let (base_policy, _) = substrate_broker::resolve_effective_policy_with_explain(&cwd, false)
        .map_err(|err| config_model::user_error(err.to_string()))?;
    let agent_files = discover_agent_files(&cwd)?;

    for path in agent_files {
        validate_agent_file(&path, &base_policy)?;
    }

    Ok(())
}

fn run_list(args: &AgentViewArgs, cli: &Cli) -> Result<()> {
    let context = resolve_command_context(cli)?;
    let report = build_list_report(&context, args);
    render_list_report(&report, args.json)?;
    Ok(())
}

fn run_status(args: &AgentViewArgs, cli: &Cli) -> Result<()> {
    let context = resolve_command_context(cli)?;
    let report = build_status_report(&context, args)?;
    render_status_report(&report, args.json)?;
    Ok(())
}

fn run_doctor(args: &AgentDoctorArgs, cli: &Cli) -> Result<i32> {
    let report = build_doctor_report(cli)?;
    let exit_code = doctor_exit_code(&report);
    render_doctor_report(&report, args.json)?;
    Ok(exit_code)
}

struct AgentCommandContext {
    effective_config: SubstrateConfig,
    base_policy: Policy,
    inventory: BTreeMap<String, AgentInventoryEntryV1>,
}

fn resolve_command_context(cli: &Cli) -> Result<AgentCommandContext> {
    let cwd = current_dir();
    let effective_config = resolve_effective_config(&cwd, cli)?;
    let (base_policy, _) = substrate_broker::resolve_effective_policy_with_explain(&cwd, false)
        .map_err(|err| config_model::user_error(err.to_string()))?;
    let inventory = load_effective_agent_inventory(&cwd, &base_policy)?;

    Ok(AgentCommandContext {
        effective_config,
        base_policy,
        inventory,
    })
}

fn current_dir() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn cli_world_enabled(cli: &Cli) -> Option<bool> {
    if cli.world {
        Some(true)
    } else if cli.no_world {
        Some(false)
    } else {
        None
    }
}

fn resolve_effective_config(cwd: &Path, cli: &Cli) -> Result<SubstrateConfig> {
    config_model::resolve_effective_config(
        cwd,
        &CliConfigOverrides {
            world_enabled: cli_world_enabled(cli),
            ..Default::default()
        },
    )
}

#[derive(Clone, Serialize)]
struct ExecutionScopeJson<'a> {
    scope: &'a str,
}

#[derive(Serialize)]
struct CapabilitiesSummaryJson {
    llm: bool,
    mcp_client: bool,
}

#[derive(Serialize)]
struct EligibilityJson<'a> {
    state: &'a str,
    reason: Option<String>,
}

#[derive(Serialize)]
struct ListAgentJson<'a> {
    agent_id: String,
    backend_id: String,
    kind: &'a str,
    execution: ExecutionScopeJson<'a>,
    role: Option<&'a str>,
    capabilities_summary: CapabilitiesSummaryJson,
    eligibility: EligibilityJson<'a>,
    protocol: &'a str,
}

#[derive(Serialize)]
struct ListReportJson<'a> {
    disabled: bool,
    scope_filter: &'a str,
    role_filter: Option<&'a str>,
    agents: Vec<ListAgentJson<'a>>,
}

fn build_list_report<'a>(
    context: &'a AgentCommandContext,
    args: &'a AgentViewArgs,
) -> ListReportJson<'a> {
    let role_filter = normalized_role_filter(args.role.as_deref());
    let agents = if context.effective_config.agents.enabled {
        context
            .inventory
            .values()
            .filter_map(|entry| {
                let scope = entry.effective_scope(&context.effective_config);
                let role = role_for_entry(&entry.file.id, &context.effective_config);
                if !matches_scope(scope, args.scope) || !matches_role(role, role_filter) {
                    return None;
                }

                let backend_id = entry.derived_backend_id();
                let eligibility_reason =
                    eligibility_reason(entry, &context.effective_config, &context.base_policy);
                let eligibility = if let Some(reason) = eligibility_reason {
                    EligibilityJson {
                        state: "denied",
                        reason: Some(reason),
                    }
                } else {
                    EligibilityJson {
                        state: "allowed",
                        reason: None,
                    }
                };

                Some(ListAgentJson {
                    agent_id: entry.file.id.clone(),
                    backend_id,
                    kind: entry.file.config.kind.as_str(),
                    execution: ExecutionScopeJson {
                        scope: scope.as_str(),
                    },
                    role,
                    capabilities_summary: CapabilitiesSummaryJson {
                        llm: entry.file.config.capabilities.llm,
                        mcp_client: entry.file.config.capabilities.mcp_client,
                    },
                    eligibility,
                    protocol: PURE_AGENT_PROTOCOL,
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    ListReportJson {
        disabled: !context.effective_config.agents.enabled,
        scope_filter: args.scope.as_str(),
        role_filter,
        agents,
    }
}

fn render_list_report(report: &ListReportJson<'_>, json_mode: bool) -> Result<()> {
    if json_mode {
        println!("{}", serde_json::to_string_pretty(report)?);
        return Ok(());
    }

    println!("disabled: {}", report.disabled);
    println!(
        "scope_filter: {}{}",
        report.scope_filter,
        report
            .role_filter
            .map(|role| format!(", role_filter: {role}"))
            .unwrap_or_default()
    );
    println!(
        "agent_id\tbackend_id\tkind\texecution.scope\trole\tcapabilities\teligibility\tprotocol"
    );

    for agent in &report.agents {
        let capabilities = capabilities_label(&agent.capabilities_summary);
        let eligibility = if let Some(reason) = &agent.eligibility.reason {
            format!("{}: {}", agent.eligibility.state, reason)
        } else {
            agent.eligibility.state.to_string()
        };
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            agent.agent_id,
            agent.backend_id,
            agent.kind,
            agent.execution.scope,
            agent.role.unwrap_or(""),
            capabilities,
            eligibility,
            agent.protocol
        );
    }

    Ok(())
}

#[derive(Clone, Serialize)]
struct StatusSessionJson {
    orchestration_session_id: String,
    agent_id: String,
    backend_id: String,
    client: String,
    router: String,
    protocol: String,
    execution: ExecutionScopeJson<'static>,
    role: Option<String>,
    last_event_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    world_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    world_generation: Option<u64>,
}

#[derive(Clone, Serialize)]
struct NestedParentJson {
    orchestration_session_id: String,
    agent_id: String,
}

#[derive(Clone, Serialize)]
struct NestedLlmRecordJson {
    parent: NestedParentJson,
    backend_id: String,
    client: String,
    router: String,
    provider: String,
    auth_authority: String,
    protocol: String,
}

#[derive(Serialize)]
struct StatusReportJson<'a> {
    disabled: bool,
    scope_filter: &'a str,
    role_filter: Option<&'a str>,
    orchestrator_agent_id: String,
    sessions: Vec<StatusSessionJson>,
    nested_llm_records: Vec<NestedLlmRecordJson>,
}

#[derive(Clone)]
struct SessionProjection {
    session: StatusSessionJson,
}

#[derive(Clone)]
struct NestedProjection {
    record: NestedLlmRecordJson,
    sort_key: (String, String, String, String, String, String),
}

fn build_status_report<'a>(
    context: &'a AgentCommandContext,
    args: &'a AgentViewArgs,
) -> Result<StatusReportJson<'a>> {
    let role_filter = normalized_role_filter(args.role.as_deref());
    let orchestrator_agent_id = context
        .effective_config
        .agents
        .hub
        .orchestrator_agent_id
        .clone();

    if !context.effective_config.agents.enabled {
        return Ok(StatusReportJson {
            disabled: true,
            scope_filter: args.scope.as_str(),
            role_filter,
            orchestrator_agent_id,
            sessions: Vec::new(),
            nested_llm_records: Vec::new(),
        });
    }

    let events = read_trace_agent_events()?;
    let mut sessions = BTreeMap::<(String, String), SessionProjection>::new();
    let mut nested =
        BTreeMap::<(String, String, String, String, String, String), NestedProjection>::new();

    for event in events {
        let Some(entry) = context.inventory.get(&event.agent_id) else {
            continue;
        };
        let role = role_for_event(&event, &entry.file.id, &context.effective_config);
        let scope = scope_for_event(&event, entry, &context.effective_config);

        if let Some(session_key) = pure_session_key(&event) {
            let mut world_id = None;
            let mut world_generation = None;
            if scope == AgentExecutionScope::World {
                let maybe_world_id = event.world_id.clone();
                let maybe_world_generation = event.world_generation.or_else(|| {
                    event
                        .data
                        .get("world_generation")
                        .and_then(serde_json::Value::as_u64)
                });
                if maybe_world_id.is_some() && maybe_world_generation.is_some() {
                    world_id = maybe_world_id;
                    world_generation = maybe_world_generation;
                }
            }

            let projection = SessionProjection {
                session: StatusSessionJson {
                    orchestration_session_id: session_key.0.clone(),
                    agent_id: entry.file.id.clone(),
                    backend_id: entry.derived_backend_id(),
                    client: entry.file.id.clone(),
                    router: PURE_AGENT_ROUTER.to_string(),
                    protocol: PURE_AGENT_PROTOCOL.to_string(),
                    execution: ExecutionScopeJson {
                        scope: scope.as_str(),
                    },
                    role: role.map(ToOwned::to_owned),
                    last_event_at: event.ts.to_rfc3339(),
                    world_id,
                    world_generation,
                },
            };

            sessions.insert(session_key, projection);
        }

        if let Some(projection) = nested_projection(&event, entry) {
            nested.insert(projection.sort_key.clone(), projection);
        }
    }

    let filtered_sessions: Vec<StatusSessionJson> = sessions
        .into_values()
        .filter(|projection| {
            matches_scope(
                scope_from_label(projection.session.execution.scope),
                args.scope,
            ) && matches_role(projection.session.role.as_deref(), role_filter)
        })
        .map(|projection| projection.session)
        .collect();

    let allowed_parents: BTreeSet<(String, String)> = filtered_sessions
        .iter()
        .map(|session| {
            (
                session.orchestration_session_id.clone(),
                session.agent_id.clone(),
            )
        })
        .collect();

    let filtered_nested: Vec<NestedLlmRecordJson> = nested
        .into_values()
        .filter(|projection| {
            allowed_parents.contains(&(
                projection.record.parent.orchestration_session_id.clone(),
                projection.record.parent.agent_id.clone(),
            ))
        })
        .map(|projection| projection.record)
        .collect();

    Ok(StatusReportJson {
        disabled: false,
        scope_filter: args.scope.as_str(),
        role_filter,
        orchestrator_agent_id,
        sessions: filtered_sessions,
        nested_llm_records: filtered_nested,
    })
}

fn render_status_report(report: &StatusReportJson<'_>, json_mode: bool) -> Result<()> {
    if json_mode {
        println!("{}", serde_json::to_string_pretty(report)?);
        return Ok(());
    }

    println!("disabled: {}", report.disabled);
    println!("orchestrator_agent_id: {}", report.orchestrator_agent_id);
    println!("scope_filter: {}", report.scope_filter);
    if let Some(role_filter) = report.role_filter {
        println!("role_filter: {role_filter}");
    }

    println!();
    println!("orchestrator");
    println!("  agent_id: {}", report.orchestrator_agent_id);

    println!();
    println!("sessions");
    for session in &report.sessions {
        let mut fields = vec![
            format!(
                "orchestration_session_id={}",
                session.orchestration_session_id
            ),
            format!("agent_id={}", session.agent_id),
            format!("backend_id={}", session.backend_id),
            format!("client={}", session.client),
            format!("router={}", session.router),
            format!("protocol={}", session.protocol),
            format!("execution.scope={}", session.execution.scope),
            format!("role={}", session.role.as_deref().unwrap_or("")),
            format!("last_event_at={}", session.last_event_at),
        ];
        if let (Some(world_id), Some(world_generation)) =
            (session.world_id.as_deref(), session.world_generation)
        {
            fields.push(format!("world_id={world_id}"));
            fields.push(format!("world_generation={world_generation}"));
        }
        println!("  {}", fields.join(" | "));
    }

    if !report.nested_llm_records.is_empty() {
        println!();
        println!("nested_llm_records");
        for record in &report.nested_llm_records {
            println!(
                "  parent.orchestration_session_id={} | parent.agent_id={} | backend_id={} | client={} | router={} | provider={} | auth_authority={} | protocol={}",
                record.parent.orchestration_session_id,
                record.parent.agent_id,
                record.backend_id,
                record.client,
                record.router,
                record.provider,
                record.auth_authority,
                record.protocol
            );
        }
    }

    Ok(())
}

fn read_trace_agent_events() -> Result<Vec<AgentEvent>> {
    let trace_path = trace_log_path()?;
    let file = match File::open(&trace_path) {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => {
            return Err(err)
                .with_context(|| format!("failed to read trace log at {}", trace_path.display()))
        }
    };

    let reader = BufReader::new(file);
    let mut events = Vec::new();
    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => continue,
        };
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if value.get("event_type").and_then(serde_json::Value::as_str) != Some("agent_event") {
            continue;
        }
        let event: AgentEvent = match serde_json::from_value(value) {
            Ok(event) => event,
            Err(_) => continue,
        };
        events.push(event);
    }

    Ok(events)
}

fn trace_log_path() -> Result<PathBuf> {
    if let Ok(path) = env::var("SHIM_TRACE_LOG") {
        return Ok(PathBuf::from(path));
    }

    Ok(substrate_paths::substrate_home()?.join("trace.jsonl"))
}

fn pure_session_key(event: &AgentEvent) -> Option<(String, String)> {
    let tuple = event.identity_tuple.as_ref()?;
    if tuple.router != PURE_AGENT_ROUTER || tuple.protocol != PURE_AGENT_PROTOCOL {
        return None;
    }
    Some((
        event.orchestration_session_id.clone(),
        event.agent_id.clone(),
    ))
}

fn nested_projection(
    event: &AgentEvent,
    entry: &AgentInventoryEntryV1,
) -> Option<NestedProjection> {
    let tuple = event.identity_tuple.as_ref()?;
    if tuple.router != NESTED_ROUTER {
        return None;
    }
    let provider = tuple.provider.clone()?;
    let auth_authority = tuple.auth_authority.clone()?;
    let sort_key = (
        event.orchestration_session_id.clone(),
        event.agent_id.clone(),
        entry.derived_backend_id(),
        provider.clone(),
        auth_authority.clone(),
        tuple.protocol.clone(),
    );

    Some(NestedProjection {
        record: NestedLlmRecordJson {
            parent: NestedParentJson {
                orchestration_session_id: event.orchestration_session_id.clone(),
                agent_id: event.agent_id.clone(),
            },
            backend_id: entry.derived_backend_id(),
            client: event.agent_id.clone(),
            router: NESTED_ROUTER.to_string(),
            provider,
            auth_authority,
            protocol: tuple.protocol.clone(),
        },
        sort_key,
    })
}

fn scope_for_event(
    event: &AgentEvent,
    entry: &AgentInventoryEntryV1,
    effective_config: &SubstrateConfig,
) -> AgentExecutionScope {
    match event
        .placement_posture
        .as_ref()
        .map(|posture| posture.execution)
    {
        Some(PlacementExecution::HostOnly) => AgentExecutionScope::Host,
        Some(PlacementExecution::InWorld) => AgentExecutionScope::World,
        None => entry.effective_scope(effective_config),
    }
}

fn role_for_entry<'a>(agent_id: &str, effective_config: &'a SubstrateConfig) -> Option<&'a str> {
    if effective_config.agents.hub.orchestrator_agent_id == agent_id {
        Some(ORCHESTRATOR_ROLE)
    } else {
        None
    }
}

fn role_for_event<'a>(
    event: &'a AgentEvent,
    agent_id: &str,
    effective_config: &'a SubstrateConfig,
) -> Option<&'a str> {
    match event.role.as_deref() {
        Some(ORCHESTRATOR_ROLE) => Some(ORCHESTRATOR_ROLE),
        _ => role_for_entry(agent_id, effective_config),
    }
}

fn eligibility_reason(
    entry: &AgentInventoryEntryV1,
    effective_config: &SubstrateConfig,
    base_policy: &Policy,
) -> Option<String> {
    if !entry.file.config.enabled {
        return Some("agent is disabled in the effective inventory".to_string());
    }

    let backend_id = entry.derived_backend_id();
    if !backend_allowed(base_policy, &backend_id) {
        return Some(format!(
            "{backend_id} is not allowlisted by effective policy agents.allowed_backends"
        ));
    }

    let _ = effective_config;
    None
}

fn backend_allowed(policy: &Policy, backend_id: &str) -> bool {
    policy
        .agents_allowed_backends
        .iter()
        .any(|allowed| allowed == backend_id)
}

fn enabled_world_member_exists(
    inventory: &BTreeMap<String, AgentInventoryEntryV1>,
    effective_config: &SubstrateConfig,
) -> bool {
    inventory.values().any(|entry| {
        entry.file.config.enabled
            && entry.effective_scope(effective_config) == AgentExecutionScope::World
    })
}

#[derive(Serialize)]
struct DoctorOrchestratorJson<'a> {
    agent_id: String,
    backend_id: String,
    execution: ExecutionScopeJson<'a>,
}

#[derive(Clone, Serialize)]
struct DoctorCheckJson {
    check: String,
    status: String,
    reason: Option<String>,
}

#[derive(Serialize)]
struct DoctorReportJson<'a> {
    healthy: bool,
    fail_closed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    orchestrator: Option<DoctorOrchestratorJson<'a>>,
    checks: Vec<DoctorCheckJson>,
}

fn build_doctor_report(cli: &Cli) -> Result<DoctorReportJson<'static>> {
    let cwd = current_dir();
    let effective_config = match resolve_effective_config(&cwd, cli) {
        Ok(config) => config,
        Err(err) => {
            if config_model::is_user_error(&err) {
                return Ok(failed_doctor_report(
                    "inventory_scan",
                    err.to_string(),
                    None,
                ));
            }
            return Err(err);
        }
    };

    let (base_policy, _) =
        match substrate_broker::resolve_effective_policy_with_explain(&cwd, false)
            .map_err(|err| config_model::user_error(err.to_string()))
        {
            Ok(value) => value,
            Err(err) => {
                if config_model::is_user_error(&err) {
                    return Ok(failed_doctor_report(
                        "inventory_scan",
                        err.to_string(),
                        None,
                    ));
                }
                return Err(err);
            }
        };

    let inventory = match load_effective_agent_inventory(&cwd, &base_policy) {
        Ok(inventory) => inventory,
        Err(err) => {
            if config_model::is_user_error(&err) {
                return Ok(failed_doctor_report(
                    "inventory_scan",
                    err.to_string(),
                    None,
                ));
            }
            return Err(err);
        }
    };

    let mut checks = vec![DoctorCheckJson {
        check: "inventory_scan".to_string(),
        status: "pass".to_string(),
        reason: None,
    }];

    let orchestrator = match validate_orchestrator_selection(&effective_config, &inventory) {
        Ok(entry) => {
            checks.push(DoctorCheckJson {
                check: "orchestrator_selection".to_string(),
                status: "pass".to_string(),
                reason: None,
            });
            DoctorOrchestratorJson {
                agent_id: entry.file.id.clone(),
                backend_id: entry.derived_backend_id(),
                execution: ExecutionScopeJson { scope: "host" },
            }
        }
        Err(reason) => {
            checks.push(DoctorCheckJson {
                check: "orchestrator_selection".to_string(),
                status: "fail".to_string(),
                reason: Some(reason),
            });
            return Ok(DoctorReportJson {
                healthy: false,
                fail_closed: true,
                orchestrator: None,
                checks,
            });
        }
    };

    if let Some(reason) = policy_allowlist_failure(&effective_config, &inventory, &base_policy) {
        checks.push(DoctorCheckJson {
            check: "policy_allowlist".to_string(),
            status: "fail".to_string(),
            reason: Some(reason),
        });
        return Ok(DoctorReportJson {
            healthy: false,
            fail_closed: true,
            orchestrator: Some(orchestrator),
            checks,
        });
    }
    checks.push(DoctorCheckJson {
        check: "policy_allowlist".to_string(),
        status: "pass".to_string(),
        reason: None,
    });

    if enabled_world_member_exists(&inventory, &effective_config) {
        if effective_config.world.enabled {
            checks.push(DoctorCheckJson {
                check: "world_boundary".to_string(),
                status: "pass".to_string(),
                reason: None,
            });
        } else {
            checks.push(DoctorCheckJson {
                check: "world_boundary".to_string(),
                status: "fail".to_string(),
                reason: Some(
                    "world-scoped member posture requires world isolation but world is disabled"
                        .to_string(),
                ),
            });
            return Ok(DoctorReportJson {
                healthy: false,
                fail_closed: true,
                orchestrator: Some(orchestrator),
                checks,
            });
        }
    } else {
        checks.push(DoctorCheckJson {
            check: "world_boundary".to_string(),
            status: "not_applicable".to_string(),
            reason: None,
        });
    }

    Ok(DoctorReportJson {
        healthy: true,
        fail_closed: false,
        orchestrator: Some(orchestrator),
        checks,
    })
}

fn failed_doctor_report(
    check: &str,
    reason: String,
    orchestrator: Option<DoctorOrchestratorJson<'static>>,
) -> DoctorReportJson<'static> {
    DoctorReportJson {
        healthy: false,
        fail_closed: true,
        orchestrator,
        checks: vec![DoctorCheckJson {
            check: check.to_string(),
            status: "fail".to_string(),
            reason: Some(reason),
        }],
    }
}

fn validate_orchestrator_selection<'a>(
    effective_config: &SubstrateConfig,
    inventory: &'a BTreeMap<String, AgentInventoryEntryV1>,
) -> std::result::Result<&'a AgentInventoryEntryV1, String> {
    if !effective_config.agents.enabled {
        return Err("agents are disabled by effective config".to_string());
    }

    let orchestrator_agent_id = effective_config.agents.hub.orchestrator_agent_id.trim();
    if orchestrator_agent_id.is_empty() {
        return Err("agents.hub.orchestrator_agent_id must select an orchestrator".to_string());
    }

    let entry = inventory.get(orchestrator_agent_id).ok_or_else(|| {
        format!(
            "agents.hub.orchestrator_agent_id '{}' is not present in the effective agent inventory",
            orchestrator_agent_id
        )
    })?;

    if !entry.file.config.enabled {
        return Err(format!(
            "selected orchestrator '{}' is disabled in the effective inventory",
            orchestrator_agent_id
        ));
    }

    if entry.effective_scope(effective_config) != AgentExecutionScope::Host {
        return Err(format!(
            "selected orchestrator '{}' must resolve to execution.scope=host",
            orchestrator_agent_id
        ));
    }

    Ok(entry)
}

fn policy_allowlist_failure(
    effective_config: &SubstrateConfig,
    inventory: &BTreeMap<String, AgentInventoryEntryV1>,
    base_policy: &Policy,
) -> Option<String> {
    let orchestrator = validate_orchestrator_selection(effective_config, inventory).ok()?;
    let orchestrator_backend_id = orchestrator.derived_backend_id();
    if !backend_allowed(base_policy, &orchestrator_backend_id) {
        return Some(format!(
            "selected orchestrator backend '{}' is not allowlisted by effective policy agents.allowed_backends",
            orchestrator_backend_id
        ));
    }

    for entry in inventory.values() {
        if !entry.file.config.enabled
            || entry.effective_scope(effective_config) != AgentExecutionScope::World
        {
            continue;
        }
        let backend_id = entry.derived_backend_id();
        if !backend_allowed(base_policy, &backend_id) {
            return Some(format!(
                "required world-scoped member backend '{}' is not allowlisted by effective policy agents.allowed_backends",
                backend_id
            ));
        }
    }

    None
}

fn doctor_exit_code(report: &DoctorReportJson<'_>) -> i32 {
    if report.healthy {
        return 0;
    }

    let Some(failed_check) = report
        .checks
        .iter()
        .find(|check| check.status == "fail")
        .map(|check| check.check.as_str())
    else {
        return 1;
    };

    match failed_check {
        "policy_allowlist" => 5,
        "world_boundary" => 3,
        "inventory_scan" | "orchestrator_selection" => 2,
        _ => 1,
    }
}

fn render_doctor_report(report: &DoctorReportJson<'_>, json_mode: bool) -> Result<()> {
    if json_mode {
        println!("{}", serde_json::to_string_pretty(report)?);
        return Ok(());
    }

    println!(
        "{}",
        if report.healthy {
            "healthy"
        } else {
            "fail_closed"
        }
    );
    if let Some(orchestrator) = &report.orchestrator {
        println!("orchestrator");
        println!("  agent_id: {}", orchestrator.agent_id);
        println!("  backend_id: {}", orchestrator.backend_id);
        println!("  execution.scope: {}", orchestrator.execution.scope);
    }
    println!("checks");
    for check in &report.checks {
        match check.reason.as_deref() {
            Some(reason) if check.status == "fail" => {
                println!("  {}: fail: {}", check.check, reason);
            }
            _ => println!("  {}: {}", check.check, check.status),
        }
    }

    Ok(())
}

fn matches_scope(scope: AgentExecutionScope, filter: AgentScopeArg) -> bool {
    match filter {
        AgentScopeArg::Any => true,
        AgentScopeArg::Host => scope == AgentExecutionScope::Host,
        AgentScopeArg::World => scope == AgentExecutionScope::World,
    }
}

fn scope_from_label(value: &str) -> AgentExecutionScope {
    match value {
        "host" => AgentExecutionScope::Host,
        "world" => AgentExecutionScope::World,
        _ => AgentExecutionScope::World,
    }
}

fn matches_role(role: Option<&str>, role_filter: Option<&str>) -> bool {
    match role_filter {
        Some(filter) => role == Some(filter),
        None => true,
    }
}

fn normalized_role_filter(role: Option<&str>) -> Option<&str> {
    role.map(str::trim).filter(|role| !role.is_empty())
}

fn capabilities_label(summary: &CapabilitiesSummaryJson) -> String {
    let mut parts = Vec::new();
    if summary.llm {
        parts.push("llm");
    }
    if summary.mcp_client {
        parts.push("mcp_client");
    }
    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join(",")
    }
}

impl AgentExecutionScope {
    fn as_str(self) -> &'static str {
        match self {
            Self::Host => "host",
            Self::World => "world",
        }
    }
}
