use crate::execution::agent_inventory::{
    discover_agent_files, load_effective_agent_inventory, validate_agent_file,
    AgentInventoryEntryV1,
};
#[cfg(unix)]
use crate::execution::agent_runtime::control::request_private_stop;
use crate::execution::agent_runtime::control::{
    hidden_owner_helper_readiness_timed_out, load_hidden_owner_helper_launch_plan,
    load_public_prompt_source, persist_hidden_owner_helper_launch_plan,
    persist_runtime_stop_closeout, public_prompt_rendered_exit_code,
    reconcile_hidden_owner_helper_start_timeout, remove_hidden_owner_helper_launch_plan,
    run_public_prompt_command, wait_for_hidden_owner_helper_readiness, HiddenOwnerHelperLaunchPlan,
    HiddenOwnerHelperParticipantPlan, HiddenOwnerHelperSessionPlan,
    HiddenOwnerHelperStartTimeoutReconciliation, OwnerHelperMode, PublicPromptAction,
    PublicPromptCommandRequest, PublicPromptInput, PublicSessionPosture,
    HIDDEN_OWNER_HELPER_SUBCOMMAND,
};
#[cfg(unix)]
use crate::execution::agent_runtime::control::{
    private_stop_transport_path, register_hidden_owner_helper_startup_prompt_listener,
    run_hidden_owner_helper_startup_prompt_stream,
    run_hidden_owner_helper_startup_prompt_stream_with_action, HiddenOwnerHelperStartupPromptPlan,
};
use crate::execution::agent_runtime::orchestration_session::{
    OrchestrationSessionPosture, OrchestrationSessionRecord,
};
use crate::execution::agent_runtime::session::AgentRuntimeReplacementParticipantInit;
#[cfg(unix)]
use crate::execution::agent_runtime::state_store::HiddenOwnerHelperLaunchReadiness;
use crate::execution::agent_runtime::validator::{
    exact_backend_selection_error_exit_code, member_selection_error_exit_code,
    validate_exact_backend_selection, validate_member_selection,
};
#[cfg(unix)]
use crate::execution::agent_runtime::StartupPromptReplayState;
use crate::execution::agent_runtime::{
    runtime_realizability_error_exit_code, validate_orchestrator_selection,
    validate_runtime_realizability, AgentRuntimeParticipantRecord, AgentRuntimeSessionRecord,
    AgentRuntimeStateStore, PublicControlAction, PublicTurnTargetKind, MEMBER_ROLE, NESTED_ROUTER,
    ORCHESTRATOR_ROLE, PURE_AGENT_PROTOCOL, PURE_AGENT_ROUTER,
};
use crate::execution::cli::{
    AgentAction, AgentCmd, AgentDoctorArgs, AgentOwnerHelperArgs, AgentScopeArg,
    AgentSessionControlArgs, AgentStartArgs, AgentToolboxAction, AgentToolboxCmd,
    AgentToolboxViewArgs, AgentTurnArgs, AgentViewArgs, AgentsAction, AgentsCmd, Cli,
};
use crate::execution::config_model::{
    self, AgentExecutionScope, AgentToolboxBindTransport, CliConfigOverrides, SubstrateConfig,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use substrate_broker::Policy;
use substrate_common::paths as substrate_paths;
use substrate_common::{AgentEvent, PlacementExecution};
#[cfg(unix)]
use tokio::runtime::Builder as TokioRuntimeBuilder;
use uuid::Uuid;
const TOOLBOX_VERSION: u32 = 1;
#[cfg(unix)]
const START_DETACH_NORMALIZATION_TIMEOUT: Duration = Duration::from_secs(10);
#[cfg(unix)]
const START_DETACH_NORMALIZATION_POLL_INTERVAL: Duration = Duration::from_millis(50);
#[cfg(unix)]
const START_ATTACHED_GRACE_TIMEOUT: Duration = Duration::from_millis(250);
#[cfg(unix)]
const TURN_DETACH_NORMALIZATION_TIMEOUT: Duration = Duration::from_secs(10);
#[cfg(unix)]
const TURN_DETACH_NORMALIZATION_POLL_INTERVAL: Duration = Duration::from_millis(50);

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
        AgentAction::Start(args) => render_public_prompt_command_result(run_start(args, cli)),
        AgentAction::Turn(args) => render_public_prompt_command_result(run_turn(args, cli)),
        AgentAction::Reattach(args) => match run_reattach(args, cli) {
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
        AgentAction::Fork(args) => match run_fork(args, cli) {
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
        AgentAction::Stop(args) => match run_stop(args, cli) {
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
        AgentAction::OwnerHelper(args) => match run_owner_helper(args) {
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
        AgentAction::Toolbox(cmd) => handle_agent_toolbox_command(cmd, cli),
    }
}

fn render_public_prompt_command_result(result: Result<()>) -> i32 {
    match result {
        Ok(()) => 0,
        Err(err) => {
            if let Some(code) = public_prompt_rendered_exit_code(&err) {
                return code;
            }
            if config_model::is_user_error(&err) {
                eprintln!("{err}");
                return 2;
            }
            eprintln!("{err:#}");
            1
        }
    }
}

fn handle_agent_toolbox_command(cmd: &AgentToolboxCmd, cli: &Cli) -> i32 {
    let result = match &cmd.action {
        AgentToolboxAction::Status(args) => run_toolbox_status(args, cli),
        AgentToolboxAction::Env(args) => run_toolbox_env(args, cli),
    };

    match result {
        Ok(code) => code,
        Err(err) if config_model::is_user_error(&err) => {
            eprintln!("{err}");
            2
        }
        Err(err) => {
            eprintln!("{err:#}");
            1
        }
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

fn run_owner_helper(args: &AgentOwnerHelperArgs) -> Result<i32> {
    let plan = load_hidden_owner_helper_launch_plan(&args.plan_file)?;
    remove_hidden_owner_helper_launch_plan(&args.plan_file)?;
    crate::repl::async_repl::run_hidden_owner_helper(plan)
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct HiddenOwnerHelperLaunchReceipt {
    pub helper_pid: u32,
    pub orchestration_session_id: String,
    pub participant_id: String,
    pub backend_id: String,
}

#[allow(dead_code)]
pub(crate) fn launch_hidden_owner_helper(
    plan: &HiddenOwnerHelperLaunchPlan,
    cli: &Cli,
) -> Result<HiddenOwnerHelperLaunchReceipt> {
    let store = AgentRuntimeStateStore::new()?;
    let plan_path = persist_hidden_owner_helper_launch_plan(&store, plan)?;
    let exe = env::current_exe()
        .context("failed to resolve current substrate executable for hidden owner-helper launch")?;
    let mut command = Command::new(exe);
    if cli.world {
        command.arg("--world");
    } else if cli.no_world {
        command.arg("--no-world");
    }
    command
        .args(["agent", HIDDEN_OWNER_HELPER_SUBCOMMAND, "--plan-file"])
        .arg(&plan_path)
        .current_dir(&plan.session.workspace_root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let mut child = command.spawn().with_context(|| {
        format!(
            "failed to spawn hidden owner-helper for orchestration session {}",
            plan.session.orchestration_session_id
        )
    })?;
    if let Err(err) = wait_for_hidden_owner_helper_readiness(&store, plan) {
        let reconciled = if plan.mode == OwnerHelperMode::Start
            && hidden_owner_helper_readiness_timed_out(&err)
        {
            Some(reconcile_hidden_owner_helper_start_timeout(&store, plan))
        } else {
            None
        };
        match reconciled {
            Some(Ok(HiddenOwnerHelperStartTimeoutReconciliation::Success)) => {}
            Some(Ok(HiddenOwnerHelperStartTimeoutReconciliation::FailureMarkedTerminal)) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = remove_hidden_owner_helper_launch_plan(&plan_path);
                return Err(anyhow::anyhow!(
                    "{}; persisted terminal startup failure for orchestration session {}",
                    err,
                    plan.orchestration_session_id(),
                ));
            }
            Some(Ok(HiddenOwnerHelperStartTimeoutReconciliation::FailureUnchanged)) | None => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = remove_hidden_owner_helper_launch_plan(&plan_path);
                return Err(err);
            }
            Some(Err(reconcile_err)) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = remove_hidden_owner_helper_launch_plan(&plan_path);
                return Err(anyhow::anyhow!(
                    "{}; additionally failed to reconcile persisted startup state: {reconcile_err:#}",
                    err
                ));
            }
        }
    }
    #[cfg(unix)]
    if let Err(err) = stabilize_hidden_owner_helper_start_return(&store, plan, &mut child) {
        let _ = child.kill();
        let _ = child.wait();
        let _ = remove_hidden_owner_helper_launch_plan(&plan_path);
        return Err(err);
    }

    Ok(HiddenOwnerHelperLaunchReceipt {
        helper_pid: child.id(),
        orchestration_session_id: plan.session.orchestration_session_id.clone(),
        participant_id: plan.participant.participant_id.clone(),
        backend_id: plan.descriptor.backend_id.clone(),
    })
}

const AGENT_CONTROL_STOP_WAIT_TIMEOUT: Duration = Duration::from_secs(10);
const AGENT_CONTROL_STOP_WAIT_POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Serialize)]
struct AgentControlResultJson<'a> {
    action: &'a str,
    orchestration_session_id: &'a str,
    backend_id: &'a str,
    scope: &'a str,
    state: &'a str,
    warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    participant_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_orchestration_session_id: Option<&'a str>,
}

fn run_start(args: &AgentStartArgs, cli: &Cli) -> Result<()> {
    let prompt = load_public_prompt_source(&PublicPromptInput {
        prompt: args.prompt_source.prompt.clone(),
        prompt_file: args.prompt_source.prompt_file.clone(),
    })
    .map_err(normalize_public_prompt_error)?;
    let context = resolve_command_context(cli)?;
    let store = AgentRuntimeStateStore::new()?;
    let plan = build_start_launch_plan(args, &context)?;

    #[cfg(not(unix))]
    {
        let _ = store;
        let _ = plan;
        let _ = prompt;
        anyhow::bail!(config_model::user_error(
            "unsupported_platform_or_posture: public start prompt streaming requires a Unix launch-time backchannel"
        ));
    }

    #[cfg(unix)]
    {
        let mut plan = plan;
        let startup_listener = register_hidden_owner_helper_startup_prompt_listener(
            &store,
            plan.orchestration_session_id(),
            plan.participant_id(),
        )
        .map_err(runtime_start_error)?;
        plan.startup_prompt = Some(HiddenOwnerHelperStartupPromptPlan {
            prompt_text: prompt.prompt_text.clone(),
            stream_path: startup_listener.path().to_path_buf(),
        });

        let receipt = launch_hidden_owner_helper(&plan, cli).map_err(runtime_start_error)?;
        match run_hidden_owner_helper_startup_prompt_stream(startup_listener, args.json) {
            Ok(()) => wait_for_start_prompt_completion_normalization(
                &store,
                &receipt.orchestration_session_id,
                &receipt.participant_id,
            )
            .map_err(runtime_start_error),
            Err(err)
                if recoverable_detached_start_retry(
                    &receipt.orchestration_session_id,
                    &receipt.participant_id,
                    receipt.helper_pid,
                    &err,
                )? =>
            {
                let retry_listener = register_hidden_owner_helper_startup_prompt_listener(
                    &store,
                    plan.orchestration_session_id(),
                    plan.participant_id(),
                )
                .map_err(runtime_start_error)?;
                plan.startup_prompt = Some(HiddenOwnerHelperStartupPromptPlan {
                    prompt_text: prompt.prompt_text,
                    stream_path: retry_listener.path().to_path_buf(),
                });
                let retry_receipt =
                    launch_hidden_owner_helper(&plan, cli).map_err(runtime_start_error)?;
                run_hidden_owner_helper_startup_prompt_stream(retry_listener, args.json)
                    .map_err(normalize_public_prompt_error)?;
                wait_for_start_prompt_completion_normalization(
                    &store,
                    &retry_receipt.orchestration_session_id,
                    &retry_receipt.participant_id,
                )
                .map_err(runtime_start_error)
            }
            Err(err) => Err(normalize_public_prompt_error(err)),
        }
    }
}

#[cfg(unix)]
fn stabilize_hidden_owner_helper_start_return(
    store: &AgentRuntimeStateStore,
    plan: &HiddenOwnerHelperLaunchPlan,
    child: &mut std::process::Child,
) -> Result<()> {
    if plan.mode != OwnerHelperMode::Start {
        return Ok(());
    }

    let grace_started_at = std::time::Instant::now();
    loop {
        if matches!(
            store.classify_hidden_owner_helper_launch_readiness(
                plan.orchestration_session_id(),
                plan.participant_id(),
                plan.requires_internal_session_id(),
            )?,
            HiddenOwnerHelperLaunchReadiness::ReadyDetached(_)
        ) {
            return Ok(());
        }
        if child
            .try_wait()
            .context("failed to poll hidden owner-helper exit status")?
            .is_some()
        {
            break;
        }
        if grace_started_at.elapsed() >= START_ATTACHED_GRACE_TIMEOUT {
            return Ok(());
        }
        thread::sleep(START_DETACH_NORMALIZATION_POLL_INTERVAL);
    }

    let normalization_started_at = std::time::Instant::now();
    loop {
        if matches!(
            store.classify_hidden_owner_helper_launch_readiness(
                plan.orchestration_session_id(),
                plan.participant_id(),
                plan.requires_internal_session_id(),
            )?,
            HiddenOwnerHelperLaunchReadiness::ReadyDetached(_)
        ) {
            return Ok(());
        }

        if normalization_started_at.elapsed() >= START_DETACH_NORMALIZATION_TIMEOUT {
            let snapshot_summary = store
                .load_orchestration_session(plan.orchestration_session_id())?
                .map(|session| {
                    format!(
                        "state={:?}, posture={:?}, attached_participant_id={:?}, shell_owner_pid={}",
                        session.state,
                        session.posture,
                        session.attached_participant_id,
                        session.shell_owner_pid,
                    )
                })
                .unwrap_or_else(|| "session_missing".to_string());
            anyhow::bail!(
                "timed out waiting for detached start normalization for orchestration session {} after hidden owner-helper {} exited ({snapshot_summary})",
                plan.orchestration_session_id(),
                child.id(),
            );
        }

        thread::sleep(START_DETACH_NORMALIZATION_POLL_INTERVAL);
    }
}

#[cfg(unix)]
fn wait_for_start_prompt_completion_normalization(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    participant_id: &str,
) -> Result<()> {
    let normalization_started_at = std::time::Instant::now();
    let mut attached_grace_started_at: Option<std::time::Instant> = None;
    loop {
        match store.classify_hidden_owner_helper_launch_readiness(
            orchestration_session_id,
            participant_id,
            true,
        )? {
            HiddenOwnerHelperLaunchReadiness::ReadyDetached(_) => return Ok(()),
            HiddenOwnerHelperLaunchReadiness::ReadyAttached => {
                let grace_started_at =
                    attached_grace_started_at.get_or_insert_with(std::time::Instant::now);
                if grace_started_at.elapsed() >= START_ATTACHED_GRACE_TIMEOUT {
                    return Ok(());
                }
            }
            HiddenOwnerHelperLaunchReadiness::Pending => {
                attached_grace_started_at = None;
            }
        }

        if normalization_started_at.elapsed() >= START_DETACH_NORMALIZATION_TIMEOUT {
            let snapshot_summary = store
                .load_orchestration_session(orchestration_session_id)?
                .map(|session| {
                    format!(
                        "state={:?}, posture={:?}, attached_participant_id={:?}, shell_owner_pid={}",
                        session.state,
                        session.posture,
                        session.attached_participant_id,
                        session.shell_owner_pid,
                    )
                })
                .unwrap_or_else(|| "session_missing".to_string());
            anyhow::bail!(
                "timed out waiting for detached start normalization after startup prompt completion for orchestration session {} ({snapshot_summary})",
                orchestration_session_id,
            );
        }

        thread::sleep(START_DETACH_NORMALIZATION_POLL_INTERVAL);
    }
}

#[cfg(unix)]
fn recoverable_detached_start_retry(
    orchestration_session_id: &str,
    participant_id: &str,
    helper_pid: u32,
    err: &anyhow::Error,
) -> Result<bool> {
    if public_prompt_rendered_exit_code(err).is_some() {
        return Ok(false);
    }

    let message = err.to_string();
    let prompt_transport_failed = message.contains("startup prompt stream")
        || message.starts_with("stream_bridge_failed:")
        || message.contains("hidden owner-helper startup prompt stream ended");
    if !prompt_transport_failed {
        return Ok(false);
    }

    let store = AgentRuntimeStateStore::new()?;
    let replay_state =
        store.startup_prompt_replay_state(orchestration_session_id, participant_id)?;
    if !replay_state.replay_safe() {
        return Ok(false);
    }
    #[cfg(unix)]
    if pid_is_alive(helper_pid) {
        return Ok(false);
    }
    Ok(matches!(
        replay_state,
        StartupPromptReplayState::NotTracked | StartupPromptReplayState::PendingAcceptance
    ))
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

fn run_turn(args: &AgentTurnArgs, cli: &Cli) -> Result<()> {
    let prompt = load_public_prompt_source(&PublicPromptInput {
        prompt: args.prompt_source.prompt.clone(),
        prompt_file: args.prompt_source.prompt_file.clone(),
    })
    .map_err(normalize_public_prompt_error)?;
    let store = AgentRuntimeStateStore::new()?;
    let target = store
        .resolve_public_turn_target(&args.session, &args.backend)
        .map_err(|err| config_model::user_error(err.to_string()))?;
    let orchestration_session_id = target.session.orchestration_session_id.clone();
    let backend_id = target.participant.handle.backend_id.clone();

    #[cfg(unix)]
    if target.session_posture == PublicSessionPosture::DetachedReattachable
        && target.target_kind == PublicTurnTargetKind::Host
    {
        let mut plan = build_resumed_turn_launch_plan(&args.session)?;
        if plan.descriptor.backend_id != backend_id {
            anyhow::bail!(config_model::user_error(format!(
                "backend_not_in_session: orchestration session {} has no exact backend slot for {}",
                args.session, args.backend
            )));
        }
        let startup_listener = register_hidden_owner_helper_startup_prompt_listener(
            &store,
            plan.orchestration_session_id(),
            plan.participant_id(),
        )
        .map_err(runtime_start_error)?;
        plan.startup_prompt = Some(HiddenOwnerHelperStartupPromptPlan {
            prompt_text: prompt.prompt_text.clone(),
            stream_path: startup_listener.path().to_path_buf(),
        });
        let receipt = launch_hidden_owner_helper(&plan, cli).map_err(runtime_start_error)?;
        run_hidden_owner_helper_startup_prompt_stream_with_action(
            startup_listener,
            args.json,
            PublicPromptAction::Turn,
        )
        .map_err(normalize_public_prompt_error)?;
        wait_for_resumed_public_turn_detach(
            &store,
            &receipt.orchestration_session_id,
            &receipt.participant_id,
        )
        .map_err(runtime_start_error)?;
        return Ok(());
    }

    let resumed_receipt: Option<HiddenOwnerHelperLaunchReceipt> = match target.session_posture {
        PublicSessionPosture::Active => None,
        PublicSessionPosture::DetachedReattachable => match target.target_kind {
            PublicTurnTargetKind::Host => unreachable!("detached host turns are handled above"),
            PublicTurnTargetKind::World => {
                anyhow::bail!(config_model::user_error(format!(
                    "unsupported_platform_or_posture: orchestration session {} backend {} is detached and cannot be recovered through the retained world-member seam without an active host owner; run `substrate agent reattach --session {}` first",
                    args.session,
                    args.backend,
                    args.session
                )));
            }
        },
        PublicSessionPosture::Terminal => {
            anyhow::bail!(config_model::user_error(format!(
                "owner_unreachable: orchestration session {} backend {} is terminal and cannot accept follow-up turns",
                args.session,
                args.backend
            )));
        }
    };
    #[cfg(not(unix))]
    let _ = &resumed_receipt;

    run_public_prompt_command(
        PublicPromptCommandRequest {
            action: PublicPromptAction::Turn,
            orchestration_session_id: Some(orchestration_session_id),
            backend_id,
            prompt,
            json: args.json,
        },
        cli.world,
        cli.no_world,
    )
    .map_err(normalize_public_prompt_error)?;

    #[cfg(unix)]
    if let Some(receipt) = resumed_receipt.as_ref() {
        wait_for_resumed_public_turn_detach(
            &store,
            &receipt.orchestration_session_id,
            &receipt.participant_id,
        )
        .map_err(runtime_start_error)?;
    }

    Ok(())
}

fn run_reattach(args: &AgentSessionControlArgs, cli: &Cli) -> Result<()> {
    let plan = build_attach_launch_plan(&args.session)?;
    let receipt = launch_hidden_owner_helper(&plan, cli).map_err(runtime_start_error)?;
    if receipt.orchestration_session_id != args.session {
        anyhow::bail!(runtime_start_error(anyhow::anyhow!(
            "hidden owner-helper attached orchestration session {} instead of requested session {}",
            receipt.orchestration_session_id,
            args.session
        )));
    }

    let store = AgentRuntimeStateStore::new()?;
    let target = store
        .resolve_public_control_target(&receipt.orchestration_session_id, PublicControlAction::Stop)
        .map_err(runtime_start_error)?;
    if target.session_posture != PublicSessionPosture::Active
        || target.active_participant.handle.participant_id != receipt.participant_id
    {
        anyhow::bail!(runtime_start_error(anyhow::anyhow!(
            "orchestration session {} did not reach durable active_attached ownership after reattach",
            receipt.orchestration_session_id
        )));
    }

    render_agent_control_result(
        args.json,
        &AgentControlResultJson {
            action: "reattach",
            orchestration_session_id: &receipt.orchestration_session_id,
            backend_id: &receipt.backend_id,
            scope: "host",
            state: "active",
            warnings: Vec::new(),
            participant_id: Some(&receipt.participant_id),
            source_orchestration_session_id: None,
        },
    )
}

fn normalize_public_prompt_error(err: anyhow::Error) -> anyhow::Error {
    if config_model::is_user_error(&err) || public_prompt_rendered_exit_code(&err).is_some() {
        return err;
    }

    let message = err.to_string();
    const USER_PREFIXES: &[&str] = &[
        "missing_backend:",
        "unknown_backend:",
        "ambiguous_backend:",
        "missing_prompt_source:",
        "malformed_prompt_source:",
        "empty_prompt:",
        "unknown_session:",
        "noncanonical_session_selector:",
        "backend_not_in_session:",
        "ambiguous_backend_slot:",
        "missing_active_parent:",
        "stale_linkage:",
        "missing_internal_session_id:",
        "session_already_owned:",
        "owner_unreachable:",
        "unsupported_platform_or_posture:",
        "policy_disallow:",
        "stream_bridge_failed:",
        "runtime_start_failed:",
    ];

    if USER_PREFIXES
        .iter()
        .any(|prefix| message.starts_with(prefix))
    {
        return config_model::user_error(message);
    }

    err
}

fn runtime_start_error(err: anyhow::Error) -> anyhow::Error {
    if config_model::is_user_error(&err) {
        return err;
    }
    config_model::user_error(format!("runtime_start_failed: {err}"))
}

fn run_fork(args: &AgentSessionControlArgs, _cli: &Cli) -> Result<()> {
    let allocation = allocate_fork_successor(&args.session)?;
    render_agent_control_result(
        args.json,
        &AgentControlResultJson {
            action: "fork",
            orchestration_session_id: &allocation.orchestration_session_id,
            backend_id: &allocation.backend_id,
            scope: "host",
            state: "parked_resumable",
            warnings: Vec::new(),
            participant_id: None,
            source_orchestration_session_id: Some(&allocation.source_orchestration_session_id),
        },
    )
}

fn run_stop(args: &AgentSessionControlArgs, _cli: &Cli) -> Result<()> {
    let store = AgentRuntimeStateStore::new()?;
    let target = store
        .resolve_public_control_target(&args.session, PublicControlAction::Stop)
        .map_err(|err| config_model::user_error(err.to_string()))?;
    let participant_id = target.active_participant.handle.participant_id.clone();
    let backend_id = target.active_participant.handle.backend_id.clone();
    let orchestration_session_id = target.session.orchestration_session_id.clone();

    if target.session_posture == PublicSessionPosture::DetachedReattachable {
        let mut session = target.session.clone();
        let mut manifest = target.active_participant.clone();
        persist_runtime_stop_closeout(&store, &mut session, &mut manifest)
            .context("failed to persist detached durable stop closeout")?;

        let final_state =
            wait_for_terminal_session_state(&store, orchestration_session_id.as_str())?
                .ok_or_else(|| {
                    config_model::user_error(format!(
                        "owner_unreachable: timed out waiting for orchestration session {} to reach a terminal state",
                        orchestration_session_id
                    ))
                })?;
        return render_agent_control_result(
            args.json,
            &AgentControlResultJson {
                action: "stop",
                orchestration_session_id: orchestration_session_id.as_str(),
                backend_id: &backend_id,
                scope: "host",
                state: final_state.as_str(),
                warnings: Vec::new(),
                participant_id: Some(&participant_id),
                source_orchestration_session_id: None,
            },
        );
    }

    #[cfg(not(unix))]
    {
        let _ = store;
        let _ = target;
        anyhow::bail!(config_model::user_error(
            "unsupported_platform_or_posture: public stop requires a Unix private owner transport"
        ));
    }

    #[cfg(unix)]
    {
        let transport_path = private_stop_transport_path(
            &store,
            orchestration_session_id.as_str(),
            participant_id.as_str(),
        );
        let rt = TokioRuntimeBuilder::new_current_thread()
            .enable_all()
            .build()
            .context("failed to initialize stop transport runtime")?;
        let outcome = rt
            .block_on(async { request_private_stop(&transport_path).await })
            .map_err(|err| config_model::user_error(format!("owner_unreachable: {err}")))?;
        match outcome {
            crate::execution::agent_runtime::control::PrivateStopOutcome::Accepted
            | crate::execution::agent_runtime::control::PrivateStopOutcome::AlreadyTerminal => {}
            crate::execution::agent_runtime::control::PrivateStopOutcome::OwnerUnreachable => {
                anyhow::bail!(config_model::user_error(format!(
                    "owner_unreachable: orchestration session {} no longer has a reachable live owner",
                    args.session
                )));
            }
            crate::execution::agent_runtime::control::PrivateStopOutcome::ProtocolError => {
                anyhow::bail!(config_model::user_error(format!(
                    "owner_unreachable: private owner transport for orchestration session {} returned a protocol error",
                    args.session
                )));
            }
        }

        let final_state = wait_for_terminal_session_state(&store, &args.session)?.ok_or_else(|| {
            config_model::user_error(format!(
                "owner_unreachable: timed out waiting for orchestration session {} to reach a terminal state",
                args.session
            ))
        })?;
        render_agent_control_result(
            args.json,
            &AgentControlResultJson {
                action: "stop",
                orchestration_session_id: &args.session,
                backend_id: &backend_id,
                scope: "host",
                state: final_state.as_str(),
                warnings: Vec::new(),
                participant_id: Some(&participant_id),
                source_orchestration_session_id: None,
            },
        )
    }
}

fn build_start_launch_plan(
    args: &AgentStartArgs,
    context: &AgentCommandContext,
) -> Result<HiddenOwnerHelperLaunchPlan> {
    let backend_id = args.backend.trim();
    if backend_id.is_empty() {
        anyhow::bail!(config_model::user_error(
            "unknown_backend: public start requires --backend <backend_id>"
        ));
    }

    let descriptor = match validate_exact_backend_selection(
        &context.effective_config,
        &context.inventory,
        AgentExecutionScope::Host,
        backend_id,
    ) {
        Ok(Some(descriptor)) => descriptor,
        Ok(None) => {
            if validate_exact_backend_selection(
                &context.effective_config,
                &context.inventory,
                AgentExecutionScope::World,
                backend_id,
            )
            .ok()
            .flatten()
            .is_some()
            {
                anyhow::bail!(config_model::user_error(format!(
                    "unsupported_platform_or_posture: backend '{}' resolves only to a world-scoped runtime; public root start is host-only in v1",
                    backend_id
                )));
            }
            anyhow::bail!(config_model::user_error(format!(
                "unknown_backend: no exact host-scoped backend match found for '{}'",
                backend_id
            )));
        }
        Err(err) => {
            let prefix = if err.reason.contains("ambiguous exact backend selection") {
                "ambiguous_backend"
            } else {
                "unknown_backend"
            };
            let _ = exact_backend_selection_error_exit_code(&err);
            anyhow::bail!(config_model::user_error(format!(
                "{prefix}: {}",
                err.reason
            )));
        }
    };
    if !backend_allowed(&context.base_policy, &descriptor.backend_id) {
        anyhow::bail!(config_model::user_error(format!(
            "policy_disallow: selected orchestrator backend '{}' is not allowlisted by effective policy agents.allowed_backends",
            descriptor.backend_id
        )));
    }

    let orchestration_session_id = Uuid::now_v7().to_string();
    Ok(HiddenOwnerHelperLaunchPlan {
        mode: OwnerHelperMode::Start,
        descriptor: (&descriptor).into(),
        session: HiddenOwnerHelperSessionPlan {
            orchestration_session_id,
            shell_trace_session_id: Uuid::now_v7().to_string(),
            workspace_root: current_dir().display().to_string(),
            world_id: None,
            world_generation: None,
        },
        participant: HiddenOwnerHelperParticipantPlan {
            participant_id: format!("ash_{}", Uuid::now_v7()),
            lease_token: Uuid::now_v7().to_string(),
            run_id: Uuid::now_v7().to_string(),
            resumed_from_participant_id: None,
            internal_uaa_session_id: None,
        },
        startup_prompt: None,
        source_orchestration_session_id: None,
    })
}

fn build_attach_launch_plan(orchestration_session_id: &str) -> Result<HiddenOwnerHelperLaunchPlan> {
    let store = AgentRuntimeStateStore::new()?;
    let target = store
        .resolve_public_control_target(orchestration_session_id, PublicControlAction::Resume)
        .map_err(|err| config_model::user_error(err.to_string()))?;
    let descriptor = target
        .host_attach_contract
        .as_ref()
        .map(|contract| contract.launch_descriptor.clone())
        .ok_or_else(|| {
            config_model::user_error(format!(
                "owner_unreachable: orchestration session {} is missing durable host attach contract state",
                orchestration_session_id
            ))
        })?;

    Ok(HiddenOwnerHelperLaunchPlan {
        mode: OwnerHelperMode::Attach,
        descriptor,
        session: HiddenOwnerHelperSessionPlan {
            orchestration_session_id: target.session.orchestration_session_id.clone(),
            shell_trace_session_id: target.session.shell_trace_session_id.clone(),
            workspace_root: target.session.workspace_root.clone(),
            world_id: target.session.world_id.clone(),
            world_generation: target.session.world_generation,
        },
        participant: HiddenOwnerHelperParticipantPlan {
            participant_id: format!("ash_{}", Uuid::now_v7()),
            lease_token: Uuid::now_v7().to_string(),
            run_id: Uuid::now_v7().to_string(),
            resumed_from_participant_id: Some(
                target.active_participant.handle.participant_id.clone(),
            ),
            internal_uaa_session_id: target
                .host_attach_contract
                .as_ref()
                .and_then(|contract| contract.continuity_uaa_session_id.clone()),
        },
        startup_prompt: None,
        source_orchestration_session_id: None,
    })
}

fn build_resumed_turn_launch_plan(
    orchestration_session_id: &str,
) -> Result<HiddenOwnerHelperLaunchPlan> {
    let mut plan = build_attach_launch_plan(orchestration_session_id)?;
    plan.mode = OwnerHelperMode::ResumeOneTurn;
    Ok(plan)
}

#[derive(Clone, Debug)]
struct ForkSuccessorAllocation {
    orchestration_session_id: String,
    backend_id: String,
    source_orchestration_session_id: String,
}

fn allocate_fork_successor(orchestration_session_id: &str) -> Result<ForkSuccessorAllocation> {
    let store = AgentRuntimeStateStore::new()?;
    let target = store
        .resolve_public_control_target(orchestration_session_id, PublicControlAction::Fork)
        .map_err(|err| config_model::user_error(err.to_string()))?;
    let attach_contract = target.host_attach_contract.clone().ok_or_else(|| {
        config_model::user_error(format!(
            "owner_unreachable: orchestration session {} is missing durable host attach contract state",
            orchestration_session_id
        ))
    })?;
    let descriptor = (&attach_contract.launch_descriptor)
        .try_into()
        .map_err(|err| config_model::user_error(format!("owner_unreachable: {err:#}")))?;
    let successor_session_id = Uuid::now_v7().to_string();
    let successor_participant_id = format!("ash_{}", Uuid::now_v7());
    let lease_token = Uuid::now_v7().to_string();
    let run_id = Uuid::now_v7().to_string();
    let mut successor_participant = AgentRuntimeParticipantRecord::new_replacement_participant(
        &descriptor,
        AgentRuntimeReplacementParticipantInit {
            orchestration_session_id: successor_session_id.clone(),
            participant_id: successor_participant_id.clone(),
            role: ORCHESTRATOR_ROLE.to_string(),
            orchestrator_participant_id: None,
            parent_participant_id: None,
            resumed_from_participant_id: target.active_participant.handle.participant_id.clone(),
            world: None,
            lease_token,
        },
    )
    .context("failed to allocate fork successor participant")?;
    successor_participant
        .transition_state(crate::execution::agent_runtime::AgentRuntimeSessionState::Ready);
    successor_participant.internal.latest_run_id = Some(run_id.clone());
    successor_participant.internal.shell_owner_pid = 0;
    successor_participant.internal.uaa_session_id = None;
    successor_participant.internal.control_owner_retained = false;
    successor_participant.internal.event_stream_active = false;
    successor_participant.internal.completion_observer_retained = false;
    successor_participant.internal.ownership_valid = false;
    successor_participant.internal.ownership_verified_at = None;
    successor_participant.internal.attached_client_present = false;
    successor_participant.internal.last_attached_at = None;
    successor_participant.internal.last_detached_at = None;
    successor_participant.internal.detach_reason = None;
    successor_participant.internal.resume_eligible = false;
    successor_participant.touch_heartbeat();

    let mut successor_session = OrchestrationSessionRecord::new(
        successor_session_id.clone(),
        Uuid::now_v7().to_string(),
        target.session.workspace_root.clone(),
        &successor_participant,
    );
    successor_session.shell_owner_pid = 0;
    successor_session.latest_run_id = Some(run_id);
    successor_session.world_id = target.session.world_id.clone();
    successor_session.world_generation = target.session.world_generation;
    successor_session.host_attach_contract = target.session.fork_successor_attach_contract();
    successor_session
        .transition_state(crate::execution::agent_runtime::OrchestrationSessionState::Active);
    successor_session.active_session_handle_id = Some(successor_participant_id);
    successor_session.pending_inbox_count = 0;
    successor_session.last_attention_at = None;
    successor_session
        .mark_parked_resumable("fork successor allocated without attached host client");

    store.persist_orchestration_session(&successor_session)?;
    store.persist_participant(&successor_participant)?;

    Ok(ForkSuccessorAllocation {
        orchestration_session_id: successor_session_id,
        backend_id: attach_contract.backend_id,
        source_orchestration_session_id: target.session.orchestration_session_id,
    })
}

#[cfg(unix)]
fn wait_for_resumed_public_turn_detach(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    resumed_participant_id: &str,
) -> Result<()> {
    let started_at = std::time::Instant::now();
    loop {
        if store
            .resumed_public_turn_detach_posture(orchestration_session_id, resumed_participant_id)?
            .is_some()
        {
            return Ok(());
        }

        let Some(record) = store.load_session(orchestration_session_id)? else {
            anyhow::bail!(
                "owner_unreachable: orchestration session {} disappeared before the resumed public turn could detach cleanly",
                orchestration_session_id
            );
        };
        if record.session.state.is_terminal()
            || record.session.posture == OrchestrationSessionPosture::Terminal
        {
            anyhow::bail!(
                "owner_unreachable: orchestration session {} became terminal before resumed participant {} could repark",
                orchestration_session_id,
                resumed_participant_id
            );
        }
        if record
            .session
            .active_participant_id()
            .is_some_and(|participant_id| participant_id != resumed_participant_id)
        {
            anyhow::bail!(
                "owner_unreachable: orchestration session {} reassigned active ownership to {} before resumed participant {} could repark",
                orchestration_session_id,
                record.session.active_participant_id().unwrap_or_default(),
                resumed_participant_id
            );
        }

        if started_at.elapsed() >= TURN_DETACH_NORMALIZATION_TIMEOUT {
            anyhow::bail!(
                "owner_unreachable: timed out waiting for orchestration session {} resumed participant {} to converge back to detached durable continuity",
                orchestration_session_id,
                resumed_participant_id
            );
        }

        thread::sleep(TURN_DETACH_NORMALIZATION_POLL_INTERVAL);
    }
}

fn wait_for_terminal_session_state(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
) -> Result<Option<OrchestrationSessionStateLabel>> {
    let started_at = std::time::Instant::now();
    loop {
        if let Some(state) = terminal_session_state(store, orchestration_session_id)? {
            return Ok(Some(state));
        }
        if started_at.elapsed() >= AGENT_CONTROL_STOP_WAIT_TIMEOUT {
            return Ok(None);
        }
        thread::sleep(AGENT_CONTROL_STOP_WAIT_POLL_INTERVAL);
    }
}

fn terminal_session_state(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
) -> Result<Option<OrchestrationSessionStateLabel>> {
    Ok(store
        .list_sessions()?
        .into_iter()
        .find(|record| record.session.orchestration_session_id == orchestration_session_id)
        .and_then(|record| {
            OrchestrationSessionStateLabel::from_session_state(&record.session.state)
        }))
}

#[derive(Clone, Copy)]
enum OrchestrationSessionStateLabel {
    Stopped,
    Invalidated,
    Failed,
}

impl OrchestrationSessionStateLabel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Invalidated => "invalidated",
            Self::Failed => "failed",
        }
    }

    fn from_session_state(
        state: &crate::execution::agent_runtime::OrchestrationSessionState,
    ) -> Option<Self> {
        match state {
            crate::execution::agent_runtime::OrchestrationSessionState::Stopped => {
                Some(Self::Stopped)
            }
            crate::execution::agent_runtime::OrchestrationSessionState::Invalidated => {
                Some(Self::Invalidated)
            }
            crate::execution::agent_runtime::OrchestrationSessionState::Failed => {
                Some(Self::Failed)
            }
            _ => None,
        }
    }
}

fn render_agent_control_result(json_mode: bool, result: &AgentControlResultJson<'_>) -> Result<()> {
    if json_mode {
        println!("{}", serde_json::to_string_pretty(result)?);
        return Ok(());
    }

    let mut line = format!(
        "{} {} [{}] -> {}",
        result.action, result.orchestration_session_id, result.backend_id, result.state
    );
    if let Some(participant_id) = result.participant_id {
        line.push_str(&format!(" participant={participant_id}"));
    }
    if let Some(source) = result.source_orchestration_session_id {
        line.push_str(&format!(" source={source}"));
    }
    println!("{line}");
    Ok(())
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
    #[serde(skip_serializing_if = "Option::is_none")]
    participant_id: Option<String>,
    agent_id: String,
    source_kind: &'static str,
    backend_id: String,
    client: String,
    router: String,
    protocol: String,
    execution: ExecutionScopeJson<'static>,
    role: Option<String>,
    posture: Option<String>,
    attached_participant_id: Option<String>,
    pending_inbox_count: Option<u64>,
    last_event_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    world_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    world_generation: Option<u64>,
}

#[derive(Clone, Serialize)]
struct NestedParentJson {
    orchestration_session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    participant_id: Option<String>,
    agent_id: String,
}

#[derive(Clone, Serialize)]
struct NestedLlmRecordJson {
    parent: NestedParentJson,
    run_id: String,
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
    warnings: Vec<String>,
    sessions: Vec<StatusSessionJson>,
    nested_llm_records: Vec<NestedLlmRecordJson>,
}

#[derive(Clone)]
struct SessionProjection {
    last_event_ts: DateTime<Utc>,
    session: StatusSessionJson,
    source: SessionProjectionSource,
}

#[derive(Clone)]
struct SessionProjectionSource {
    identity: StatusIdentityKey,
    run_id: Option<String>,
    ts: DateTime<Utc>,
    is_world_scoped: bool,
    has_top_level_world_id: bool,
    has_top_level_world_generation: bool,
}

#[derive(Clone)]
struct SelectedParentRun {
    participant_id: Option<String>,
    run_id: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct StatusIdentityKey {
    orchestration_session_id: String,
    agent_id: String,
    execution_scope: &'static str,
    participant_id: Option<String>,
}

impl StatusIdentityKey {
    fn coarse(&self) -> Self {
        Self {
            orchestration_session_id: self.orchestration_session_id.clone(),
            agent_id: self.agent_id.clone(),
            execution_scope: self.execution_scope,
            participant_id: None,
        }
    }
}

#[derive(Clone)]
struct NestedProjection {
    sort_key: (String, String, String),
    source: NestedProjectionSource,
    backend_id: String,
    client: String,
    protocol: String,
    provider: Option<String>,
    auth_authority: Option<String>,
}

#[derive(Clone)]
struct NestedProjectionSource {
    parent_identity: StatusIdentityKey,
    run_id: String,
    parent_run_id: Option<String>,
    parent_participant_id: Option<String>,
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
            warnings: Vec::new(),
            sessions: Vec::new(),
            nested_llm_records: Vec::new(),
        });
    }

    let events = read_trace_agent_events()?;
    let state_store = AgentRuntimeStateStore::new()?;
    let status_sessions = state_store.list_status_sessions_for_agent(&orchestrator_agent_id)?;
    let mut warnings = status_sessions
        .iter()
        .flat_map(|session| session.warnings.iter().cloned())
        .collect::<BTreeSet<_>>();
    let mut sessions = BTreeMap::<StatusIdentityKey, SessionProjection>::new();
    let mut nested = BTreeMap::<(String, String, String), NestedProjection>::new();
    let mut historical_parent_runs_exact = BTreeMap::<StatusIdentityKey, BTreeSet<String>>::new();
    let mut historical_parent_runs_coarse = BTreeMap::<StatusIdentityKey, BTreeSet<String>>::new();

    for event in events {
        let Some(entry) = context.inventory.get(&event.agent_id) else {
            continue;
        };
        let is_selected_orchestrator =
            context.effective_config.agents.hub.orchestrator_agent_id == entry.file.id;
        let role = role_for_event(
            &event,
            &entry.file.id,
            is_selected_orchestrator,
            &context.effective_config,
        );
        let scope = scope_for_event(
            &event,
            entry,
            is_selected_orchestrator,
            &context.effective_config,
        );

        if let Some(identity) = pure_session_key(&event, scope) {
            historical_parent_runs_exact
                .entry(identity.clone())
                .or_default()
                .insert(event.run_id.clone());
            historical_parent_runs_coarse
                .entry(identity.coarse())
                .or_default()
                .insert(event.run_id.clone());
            let world_id = event.world_id.clone();
            let world_generation = event.world_generation;

            let projection = SessionProjection {
                last_event_ts: event.ts,
                session: StatusSessionJson {
                    orchestration_session_id: identity.orchestration_session_id.clone(),
                    participant_id: identity.participant_id.clone(),
                    agent_id: entry.file.id.clone(),
                    source_kind: "trace_fallback",
                    backend_id: entry.derived_backend_id(),
                    client: entry.file.id.clone(),
                    router: PURE_AGENT_ROUTER.to_string(),
                    protocol: PURE_AGENT_PROTOCOL.to_string(),
                    execution: ExecutionScopeJson {
                        scope: scope.as_str(),
                    },
                    role: role.map(ToOwned::to_owned),
                    posture: None,
                    attached_participant_id: None,
                    pending_inbox_count: None,
                    last_event_at: event.ts.to_rfc3339(),
                    world_id: if scope == AgentExecutionScope::World {
                        world_id.clone()
                    } else {
                        None
                    },
                    world_generation: if scope == AgentExecutionScope::World {
                        world_generation
                    } else {
                        None
                    },
                },
                source: SessionProjectionSource {
                    identity: identity.clone(),
                    run_id: Some(event.run_id.clone()),
                    ts: event.ts,
                    is_world_scoped: scope == AgentExecutionScope::World,
                    has_top_level_world_id: world_id.is_some(),
                    has_top_level_world_generation: world_generation.is_some(),
                },
            };

            let should_replace = match sessions.get(&identity) {
                Some(existing) => projection.last_event_ts >= existing.last_event_ts,
                None => true,
            };
            if should_replace {
                sessions.insert(identity, projection);
            }
        }

        if let Some(projection) = nested_projection(&event, entry, scope) {
            nested.insert(projection.sort_key.clone(), projection);
        }
    }

    let mut filtered_session_projections = Vec::new();
    for projection in sessions.into_values() {
        if !matches_scope(
            scope_from_label(projection.session.execution.scope),
            args.scope,
        ) || !matches_role(projection.session.role.as_deref(), role_filter)
        {
            continue;
        }

        if projection.source.is_world_scoped
            && (!projection.source.has_top_level_world_id
                || !projection.source.has_top_level_world_generation)
        {
            return Err(config_model::user_error(format!(
                "malformed world identity on newest selected world-scoped pure-agent status event: agent_id={} orchestration_session_id={} run_id={} ts={} requires top-level world_id and world_generation",
                projection.source.identity.agent_id,
                projection.source.identity.orchestration_session_id,
                projection.source.run_id.as_deref().unwrap_or("<missing>"),
                projection.source.ts.to_rfc3339(),
            )));
        }

        filtered_session_projections.push(projection);
    }

    let mut selected_session_projections = Vec::new();
    for status_session in &status_sessions {
        for projection in live_session_status_projections(status_session) {
            if !matches_scope(
                scope_from_label(projection.session.execution.scope),
                args.scope,
            ) || !matches_role(projection.session.role.as_deref(), role_filter)
            {
                continue;
            }
            selected_session_projections.push(projection);
        }
    }

    let live_fallback_suppression_exact = selected_session_projections
        .iter()
        .map(session_fallback_suppression_key)
        .collect::<BTreeSet<_>>();
    let live_fallback_suppression_coarse = live_fallback_suppression_exact
        .iter()
        .map(StatusIdentityKey::coarse)
        .collect::<BTreeSet<_>>();
    let invalidated_fallback_suppression_exact = state_store
        .list_invalidated_participants_across_sources()?
        .into_iter()
        .filter(|participant| {
            participant.handle.role == MEMBER_ROLE
                && participant.handle.execution.scope == AgentExecutionScope::World
        })
        .map(participant_fallback_suppression_key)
        .collect::<BTreeSet<_>>();
    let invalidated_fallback_suppression_coarse = invalidated_fallback_suppression_exact
        .iter()
        .map(StatusIdentityKey::coarse)
        .collect::<BTreeSet<_>>();
    let coarse_trace_identities = filtered_session_projections
        .iter()
        .filter(|projection| projection.source.identity.participant_id.is_none())
        .map(|projection| projection.source.identity.coarse())
        .collect::<BTreeSet<_>>();
    for coarse_identity in coarse_trace_identities {
        let participant_scoped_count = filtered_session_projections
            .iter()
            .filter(|projection| projection.source.identity.coarse() == coarse_identity)
            .filter(|projection| projection.source.identity.participant_id.is_some())
            .count();
        if participant_scoped_count > 0 {
            warnings.insert(format!(
                "Status fallback for agent {} in orchestration session {} includes participant-less trace rows, so fallback selection remains coarse until participant-scoped records are available.",
                coarse_identity.agent_id,
                coarse_identity.orchestration_session_id,
            ));
        }
    }
    selected_session_projections.extend(filtered_session_projections.into_iter().filter(
        |projection| {
            let suppression_key = session_fallback_suppression_key(projection);
            if suppression_key.participant_id.is_some() {
                !live_fallback_suppression_exact.contains(&suppression_key)
                    && !invalidated_fallback_suppression_exact.contains(&suppression_key)
            } else {
                let coarse_key = suppression_key.coarse();
                !live_fallback_suppression_coarse.contains(&coarse_key)
                    && !invalidated_fallback_suppression_coarse.contains(&coarse_key)
            }
        },
    ));

    let mut selected_parent_runs_exact =
        BTreeMap::<StatusIdentityKey, Vec<SelectedParentRun>>::new();
    let mut selected_parent_runs_coarse =
        BTreeMap::<StatusIdentityKey, Vec<SelectedParentRun>>::new();
    for projection in &selected_session_projections {
        let Some(run_id) = projection.source.run_id.as_ref() else {
            continue;
        };
        let selected_parent_run = SelectedParentRun {
            participant_id: projection.source.identity.participant_id.clone(),
            run_id: run_id.clone(),
        };
        selected_parent_runs_exact
            .entry(projection.source.identity.clone())
            .or_default()
            .push(selected_parent_run.clone());
        selected_parent_runs_coarse
            .entry(projection.source.identity.coarse())
            .or_default()
            .push(selected_parent_run);
    }

    let mut filtered_nested = Vec::new();
    for projection in nested.into_values() {
        let selected_parent_runs = selected_parent_run_candidates(
            &projection,
            &selected_parent_runs_exact,
            &selected_parent_runs_coarse,
        );
        if selected_parent_runs.is_empty() {
            continue;
        }
        if projection.source.parent_participant_id.is_none()
            && selected_parent_runs
                .iter()
                .filter_map(|candidate| candidate.participant_id.as_deref())
                .collect::<BTreeSet<_>>()
                .len()
                > 1
        {
            warnings.insert(format!(
                "Nested status correlation for agent {} in orchestration session {} is using coarse parent matching because parent_participant_id is absent.",
                projection.source.parent_identity.agent_id,
                projection.source.parent_identity.orchestration_session_id,
            ));
        }
        let parent_run_id = projection.source.parent_run_id.as_deref();
        if let Some(selected_parent_run) = selected_parent_runs
            .iter()
            .find(|candidate| parent_run_id == Some(candidate.run_id.as_str()))
        {
            let missing_fields = missing_required_nested_fields(&projection);
            if !missing_fields.is_empty() {
                return Err(config_model::user_error(format!(
                    "malformed nested tuple on selected status surface: agent_id={} orchestration_session_id={} run_id={} missing_fields={} requires provider and auth_authority on selected nested substrate_gateway status rows",
                    projection.source.parent_identity.agent_id,
                    projection.source.parent_identity.orchestration_session_id,
                    projection.source.run_id,
                    missing_fields.join(","),
                )));
            }

            filtered_nested.push(NestedLlmRecordJson {
                parent: NestedParentJson {
                    orchestration_session_id: projection
                        .source
                        .parent_identity
                        .orchestration_session_id
                        .clone(),
                    participant_id: selected_parent_run.participant_id.clone(),
                    agent_id: projection.source.parent_identity.agent_id.clone(),
                },
                run_id: projection.source.run_id.clone(),
                backend_id: projection.backend_id,
                client: projection.client,
                router: NESTED_ROUTER.to_string(),
                provider: projection
                    .provider
                    .expect("missing required nested provider already validated"),
                auth_authority: projection
                    .auth_authority
                    .expect("missing required nested auth_authority already validated"),
                protocol: projection.protocol,
            });
            continue;
        }

        let invalid_parent_run_id = format_invalid_parent_run_id(parent_run_id);
        let historical_match = historical_parent_run_matches(
            &projection.source.parent_identity,
            parent_run_id,
            &historical_parent_runs_exact,
            &historical_parent_runs_coarse,
        );
        if historical_match {
            continue;
        }

        return Err(config_model::user_error(format!(
            "malformed nested parent correlation on selected status surface: agent_id={} orchestration_session_id={} run_id={} parent_run_id={} requires parent_run_id to match the winning selected pure-agent run or a known historical pure-agent run for the same session",
            projection.source.parent_identity.agent_id,
            projection.source.parent_identity.orchestration_session_id,
            projection.source.run_id,
            invalid_parent_run_id,
        )));
    }

    let mut filtered_sessions: Vec<StatusSessionJson> = selected_session_projections
        .into_iter()
        .map(|projection| projection.session)
        .collect();
    filtered_sessions.sort_by(|left, right| {
        left.orchestration_session_id
            .cmp(&right.orchestration_session_id)
            .then(
                left.participant_id
                    .as_deref()
                    .unwrap_or("")
                    .cmp(right.participant_id.as_deref().unwrap_or("")),
            )
            .then(left.agent_id.cmp(&right.agent_id))
            .then(left.source_kind.cmp(right.source_kind))
    });

    Ok(StatusReportJson {
        disabled: false,
        scope_filter: args.scope.as_str(),
        role_filter,
        orchestrator_agent_id,
        warnings: warnings.into_iter().collect(),
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
    if !report.warnings.is_empty() {
        println!("warnings:");
        for warning in &report.warnings {
            println!("  - {warning}");
        }
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
            format!(
                "participant_id={}",
                session.participant_id.as_deref().unwrap_or("")
            ),
            format!("agent_id={}", session.agent_id),
            format!("source_kind={}", session.source_kind),
            format!("backend_id={}", session.backend_id),
            format!("client={}", session.client),
            format!("router={}", session.router),
            format!("protocol={}", session.protocol),
            format!("execution.scope={}", session.execution.scope),
            format!("role={}", session.role.as_deref().unwrap_or("")),
            format!(
                "posture={}",
                human_status_option_str(session.posture.as_deref(), session.source_kind)
            ),
            format!(
                "attached_participant_id={}",
                human_status_option_str(
                    session.attached_participant_id.as_deref(),
                    session.source_kind
                )
            ),
            format!(
                "pending_inbox_count={}",
                human_status_option_u64(session.pending_inbox_count, session.source_kind)
            ),
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
                "  parent.orchestration_session_id={} | parent.participant_id={} | parent.agent_id={} | run_id={} | backend_id={} | client={} | router={} | provider={} | auth_authority={} | protocol={}",
                record.parent.orchestration_session_id,
                record.parent.participant_id.as_deref().unwrap_or(""),
                record.parent.agent_id,
                record.run_id,
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

fn run_toolbox_status(args: &AgentToolboxViewArgs, cli: &Cli) -> Result<i32> {
    let context = resolve_command_context(cli)?;
    let report = build_toolbox_status_report(&context)?;
    render_toolbox_status_report(&report, args.json)?;
    Ok(0)
}

fn run_toolbox_env(args: &AgentToolboxViewArgs, cli: &Cli) -> Result<i32> {
    let context = resolve_command_context(cli)?;
    let status = build_toolbox_status_report(&context)?;
    match status.eligibility.state.as_str() {
        "allowed" => {
            let report = build_toolbox_env_report(&status)?;
            render_toolbox_env_report(&report, args.json)?;
            Ok(0)
        }
        "disabled" => {
            eprintln!(
                "{}",
                status
                    .eligibility
                    .reason
                    .as_deref()
                    .unwrap_or("toolbox is disabled by the effective config")
            );
            Ok(2)
        }
        "denied" => {
            eprintln!(
                "{}",
                status
                    .eligibility
                    .reason
                    .as_deref()
                    .unwrap_or("toolbox access is denied by effective policy")
            );
            Ok(5)
        }
        "unsupported" => {
            eprintln!(
                "{}",
                status
                    .eligibility
                    .reason
                    .as_deref()
                    .unwrap_or("toolbox transport is unsupported")
            );
            Ok(4)
        }
        "dependency_unavailable" => {
            eprintln!(
                "{}",
                status
                    .eligibility
                    .reason
                    .as_deref()
                    .unwrap_or("toolbox environment hints require an active orchestrator session",)
            );
            Ok(3)
        }
        other => Err(anyhow::anyhow!(
            "unexpected toolbox eligibility state '{other}'"
        )),
    }
}

#[derive(Clone, Serialize)]
struct ToolboxOrchestratorJson<'a> {
    agent_id: String,
    backend_id: String,
    role: &'a str,
    execution: ExecutionScopeJson<'a>,
}

#[derive(Clone, Serialize)]
struct ToolboxEligibilityJson {
    state: String,
    reason: Option<String>,
}

#[derive(Clone, Serialize)]
struct ToolboxActiveWorldBindingJson {
    world_id: String,
    world_generation: u64,
}

#[derive(Clone, Serialize)]
struct ToolboxStatusReportJson<'a> {
    toolbox_enabled: bool,
    toolbox_version: u32,
    transport: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    endpoint_template: Option<String>,
    active_orchestration_session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    active_world_binding: Option<ToolboxActiveWorldBindingJson>,
    eligibility: ToolboxEligibilityJson,
    #[serde(skip_serializing_if = "Option::is_none")]
    orchestrator: Option<ToolboxOrchestratorJson<'a>>,
}

#[derive(Serialize)]
struct ToolboxEnvReportJson {
    #[serde(rename = "SUBSTRATE_AGENT_TOOLBOX_ENDPOINT")]
    substrate_agent_toolbox_endpoint: String,
    #[serde(rename = "SUBSTRATE_AGENT_TOOLBOX_VERSION")]
    substrate_agent_toolbox_version: String,
}

fn build_toolbox_status_report<'a>(
    context: &'a AgentCommandContext,
) -> Result<ToolboxStatusReportJson<'a>> {
    let transport = toolbox_transport_label(context.effective_config.agents.toolbox.bind.transport);

    if !context.effective_config.agents.enabled {
        return Ok(ToolboxStatusReportJson {
            toolbox_enabled: false,
            toolbox_version: TOOLBOX_VERSION,
            transport,
            endpoint: None,
            endpoint_template: None,
            active_orchestration_session_id: None,
            active_world_binding: None,
            eligibility: ToolboxEligibilityJson {
                state: "disabled".to_string(),
                reason: Some("agents are disabled by effective config".to_string()),
            },
            orchestrator: None,
        });
    }

    let orchestrator =
        validate_orchestrator_selection(&context.effective_config, &context.inventory)
            .map_err(config_model::user_error)?;
    let orchestrator_report = ToolboxOrchestratorJson {
        agent_id: orchestrator.file.id.clone(),
        backend_id: orchestrator.derived_backend_id(),
        role: ORCHESTRATOR_ROLE,
        execution: ExecutionScopeJson { scope: "host" },
    };

    if !context.effective_config.agents.toolbox.enabled {
        return Ok(ToolboxStatusReportJson {
            toolbox_enabled: false,
            toolbox_version: TOOLBOX_VERSION,
            transport,
            endpoint: None,
            endpoint_template: None,
            active_orchestration_session_id: None,
            active_world_binding: None,
            eligibility: ToolboxEligibilityJson {
                state: "disabled".to_string(),
                reason: Some("agents.toolbox.enabled is false in the effective config".to_string()),
            },
            orchestrator: Some(orchestrator_report),
        });
    }

    if !backend_allowed(&context.base_policy, &orchestrator_report.backend_id) {
        return Ok(ToolboxStatusReportJson {
            toolbox_enabled: true,
            toolbox_version: TOOLBOX_VERSION,
            transport,
            endpoint: None,
            endpoint_template: None,
            active_orchestration_session_id: None,
            active_world_binding: None,
            eligibility: ToolboxEligibilityJson {
                state: "denied".to_string(),
                reason: Some(format!(
                    "selected orchestrator backend '{}' is not allowlisted by effective policy agents.allowed_backends",
                    orchestrator_report.backend_id
                )),
            },
            orchestrator: Some(orchestrator_report),
        });
    }

    match context.effective_config.agents.toolbox.bind.transport {
        AgentToolboxBindTransport::Tcp => Ok(ToolboxStatusReportJson {
            toolbox_enabled: true,
            toolbox_version: TOOLBOX_VERSION,
            transport,
            endpoint: None,
            endpoint_template: None,
            active_orchestration_session_id: None,
            active_world_binding: None,
            eligibility: ToolboxEligibilityJson {
                state: "unsupported".to_string(),
                reason: Some(
                    "toolbox TCP transport is not yet supported because no deterministic pre-runtime loopback port contract exists"
                        .to_string(),
                ),
            },
            orchestrator: Some(orchestrator_report),
        }),
        AgentToolboxBindTransport::Uds => {
            let endpoint_template = Some(toolbox_uds_endpoint_template()?);
            let latest_session = AgentRuntimeStateStore::new()?
                .resolve_single_live_session_for_agent(&orchestrator.file.id)
                .map_err(|err| config_model::user_error(err.to_string()))?;

            match latest_session {
                Some(session_record) => Ok(ToolboxStatusReportJson {
                    toolbox_enabled: true,
                    toolbox_version: TOOLBOX_VERSION,
                    transport,
                    endpoint: Some(toolbox_uds_endpoint(
                        session_record.orchestration_session_id(),
                    )?),
                    endpoint_template,
                    active_orchestration_session_id: Some(
                        session_record.orchestration_session_id().to_string(),
                    ),
                    active_world_binding: toolbox_active_world_binding(&session_record.session),
                    eligibility: ToolboxEligibilityJson {
                        state: "allowed".to_string(),
                        reason: None,
                    },
                    orchestrator: Some(orchestrator_report),
                }),
                None => Ok(ToolboxStatusReportJson {
                    toolbox_enabled: true,
                    toolbox_version: TOOLBOX_VERSION,
                    transport,
                    endpoint: None,
                    endpoint_template,
                    active_orchestration_session_id: None,
                    active_world_binding: None,
                    eligibility: ToolboxEligibilityJson {
                        state: "dependency_unavailable".to_string(),
                        reason: Some(
                            "no live host-scoped orchestrator participant found for the selected orchestrator"
                                .to_string(),
                        ),
                    },
                    orchestrator: Some(orchestrator_report),
                }),
            }
        }
    }
}

fn render_toolbox_status_report(
    report: &ToolboxStatusReportJson<'_>,
    json_mode: bool,
) -> Result<()> {
    if json_mode {
        println!("{}", serde_json::to_string_pretty(report)?);
        return Ok(());
    }

    println!("toolbox_enabled: {}", report.toolbox_enabled);
    println!("toolbox_version: {}", report.toolbox_version);
    println!("transport: {}", report.transport);
    let eligibility = if let Some(reason) = &report.eligibility.reason {
        format!("{}: {}", report.eligibility.state, reason)
    } else {
        report.eligibility.state.clone()
    };
    println!("eligibility: {eligibility}");

    if let Some(session_id) = &report.active_orchestration_session_id {
        println!("active_orchestration_session_id: {session_id}");
    }
    if let Some(endpoint) = &report.endpoint {
        println!("endpoint: {endpoint}");
    }
    if let Some(endpoint_template) = &report.endpoint_template {
        println!("endpoint_template: {endpoint_template}");
    }
    if let Some(binding) = &report.active_world_binding {
        println!(
            "active_world_binding: world_id={} | world_generation={}",
            binding.world_id, binding.world_generation
        );
    }
    if let Some(orchestrator) = &report.orchestrator {
        println!(
            "orchestrator: agent_id={} | backend_id={} | role={} | execution.scope={}",
            orchestrator.agent_id,
            orchestrator.backend_id,
            orchestrator.role,
            orchestrator.execution.scope
        );
    }

    Ok(())
}

fn toolbox_active_world_binding(
    session: &crate::execution::agent_runtime::OrchestrationSessionRecord,
) -> Option<ToolboxActiveWorldBindingJson> {
    let world_id = session.world_id.as_ref()?;
    let world_generation = session.world_generation?;
    if world_id.trim().is_empty() {
        return None;
    }

    Some(ToolboxActiveWorldBindingJson {
        world_id: world_id.clone(),
        world_generation,
    })
}

fn build_toolbox_env_report(report: &ToolboxStatusReportJson<'_>) -> Result<ToolboxEnvReportJson> {
    Ok(ToolboxEnvReportJson {
        substrate_agent_toolbox_endpoint: report
            .endpoint
            .clone()
            .ok_or_else(|| anyhow::anyhow!("missing toolbox endpoint for allowed session"))?,
        substrate_agent_toolbox_version: TOOLBOX_VERSION.to_string(),
    })
}

fn render_toolbox_env_report(report: &ToolboxEnvReportJson, json_mode: bool) -> Result<()> {
    if json_mode {
        println!("{}", serde_json::to_string_pretty(report)?);
        return Ok(());
    }

    println!(
        "export SUBSTRATE_AGENT_TOOLBOX_ENDPOINT={}",
        shell_single_quote(&report.substrate_agent_toolbox_endpoint)
    );
    println!(
        "export SUBSTRATE_AGENT_TOOLBOX_VERSION={}",
        shell_single_quote(&report.substrate_agent_toolbox_version)
    );
    Ok(())
}

fn toolbox_transport_label(transport: AgentToolboxBindTransport) -> &'static str {
    match transport {
        AgentToolboxBindTransport::Uds => "uds",
        AgentToolboxBindTransport::Tcp => "tcp",
    }
}

fn toolbox_uds_endpoint(orchestration_session_id: &str) -> Result<String> {
    Ok(format!(
        "unix://{}",
        substrate_paths::substrate_home()?
            .join("run")
            .join("agent-toolbox")
            .join(format!("{orchestration_session_id}.sock"))
            .display()
    ))
}

fn toolbox_uds_endpoint_template() -> Result<String> {
    Ok(format!(
        "unix://{}",
        substrate_paths::substrate_home()?
            .join("run")
            .join("agent-toolbox")
            .join("<orchestration_session_id>.sock")
            .display()
    ))
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn live_participant_status_projection(
    session: &OrchestrationSessionRecord,
    participant: &AgentRuntimeParticipantRecord,
) -> SessionProjection {
    SessionProjection {
        last_event_ts: participant.last_status_at(),
        session: StatusSessionJson {
            orchestration_session_id: participant.handle.orchestration_session_id.clone(),
            participant_id: Some(participant.handle.participant_id.clone()),
            agent_id: participant.handle.agent_id.clone(),
            source_kind: "live_runtime",
            backend_id: participant.handle.backend_id.clone(),
            client: participant.handle.agent_id.clone(),
            router: PURE_AGENT_ROUTER.to_string(),
            protocol: participant.handle.protocol.clone(),
            execution: ExecutionScopeJson {
                scope: match participant.handle.execution.scope {
                    AgentExecutionScope::Host => "host",
                    AgentExecutionScope::World => "world",
                },
            },
            role: Some(participant.handle.role.clone()),
            posture: Some(orchestration_session_posture_label(session.posture).to_string()),
            attached_participant_id: session.attached_participant_id.clone(),
            pending_inbox_count: Some(session.pending_inbox_count),
            last_event_at: participant.last_status_at().to_rfc3339(),
            world_id: participant.handle.world_id.clone(),
            world_generation: participant.handle.world_generation,
        },
        source: SessionProjectionSource {
            identity: StatusIdentityKey {
                orchestration_session_id: participant.handle.orchestration_session_id.clone(),
                agent_id: participant.handle.agent_id.clone(),
                execution_scope: execution_scope_label(participant.handle.execution.scope),
                participant_id: Some(participant.handle.participant_id.clone()),
            },
            run_id: participant.internal.latest_run_id.clone(),
            ts: participant.last_status_at(),
            is_world_scoped: participant.handle.execution.scope == AgentExecutionScope::World,
            has_top_level_world_id: participant.handle.world_id.is_some(),
            has_top_level_world_generation: participant.handle.world_generation.is_some(),
        },
    }
}

fn live_session_status_projections(session: &AgentRuntimeSessionRecord) -> Vec<SessionProjection> {
    session
        .status_visible_participants()
        .into_iter()
        .map(|participant| live_participant_status_projection(&session.session, &participant))
        .collect()
}

fn orchestration_session_posture_label(posture: OrchestrationSessionPosture) -> &'static str {
    match posture {
        OrchestrationSessionPosture::ActiveAttached => "active_attached",
        OrchestrationSessionPosture::ParkedResumable => "parked_resumable",
        OrchestrationSessionPosture::AwaitingAttention => "awaiting_attention",
        OrchestrationSessionPosture::Terminal => "terminal",
    }
}

fn human_status_option_str(value: Option<&str>, source_kind: &str) -> String {
    match value {
        Some(value) => value.to_string(),
        None if source_kind == "trace_fallback" => "<unknown>".to_string(),
        None => "<none>".to_string(),
    }
}

fn human_status_option_u64(value: Option<u64>, source_kind: &str) -> String {
    match value {
        Some(value) => value.to_string(),
        None if source_kind == "trace_fallback" => "<unknown>".to_string(),
        None => "<none>".to_string(),
    }
}

fn session_fallback_suppression_key(projection: &SessionProjection) -> StatusIdentityKey {
    projection.source.identity.clone()
}

fn participant_fallback_suppression_key(
    participant: AgentRuntimeParticipantRecord,
) -> StatusIdentityKey {
    StatusIdentityKey {
        orchestration_session_id: participant.handle.orchestration_session_id,
        agent_id: participant.handle.agent_id,
        execution_scope: execution_scope_label(participant.handle.execution.scope),
        participant_id: Some(participant.handle.participant_id),
    }
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

fn pure_session_key(event: &AgentEvent, scope: AgentExecutionScope) -> Option<StatusIdentityKey> {
    let tuple = event.identity_tuple.as_ref()?;
    if tuple.router != PURE_AGENT_ROUTER || tuple.protocol != PURE_AGENT_PROTOCOL {
        return None;
    }
    Some(StatusIdentityKey {
        orchestration_session_id: event.orchestration_session_id.clone(),
        agent_id: event.agent_id.clone(),
        execution_scope: execution_scope_label(scope),
        participant_id: event.participant_id.clone(),
    })
}

fn nested_projection(
    event: &AgentEvent,
    entry: &AgentInventoryEntryV1,
    scope: AgentExecutionScope,
) -> Option<NestedProjection> {
    let tuple = event.identity_tuple.as_ref()?;
    if tuple.router != NESTED_ROUTER {
        return None;
    }
    let sort_key = (
        event.orchestration_session_id.clone(),
        event.agent_id.clone(),
        event.run_id.clone(),
    );

    Some(NestedProjection {
        sort_key,
        source: NestedProjectionSource {
            parent_identity: StatusIdentityKey {
                orchestration_session_id: event.orchestration_session_id.clone(),
                agent_id: event.agent_id.clone(),
                execution_scope: execution_scope_label(scope),
                participant_id: event.parent_participant_id.clone(),
            },
            run_id: event.run_id.clone(),
            parent_run_id: event.parent_run_id.clone(),
            parent_participant_id: event.parent_participant_id.clone(),
        },
        backend_id: entry.derived_backend_id(),
        client: event.agent_id.clone(),
        protocol: tuple.protocol.clone(),
        provider: tuple.provider.clone(),
        auth_authority: tuple.auth_authority.clone(),
    })
}

fn missing_required_nested_fields(projection: &NestedProjection) -> Vec<&'static str> {
    let mut missing_fields = Vec::new();
    if projection.provider.is_none() {
        missing_fields.push("provider");
    }
    if projection.auth_authority.is_none() {
        missing_fields.push("auth_authority");
    }
    missing_fields
}

fn format_invalid_parent_run_id(parent_run_id: Option<&str>) -> String {
    match parent_run_id {
        Some("") => "<empty>".to_string(),
        Some(value) => value.to_string(),
        None => "<missing>".to_string(),
    }
}

fn selected_parent_run_candidates<'a>(
    projection: &NestedProjection,
    exact: &'a BTreeMap<StatusIdentityKey, Vec<SelectedParentRun>>,
    coarse: &'a BTreeMap<StatusIdentityKey, Vec<SelectedParentRun>>,
) -> Vec<&'a SelectedParentRun> {
    if let Some(selected) = exact.get(&projection.source.parent_identity) {
        return selected.iter().collect();
    }

    coarse
        .get(&projection.source.parent_identity.coarse())
        .map(|selected| selected.iter().collect())
        .unwrap_or_default()
}

fn historical_parent_run_matches(
    parent_identity: &StatusIdentityKey,
    parent_run_id: Option<&str>,
    exact: &BTreeMap<StatusIdentityKey, BTreeSet<String>>,
    coarse: &BTreeMap<StatusIdentityKey, BTreeSet<String>>,
) -> bool {
    parent_run_id.is_some_and(|candidate| {
        exact
            .get(parent_identity)
            .is_some_and(|runs| runs.contains(candidate))
            || coarse
                .get(&parent_identity.coarse())
                .is_some_and(|runs| runs.contains(candidate))
    })
}

fn execution_scope_label(scope: AgentExecutionScope) -> &'static str {
    match scope {
        AgentExecutionScope::Host => "host",
        AgentExecutionScope::World => "world",
    }
}

fn scope_for_event(
    event: &AgentEvent,
    entry: &AgentInventoryEntryV1,
    is_selected_orchestrator: bool,
    effective_config: &SubstrateConfig,
) -> AgentExecutionScope {
    if is_selected_orchestrator {
        return entry.effective_scope(effective_config);
    }

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
    is_selected_orchestrator: bool,
    effective_config: &'a SubstrateConfig,
) -> Option<&'a str> {
    if is_selected_orchestrator {
        return role_for_entry(agent_id, effective_config);
    }

    match event.role.as_deref() {
        Some(MEMBER_ROLE) => Some(MEMBER_ROLE),
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
    #[serde(skip)]
    world_boundary_exit_code: Option<i32>,
    #[serde(skip)]
    runtime_realizability_exit_code: Option<i32>,
    #[serde(skip)]
    member_selection_exit_code: Option<i32>,
}

enum RequiredWorldBoundaryState {
    Ready,
    Failed { reason: String, exit_code: i32 },
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
                world_boundary_exit_code: None,
                runtime_realizability_exit_code: None,
                member_selection_exit_code: None,
            });
        }
    };

    let runtime_descriptor = match validate_runtime_realizability(
        inventory
            .get(&orchestrator.agent_id)
            .expect("selected orchestrator must exist in effective inventory"),
        &effective_config,
    ) {
        Ok(descriptor) => {
            checks.push(DoctorCheckJson {
                check: "runtime_realizability".to_string(),
                status: "pass".to_string(),
                reason: None,
            });
            descriptor
        }
        Err(err) => {
            let exit_code = runtime_realizability_error_exit_code(&err);
            checks.push(DoctorCheckJson {
                check: "runtime_realizability".to_string(),
                status: "fail".to_string(),
                reason: Some(err.reason),
            });
            return Ok(DoctorReportJson {
                healthy: false,
                fail_closed: true,
                orchestrator: Some(orchestrator),
                checks,
                world_boundary_exit_code: None,
                runtime_realizability_exit_code: Some(exit_code),
                member_selection_exit_code: None,
            });
        }
    };
    let _ = runtime_descriptor;

    match validate_passive_participant_store() {
        Ok(()) => {
            checks.push(DoctorCheckJson {
                check: "participant_store".to_string(),
                status: "pass".to_string(),
                reason: None,
            });
        }
        Err(err) => {
            checks.push(DoctorCheckJson {
                check: "participant_store".to_string(),
                status: "fail".to_string(),
                reason: Some(err.to_string()),
            });
            return Ok(DoctorReportJson {
                healthy: false,
                fail_closed: true,
                orchestrator: Some(orchestrator),
                checks,
                world_boundary_exit_code: None,
                runtime_realizability_exit_code: None,
                member_selection_exit_code: None,
            });
        }
    }

    let selected_member = match validate_member_selection(&effective_config, &inventory) {
        Ok(Some(descriptor)) => {
            checks.push(DoctorCheckJson {
                check: "member_selection".to_string(),
                status: "pass".to_string(),
                reason: None,
            });
            Some(descriptor)
        }
        Ok(None) => {
            checks.push(DoctorCheckJson {
                check: "member_selection".to_string(),
                status: "not_applicable".to_string(),
                reason: None,
            });
            None
        }
        Err(err) => {
            let exit_code = member_selection_error_exit_code(&err);
            checks.push(DoctorCheckJson {
                check: "member_selection".to_string(),
                status: "fail".to_string(),
                reason: Some(err.reason),
            });
            return Ok(DoctorReportJson {
                healthy: false,
                fail_closed: true,
                orchestrator: Some(orchestrator),
                checks,
                world_boundary_exit_code: None,
                runtime_realizability_exit_code: None,
                member_selection_exit_code: Some(exit_code),
            });
        }
    };

    if let Some(reason) = policy_allowlist_failure(
        &effective_config,
        &inventory,
        &base_policy,
        selected_member.as_ref(),
    ) {
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
            world_boundary_exit_code: None,
            runtime_realizability_exit_code: None,
            member_selection_exit_code: None,
        });
    }
    checks.push(DoctorCheckJson {
        check: "policy_allowlist".to_string(),
        status: "pass".to_string(),
        reason: None,
    });

    if selected_member.is_some() {
        if !effective_config.world.enabled {
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
                world_boundary_exit_code: Some(3),
                runtime_realizability_exit_code: None,
                member_selection_exit_code: None,
            });
        }

        match verify_required_world_boundary(cli) {
            RequiredWorldBoundaryState::Ready => {
                checks.push(DoctorCheckJson {
                    check: "world_boundary".to_string(),
                    status: "pass".to_string(),
                    reason: None,
                });
            }
            RequiredWorldBoundaryState::Failed { reason, exit_code } => {
                checks.push(DoctorCheckJson {
                    check: "world_boundary".to_string(),
                    status: "fail".to_string(),
                    reason: Some(reason),
                });
                return Ok(DoctorReportJson {
                    healthy: false,
                    fail_closed: true,
                    orchestrator: Some(orchestrator),
                    checks,
                    world_boundary_exit_code: Some(exit_code),
                    runtime_realizability_exit_code: None,
                    member_selection_exit_code: None,
                });
            }
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
        world_boundary_exit_code: None,
        runtime_realizability_exit_code: None,
        member_selection_exit_code: None,
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
        world_boundary_exit_code: None,
        runtime_realizability_exit_code: None,
        member_selection_exit_code: None,
    }
}

fn verify_required_world_boundary(cli: &Cli) -> RequiredWorldBoundaryState {
    let exe = match env::current_exe() {
        Ok(path) => path,
        Err(err) => {
            return RequiredWorldBoundaryState::Failed {
                reason: format!(
                    "failed to resolve the substrate executable for required world-boundary validation: {err}"
                ),
                exit_code: 3,
            }
        }
    };

    let mut command = Command::new(exe);
    if cli.world {
        command.arg("--world");
    } else if cli.no_world {
        command.arg("--no-world");
    }

    let output = match command.args(["world", "doctor", "--json"]).output() {
        Ok(output) => output,
        Err(err) => {
            return RequiredWorldBoundaryState::Failed {
                reason: format!(
                    "failed to run `substrate world doctor --json` for required world-boundary validation: {err}"
                ),
                exit_code: 3,
            }
        }
    };

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let value: serde_json::Value = match serde_json::from_slice(&output.stdout) {
        Ok(value) => value,
        Err(err) => {
            let reason = if stderr.is_empty() {
                format!(
                    "required world-boundary validation returned invalid JSON from `substrate world doctor --json`: {err}"
                )
            } else {
                format!(
                    "required world-boundary validation returned invalid JSON from `substrate world doctor --json`: {err}; stderr: {stderr}"
                )
            };
            return RequiredWorldBoundaryState::Failed {
                reason,
                exit_code: 3,
            };
        }
    };

    let world_status = value
        .pointer("/world/status")
        .and_then(serde_json::Value::as_str);
    let ok = output.status.success()
        && value
            .get("ok")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);

    if ok {
        return RequiredWorldBoundaryState::Ready;
    }

    let exit_code = classify_world_boundary_exit_code(output.status.code(), world_status);
    RequiredWorldBoundaryState::Failed {
        reason: format_world_boundary_failure_reason(&value, world_status, &stderr, exit_code),
        exit_code,
    }
}

fn classify_world_boundary_exit_code(
    process_exit_code: Option<i32>,
    world_status: Option<&str>,
) -> i32 {
    match process_exit_code {
        Some(3) => 3,
        Some(4) => 4,
        _ => match world_status {
            Some("unreachable") => 3,
            Some("not_provisioned") | Some("missing_prereqs") | Some("unsupported") => 4,
            _ => 3,
        },
    }
}

fn format_world_boundary_failure_reason(
    value: &serde_json::Value,
    world_status: Option<&str>,
    stderr: &str,
    exit_code: i32,
) -> String {
    let summary = match exit_code {
        4 => "required world-scoped member posture is unsupported or not provisioned on this platform/build",
        _ => "required world-scoped member boundary is unavailable",
    };

    let detail = value
        .get("world_disable_reason")
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            value
                .pointer("/host/world_socket/probe_error")
                .and_then(serde_json::Value::as_str)
        })
        .or_else(|| {
            value
                .pointer("/host/error")
                .and_then(serde_json::Value::as_str)
        })
        .or_else(|| {
            value
                .pointer("/world/landlock/reason")
                .and_then(serde_json::Value::as_str)
        })
        .or_else(|| {
            value
                .pointer("/world/world_fs_strategy/probe/failure_reason")
                .and_then(serde_json::Value::as_str)
        })
        .or_else(|| (!stderr.is_empty()).then_some(stderr));

    match (world_status, detail) {
        (Some(status), Some(detail)) => format!("{summary} (world.status={status}): {detail}"),
        (Some(status), None) => format!("{summary} (world.status={status})"),
        (None, Some(detail)) => format!("{summary}: {detail}"),
        (None, None) => summary.to_string(),
    }
}

fn policy_allowlist_failure(
    effective_config: &SubstrateConfig,
    inventory: &BTreeMap<String, AgentInventoryEntryV1>,
    base_policy: &Policy,
    selected_member: Option<
        &crate::execution::agent_runtime::validator::RuntimeSelectionDescriptor,
    >,
) -> Option<String> {
    let orchestrator = validate_orchestrator_selection(effective_config, inventory).ok()?;
    let orchestrator_backend_id = orchestrator.derived_backend_id();
    if !backend_allowed(base_policy, &orchestrator_backend_id) {
        return Some(format!(
            "selected orchestrator backend '{}' is not allowlisted by effective policy agents.allowed_backends",
            orchestrator_backend_id
        ));
    }

    if let Some(selected_member) = selected_member {
        if !backend_allowed(base_policy, &selected_member.backend_id) {
            return Some(format!(
                "required world-scoped member backend '{}' is not allowlisted by effective policy agents.allowed_backends",
                selected_member.backend_id
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
        "world_boundary" => report.world_boundary_exit_code.unwrap_or(3),
        "runtime_realizability" => report.runtime_realizability_exit_code.unwrap_or(2),
        "member_selection" => report.member_selection_exit_code.unwrap_or(2),
        "inventory_scan" | "orchestrator_selection" | "participant_store" => 2,
        _ => 1,
    }
}

fn validate_passive_participant_store() -> Result<()> {
    let state_store = AgentRuntimeStateStore::new()?;
    state_store.list_participants()?;
    Ok(())
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
