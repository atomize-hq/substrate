use crate::execution::agent_inventory::{
    discover_agent_files, load_agent_inventory_snapshot_v3, load_effective_agent_inventory,
    validate_agent_file, AgentInventoryEntryV1, AgentInventorySnapshotV3,
};
use crate::execution::agent_runtime::auto_attach::SessionAutoAttachClaim;
#[cfg(all(unix, not(target_os = "linux")))]
use crate::execution::agent_runtime::control::run_hidden_owner_helper_startup_prompt_stream_with_action;
#[cfg(not(target_os = "linux"))]
use crate::execution::agent_runtime::control::toolbox_transport_path;
#[cfg(target_os = "linux")]
use crate::execution::agent_runtime::control::PersistedWorldBinding;
#[cfg(target_os = "linux")]
use crate::execution::agent_runtime::control::{
    acquire_authority_managed_successor_launch_permit, discover_toolbox_episode_transport,
    launch_authority_managed_successor_owner_helper, load_authority_managed_successor_launch_plan,
    wait_for_authority_managed_successor_completion, AuthorityManagedSuccessorLaunchPlanV1,
    PrivateTransportAvailabilityV1, ResolvedRuntimeBackendKind, ResolvedRuntimeDescriptor,
};
use crate::execution::agent_runtime::control::{
    authoritative_host_toolbox_surface_enabled, load_hidden_owner_helper_launch_plan,
    load_public_prompt_source, maybe_compose_prompt_with_authoritative_host_toolbox_contract,
    persist_runtime_stop_closeout, public_prompt_rendered_exit_code,
    remove_hidden_owner_helper_launch_plan, run_public_prompt_command,
    toolbox_transport_path_for_home, HiddenOwnerHelperLaunchPlan, HiddenOwnerHelperLaunchReceipt,
    HiddenOwnerHelperParticipantPlan, HiddenOwnerHelperSessionPlan, OwnerHelperMode,
    PublicPromptAction, PublicPromptCommandRequest, PublicPromptInput, PublicSessionPosture,
};
#[cfg(unix)]
use crate::execution::agent_runtime::control::{
    durable_start_stop_transport_path, request_private_authority_managed_stop, request_private_stop,
};
#[cfg(all(unix, not(target_os = "linux")))]
use crate::execution::agent_runtime::control::{
    launch_hidden_owner_helper, register_hidden_owner_helper_startup_prompt_listener,
};
#[cfg(unix)]
use crate::execution::agent_runtime::control::{
    launch_hidden_owner_helper_for_durable_start, private_stop_transport_path,
    public_prompt_rendered_exit, reconcile_resumed_public_turn_detach_timeout,
    register_durable_start_startup_prompt_listener, render_committed_public_start_transaction,
    render_indeterminate_public_start_transaction,
    run_hidden_owner_helper_startup_prompt_stream_with_public_identity,
    verify_public_start_continuity_settled, HiddenOwnerHelperStartupPromptPlan,
};
use crate::execution::agent_runtime::dispatch_contract::LiveToolSupportPosture;
#[cfg(target_os = "linux")]
use crate::execution::agent_runtime::host_session_authority::fork_successor::{
    AllocateForkSuccessorRequestV1, ForkSuccessorAllocationOutcomeV1,
};
#[cfg(unix)]
use crate::execution::agent_runtime::host_session_authority::schema::StartContinuationHandleStateV2;
#[cfg(unix)]
use crate::execution::agent_runtime::host_session_authority::schema::TimestampV1;
#[cfg(target_os = "linux")]
use crate::execution::agent_runtime::host_session_authority::schema::{
    AgentExecutionScopeV1, HostPostTurnDispositionV1, HostSessionAuthorityPreconditionV1,
    HostSessionPostureV1, HostSessionTransitionCallerKindV1, HostSessionTransitionCallerV1,
    HostSessionTransitionModeV1, RuntimeBackendKindV1,
};
#[cfg(target_os = "linux")]
use crate::execution::agent_runtime::host_session_authority::stop::{
    stop_delivery, CompleteHostSessionStopRequestV1, HostSessionStopCompletionOutcomeV1,
    HostSessionStopIssueOutcomeV1, IssueHostSessionStopRequestV1,
};
#[cfg(target_os = "linux")]
use crate::execution::agent_runtime::host_session_authority::store_schema::{
    HostSessionStopIntentStateV1, HostSessionTransitionInputHandoffV1,
    HostSessionTransitionIntentStateV3, HostSessionTransitionIntentV3,
};
#[cfg(unix)]
use crate::execution::agent_runtime::host_session_authority::store_schema::{
    StartTransactionRecordV1, StartTransactionStateV1,
};
#[cfg(target_os = "linux")]
use crate::execution::agent_runtime::host_session_authority::transition::{
    ApplyHostSessionTransitionRequestV1, ClaimHostSessionTransitionRequestV1,
    IssueSuccessorTransitionRequestV1, SuccessorTransitionApplicationOutcomeV1,
    SuccessorTransitionClaimOutcomeV1, SuccessorTransitionIssueOutcomeV1,
};
#[cfg(unix)]
use crate::execution::agent_runtime::host_session_authority::trusted_fs::TrustedAuthorityRoot;
#[cfg(unix)]
use crate::execution::agent_runtime::host_session_authority::HostSessionAuthority;
use crate::execution::agent_runtime::orchestration_session::HostAttachContract;
use crate::execution::agent_runtime::orchestration_session::{
    OrchestrationSessionPosture, OrchestrationSessionRecord,
};
#[cfg(all(unix, not(target_os = "linux")))]
use crate::execution::agent_runtime::resolve_persisted_host_attach_contract;
#[cfg(not(target_os = "linux"))]
use crate::execution::agent_runtime::session::AgentRuntimeReplacementParticipantInit;
#[cfg(any(test, not(target_os = "linux")))]
use crate::execution::agent_runtime::validator::RuntimeSelectionDescriptor;
use crate::execution::agent_runtime::validator::{
    materialize_runtime_descriptor, member_selection_error_exit_code,
    resolve_live_tool_support_posture, resolve_selected_orchestrator_live_tool_support_posture,
    validate_member_selection,
};
#[cfg(unix)]
use crate::execution::agent_runtime::{
    resolve_inventory_contract_for_exact_backend, runtime_realizability_error_exit_code,
    validate_orchestrator_selection, validate_runtime_realizability, AgentRuntimeParticipantRecord,
    AgentRuntimeSessionRecord, AgentRuntimeStateStore, AttachLaunchKnobs, AttachModePreference,
    DispatchBaselineKind, DispatchCallerKind, DispatchCapabilityOverrideSet,
    DispatchRequestEnvelope, HostExecutionClientStart, PublicAttachAction, PublicControlAction,
    PublicTurnTargetKind, ResolvedLaunchContract, StartupPromptReplayState,
    MANUAL_REATTACH_ATTACH_RESTORED_REASON, MEMBER_ROLE, NESTED_ROUTER, ORCHESTRATOR_ROLE,
    PURE_AGENT_PROTOCOL, PURE_AGENT_ROUTER,
};
use crate::execution::cli::{
    AgentAction, AgentCmd, AgentDisableCapabilityArg, AgentDoctorArgs, AgentOwnerHelperArgs,
    AgentScopeArg, AgentSessionControlArgs, AgentStartArgs, AgentStartScopeArg, AgentToolboxAction,
    AgentToolboxCmd, AgentToolboxViewArgs, AgentTurnArgs, AgentViewArgs, AgentsAction, AgentsCmd,
    Cli,
};
use crate::execution::config_model::{
    self, AgentExecutionScope, AgentToolboxBindTransport, CliConfigOverrides, SubstrateConfig,
};
#[cfg(target_os = "linux")]
use crate::execution::policy_snapshot;
use crate::execution::prompt_fulfillment::HOST_TOOLBOX_CONTRACT_VERSION_V1;
#[cfg(target_os = "linux")]
use crate::execution::{ReplPersistentSessionClient, ReplSessionStartParams};
#[cfg(target_os = "linux")]
use crate::repl::async_repl::prepare_public_start_authority_before_transport;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
#[cfg(target_os = "linux")]
use fs2::FileExt;
#[cfg(target_os = "linux")]
use serde::Deserialize;
use serde::Serialize;
#[cfg(unix)]
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
#[cfg(target_os = "linux")]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(target_os = "linux")]
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use substrate_broker::Policy;
use substrate_common::paths as substrate_paths;
use substrate_common::{AgentEvent, PlacementExecution};
#[cfg(unix)]
use tokio::runtime::Builder as TokioRuntimeBuilder;
#[cfg(target_os = "linux")]
use transport_api_types::InstallBootstrapContextCarrierV1;
use transport_api_types::{AgentSelectionPlacementV1, ResolvedAgentSelectionV1};
#[cfg(target_os = "linux")]
use transport_api_types::{SharedWorldOwnerAction, SharedWorldOwnerSpec};
use uuid::Uuid;
const TOOLBOX_VERSION: u32 = HOST_TOOLBOX_CONTRACT_VERSION_V1;
#[cfg(unix)]
const START_ATTACHED_GRACE_TIMEOUT: Duration = Duration::from_millis(250);
#[cfg(unix)]
const TURN_DETACH_NORMALIZATION_TIMEOUT: Duration = Duration::from_secs(10);
#[cfg(unix)]
const TURN_DETACH_NORMALIZATION_POLL_INTERVAL: Duration = Duration::from_millis(50);

pub(crate) fn handle_agent_command(
    cmd: &AgentCmd,
    cli: &Cli,
    #[cfg(target_os = "linux")] install_context: &InstallBootstrapContextCarrierV1,
) -> i32 {
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
        AgentAction::Doctor(args) => match run_doctor(
            args,
            cli,
            #[cfg(target_os = "linux")]
            install_context,
        ) {
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
        AgentAction::Start(args) => render_public_prompt_command_result(run_start(
            args,
            cli,
            #[cfg(target_os = "linux")]
            install_context,
        )),
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
        AgentAction::OwnerHelper(args) => match run_owner_helper(
            args,
            #[cfg(target_os = "linux")]
            install_context,
        ) {
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
        AgentsAction::Catalog(_) => Err(anyhow::anyhow!(
            "catalog requires the accepted-home early dispatcher"
        )),
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

fn run_doctor(
    args: &AgentDoctorArgs,
    cli: &Cli,
    #[cfg(target_os = "linux")] install_context: &InstallBootstrapContextCarrierV1,
) -> Result<i32> {
    let report = build_doctor_report(
        cli,
        #[cfg(target_os = "linux")]
        install_context,
    )?;
    let exit_code = doctor_exit_code(&report);
    render_doctor_report(&report, args.json)?;
    Ok(exit_code)
}

fn run_owner_helper(
    args: &AgentOwnerHelperArgs,
    #[cfg(target_os = "linux")] install_context: &InstallBootstrapContextCarrierV1,
) -> Result<i32> {
    #[cfg(target_os = "linux")]
    if args
        .plan_file
        .parent()
        .and_then(|path| path.file_name())
        .and_then(std::ffi::OsStr::to_str)
        == Some("ordinary-successor-plans-v1")
    {
        let envelope =
            crate::execution::agent_runtime::control::load_ordinary_successor_helper_envelope(
                &args.plan_file,
            )?;
        // Capture only parent-owned envelope transport, CLOEXEC and RAII,
        // before slow authentication. No Accepted or authority effects occur.
        let transport_plan = crate::execution::agent_runtime::control::validate_ordinary_successor_helper_envelope(
            &crate::execution::agent_runtime::control::decode_ordinary_successor_helper_envelope(&envelope)?
        )?;
        let startup_path = &transport_plan
            .helper_plan
            .startup_prompt
            .as_ref()
            .context("ordinary successor has no exact startup reply path")?
            .stream_path;
        let startup_reply =
            crate::execution::agent_runtime::control::take_ordinary_successor_startup_stream_v1(
                startup_path,
            )?;
        let (selection, successor) =
            crate::execution::agent_runtime::control::authenticate_ordinary_successor_helper(
                &envelope,
                install_context,
            )?;
        let plan = successor.helper_plan.clone();
        remove_hidden_owner_helper_launch_plan(&args.plan_file)?;
        return crate::repl::async_repl::run_ordinary_hidden_owner_helper(
            plan,
            successor,
            selection,
            install_context.clone(),
            startup_reply,
        );
    }
    #[cfg(target_os = "linux")]
    let authority_successor = (args
        .plan_file
        .parent()
        .and_then(|path| path.file_name())
        .and_then(std::ffi::OsStr::to_str)
        == Some("successor-plans"))
    .then(|| load_authority_managed_successor_launch_plan(&args.plan_file))
    .transpose()?;
    #[cfg(target_os = "linux")]
    if authority_successor.is_some() {
        anyhow::bail!("ordinary public successor requires the strict original-selection envelope");
    }
    #[cfg(target_os = "linux")]
    let plan = match authority_successor.as_ref() {
        Some(successor) => successor.helper_plan.clone(),
        None => load_hidden_owner_helper_launch_plan(&args.plan_file)?,
    };
    #[cfg(not(target_os = "linux"))]
    let plan = load_hidden_owner_helper_launch_plan(&args.plan_file)?;
    #[cfg(target_os = "linux")]
    {
        if plan.mode != OwnerHelperMode::Start {
            anyhow::bail!("ordinary public successor inner plan cannot enter generic Existing");
        }
        let selected = crate::execution::agent_runtime::control::recover_ordinary_public_selection(
            install_context,
            plan.orchestration_session_id(),
        )?;
        crate::execution::agent_runtime::control::authenticate_ordinary_public_helper_run(
            &selected, &plan, None,
        )?
        .revalidate_before_effect()?;
    }
    remove_hidden_owner_helper_launch_plan(&args.plan_file)?;
    crate::repl::async_repl::run_hidden_owner_helper(
        plan,
        #[cfg(target_os = "linux")]
        authority_successor,
        #[cfg(target_os = "linux")]
        install_context.context.intended_host_principal.clone(),
    )
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

#[derive(Clone, Debug)]
struct StartPromptPublicIdentity {
    #[cfg_attr(not(unix), allow(dead_code))]
    backend_id: String,
    scope: AgentExecutionScope,
}

#[derive(Debug)]
struct HostStartLaunchPlan {
    helper_plan: HiddenOwnerHelperLaunchPlan,
    resolved_contract: ResolvedLaunchContract,
    public_identity: StartPromptPublicIdentity,
    native_selection: ResolvedAgentSelectionV1,
    requested_selection: ResolvedAgentSelectionV1,
}

fn run_start(
    args: &AgentStartArgs,
    cli: &Cli,
    #[cfg(target_os = "linux")] install_context: &InstallBootstrapContextCarrierV1,
) -> Result<()> {
    let prompt = load_public_prompt_source(&PublicPromptInput {
        prompt: args.prompt_source.prompt.clone(),
        prompt_file: args.prompt_source.prompt_file.clone(),
    })
    .map_err(normalize_public_prompt_error)?;
    let context = resolve_command_context(cli)?;
    #[cfg(target_os = "linux")]
    let configured =
        crate::execution::install_bootstrap::acquire_ordinary_configured_home(install_context)?;
    #[cfg(target_os = "linux")]
    let invocation_root = TrustedAuthorityRoot::open_for_owner(
        Path::new(&configured.accepted_home().physical_path),
        u32::try_from(configured.intended_uid())?,
    )?;
    #[cfg(target_os = "linux")]
    let invocation_workspace = {
        use crate::execution::agent_runtime::host_session_authority::schema::{
            CanonicalDirectoryV1, DirectoryPhysicalIdentityV1,
        };
        use std::os::unix::fs::MetadataExt;
        let path = fs::canonicalize(current_dir())?;
        let metadata = fs::metadata(&path)?;
        CanonicalDirectoryV1 {
            physical_path: path
                .to_str()
                .context("public workspace requires UTF-8")?
                .into(),
            physical_identity: DirectoryPhysicalIdentityV1::Linux {
                device_id: metadata.dev(),
                inode: metadata.ino(),
            },
        }
    };
    #[cfg(target_os = "linux")]
    let held_workspace = crate::execution::agent_runtime::host_session_authority::trusted_fs::TrustedWorkspaceRoot::open_exact(
        &invocation_workspace,
    )?;
    #[cfg(target_os = "linux")]
    let revalidate_install = || -> Result<()> {
        crate::execution::install_bootstrap::bind_unix_install_bootstrap_context(install_context)?;
        let encoded = install_context.encode().map_err(anyhow::Error::msg)?;
        crate::execution::install_bootstrap::validate_install_bootstrap_projections(
            install_context,
            &encoded,
            |key| env::var_os(key),
        )?;
        configured.revalidate()?;
        invocation_root.revalidate()?;
        held_workspace.revalidate()?;
        let observed = public_start_authority()?;
        if observed.bootstrap_home().identity()? != invocation_root.identity()
            || invocation_root.identity().physical_path != configured.accepted_home().physical_path
            || fs::canonicalize(current_dir())?.to_str()
                != Some(invocation_workspace.physical_path.as_str())
        {
            anyhow::bail!("public invocation install root or workspace changed");
        }
        let same_home = match (
            &invocation_root.identity().physical_identity,
            &configured.accepted_home().physical_identity,
        ) {
            (
                crate::execution::agent_runtime::host_session_authority::schema::DirectoryPhysicalIdentityV1::Linux { device_id: a, inode: b },
                config_projection::DirectoryPhysicalIdentityV1::Linux { device_id: c, inode: d },
            ) => a == c && b == d,
            _ => false,
        };
        if !same_home {
            anyhow::bail!("public invocation physical configured home changed");
        }
        Ok(())
    };
    #[cfg(target_os = "linux")]
    revalidate_install()?;
    let store = AgentRuntimeStateStore::new()?;
    #[cfg(target_os = "linux")]
    let mut start_plan = build_start_launch_plan(args, &context)?;
    #[cfg(not(target_os = "linux"))]
    let start_plan = build_start_launch_plan(args, &context)?;

    #[cfg(target_os = "linux")]
    let selections =
        crate::execution::agent_runtime::native_launch_context::OrdinaryStartSelectionsV1 {
            native: start_plan.native_selection.clone(),
            requested: start_plan.requested_selection.clone(),
        };
    #[cfg(target_os = "linux")]
    let sources =
        crate::execution::agent_runtime::native_launch_context::CapturedOrdinarySourcesV1::capture(
            &context.effective_config,
            &context.selection_snapshot,
            &selections,
        )?;
    #[cfg(target_os = "linux")]
    let planned_workspace = start_plan.helper_plan.session.workspace_root.clone();
    #[cfg(target_os = "linux")]
    let revalidate_invocation = || -> Result<()> {
        revalidate_install()?;
        if planned_workspace != invocation_workspace.physical_path {
            anyhow::bail!("public birth workspace differs from invocation custody");
        }
        let fresh = resolve_command_context(cli)?;
        let fresh_sources = crate::execution::agent_runtime::native_launch_context::CapturedOrdinarySourcesV1::capture(
            &fresh.effective_config, &fresh.selection_snapshot, &selections)?;
        if serde_json::to_value(&fresh_sources)? != serde_json::to_value(&sources)?
            || serde_json::to_value(&fresh.base_policy)?
                != serde_json::to_value(&context.base_policy)?
        {
            anyhow::bail!("public Start source/policy changed before birth effect");
        }
        revalidate_install()?;
        Ok(())
    };

    #[cfg(unix)]
    let (startup_prompt_text, prompt_sha256, request_key_sha256) = {
        let authoritative_toolbox = authoritative_host_toolbox_surface_enabled(
            &start_plan.resolved_contract.backend_id,
            start_plan.resolved_contract.execution_scope,
            ORCHESTRATOR_ROLE,
            &context.effective_config,
            &context.base_policy,
        );
        let prompt_text = maybe_compose_prompt_with_authoritative_host_toolbox_contract(
            &prompt.prompt_text,
            authoritative_toolbox,
        );
        let prompt_sha256 = public_start_digest(
            "substrate.a1.3-p1.public-start-prompt.v1",
            &[prompt_text.as_bytes()],
        );
        let request_key_sha256 = public_start_request_key(
            &start_plan,
            &prompt_sha256,
            &start_plan.public_identity.backend_id,
            start_plan.public_identity.scope,
        )?;
        (prompt_text, prompt_sha256, request_key_sha256)
    };

    #[cfg(target_os = "linux")]
    {
        revalidate_invocation()?;
        ensure_public_start_authority_bootstrapped()?;
    }
    #[cfg(unix)]
    let mut retry_transaction = retryable_public_start_transaction(&request_key_sha256)?;
    #[cfg(unix)]
    {
        if let Some(transaction) = retry_transaction.as_ref() {
            adopt_retryable_public_start_plan(
                &mut start_plan,
                transaction,
                &prompt_sha256,
                &request_key_sha256,
            )?;
        }
        start_plan.helper_plan.participant.lease_token = public_start_digest(
            "substrate.a1.3-p1.public-start-lease.v1",
            &[
                request_key_sha256.as_bytes(),
                start_plan.helper_plan.participant.participant_id.as_bytes(),
            ],
        );
    }

    if start_plan.public_identity.scope == AgentExecutionScope::World {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = prompt;
            anyhow::bail!(config_model::user_error(
                "unsupported_platform_or_posture: public world-scoped root start is supported on Linux only in this slice"
            ));
        }
        #[cfg(target_os = "linux")]
        {
            if retry_transaction.is_none() {
                revalidate_invocation()?;
                let world_binding = establish_public_world_start_binding(
                    start_plan.helper_plan.orchestration_session_id(),
                    &start_plan.helper_plan.session.workspace_root,
                )
                .map_err(runtime_start_error)?;
                start_plan.helper_plan.session.world_id = Some(world_binding.world_id);
                start_plan.helper_plan.session.world_generation =
                    Some(world_binding.world_generation);
            }
        }
    }

    #[cfg(target_os = "linux")]
    if retry_transaction.as_ref().is_none_or(|transaction| {
        matches!(
            transaction.state,
            StartTransactionStateV1::PromptNotSubmitted
        )
    }) {
        start_plan.helper_plan.startup_prompt = Some(HiddenOwnerHelperStartupPromptPlan {
            prompt_text: startup_prompt_text.clone(),
            stream_path: PathBuf::from("/tmp/substrate-public-start-transport-pending"),
            request_key_sha256: Some(request_key_sha256.clone()),
            prompt_sha256: Some(prompt_sha256.clone()),
            public_backend_id: Some(start_plan.public_identity.backend_id.clone()),
            public_scope: Some(start_plan.public_identity.scope),
        });
        revalidate_invocation()?;
        prepare_public_start_authority_before_transport(
            &start_plan.helper_plan,
            install_context,
            invocation_root.identity(),
            &invocation_workspace,
            &selections,
            &sources,
            &context.effective_config,
            &context.inventory,
            &start_plan.resolved_contract,
        )
        .map_err(runtime_start_error)?;
    }

    let HostStartLaunchPlan {
        helper_plan,
        resolved_contract,
        public_identity,
        native_selection: _,
        requested_selection: _,
    } = start_plan;

    #[cfg(not(unix))]
    {
        let _ = store;
        let _ = helper_plan;
        let _ = resolved_contract;
        let _ = public_identity;
        let _ = prompt;
        anyhow::bail!(config_model::user_error(
            "unsupported_platform_or_posture: public start prompt streaming requires a Unix launch-time backchannel"
        ));
    }

    #[cfg(unix)]
    {
        let _ = &resolved_contract;
        if let Some(transaction) = retry_transaction.as_ref() {
            if matches!(
                transaction.state,
                StartTransactionStateV1::PromptSubmissionNoReplayBarrier { .. }
                    | StartTransactionStateV1::PromptSubmissionIndeterminate { .. }
            ) {
                let indeterminate =
                    declare_or_join_public_start_submission_indeterminate(transaction)?;
                return render_indeterminate_public_start_result(&indeterminate, args.json);
            }
        }
        if retry_transaction.as_ref().is_some_and(|transaction| {
            matches!(
                transaction.state,
                StartTransactionStateV1::ContinuationRegistered { .. }
            )
        }) {
            retry_transaction = Some(wait_for_committed_public_start_transaction(
                &request_key_sha256,
            )?);
        }
        if let Some(transaction) = retry_transaction.as_ref() {
            if matches!(
                transaction.state,
                StartTransactionStateV1::TurnSettledAwaitingResponse { .. }
            ) {
                return render_and_acknowledge_committed_public_start(
                    transaction,
                    &request_key_sha256,
                    args.json,
                );
            }
        }
        let public_backend_id = public_identity.backend_id.clone();
        let public_scope = public_identity.scope;
        let stream_start_result = |listener| {
            run_hidden_owner_helper_startup_prompt_stream_with_public_identity(
                listener,
                args.json,
                PublicPromptAction::Start,
                &public_backend_id,
                public_scope,
            )
        };
        let mut plan = helper_plan;
        let startup_listener = register_durable_start_startup_prompt_listener(
            plan.orchestration_session_id(),
            plan.participant_id(),
        )
        .map_err(runtime_start_error)?;
        plan.startup_prompt = Some(HiddenOwnerHelperStartupPromptPlan {
            prompt_text: startup_prompt_text.clone(),
            stream_path: startup_listener.path().to_path_buf(),
            request_key_sha256: Some(request_key_sha256.clone()),
            prompt_sha256: Some(prompt_sha256.clone()),
            public_backend_id: Some(public_backend_id.clone()),
            public_scope: Some(public_scope),
        });

        let receipt = launch_hidden_owner_helper_for_durable_start(&plan, cli.world, cli.no_world)
            .map_err(runtime_start_error)?;
        match stream_start_result(startup_listener) {
            Ok(()) => {
                verify_public_start_continuity_settled(&receipt.orchestration_session_id)
                    .map_err(runtime_start_error)?;
                let transaction = retryable_public_start_transaction(&request_key_sha256)?
                    .ok_or_else(|| {
                        anyhow::anyhow!("settled public Start transaction disappeared")
                    })?;
                acknowledge_public_start_response(&transaction, &request_key_sha256)
            }
            Err(err) => {
                if verify_public_start_continuity_settled(&receipt.orchestration_session_id).is_ok()
                {
                    let transaction = retryable_public_start_transaction(&request_key_sha256)?
                        .ok_or_else(|| {
                            anyhow::anyhow!("settled public Start transaction disappeared")
                        })?;
                    join_committed_public_start_after_stream_error(
                        err,
                        || acknowledge_public_start_response(&transaction, &request_key_sha256),
                        || {
                            render_and_acknowledge_committed_public_start(
                                &transaction,
                                &request_key_sha256,
                                args.json,
                            )
                        },
                    )
                } else {
                    let _legacy_replay_observation_only = store
                        .startup_prompt_replay_state(
                            &receipt.orchestration_session_id,
                            &receipt.participant_id,
                        )
                        .map(StartupPromptReplayState::replay_safe);
                    let transaction = retryable_public_start_transaction(&request_key_sha256)?
                        .ok_or_else(|| {
                            anyhow::anyhow!("durable public Start transaction disappeared")
                        })?;
                    if transaction.orchestration_session_id != receipt.orchestration_session_id
                        || transaction.authoritative_participant_id != receipt.participant_id
                    {
                        anyhow::bail!(config_model::user_error(
                            "runtime_start_failed: failed Start transport does not exact-join its durable transaction"
                        ));
                    }
                    match transaction.state {
                        StartTransactionStateV1::PromptSubmissionNoReplayBarrier { .. }
                        | StartTransactionStateV1::PromptSubmissionIndeterminate { .. } => {
                            let indeterminate =
                                declare_or_join_public_start_submission_indeterminate(
                                    &transaction,
                                )?;
                            #[cfg(target_os = "linux")]
                            if public_prompt_rendered_exit_code(&err).is_some() {
                                return Err(err);
                            }
                            render_indeterminate_public_start_result(&indeterminate, args.json)
                        }
                        StartTransactionStateV1::ContinuationRegistered { .. } => {
                            let committed =
                                wait_for_committed_public_start_transaction(&request_key_sha256)?;
                            render_and_acknowledge_committed_public_start(
                                &committed,
                                &request_key_sha256,
                                args.json,
                            )
                        }
                        StartTransactionStateV1::TurnSettledAwaitingResponse { .. } => {
                            render_and_acknowledge_committed_public_start(
                                &transaction,
                                &request_key_sha256,
                                args.json,
                            )
                        }
                        StartTransactionStateV1::PromptNotSubmitted
                        | StartTransactionStateV1::PublicResponseDelivered { .. } => {
                            Err(normalize_public_prompt_error(err))
                        }
                    }
                }
            }
        }
    }
}

#[cfg(unix)]
fn public_start_digest(domain: &str, parts: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update((part.len() as u64).to_be_bytes());
        hasher.update(part);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(unix)]
fn public_start_request_key(
    plan: &HostStartLaunchPlan,
    prompt_sha256: &str,
    public_backend_id: &str,
    public_scope: AgentExecutionScope,
) -> Result<String> {
    plan.native_selection
        .validate()
        .map_err(config_model::user_error)?;
    plan.requested_selection
        .validate()
        .map_err(config_model::user_error)?;
    let requested_placement = match public_scope {
        AgentExecutionScope::Host => AgentSelectionPlacementV1::Host,
        AgentExecutionScope::World => AgentSelectionPlacementV1::World,
    };
    if plan.native_selection.placement != AgentSelectionPlacementV1::Host
        || plan.native_selection.backend_id != plan.helper_plan.descriptor.backend_id
        || plan.requested_selection.placement != requested_placement
        || plan.requested_selection.backend_id != public_backend_id
        || (public_scope == AgentExecutionScope::Host
            && plan.native_selection != plan.requested_selection)
    {
        return Err(config_model::user_error(
            "Start selections conflict with the launch identities",
        ));
    }
    let stable_request = serde_json::json!({
        "schema_version": 2,
        "native_selection": &plan.native_selection,
        "requested_selection": &plan.requested_selection,
        "descriptor": &plan.helper_plan.descriptor,
        "workspace_root": &plan.helper_plan.session.workspace_root,
        "host_attach_contract": &plan.helper_plan.host_attach_contract,
        "public_backend_id": public_backend_id,
        "public_scope": match public_scope {
            AgentExecutionScope::Host => "host",
            AgentExecutionScope::World => "world",
        },
        "prompt_sha256": prompt_sha256,
    });
    let bytes = crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(
        &stable_request,
    )
    .context("failed to encode the durable public Start request key")?;
    Ok(public_start_digest(
        "substrate.selected-public-start-request.v1",
        &[&bytes],
    ))
}

#[cfg(unix)]
fn public_start_authority() -> Result<HostSessionAuthority> {
    let substrate_home = substrate_paths::substrate_home()?;
    HostSessionAuthority::from_trusted_root(
        TrustedAuthorityRoot::open(&substrate_home)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?,
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))
}

#[cfg(target_os = "linux")]
fn ensure_public_start_authority_bootstrapped() -> Result<()> {
    let authority = public_start_authority()?;
    // Existing selected births are V2/V3. The V1 bootstrap cannot join them.
    // A failed strict versioned read still has to pass the original bootstrap.
    if authority.read_preserved_start_root_v2().is_ok() {
        return Ok(());
    }
    authority
        .bootstrap()
        .map(|_| ())
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}

#[cfg(unix)]
fn retryable_public_start_transaction(
    request_key_sha256: &str,
) -> Result<Option<StartTransactionRecordV1>> {
    public_start_authority()?
        .retryable_start_transaction(request_key_sha256)
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}

#[cfg(unix)]
fn declare_or_join_public_start_submission_indeterminate(
    transaction: &StartTransactionRecordV1,
) -> Result<StartTransactionRecordV1> {
    let authority = public_start_authority()?;
    let current = authority
        .retryable_start_transaction(&transaction.request_key_sha256)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?
        .ok_or_else(|| anyhow::anyhow!("durable public Start transaction disappeared"))?;
    if current != *transaction {
        anyhow::bail!(config_model::user_error(
            "runtime_start_failed: indeterminate Start recovery does not exact-join the durable transaction"
        ));
    }
    match current.state {
        StartTransactionStateV1::PromptSubmissionNoReplayBarrier { .. } => {
            let declared_at =
                TimestampV1::parse(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
                    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
            authority
                .mark_start_submission_outcome_indeterminate(
                    &current.transaction_id,
                    &current.request_key_sha256,
                    declared_at,
                )
                .map_err(|error| anyhow::anyhow!(error.to_string()))
        }
        StartTransactionStateV1::PromptSubmissionIndeterminate { .. } => Ok(current),
        _ => anyhow::bail!(config_model::user_error(
            "runtime_start_failed: Start submission is not in an indeterminate recovery phase"
        )),
    }
}

#[cfg(unix)]
fn public_start_submission_outcome_unknown_error(
    transaction: &StartTransactionRecordV1,
) -> anyhow::Error {
    config_model::user_error(format!(
        "submission_outcome_unknown: durable Start transaction {} crossed the no-replay barrier without committed authenticated provider acceptance; the real prompt was not replayed",
        transaction.transaction_id
    ))
}

#[cfg(unix)]
fn adopt_retryable_public_start_plan(
    plan: &mut HostStartLaunchPlan,
    transaction: &StartTransactionRecordV1,
    prompt_sha256: &str,
    request_key_sha256: &str,
) -> Result<()> {
    let expected_scope = match plan.public_identity.scope {
        AgentExecutionScope::Host => "host",
        AgentExecutionScope::World => "world",
    };
    if transaction.request_key_sha256 != request_key_sha256
        || transaction.prompt_sha256 != prompt_sha256
        || transaction.backend_id != plan.helper_plan.descriptor.backend_id
        || transaction.protocol != plan.helper_plan.descriptor.protocol
        || transaction.workspace_root != plan.helper_plan.session.workspace_root
        || transaction.public_backend_id != plan.public_identity.backend_id
        || transaction.public_scope != expected_scope
    {
        anyhow::bail!(config_model::user_error(
            "runtime_start_failed: retry does not exact-join the committed Start transaction"
        ));
    }
    plan.helper_plan.session.orchestration_session_id =
        transaction.orchestration_session_id.clone();
    plan.helper_plan.session.shell_trace_session_id = transaction.shell_trace_session_id.clone();
    plan.helper_plan.session.world_id = transaction.world_id.clone();
    plan.helper_plan.session.world_generation = transaction.world_generation;
    plan.helper_plan.participant.participant_id = transaction.authoritative_participant_id.clone();
    plan.helper_plan.participant.run_id = transaction.start_run_id.clone();
    Ok(())
}

#[cfg(unix)]
fn wait_for_committed_public_start_transaction(
    request_key_sha256: &str,
) -> Result<StartTransactionRecordV1> {
    let started_at = std::time::Instant::now();
    loop {
        let transaction = retryable_public_start_transaction(request_key_sha256)?
            .ok_or_else(|| anyhow::anyhow!("durable public Start transaction disappeared"))?;
        match transaction.state {
            StartTransactionStateV1::TurnSettledAwaitingResponse { .. } => return Ok(transaction),
            StartTransactionStateV1::ContinuationRegistered { .. } => {}
            StartTransactionStateV1::PromptSubmissionNoReplayBarrier { .. }
            | StartTransactionStateV1::PromptSubmissionIndeterminate { .. } => {
                return Err(public_start_submission_outcome_unknown_error(&transaction))
            }
            StartTransactionStateV1::PromptNotSubmitted => {
                anyhow::bail!(
                    "durable Start retry regressed to the pre-submission transaction phase"
                )
            }
            StartTransactionStateV1::PublicResponseDelivered { .. } => {
                anyhow::bail!("durable Start retry unexpectedly joined a delivered transaction")
            }
        }
        if started_at.elapsed() >= Duration::from_secs(30) {
            anyhow::bail!(config_model::user_error(
                "owner_unreachable: durable Start remains in progress; the real prompt was not replayed"
            ));
        }
        thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(unix)]
fn acknowledge_public_start_response(
    transaction: &StartTransactionRecordV1,
    request_key_sha256: &str,
) -> Result<()> {
    let delivered_at =
        TimestampV1::parse(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    public_start_authority()?
        .mark_start_response_delivered(
            &transaction.transaction_id,
            request_key_sha256,
            delivered_at,
        )
        .map(|_| ())
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}

#[cfg(unix)]
fn render_and_acknowledge_committed_public_start(
    transaction: &StartTransactionRecordV1,
    request_key_sha256: &str,
    json: bool,
) -> Result<()> {
    commit_rendered_public_start_response(
        || render_committed_public_start_transaction(transaction, json),
        || acknowledge_public_start_response(transaction, request_key_sha256),
    )
}

#[cfg(unix)]
fn render_indeterminate_public_start_result(
    transaction: &StartTransactionRecordV1,
    json: bool,
) -> Result<()> {
    let exit_code = render_indeterminate_public_start_transaction(transaction, json)?;
    Err(public_prompt_rendered_exit(exit_code))
}

#[cfg(unix)]
fn commit_rendered_public_start_response(
    render: impl FnOnce() -> Result<i32>,
    acknowledge: impl FnOnce() -> Result<()>,
) -> Result<()> {
    let exit_code = render()?;
    acknowledge()?;
    if exit_code == 0 {
        Ok(())
    } else {
        Err(public_prompt_rendered_exit(exit_code))
    }
}

#[cfg(unix)]
fn join_committed_public_start_after_stream_error(
    stream_error: anyhow::Error,
    acknowledge_already_rendered: impl FnOnce() -> Result<()>,
    render_and_acknowledge: impl FnOnce() -> Result<()>,
) -> Result<()> {
    if public_prompt_rendered_exit_code(&stream_error).is_some() {
        acknowledge_already_rendered()?;
        Err(stream_error)
    } else {
        render_and_acknowledge()
    }
}

fn run_turn(args: &AgentTurnArgs, cli: &Cli) -> Result<()> {
    let prompt = load_public_prompt_source(&PublicPromptInput {
        prompt: args.prompt_source.prompt.clone(),
        prompt_file: args.prompt_source.prompt_file.clone(),
    })
    .map_err(normalize_public_prompt_error)?;

    #[cfg(target_os = "linux")]
    let ordinary_source = {
        let carrier = crate::execution::install_bootstrap::checked_install_bootstrap_context_from_projections()?;
        let selected = crate::execution::agent_runtime::control::recover_ordinary_public_selection(
            &carrier,
            &args.session,
        )?;
        crate::execution::agent_runtime::control::revalidate_ordinary_public_source_permission(
            &selected,
        )?;
        selected
    };

    #[cfg(target_os = "linux")]
    let mut public_successor_guard = acquire_public_successor_request_guard(
        &substrate_paths::substrate_home()?,
        &args.session,
        HostSessionTransitionModeV1::ResumeOneTurn,
        &public_start_digest(
            "substrate.public-successor-request.v1",
            &[
                args.session.as_bytes(),
                args.backend.as_bytes(),
                prompt.prompt_text.as_bytes(),
            ],
        ),
    )
    .map_err(runtime_start_error)?;

    #[cfg(target_os = "linux")]
    let hsa_active_identity = {
        let substrate_home = substrate_paths::substrate_home()?;
        let state_root_path = substrate_home.join("authority-v1/state-root-v1.json");
        if state_root_path.exists() {
            let authority = HostSessionAuthority::from_trusted_root(
                TrustedAuthorityRoot::open(&substrate_home)
                    .map_err(|error| runtime_start_error(anyhow::anyhow!(error.to_string())))?,
            )
            .map_err(|error| runtime_start_error(anyhow::anyhow!(error.to_string())))?;
            let root = authority
                .read_a12b_root()
                .map_err(|error| runtime_start_error(anyhow::anyhow!(error.to_string())))?;
            if root.session_namespace_map.contains_key(&args.session) {
                let current = authority
                    .resolve_current_exact(&args.session, None)
                    .map_err(|error| runtime_start_error(anyhow::anyhow!(error.to_string())))?;
                let backend_id = current.caller.descriptor.backend_id.clone();
                if ordinary_source.requested_selection().backend_id != args.backend {
                    anyhow::bail!(config_model::user_error(format!(
                        "backend_not_in_session: orchestration session {} has no exact backend slot for {}",
                        args.session, args.backend
                    )));
                }
                let captured_config = ordinary_source.captured_effective_config();
                let current_policy =
                    crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
                        Path::new(ordinary_source.workspace_root()),
                        &ordinary_source.held_authority().bootstrap_home(),
                    )?;
                let host_toolbox_surface_authoritative = authoritative_host_toolbox_surface_enabled(
                    &backend_id,
                    AgentExecutionScope::Host,
                    ORCHESTRATOR_ROLE,
                    &captured_config,
                    &current_policy,
                );
                let exact_prompt = maybe_compose_prompt_with_authoritative_host_toolbox_contract(
                    &prompt.prompt_text,
                    host_toolbox_surface_authoritative,
                );
                let mut in_flight_resume = None;
                for intent in root.successor_transition_intent_map.values() {
                    let HostSessionTransitionIntentStateV3::Applied {
                        authority_revision_after,
                        authority_record_commitment,
                        active_authoritative_participant_id,
                        post_turn,
                        ..
                    } = &intent.state
                    else {
                        continue;
                    };
                    if intent.mode == HostSessionTransitionModeV1::ResumeOneTurn
                        && intent.caller.kind == HostSessionTransitionCallerKindV1::PublicCli
                        && intent.orchestration_session_id == args.session
                        && intent.target_authoritative_participant_id
                            == current.caller.participant_id
                        && active_authoritative_participant_id == &current.caller.participant_id
                        && *authority_revision_after == current.observation.authority_revision
                        && authority_record_commitment
                            == &current.observation.authority_record_commitment
                        && matches!(
                            post_turn.as_ref(),
                            crate::execution::agent_runtime::host_session_authority::store_schema::HostSessionPostTurnApplicationV2::Pending { .. }
                                | crate::execution::agent_runtime::host_session_authority::store_schema::HostSessionPostTurnApplicationV2::AwaitingObligationCut { .. }
                        )
                        && in_flight_resume.replace(intent).is_some()
                    {
                        anyhow::bail!(runtime_start_error(anyhow::anyhow!(
                            "current HSA authority has ambiguous in-flight public ResumeOneTurn transitions"
                        )));
                    }
                }
                let mut resume_required = matches!(
                    current.authority.lifecycle_posture,
                    HostSessionPostureV1::ParkedResumable
                        | HostSessionPostureV1::DetachedReconciled
                        | HostSessionPostureV1::AwaitingAttention
                        | HostSessionPostureV1::StaleRecoverable
                );
                if let Some(intent) = in_flight_resume {
                    let joined = coordinated_applied_public_successor(
                        &authority,
                        &root,
                        &current,
                        HostSessionTransitionModeV1::ResumeOneTurn,
                        Some(exact_prompt.as_bytes()),
                        &public_successor_guard,
                    )?
                    .ok_or_else(|| {
                        runtime_start_error(anyhow::anyhow!(
                            "in-flight public ResumeOneTurn was not durably applied"
                        ))
                    })?;
                    if joined.intent_id != intent.intent_id {
                        anyhow::bail!(runtime_start_error(anyhow::anyhow!(
                            "in-flight public ResumeOneTurn changed durable request identity"
                        )));
                    }
                    match &intent.input_handoff {
                        HostSessionTransitionInputHandoffV1::Pending { .. } => {
                            resume_required = true;
                        }
                        HostSessionTransitionInputHandoffV1::Accepted { .. } => {
                            anyhow::bail!(config_model::user_error(
                                "submission_outcome_unknown: ResumeOneTurn input was already durably accepted; retry will not resubmit the provider prompt"
                            ));
                        }
                        HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance {
                            ..
                        }
                        | HostSessionTransitionInputHandoffV1::NotApplicable => {
                            anyhow::bail!(runtime_start_error(anyhow::anyhow!(
                                "in-flight public ResumeOneTurn input handoff is not retryable"
                            )));
                        }
                    }
                }
                if resume_required {
                    mark_public_successor_request_in_flight(&mut public_successor_guard)
                        .map_err(runtime_start_error)?;
                    let mut launch = build_ordinary_public_resumed_turn_launch_plan(
                        &ordinary_source,
                        &args.session,
                        &exact_prompt,
                        &public_successor_guard,
                    )?;
                    if launch.helper_plan.descriptor.backend_id != backend_id {
                        anyhow::bail!(config_model::user_error(format!(
                            "backend_not_in_session: orchestration session {} has no exact backend slot for {}",
                            args.session, args.backend
                        )));
                    }
                    let launch_permit = acquire_authority_managed_successor_launch_permit(&launch)
                        .map_err(runtime_start_error)?;
                    if launch_permit.joined_completed_launch() {
                        wait_for_authority_managed_successor_completion(&launch)
                            .map_err(runtime_start_error)?;
                        acknowledge_public_successor_request(&public_successor_guard)
                            .map_err(runtime_start_error)?;
                        return Ok(());
                    }
                    let startup_listener = register_durable_start_startup_prompt_listener(
                        launch.helper_plan.orchestration_session_id(),
                        launch.helper_plan.participant_id(),
                    )
                    .map_err(runtime_start_error)?;
                    launch.helper_plan.startup_prompt = Some(HiddenOwnerHelperStartupPromptPlan {
                        prompt_text: exact_prompt,
                        stream_path: startup_listener.path().to_path_buf(),
                        request_key_sha256: None,
                        prompt_sha256: None,
                        public_backend_id: None,
                        public_scope: None,
                    });
                    let applied_selection = Arc::new(ordinary_source.refresh_original_selection()?);
                    let carrier = crate::execution::install_bootstrap::checked_install_bootstrap_context_from_projections()?;
                    let _successor_launch = crate::execution::agent_runtime::control::launch_ordinary_successor_owner_helper(
                        &applied_selection, &carrier, &launch, launch_permit, cli.world, cli.no_world,
                    )
                    .map_err(runtime_start_error)?;
                    run_hidden_owner_helper_startup_prompt_stream_with_public_identity(
                        startup_listener,
                        args.json,
                        PublicPromptAction::Turn,
                        &ordinary_source.requested_selection().backend_id,
                        match ordinary_source.requested_selection().placement {
                            AgentSelectionPlacementV1::Host => AgentExecutionScope::Host,
                            AgentSelectionPlacementV1::World => AgentExecutionScope::World,
                        },
                    )
                    .map_err(normalize_public_prompt_error)?;
                    wait_for_authority_managed_successor_completion(&launch)
                        .map_err(runtime_start_error)?;
                    acknowledge_public_successor_request(&public_successor_guard)
                        .map_err(runtime_start_error)?;
                    return Ok(());
                }
                match current.authority.lifecycle_posture {
                    HostSessionPostureV1::ActiveAttached => Some((
                        current
                            .authority
                            .active_authoritative_participant_id
                            .clone()
                            .ok_or_else(|| {
                                runtime_start_error(anyhow::anyhow!(
                                    "active HSA authority has no authoritative participant"
                                ))
                            })?,
                        backend_id,
                    )),
                    HostSessionPostureV1::Terminal | HostSessionPostureV1::Invalid => {
                        anyhow::bail!(config_model::user_error(format!(
                            "owner_unreachable: orchestration session {} backend {} is terminal and cannot accept follow-up turns",
                            args.session, args.backend
                        )));
                    }
                    HostSessionPostureV1::ParkedResumable
                    | HostSessionPostureV1::DetachedReconciled
                    | HostSessionPostureV1::AwaitingAttention
                    | HostSessionPostureV1::StaleRecoverable => unreachable!(
                        "resumable HSA postures are routed before active target resolution"
                    ),
                }
            } else {
                None
            }
        } else {
            None
        }
    };

    #[cfg(target_os = "linux")]
    if let Some((_participant_id, backend_id)) = hsa_active_identity.as_ref() {
        run_public_prompt_command(
            PublicPromptCommandRequest {
                action: PublicPromptAction::Turn,
                orchestration_session_id: Some(args.session.clone()),
                backend_id: backend_id.clone(),
                prompt: prompt.clone(),
                json: args.json,
            },
            cli.world,
            cli.no_world,
        )
        .map_err(normalize_public_prompt_error)?;
        acknowledge_public_successor_request(&public_successor_guard)
            .map_err(runtime_start_error)?;
        return Ok(());
    }

    let store = AgentRuntimeStateStore::new()?;
    let target = store
        .resolve_public_turn_target(&args.session, &args.backend)
        .map_err(|err| config_model::user_error(err.to_string()))?;
    let orchestration_session_id = target.session.orchestration_session_id.clone();
    let backend_id = target.participant.handle.backend_id.clone();
    #[cfg(target_os = "linux")]
    if let Some((hsa_participant_id, hsa_backend_id)) = hsa_active_identity.as_ref() {
        if target.participant.handle.participant_id != *hsa_participant_id
            || backend_id != *hsa_backend_id
        {
            anyhow::bail!(runtime_start_error(anyhow::anyhow!(
                "active public turn target does not match exact current HSA identity"
            )));
        }
    }
    #[cfg(all(unix, not(target_os = "linux")))]
    let exact_backend_id = backend_id.clone();

    #[cfg(all(unix, not(target_os = "linux")))]
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
        let context = resolve_command_context(cli)?;
        let host_toolbox_surface_authoritative = authoritative_host_toolbox_surface_enabled(
            &plan.descriptor.backend_id,
            plan.descriptor.execution_scope,
            ORCHESTRATOR_ROLE,
            &context.effective_config,
            &context.base_policy,
        );
        plan.startup_prompt = Some(HiddenOwnerHelperStartupPromptPlan {
            prompt_text: maybe_compose_prompt_with_authoritative_host_toolbox_contract(
                &prompt.prompt_text,
                host_toolbox_surface_authoritative,
            ),
            stream_path: startup_listener.path().to_path_buf(),
            request_key_sha256: None,
            prompt_sha256: None,
            public_backend_id: None,
            public_scope: None,
        });
        let receipt = launch_hidden_owner_helper(&plan, cli.world, cli.no_world)
            .map_err(runtime_start_error)?;
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
            &exact_backend_id,
        )
        .map_err(runtime_start_error)?;
        return Ok(());
    }

    let resumed_receipt: Option<HiddenOwnerHelperLaunchReceipt> = match target.session_posture {
        PublicSessionPosture::Active => None,
        PublicSessionPosture::DetachedReattachable => match target.target_kind {
            PublicTurnTargetKind::Host => {
                #[cfg(all(unix, not(target_os = "linux")))]
                unreachable!("detached host turns are handled above");
                #[cfg(target_os = "linux")]
                anyhow::bail!(config_model::user_error(format!(
                    "unsupported_platform_or_posture: orchestration session {} backend {} has no exact HSA ResumeOneTurn authority",
                    args.session, args.backend
                )));
                #[cfg(not(unix))]
                anyhow::bail!(config_model::user_error(format!(
                    "unsupported_platform_or_posture: orchestration session {} backend {} is detached",
                    args.session, args.backend
                )));
            }
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
    #[cfg(not(all(unix, not(target_os = "linux"))))]
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

    #[cfg(all(unix, not(target_os = "linux")))]
    if let Some(receipt) = resumed_receipt.as_ref() {
        wait_for_resumed_public_turn_detach(
            &store,
            &receipt.orchestration_session_id,
            &receipt.participant_id,
            &exact_backend_id,
        )
        .map_err(runtime_start_error)?;
    }

    #[cfg(target_os = "linux")]
    acknowledge_public_successor_request(&public_successor_guard).map_err(runtime_start_error)?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn run_reattach(args: &AgentSessionControlArgs, cli: &Cli) -> Result<()> {
    let store = AgentRuntimeStateStore::new()?;
    let mut public_successor_guard = acquire_public_successor_request_guard(
        &substrate_paths::substrate_home()?,
        &args.session,
        HostSessionTransitionModeV1::Attach,
        &public_start_digest(
            "substrate.public-successor-request.v1",
            &[args.session.as_bytes(), b"reattach"],
        ),
    )
    .map_err(runtime_start_error)?;
    mark_public_successor_request_in_flight(&mut public_successor_guard)
        .map_err(runtime_start_error)?;
    let plan = build_attach_launch_plan_with_public_coordination(
        &args.session,
        AttachLaunchIntent::Reattach,
        &public_successor_guard,
    )?;
    let launch_permit =
        acquire_authority_managed_successor_launch_permit(&plan).map_err(runtime_start_error)?;
    let has_legacy_control_target = store
        .resolve_public_control_target(&args.session, PublicControlAction::Stop)
        .is_ok();
    let claimed_auto_attach = if has_legacy_control_target {
        match store
            .claim_session_auto_attach(&args.session, "manual::reattach")
            .map_err(runtime_start_error)?
        {
            SessionAutoAttachClaim::Claimed {
                obligation_id,
                attach_claim_owner,
            } => Some((obligation_id, attach_claim_owner)),
            SessionAutoAttachClaim::NoCandidate { .. }
            | SessionAutoAttachClaim::AlreadyClaimed { .. } => None,
        }
    } else {
        None
    };
    let successor_launch = match launch_authority_managed_successor_owner_helper(
        &plan,
        launch_permit,
        cli.world,
        cli.no_world,
    ) {
        Ok(receipt) => receipt,
        Err(err) => {
            if let Err(release_err) = release_manual_reattach_auto_attach_claim(
                &store,
                &args.session,
                claimed_auto_attach.as_ref(),
            ) {
                return Err(runtime_start_error(anyhow::anyhow!(
                    "manual reattach launch failed: {err}; additionally failed to release the session auto-attach claim: {release_err}"
                )));
            }
            return Err(runtime_start_error(err));
        }
    };
    let receipt = &successor_launch.launch_receipt;
    if receipt.orchestration_session_id != args.session {
        anyhow::bail!(runtime_start_error(anyhow::anyhow!(
            "authority-managed owner-helper attached orchestration session {} instead of requested session {}",
            receipt.orchestration_session_id,
            args.session
        )));
    }
    if let Err(err) = wait_for_authority_managed_successor_completion(&plan) {
        if let Err(release_err) = release_manual_reattach_auto_attach_claim(
            &store,
            &args.session,
            claimed_auto_attach.as_ref(),
        ) {
            return Err(runtime_start_error(anyhow::anyhow!(
                "manual reattach authority reconciliation failed: {err}; additionally failed to release the session auto-attach claim: {release_err}"
            )));
        }
        return Err(runtime_start_error(err));
    }

    if has_legacy_control_target {
        store
            .settle_session_auto_attach_after_attach_restored(
                &receipt.orchestration_session_id,
                MANUAL_REATTACH_ATTACH_RESTORED_REASON,
            )
            .map_err(runtime_start_error)?;
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
    )?;
    acknowledge_public_successor_request(&public_successor_guard).map_err(runtime_start_error)
}

#[cfg(not(target_os = "linux"))]
fn run_reattach(args: &AgentSessionControlArgs, cli: &Cli) -> Result<()> {
    let store = AgentRuntimeStateStore::new()?;
    let plan = build_attach_launch_plan(&args.session, AttachLaunchIntent::Reattach)?;
    let claimed_auto_attach = match store
        .claim_session_auto_attach(&args.session, "manual::reattach")
        .map_err(runtime_start_error)?
    {
        SessionAutoAttachClaim::Claimed {
            obligation_id,
            attach_claim_owner,
        } => Some((obligation_id, attach_claim_owner)),
        SessionAutoAttachClaim::NoCandidate { .. }
        | SessionAutoAttachClaim::AlreadyClaimed { .. } => None,
    };
    let receipt = match launch_hidden_owner_helper(&plan, cli.world, cli.no_world) {
        Ok(receipt) => receipt,
        Err(err) => {
            if let Err(release_err) = release_manual_reattach_auto_attach_claim(
                &store,
                &args.session,
                claimed_auto_attach.as_ref(),
            ) {
                return Err(runtime_start_error(anyhow::anyhow!(
                    "manual reattach launch failed: {err}; additionally failed to release the session auto-attach claim: {release_err}"
                )));
            }
            return Err(runtime_start_error(err));
        }
    };
    if receipt.orchestration_session_id != args.session {
        anyhow::bail!(runtime_start_error(anyhow::anyhow!(
            "hidden owner-helper attached orchestration session {} instead of requested session {}",
            receipt.orchestration_session_id,
            args.session
        )));
    }

    verify_manual_reattach_restored_ownership(
        &store,
        &args.session,
        &receipt,
        claimed_auto_attach.as_ref(),
    )?;
    store
        .settle_session_auto_attach_after_attach_restored(
            &receipt.orchestration_session_id,
            MANUAL_REATTACH_ATTACH_RESTORED_REASON,
        )
        .map_err(runtime_start_error)?;

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

fn release_manual_reattach_auto_attach_claim(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    claimed_auto_attach: Option<&(String, String)>,
) -> Result<()> {
    let Some((obligation_id, attach_claim_owner)) = claimed_auto_attach else {
        return Ok(());
    };
    let _ = store.release_session_auto_attach_claim(
        orchestration_session_id,
        obligation_id,
        attach_claim_owner,
    )?;
    Ok(())
}

fn verify_manual_reattach_restored_ownership(
    store: &AgentRuntimeStateStore,
    requested_session_id: &str,
    receipt: &HiddenOwnerHelperLaunchReceipt,
    claimed_auto_attach: Option<&(String, String)>,
) -> Result<()> {
    let target = match store
        .resolve_public_control_target(&receipt.orchestration_session_id, PublicControlAction::Stop)
    {
        Ok(target) => target,
        Err(err) => {
            if let Err(release_err) = release_manual_reattach_auto_attach_claim(
                store,
                requested_session_id,
                claimed_auto_attach,
            ) {
                return Err(runtime_start_error(anyhow::anyhow!(
                    "manual reattach durable-ownership verification failed: {err}; additionally failed to release the session auto-attach claim: {release_err}"
                )));
            }
            return Err(runtime_start_error(err));
        }
    };
    if target.session_posture != PublicSessionPosture::Active
        || target.active_participant.handle.participant_id != receipt.participant_id
    {
        if let Err(release_err) = release_manual_reattach_auto_attach_claim(
            store,
            requested_session_id,
            claimed_auto_attach,
        ) {
            return Err(runtime_start_error(anyhow::anyhow!(
                "manual reattach durable-ownership verification failed: orchestration session {} did not reach durable active_attached ownership after reattach; additionally failed to release the session auto-attach claim: {release_err}",
                receipt.orchestration_session_id
            )));
        }
        anyhow::bail!(runtime_start_error(anyhow::anyhow!(
            "orchestration session {} did not reach durable active_attached ownership after reattach",
            receipt.orchestration_session_id
        )));
    }

    Ok(())
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

fn dispatch_resolution_user_error(
    classifier: &str,
    err: crate::execution::agent_runtime::DispatchResolutionError,
) -> anyhow::Error {
    config_model::user_error(format!("{classifier}: {err}"))
}

fn runtime_materialization_user_error(
    classifier: &str,
    reason: impl Into<String>,
) -> anyhow::Error {
    let reason = reason.into();
    let field = if reason.contains("config.kind=") {
        "config.kind"
    } else if reason.contains("cli.mode=") {
        "cli.mode"
    } else if reason.contains("config.cli.binary") {
        "config.cli.binary"
    } else {
        "runtime_descriptor"
    };
    config_model::user_error(format!(
        "{classifier}: runtime materialization rejected field '{field}': {reason}"
    ))
}

fn start_dispatch_classifier(
    err: &crate::execution::agent_runtime::DispatchResolutionError,
) -> &'static str {
    use crate::execution::agent_runtime::DispatchResolutionErrorKind;

    match err.kind {
        DispatchResolutionErrorKind::AmbiguousBaselineSelection => "ambiguous_backend",
        DispatchResolutionErrorKind::OverrideDeniedByPolicy => "policy_disallow",
        DispatchResolutionErrorKind::BaselineNotFound => "unknown_backend",
        DispatchResolutionErrorKind::BaselineIneligible
        | DispatchResolutionErrorKind::InvalidPolicyOverlay
        | DispatchResolutionErrorKind::RuntimeUnrealizableAfterResolution
        | DispatchResolutionErrorKind::UnknownOverrideFamily
        | DispatchResolutionErrorKind::OverrideNotSupportedForCaller
        | DispatchResolutionErrorKind::OverrideExceedsBaseline
        | DispatchResolutionErrorKind::MissingRequiredAttachContinuity => "runtime_start_failed",
    }
}

fn build_start_capability_overrides(
    disabled_capabilities: &[AgentDisableCapabilityArg],
) -> DispatchCapabilityOverrideSet {
    let mut overrides = DispatchCapabilityOverrideSet::default();
    for capability in disabled_capabilities {
        match capability {
            AgentDisableCapabilityArg::SessionResume => overrides.session_resume = Some(false),
            AgentDisableCapabilityArg::SessionFork => overrides.session_fork = Some(false),
            AgentDisableCapabilityArg::SessionStop => overrides.session_stop = Some(false),
            AgentDisableCapabilityArg::StatusSnapshot => overrides.status_snapshot = Some(false),
            AgentDisableCapabilityArg::EventStream => overrides.event_stream = Some(false),
        }
    }
    overrides
}

fn start_scope_from_arg(scope: AgentStartScopeArg) -> AgentExecutionScope {
    match scope {
        AgentStartScopeArg::Host => AgentExecutionScope::Host,
        AgentStartScopeArg::World => AgentExecutionScope::World,
    }
}

fn alternate_execution_scope(scope: AgentExecutionScope) -> AgentExecutionScope {
    match scope {
        AgentExecutionScope::Host => AgentExecutionScope::World,
        AgentExecutionScope::World => AgentExecutionScope::Host,
    }
}

fn host_execution_client_start_for_scope(
    requested_execution_scope: AgentExecutionScope,
) -> HostExecutionClientStart {
    match requested_execution_scope {
        AgentExecutionScope::Host => HostExecutionClientStart::StartNow,
        AgentExecutionScope::World => HostExecutionClientStart::Defer,
    }
}

fn require_start_backend_id(args: &AgentStartArgs) -> Result<&str> {
    let backend_id = args.backend.as_deref().unwrap_or("").trim();
    if backend_id.is_empty() {
        anyhow::bail!(config_model::user_error(
            "unknown_backend: an explicit --backend must name a nonempty backend id"
        ));
    }
    Ok(backend_id)
}

fn build_start_dispatch_envelope(
    args: &AgentStartArgs,
    backend_id: &str,
    requested_execution_scope: AgentExecutionScope,
) -> DispatchRequestEnvelope {
    DispatchRequestEnvelope {
        caller_kind: DispatchCallerKind::HumanStart,
        baseline_kind: DispatchBaselineKind::InventoryLaunch,
        backend_id: Some(backend_id.to_string()),
        orchestration_session_id: None,
        requested_execution_scope_override: Some(requested_execution_scope),
        capability_overrides: build_start_capability_overrides(&args.disable_capability),
        attach_launch_knobs: AttachLaunchKnobs {
            requested_execution_scope,
            host_execution_client_start: host_execution_client_start_for_scope(
                requested_execution_scope,
            ),
            attach_mode_preference: AttachModePreference::ContinuityRequired,
        },
        has_prompt_payload: true,
    }
}

fn build_persisted_attach_dispatch_envelope(
    caller_kind: DispatchCallerKind,
    orchestration_session_id: &str,
    backend_id: &str,
    host_execution_client_start: HostExecutionClientStart,
    attach_mode_preference: AttachModePreference,
    has_prompt_payload: bool,
) -> DispatchRequestEnvelope {
    DispatchRequestEnvelope {
        caller_kind,
        baseline_kind: DispatchBaselineKind::PersistedHostAttach,
        backend_id: Some(backend_id.to_string()),
        orchestration_session_id: Some(orchestration_session_id.to_string()),
        requested_execution_scope_override: None,
        capability_overrides: DispatchCapabilityOverrideSet::default(),
        attach_launch_knobs: AttachLaunchKnobs {
            requested_execution_scope: AgentExecutionScope::Host,
            host_execution_client_start,
            attach_mode_preference,
        },
        has_prompt_payload,
    }
}

fn probe_start_scope_candidate(
    args: &AgentStartArgs,
    context: &AgentCommandContext,
    cwd: &Path,
    backend_id: &str,
    requested_execution_scope: AgentExecutionScope,
) -> Result<Option<ResolvedLaunchContract>> {
    let envelope = build_start_dispatch_envelope(args, backend_id, requested_execution_scope);
    resolve_inventory_contract_for_exact_backend(
        cwd,
        &context.effective_config,
        &context.inventory,
        &context.base_policy,
        &envelope,
        requested_execution_scope,
    )
    .map_err(|err| dispatch_resolution_user_error(start_dispatch_classifier(&err), err))
}

fn resolve_requested_start_scope(
    args: &AgentStartArgs,
    context: &AgentCommandContext,
) -> Result<AgentExecutionScope> {
    if let Some(scope) = args.scope {
        return Ok(start_scope_from_arg(scope));
    }

    let preferred_scope = context.effective_config.agents.defaults.execution.scope;
    if args.backend.is_none() {
        return Ok(preferred_scope);
    }
    let backend_id = require_start_backend_id(args)?;
    let cwd = current_dir();
    if probe_start_scope_candidate(args, context, &cwd, backend_id, preferred_scope)?.is_some() {
        return Ok(preferred_scope);
    }

    let fallback_scope = alternate_execution_scope(preferred_scope);
    if probe_start_scope_candidate(args, context, &cwd, backend_id, fallback_scope)?.is_some() {
        return Ok(fallback_scope);
    }

    anyhow::bail!(config_model::user_error(format!(
        "unknown_backend: baseline truth rejected field 'backend_id': no exact backend match found for '{}' after resolving omitted --scope through effective {} then {} defaults",
        backend_id,
        execution_scope_label(preferred_scope),
        execution_scope_label(fallback_scope),
    )));
}

fn permissive_inventory_selection_policy(
    inventory: &BTreeMap<String, AgentInventoryEntryV1>,
) -> Policy {
    Policy {
        agents_allowed_backends: inventory
            .values()
            .map(AgentInventoryEntryV1::derived_backend_id)
            .collect(),
        ..Policy::default()
    }
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
    #[cfg(target_os = "linux")]
    if let Some(authority) = hsa_stop_authority_for_session(&args.session)? {
        return run_hsa_stop(args, &authority);
    }

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
                        "owner_unreachable: recovery_failed: timed out waiting for orchestration session {} to reach a terminal state",
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
            .map_err(|err| config_model::user_error(format!("owner_unreachable: {err:#}")))?;
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
                "owner_unreachable: recovery_failed: timed out waiting for orchestration session {} to reach a terminal state",
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

#[cfg(target_os = "linux")]
fn hsa_stop_authority_for_session(
    orchestration_session_id: &str,
) -> Result<Option<HostSessionAuthority>> {
    let authority = public_start_authority()?;
    if authority
        .resolve_current_exact(orchestration_session_id, None)
        .is_ok()
    {
        return Ok(Some(authority));
    }
    match authority.read_a12b_root() {
        Ok(root)
            if root
                .session_namespace_map
                .contains_key(orchestration_session_id) =>
        {
            Err(config_model::user_error(
                "authority_invalid: discovered HSA Stop session failed exact authority resolution",
            ))
        }
        Ok(_) => Ok(None),
        Err(strict_error) => {
            let greenfield_v1 = authority.read_root().is_ok_and(|root| {
                crate::execution::agent_runtime::host_session_authority::start_continuity::retryable_start_transaction_from_v1(&root).is_ok()
            });
            if greenfield_v1 {
                Ok(None)
            } else {
                Err(config_model::user_error(format!(
                    "authority_invalid: cannot resolve strict HSA Stop authority: {strict_error}"
                )))
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn run_hsa_stop(args: &AgentSessionControlArgs, authority: &HostSessionAuthority) -> Result<()> {
    let current = authority
        .resolve_current_exact(&args.session, None)
        .map_err(|error| config_model::user_error(error.to_string()))?;
    let backend_id = current.caller.descriptor.backend_id.clone();
    let participant_id = current.caller.participant_id.clone();

    if current.authority.lifecycle_posture == HostSessionPostureV1::Terminal {
        return render_hsa_stop_result(args, &backend_id, &participant_id);
    }
    let intent = match authority
        .stop_transaction_for_session(&args.session)
        .map_err(|error| config_model::user_error(error.to_string()))?
    {
        Some(intent) => intent,
        None => {
            let authority_revision = current.observation.authority_revision.to_string();
            let request_digest = public_start_digest(
                "substrate.hsa-public-stop-request.v1",
                &[
                    current.observation.authority_store_id.as_bytes(),
                    args.session.as_bytes(),
                    participant_id.as_bytes(),
                    authority_revision.as_bytes(),
                ],
            );
            let request = IssueHostSessionStopRequestV1 {
                intent_id: format!("stop-intent-{request_digest}"),
                request_id: format!("stop-request-{request_digest}"),
                orchestration_session_id: args.session.clone(),
                caller: HostSessionTransitionCallerV1 {
                    kind: HostSessionTransitionCallerKindV1::PublicCli,
                    caller_participant_id: None,
                    auto_attach_obligation_id: None,
                    auto_attach_claim_owner: None,
                },
                expected_authority: current.observation.clone(),
                authoritative_participant_id: participant_id.clone(),
                authoritative_lineage: current.authority.authoritative_participant_lineage.clone(),
                issued_at: current_timestamp_v1()?,
            };
            match authority
                .issue_stop(&current, &request)
                .map_err(|error| config_model::user_error(error.to_string()))?
            {
                HostSessionStopIssueOutcomeV1::Issued(intent)
                | HostSessionStopIssueOutcomeV1::Joined(intent) => intent,
                HostSessionStopIssueOutcomeV1::AlreadyTerminal(_) => {
                    return render_hsa_stop_result(args, &backend_id, &participant_id)
                }
            }
        }
    };

    match &intent.state {
        HostSessionStopIntentStateV1::Completed { .. } => {
            return render_hsa_stop_result(args, &backend_id, &participant_id);
        }
        HostSessionStopIntentStateV1::Issued
            if intent.authority_before.lifecycle_posture
                != HostSessionPostureV1::ActiveAttached =>
        {
            let completion = CompleteHostSessionStopRequestV1 {
                intent_id: intent.intent_id.clone(),
                request_id: intent.request_id.clone(),
                payload_commitment: intent.payload_commitment.clone(),
                orchestration_session_id: intent.orchestration_session_id.clone(),
                authoritative_participant_id: intent.authoritative_participant_id.clone(),
                caller: HostSessionTransitionCallerV1 {
                    kind: HostSessionTransitionCallerKindV1::PublicCli,
                    caller_participant_id: None,
                    auto_attach_obligation_id: None,
                    auto_attach_claim_owner: None,
                },
                delivery_acceptance_id: None,
                result_id: format!("{}:result", intent.intent_id),
                completed_at: current_timestamp_v1()?,
            };
            match authority
                .complete_stop(&completion)
                .map_err(|error| config_model::user_error(error.to_string()))?
            {
                HostSessionStopCompletionOutcomeV1::Completed(_)
                | HostSessionStopCompletionOutcomeV1::Joined(_) => {
                    return render_hsa_stop_result(args, &backend_id, &participant_id)
                }
            }
        }
        HostSessionStopIntentStateV1::Issued
        | HostSessionStopIntentStateV1::DeliveryAccepted { .. } => {}
    }

    if intent.authority_before.lifecycle_posture != HostSessionPostureV1::ActiveAttached {
        anyhow::bail!(config_model::user_error(
            "authority_conflict: non-active HSA Stop unexpectedly requires private delivery"
        ));
    }
    let transport_path = durable_start_stop_transport_path(&args.session, &participant_id)
        .map_err(|error| config_model::user_error(error.to_string()))?;
    let delivery = stop_delivery(&intent);
    let runtime = TokioRuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to initialize HSA Stop transport runtime")?;
    let outcome = runtime
        .block_on(request_private_authority_managed_stop(
            &transport_path,
            &delivery,
        ))
        .map_err(|error| config_model::user_error(format!("owner_unreachable: {error:#}")))?;
    match outcome {
        crate::execution::agent_runtime::control::PrivateStopOutcome::Accepted
        | crate::execution::agent_runtime::control::PrivateStopOutcome::AlreadyTerminal => {}
        crate::execution::agent_runtime::control::PrivateStopOutcome::OwnerUnreachable => {
            anyhow::bail!(config_model::user_error(format!(
                "owner_unreachable: orchestration session {} no longer has a reachable bound HSA owner",
                args.session
            )))
        }
        crate::execution::agent_runtime::control::PrivateStopOutcome::ProtocolError => {
            anyhow::bail!(config_model::user_error(format!(
                "protocol_error: bound HSA owner rejected Stop identity for orchestration session {}",
                args.session
            )))
        }
    }

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let transaction = authority
            .stop_transaction_for_session(&args.session)
            .map_err(|error| config_model::user_error(error.to_string()))?
            .ok_or_else(|| {
                config_model::user_error(
                    "authority_conflict: committed HSA Stop intent disappeared",
                )
            })?;
        if matches!(
            transaction.state,
            HostSessionStopIntentStateV1::Completed { .. }
        ) {
            let resolved = authority
                .resolve_current_exact(&args.session, None)
                .map_err(|error| config_model::user_error(error.to_string()))?;
            if resolved.authority.lifecycle_posture != HostSessionPostureV1::Terminal {
                anyhow::bail!(config_model::user_error(
                    "authority_conflict: HSA Stop result does not resolve durable Terminal authority"
                ));
            }
            return render_hsa_stop_result(args, &backend_id, &participant_id);
        }
        if Instant::now() >= deadline {
            anyhow::bail!(config_model::user_error(format!(
                "owner_unreachable: timed out waiting for authenticated HSA Stop closeout for orchestration session {}",
                args.session
            )));
        }
        thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(target_os = "linux")]
fn render_hsa_stop_result(
    args: &AgentSessionControlArgs,
    backend_id: &str,
    participant_id: &str,
) -> Result<()> {
    render_agent_control_result(
        args.json,
        &AgentControlResultJson {
            action: "stop",
            orchestration_session_id: &args.session,
            backend_id,
            scope: "host",
            state: "terminal",
            warnings: Vec::new(),
            participant_id: Some(participant_id),
            source_orchestration_session_id: None,
        },
    )
}

#[cfg(target_os = "linux")]
fn current_timestamp_v1() -> Result<TimestampV1> {
    TimestampV1::parse(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}

fn build_start_launch_plan(
    args: &AgentStartArgs,
    context: &AgentCommandContext,
) -> Result<HostStartLaunchPlan> {
    match resolve_requested_start_scope(args, context)? {
        AgentExecutionScope::Host => build_host_start_launch_plan(args, context),
        AgentExecutionScope::World => build_world_start_session_birth_plan(args, context),
    }
}

fn resolve_start_selection(
    args: &AgentStartArgs,
    context: &AgentCommandContext,
    scope: AgentExecutionScope,
) -> Result<ResolvedAgentSelectionV1> {
    let agent = args.agent.as_deref().map(str::trim);
    let backend = args.backend.as_deref().map(str::trim);
    if agent.is_some_and(str::is_empty) || backend.is_some_and(str::is_empty) {
        return Err(config_model::user_error(
            "explicit agent/backend selection must be nonempty",
        ));
    }
    context
        .selection_snapshot
        .resolve_selection(&context.effective_config, agent, backend, scope)
}

fn validate_start_selection_contract(
    selection: &ResolvedAgentSelectionV1,
    contract: &ResolvedLaunchContract,
) -> Result<()> {
    use crate::execution::agent_runtime::mapping::AgentRuntimeBackendKind;
    use transport_api_types::AgentHarnessFamilyV1;
    selection.validate().map_err(config_model::user_error)?;
    let family = match selection.selection.harness {
        AgentHarnessFamilyV1::Codex => AgentRuntimeBackendKind::Codex,
        AgentHarnessFamilyV1::ClaudeCode => AgentRuntimeBackendKind::ClaudeCode,
    };
    let placement = match contract.execution_scope {
        AgentExecutionScope::Host => AgentSelectionPlacementV1::Host,
        AgentExecutionScope::World => AgentSelectionPlacementV1::World,
    };
    if selection.backend_id != contract.backend_id
        || selection.realized_agent_id != contract.agent_id
        || family != contract.backend_kind
        || selection.placement != placement
    {
        return Err(config_model::user_error(
            "complete selection conflicts with the policy-validated runtime contract",
        ));
    }
    Ok(())
}

fn build_host_start_launch_plan(
    args: &AgentStartArgs,
    context: &AgentCommandContext,
) -> Result<HostStartLaunchPlan> {
    let selection = resolve_start_selection(args, context, AgentExecutionScope::Host)?;
    let backend_id = selection.backend_id.as_str();

    let cwd = current_dir();
    let envelope = build_start_dispatch_envelope(args, backend_id, AgentExecutionScope::Host);
    let resolved = match resolve_inventory_contract_for_exact_backend(
        &cwd,
        &context.effective_config,
        &context.inventory,
        &context.base_policy,
        &envelope,
        AgentExecutionScope::Host,
    ) {
        Ok(Some(contract)) => contract,
        Ok(None) => {
            let permissive_policy = permissive_inventory_selection_policy(&context.inventory);
            match resolve_inventory_contract_for_exact_backend(
                &cwd,
                &context.effective_config,
                &context.inventory,
                &permissive_policy,
                &envelope,
                AgentExecutionScope::World,
            ) {
                Ok(Some(_)) => {
                    anyhow::bail!(config_model::user_error(format!(
                        "unsupported_platform_or_posture: baseline truth rejected field 'requested_execution_scope': backend '{}' resolves only to a world-scoped runtime; public root start is host-only in v1",
                        backend_id
                    )));
                }
                Ok(None) => {
                    anyhow::bail!(config_model::user_error(format!(
                        "unknown_backend: baseline truth rejected field 'backend_id': no exact host-scoped backend match found for '{}'",
                        backend_id
                    )));
                }
                Err(err) => anyhow::bail!(dispatch_resolution_user_error(
                    start_dispatch_classifier(&err),
                    err,
                )),
            }
        }
        Err(err) => anyhow::bail!(dispatch_resolution_user_error(
            start_dispatch_classifier(&err),
            err,
        )),
    };
    validate_start_selection_contract(&selection, &resolved)?;
    let descriptor = materialize_runtime_descriptor(&resolved)
        .map_err(|err| runtime_materialization_user_error("runtime_start_failed", err.reason))?;

    let orchestration_session_id = Uuid::now_v7().to_string();
    Ok(HostStartLaunchPlan {
        helper_plan: HiddenOwnerHelperLaunchPlan {
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
            host_attach_contract: HostAttachContract::from_resolved_contract(&resolved, None),
            startup_prompt: None,
            source_orchestration_session_id: None,
        },
        resolved_contract: resolved,
        public_identity: StartPromptPublicIdentity {
            backend_id: backend_id.to_string(),
            scope: AgentExecutionScope::Host,
        },
        native_selection: selection.clone(),
        requested_selection: selection,
    })
}

#[allow(dead_code)]
fn build_world_start_host_attach_dispatch_envelope(
    args: &AgentStartArgs,
    backend_id: &str,
) -> DispatchRequestEnvelope {
    DispatchRequestEnvelope {
        caller_kind: DispatchCallerKind::HumanStart,
        baseline_kind: DispatchBaselineKind::InventoryLaunch,
        backend_id: Some(backend_id.to_string()),
        orchestration_session_id: None,
        requested_execution_scope_override: Some(AgentExecutionScope::Host),
        capability_overrides: build_start_capability_overrides(&args.disable_capability),
        attach_launch_knobs: AttachLaunchKnobs {
            requested_execution_scope: AgentExecutionScope::Host,
            host_execution_client_start: HostExecutionClientStart::StartNow,
            attach_mode_preference: AttachModePreference::ContinuityRequired,
        },
        has_prompt_payload: false,
    }
}

#[allow(dead_code)]
fn build_world_start_session_birth_plan(
    args: &AgentStartArgs,
    context: &AgentCommandContext,
) -> Result<HostStartLaunchPlan> {
    let requested_selection = resolve_start_selection(args, context, AgentExecutionScope::World)?;
    let backend_id = requested_selection.backend_id.as_str();

    let cwd = current_dir();
    let world_envelope =
        build_start_dispatch_envelope(args, backend_id, AgentExecutionScope::World);
    let requested_world_contract = resolve_inventory_contract_for_exact_backend(
        &cwd,
        &context.effective_config,
        &context.inventory,
        &context.base_policy,
        &world_envelope,
        AgentExecutionScope::World,
    )
    .map_err(|err| dispatch_resolution_user_error(start_dispatch_classifier(&err), err))?
    .ok_or_else(|| {
        config_model::user_error(format!(
            "unknown_backend: baseline truth rejected field 'backend_id': no exact world-scoped backend match found for '{}'",
            backend_id
        ))
    })?;

    validate_start_selection_contract(&requested_selection, &requested_world_contract)?;
    let orchestrator =
        validate_orchestrator_selection(&context.effective_config, &context.inventory)
            .map_err(config_model::user_error)?;
    let native_selection = context.selection_snapshot.resolve_selection(
        &context.effective_config,
        Some(&orchestrator.file.id),
        None,
        AgentExecutionScope::Host,
    )?;
    let host_backend_id = orchestrator.derived_backend_id();
    let host_attach_envelope =
        build_world_start_host_attach_dispatch_envelope(args, &host_backend_id);
    let host_attach_resolved = resolve_inventory_contract_for_exact_backend(
        &cwd,
        &context.effective_config,
        &context.inventory,
        &context.base_policy,
        &host_attach_envelope,
        AgentExecutionScope::Host,
    )
    .map_err(|err| dispatch_resolution_user_error(start_dispatch_classifier(&err), err))?
    .ok_or_else(|| {
        config_model::user_error(format!(
            "runtime_start_failed: baseline truth rejected field 'backend_id': no exact host-scoped orchestrator backend match found for '{}'",
            host_backend_id
        ))
    })?;
    validate_start_selection_contract(&native_selection, &host_attach_resolved)?;
    let host_attach_contract = HostAttachContract::from_resolved_contract(&host_attach_resolved, None)
        .ok_or_else(|| {
            config_model::user_error(format!(
                "runtime_start_failed: resolved host attach contract for backend '{}' did not remain host scoped",
                host_backend_id
            ))
        })?;
    let descriptor = materialize_runtime_descriptor(&host_attach_resolved)
        .map_err(|err| runtime_materialization_user_error("runtime_start_failed", err.reason))?;

    let orchestration_session_id = Uuid::now_v7().to_string();
    Ok(HostStartLaunchPlan {
        helper_plan: HiddenOwnerHelperLaunchPlan {
            mode: OwnerHelperMode::Start,
            descriptor: (&descriptor).into(),
            session: HiddenOwnerHelperSessionPlan {
                orchestration_session_id,
                shell_trace_session_id: Uuid::now_v7().to_string(),
                workspace_root: cwd.display().to_string(),
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
            host_attach_contract: Some(host_attach_contract),
            startup_prompt: None,
            source_orchestration_session_id: None,
        },
        resolved_contract: host_attach_resolved,
        public_identity: StartPromptPublicIdentity {
            backend_id: requested_world_contract.backend_id,
            scope: AgentExecutionScope::World,
        },
        native_selection,
        requested_selection,
    })
}

#[cfg(target_os = "linux")]
fn establish_public_world_start_binding(
    orchestration_session_id: &str,
    workspace_root: &str,
) -> Result<PersistedWorldBinding> {
    let rt = TokioRuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to initialize world-start launch runtime")?;
    rt.block_on(async {
        establish_public_world_start_binding_async(orchestration_session_id, workspace_root).await
    })
}

#[cfg(target_os = "linux")]
async fn establish_public_world_start_binding_async(
    orchestration_session_id: &str,
    workspace_root: &str,
) -> Result<PersistedWorldBinding> {
    let requested_cwd = workspace_root.to_string();
    let requested_path = PathBuf::from(&requested_cwd);
    let resolved = policy_snapshot::resolve_policy_snapshot_for_cwd(requested_path.as_path())
        .context("policy snapshot (public world start)")?;
    let world_network_policy = policy_snapshot::resolve_world_network_policy_for_snapshot(
        resolved.snapshot,
        requested_path.as_path(),
    )
    .context("world network policy (public world start)")?;
    let world_network = policy_snapshot::request_world_network_routing(&world_network_policy);
    let (mut start_params, _inherit_from_host) = ReplSessionStartParams::for_cwd_and_snapshot(
        requested_cwd,
        requested_path.as_path(),
        world_network_policy.snapshot,
        world_network,
    )
    .context("failed to build shared-world session start parameters")?;
    start_params.shared_world = Some(SharedWorldOwnerSpec {
        orchestration_session_id: orchestration_session_id.to_string(),
        action: SharedWorldOwnerAction::AttachOrCreate,
    });

    let world_client = ReplPersistentSessionClient::start_with(start_params, Arc::new(|_| {}))
        .await
        .context("failed to open authoritative shared world for public world start")?;
    let ready = world_client.ready().clone();
    let authoritative_binding = ready.shared_world.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "runtime_start_failed: shared world ready frame omitted authoritative binding proof"
        )
    })?;
    let world_binding = PersistedWorldBinding {
        world_id: authoritative_binding.world_id.clone(),
        world_generation: authoritative_binding.world_generation,
    };
    let _ = world_client.close().await;
    Ok(world_binding)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AttachLaunchIntent {
    Reattach,
    DetachedTurn,
}

struct PublicAttachRuntimeContinuity {
    source_participant_id: String,
    continuity_session_id: String,
}

impl AttachLaunchIntent {
    fn caller_kind(self) -> DispatchCallerKind {
        match self {
            Self::Reattach => DispatchCallerKind::HumanReattach,
            Self::DetachedTurn => DispatchCallerKind::HumanTurn,
        }
    }

    fn owner_helper_mode(self) -> OwnerHelperMode {
        match self {
            Self::Reattach => OwnerHelperMode::Attach,
            Self::DetachedTurn => OwnerHelperMode::ResumeOneTurn,
        }
    }

    fn public_attach_action(self) -> PublicAttachAction {
        match self {
            Self::Reattach => PublicAttachAction::Reattach,
            Self::DetachedTurn => PublicAttachAction::DetachedTurn,
        }
    }

    fn has_prompt_payload(self) -> bool {
        matches!(self, Self::DetachedTurn)
    }

    fn continuity_label(self) -> &'static str {
        match self {
            Self::Reattach => "control-only reattach",
            Self::DetachedTurn => "detached turn recovery",
        }
    }
}

fn build_public_attach_runtime_continuity(
    orchestration_session_id: &str,
    target: &crate::execution::agent_runtime::state_store::ResolvedPublicAttachTarget,
    attach_contract: &HostAttachContract,
    intent: AttachLaunchIntent,
) -> Result<PublicAttachRuntimeContinuity> {
    let continuity_session_id = attach_contract
        .public_attach_continuity_session_id()
        .ok_or_else(|| {
            config_model::user_error(format!(
                "owner_unreachable: orchestration session {} no longer has continuity required for {}",
                orchestration_session_id,
                intent.continuity_label()
            ))
        })?;

    Ok(PublicAttachRuntimeContinuity {
        source_participant_id: target.active_participant.handle.participant_id.clone(),
        continuity_session_id: continuity_session_id.to_string(),
    })
}

#[cfg(target_os = "linux")]
fn retain_linux_legacy_attach_compatibility_surface(
    store: &AgentRuntimeStateStore,
    target: &crate::execution::agent_runtime::state_store::ResolvedPublicAttachTarget,
    runtime: &PublicAttachRuntimeContinuity,
    receipt: &HiddenOwnerHelperLaunchReceipt,
    intent: AttachLaunchIntent,
) {
    let _ = (&target.active_participant, &target.host_attach_contract);
    let _ = (
        &runtime.source_participant_id,
        &runtime.continuity_session_id,
    );
    let _ = store.resolve_public_attach_target("", intent.public_attach_action());
    let _ = verify_manual_reattach_restored_ownership(store, "", receipt, None);
    let _ = (
        intent.caller_kind(),
        intent.has_prompt_payload(),
        intent.continuity_label(),
    );
}

#[cfg(target_os = "linux")]
const _: fn(
    &AgentRuntimeStateStore,
    &crate::execution::agent_runtime::state_store::ResolvedPublicAttachTarget,
    &PublicAttachRuntimeContinuity,
    &HiddenOwnerHelperLaunchReceipt,
    AttachLaunchIntent,
) = retain_linux_legacy_attach_compatibility_surface;

#[cfg(target_os = "linux")]
const _: fn(
    &str,
    &crate::execution::agent_runtime::state_store::ResolvedPublicAttachTarget,
    &HostAttachContract,
    AttachLaunchIntent,
) -> Result<PublicAttachRuntimeContinuity> = build_public_attach_runtime_continuity;

#[cfg(target_os = "linux")]
const _: fn(
    DispatchCallerKind,
    &str,
    &str,
    HostExecutionClientStart,
    AttachModePreference,
    bool,
) -> DispatchRequestEnvelope = build_persisted_attach_dispatch_envelope;

#[cfg(target_os = "linux")]
fn build_attach_launch_plan(
    orchestration_session_id: &str,
    intent: AttachLaunchIntent,
) -> Result<AuthorityManagedSuccessorLaunchPlanV1> {
    let store = AgentRuntimeStateStore::new()?;
    build_attach_launch_plan_with_store(&store, orchestration_session_id, intent)
}

#[cfg(target_os = "linux")]
const _: fn(&str, AttachLaunchIntent) -> Result<AuthorityManagedSuccessorLaunchPlanV1> =
    build_attach_launch_plan;

#[cfg(target_os = "linux")]
fn build_attach_launch_plan_with_store(
    _store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    intent: AttachLaunchIntent,
) -> Result<AuthorityManagedSuccessorLaunchPlanV1> {
    build_authority_managed_successor_launch_plan(orchestration_session_id, intent, None, None)
}

#[cfg(target_os = "linux")]
fn build_attach_launch_plan_with_public_coordination(
    orchestration_session_id: &str,
    intent: AttachLaunchIntent,
    coordination: &PublicSuccessorRequestGuardV1,
) -> Result<AuthorityManagedSuccessorLaunchPlanV1> {
    build_authority_managed_successor_launch_plan_with_stale_retry(
        orchestration_session_id,
        intent,
        None,
        Some(coordination),
    )
}

#[cfg(target_os = "linux")]
fn build_authority_managed_successor_launch_plan_with_stale_retry(
    orchestration_session_id: &str,
    launch_intent: AttachLaunchIntent,
    transition_input: Option<&str>,
    coordination: Option<&PublicSuccessorRequestGuardV1>,
) -> Result<AuthorityManagedSuccessorLaunchPlanV1> {
    const ATTEMPTS: usize = 8;
    const RETRY_INTERVAL: Duration = Duration::from_millis(10);

    for attempt in 0..ATTEMPTS {
        match build_authority_managed_successor_launch_plan(
            orchestration_session_id,
            launch_intent,
            transition_input,
            coordination,
        ) {
            Ok(plan) => return Ok(plan),
            Err(error)
                if attempt + 1 < ATTEMPTS
                    && error
                        .chain()
                        .any(|cause| cause.to_string() == "stale authority root revision") =>
            {
                thread::sleep(RETRY_INTERVAL);
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!("bounded successor-plan retry loop always returns")
}

#[cfg(not(target_os = "linux"))]
fn build_attach_launch_plan(
    orchestration_session_id: &str,
    intent: AttachLaunchIntent,
) -> Result<HiddenOwnerHelperLaunchPlan> {
    let store = AgentRuntimeStateStore::new()?;
    build_attach_launch_plan_with_store(&store, orchestration_session_id, intent)
}

#[cfg(not(target_os = "linux"))]
fn build_attach_launch_plan_with_store(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    intent: AttachLaunchIntent,
) -> Result<HiddenOwnerHelperLaunchPlan> {
    let mut target = store
        .resolve_public_attach_target(orchestration_session_id, intent.public_attach_action())
        .map_err(|err| config_model::user_error(err.to_string()))?;
    #[cfg(target_os = "linux")]
    refresh_public_attach_target_world_binding(store, &mut target)?;
    let attach_contract = target.host_attach_contract.clone().ok_or_else(|| {
        config_model::user_error(format!(
            "owner_unreachable: orchestration session {} is missing durable host attach contract state",
            orchestration_session_id
        ))
    })?;
    let envelope = build_persisted_attach_dispatch_envelope(
        intent.caller_kind(),
        orchestration_session_id,
        &attach_contract.backend_id,
        HostExecutionClientStart::StartNow,
        AttachModePreference::ContinuityRequired,
        intent.has_prompt_payload(),
    );
    let resolved = resolve_persisted_host_attach_contract(&envelope, &attach_contract)
        .map_err(|err| dispatch_resolution_user_error("owner_unreachable", err))?;
    let descriptor = materialize_runtime_descriptor(&resolved)
        .map_err(|err| runtime_materialization_user_error("owner_unreachable", err.reason))?;
    let attach_runtime_continuity = build_public_attach_runtime_continuity(
        orchestration_session_id,
        &target,
        &attach_contract,
        intent,
    )?;

    Ok(HiddenOwnerHelperLaunchPlan {
        mode: intent.owner_helper_mode(),
        descriptor: (&descriptor).into(),
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
            resumed_from_participant_id: Some(attach_runtime_continuity.source_participant_id),
            internal_uaa_session_id: Some(attach_runtime_continuity.continuity_session_id),
        },
        host_attach_contract: Some(attach_contract),
        startup_prompt: None,
        source_orchestration_session_id: None,
    })
}

#[cfg(target_os = "linux")]
fn exact_settled_start_continuation(
    authority: &HostSessionAuthority,
    root: &crate::execution::agent_runtime::host_session_authority::store_schema::StateRootV3,
    resolved: &crate::execution::agent_runtime::host_session_authority::ResolvedCurrentAuthorityV1,
) -> Result<(
    crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
    String,
)> {
    let owner = resolved
        .authority
        .active_authoritative_participant_id
        .as_deref()
        .context("current continuation source has no authoritative participant")?;
    if owner != resolved.caller.participant_id {
        anyhow::bail!("current continuation source differs from exact caller");
    }
    exact_settled_continuation_for_authority(
        authority,
        root,
        &resolved.authority,
        &resolved.caller.descriptor,
        None,
    )
}

#[cfg(target_os = "linux")]
fn exact_settled_continuation_for_authority(
    authority: &HostSessionAuthority,
    root: &crate::execution::agent_runtime::host_session_authority::store_schema::StateRootV3,
    source: &crate::execution::agent_runtime::host_session_authority::store_schema::DurableSessionAuthorityV1,
    descriptor: &crate::execution::agent_runtime::host_session_authority::schema::AgentDescriptorV1,
    expected_handle: Option<
        &crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
    >,
) -> Result<(
    crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
    String,
)> {
    use crate::execution::agent_runtime::host_session_authority::schema::{
        AuthorityObjectKindV1, ResumeHandleHashInputV1,
    };
    let owner = source
        .active_authoritative_participant_id
        .as_deref()
        .context("continuation source has no authoritative participant")?;
    let mut settled = None;
    for reference in &source.internal_resume_handle_refs {
        if reference.object_kind != AuthorityObjectKindV1::ResumeHandle {
            anyhow::bail!("current continuation authority contains a non-handle reference");
        }
        if expected_handle.is_some_and(|expected| expected != reference) {
            continue;
        }
        let bytes = authority
            .read_authority_object_v2_at(root.root_revision, reference)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        let native_session = match reference.schema_version {
            1 => {
                // The caller supplies the authenticated current or exact-Applied
                // source authority. Typed history validates this produced handle's
                // completion/native lineage; match that source's active owner.
                let handle: ResumeHandleHashInputV1 =
                    crate::execution::agent_runtime::host_session_authority::canonical_json::from_slice(&bytes)
                        .context("decode produced current continuation handle")?;
                if handle.participant_id != owner {
                    continue;
                }
                if handle.schema_version != 1
                    || handle.orchestration_session_id != source.orchestration_session_id
                    || handle.backend_id != descriptor.backend_id
                    || handle.protocol != descriptor.protocol
                    || handle.internal_uaa_session_id.trim().is_empty()
                {
                    anyhow::bail!("produced continuation does not join exact source identity");
                }
                handle.internal_uaa_session_id
            }
            2 | 3 => {
                let handle = crate::execution::agent_runtime::host_session_authority::start_continuity::decode_start_continuation_handle(reference, &bytes)
                    .context("decode versioned durable Start continuation handle")?;
                if handle.world_controller().is_some_and(|world| {
                    source.world_binding.as_ref() != Some(&world.world_binding)
                }) {
                    anyhow::bail!("World continuation differs from current session placement");
                }
                if handle.authority_store_id != root.authority_store_id
                    || handle.orchestration_session_id != source.orchestration_session_id
                    || handle.backend_id != descriptor.backend_id
                    || handle.protocol != descriptor.protocol
                    || handle.internal_uaa_session_id.trim().is_empty()
                {
                    anyhow::bail!(config_model::user_error(
                        "owner_unreachable: durable Start continuation does not authenticate current authority",
                    ));
                }
                if handle.participant_id != owner
                    || !matches!(handle.state, StartContinuationHandleStateV2::Settled { .. })
                {
                    continue;
                }
                handle.internal_uaa_session_id.clone()
            }
            _ => continue,
        };
        if settled
            .replace((reference.clone(), native_session))
            .is_some()
        {
            anyhow::bail!(config_model::user_error(
                "owner_unreachable: current authority has ambiguous owner-bound continuations",
            ));
        }
    }
    settled.ok_or_else(|| {
        config_model::user_error(
            "owner_unreachable: current authority has no settled owner-bound continuation",
        )
    })
}

#[cfg(target_os = "linux")]
fn exact_applied_successor_source_continuation(
    authority: &HostSessionAuthority,
    root: &crate::execution::agent_runtime::host_session_authority::store_schema::StateRootV3,
    applied: &HostSessionTransitionIntentV3,
) -> Result<(
    crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
    String,
)> {
    let HostSessionAuthorityPreconditionV1::ExpectedRevision {
        authority_revision,
        authority_record_commitment,
        active_authoritative_participant_id,
        authoritative_lineage_commitment,
        lifecycle_posture,
    } = &applied.authority_precondition
    else {
        anyhow::bail!("applied successor omitted exact source precondition");
    };
    if !matches!(
        applied.state,
        HostSessionTransitionIntentStateV3::Applied { .. }
    ) || root.successor_transition_intent_map.get(&applied.intent_id) != Some(applied)
    {
        anyhow::bail!("applied successor source requires its exact stored operation");
    }
    let source = authority
        .resolve_exact_at_revision(&applied.orchestration_session_id, *authority_revision)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let observation = source.observation();
    if observation.authority_store_id != root.authority_store_id
        || observation.bootstrap_home != root.bootstrap_home
        || source.authority_record_commitment != *authority_record_commitment
        || source.authoritative_lineage_commitment != *authoritative_lineage_commitment
        || source
            .authority
            .active_authoritative_participant_id
            .as_ref()
            != Some(active_authoritative_participant_id)
        || source.authority.lifecycle_posture != *lifecycle_posture
        || applied.source_authoritative_participant_id.as_ref()
            != Some(active_authoritative_participant_id)
        || applied.caller.caller_participant_id.as_ref()
            != Some(active_authoritative_participant_id)
        || source.authority.workspace_binding != applied.workspace_binding
        || source.authority.world_binding != applied.world_binding
        || source.authority.shell_trace_session_id != applied.shell_trace_session_id
        || source.authority.host_attach_contract_ref.as_ref()
            != Some(&applied.host_attach_contract_ref)
    {
        anyhow::bail!("applied successor exact source authority differs");
    }
    let expected_handle = match (applied.mode, applied.resume_handle_ref.as_ref()) {
        (HostSessionTransitionModeV1::Attach, None) => None,
        (HostSessionTransitionModeV1::ResumeOneTurn, Some(reference)) => Some(reference),
        _ => anyhow::bail!("applied successor changed its continuation mode or reference"),
    };
    let bytes = authority
        .read_authority_object_v2_at(root.root_revision, &applied.descriptor_ref)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let descriptor: crate::execution::agent_runtime::host_session_authority::schema::AgentDescriptorHashInputV1 =
        crate::execution::agent_runtime::host_session_authority::canonical_json::from_slice(&bytes)
            .context("decode applied successor source descriptor")?;
    // This recovers only the existing operation's native continuity. It grants
    // no fresh issuance, completion or replay permission.
    exact_settled_continuation_for_authority(
        authority,
        root,
        &source.authority,
        &descriptor.descriptor,
        expected_handle,
    )
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PublicSuccessorRequestStateV1 {
    schema_version: u32,
    request_commitment: String,
    mode: HostSessionTransitionModeV1,
    orchestration_session_id: String,
    intent_id: String,
    issuer_request_id: String,
    target_participant_id: String,
    participant_lease_token: String,
    run_id: String,
    claim_id: String,
    claimant_attempt_id: String,
    #[serde(default)]
    acknowledged: bool,
}

#[cfg(target_os = "linux")]
struct PublicSuccessorRequestGuardV1 {
    _lock_file: File,
    state_root: PathBuf,
    state_path: PathBuf,
    state: PublicSuccessorRequestStateV1,
}

#[cfg(target_os = "linux")]
fn acquire_public_successor_request_guard(
    substrate_home: &Path,
    orchestration_session_id: &str,
    mode: HostSessionTransitionModeV1,
    request_commitment: &str,
) -> Result<PublicSuccessorRequestGuardV1> {
    match mode {
        HostSessionTransitionModeV1::Attach | HostSessionTransitionModeV1::ResumeOneTurn => {}
        HostSessionTransitionModeV1::Start => {
            anyhow::bail!("public successor request coordination cannot use Start")
        }
    }
    let key = public_start_digest(
        "substrate.public-successor-coordinate.v1",
        &[orchestration_session_id.as_bytes()],
    );
    let root = substrate_home
        .join("runtime-control")
        .join("durable-start")
        .join("public-successor-requests");
    fs::create_dir_all(&root).with_context(|| format!("failed to create {}", root.display()))?;
    let lock_path = root.join(format!("{key}.lock"));
    let state_path = root.join(format!("{key}.json"));
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&lock_path)
        .with_context(|| {
            format!(
                "failed to open public successor request guard {}",
                lock_path.display()
            )
        })?;
    match file.try_lock_exclusive() {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
            file.lock_exclusive().with_context(|| {
                format!(
                    "failed to join the in-flight public successor request for {orchestration_session_id}"
                )
            })?;
        }
        Err(error) => {
            return Err(anyhow::Error::new(error).context(format!(
                "failed to coordinate public successor request for {orchestration_session_id}"
            )));
        }
    }

    let existing = if state_path.exists() {
        let bytes = fs::read(&state_path)
            .with_context(|| format!("failed to read {}", state_path.display()))?;
        Some(
            serde_json::from_slice::<PublicSuccessorRequestStateV1>(&bytes)
                .with_context(|| format!("failed to decode {}", state_path.display()))?,
        )
    } else {
        None
    };
    let existing_identity_is_well_formed = existing.as_ref().is_some_and(|state| {
        state.schema_version == 1
            && !state.request_commitment.is_empty()
            && !state.orchestration_session_id.is_empty()
            && !state.intent_id.is_empty()
            && !state.issuer_request_id.is_empty()
            && !state.target_participant_id.is_empty()
            && !state.participant_lease_token.is_empty()
            && !state.run_id.is_empty()
            && !state.claim_id.is_empty()
            && !state.claimant_attempt_id.is_empty()
    });
    if existing.is_some() && !existing_identity_is_well_formed {
        anyhow::bail!(
            "durable public successor request identity is malformed for {orchestration_session_id}"
        );
    }
    let existing_is_exact = existing_identity_is_well_formed
        && existing.as_ref().is_some_and(|state| {
            state.request_commitment == request_commitment
                && state.mode == mode
                && state.orchestration_session_id == orchestration_session_id
        });
    if existing.as_ref().is_some_and(|state| !state.acknowledged) && !existing_is_exact {
        anyhow::bail!(
            "durable public successor request identity conflicts with the retry for {orchestration_session_id}"
        );
    }
    let state = if existing_is_exact {
        existing.expect("checked exact durable public successor request")
    } else {
        PublicSuccessorRequestStateV1 {
            schema_version: 1,
            request_commitment: request_commitment.to_string(),
            mode,
            orchestration_session_id: orchestration_session_id.to_string(),
            intent_id: format!("intent_{}", Uuid::now_v7()),
            issuer_request_id: format!("request_{}", Uuid::now_v7()),
            target_participant_id: format!("ash_{}", Uuid::now_v7()),
            participant_lease_token: Uuid::now_v7().to_string(),
            run_id: Uuid::now_v7().to_string(),
            claim_id: format!("claim_{}", Uuid::now_v7()),
            claimant_attempt_id: format!("attempt_{}", Uuid::now_v7()),
            acknowledged: false,
        }
    };
    persist_public_successor_request_state(&root, &state_path, &state)?;
    Ok(PublicSuccessorRequestGuardV1 {
        _lock_file: file,
        state_root: root,
        state_path,
        state,
    })
}

#[cfg(target_os = "linux")]
fn acknowledge_public_successor_request(
    coordination: &PublicSuccessorRequestGuardV1,
) -> Result<()> {
    let bytes = fs::read(&coordination.state_path)
        .with_context(|| format!("failed to read {}", coordination.state_path.display()))?;
    let mut persisted = serde_json::from_slice::<PublicSuccessorRequestStateV1>(&bytes)
        .with_context(|| format!("failed to decode {}", coordination.state_path.display()))?;
    if persisted != coordination.state {
        anyhow::bail!("durable public successor request changed before acknowledgement");
    }
    persisted.acknowledged = true;
    persist_public_successor_request_state(
        &coordination.state_root,
        &coordination.state_path,
        &persisted,
    )
}

#[cfg(target_os = "linux")]
fn mark_public_successor_request_in_flight(
    coordination: &mut PublicSuccessorRequestGuardV1,
) -> Result<()> {
    if !coordination.state.acknowledged {
        return Ok(());
    }
    let bytes = fs::read(&coordination.state_path)
        .with_context(|| format!("failed to read {}", coordination.state_path.display()))?;
    let persisted = serde_json::from_slice::<PublicSuccessorRequestStateV1>(&bytes)
        .with_context(|| format!("failed to decode {}", coordination.state_path.display()))?;
    if persisted != coordination.state {
        anyhow::bail!("durable public successor request changed before launch");
    }
    coordination.state.acknowledged = false;
    persist_public_successor_request_state(
        &coordination.state_root,
        &coordination.state_path,
        &coordination.state,
    )
}

#[cfg(target_os = "linux")]
fn persist_public_successor_request_state(
    root: &Path,
    state_path: &Path,
    state: &PublicSuccessorRequestStateV1,
) -> Result<()> {
    let temp_path = root.join(format!(".request-state-{}.tmp", Uuid::now_v7()));
    let bytes = serde_json::to_vec_pretty(state)?;
    let mut temp = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(&temp_path)
        .with_context(|| format!("failed to create {}", temp_path.display()))?;
    use std::io::Write as _;
    temp.write_all(&bytes)
        .with_context(|| format!("failed to write {}", temp_path.display()))?;
    temp.sync_all()
        .with_context(|| format!("failed to sync {}", temp_path.display()))?;
    fs::rename(&temp_path, state_path).with_context(|| {
        format!(
            "failed to publish public successor request state {}",
            state_path.display()
        )
    })?;
    File::open(root)
        .with_context(|| format!("failed to open {}", root.display()))?
        .sync_all()
        .with_context(|| format!("failed to sync {}", root.display()))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn current_applied_public_successor(
    authority: &HostSessionAuthority,
    root: &crate::execution::agent_runtime::host_session_authority::store_schema::StateRootV3,
    resolved: &crate::execution::agent_runtime::host_session_authority::ResolvedCurrentAuthorityV1,
    mode: HostSessionTransitionModeV1,
    transition_input: Option<&[u8]>,
) -> Result<Option<HostSessionTransitionIntentV3>> {
    let Some(active_participant_id) = resolved
        .authority
        .active_authoritative_participant_id
        .as_deref()
    else {
        return Ok(None);
    };
    let mut exact = None;
    for intent in root.successor_transition_intent_map.values() {
        let HostSessionTransitionIntentStateV3::Applied {
            authority_revision_after,
            authority_record_commitment,
            active_authoritative_participant_id: applied_participant_id,
            startup_ownership,
            post_turn,
            ..
        } = &intent.state
        else {
            continue;
        };
        if intent.mode != mode
            || intent.orchestration_session_id != resolved.authority.orchestration_session_id
            || intent.caller.kind != HostSessionTransitionCallerKindV1::PublicCli
            || intent.target_authoritative_participant_id != active_participant_id
            || applied_participant_id != active_participant_id
            || *authority_revision_after != resolved.observation.authority_revision
            || authority_record_commitment != &resolved.observation.authority_record_commitment
        {
            continue;
        }
        let joinable = match mode {
            HostSessionTransitionModeV1::Attach => matches!(
                startup_ownership.as_ref(),
                crate::execution::agent_runtime::host_session_authority::store_schema::HostSessionStartupOwnershipApplicationV1::Pending { .. }
            ),
            HostSessionTransitionModeV1::ResumeOneTurn => {
                matches!(
                    intent.input_handoff,
                    HostSessionTransitionInputHandoffV1::Pending { .. }
                ) && matches!(
                    post_turn.as_ref(),
                    crate::execution::agent_runtime::host_session_authority::store_schema::HostSessionPostTurnApplicationV2::Pending { .. }
                )
            }
            HostSessionTransitionModeV1::Start => false,
        };
        if !joinable {
            continue;
        }
        match (transition_input, intent.transition_input_ref.as_ref()) {
            (Some(expected), Some(reference)) => {
                let actual = authority
                    .read_authority_object_v2_at(root.root_revision, reference)
                    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
                if actual != expected {
                    continue;
                }
            }
            (None, None) => {}
            _ => continue,
        }
        if exact.replace(intent.clone()).is_some() {
            anyhow::bail!(
                "current HSA authority has ambiguous applied public successor transitions"
            );
        }
    }
    Ok(exact)
}

#[cfg(target_os = "linux")]
fn coordinated_applied_public_successor(
    authority: &HostSessionAuthority,
    root: &crate::execution::agent_runtime::host_session_authority::store_schema::StateRootV3,
    resolved: &crate::execution::agent_runtime::host_session_authority::ResolvedCurrentAuthorityV1,
    mode: HostSessionTransitionModeV1,
    transition_input: Option<&[u8]>,
    coordination: &PublicSuccessorRequestGuardV1,
) -> Result<Option<HostSessionTransitionIntentV3>> {
    let Some(intent) = root
        .successor_transition_intent_map
        .get(&coordination.state.intent_id)
    else {
        if root.successor_transition_intent_map.values().any(|intent| {
            intent.orchestration_session_id == coordination.state.orchestration_session_id
                && intent.mode == mode
                && intent.caller.kind == HostSessionTransitionCallerKindV1::PublicCli
                && matches!(
                    intent.state,
                    HostSessionTransitionIntentStateV3::Issued
                        | HostSessionTransitionIntentStateV3::Claimed { .. }
                )
        }) {
            anyhow::bail!(
                "durable public successor request identity conflicts with a nonterminal HSA transition"
            );
        }
        return Ok(None);
    };
    if intent.mode != mode
        || intent.orchestration_session_id != coordination.state.orchestration_session_id
        || intent.issuer_request_id != coordination.state.issuer_request_id
        || intent.target_authoritative_participant_id != coordination.state.target_participant_id
        || intent.run_id != coordination.state.run_id
        || intent.caller.kind != HostSessionTransitionCallerKindV1::PublicCli
    {
        anyhow::bail!("coordinated public successor intent identity was substituted");
    }
    let exact_request = IssueSuccessorTransitionRequestV1 {
        intent_id: intent.intent_id.clone(),
        issuer_request_id: intent.issuer_request_id.clone(),
        mode: intent.mode,
        authority_precondition: intent.authority_precondition.clone(),
        orchestration_session_id: intent.orchestration_session_id.clone(),
        shell_trace_session_id: intent.shell_trace_session_id.clone(),
        caller: intent.caller.clone(),
        source_authoritative_participant_id: intent.source_authoritative_participant_id.clone(),
        target_authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
        target_participant_lease_token: coordination
            .state
            .participant_lease_token
            .as_bytes()
            .to_vec(),
        run_id: intent.run_id.clone(),
        resulting_authoritative_lineage: intent.resulting_authoritative_lineage.clone(),
        workspace_binding: intent.workspace_binding.clone(),
        world_binding: intent.world_binding.clone(),
        resume_handle_ref: intent.resume_handle_ref.clone(),
        transition_input: transition_input.map(<[u8]>::to_vec),
        post_turn_disposition: intent.post_turn_disposition,
    };
    let joined = match authority
        .issue_successor(resolved, &exact_request)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?
    {
        SuccessorTransitionIssueOutcomeV1::Joined(intent) => intent,
        SuccessorTransitionIssueOutcomeV1::Issued(_) => {
            anyhow::bail!("coordinated public successor exact join issued a replacement intent")
        }
    };
    if joined != *intent {
        anyhow::bail!("coordinated public successor exact join changed durable identity");
    }
    match &joined.state {
        HostSessionTransitionIntentStateV3::Applied { .. } => Ok(Some(joined)),
        HostSessionTransitionIntentStateV3::Issued
        | HostSessionTransitionIntentStateV3::Claimed { .. } => Ok(None),
        HostSessionTransitionIntentStateV3::Rejected { .. }
        | HostSessionTransitionIntentStateV3::Expired { .. } => {
            anyhow::bail!("coordinated public successor is durably terminal without application")
        }
    }
}

#[cfg(target_os = "linux")]
fn build_authority_managed_successor_launch_plan(
    orchestration_session_id: &str,
    launch_intent: AttachLaunchIntent,
    transition_input: Option<&str>,
    coordination: Option<&PublicSuccessorRequestGuardV1>,
) -> Result<AuthorityManagedSuccessorLaunchPlanV1> {
    build_authority_managed_successor_launch_plan_with_ordinary_source(
        orchestration_session_id,
        launch_intent,
        transition_input,
        coordination,
        None,
    )
}

#[cfg(target_os = "linux")]
fn build_authority_managed_successor_launch_plan_with_ordinary_source(
    orchestration_session_id: &str,
    launch_intent: AttachLaunchIntent,
    transition_input: Option<&str>,
    coordination: Option<&PublicSuccessorRequestGuardV1>,
    ordinary: Option<&Arc<crate::execution::agent_runtime::native_launch_context::AuthenticatedOrdinarySelectionV1>>,
) -> Result<AuthorityManagedSuccessorLaunchPlanV1> {
    let revalidate_source = || -> Result<()> {
        if let Some(selection) = ordinary {
            if selection.original_session_id() != orchestration_session_id {
                anyhow::bail!("ordinary public successor source session changed");
            }
            crate::execution::agent_runtime::control::revalidate_ordinary_public_source_permission(
                selection,
            )?;
        }
        Ok(())
    };
    revalidate_source()?;
    let authority = public_start_authority()?;
    let mut resolved = authority
        .resolve_current_exact(orchestration_session_id, None)
        .map_err(|error| config_model::user_error(format!("owner_unreachable: {error}")))?;
    let mode = match launch_intent {
        AttachLaunchIntent::Reattach => HostSessionTransitionModeV1::Attach,
        AttachLaunchIntent::DetachedTurn => HostSessionTransitionModeV1::ResumeOneTurn,
    };
    if matches!(mode, HostSessionTransitionModeV1::ResumeOneTurn) != transition_input.is_some() {
        anyhow::bail!(config_model::user_error(
            "owner_unreachable: authority-managed ResumeOneTurn requires the exact public turn input"
        ));
    }
    let root = authority
        .read_a12b_root()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let existing_applied = match coordination {
        Some(coordination) => coordinated_applied_public_successor(
            &authority,
            &root,
            &resolved,
            mode,
            transition_input.map(str::as_bytes),
            coordination,
        )?,
        None => current_applied_public_successor(
            &authority,
            &root,
            &resolved,
            mode,
            transition_input.map(str::as_bytes),
        )?,
    };
    let (resume_handle_ref, internal_uaa_session_id) = match existing_applied.as_ref() {
        Some(applied) => exact_applied_successor_source_continuation(&authority, &root, applied)?,
        None => exact_settled_start_continuation(&authority, &root, &resolved)?,
    };
    let applied = match existing_applied {
        Some(applied) => applied,
        None => {
            let source_participant_id = resolved
                .authority
                .active_authoritative_participant_id
                .clone()
                .ok_or_else(|| {
                    config_model::user_error(
                        "owner_unreachable: current HSA authority has no active participant",
                    )
                })?;
            let target_participant_id = coordination
                .map(|coordination| coordination.state.target_participant_id.clone())
                .unwrap_or_else(|| format!("ash_{}", Uuid::now_v7()));
            let lease_token = coordination
                .map(|coordination| coordination.state.participant_lease_token.clone())
                .unwrap_or_else(|| Uuid::now_v7().to_string());
            let run_id = coordination
                .map(|coordination| coordination.state.run_id.clone())
                .unwrap_or_else(|| Uuid::now_v7().to_string());
            let mut lineage = resolved.authority.authoritative_participant_lineage.clone();
            lineage.push(target_participant_id.clone());
            let request = IssueSuccessorTransitionRequestV1 {
                intent_id: coordination
                    .map(|coordination| coordination.state.intent_id.clone())
                    .unwrap_or_else(|| format!("intent_{}", Uuid::now_v7())),
                issuer_request_id: coordination
                    .map(|coordination| coordination.state.issuer_request_id.clone())
                    .unwrap_or_else(|| format!("request_{}", Uuid::now_v7())),
                mode,
                authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedRevision {
                    authority_revision: resolved.observation.authority_revision,
                    authority_record_commitment: resolved
                        .observation
                        .authority_record_commitment
                        .clone(),
                    active_authoritative_participant_id: source_participant_id.clone(),
                    authoritative_lineage_commitment: resolved
                        .observation
                        .authoritative_lineage_commitment
                        .clone(),
                    lifecycle_posture: resolved.authority.lifecycle_posture,
                },
                orchestration_session_id: orchestration_session_id.to_string(),
                shell_trace_session_id: resolved.authority.shell_trace_session_id.clone(),
                caller: HostSessionTransitionCallerV1 {
                    kind: HostSessionTransitionCallerKindV1::PublicCli,
                    caller_participant_id: Some(source_participant_id.clone()),
                    auto_attach_obligation_id: None,
                    auto_attach_claim_owner: None,
                },
                source_authoritative_participant_id: Some(source_participant_id),
                target_authoritative_participant_id: target_participant_id,
                target_participant_lease_token: lease_token.into_bytes(),
                run_id,
                resulting_authoritative_lineage: lineage,
                workspace_binding: resolved.authority.workspace_binding.clone(),
                world_binding: resolved.authority.world_binding.clone(),
                resume_handle_ref: matches!(mode, HostSessionTransitionModeV1::ResumeOneTurn)
                    .then_some(resume_handle_ref.clone()),
                transition_input: transition_input.map(|input| input.as_bytes().to_vec()),
                post_turn_disposition: matches!(mode, HostSessionTransitionModeV1::ResumeOneTurn)
                    .then_some(HostPostTurnDispositionV1::ReconcileToAttentionParkOrTerminal),
            };
            revalidate_source()?;
            let issued = match authority
                .issue_successor(&resolved, &request)
                .map_err(|error| anyhow::anyhow!(error.to_string()))?
            {
                SuccessorTransitionIssueOutcomeV1::Issued(intent)
                | SuccessorTransitionIssueOutcomeV1::Joined(intent) => intent,
            };
            maybe_inject_public_successor_launcher_crash("after_issue")?;
            let claim_id = coordination
                .map(|coordination| coordination.state.claim_id.clone())
                .unwrap_or_else(|| format!("claim_{}", Uuid::now_v7()));
            let claimant_attempt_id = coordination
                .map(|coordination| coordination.state.claimant_attempt_id.clone())
                .unwrap_or_else(|| format!("attempt_{}", Uuid::now_v7()));
            revalidate_source()?;
            let claimed = match authority
                .claim_successor(&ClaimHostSessionTransitionRequestV1 {
                    intent_id: issued.intent_id.clone(),
                    issuer_request_id: issued.issuer_request_id.clone(),
                    payload_commitment: issued.payload_commitment.clone(),
                    expected_intent_revision: issued.intent_revision,
                    claim_id: claim_id.clone(),
                    claimant_attempt_id,
                })
                .map_err(|error| anyhow::anyhow!(error.to_string()))?
            {
                SuccessorTransitionClaimOutcomeV1::Claimed(intent)
                | SuccessorTransitionClaimOutcomeV1::Reclaimed(intent)
                | SuccessorTransitionClaimOutcomeV1::Joined(intent) => intent,
            };
            maybe_inject_public_successor_launcher_crash("after_claim")?;
            let HostSessionTransitionIntentStateV3::Claimed { claim_revision, .. } = &claimed.state
            else {
                anyhow::bail!(
                    "authority-managed successor claim did not retain exact claim evidence"
                );
            };
            revalidate_source()?;
            let applied = match authority
                .apply_successor(&ApplyHostSessionTransitionRequestV1 {
                    intent_id: claimed.intent_id.clone(),
                    issuer_request_id: claimed.issuer_request_id.clone(),
                    payload_commitment: claimed.payload_commitment.clone(),
                    expected_intent_revision: claimed.intent_revision,
                    claim_id,
                    expected_claim_revision: *claim_revision,
                })
                .map_err(|error| anyhow::anyhow!(error.to_string()))?
            {
                SuccessorTransitionApplicationOutcomeV1::Applied(intent)
                | SuccessorTransitionApplicationOutcomeV1::Joined(intent) => intent,
            };
            maybe_inject_public_successor_launcher_crash("after_apply")?;
            applied
        }
    };
    resolved = authority
        .resolve_current_exact(orchestration_session_id, None)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let root = authority
        .read_a12b_root()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let lease_token = match coordination {
        Some(coordination) => coordination
            .state
            .participant_lease_token
            .as_bytes()
            .to_vec(),
        None => authority
            .read_authority_object_v2_at(
                root.root_revision,
                &applied.target_participant_lease_token_ref,
            )
            .map_err(|error| anyhow::anyhow!(error.to_string()))?,
    };
    let descriptor = &resolved.caller.descriptor;
    let descriptor = ResolvedRuntimeDescriptor {
        agent_id: descriptor.agent_id.clone(),
        backend_id: descriptor.backend_id.clone(),
        backend_kind: match descriptor.backend_kind {
            RuntimeBackendKindV1::Codex => ResolvedRuntimeBackendKind::Codex,
            RuntimeBackendKindV1::ClaudeCode => ResolvedRuntimeBackendKind::ClaudeCode,
        },
        protocol: descriptor.protocol.clone(),
        execution_scope: match descriptor.execution_scope {
            AgentExecutionScopeV1::Host => AgentExecutionScope::Host,
            AgentExecutionScopeV1::World => AgentExecutionScope::World,
        },
        binary_path: descriptor.binary_path.clone(),
    };
    let world_binding = applied.world_binding.as_ref();
    let helper_plan = HiddenOwnerHelperLaunchPlan {
        mode: launch_intent.owner_helper_mode(),
        descriptor,
        session: HiddenOwnerHelperSessionPlan {
            orchestration_session_id: applied.orchestration_session_id.clone(),
            shell_trace_session_id: applied.shell_trace_session_id.clone(),
            workspace_root: applied
                .workspace_binding
                .workspace_root
                .physical_path
                .clone(),
            world_id: world_binding.map(|binding| binding.world_id.clone()),
            world_generation: world_binding.map(|binding| binding.world_generation),
        },
        participant: HiddenOwnerHelperParticipantPlan {
            participant_id: applied.target_authoritative_participant_id.clone(),
            lease_token: String::from_utf8(lease_token)
                .context("authority-managed participant lease token is not UTF-8")?,
            run_id: applied.run_id.clone(),
            resumed_from_participant_id: applied.source_authoritative_participant_id.clone(),
            internal_uaa_session_id: Some(internal_uaa_session_id),
        },
        host_attach_contract: None,
        startup_prompt: None,
        source_orchestration_session_id: None,
    };
    Ok(AuthorityManagedSuccessorLaunchPlanV1 {
        schema_version: 1,
        authority_store_id: root.authority_store_id.clone(),
        helper_plan,
        applied_transition: applied,
    })
}

#[cfg(target_os = "linux")]
fn build_ordinary_public_resumed_turn_launch_plan(
    selection: &Arc<
        crate::execution::agent_runtime::native_launch_context::AuthenticatedOrdinarySelectionV1,
    >,
    orchestration_session_id: &str,
    input: &str,
    coordination: &PublicSuccessorRequestGuardV1,
) -> Result<AuthorityManagedSuccessorLaunchPlanV1> {
    for attempt in 0..8 {
        match build_authority_managed_successor_launch_plan_with_ordinary_source(
            orchestration_session_id,
            AttachLaunchIntent::DetachedTurn,
            Some(input),
            Some(coordination),
            Some(selection),
        ) {
            Ok(plan) => return Ok(plan),
            Err(error)
                if attempt + 1 < 8
                    && error
                        .chain()
                        .any(|cause| cause.to_string() == "stale authority root revision") =>
            {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(error),
        }
    }
    anyhow::bail!("ordinary public successor retry exhausted")
}

#[cfg(all(target_os = "linux", test))]
thread_local! {
    static PUBLIC_SUCCESSOR_LAUNCHER_FAULT: std::cell::Cell<Option<&'static str>> =
        const { std::cell::Cell::new(None) };
}

#[cfg(all(target_os = "linux", test))]
fn inject_public_successor_launcher_fault_for_test(stage: &'static str) {
    PUBLIC_SUCCESSOR_LAUNCHER_FAULT.set(Some(stage));
}

#[cfg(all(target_os = "linux", test))]
fn maybe_inject_public_successor_launcher_crash(stage: &str) -> Result<()> {
    let injected = PUBLIC_SUCCESSOR_LAUNCHER_FAULT.take();
    if injected == Some(stage) {
        anyhow::bail!("injected launcher crash {stage}");
    }
    PUBLIC_SUCCESSOR_LAUNCHER_FAULT.set(injected);
    Ok(())
}

#[cfg(all(target_os = "linux", not(test)))]
fn maybe_inject_public_successor_launcher_crash(_stage: &str) -> Result<()> {
    Ok(())
}

#[cfg(all(unix, not(target_os = "linux")))]
fn build_resumed_turn_launch_plan(
    orchestration_session_id: &str,
) -> Result<HiddenOwnerHelperLaunchPlan> {
    build_attach_launch_plan(orchestration_session_id, AttachLaunchIntent::DetachedTurn)
}

#[derive(Clone, Debug)]
struct ForkSuccessorAllocation {
    orchestration_session_id: String,
    backend_id: String,
    source_orchestration_session_id: String,
}

// Linux no longer executes the legacy fork authority path. Keep the frozen cross-platform
// surface type-checked until its definitions can be removed by a separately admitted change.
#[cfg(target_os = "linux")]
const _: PublicControlAction = PublicControlAction::Fork;
#[cfg(target_os = "linux")]
const _: fn(&OrchestrationSessionRecord) -> Option<HostAttachContract> =
    OrchestrationSessionRecord::fork_successor_attach_contract;
#[cfg(target_os = "linux")]
const _: fn(
    &crate::execution::agent_runtime::state_store::ResolvedPublicControlTarget,
) -> Option<&HostAttachContract> = |target| target.host_attach_contract.as_ref();

fn allocate_fork_successor(orchestration_session_id: &str) -> Result<ForkSuccessorAllocation> {
    #[cfg(not(target_os = "linux"))]
    {
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
        let descriptor = RuntimeSelectionDescriptor::try_from(&attach_contract.launch_descriptor)
            .map_err(|err| {
                runtime_materialization_user_error(
                    "owner_unreachable",
                    format!(
                        "persisted host attach contract stored an invalid successor launch descriptor: {err:#}"
                    ),
                )
            })?;
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
                resumed_from_participant_id: target
                    .active_participant
                    .handle
                    .participant_id
                    .clone(),
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
            target.session.fork_successor_attach_contract(),
        );
        successor_session.shell_owner_pid = 0;
        successor_session.latest_run_id = Some(run_id);
        successor_session.world_id = target.session.world_id.clone();
        successor_session.world_generation = target.session.world_generation;
        successor_session
            .transition_state(crate::execution::agent_runtime::OrchestrationSessionState::Active);
        successor_session.active_session_handle_id = Some(successor_participant_id);
        successor_session.pending_inbox_count = 0;
        successor_session.last_attention_at = None;
        successor_session
            .mark_parked_resumable("fork successor allocated without attached host client");

        store.persist_orchestration_session(&successor_session)?;
        store.persist_participant(&successor_participant)?;

        return Ok(ForkSuccessorAllocation {
            orchestration_session_id: successor_session_id,
            backend_id: attach_contract.backend_id,
            source_orchestration_session_id: target.session.orchestration_session_id,
        });
    }

    #[cfg(target_os = "linux")]
    {
        let authority = public_start_authority()?;
        let source = authority
            .resolve_current_exact(orchestration_session_id, None)
            .map_err(|error| config_model::user_error(error.to_string()))?;
        if !source.host_attach_contract.capabilities.session_fork {
            anyhow::bail!(config_model::user_error(
                "durable host attach contract does not allow fork"
            ));
        }
        let source_participant_id = source
            .authority
            .active_authoritative_participant_id
            .clone()
            .ok_or_else(|| {
                config_model::user_error(
                    "authority_invalid: fork source has no active authoritative participant",
                )
            })?;
        if source_participant_id != source.caller.participant_id {
            anyhow::bail!(config_model::user_error(
                "authority_invalid: fork source participant does not match exact caller"
            ));
        }

        let allocation_nonce = Uuid::now_v7();
        let target_orchestration_session_id = Uuid::now_v7().to_string();
        let target_shell_trace_session_id = Uuid::now_v7().to_string();
        let target_authoritative_participant_id = format!("ash_{}", Uuid::now_v7());
        let resulting_authoritative_lineage = source
            .authority
            .authoritative_participant_lineage
            .iter()
            .cloned()
            .chain(std::iter::once(target_authoritative_participant_id.clone()))
            .collect();
        let request = AllocateForkSuccessorRequestV1 {
            schema_version: 1,
            allocation_id: format!("fork-allocation-{allocation_nonce}"),
            request_id: format!("fork-request-{allocation_nonce}"),
            authority_store_id: source.observation.authority_store_id.clone(),
            bootstrap_home: source.observation.bootstrap_home.clone(),
            expected_source_root_revision: source.observation.root_revision,
            source_orchestration_session_id: source.authority.orchestration_session_id.clone(),
            source_shell_trace_session_id: source.authority.shell_trace_session_id.clone(),
            source_authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedRevision {
                authority_revision: source.observation.authority_revision,
                authority_record_commitment: source.observation.authority_record_commitment.clone(),
                active_authoritative_participant_id: source_participant_id,
                authoritative_lineage_commitment: source
                    .observation
                    .authoritative_lineage_commitment
                    .clone(),
                lifecycle_posture: source.authority.lifecycle_posture,
            },
            source_authoritative_participant_lineage: source
                .authority
                .authoritative_participant_lineage
                .clone(),
            target_orchestration_session_id: target_orchestration_session_id.clone(),
            target_shell_trace_session_id,
            target_authoritative_participant_id,
            resulting_authoritative_lineage,
            workspace_binding: source.authority.workspace_binding.clone(),
            world_binding: source.authority.world_binding.clone(),
            allocated_at: current_timestamp_v1()?,
        };
        let allocation = match authority
            .allocate_fork_successor(&request)
            .map_err(|error| config_model::user_error(error.to_string()))?
        {
            ForkSuccessorAllocationOutcomeV1::Allocated(allocation)
            | ForkSuccessorAllocationOutcomeV1::Joined(allocation) => allocation,
        };
        if allocation.target_authority.authority_revision != 1
            || allocation.target_authority.lifecycle_posture
                != HostSessionPostureV1::ParkedResumable
        {
            anyhow::bail!(
                "HSA fork successor allocation returned a non-canonical target authority"
            );
        }

        Ok(ForkSuccessorAllocation {
            orchestration_session_id: allocation.target_authority.orchestration_session_id,
            backend_id: source.caller.descriptor.backend_id,
            source_orchestration_session_id: source.authority.orchestration_session_id,
        })
    }
}

#[cfg(target_os = "linux")]
fn refresh_public_session_world_binding(
    store: &AgentRuntimeStateStore,
    session: &mut OrchestrationSessionRecord,
) -> Result<()> {
    let Some(current_binding) = session.authoritative_world_binding() else {
        return Ok(());
    };
    let Some(authoritative_binding) = store
        .recover_active_shared_world_binding_from_local_metadata(
            &session.orchestration_session_id,
        )?
    else {
        return Ok(());
    };
    if authoritative_binding == current_binding {
        return Ok(());
    }

    store.set_orchestration_session_world_binding(
        session,
        authoritative_binding.world_id,
        authoritative_binding.world_generation,
    )?;
    store.invalidate_stale_world_members_for_session(
        &session.orchestration_session_id,
        authoritative_binding.world_generation,
    )?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn refresh_public_control_target_world_binding(
    store: &AgentRuntimeStateStore,
    target: &mut crate::execution::agent_runtime::state_store::ResolvedPublicControlTarget,
) -> Result<()> {
    refresh_public_session_world_binding(store, &mut target.session)
}

// The legacy Linux caller was removed by HSA episode demotion; retain only a type-level link to
// the frozen compatibility helper so this migration does not add a lint delta.
#[cfg(target_os = "linux")]
const _: fn(
    &AgentRuntimeStateStore,
    &mut crate::execution::agent_runtime::state_store::ResolvedPublicControlTarget,
) -> Result<()> = refresh_public_control_target_world_binding;

#[cfg(target_os = "linux")]
fn refresh_public_attach_target_world_binding(
    store: &AgentRuntimeStateStore,
    target: &mut crate::execution::agent_runtime::state_store::ResolvedPublicAttachTarget,
) -> Result<()> {
    refresh_public_session_world_binding(store, &mut target.session)
}

#[cfg(target_os = "linux")]
const _: fn(
    &AgentRuntimeStateStore,
    &mut crate::execution::agent_runtime::state_store::ResolvedPublicAttachTarget,
) -> Result<()> = refresh_public_attach_target_world_binding;

#[cfg(unix)]
fn wait_for_resumed_public_turn_detach(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    resumed_participant_id: &str,
    backend_id: &str,
) -> Result<()> {
    let started_at = std::time::Instant::now();
    loop {
        if resumed_public_turn_detach_ready(
            store,
            orchestration_session_id,
            resumed_participant_id,
            backend_id,
        )? && resumed_public_turn_detach_stable_after_grace(
            store,
            orchestration_session_id,
            resumed_participant_id,
            backend_id,
        )? {
            return Ok(());
        }
        if reconcile_resumed_public_turn_detach_timeout(
            store,
            orchestration_session_id,
            resumed_participant_id,
        )? && resumed_public_turn_detach_stable_after_grace(
            store,
            orchestration_session_id,
            resumed_participant_id,
            backend_id,
        )? {
            return Ok(());
        }
        if resumed_public_turn_detach_ready(
            store,
            orchestration_session_id,
            resumed_participant_id,
            backend_id,
        )? && resumed_public_turn_detach_stable_after_grace(
            store,
            orchestration_session_id,
            resumed_participant_id,
            backend_id,
        )? {
            return Ok(());
        }

        let obstruction = match store.load_session(orchestration_session_id)? {
            None => {
                Some(format!(
                    "orchestration session {} disappeared before the resumed public turn could detach cleanly",
                    orchestration_session_id
                ))
            }
            Some(record)
                if record.session.state.is_terminal()
                    || record.session.posture == OrchestrationSessionPosture::Terminal =>
            {
                let participant_summary = record
                    .participants
                    .iter()
                    .find(|candidate| candidate.participant_id() == resumed_participant_id)
                    .map(|participant| {
                        format!(
                            "participant_state={:?}, participant_reason={}",
                            participant.handle.state,
                            participant
                                .internal
                                .termination_reason
                                .as_deref()
                                .unwrap_or("<none>")
                        )
                    })
                    .unwrap_or_else(|| "participant_state=<missing>".to_string());
                Some(format!(
                    "orchestration session {} became terminal before resumed participant {} could repark (session_state={:?}, session_reason={}, {})",
                    orchestration_session_id,
                    resumed_participant_id,
                    record.session.state,
                    record
                        .session
                        .invalidation_reason
                        .as_deref()
                        .unwrap_or("<none>"),
                    participant_summary
                ))
            }
            Some(record)
                if record
                    .session
                    .active_participant_id()
                    .is_some_and(|participant_id| participant_id != resumed_participant_id) =>
            {
                Some(format!(
                    "orchestration session {} reassigned active ownership to {} before resumed participant {} could repark",
                    orchestration_session_id,
                    record.session.active_participant_id().unwrap_or_default(),
                    resumed_participant_id
                ))
            }
            Some(_) => None,
        };

        if started_at.elapsed() >= TURN_DETACH_NORMALIZATION_TIMEOUT {
            if let Some(obstruction) = obstruction {
                anyhow::bail!(
                    "owner_unreachable: timed out waiting for orchestration session {} resumed participant {} to converge back to detached durable continuity; last observed obstruction: {}",
                    orchestration_session_id,
                    resumed_participant_id,
                    obstruction
                );
            }
            anyhow::bail!(
                "owner_unreachable: timed out waiting for orchestration session {} resumed participant {} to converge back to detached durable continuity",
                orchestration_session_id,
                resumed_participant_id
            );
        }

        thread::sleep(TURN_DETACH_NORMALIZATION_POLL_INTERVAL);
    }
}

#[cfg(target_os = "linux")]
const _: fn(&AgentRuntimeStateStore, &str, &str, &str) -> Result<()> =
    wait_for_resumed_public_turn_detach;

#[cfg(unix)]
fn resumed_public_turn_detach_ready(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    resumed_participant_id: &str,
    backend_id: &str,
) -> Result<bool> {
    Ok(store
        .resumed_public_turn_detach_posture(orchestration_session_id, resumed_participant_id)?
        .is_some()
        || exact_detached_host_turn_target_ready(store, orchestration_session_id, backend_id)?)
}

#[cfg(unix)]
fn resumed_public_turn_detach_stable_after_grace(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    resumed_participant_id: &str,
    backend_id: &str,
) -> Result<bool> {
    thread::sleep(START_ATTACHED_GRACE_TIMEOUT);
    Ok(resumed_public_turn_detach_ready(
        store,
        orchestration_session_id,
        resumed_participant_id,
        backend_id,
    )? && resumed_public_turn_runtime_released(store, resumed_participant_id)?)
}

#[cfg(unix)]
fn exact_detached_host_turn_target_ready(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    backend_id: &str,
) -> Result<bool> {
    match store.resolve_public_turn_target(orchestration_session_id, backend_id) {
        Ok(target) => Ok(target.target_kind == PublicTurnTargetKind::Host
            && target.session_posture == PublicSessionPosture::DetachedReattachable),
        Err(_) => Ok(false),
    }
}

#[cfg(unix)]
fn resumed_public_turn_runtime_released(
    store: &AgentRuntimeStateStore,
    resumed_participant_id: &str,
) -> Result<bool> {
    let Some(participant) = store.load_participant(resumed_participant_id)? else {
        return Ok(false);
    };
    Ok(!participant.has_valid_ownership() && participant.internal.shell_owner_pid == 0)
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
    selection_snapshot: AgentInventorySnapshotV3,
}

fn resolve_command_context(cli: &Cli) -> Result<AgentCommandContext> {
    let cwd = current_dir();
    let effective_config = resolve_effective_config(&cwd, cli)?;
    let (base_policy, _) = substrate_broker::resolve_effective_policy_with_explain(&cwd, false)
        .map_err(|err| config_model::user_error(err.to_string()))?;
    let selection_snapshot = load_agent_inventory_snapshot_v3(&cwd, &base_policy)?;
    let inventory = selection_snapshot.entries.clone();

    Ok(AgentCommandContext {
        effective_config,
        base_policy,
        inventory,
        selection_snapshot,
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
    display_label: String,
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
                    display_label: display_label_for_entry(entry),
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
        "agent_id\tdisplay_label\tbackend_id\tkind\texecution.scope\trole\tcapabilities\teligibility\tprotocol"
    );

    for agent in &report.agents {
        let capabilities = capabilities_label(&agent.capabilities_summary);
        let eligibility = if let Some(reason) = &agent.eligibility.reason {
            format!("{}: {}", agent.eligibility.state, reason)
        } else {
            agent.eligibility.state.to_string()
        };
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            agent.agent_id,
            agent.display_label,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    live_tool_support: Option<LiveToolSupportPostureJson>,
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
        live_tool_support: live_tool_support_posture_json_for_selected_orchestrator(
            orchestrator,
            &context.effective_config,
        ),
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
            #[cfg(target_os = "linux")]
            {
                let authority = public_start_authority()?;
                let mut projections = Vec::new();
                match authority.read_a12b_root() {
                    Ok(root) => {
                        for orchestration_session_id in root.session_namespace_map.keys() {
                            let current = authority
                                .resolve_current_exact(orchestration_session_id, None)
                                .map_err(|error| config_model::user_error(error.to_string()))?;
                            if current.authority.lifecycle_posture
                                != HostSessionPostureV1::ActiveAttached
                                || current.caller.descriptor.agent_id != orchestrator.file.id
                                || current.caller.descriptor.backend_id
                                    != orchestrator_report.backend_id
                            {
                                continue;
                            }
                            projections.push(
                                discover_toolbox_episode_transport(orchestration_session_id)
                                    .map_err(|error| {
                                        config_model::user_error(error.to_string())
                                    })?,
                            );
                        }
                    }
                    Err(strict_error) if authority.read_root().is_ok() => {
                        return Err(config_model::user_error(format!(
                            "authority_invalid: cannot resolve strict HSA toolbox authority: {strict_error}"
                        )));
                    }
                    Err(_) => {}
                }

                let available_count = projections
                    .iter()
                    .filter(|projection| {
                        projection.availability == PrivateTransportAvailabilityV1::Available
                    })
                    .count();
                if available_count > 1 || (available_count == 0 && projections.len() > 1) {
                    anyhow::bail!(config_model::user_error(
                        "authority_conflict: multiple exact live HSA toolbox episodes match the selected orchestrator"
                    ));
                }
                let projection = if available_count == 1 {
                    projections.into_iter().find(|projection| {
                        projection.availability == PrivateTransportAvailabilityV1::Available
                    })
                } else {
                    projections.into_iter().next()
                };
                if let Some(projection) = projection {
                    let available =
                        projection.availability == PrivateTransportAvailabilityV1::Available;
                    return Ok(ToolboxStatusReportJson {
                        toolbox_enabled: true,
                        toolbox_version: TOOLBOX_VERSION,
                        transport,
                        endpoint: available
                            .then(|| format!("unix://{}", projection.endpoint_path.display())),
                        endpoint_template,
                        active_orchestration_session_id: Some(
                            projection.binding.orchestration_session_id.clone(),
                        ),
                        active_world_binding: projection.world_id.map(|world_id| {
                            ToolboxActiveWorldBindingJson {
                                world_id,
                                world_generation: projection
                                    .world_generation
                                    .expect("exact HSA world binding is complete"),
                            }
                        }),
                        eligibility: ToolboxEligibilityJson {
                            state: if available {
                                "allowed".to_string()
                            } else {
                                "dependency_unavailable".to_string()
                            },
                            reason: (!available).then(|| {
                                format!(
                                    "toolbox episode transport is {} while durable HSA authority remains intact",
                                    private_transport_availability_label(projection.availability)
                                )
                            }),
                        },
                        orchestrator: Some(orchestrator_report),
                    });
                }

                let legacy_session = AgentRuntimeStateStore::new()?
                    .resolve_single_live_session_for_agent(&orchestrator.file.id)
                    .map_err(|error| config_model::user_error(error.to_string()))?;
                Ok(ToolboxStatusReportJson {
                    toolbox_enabled: true,
                    toolbox_version: TOOLBOX_VERSION,
                    transport,
                    endpoint: None,
                    endpoint_template,
                    active_orchestration_session_id: legacy_session
                        .as_ref()
                        .map(|session| session.orchestration_session_id().to_string()),
                    active_world_binding: None,
                    eligibility: ToolboxEligibilityJson {
                        state: "dependency_unavailable".to_string(),
                        reason: Some(if legacy_session.is_some() {
                            "legacy session projection has no exact current HSA toolbox authority"
                                .to_string()
                        } else {
                            "no live host-scoped HSA orchestrator participant found for the selected orchestrator"
                                .to_string()
                        }),
                    },
                    orchestrator: Some(orchestrator_report),
                })
            }

            #[cfg(not(target_os = "linux"))]
            {
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
        if let Some(live_tool_support) = &orchestrator.live_tool_support {
            println!(
                "  live_tool_support: runtime_family={} | validation_state={} | support_state={} | reason={}",
                live_tool_support.runtime_family,
                live_tool_support.validation_state,
                live_tool_support.support_state,
                live_tool_support.reason
            );
        }
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
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

#[cfg(target_os = "linux")]
fn private_transport_availability_label(
    availability: PrivateTransportAvailabilityV1,
) -> &'static str {
    match availability {
        PrivateTransportAvailabilityV1::Available => "available",
        PrivateTransportAvailabilityV1::Missing => "missing",
        PrivateTransportAvailabilityV1::Refused => "refused",
        PrivateTransportAvailabilityV1::Failed => "failed",
        PrivateTransportAvailabilityV1::Stale => "stale_or_orphaned",
    }
}

#[cfg(not(target_os = "linux"))]
fn toolbox_uds_endpoint(orchestration_session_id: &str) -> Result<String> {
    Ok(format!(
        "unix://{}",
        toolbox_transport_path(orchestration_session_id)?.display()
    ))
}

fn toolbox_uds_endpoint_template() -> Result<String> {
    Ok(format!(
        "unix://{}",
        toolbox_transport_path_for_home(
            &substrate_paths::substrate_home()?,
            "<orchestration_session_id>",
        )
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

fn born_unattached_status_projection(
    session: &AgentRuntimeSessionRecord,
) -> Option<SessionProjection> {
    let anchor =
        crate::execution::agent_runtime::state_store::born_unattached_status_anchor(session)?;
    let world_id = session.session.world_id.clone()?;
    let world_generation = session.session.world_generation?;
    let last_event_ts = session.session.last_active_at.max(anchor.last_status_at());
    Some(SessionProjection {
        last_event_ts,
        session: StatusSessionJson {
            orchestration_session_id: session.session.orchestration_session_id.clone(),
            participant_id: None,
            agent_id: anchor.handle.agent_id.clone(),
            source_kind: "live_runtime",
            backend_id: anchor.handle.backend_id.clone(),
            client: anchor.handle.agent_id.clone(),
            router: PURE_AGENT_ROUTER.to_string(),
            protocol: anchor.handle.protocol.clone(),
            execution: ExecutionScopeJson { scope: "world" },
            role: Some(anchor.handle.role.clone()),
            posture: Some(orchestration_session_posture_label(session.session.posture).to_string()),
            attached_participant_id: session.session.attached_participant_id.clone(),
            pending_inbox_count: Some(session.session.pending_inbox_count),
            last_event_at: last_event_ts.to_rfc3339(),
            world_id: Some(world_id),
            world_generation: Some(world_generation),
        },
        source: SessionProjectionSource {
            identity: StatusIdentityKey {
                orchestration_session_id: session.session.orchestration_session_id.clone(),
                agent_id: anchor.handle.agent_id.clone(),
                execution_scope: "world",
                participant_id: None,
            },
            run_id: anchor.internal.latest_run_id.clone(),
            ts: last_event_ts,
            is_world_scoped: true,
            has_top_level_world_id: true,
            has_top_level_world_generation: true,
        },
    })
}

fn live_session_status_projections(session: &AgentRuntimeSessionRecord) -> Vec<SessionProjection> {
    let mut projections = session
        .status_visible_participants()
        .into_iter()
        .map(|participant| live_participant_status_projection(&session.session, &participant))
        .collect::<Vec<_>>();
    if projections.is_empty() {
        if let Some(projection) = born_unattached_status_projection(session) {
            projections.push(projection);
        }
    }
    projections
}

fn orchestration_session_posture_label(posture: OrchestrationSessionPosture) -> &'static str {
    match posture {
        OrchestrationSessionPosture::ActiveAttached => "active_attached",
        OrchestrationSessionPosture::BornUnattached => "born_unattached",
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
    if projection.source.parent_participant_id.is_some() {
        return Vec::new();
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
        let exact_match = exact
            .get(parent_identity)
            .is_some_and(|runs| runs.contains(candidate));
        if parent_identity.participant_id.is_some() {
            return exact_match;
        }

        exact_match
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

// Placement-aware version-2 rows expose a human-facing label separate from the exact selector.
fn display_label_for_entry(entry: &AgentInventoryEntryV1) -> String {
    if entry.file.version == 2 {
        if let Some(logical_agent_id) = entry.file.id.strip_suffix("-host") {
            return format!("{logical_agent_id} (host)");
        }
        if let Some(logical_agent_id) = entry.file.id.strip_suffix("-world") {
            return format!("{logical_agent_id} (world)");
        }
    }

    entry.file.id.clone()
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

#[derive(Clone, Serialize)]
struct LiveToolSupportPostureJson {
    runtime_family: String,
    validation_state: String,
    support_state: String,
    reason: String,
}

fn live_tool_support_posture_json(posture: LiveToolSupportPosture) -> LiveToolSupportPostureJson {
    LiveToolSupportPostureJson {
        runtime_family: posture.runtime_family.as_agent_kind_str().to_string(),
        validation_state: posture.validation_state.as_str().to_string(),
        support_state: posture.support_state.as_str().to_string(),
        reason: posture.reason.to_string(),
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn live_tool_support_posture_json_for_entry(
    entry: &AgentInventoryEntryV1,
) -> Option<LiveToolSupportPostureJson> {
    resolve_live_tool_support_posture(entry)
        .ok()
        .map(live_tool_support_posture_json)
}

fn live_tool_support_posture_json_for_selected_orchestrator(
    entry: &AgentInventoryEntryV1,
    effective_config: &SubstrateConfig,
) -> Option<LiveToolSupportPostureJson> {
    validate_runtime_realizability(entry, effective_config).ok()?;
    resolve_selected_orchestrator_live_tool_support_posture(entry)
        .ok()
        .map(live_tool_support_posture_json)
}

#[derive(Serialize)]
struct DoctorOrchestratorJson<'a> {
    agent_id: String,
    display_label: String,
    backend_id: String,
    execution: ExecutionScopeJson<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    live_tool_support: Option<LiveToolSupportPostureJson>,
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

fn required_world_boundary_command_argv(
    cli: &Cli,
    #[cfg(target_os = "linux")] install_context: &InstallBootstrapContextCarrierV1,
) -> Result<Vec<OsString>> {
    let mut args = Vec::new();
    #[cfg(target_os = "linux")]
    {
        let encoded = install_context
            .encode()
            .context("failed to encode authenticated install bootstrap context")?;
        args.push(OsString::from("--install-bootstrap-context-v1"));
        args.push(OsString::from(encoded));
    }
    if cli.world {
        args.push(OsString::from("--world"));
    } else if cli.no_world {
        args.push(OsString::from("--no-world"));
    }
    args.extend(
        ["world", "doctor", "--json"]
            .into_iter()
            .map(OsString::from),
    );
    Ok(args)
}

fn build_doctor_report(
    cli: &Cli,
    #[cfg(target_os = "linux")] install_context: &InstallBootstrapContextCarrierV1,
) -> Result<DoctorReportJson<'static>> {
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

    let mut orchestrator = match validate_orchestrator_selection(&effective_config, &inventory) {
        Ok(entry) => {
            checks.push(DoctorCheckJson {
                check: "orchestrator_selection".to_string(),
                status: "pass".to_string(),
                reason: None,
            });
            DoctorOrchestratorJson {
                agent_id: entry.file.id.clone(),
                display_label: display_label_for_entry(entry),
                backend_id: entry.derived_backend_id(),
                execution: ExecutionScopeJson { scope: "host" },
                live_tool_support: None,
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
            orchestrator.live_tool_support = Some(live_tool_support_posture_json(
                descriptor.live_tool_support_posture(),
            ));
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

        match verify_required_world_boundary(
            cli,
            #[cfg(target_os = "linux")]
            install_context,
        ) {
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

fn verify_required_world_boundary(
    cli: &Cli,
    #[cfg(target_os = "linux")] install_context: &InstallBootstrapContextCarrierV1,
) -> RequiredWorldBoundaryState {
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
    let argv = match required_world_boundary_command_argv(
        cli,
        #[cfg(target_os = "linux")]
        install_context,
    ) {
        Ok(argv) => argv,
        Err(err) => {
            return RequiredWorldBoundaryState::Failed {
                reason: format!(
                    "failed to prepare `substrate world doctor --json` for required world-boundary validation: {err}"
                ),
                exit_code: 3,
            };
        }
    };

    let output = match command.args(&argv).output() {
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
        println!("  display_label: {}", orchestrator.display_label);
        println!("  backend_id: {}", orchestrator.backend_id);
        println!("  execution.scope: {}", orchestrator.execution.scope);
        if let Some(live_tool_support) = &orchestrator.live_tool_support {
            println!(
                "  live_tool_support: runtime_family={} | validation_state={} | support_state={} | reason={}",
                live_tool_support.runtime_family,
                live_tool_support.validation_state,
                live_tool_support.support_state,
                live_tool_support.reason
            );
        }
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

#[cfg(all(test, target_os = "linux"))]
pub(crate) use tests::ordinary_component_public_start_keys_for_test;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::agent_inventory::{
        AgentCapabilitiesV1, AgentCliConfigV1, AgentCliRuntimeFamily, AgentConfigKind,
        AgentConfigV1, AgentExecutionConfigV1, AgentFileV1, AgentInventoryEntryV1,
    };
    use crate::execution::agent_runtime::control::ResolvedRuntimeBackendKind;
    use crate::execution::agent_runtime::dispatch_contract::{
        LiveToolSupportState, LiveToolValidationState,
    };
    #[cfg(target_os = "linux")]
    use crate::execution::agent_runtime::host_session_authority::schema::{
        AgentDescriptorV1, AuthorityObjectCommitmentV1, HostAttachCapabilitiesV1,
        HostAttachExecutionClientStartV1, HostAttachLaunchKnobsV1, HostAttachModePreferenceV1,
        HostPostTurnProtocolActorV1, HostPostTurnProtocolEventKindV1, HostPostTurnTerminalReasonV1,
        HostStartupOwnershipProtocolActorV1, HostStartupOwnershipProtocolEventV1,
        PolicyObjectHashInputV1, WorkspaceBindingV1,
    };
    #[cfg(target_os = "linux")]
    use crate::execution::agent_runtime::host_session_authority::start_continuity::{
        EstablishStartContinuationRequestV1, SettleStartTurnRequestV1, StartContinuationOutcomeV1,
        StartTransactionBeginOutcomeV1, StartTurnCompletionKindV1, StartTurnSettlementOutcomeV1,
    };
    #[cfg(target_os = "linux")]
    use crate::execution::agent_runtime::host_session_authority::store_schema::HostSessionTransitionIntentStateV2;
    #[cfg(target_os = "linux")]
    use crate::execution::agent_runtime::host_session_authority::transition::{
        IssueHostSessionTransitionRequestV1, PostTurnResolutionOutcomeV1, ResolvePostTurnRequestV1,
        ResolveStartupOwnershipRequestV1, StartContractMaterialV1,
        StartupOwnershipResolutionOutcomeV1, TransitionApplicationOutcomeV1,
        TransitionClaimOutcomeV1, TransitionIssueOutcomeV1,
    };
    use crate::execution::agent_runtime::mapping::PURE_AGENT_PROTOCOL;
    #[cfg(target_os = "linux")]
    use crate::execution::agent_runtime::state_store::AcceptedWorldWorkIdentityV1;
    use crate::execution::agent_runtime::{
        mapping::AgentRuntimeBackendKind, AgentRuntimeParticipantRecord, AgentRuntimeSessionState,
        OrchestrationObligationAttachState, OrchestrationObligationKind,
        OrchestrationObligationRecord, OrchestrationSessionRecord, OrchestrationSessionState,
    };
    use crate::execution::config_model::{AgentCliMode, AgentExecutionScope};
    #[cfg(target_os = "linux")]
    use crate::execution::install_bootstrap::current_unix_principal_and_home;
    use clap::Parser;
    use serial_test::serial;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    #[cfg(target_os = "linux")]
    use transport_api_types::InstallBootstrapContextV1;

    const SHARED_WORLD_METADATA_ROOT_TEST_ENV: &str = "SUBSTRATE_TEST_SHARED_WORLD_METADATA_ROOT";

    #[cfg(target_os = "linux")]
    #[test]
    fn concurrent_public_successor_requests_reuse_one_durable_identity_before_issuance() {
        let temp = tempfile::tempdir().expect("public successor coordination tempdir");
        let first_home = temp.path().to_path_buf();
        let second_home = first_home.clone();
        let (first_ready_tx, first_ready_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let first = std::thread::spawn(move || {
            let guard = acquire_public_successor_request_guard(
                &first_home,
                "session-exact",
                HostSessionTransitionModeV1::Attach,
                "request-commitment",
            )
            .expect("acquire first public successor request");
            first_ready_tx
                .send(guard.state.clone())
                .expect("publish first request identity");
            release_rx.recv().expect("release first request guard");
        });
        let first_state = first_ready_rx
            .recv()
            .expect("receive first request identity");
        let second = std::thread::spawn(move || {
            acquire_public_successor_request_guard(
                &second_home,
                "session-exact",
                HostSessionTransitionModeV1::Attach,
                "request-commitment",
            )
            .expect("acquire contended public successor request")
        });
        std::thread::sleep(Duration::from_millis(200));
        assert!(
            !second.is_finished(),
            "concurrent request must wait before reading or issuing HSA state"
        );
        release_tx.send(()).expect("release first request");
        first.join().expect("join first request owner");
        let second_guard = second.join().expect("join second request owner");
        assert_eq!(second_guard.state, first_state);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn uncontended_public_successor_retry_reuses_persisted_identity_after_launcher_drop() {
        let temp = tempfile::tempdir().expect("public successor coordination tempdir");
        for (session, mode) in [
            (
                "session-attach-crash-retry",
                HostSessionTransitionModeV1::Attach,
            ),
            (
                "session-resume-crash-retry",
                HostSessionTransitionModeV1::ResumeOneTurn,
            ),
        ] {
            let first = acquire_public_successor_request_guard(
                temp.path(),
                session,
                mode,
                "request-commitment",
            )
            .expect("acquire first public successor request");
            let first_state = first.state.clone();
            drop(first);

            let mismatch = match acquire_public_successor_request_guard(
                temp.path(),
                session,
                mode,
                "substituted-request-commitment",
            ) {
                Ok(_) => panic!("mismatched crash retry must fail closed"),
                Err(error) => error,
            };
            assert!(mismatch.to_string().contains("identity conflicts"));

            let retry = acquire_public_successor_request_guard(
                temp.path(),
                session,
                mode,
                "request-commitment",
            )
            .expect("acquire uncontended crash retry");

            assert_eq!(retry.state, first_state);
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn acknowledged_public_successor_exact_joins_then_distinct_request_rotates_identity() {
        let temp = tempfile::tempdir().expect("public successor coordination tempdir");
        let first = acquire_public_successor_request_guard(
            temp.path(),
            "session-acknowledged",
            HostSessionTransitionModeV1::Attach,
            "request-commitment",
        )
        .expect("acquire first public successor request");
        let first_state = first.state.clone();
        acknowledge_public_successor_request(&first).expect("acknowledge public successor result");
        drop(first);
        let mut joined = acquire_public_successor_request_guard(
            temp.path(),
            "session-acknowledged",
            HostSessionTransitionModeV1::Attach,
            "request-commitment",
        )
        .expect("exact acknowledged retry must join durable request identity");
        assert_eq!(joined.state.intent_id, first_state.intent_id);
        assert!(joined.state.acknowledged);
        mark_public_successor_request_in_flight(&mut joined)
            .expect("mark exact acknowledged identity in flight before HSA issue");
        assert!(!joined.state.acknowledged);
        acknowledge_public_successor_request(&joined)
            .expect("re-acknowledge exact public successor identity");
        drop(joined);

        let sequential = acquire_public_successor_request_guard(
            temp.path(),
            "session-acknowledged",
            HostSessionTransitionModeV1::Attach,
            "distinct-request-commitment",
        )
        .expect("acquire distinct later public successor request");
        assert_ne!(sequential.state.intent_id, first_state.intent_id);
        assert!(!sequential.state.acknowledged);
    }

    #[cfg(unix)]
    #[test]
    fn committed_start_response_is_acknowledged_only_after_successful_exact_rendering() {
        use std::cell::Cell;

        let acknowledged = Cell::new(false);
        let error = commit_rendered_public_start_response(
            || anyhow::bail!("injected response rendering failure"),
            || {
                acknowledged.set(true);
                Ok(())
            },
        )
        .expect_err("failed rendering must retain the awaiting-response transaction phase");
        assert!(error.to_string().contains("rendering failure"));
        assert!(!acknowledged.get());

        let error = commit_rendered_public_start_response(
            || Ok(1),
            || {
                acknowledged.set(true);
                Ok(())
            },
        )
        .expect_err("a rendered terminal failure must preserve its public exit status");
        assert_eq!(public_prompt_rendered_exit_code(&error), Some(1));
        assert!(acknowledged.get());
    }

    #[cfg(unix)]
    #[test]
    fn exact_terminal_start_stream_result_is_acknowledged_without_second_render() {
        use std::cell::Cell;

        let acknowledged = Cell::new(0);
        let rendered = Cell::new(0);
        let error = join_committed_public_start_after_stream_error(
            public_prompt_rendered_exit(1),
            || {
                acknowledged.set(acknowledged.get() + 1);
                Ok(())
            },
            || {
                rendered.set(rendered.get() + 1);
                Ok(())
            },
        )
        .expect_err("a rendered terminal Start failure must preserve its exit status");

        assert_eq!(public_prompt_rendered_exit_code(&error), Some(1));
        assert_eq!(acknowledged.get(), 1);
        assert_eq!(
            rendered.get(),
            0,
            "an exact terminal envelope already emitted by the stream must not be rendered twice"
        );
    }

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &Path) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(previous) = &self.previous {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    fn command_context_for_test(
        mut effective_config: SubstrateConfig,
        base_policy: Policy,
        inventory: BTreeMap<String, AgentInventoryEntryV1>,
    ) -> AgentCommandContext {
        let selection_snapshot =
            start_selection_snapshot_for_test(&mut effective_config, &inventory);
        AgentCommandContext {
            effective_config,
            base_policy,
            inventory,
            selection_snapshot,
        }
    }

    // These controlled unit fixtures use the internal flattened descriptors;
    // production obtains definitions only from the strict V3 source loader.
    fn start_selection_snapshot_for_test(
        config: &mut SubstrateConfig,
        inventory: &BTreeMap<String, AgentInventoryEntryV1>,
    ) -> AgentInventorySnapshotV3 {
        use crate::execution::agent_selection::{
            ConfiguredAgentDefinitionV3, ConfiguredAgentPlacementV3,
        };
        use transport_api_types::{
            AgentConnectionAuthV1, AgentConnectionBillingV1, AgentConnectionProfileV1,
            AgentHarnessFamilyV1, AgentSelectionSourceScopeV1, AgentSelectionSourceV1,
            AgentSelectionTupleV1,
        };
        let source = AgentSelectionSourceV1 {
            scope: AgentSelectionSourceScopeV1::BuiltIn,
            root_identity: "unit-test-config".into(),
            relative_path: "config/agent-routes".into(),
            revision: "fixture-v1".into(),
            sha256: "a".repeat(64),
        };
        config.selection_sources = vec![source.clone()];
        config.selection_preference_source = Some(source.clone());
        config.selection_profiles = vec![
            AgentConnectionProfileV1 {
                id: "built_in:codex-subscription".into(),
                revision: "1".into(),
                provider: "openai".into(),
                billing: AgentConnectionBillingV1::Subscription,
                endpoint_identity: "openai-codex-subscription".into(),
                upstream_base_url: "https://chatgpt.com/backend-api".into(),
                auth: AgentConnectionAuthV1::CodexAccount {
                    reference: "configured-account".into(),
                },
                source: source.clone(),
            },
            AgentConnectionProfileV1 {
                id: "built_in:claude-existing".into(),
                revision: "1".into(),
                provider: "anthropic".into(),
                billing: AgentConnectionBillingV1::Api,
                endpoint_identity: "anthropic-api".into(),
                upstream_base_url: "https://api.anthropic.com".into(),
                auth: AgentConnectionAuthV1::ApiKeyEnvironment {
                    name: "ANTHROPIC_API_KEY".into(),
                },
                source: source.clone(),
            },
        ];
        let mut definitions = BTreeMap::<String, ConfiguredAgentDefinitionV3>::new();
        for entry in inventory.values() {
            let scope = entry.effective_scope(config);
            let placement = match scope {
                AgentExecutionScope::Host => AgentSelectionPlacementV1::Host,
                AgentExecutionScope::World => AgentSelectionPlacementV1::World,
            };
            let suffix = format!("-{}", placement.as_str());
            let Some(logical) = entry.file.id.strip_suffix(&suffix) else {
                continue;
            };
            let Some(family) = entry.cli_runtime_family() else {
                continue;
            };
            let (harness, provider, profile, model) = match family {
                AgentCliRuntimeFamily::Codex => (
                    AgentHarnessFamilyV1::Codex,
                    "openai",
                    "built_in:codex-subscription",
                    "codex-mini-latest",
                ),
                AgentCliRuntimeFamily::ClaudeCode => (
                    AgentHarnessFamilyV1::ClaudeCode,
                    "anthropic",
                    "built_in:claude-existing",
                    "claude-sonnet-4-5-20250929",
                ),
            };
            let definition = definitions.entry(logical.to_string()).or_insert_with(|| {
                ConfiguredAgentDefinitionV3 {
                    logical_agent_id: logical.into(),
                    enabled: true,
                    source: AgentSelectionSourceV1 {
                        scope: AgentSelectionSourceScopeV1::Global,
                        root_identity: "unit-test-inventory".into(),
                        relative_path: format!("agents/{logical}.yaml"),
                        revision: "fixture-v1".into(),
                        sha256: "b".repeat(64),
                    },
                    placements: BTreeMap::new(),
                }
            });
            definition.placements.insert(
                placement,
                ConfiguredAgentPlacementV3 {
                    enabled: entry.file.config.enabled,
                    runtime_family: harness,
                    selection: AgentSelectionTupleV1 {
                        harness,
                        provider: provider.into(),
                        connection_profile: profile.into(),
                        model: model.into(),
                    },
                },
            );
        }
        AgentInventorySnapshotV3 {
            entries: inventory.clone(),
            definitions: definitions.into_values().collect(),
        }
    }

    fn effective_config_for_test(default_scope: AgentExecutionScope) -> SubstrateConfig {
        SubstrateConfig {
            agents: crate::execution::config_model::AgentsConfig {
                enabled: true,
                defaults: crate::execution::config_model::AgentDefaultsConfig {
                    selection: crate::execution::config_model::AgentDefaultsConfig::default()
                        .selection,
                    execution: crate::execution::config_model::AgentDefaultsExecutionConfig {
                        scope: default_scope,
                    },
                    cli: crate::execution::config_model::AgentDefaultsCliConfig {
                        mode: AgentCliMode::Persistent,
                    },
                },
                hub: crate::execution::config_model::AgentHubConfig {
                    orchestrator_agent_id: "codex-host".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn base_policy_for_test(allowed_backends: &[&str]) -> Policy {
        Policy {
            agents_allowed_backends: allowed_backends
                .iter()
                .map(|id| (*id).to_string())
                .collect(),
            ..Policy::default()
        }
    }

    fn inventory_for_test(
        entries: Vec<AgentInventoryEntryV1>,
    ) -> BTreeMap<String, AgentInventoryEntryV1> {
        entries
            .into_iter()
            .map(|entry| (entry.file.id.clone(), entry))
            .collect()
    }

    fn host_start_args() -> AgentStartArgs {
        AgentStartArgs {
            agent: None,
            backend: Some("cli:codex-host".to_string()),
            scope: Some(AgentStartScopeArg::Host),
            disable_capability: vec![AgentDisableCapabilityArg::SessionFork],
            prompt_source: crate::execution::cli::PublicPromptArgs::default(),
            json: false,
        }
    }

    fn world_start_args() -> AgentStartArgs {
        AgentStartArgs {
            agent: None,
            backend: Some("cli:claude_code-world".to_string()),
            scope: Some(AgentStartScopeArg::World),
            disable_capability: vec![AgentDisableCapabilityArg::SessionStop],
            prompt_source: crate::execution::cli::PublicPromptArgs::default(),
            json: false,
        }
    }

    fn with_state_store<T>(test: impl FnOnce(&AgentRuntimeStateStore) -> T) -> T {
        let authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let temp = TempDir::new().expect("tempdir");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
                .expect("private test substrate home");
        }
        authority_env.install_home(temp.path());
        let _shared_world_root_guard = EnvVarGuard::set(
            SHARED_WORLD_METADATA_ROOT_TEST_ENV,
            temp.path().join("shared-worlds").as_path(),
        );
        let store = AgentRuntimeStateStore::new().expect("state store");
        let result = test(&store);
        std::env::remove_var(SHARED_WORLD_METADATA_ROOT_TEST_ENV);
        result
    }

    #[cfg(unix)]
    fn with_private_hsa_state_store<T>(
        test: impl FnOnce(&AgentRuntimeStateStore, &Path) -> T,
    ) -> T {
        use std::os::unix::fs::PermissionsExt;

        let authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").expect("tests require HOME")).join(".cache")
            });
        fs::create_dir_all(&safe_parent).expect("safe authority test parent");
        let temp = tempfile::Builder::new()
            .prefix("substrate-agents-cmd-")
            .tempdir_in(safe_parent)
            .expect("private authority tempdir");
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
            .expect("private authority tempdir mode");
        authority_env.install_home(temp.path());
        let shared_world_root = temp.path().join("shared-worlds");
        let _shared_world_root_guard = EnvVarGuard::set(
            SHARED_WORLD_METADATA_ROOT_TEST_ENV,
            shared_world_root.as_path(),
        );
        let store = AgentRuntimeStateStore::new().expect("state store");
        let result = test(&store, &shared_world_root);
        std::env::remove_var(SHARED_WORLD_METADATA_ROOT_TEST_ENV);
        result
    }

    #[cfg(target_os = "linux")]
    fn hsa_test_timestamp(value: &str) -> TimestampV1 {
        TimestampV1::parse(value).expect("valid HSA test timestamp")
    }

    #[cfg(target_os = "linux")]
    fn seed_settled_public_start_authority(orchestration_session_id: &str) {
        let authority = public_start_authority().expect("open private public Start authority");
        let root = authority.bootstrap().expect("bootstrap private authority");
        let workspace_binding = WorkspaceBindingV1 {
            workspace_root: root.bootstrap_home.clone(),
            authority_store_root: root.bootstrap_home.clone(),
            authority_store_id: root.authority_store_id,
        };
        let request = IssueHostSessionTransitionRequestV1 {
            intent_id: "intent-start-crash-recovery".into(),
            issuer_request_id: "request-start-crash-recovery".into(),
            mode: HostSessionTransitionModeV1::Start,
            authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
            orchestration_session_id: orchestration_session_id.into(),
            shell_trace_session_id: "trace-start-crash-recovery".into(),
            caller: HostSessionTransitionCallerV1 {
                kind: HostSessionTransitionCallerKindV1::PublicCli,
                caller_participant_id: None,
                auto_attach_obligation_id: None,
                auto_attach_claim_owner: None,
            },
            source_authoritative_participant_id: None,
            target_authoritative_participant_id: "participant-start-crash-recovery".into(),
            target_participant_lease_token: b"lease-start-crash-recovery".to_vec(),
            run_id: "run-start-crash-recovery".into(),
            resulting_authoritative_lineage: vec!["participant-start-crash-recovery".into()],
            workspace_binding: workspace_binding.clone(),
            world_binding: None,
            start_contract: StartContractMaterialV1 {
                descriptor: AgentDescriptorV1 {
                    schema_version: 1,
                    agent_id: "codex".into(),
                    backend_id: "cli:codex-host".into(),
                    backend_kind: RuntimeBackendKindV1::Codex,
                    protocol: "substrate.agent.session".into(),
                    execution_scope: AgentExecutionScopeV1::Host,
                    binary_path: "/usr/bin/codex".into(),
                },
                policy: PolicyObjectHashInputV1 {
                    schema_version: 1,
                    policy_revision: "policy-crash-recovery".into(),
                    canonical_policy_snapshot_sha256: "a".repeat(64),
                },
                capabilities: HostAttachCapabilitiesV1 {
                    session_resume: true,
                    session_fork: true,
                    session_stop: true,
                    status_snapshot: true,
                    event_stream: true,
                },
                launch_knobs: HostAttachLaunchKnobsV1 {
                    requested_execution_scope: AgentExecutionScopeV1::Host,
                    host_execution_client_start: HostAttachExecutionClientStartV1::StartNow,
                    attach_mode_preference: HostAttachModePreferenceV1::ContinuityPreferred,
                },
            },
            transition_input: None,
        };
        let TransitionIssueOutcomeV1::Issued(issued) = authority
            .issue_start_at(
                &request,
                hsa_test_timestamp("2026-08-29T12:00:00.000000000Z"),
                300,
            )
            .expect("issue seed Start")
        else {
            panic!("seed Start must issue");
        };
        let claim_request = ClaimHostSessionTransitionRequestV1 {
            intent_id: issued.intent_id.clone(),
            issuer_request_id: issued.issuer_request_id.clone(),
            payload_commitment: issued.payload_commitment.clone(),
            expected_intent_revision: issued.intent_revision,
            claim_id: "claim-start-crash-recovery".into(),
            claimant_attempt_id: "attempt-start-crash-recovery".into(),
        };
        let TransitionClaimOutcomeV1::Claimed(claimed) = authority
            .claim_start_at(
                &claim_request,
                hsa_test_timestamp("2026-08-29T12:00:01.000000000Z"),
                30,
            )
            .expect("claim seed Start")
        else {
            panic!("seed Start must claim");
        };
        let HostSessionTransitionIntentStateV2::Claimed { claim_revision, .. } = claimed.state
        else {
            panic!("seed Start claim must retain claim evidence");
        };
        let TransitionApplicationOutcomeV1::Applied(applied) = authority
            .apply_start_at(
                &ApplyHostSessionTransitionRequestV1 {
                    intent_id: claimed.intent_id,
                    issuer_request_id: claimed.issuer_request_id,
                    payload_commitment: claimed.payload_commitment,
                    expected_intent_revision: claimed.intent_revision,
                    claim_id: claim_request.claim_id,
                    expected_claim_revision: claim_revision,
                },
                hsa_test_timestamp("2026-08-29T12:00:02.000000000Z"),
            )
            .expect("apply seed Start")
        else {
            panic!("seed Start must apply");
        };
        let HostSessionTransitionIntentStateV2::Applied {
            application_result_ref,
            authority_revision_after,
            ..
        } = &applied.state
        else {
            panic!("seed Start must remain applied");
        };
        let root = authority.read_a12a_root().expect("read applied Start root");
        let registration = EstablishStartContinuationRequestV1 {
            start_transaction_id: "stx-crash-recovery".into(),
            request_key_sha256: "b".repeat(64),
            authority_store_id: root.authority_store_id.clone(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            intent_id: request.intent_id.clone(),
            issuer_request_id: request.issuer_request_id.clone(),
            payload_commitment: applied.payload_commitment.clone(),
            application_result_ref: application_result_ref.clone(),
            run_id: request.run_id.clone(),
            expected_authority_revision: *authority_revision_after,
            authoritative_participant_id: request.target_authoritative_participant_id.clone(),
            backend_id: request.start_contract.descriptor.backend_id.clone(),
            protocol: request.start_contract.descriptor.protocol.clone(),
            exchange_id: "exchange-start-crash-recovery".into(),
            exchange_sequence: 1,
            provider_event_kind: "turn.exchange_opened".into(),
            exchange_evidence_sha256: "c".repeat(64),
            internal_uaa_session_id: "uaa-session-crash-recovery".into(),
            observed_at: hsa_test_timestamp("2026-08-29T12:00:04.000000000Z"),
        };
        let transaction = StartTransactionRecordV1 {
            schema_version: 1,
            transaction_id: registration.start_transaction_id.clone(),
            request_key_sha256: registration.request_key_sha256.clone(),
            prompt_sha256: "d".repeat(64),
            authority_store_id: registration.authority_store_id.clone(),
            orchestration_session_id: registration.orchestration_session_id.clone(),
            shell_trace_session_id: request.shell_trace_session_id.clone(),
            authoritative_participant_id: registration.authoritative_participant_id.clone(),
            backend_id: registration.backend_id.clone(),
            protocol: registration.protocol.clone(),
            workspace_root: workspace_binding.workspace_root.physical_path.clone(),
            world_id: None,
            world_generation: None,
            public_backend_id: registration.backend_id.clone(),
            public_scope: "host".into(),
            start_intent_id: registration.intent_id.clone(),
            start_issuer_request_id: registration.issuer_request_id.clone(),
            start_payload_commitment: registration.payload_commitment.clone(),
            start_application_result_ref: registration.application_result_ref.clone(),
            start_run_id: registration.run_id.clone(),
            start_authority_revision: registration.expected_authority_revision,
            created_at: hsa_test_timestamp("2026-08-29T12:00:02.000000000Z"),
            updated_at: hsa_test_timestamp("2026-08-29T12:00:02.000000000Z"),
            state: StartTransactionStateV1::PromptNotSubmitted,
        };
        assert!(matches!(
            authority
                .begin_start_transaction(&transaction)
                .expect("begin Start transaction"),
            StartTransactionBeginOutcomeV1::Applied(_)
        ));
        authority
            .mark_start_prompt_submission_no_replay_barrier(
                &registration.start_transaction_id,
                &registration.request_key_sha256,
                hsa_test_timestamp("2026-08-29T12:00:03.000000000Z"),
            )
            .expect("commit Start no-replay barrier");
        let StartContinuationOutcomeV1::Applied(registered) = authority
            .establish_start_continuation(&registration)
            .expect("establish seed Start continuation")
        else {
            panic!("seed Start continuation must apply");
        };
        let settlement = SettleStartTurnRequestV1 {
            start_transaction_id: registration.start_transaction_id.clone(),
            request_key_sha256: registration.request_key_sha256.clone(),
            authority_store_id: registration.authority_store_id.clone(),
            orchestration_session_id: registration.orchestration_session_id.clone(),
            intent_id: registration.intent_id.clone(),
            issuer_request_id: registration.issuer_request_id.clone(),
            payload_commitment: registration.payload_commitment.clone(),
            application_result_ref: registration.application_result_ref.clone(),
            run_id: registration.run_id.clone(),
            authoritative_participant_id: registration.authoritative_participant_id.clone(),
            backend_id: registration.backend_id.clone(),
            protocol: registration.protocol.clone(),
            registered_resume_handle_ref: registered.resume_handle_ref,
            expected_authority_revision: registered.authority_revision_after,
            protocol_actor: HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
                participant_id: registration.authoritative_participant_id.clone(),
            },
            event_id: "event-start-crash-recovery".into(),
            event_sequence: 2,
            provider_event_kind: "turn.completed".into(),
            thread_id: registration.internal_uaa_session_id,
            turn_id: "turn-start-crash-recovery".into(),
            completion_evidence_sha256: "e".repeat(64),
            kind: StartTurnCompletionKindV1::ResumableClean,
            obligation_ledger_read: None,
            completed_at: hsa_test_timestamp("2026-08-29T12:00:05.000000000Z"),
        };
        assert!(matches!(
            authority
                .settle_start_turn(&settlement)
                .expect("settle seed Start turn"),
            StartTurnSettlementOutcomeV1::Applied(_)
        ));
        let settled = authority
            .resolve_current_exact(orchestration_session_id, None)
            .expect("resolve settled seed Start authority");
        assert_eq!(
            settled.authority.lifecycle_posture,
            HostSessionPostureV1::ParkedResumable
        );
    }

    #[cfg(target_os = "linux")]
    fn opaque_successor_commitment_for_guard_test(
        commitment: &AuthorityObjectCommitmentV1,
    ) -> substrate_common::OpaqueAuthorityCommitmentV1 {
        match commitment {
            AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex } => {
                substrate_common::OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                    digest_hex: digest_hex.clone(),
                }
            }
            AuthorityObjectCommitmentV1::StoreHmacSha256 {
                key_id,
                domain,
                digest_hex,
            } => substrate_common::OpaqueAuthorityCommitmentV1::StoreHmacSha256 {
                key_id: key_id.clone(),
                domain: domain.clone(),
                digest_hex: digest_hex.clone(),
            },
        }
    }

    #[cfg(target_os = "linux")]
    fn settle_authority_managed_successor_for_guard_test(
        plan: &AuthorityManagedSuccessorLaunchPlanV1,
    ) {
        let authority = public_start_authority().expect("open successor guard test authority");
        let HostSessionTransitionIntentStateV3::Applied { applied_at, .. } =
            &plan.applied_transition.state
        else {
            panic!("successor guard test requires an Applied transition");
        };
        match plan.applied_transition.mode {
            HostSessionTransitionModeV1::Attach => {
                let request = ResolveStartupOwnershipRequestV1 {
                    intent_id: plan.applied_transition.intent_id.clone(),
                    issuer_request_id: plan.applied_transition.issuer_request_id.clone(),
                    payload_commitment: plan.applied_transition.payload_commitment.clone(),
                    protocol_actor:
                        HostStartupOwnershipProtocolActorV1::TargetAuthoritativeParticipant {
                            participant_id: plan
                                .applied_transition
                                .target_authoritative_participant_id
                                .clone(),
                        },
                    protocol_event: HostStartupOwnershipProtocolEventV1::OwnershipAccepted {
                        ownership_acknowledgement_id: "guard-test-attach-accepted".into(),
                    },
                    observed_at: applied_at.clone(),
                };
                assert!(matches!(
                    authority
                        .resolve_startup_ownership_at(&request, applied_at.clone(),)
                        .expect("settle Attach startup ownership for guard test"),
                    StartupOwnershipResolutionOutcomeV1::ResolvedSuccessor(_)
                ));
            }
            HostSessionTransitionModeV1::ResumeOneTurn => {
                let HostSessionTransitionIntentStateV3::Applied {
                    claim_id,
                    claimant_attempt_id,
                    authority_revision_after,
                    ..
                } = &plan.applied_transition.state
                else {
                    panic!("successor guard test requires an Applied ResumeOneTurn")
                };
                let request = ResolvePostTurnRequestV1 {
                    intent_id: plan.applied_transition.intent_id.clone(),
                    issuer_request_id: plan.applied_transition.issuer_request_id.clone(),
                    payload_commitment: plan.applied_transition.payload_commitment.clone(),
                    protocol_actor: HostPostTurnProtocolActorV1::LaunchApplicationClaimant {
                        claim_id: claim_id.clone(),
                        claimant_attempt_id: claimant_attempt_id.clone(),
                    },
                    event_id: "guard-test-resume-terminal".into(),
                    event_sequence: 1,
                    kind: HostPostTurnProtocolEventKindV1::TerminalFailure {
                        reason: HostPostTurnTerminalReasonV1::ResumeRuntimeCreationRejected,
                    },
                    emitted_at: applied_at.clone(),
                    acceptance_record_id: "guard-test-acceptance".into(),
                    acceptance_record_revision: 1,
                    stream_id: "guard-test-stream".into(),
                    accepted_work_identity: AcceptedWorldWorkIdentityV1::RetainedTurn {
                        active_run_id: plan.applied_transition.run_id.clone(),
                        message_id: "guard-test-message".into(),
                        target_participant_id: plan
                            .applied_transition
                            .target_authoritative_participant_id
                            .clone(),
                    },
                    host_transition_correlation:
                        substrate_common::HostTransitionWorkCorrelationV1 {
                            schema_version: 1,
                            authority_store_id: plan.authority_store_id.clone(),
                            orchestration_session_id: plan
                                .applied_transition
                                .orchestration_session_id
                                .clone(),
                            authoritative_participant_id: plan
                                .applied_transition
                                .target_authoritative_participant_id
                                .clone(),
                            transition_intent_id: plan.applied_transition.intent_id.clone(),
                            transition_intent_revision_observed: plan
                                .applied_transition
                                .intent_revision,
                            transition_run_id: plan.applied_transition.run_id.clone(),
                            transition_payload_commitment:
                                opaque_successor_commitment_for_guard_test(
                                    &plan.applied_transition.payload_commitment,
                                ),
                            authority_revision_observed: *authority_revision_after,
                        },
                    completed_at: applied_at.clone(),
                };
                assert!(matches!(
                    authority
                        .resolve_post_turn(&request)
                        .expect("settle ResumeOneTurn post-turn result for guard test"),
                    PostTurnResolutionOutcomeV1::Applied(_)
                ));
            }
            HostSessionTransitionModeV1::Start => {
                panic!("Start does not use the authority-managed successor guard")
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn with_applied_public_successor_for_guard_test(
        mode: HostSessionTransitionModeV1,
        test: impl FnOnce(AuthorityManagedSuccessorLaunchPlanV1),
    ) {
        with_private_hsa_state_store(|_, _| {
            let orchestration_session_id = match mode {
                HostSessionTransitionModeV1::Attach => "session-attach-completion-guard",
                HostSessionTransitionModeV1::ResumeOneTurn => "session-resume-completion-guard",
                HostSessionTransitionModeV1::Start => {
                    panic!("Start does not use the authority-managed successor guard")
                }
            };
            seed_settled_public_start_authority(orchestration_session_id);
            let substrate_home = substrate_paths::substrate_home().expect("private substrate home");
            let request_guard = acquire_public_successor_request_guard(
                &substrate_home,
                orchestration_session_id,
                mode,
                "successor-completion-guard-request",
            )
            .expect("acquire public successor request guard for completion race");
            let plan = build_authority_managed_successor_launch_plan(
                orchestration_session_id,
                match mode {
                    HostSessionTransitionModeV1::Attach => AttachLaunchIntent::Reattach,
                    HostSessionTransitionModeV1::ResumeOneTurn => AttachLaunchIntent::DetachedTurn,
                    HostSessionTransitionModeV1::Start => unreachable!(),
                },
                matches!(mode, HostSessionTransitionModeV1::ResumeOneTurn)
                    .then_some("successor completion guard prompt"),
                Some(&request_guard),
            )
            .expect("build Applied successor plan for completion guard test");
            drop(request_guard);
            test(plan);
        });
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial]
    fn completed_public_successor_retry_joins_without_uncontended_relaunch() {
        for mode in [
            HostSessionTransitionModeV1::Attach,
            HostSessionTransitionModeV1::ResumeOneTurn,
        ] {
            with_applied_public_successor_for_guard_test(mode, |plan| {
                let original_permit = acquire_authority_managed_successor_launch_permit(&plan)
                    .expect("acquire original successor launch permit");
                assert!(!original_permit.joined_completed_launch());
                settle_authority_managed_successor_for_guard_test(&plan);
                drop(original_permit);

                let authority = public_start_authority().expect("reopen successor test authority");
                let completed_root = authority
                    .read_a12b_root()
                    .expect("read completed successor root");
                let retry_permit = acquire_authority_managed_successor_launch_permit(&plan)
                    .expect("exact completed retry must join");
                assert!(
                    retry_permit.joined_completed_launch(),
                    "an uncontended completed retry must not regain launch leadership"
                );
                let receipt = launch_authority_managed_successor_owner_helper(
                    &plan,
                    retry_permit,
                    false,
                    true,
                )
                .expect("joined completion must not spawn a helper");
                assert_eq!(receipt.launch_receipt.helper_pid, 0);
                assert_eq!(
                    authority
                        .read_a12b_root()
                        .expect("read HSA after completed exact join"),
                    completed_root,
                    "completed exact join must not create a successor identity"
                );

                let mut substituted_plan = plan.clone();
                substituted_plan.applied_transition.issued_at =
                    hsa_test_timestamp("2026-08-29T12:09:59.000000000Z");
                let mismatch =
                    match acquire_authority_managed_successor_launch_permit(&substituted_plan) {
                        Ok(_) => panic!("substituted completed identity must fail closed"),
                        Err(error) => error,
                    };
                assert!(mismatch.to_string().contains("identity was substituted"));
                assert_eq!(
                    authority
                        .read_a12b_root()
                        .expect("read HSA after completed identity mismatch"),
                    completed_root,
                    "identity mismatch must neither join nor relaunch"
                );
            });
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial]
    fn public_successor_completion_between_probe_and_guard_acquisition_joins_without_relaunch() {
        for mode in [
            HostSessionTransitionModeV1::Attach,
            HostSessionTransitionModeV1::ResumeOneTurn,
        ] {
            with_applied_public_successor_for_guard_test(mode, |plan| {
                let hook_plan = plan.clone();
                crate::execution::agent_runtime::control::inject_authority_successor_completion_race_for_test(
                    move || settle_authority_managed_successor_for_guard_test(&hook_plan),
                );
                let retry_permit = acquire_authority_managed_successor_launch_permit(&plan)
                    .expect("completion race retry must exact-join");
                assert!(
                    retry_permit.joined_completed_launch(),
                    "completion committed after the first probe must suppress leadership"
                );
                let authority = public_start_authority().expect("reopen raced successor authority");
                let completed_root = authority
                    .read_a12b_root()
                    .expect("read raced completed successor root");
                let receipt = launch_authority_managed_successor_owner_helper(
                    &plan,
                    retry_permit,
                    false,
                    true,
                )
                .expect("raced joined completion must not spawn a helper");
                assert_eq!(receipt.launch_receipt.helper_pid, 0);
                assert_eq!(
                    authority
                        .read_a12b_root()
                        .expect("read HSA after raced exact join"),
                    completed_root,
                    "raced completion join must not create a successor identity"
                );
                assert_eq!(completed_root.successor_transition_intent_map.len(), 1);
                assert!(completed_root
                    .successor_transition_intent_map
                    .contains_key(&plan.applied_transition.intent_id));
            });
        }
    }

    #[cfg(target_os = "linux")]
    fn assert_public_successor_static_identity_preserved(
        actual: &HostSessionTransitionIntentV3,
        original: &HostSessionTransitionIntentV3,
    ) {
        assert_eq!(actual.intent_id, original.intent_id);
        assert_eq!(actual.issuer_request_id, original.issuer_request_id);
        assert_eq!(actual.mode, original.mode);
        assert_eq!(
            actual.authority_precondition,
            original.authority_precondition
        );
        assert_eq!(
            actual.orchestration_session_id,
            original.orchestration_session_id
        );
        assert_eq!(
            actual.shell_trace_session_id,
            original.shell_trace_session_id
        );
        assert_eq!(actual.caller, original.caller);
        assert_eq!(
            actual.source_authoritative_participant_id,
            original.source_authoritative_participant_id
        );
        assert_eq!(
            actual.target_authoritative_participant_id,
            original.target_authoritative_participant_id
        );
        assert_eq!(
            actual.target_participant_lease_token_ref,
            original.target_participant_lease_token_ref
        );
        assert_eq!(actual.run_id, original.run_id);
        assert_eq!(
            actual.resulting_authoritative_lineage,
            original.resulting_authoritative_lineage
        );
        assert_eq!(actual.workspace_binding, original.workspace_binding);
        assert_eq!(actual.world_binding, original.world_binding);
        assert_eq!(actual.descriptor_ref, original.descriptor_ref);
        assert_eq!(
            actual.host_attach_contract_ref,
            original.host_attach_contract_ref
        );
        assert_eq!(actual.resume_handle_ref, original.resume_handle_ref);
        assert_eq!(actual.transition_input_ref, original.transition_input_ref);
        assert_eq!(actual.post_turn_disposition, original.post_turn_disposition);
        assert_eq!(actual.transport_payload_ref, original.transport_payload_ref);
        assert_eq!(actual.payload_commitment, original.payload_commitment);
    }

    #[cfg(target_os = "linux")]
    fn assert_public_successor_launcher_crash_recovery(
        mode: HostSessionTransitionModeV1,
        crash_after: &'static str,
    ) {
        with_private_hsa_state_store(|_, _| {
            let orchestration_session_id = match mode {
                HostSessionTransitionModeV1::Attach => "session-attach-launcher-crash",
                HostSessionTransitionModeV1::ResumeOneTurn => {
                    "session-resume-one-turn-launcher-crash"
                }
                HostSessionTransitionModeV1::Start => panic!("Start is not a successor mode"),
            };
            seed_settled_public_start_authority(orchestration_session_id);
            let substrate_home = substrate_paths::substrate_home().expect("private substrate home");
            let transition_input = matches!(mode, HostSessionTransitionModeV1::ResumeOneTurn)
                .then_some("exact public successor transition input");
            let launch_intent = match mode {
                HostSessionTransitionModeV1::Attach => AttachLaunchIntent::Reattach,
                HostSessionTransitionModeV1::ResumeOneTurn => AttachLaunchIntent::DetachedTurn,
                HostSessionTransitionModeV1::Start => unreachable!(),
            };
            let request_commitment = "exact-public-successor-request-commitment";
            let first = acquire_public_successor_request_guard(
                &substrate_home,
                orchestration_session_id,
                mode,
                request_commitment,
            )
            .expect("acquire initial public successor request guard");
            let durable_request = first.state.clone();
            let request_state_root = first.state_root.clone();
            let request_state_path = first.state_path.clone();
            inject_public_successor_launcher_fault_for_test(crash_after);
            let crash = build_authority_managed_successor_launch_plan(
                orchestration_session_id,
                launch_intent,
                transition_input,
                Some(&first),
            )
            .expect_err("launcher fault must interrupt before apply");
            assert!(crash.to_string().contains(crash_after));

            let authority = public_start_authority().expect("reopen public Start authority");
            let after_crash = authority.read_a12b_root().expect("read HSA after crash");
            assert_eq!(after_crash.successor_transition_intent_map.len(), 1);
            let original = after_crash
                .successor_transition_intent_map
                .get(&durable_request.intent_id)
                .expect("original durable successor intent")
                .clone();
            assert_eq!(
                original.issuer_request_id,
                durable_request.issuer_request_id
            );
            assert_eq!(
                original.target_authoritative_participant_id,
                durable_request.target_participant_id
            );
            assert_eq!(original.run_id, durable_request.run_id);
            assert_eq!(original.mode, mode);
            assert!(match (&original.state, crash_after) {
                (HostSessionTransitionIntentStateV3::Issued, "after_issue") => true,
                (
                    HostSessionTransitionIntentStateV3::Claimed {
                        claim_id,
                        claimant_attempt_id,
                        ..
                    },
                    "after_claim",
                ) => {
                    claim_id == &durable_request.claim_id
                        && claimant_attempt_id == &durable_request.claimant_attempt_id
                }
                _ => false,
            });
            if transition_input.is_some() {
                assert!(original.transition_input_ref.is_some());
            } else {
                assert!(original.transition_input_ref.is_none());
            }
            drop(first);

            let mismatch = match acquire_public_successor_request_guard(
                &substrate_home,
                orchestration_session_id,
                mode,
                "substituted-public-successor-request-commitment",
            ) {
                Ok(_) => panic!("mismatched retry identity must fail closed"),
                Err(error) => error,
            };
            assert!(mismatch.to_string().contains("identity conflicts"));
            assert_eq!(
                authority.read_a12b_root().expect("read HSA after mismatch"),
                after_crash,
                "a mismatched retry must not mutate durable HSA state"
            );

            let mut substituted_request = durable_request.clone();
            substituted_request.intent_id = "intent-substituted-crash-retry".into();
            persist_public_successor_request_state(
                &request_state_root,
                &request_state_path,
                &substituted_request,
            )
            .expect("inject conflicting persisted request identity");
            let conflicting_retry = acquire_public_successor_request_guard(
                &substrate_home,
                orchestration_session_id,
                mode,
                request_commitment,
            )
            .expect("acquire substituted durable retry identity");
            let conflict = build_authority_managed_successor_launch_plan(
                orchestration_session_id,
                launch_intent,
                transition_input,
                Some(&conflicting_retry),
            )
            .expect_err("persisted identity conflict must fail before successor issuance");
            assert!(conflict.to_string().contains("identity conflicts"));
            assert_eq!(
                authority
                    .read_a12b_root()
                    .expect("read HSA after persisted identity conflict"),
                after_crash,
                "persisted identity conflict must not strand a successor transition"
            );
            drop(conflicting_retry);
            persist_public_successor_request_state(
                &request_state_root,
                &request_state_path,
                &durable_request,
            )
            .expect("restore exact durable request identity after conflict proof");

            let retry = acquire_public_successor_request_guard(
                &substrate_home,
                orchestration_session_id,
                mode,
                request_commitment,
            )
            .expect("acquire uncontended exact retry");
            assert_eq!(retry.state, durable_request);
            inject_public_successor_launcher_fault_for_test("after_apply");
            let apply_crash = build_authority_managed_successor_launch_plan(
                orchestration_session_id,
                launch_intent,
                transition_input,
                Some(&retry),
            )
            .expect_err("retry must apply the original intent before the injected crash");
            assert!(apply_crash.to_string().contains("after_apply"));
            let after_apply = authority
                .read_a12b_root()
                .expect("read HSA after exact retry apply");
            assert_eq!(after_apply.successor_transition_intent_map.len(), 1);
            let applied = after_apply
                .successor_transition_intent_map
                .get(&durable_request.intent_id)
                .expect("original successor intent after apply")
                .clone();
            assert_public_successor_static_identity_preserved(&applied, &original);
            assert!(matches!(
                &applied.state,
                HostSessionTransitionIntentStateV3::Applied {
                    claim_id,
                    claimant_attempt_id,
                    active_authoritative_participant_id,
                    ..
                } if claim_id == &durable_request.claim_id
                    && claimant_attempt_id == &durable_request.claimant_attempt_id
                    && active_authoritative_participant_id == &durable_request.target_participant_id
            ));
            drop(retry);

            build_authority_managed_successor_launch_plan(
                orchestration_session_id,
                AttachLaunchIntent::DetachedTurn,
                Some("different fresh input must not use an Applied operation's source"),
                None,
            )
            .expect_err("different fresh input must not reuse an Applied operation's source");
            assert_eq!(
                authority.read_a12b_root().unwrap(),
                after_apply,
                "a refused fresh input must neither issue nor rewrite the Applied operation"
            );

            let applied_join = acquire_public_successor_request_guard(
                &substrate_home,
                orchestration_session_id,
                mode,
                request_commitment,
            )
            .expect("acquire exact already-applied retry");
            assert_eq!(applied_join.state, durable_request);
            let launch_plan = build_authority_managed_successor_launch_plan(
                orchestration_session_id,
                launch_intent,
                transition_input,
                Some(&applied_join),
            )
            .expect("exact retry must join the already-applied transition");
            assert_eq!(launch_plan.applied_transition, applied);
            assert_eq!(
                launch_plan.helper_plan.participant.participant_id,
                durable_request.target_participant_id
            );
            assert_eq!(
                launch_plan.helper_plan.participant.lease_token,
                durable_request.participant_lease_token
            );
            assert_eq!(
                launch_plan.helper_plan.participant.run_id,
                durable_request.run_id
            );
            assert_eq!(
                launch_plan
                    .helper_plan
                    .participant
                    .internal_uaa_session_id
                    .as_deref(),
                Some("uaa-session-crash-recovery"),
                "exact retry must recover the seed producer's original native session"
            );
            assert_eq!(
                authority
                    .read_a12b_root()
                    .expect("read HSA after exact applied join"),
                after_apply,
                "an already-applied exact join must not issue a successor"
            );
            acknowledge_public_successor_request(&applied_join)
                .expect("acknowledge joined public successor result");
            drop(applied_join);

            let acknowledged_join = acquire_public_successor_request_guard(
                &substrate_home,
                orchestration_session_id,
                mode,
                request_commitment,
            )
            .expect("acquire exact acknowledged applied retry");
            assert!(acknowledged_join.state.acknowledged);
            assert_eq!(acknowledged_join.state.intent_id, durable_request.intent_id);
            let acknowledged_plan = build_authority_managed_successor_launch_plan(
                orchestration_session_id,
                launch_intent,
                transition_input,
                Some(&acknowledged_join),
            )
            .expect("acknowledged exact retry must join the committed result idempotently");
            assert_eq!(acknowledged_plan.applied_transition, applied);
            assert_eq!(
                authority
                    .read_a12b_root()
                    .expect("read HSA after acknowledged exact join"),
                after_apply,
                "an acknowledged exact join must not issue a successor"
            );
        });
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial]
    fn public_attach_fault_after_issue_or_claim_reuses_exact_persisted_transition() {
        for crash_after in ["after_issue", "after_claim"] {
            assert_public_successor_launcher_crash_recovery(
                HostSessionTransitionModeV1::Attach,
                crash_after,
            );
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial]
    fn public_resume_one_turn_fault_after_issue_or_claim_reuses_exact_persisted_transition() {
        for crash_after in ["after_issue", "after_claim"] {
            assert_public_successor_launcher_crash_recovery(
                HostSessionTransitionModeV1::ResumeOneTurn,
                crash_after,
            );
        }
    }

    #[cfg(target_os = "linux")]
    fn install_context_for_test(selected_prefix: &Path) -> InstallBootstrapContextCarrierV1 {
        let (principal, _) = current_unix_principal_and_home().expect("current Unix principal");
        let transport_api_types::PlatformPrincipalV1::Unix { account, uid } = principal else {
            panic!("expected Unix principal");
        };
        let context = InstallBootstrapContextV1::new_unix(
            selected_prefix.to_str().expect("UTF-8 selected prefix"),
            &account,
            uid,
        )
        .expect("install bootstrap context");
        InstallBootstrapContextCarrierV1::from_context(context).expect("install bootstrap carrier")
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn required_world_boundary_command_argv_carries_authenticated_install_context() {
        let temp = TempDir::new().expect("tempdir");
        let selected_prefix = temp.path().join("selected-prefix");
        fs::create_dir_all(&selected_prefix).expect("create selected prefix");
        let install_context = install_context_for_test(&selected_prefix);
        let expected = install_context.encode().expect("encode carrier");
        let cli = Cli::try_parse_from(["substrate", "agent", "doctor", "--json"])
            .expect("CLI should parse");

        let argv = required_world_boundary_command_argv(&cli, &install_context)
            .expect("required world-boundary argv");
        let rendered = argv
            .iter()
            .map(|value| value.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            rendered,
            vec![
                "--install-bootstrap-context-v1".to_string(),
                expected,
                "world".to_string(),
                "doctor".to_string(),
                "--json".to_string(),
            ]
        );
    }

    #[allow(
        dead_code,
        reason = "retain the shared-world-root command test fixture helper while current cases use per-test setup"
    )]
    fn with_state_store_and_shared_world_root<T>(
        test: impl FnOnce(&AgentRuntimeStateStore, &Path) -> T,
    ) -> T {
        let authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let temp = TempDir::new().expect("tempdir");
        let shared_world_root = temp.path().join("shared-worlds");
        authority_env.install_home(temp.path());
        let _shared_world_root_guard = EnvVarGuard::set(
            SHARED_WORLD_METADATA_ROOT_TEST_ENV,
            shared_world_root.as_path(),
        );
        let store = AgentRuntimeStateStore::new().expect("state store");
        let result = test(&store, &shared_world_root);
        std::env::remove_var(SHARED_WORLD_METADATA_ROOT_TEST_ENV);
        result
    }

    fn host_descriptor() -> RuntimeSelectionDescriptor {
        RuntimeSelectionDescriptor {
            agent_id: "codex-host".to_string(),
            backend_id: "cli:codex-host".to_string(),
            backend_kind: AgentRuntimeBackendKind::Codex,
            protocol: PURE_AGENT_PROTOCOL.to_string(),
            execution_scope: AgentExecutionScope::Host,
            binary_path: PathBuf::from("/bin/sh"),
        }
    }

    fn detached_orchestrator(
        orchestration_session_id: &str,
        participant_id: &str,
    ) -> (OrchestrationSessionRecord, AgentRuntimeParticipantRecord) {
        let mut participant = AgentRuntimeParticipantRecord::new_orchestrator_participant(
            &host_descriptor(),
            orchestration_session_id.to_string(),
            participant_id.to_string(),
            format!("lease_{participant_id}"),
        )
        .expect("orchestrator participant");
        participant.transition_state(AgentRuntimeSessionState::Ready);
        participant.set_uaa_session_id(format!("uaa-{orchestration_session_id}"));
        participant.mark_client_detached("owner detached cleanly");
        participant.touch_heartbeat();

        let mut orchestration = OrchestrationSessionRecord::new(
            orchestration_session_id.to_string(),
            format!("trace_{orchestration_session_id}"),
            "/workspace".to_string(),
            &participant,
            HostAttachContract::from_manifest_for_test(&participant),
        );
        orchestration.transition_state(OrchestrationSessionState::Active);
        orchestration.bind_active_session_handle(participant.handle.participant_id.clone());
        orchestration.mark_parked_resumable("owner detached cleanly");
        (orchestration, participant)
    }

    #[cfg(target_os = "linux")]
    fn write_shared_world_metadata_for_test(
        metadata_root: &Path,
        world_id: &str,
        orchestration_session_id: &str,
        world_generation: u64,
    ) -> PathBuf {
        let metadata_dir = metadata_root.join(world_id);
        fs::create_dir_all(&metadata_dir).expect("shared world metadata dir");
        let metadata_path = metadata_dir.join("session.json");
        fs::write(
            &metadata_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "world_id": world_id,
                "project_dir": "/workspace",
                "isolate_network": true,
                "always_isolate": true,
                "allowed_domains": [],
                "cgroup_path": "/tmp",
                "started_at_unix_millis": 0,
                "owner_mode": "shared_orchestration",
                "orchestration_session_id": orchestration_session_id,
                "world_generation": world_generation,
                "binding_state": "active",
            }))
            .expect("serialize shared world metadata"),
        )
        .expect("write shared world metadata");
        metadata_dir
    }

    fn eligible_obligation(
        orchestration_session_id: &str,
        obligation_id: &str,
    ) -> OrchestrationObligationRecord {
        let mut obligation = OrchestrationObligationRecord::new(
            orchestration_session_id.to_string(),
            obligation_id.to_string(),
            OrchestrationObligationKind::Blocked,
            format!("summary for {obligation_id}"),
        );
        obligation.attention_required = true;
        obligation.attach_state = OrchestrationObligationAttachState::Eligible;
        obligation
    }

    fn omitted_scope_start_args(backend_id: &str) -> AgentStartArgs {
        AgentStartArgs {
            agent: None,
            backend: Some(backend_id.to_string()),
            scope: None,
            disable_capability: Vec::new(),
            prompt_source: crate::execution::cli::PublicPromptArgs::default(),
            json: false,
        }
    }

    fn inventory_entry_for_test(
        agent_id: &str,
        runtime_family: &str,
        scope: AgentExecutionScope,
    ) -> AgentInventoryEntryV1 {
        let binary = std::env::current_exe()
            .expect("current test binary")
            .display()
            .to_string();
        AgentInventoryEntryV1 {
            path: PathBuf::from(format!("{agent_id}.yaml")),
            file: AgentFileV1 {
                version: 1,
                id: agent_id.to_string(),
                config: AgentConfigV1 {
                    enabled: true,
                    kind: AgentConfigKind::Cli,
                    protocol: Some(PURE_AGENT_PROTOCOL.to_string()),
                    execution: AgentExecutionConfigV1 { scope: Some(scope) },
                    cli: Some(AgentCliConfigV1 {
                        binary,
                        mode: Some(AgentCliMode::Persistent),
                        runtime_family: Some(match runtime_family {
                            "codex" => AgentCliRuntimeFamily::Codex,
                            "claude_code" => AgentCliRuntimeFamily::ClaudeCode,
                            other => panic!("unexpected runtime_family `{other}`"),
                        }),
                    }),
                    api: None,
                    capabilities: AgentCapabilitiesV1 {
                        session_start: true,
                        session_resume: true,
                        session_fork: true,
                        session_stop: true,
                        status_snapshot: true,
                        event_stream: true,
                        llm: true,
                        mcp_client: false,
                    },
                },
                policy_overlay: None,
            },
        }
    }

    fn inventory_entry_with_optional_scope_for_test(
        agent_id: &str,
        runtime_family: &str,
        scope: Option<AgentExecutionScope>,
    ) -> AgentInventoryEntryV1 {
        let mut entry = inventory_entry_for_test(
            agent_id,
            runtime_family,
            scope.unwrap_or(AgentExecutionScope::Host),
        );
        entry.file.config.execution.scope = scope;
        entry
    }

    #[test]
    fn live_tool_support_posture_json_tracks_runtime_family_not_agent_id() {
        let entry = inventory_entry_for_test(
            "workspace_orchestrator_alias",
            "codex",
            AgentExecutionScope::Host,
        );

        let posture = live_tool_support_posture_json_for_entry(&entry)
            .expect("posture should resolve from runtime_family");
        assert_eq!(posture.runtime_family, "codex");
        assert_eq!(posture.validation_state, "smoke_validated");
        assert_eq!(posture.support_state, "first_supported_floor");
    }

    #[test]
    fn toolbox_status_keeps_selected_claude_runtime_readable_while_marking_posture_validated() {
        with_state_store(|_| {
            let agent_id = "host_orchestrator_alias";
            let effective_config = SubstrateConfig {
                agents: crate::execution::config_model::AgentsConfig {
                    enabled: true,
                    hub: crate::execution::config_model::AgentHubConfig {
                        orchestrator_agent_id: agent_id.to_string(),
                        ..Default::default()
                    },
                    toolbox: crate::execution::config_model::AgentToolboxConfig {
                        enabled: true,
                        bind: crate::execution::config_model::AgentToolboxBindConfig {
                            transport: AgentToolboxBindTransport::Uds,
                        },
                    },
                    ..Default::default()
                },
                ..Default::default()
            };
            let base_policy = Policy {
                agents_allowed_backends: vec![format!("cli:{agent_id}")],
                ..Policy::default()
            };
            let mut inventory = BTreeMap::new();
            inventory.insert(
                agent_id.to_string(),
                inventory_entry_for_test(agent_id, "claude_code", AgentExecutionScope::Host),
            );
            let context = command_context_for_test(effective_config, base_policy, inventory);

            let report =
                build_toolbox_status_report(&context).expect("toolbox status should stay readable");
            assert_eq!(report.eligibility.state, "dependency_unavailable");
            let posture = report
                .orchestrator
                .and_then(|orchestrator| orchestrator.live_tool_support)
                .expect("selected host orchestrator should publish support posture");
            assert_eq!(posture.runtime_family, "claude_code");
            assert_eq!(
                posture.validation_state,
                LiveToolValidationState::SmokeValidated.as_str()
            );
            assert_eq!(
                posture.support_state,
                LiveToolSupportState::SelectedRuntimeSupported.as_str()
            );
        });
    }

    #[test]
    fn toolbox_status_hides_selected_runtime_posture_when_binary_is_unrealizable() {
        with_state_store(|_| {
            let agent_id = "host_orchestrator_alias";
            let effective_config = SubstrateConfig {
                agents: crate::execution::config_model::AgentsConfig {
                    enabled: true,
                    hub: crate::execution::config_model::AgentHubConfig {
                        orchestrator_agent_id: agent_id.to_string(),
                        ..Default::default()
                    },
                    toolbox: crate::execution::config_model::AgentToolboxConfig {
                        enabled: true,
                        bind: crate::execution::config_model::AgentToolboxBindConfig {
                            transport: AgentToolboxBindTransport::Uds,
                        },
                    },
                    ..Default::default()
                },
                ..Default::default()
            };
            let base_policy = Policy {
                agents_allowed_backends: vec![format!("cli:{agent_id}")],
                ..Policy::default()
            };
            let mut entry =
                inventory_entry_for_test(agent_id, "claude_code", AgentExecutionScope::Host);
            entry.file.config.cli.as_mut().expect("cli config").binary =
                "__definitely_missing_selected_claude_binary__".to_string();
            let mut inventory = BTreeMap::new();
            inventory.insert(agent_id.to_string(), entry);
            let context = command_context_for_test(effective_config, base_policy, inventory);

            let report =
                build_toolbox_status_report(&context).expect("toolbox status should stay readable");
            assert_eq!(report.eligibility.state, "dependency_unavailable");
            let orchestrator = report
                .orchestrator
                .expect("selected host orchestrator should still be reported");
            assert!(
                orchestrator.live_tool_support.is_none(),
                "toolbox status must not overclaim selected-runtime support when the configured binary is not runtime-realizable"
            );
        });
    }

    #[test]
    #[serial]
    fn host_start_launch_plan_keeps_eager_host_attach_behavior() {
        let context = command_context_for_test(
            effective_config_for_test(AgentExecutionScope::Host),
            base_policy_for_test(&["cli:codex-host"]),
            inventory_for_test(vec![inventory_entry_for_test(
                "codex-host",
                "codex",
                AgentExecutionScope::Host,
            )]),
        );
        let plan =
            build_host_start_launch_plan(&host_start_args(), &context).expect("host launch plan");

        assert_eq!(plan.helper_plan.descriptor.backend_id, "cli:codex-host");
        assert_eq!(
            plan.helper_plan
                .host_attach_contract
                .as_ref()
                .expect("host attach contract")
                .attach_launch_knobs
                .host_execution_client_start,
            crate::execution::agent_runtime::orchestration_session::HostAttachExecutionClientStart::StartNow
        );
    }

    #[test]
    #[serial]
    fn manual_reattach_verification_failure_releases_claimed_auto_attach_obligation() {
        with_state_store(|store| {
            let orchestration_session_id = "sess_manual_reattach_verify_failure";
            let obligation_id = "obl_manual_reattach_verify_failure";
            let participant_id = "ash_manual_reattach_verify_failure";
            let (session, participant) =
                detached_orchestrator(orchestration_session_id, participant_id);
            store
                .persist_orchestration_session(&session)
                .expect("persist session");
            store
                .persist_participant(&participant)
                .expect("persist participant");
            store
                .persist_obligation(&eligible_obligation(
                    orchestration_session_id,
                    obligation_id,
                ))
                .expect("persist obligation");

            let claimed_auto_attach = match store
                .claim_session_auto_attach(orchestration_session_id, "manual::reattach")
                .expect("claim manual reattach auto-attach")
            {
                SessionAutoAttachClaim::Claimed {
                    obligation_id,
                    attach_claim_owner,
                } => Some((obligation_id, attach_claim_owner)),
                other => panic!("expected claimed auto-attach obligation, got {other:?}"),
            };
            let receipt = HiddenOwnerHelperLaunchReceipt {
                helper_pid: 0,
                orchestration_session_id: orchestration_session_id.to_string(),
                participant_id: participant.handle.participant_id.clone(),
                backend_id: participant.handle.backend_id.clone(),
            };

            let err = verify_manual_reattach_restored_ownership(
                store,
                orchestration_session_id,
                &receipt,
                claimed_auto_attach.as_ref(),
            )
            .expect_err("detached session posture must fail durable ownership verification");
            assert!(
                err.to_string()
                    .contains("did not reach durable active_attached ownership after reattach"),
                "verification failure should stay explanation-ready: {err:#}"
            );

            let obligation = store
                .load_obligation(orchestration_session_id, obligation_id)
                .expect("reload obligation")
                .expect("obligation exists");
            assert_eq!(
                obligation.attach_state,
                OrchestrationObligationAttachState::Eligible
            );
            assert_eq!(obligation.attach_claim_owner, None);
        });
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial]
    fn build_attach_launch_plan_distinguishes_reattach_from_detached_turn_modes() {
        with_private_hsa_state_store(|store, _| {
            let orchestration_session_id = "sess_attach_intents";
            public_start_authority()
                .expect("open HSA")
                .bootstrap()
                .expect("bootstrap empty HSA");

            for intent in [
                AttachLaunchIntent::Reattach,
                AttachLaunchIntent::DetachedTurn,
            ] {
                let error =
                    build_attach_launch_plan_with_store(store, orchestration_session_id, intent)
                        .expect_err(
                        "legacy StateStore-only targets must not produce public successor plans",
                    );
                assert!(
                    error.to_string().contains("owner_unreachable")
                        || error.to_string().contains("authority"),
                    "public Attach/ResumeOneTurn must fail closed without exact HSA authority: {error:#}"
                );
            }
        });
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial]
    fn build_attach_launch_plan_refreshes_stale_shared_world_binding_from_local_metadata() {
        with_private_hsa_state_store(|store, shared_world_root| {
            let orchestration_session_id = "sess_attach_binding_refresh";
            public_start_authority()
                .expect("open HSA")
                .bootstrap()
                .expect("bootstrap empty HSA");

            let world_id = format!("wld_attach_binding_refresh_{}", Uuid::now_v7());
            let metadata_dir = write_shared_world_metadata_for_test(
                shared_world_root,
                &world_id,
                orchestration_session_id,
                0,
            );

            let error = build_attach_launch_plan_with_store(
                store,
                orchestration_session_id,
                AttachLaunchIntent::Reattach,
            )
            .expect_err("legacy world metadata cannot authorize public HSA Attach");

            assert!(
                error.to_string().contains("owner_unreachable")
                    || error.to_string().contains("authority"),
                "public Attach must reject compatibility-only world binding evidence: {error:#}"
            );
            fs::remove_dir_all(metadata_dir).expect("remove shared world metadata dir");
        });
    }

    #[test]
    #[serial]
    fn world_start_launch_plan_builds_host_helper_with_world_visible_identity() {
        let context = command_context_for_test(
            effective_config_for_test(AgentExecutionScope::Host),
            base_policy_for_test(&["cli:codex-host", "cli:claude_code-world"]),
            inventory_for_test(vec![
                inventory_entry_for_test("codex-host", "codex", AgentExecutionScope::Host),
                inventory_entry_for_test(
                    "claude_code-world",
                    "claude_code",
                    AgentExecutionScope::World,
                ),
            ]),
        );
        let plan = build_world_start_session_birth_plan(&world_start_args(), &context)
            .expect("world session birth plan");

        assert_eq!(plan.public_identity.backend_id, "cli:claude_code-world");
        assert_eq!(plan.public_identity.scope, AgentExecutionScope::World);
        assert_eq!(plan.helper_plan.descriptor.backend_id, "cli:codex-host");
        let attach_contract = plan
            .helper_plan
            .host_attach_contract
            .as_ref()
            .expect("persisted attach contract");
        assert_eq!(attach_contract.backend_id, "cli:codex-host");
        assert_eq!(
            attach_contract
                .attach_launch_knobs
                .requested_execution_scope,
            AgentExecutionScope::Host
        );
        assert_eq!(
            attach_contract.attach_launch_knobs.host_execution_client_start,
            crate::execution::agent_runtime::orchestration_session::HostAttachExecutionClientStart::StartNow
        );
        assert!(!attach_contract.capabilities.session_stop);
        assert_eq!(plan.helper_plan.session.world_id, None);
        assert_eq!(plan.helper_plan.session.world_generation, None);
    }

    #[test]
    #[serial]
    fn build_start_launch_plan_routes_world_scope_to_host_helper_path() {
        let context = command_context_for_test(
            effective_config_for_test(AgentExecutionScope::Host),
            base_policy_for_test(&["cli:codex-host", "cli:claude_code-world"]),
            inventory_for_test(vec![
                inventory_entry_for_test("codex-host", "codex", AgentExecutionScope::Host),
                inventory_entry_for_test(
                    "claude_code-world",
                    "claude_code",
                    AgentExecutionScope::World,
                ),
            ]),
        );
        let plan = build_start_launch_plan(&world_start_args(), &context)
            .expect("world start launch plan");

        assert_eq!(plan.public_identity.backend_id, "cli:claude_code-world");
        assert_eq!(plan.public_identity.scope, AgentExecutionScope::World);
        assert_eq!(plan.helper_plan.descriptor.backend_id, "cli:codex-host");
    }

    #[test]
    #[serial]
    fn build_start_launch_plan_resolves_omitted_scope_from_workspace_defaults_before_global() {
        let context = command_context_for_test(
            effective_config_for_test(AgentExecutionScope::World),
            base_policy_for_test(&["cli:codex-host", "cli:claude_code-world"]),
            inventory_for_test(vec![
                inventory_entry_for_test("codex-host", "codex", AgentExecutionScope::Host),
                inventory_entry_for_test(
                    "claude_code-world",
                    "claude_code",
                    AgentExecutionScope::World,
                ),
            ]),
        );
        let plan =
            build_start_launch_plan(&omitted_scope_start_args("cli:claude_code-world"), &context)
                .expect("omitted scope launch plan");

        assert_eq!(plan.public_identity.backend_id, "cli:claude_code-world");
        assert_eq!(plan.public_identity.scope, AgentExecutionScope::World);
        assert_eq!(plan.helper_plan.descriptor.backend_id, "cli:codex-host");
    }

    #[test]
    #[serial]
    fn build_start_launch_plan_resolves_omitted_scope_from_global_defaults_when_workspace_unset() {
        let context = command_context_for_test(
            effective_config_for_test(AgentExecutionScope::Host),
            base_policy_for_test(&["cli:codex-host", "cli:claude_code-host"]),
            inventory_for_test(vec![
                inventory_entry_for_test("codex-host", "codex", AgentExecutionScope::Host),
                inventory_entry_with_optional_scope_for_test(
                    "claude_code-host",
                    "claude_code",
                    None,
                ),
            ]),
        );
        let plan =
            build_start_launch_plan(&omitted_scope_start_args("cli:claude_code-host"), &context)
                .expect("omitted scope launch plan");

        assert_eq!(
            plan.helper_plan.descriptor.backend_id,
            "cli:claude_code-host"
        );
        assert_eq!(
            plan.resolved_contract.execution_scope,
            AgentExecutionScope::Host
        );
    }

    #[test]
    #[serial]
    fn build_start_launch_plan_resolves_selected_claude_code_host_backend_without_hidden_fallback()
    {
        let context = command_context_for_test(
            effective_config_for_test(AgentExecutionScope::Host),
            base_policy_for_test(&["cli:codex-host", "cli:claude_code-host"]),
            inventory_for_test(vec![
                inventory_entry_for_test("codex-host", "codex", AgentExecutionScope::Host),
                inventory_entry_with_optional_scope_for_test(
                    "claude_code-host",
                    "claude_code",
                    None,
                ),
            ]),
        );
        let mut args = omitted_scope_start_args("cli:claude_code-host");
        args.prompt_source.prompt = Some("hello from claude start".to_string());
        let plan = build_start_launch_plan(&args, &context)
            .expect("selected claude_code host launch plan should still resolve");

        assert_eq!(plan.public_identity.backend_id, "cli:claude_code-host");
        assert_eq!(
            plan.helper_plan.descriptor.backend_id,
            "cli:claude_code-host"
        );
        assert_eq!(
            plan.helper_plan.descriptor.backend_kind,
            ResolvedRuntimeBackendKind::ClaudeCode
        );
        assert_eq!(
            plan.resolved_contract.execution_scope,
            AgentExecutionScope::Host
        );
    }

    #[test]
    #[serial]
    fn world_start_session_birth_plan_fails_closed_without_host_attach_backend() {
        let context = command_context_for_test(
            effective_config_for_test(AgentExecutionScope::Host),
            base_policy_for_test(&["cli:claude_code-world"]),
            inventory_for_test(vec![
                inventory_entry_for_test("codex-host", "codex", AgentExecutionScope::Host),
                inventory_entry_for_test(
                    "claude_code-world",
                    "claude_code",
                    AgentExecutionScope::World,
                ),
            ]),
        );
        let err = build_world_start_session_birth_plan(&world_start_args(), &context)
            .expect_err("missing host attach backend must fail closed");

        assert!(err.to_string().contains("policy_disallow"));
    }

    fn selected_start_context_for_test() -> AgentCommandContext {
        command_context_for_test(
            effective_config_for_test(AgentExecutionScope::Host),
            base_policy_for_test(&["cli:codex-host", "cli:codex-world", "cli:claude_code-world"]),
            inventory_for_test(vec![
                inventory_entry_for_test("codex-host", "codex", AgentExecutionScope::Host),
                inventory_entry_for_test("codex-world", "codex", AgentExecutionScope::World),
                inventory_entry_for_test(
                    "claude_code-world",
                    "claude_code",
                    AgentExecutionScope::World,
                ),
            ]),
        )
    }

    #[test]
    #[serial]
    fn test_start_without_explicit_selector_uses_exact_complete_preference() {
        let context = selected_start_context_for_test();
        let mut args = host_start_args();
        args.backend = None;
        let plan = build_start_launch_plan(&args, &context).unwrap();
        assert_eq!(plan.native_selection, plan.requested_selection);
        assert_eq!(plan.native_selection.backend_id, "cli:codex-host");
        assert_eq!(
            plan.native_selection.selection,
            context.effective_config.agents.defaults.selection
        );
        assert_eq!(
            plan.native_selection.selection_origin,
            transport_api_types::AgentSelectionOriginV1::ConfiguredDefault
        );
        assert_eq!(
            plan.native_selection.preference_source,
            context.effective_config.selection_preference_source
        );
    }

    #[test]
    #[serial]
    fn test_start_defaults_refuse_zero_or_multiple_visible_matches() {
        let mut context = selected_start_context_for_test();
        let mut args = host_start_args();
        args.backend = None;
        context.effective_config.agents.defaults.selection.model =
            "missing-configured-model".into();
        assert!(build_start_launch_plan(&args, &context)
            .unwrap_err()
            .to_string()
            .contains("found 0"));
        context.effective_config.agents.defaults.selection.model = "codex-mini-latest".into();
        let mut duplicate = context
            .selection_snapshot
            .definitions
            .iter()
            .find(|entry| entry.logical_agent_id == "codex")
            .unwrap()
            .clone();
        duplicate.logical_agent_id = "other".into();
        duplicate.source.relative_path = "agents/other.yaml".into();
        context.selection_snapshot.definitions.push(duplicate);
        assert!(build_start_launch_plan(&args, &context)
            .unwrap_err()
            .to_string()
            .contains("found 2"));
    }

    #[test]
    #[serial]
    fn test_explicit_agent_and_backend_keep_their_tuple_and_refuse_conflict() {
        let mut context = selected_start_context_for_test();
        context.effective_config.agents.defaults.selection.model = "different-default".into();
        let mut args = world_start_args();
        args.backend = None;
        args.agent = Some("claude_code".into());
        let plan = build_start_launch_plan(&args, &context).unwrap();
        assert_eq!(
            plan.requested_selection.upstream_model_id,
            "claude-sonnet-4-5-20250929"
        );
        assert_eq!(
            plan.requested_selection.selection_origin,
            transport_api_types::AgentSelectionOriginV1::ExplicitAgent
        );
        assert_eq!(plan.requested_selection.preference_source, None);
        assert_eq!(plan.native_selection.backend_id, "cli:codex-host");
        assert_eq!(plan.native_selection.selection.model, "codex-mini-latest");
        args.backend = Some("cli:codex-world".into());
        assert!(build_start_launch_plan(&args, &context)
            .unwrap_err()
            .to_string()
            .contains("found 0"));
        args.backend = Some(String::new());
        assert!(build_start_launch_plan(&args, &context).is_err());
        args.backend = None;
        args.agent = Some(" ".into());
        assert!(build_start_launch_plan(&args, &context).is_err());
    }

    #[test]
    #[serial]
    fn test_start_default_does_not_choose_from_multiple_placements_by_order() {
        let mut context = selected_start_context_for_test();
        context.effective_config.agents.defaults.execution.scope = AgentExecutionScope::World;
        let args = AgentStartArgs::default();
        let plan = build_start_launch_plan(&args, &context).unwrap();
        assert_eq!(plan.requested_selection.backend_id, "cli:codex-world");
        assert_eq!(plan.native_selection.backend_id, "cli:codex-host");
        assert_eq!(
            plan.requested_selection.selection_origin,
            transport_api_types::AgentSelectionOriginV1::ConfiguredDefault
        );
        context.selection_snapshot.definitions.reverse();
        let reordered = build_start_launch_plan(&args, &context).unwrap();
        assert_eq!(reordered.requested_selection, plan.requested_selection);
        assert_eq!(reordered.native_selection, plan.native_selection);
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn test_start_request_key_binds_model_profile_sources_and_default_origin() {
        let context = selected_start_context_for_test();
        let mut args = host_start_args();
        args.backend = None;
        let plan = build_start_launch_plan(&args, &context).unwrap();
        let key = |plan: &HostStartLaunchPlan| {
            public_start_request_key(
                plan,
                &"c".repeat(64),
                &plan.public_identity.backend_id,
                plan.public_identity.scope,
            )
            .unwrap()
        };
        let original = key(&plan);
        for mutation in [
            "model",
            "profile-identity",
            "profile-revision",
            "inventory-source",
            "config-source",
            "origin",
        ] {
            let mut changed = build_start_launch_plan(&args, &context).unwrap();
            let selected = &mut changed.native_selection;
            match mutation {
                "model" => {
                    selected.selection.model = "another-configured-model".into();
                    selected.routed_model_id = selected.selection.model.clone();
                    selected.upstream_model_id = selected.selection.model.clone();
                }
                "profile-identity" => {
                    selected.connection_profile.id = "built_in:another-subscription".into();
                    selected.selection.connection_profile = selected.connection_profile.id.clone();
                }
                "profile-revision" => selected.connection_profile.revision = "2".into(),
                "inventory-source" => selected.inventory_source.sha256 = "d".repeat(64),
                "config-source" => {
                    selected.config_sources[0].sha256 = "e".repeat(64);
                    selected.connection_profile.source = selected.config_sources[0].clone();
                    selected.preference_source = Some(selected.config_sources[0].clone());
                }
                "origin" => {
                    selected.selection_origin =
                        transport_api_types::AgentSelectionOriginV1::ExplicitAgent;
                    selected.preference_source = None;
                }
                _ => unreachable!(),
            }
            changed.native_selection = changed.native_selection.clone().seal().unwrap();
            changed.requested_selection = changed.native_selection.clone();
            assert_ne!(key(&changed), original, "{mutation}");
        }
        let mut wrong = build_start_launch_plan(&args, &context).unwrap();
        wrong.native_selection.selection_digest = "0".repeat(64);
        assert!(public_start_request_key(
            &wrong,
            &"c".repeat(64),
            &wrong.public_identity.backend_id,
            wrong.public_identity.scope
        )
        .is_err());
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn ordinary_component_public_start_keys_for_test(
        helper: &HiddenOwnerHelperLaunchPlan,
        selections: &crate::execution::agent_runtime::native_launch_context::OrdinaryStartSelectionsV1,
    ) -> Result<(String, String)> {
        let context = selected_start_context_for_test();
        let mut args = host_start_args();
        args.backend = None;
        let mut plan = build_start_launch_plan(&args, &context)?;
        plan.helper_plan = helper.clone();
        plan.native_selection = selections.native.clone();
        plan.requested_selection = selections.requested.clone();
        let prompt = helper
            .startup_prompt
            .as_ref()
            .context("component Start prompt absent")?;
        let prompt_sha = public_start_digest(
            "substrate.a1.3-p1.public-start-prompt.v1",
            &[prompt.prompt_text.as_bytes()],
        );
        let key = public_start_request_key(
            &plan,
            &prompt_sha,
            &selections.requested.backend_id,
            AgentExecutionScope::World,
        )?;
        Ok((prompt_sha, key))
    }
    #[cfg(target_os = "linux")]
    #[test]
    #[serial]
    fn test_selected_helper_actual_public_producer() -> Result<()> {
        for (world, interleaved_r0) in [(false, false), (true, false), (true, true)] {
            crate::execution::agent_runtime::native_launch_context::with_selected_helper_component_fixture_for_test(
                world, interleaved_r0, |helper,selections,prompt_sha,public_backend,scope| {
                    // Use the actual production schema2 producer. The unrelated
                    // dispatch contract is never an authentication input here.
                    let context=selected_start_context_for_test();
                    let mut args=host_start_args();args.backend=None;
                    let mut plan=build_start_launch_plan(&args,&context)?;
                    plan.helper_plan=helper.clone();
                    plan.native_selection=selections.native.clone();
                    plan.requested_selection=selections.requested.clone();

                    let original=public_start_request_key(&plan,prompt_sha,public_backend,scope)?;
                    for role in ["native","requested"] {
                        for mutation in ["model","profile-identity","profile-revision","profile-source",
                            "inventory-hash","inventory-root","inventory-path","inventory-revision",
                            "config-hash","origin","escaped-source","control-encoding"] {
                            let mut changed=plan.native_selection.clone();
                            if role=="requested" {changed=plan.requested_selection.clone();}
                            match mutation {
                                "model" => {changed.selection.model="selected-other-model".into();
                                    changed.routed_model_id=changed.selection.model.clone();
                                    changed.upstream_model_id=changed.selection.model.clone();}
                                "profile-identity" => {changed.connection_profile.id="built_in:other".into();
                                    changed.selection.connection_profile=changed.connection_profile.id.clone();}
                                "profile-revision" => changed.connection_profile.revision="other".into(),
                                "profile-source" => {
                                    let old=changed.connection_profile.source.clone();
                                    changed.connection_profile.source.sha256="d".repeat(64);
                                    for source in &mut changed.config_sources {
                                        if source==&old {*source=changed.connection_profile.source.clone();}
                                    }
                                    if changed.preference_source.as_ref()==Some(&old) {
                                        changed.preference_source=Some(changed.connection_profile.source.clone());
                                    }
                                },
                                "inventory-hash" => changed.inventory_source.sha256="e".repeat(64),
                                "inventory-root" => changed.inventory_source.root_identity="/other/root".into(),
                                "inventory-path" => changed.inventory_source.relative_path="agents/other.yaml".into(),
                                "inventory-revision" => changed.inventory_source.revision="other".into(),
                                "config-hash" => {
                                    let old=changed.config_sources[0].clone();
                                    changed.config_sources[0].sha256="f".repeat(64);
                                    if changed.connection_profile.source==old {
                                        changed.connection_profile.source=changed.config_sources[0].clone();
                                    }
                                    if changed.preference_source.as_ref()==Some(&old) {
                                        changed.preference_source=Some(changed.config_sources[0].clone());
                                    }
                                }
                                "origin" => {changed.selection_origin=transport_api_types::AgentSelectionOriginV1::ExplicitBackend;
                                    changed.preference_source=None;}
                                "escaped-source" => changed.inventory_source.root_identity="/quoted\"\\λ".into(),
                                "control-encoding" => {changed.selection.model="selected\t\"\\\u{0001}λ".into();
                                    changed.routed_model_id=changed.selection.model.clone();
                                    changed.upstream_model_id=changed.selection.model.clone();}
                                _ => unreachable!(),
                            }
                            if mutation=="control-encoding" {assert!(changed.seal().is_err());continue;}
                            let changed=changed.seal().map_err(anyhow::Error::msg)?;
                            let saved_native=plan.native_selection.clone();
                            let saved_requested=plan.requested_selection.clone();
                            if scope==AgentExecutionScope::Host {
                                plan.native_selection=changed.clone();plan.requested_selection=changed;
                            } else if role=="native" {plan.native_selection=changed;}
                            else {plan.requested_selection=changed;}
                            assert_ne!(public_start_request_key(&plan,prompt_sha,public_backend,scope)?,original,
                                "{role}/{mutation}");
                            plan.native_selection=saved_native;plan.requested_selection=saved_requested;
                        }
                    }
                    Ok(original)

                })?;
        }
        Ok(())
    }
}

#[cfg(all(test, target_os = "linux"))]
mod public_configured_helper_tests {
    use super::*;
    use anyhow::ensure;
    use std::io::{BufRead, Write};
    use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};

    fn checked_public_completion(
        bytes: &[u8],
        action: PublicPromptAction,
        backend: &str,
        scope: &str,
        expected_session: Option<&str>,
    ) -> Result<String> {
        use crate::execution::agent_runtime::control::PublicPromptEnvelope;
        let text = std::str::from_utf8(bytes)?;
        ensure!(
            text.ends_with('\n'),
            "public JSON stream is not newline terminated"
        );
        let mut accepted: Option<(String, String)> = None;
        let mut completed = false;
        for (index, line) in text.lines().enumerate() {
            ensure!(
                !line.is_empty() && !completed,
                "empty frame or frame after terminal"
            );
            // Typed parsing rejects duplicate known fields. Roundtrip equality also
            // rejects unknown fields silently ignored by the supported enum.
            let frame: PublicPromptEnvelope = serde_json::from_str(line)
                .with_context(|| format!("invalid public JSON envelope at frame {index}"))?;
            ensure!(
                serde_json::to_value(&frame)? == serde_json::from_str::<serde_json::Value>(line)?,
                "public frame differs from supported envelope schema"
            );
            match frame {
                PublicPromptEnvelope::Accepted {
                    version,
                    action: actual,
                    orchestration_session_id,
                    backend_id,
                    participant_id,
                    scope: actual_scope,
                } => {
                    ensure!(
                        index == 0
                            && accepted.is_none()
                            && version == 1
                            && actual == action
                            && backend_id == backend
                            && actual_scope == scope,
                        "public acceptance identity/order changed"
                    );
                    ensure!(
                        !orchestration_session_id.is_empty()
                            && expected_session
                                .is_none_or(|expected| expected == orchestration_session_id),
                        "public acceptance session changed"
                    );
                    let participant = participant_id
                        .filter(|id| !id.is_empty())
                        .context("public acceptance participant missing")?;
                    accepted = Some((orchestration_session_id, participant));
                }
                PublicPromptEnvelope::Event {
                    version,
                    event_kind,
                    data,
                } => {
                    ensure!(
                        accepted.is_some()
                            && version == 1
                            && matches!(event_kind.as_str(), "message" | "stderr")
                            && data.is_object(),
                        "public event schema/order changed"
                    );
                }
                PublicPromptEnvelope::Completed {
                    version,
                    action: actual,
                    orchestration_session_id,
                    backend_id,
                    participant_id,
                    turn_outcome,
                    session_posture,
                    state,
                    warnings,
                } => {
                    let identity = accepted
                        .as_ref()
                        .context("public completion preceded acceptance")?;
                    ensure!(
                        version == 1
                            && actual == action
                            && backend_id == backend
                            && orchestration_session_id == identity.0
                            && participant_id.as_deref() == Some(identity.1.as_str())
                            && turn_outcome == "success"
                            && session_posture == PublicSessionPosture::DetachedReattachable
                            && matches!(state.as_str(), "active" | "stopped")
                            && warnings.is_empty(),
                        "public terminal identity/outcome changed"
                    );
                    completed = true;
                }
                PublicPromptEnvelope::Warning { .. } | PublicPromptEnvelope::Failed { .. } => {
                    anyhow::bail!("controlled public helper emitted warning/failure");
                }
            }
        }
        ensure!(completed, "public stream has no terminal completion");
        Ok(accepted.context("public stream has no acceptance")?.0)
    }

    #[test]
    fn test_public_fixture_rejects_invalid_envelopes() -> Result<()> {
        let accepted = serde_json::json!({
            "kind":"accepted","version":1,"action":"start",
            "orchestration_session_id":"session","backend_id":"cli:codex-host",
            "participant_id":"participant","scope":"host",
        });
        let completed = serde_json::json!({
            "kind":"completed","version":1,"action":"start",
            "orchestration_session_id":"session","backend_id":"cli:codex-host",
            "participant_id":"participant","turn_outcome":"success",
            "session_posture":"detached_reattachable","state":"active","warnings":[],
        });
        let valid = format!("{accepted}\n{completed}\n");
        ensure!(
            checked_public_completion(
                valid.as_bytes(),
                PublicPromptAction::Start,
                "cli:codex-host",
                "host",
                None,
            )? == "session"
        );
        let mut unknown = completed.clone();
        unknown["undeclared"] = true.into();
        for bad in [
            format!("{accepted}\nnot-json\n{completed}\n"),
            format!("{completed}\n"),
            format!("{completed}\n{accepted}\n"),
            format!("{valid}{completed}\n"),
            format!("{accepted}\n{unknown}\n"),
            format!("{accepted}\n"),
            valid.trim_end().to_string(),
        ] {
            ensure!(
                checked_public_completion(
                    bad.as_bytes(),
                    PublicPromptAction::Start,
                    "cli:codex-host",
                    "host",
                    None,
                )
                .is_err(),
                "invalid public envelope sequence accepted"
            );
        }
        ensure!(checked_public_completion(
            valid.as_bytes(),
            PublicPromptAction::Turn,
            "cli:codex-host",
            "host",
            None,
        )
        .is_err());
        ensure!(checked_public_completion(
            valid.as_bytes(),
            PublicPromptAction::Start,
            "cli:wrong",
            "host",
            None,
        )
        .is_err());
        Ok(())
    }

    fn native_entry_count(workspace: &Path) -> Result<usize> {
        let stages: Vec<serde_json::Value> =
            fs::read_to_string(workspace.join("host-native-stages.jsonl"))?
                .lines()
                .map(serde_json::from_str)
                .collect::<std::result::Result<_, _>>()?;
        Ok(stages
            .iter()
            .filter(|entry| entry["stage"] == "native_entered")
            .count())
    }

    fn checkpoint(phase: &str, carrier: Option<&str>) -> Result<()> {
        println!(
            "TASK11_FIXTURE {}",
            serde_json::json!({"phase":phase,"carrier":carrier})
        );
        std::io::stdout().flush()?;
        Ok(())
    }

    fn proceed(expected: &str) -> Result<()> {
        let mut line = String::new();
        std::io::stdin().lock().read_line(&mut line)?;
        ensure!(
            line.trim_end() == expected,
            "public helper fixture phase changed"
        );
        Ok(())
    }

    fn private_file(path: &Path, bytes: &[u8]) -> Result<()> {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(())
    }

    fn public_command(carrier: &str, world: bool, args: &[&str]) -> Command {
        let mut command = Command::new("/fixture/substrate");
        command.arg("--install-bootstrap-context-v1").arg(carrier);
        command.arg(if world { "--world" } else { "--no-world" });
        command.args(args).current_dir("/home/fixture/workspace");
        command
    }

    fn wait_public_command(
        child: std::process::Child,
        workspace: &Path,
        turn: usize,
    ) -> Result<std::process::Output> {
        wait_public_command_with_budget(
            child,
            workspace,
            turn,
            Duration::from_secs(if turn == 3 { 180 } else { 45 }),
            None,
        )
    }

    fn wait_public_command_with_budget(
        mut child: std::process::Child,
        workspace: &Path,
        turn: usize,
        budget: Duration,
        mut progress: Option<&mut dyn FnMut() -> Result<()>>,
    ) -> Result<std::process::Output> {
        use std::io::Read as _;

        // Bounded debug-history observation; this is not a product latency SLA.
        let deadline = Instant::now() + budget;
        let native_log = workspace.join("host-native.jsonl");
        let mut released = false;
        let mut stdout = child.stdout.take().context("public stdout pipe missing")?;
        let mut stderr = child.stderr.take().context("public stderr pipe missing")?;
        thread::scope(|scope| {
            // Drain both real pipes while polling; output must not block CLI exit.
            let stdout_reader = scope.spawn(move || {
                let mut bytes = Vec::new();
                stdout.read_to_end(&mut bytes).map(|_| bytes)
            });
            let stderr_reader = scope.spawn(move || {
                let mut bytes = Vec::new();
                stderr.read_to_end(&mut bytes).map(|_| bytes)
            });
            let observed = (|| -> Result<bool> {
                loop {
                    if let Some(progress) = progress.as_mut() {
                        progress()?;
                    }
                    if !released && native_log.is_file() {
                        let rows: Vec<_> = fs::read_to_string(&native_log)?
                            .lines()
                            .map(serde_json::from_str::<serde_json::Value>)
                            .collect::<std::result::Result<_, _>>()?;
                        if rows.len() == turn {
                            let pid = rows[turn - 1]["pid"]
                                .as_u64()
                                .context("native PID missing")?;
                            let stages: Vec<serde_json::Value> =
                                fs::read_to_string(workspace.join("host-native-stages.jsonl"))?
                                    .lines()
                                    .map(serde_json::from_str)
                                    .collect::<std::result::Result<_, _>>()?;
                            if stages.iter().any(|stage| {
                                stage["stage"] == "events_emitted" && stage["pid"] == pid
                            }) {
                                // This fixture stage is after terminal JSON was flushed, before
                                // native exit. It does not itself export the SDK wait/stdio ACK.
                                ensure!(
                                    child.try_wait()?.is_none(),
                                    "public completion preceded held native exit"
                                );
                                private_file(
                                    &workspace.join(format!("host-release-{turn}")),
                                    b"release\n",
                                )?;
                                released = true;
                            }
                        }
                    }
                    if child.try_wait()?.is_some() {
                        return Ok(false);
                    }
                    if Instant::now() >= deadline {
                        return Ok(true);
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            })();
            // Only this unreaped, still-owned CLI child and its own threads.
            let timeout_wait = matches!(observed.as_ref(), Ok(true)).then(|| -> Result<_> {
                let process = PathBuf::from(format!("/proc/{}", child.id()));
                let number = |value: &str| {
                    value.strip_prefix("0x").map_or_else(
                        || value.parse::<i64>().ok(),
                        |hex| i64::from_str_radix(hex, 16).ok(),
                    )
                };
                let mut threads = Vec::new();
                for entry in fs::read_dir(process.join("task"))? {
                    let entry = entry?;
                    let task = entry.path();
                    let syscall =
                        fs::read_to_string(task.join("syscall")).map_err(|e| e.to_string());
                    let mut state = serde_json::json!({
                        "tid": entry.file_name().to_string_lossy().into_owned(),
                        "name": fs::read_to_string(task.join("comm")).map_err(|e| e.to_string()),
                        "wchan": fs::read_to_string(task.join("wchan")).map_err(|e| e.to_string()),
                        "syscall": &syscall,
                    });
                    if let Ok(line) = &syscall {
                        let arguments: Vec<_> = line.split_whitespace().collect();
                        if let [call, fd, flags, ..] = arguments.as_slice() {
                            if let (Some(call), Some(fd), Some(flags)) =
                                (number(call), number(fd), number(flags))
                            {
                                if call == libc::SYS_flock
                                    && fd >= 0
                                    && flags & i64::from(libc::LOCK_NB) == 0
                                    && flags & i64::from(libc::LOCK_SH | libc::LOCK_EX) != 0
                                {
                                    state["blocking_flock"] = serde_json::json!({
                                        "fd": fd,
                                        "target": fs::read_link(process.join("fd").join(fd.to_string()))
                                            .map_err(|e| e.to_string()),
                                    });
                                }
                            }
                        }
                    }
                    threads.push(state);
                }
                Ok(threads)
            });
            if !matches!(observed.as_ref(), Ok(false)) {
                let _ = child.kill();
            }
            // Retain actual child and reader custody on observation errors too.
            let status = child.wait();
            let stdout = stdout_reader
                .join()
                .map_err(|_| anyhow::anyhow!("public stdout reader panicked"))??;
            let stderr = stderr_reader
                .join()
                .map_err(|_| anyhow::anyhow!("public stderr reader panicked"))??;
            let output = std::process::Output {
                status: status?,
                stdout,
                stderr,
            };
            let timed_out = observed.with_context(|| {
                format!(
                    "public command observation failed: turn={turn}; released={released}; status={}; stderr={}; stdout={}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr),
                    String::from_utf8_lossy(&output.stdout)
                )
            })?;
            ensure!(
                !timed_out,
                "public command timed out: turn={turn}; released={released}; wait={timeout_wait:?}; status={}; stderr={}; stdout={}",
                output.status,
                String::from_utf8_lossy(&output.stderr),
                String::from_utf8_lossy(&output.stdout)
            );
            ensure!(
                released,
                "public command ended before its held-terminal stage: turn={turn}; status={}; stderr={}; stdout={}",
                output.status,
                String::from_utf8_lossy(&output.stderr),
                String::from_utf8_lossy(&output.stdout)
            );
            Ok(output)
        })
    }

    fn configured_public_helper_fixture(
        world: bool,
        claude: bool,
        issued_negative: bool,
        failed_start: bool,
    ) -> Result<()> {
        configured_public_helper_fixture_with_world_work(
            world,
            claude,
            issued_negative,
            failed_start,
            false,
            false,
        )
    }

    fn configured_public_helper_fixture_with_world_work(
        world: bool,
        claude: bool,
        issued_negative: bool,
        failed_start: bool,
        world_work: bool,
        current_effects: bool,
    ) -> Result<()> {
        ensure!(!current_effects || world_work);
        ensure!(!world_work || (world && !claude && !issued_negative && !failed_start));
        ensure!(!failed_start || (!world && !claude && !issued_negative));
        ensure!(
            std::env::var("TASK11_PUBLIC_HELPER_FIXTURE").as_deref()
                == Ok(if claude {
                    "claude01"
                } else if world_work {
                    "world-work01"
                } else if world {
                    "world01"
                } else {
                    "host01"
                })
        );
        ensure!(unsafe { libc::getuid() } == 1000 && unsafe { libc::geteuid() } == 1000);
        ensure!(
            !Path::new("/home/spenser").exists(),
            "host home must not be imported"
        );
        let executable = fs::metadata("/fixture/substrate")?;
        ensure!(
            executable.is_file()
                && executable.uid() == 1000
                && executable.mode() & 0o022 == 0
                && executable.mode() & 0o111 != 0
        );
        let home = Path::new("/home/fixture/.substrate");
        let workspace = Path::new("/home/fixture/workspace");
        fs::create_dir(home)?;
        fs::set_permissions(home, fs::Permissions::from_mode(0o700))?;
        fs::DirBuilder::new()
            .mode(0o700)
            .create(home.join("agents"))?;
        fs::create_dir(workspace)?;
        let config = "agents:\n  enabled: true\n  defaults:\n    selection: {harness: codex, provider: openai, connection_profile: \"built_in:codex-subscription\", model: gpt-6-astra}\n  hub:\n    orchestrator_agent_id: codex-host\n";
        let config = if world_work {
            format!("{config}  toolbox:\n    enabled: true\n    bind:\n      transport: uds\n")
        } else if claude {
            config
                .replace(
                    "codex, provider: openai",
                    "claude_code, provider: anthropic",
                )
                .replace("built_in:codex-subscription", "built_in:claude-existing")
                .replace("gpt-6-astra", "claude-sonnet-4-5-20250929")
                .replace("codex-host", "claude_code-host")
        } else {
            config.into()
        };
        private_file(&home.join("config.yaml"), config.as_bytes())?;
        let mut inventory: serde_yaml::Value = serde_yaml::from_str(if claude {
            include_str!("../../../../config/agents/claude_code.yaml")
        } else {
            include_str!("../../../../config/agents/codex.yaml")
        })?;
        if !claude {
            for placement in ["host", "world"] {
                inventory["config"]["placements"][placement]["selection"]["model"] =
                    "gpt-6-astra".into();
            }
        }
        inventory["config"]["placements"]["host"]["cli"]["binary"] =
            workspace.join("codex").to_string_lossy().to_string().into();
        let inventory_path = home.join(if claude {
            "agents/claude_code.yaml"
        } else {
            "agents/codex.yaml"
        });
        private_file(
            &inventory_path,
            serde_yaml::to_string(&inventory)?.as_bytes(),
        )?;
        // The no-toolbox cases preserve the complete raw request; their native
        // check adds only SDK LF. The composition case checks its runtime brief.
        for turn in 1..=3 {
            let prompt = format!("synthetic host turn {turn}");
            private_file(
                &workspace.join(format!("expected-prompt-{turn}.txt")),
                prompt.as_bytes(),
            )?;
        }
        let fixture_source = include_str!("agent_runtime/privileged_host_fixture_preview.rs");
        let script = fixture_source
            .split_once("const HOST_NATIVE_FIXTURE: &str = r##\"")
            .context("native fixture declaration missing")?
            .1
            .split_once("\"##;")
            .context("native fixture end missing")?
            .0;
        let script = script.replace(
            "assert sys.stdin.buffer.read() == expected_prompt.encode('utf-8') + b'\\n'",
            r#"actual_prompt = sys.stdin.buffer.read()
expected_prompt_bytes = (root / ('expected-prompt-%s.txt' % turn)).read_bytes() + b'\n'
assert actual_prompt == expected_prompt_bytes, json.dumps({
    "kind": "task11_synthetic_prompt_mismatch",
    "actual_bytes": len(actual_prompt),
    "expected_bytes": len(expected_prompt_bytes),
    "actual_sha256": __import__("hashlib").sha256(actual_prompt).hexdigest(),
    "expected_sha256": __import__("hashlib").sha256(expected_prompt_bytes).hexdigest(),
    "first_difference": next((i for i, (a, b) in enumerate(zip(actual_prompt, expected_prompt_bytes)) if a != b), min(len(actual_prompt), len(expected_prompt_bytes))),
})"#,
        ).replace(
            "assert args[args.index('--model') + 1] == 'codex-mini-latest'",
            "assert args[args.index('--model') + 1] == 'gpt-6-astra'",
        );
        let script = if claude {
            PUBLIC_CLAUDE_NATIVE_FIXTURE.into()
        } else {
            script
        };
        let script = if failed_start {
            ensure!(script.matches("stage('route_authenticated')").count() == 1);
            script.replace(
                "stage('route_authenticated')",
                "stage('route_authenticated')
stage('controlled_failure')
raise SystemExit(17)",
            )
        } else {
            script
        };
        let script = if world_work {
            let fragment = if current_effects {
                include_str!("agent_runtime/ordinary_public_current_effects_controller.py")
            } else {
                include_str!("agent_runtime/ordinary_public_controller_fixture.py")
            };
            let early = "stage('route_authenticated')";
            ensure!(script.matches(early).count() == 1);
            let script = script.replace(early, &format!("{early}\n{fragment}"));
            // Toolbox-enabled native input intentionally includes its runtime
            // brief. Preserve the exact user request once inside that envelope;
            // the original no-toolbox fixture retains its full-byte assertion.
            let prompt_check = "assert actual_prompt == expected_prompt_bytes,";
            ensure!(script.matches(prompt_check).count() == 1);
            let script = script.replace(
                prompt_check,
                "assert actual_prompt.count(expected_prompt_bytes[:-1]) == 1,",
            );
            let events = "for event in [dict(type='thread.started' if turn == 1 else 'thread.resumed', thread_id=session),\n              dict(type='turn.started', thread_id=session, turn_id='host-turn-%s' % turn),\n              dict(type='turn.completed', thread_id=session, turn_id='host-turn-%s' % turn)]:";
            ensure!(script.matches(events).count() == 1);
            script.replace(events, "for event in [dict(type='turn.completed', thread_id=session, turn_id='host-turn-%s' % turn)]:")
        } else {
            script
        };
        if world_work {
            fs::DirBuilder::new()
                .mode(0o700)
                .create(workspace.join("world-observations"))?;
            private_file(
                &workspace.join("world-native-source.py"),
                if current_effects {
                    include_bytes!("agent_runtime/ordinary_public_current_effects_worker.py")
                        .as_slice()
                } else {
                    include_bytes!("agent_runtime/ordinary_public_worker_fixture.py").as_slice()
                },
            )?;
        }
        private_file(&workspace.join("codex"), script.as_bytes())?;
        fs::set_permissions(workspace.join("codex"), fs::Permissions::from_mode(0o700))?;
        private_file(&home.join("policy.yaml"), b"id: public-configured-helper-fixture\nworld_fs:\n  host_visible: true\n  fail_closed: {routing: false}\n  caged_required: false\n  write: {enabled: true}\nagents:\n  allowed_backends: [cli:codex-host, cli:codex-world]\nllm:\n  allowed_backends: [cli:codex-host, cli:codex-world]\n  constraints:\n    routers: [substrate_gateway]\n    providers: [openai]\n    protocols: [openai.responses]\n    auth_authorities: [codex_subscription]\n  secrets:\n    env_allowed: [SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN, SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID]\n")?;
        if claude {
            let policy = fs::read_to_string(home.join("policy.yaml"))?
                .replace("[cli:codex-host, cli:codex-world]", "[cli:codex-host, cli:codex-world, cli:claude_code-host, cli:claude_code-world]")
                .replace("providers: [openai]", "providers: [openai, anthropic]")
                .replace("protocols: [openai.responses]", "protocols: [openai.responses, anthropic.messages]")
                .replace("auth_authorities: [codex_subscription]", "auth_authorities: [codex_subscription, anthropic_api_key]")
                .replace("env_allowed: [", "env_allowed: [ANTHROPIC_API_KEY, ");
            fs::write(home.join("policy.yaml"), policy)?;
        }
        if world_work {
            let policy = fs::read_to_string(home.join("policy.yaml"))?;
            ensure!(policy.matches("\nllm:\n").count() == 1);
            fs::write(home.join("policy.yaml"), policy.replace("\nllm:\n", "\n  world_dispatch:\n    enabled: true\n    allowed_backends: [cli:codex-world]\n    allowed_actions: [spawn_world_worker, fork_world_worker, continue_world_worker]\n    allowed_modes: [retained]\n    max_live_retained_workers: 4\nllm:\n").as_bytes())?;
        }
        if current_effects {
            let policy = fs::read_to_string(home.join("policy.yaml"))?
                .replace("continue_world_worker]", "continue_world_worker, cancel_world_work]")
                .replace("  world_dispatch:\n", "  host_credentials:\n    read:\n      allowed_backends: [cli:codex-host, cli:codex-world]\n  world_dispatch:\n");
            fs::write(home.join("policy.yaml"), policy.as_bytes())?;
            fs::DirBuilder::new()
                .mode(0o700)
                .create("/home/fixture/.codex")?;
            private_file(Path::new("/home/fixture/.codex/auth.json"), br#"{"tokens":{"access_token":"synthetic-fixed-host-token","account_id":"synthetic-fixed-host-account"}}"#)?;
        }
        let carrier = InstallBootstrapContextCarrierV1::from_context(
            transport_api_types::InstallBootstrapContextV1::new_unix(
                home.to_str().unwrap(),
                "fixture",
                1000,
            )
            .map_err(anyhow::Error::msg)?,
        )
        .map_err(anyhow::Error::msg)?;
        let encoded = carrier.encode().map_err(anyhow::Error::msg)?;
        checkpoint("files", Some(&encoded))?;
        proceed("published")?;
        let held = crate::execution::install_bootstrap::acquire_ordinary_configured_home(&carrier)?;
        held.revalidate()?;
        checkpoint("admitted", None)?;
        proceed("service")?;
        let scope = if world { "world" } else { "host" };
        let mut start = public_command(
            &encoded,
            world,
            &[
                "agent",
                "start",
                "--scope",
                scope,
                "--prompt",
                "synthetic host turn 1",
                "--json",
            ],
        );
        start
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        if current_effects {
            genuine_current_effects::run(start, workspace, held)?;
            checkpoint("current-effects-complete", None)?;
            return Ok(());
        }
        if failed_start {
            use crate::execution::agent_runtime::control::PublicPromptEnvelope;
            let parse = |bytes: &[u8]| -> Result<Vec<PublicPromptEnvelope>> {
                let text = std::str::from_utf8(bytes)?;
                ensure!(text.ends_with('\n'), "failure stream lost its final LF");
                text.lines()
                    .map(|line| {
                        ensure!(!line.is_empty(), "empty public failure frame");
                        let frame: PublicPromptEnvelope = serde_json::from_str(line)?;
                        ensure!(
                            serde_json::to_value(&frame)?
                                == serde_json::from_str::<serde_json::Value>(line)?,
                            "failure frame differs from supported schema"
                        );
                        Ok(frame)
                    })
                    .collect()
            };
            let output = start.output()?;
            ensure!(
                output.status.code() == Some(1) && output.stderr.is_empty(),
                "controlled Start failed: status={}; stderr={}; stdout={}",
                output.status,
                String::from_utf8_lossy(&output.stderr),
                String::from_utf8_lossy(&output.stdout)
            );
            let frames = parse(&output.stdout)?;
            let [PublicPromptEnvelope::Accepted {
                version: 1,
                action: PublicPromptAction::Start,
                orchestration_session_id,
                backend_id,
                participant_id: Some(participant),
                scope: actual_scope,
            }, PublicPromptEnvelope::Failed {
                version: 1,
                terminal: true,
                stage,
                error_code,
                message,
            }] = frames.as_slice()
            else {
                anyhow::bail!("failed public Start did not emit Accepted then exactly one Failed");
            };
            ensure!(
                !orchestration_session_id.is_empty()
                    && !participant.is_empty()
                    && backend_id == "cli:codex-host"
                    && actual_scope == "host"
                    && stage == "runtime"
                    && error_code == "submission_outcome_unknown"
                    && message.contains("codex exited"),
                "failed public Start changed its identity or checked native failure"
            );
            let session = orchestration_session_id.clone();
            let verifier = crate::execution::agent_runtime::native_launch_context::OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(Arc::clone(&held))?;
            let before = verifier.authenticate(&session)?;
            ensure!(before.current_participant_id() == participant);
            let original_identity = (
                before.original_start_intent_id().to_string(),
                before.original_start_issuer_request_id()?,
                before.context_sha256()?,
            );
            ensure!(native_entry_count(workspace)? == 1);
            let retry = start.output()?;
            ensure!(
                retry.status.code() == Some(1) && retry.stderr.is_empty(),
                "controlled Start retry failed: status={}; stderr={}; stdout={}",
                retry.status,
                String::from_utf8_lossy(&retry.stderr),
                String::from_utf8_lossy(&retry.stdout)
            );
            let retry_frames = parse(&retry.stdout)?;
            ensure!(
                matches!(retry_frames.as_slice(),
                    [PublicPromptEnvelope::Failed {
                        version: 1, terminal: true, stage, error_code, message,
                    }] if stage == "runtime"
                        && error_code == "submission_outcome_unknown"
                        && message.contains("the real prompt was not replayed")),
                "indeterminate retry must return exactly one refusal without new acceptance"
            );
            let after = verifier.authenticate(&session)?;
            ensure!(
                after.original_session_id() == session
                    && after.current_participant_id() == participant
                    && original_identity
                        == (
                            after.original_start_intent_id().to_string(),
                            after.original_start_issuer_request_id()?,
                            after.context_sha256()?,
                        ),
                "response-only indeterminate retry changed original authority identities"
            );
            ensure!(
                native_entry_count(workspace)? == 1,
                "indeterminate Start was redelivered"
            );
            let stages: Vec<serde_json::Value> =
                fs::read_to_string(workspace.join("host-native-stages.jsonl"))?
                    .lines()
                    .map(serde_json::from_str)
                    .collect::<std::result::Result<_, _>>()?;
            ensure!(
                stages
                    .iter()
                    .filter(|row| row["stage"] == "controlled_failure")
                    .count()
                    == 1
            );
            checkpoint("failed-start-complete", None)?;
            return Ok(());
        }
        let output = if world_work {
            // New composed body: one actual World native per controller turn,
            // plus controller startup/closeout. Product deadlines are unchanged.
            wait_public_command_with_budget(
                start.spawn()?,
                workspace,
                1,
                Duration::from_secs(150),
                None,
            )?
        } else {
            wait_public_command(start.spawn()?, workspace, 1)?
        };
        ensure!(
            output.status.success(),
            "public Start failed: status={}; stderr={}; stdout={}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
        let public_backend = if claude {
            "cli:claude_code-host"
        } else if world {
            "cli:codex-world"
        } else {
            "cli:codex-host"
        };
        let session = checked_public_completion(
            &output.stdout,
            PublicPromptAction::Start,
            public_backend,
            scope,
            None,
        )?;
        let session = session.as_str();
        let verifier = crate::execution::agent_runtime::native_launch_context::OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(held)?;
        let original = verifier.authenticate(session)?;
        let mut controller_participants = vec![original.current_participant_id().to_string()];
        ensure!(
            original.requested_selection().selection_origin
                == transport_api_types::AgentSelectionOriginV1::ConfiguredDefault
        );
        let original_model = if claude {
            "claude-sonnet-4-5-20250929"
        } else {
            "gpt-6-astra"
        };
        let routed_model = if claude {
            "claude-sonnet-4.5"
        } else {
            "gpt-6-astra"
        };
        for selection in [original.native_selection(), original.requested_selection()] {
            ensure!(selection.selection.model == original_model);
            ensure!(selection.routed_model_id == routed_model);
            ensure!(selection.upstream_model_id == original_model);
            ensure!(!selection.inventory_source.sha256.is_empty());
        }
        ensure!(
            original.native_selection().selection_origin
                == if world {
                    transport_api_types::AgentSelectionOriginV1::ExplicitAgent
                } else {
                    transport_api_types::AgentSelectionOriginV1::ConfiguredDefault
                }
        );
        let inventory_sha = format!("{:x}", Sha256::digest(fs::read(&inventory_path)?));
        let config_sha = format!("{:x}", Sha256::digest(config.as_bytes()));
        for selection in [original.native_selection(), original.requested_selection()] {
            selection.validate().map_err(anyhow::Error::msg)?;
            ensure!(selection.inventory_source.sha256 == inventory_sha);
            ensure!(selection
                .config_sources
                .iter()
                .any(|source| source.sha256 == config_sha));
            ensure!(selection.selection.provider == if claude { "anthropic" } else { "openai" });
            ensure!(
                selection.selection.connection_profile
                    == if claude {
                        "built_in:claude-existing"
                    } else {
                        "built_in:codex-subscription"
                    }
            );
            ensure!(
                selection.selection.harness
                    == if claude {
                        transport_api_types::AgentHarnessFamilyV1::ClaudeCode
                    } else {
                        transport_api_types::AgentHarnessFamilyV1::Codex
                    }
            );
        }
        ensure!(original.native_selection().selection.model == original_model);
        ensure!(original.requested_selection().selection.model == original_model);
        ensure!(original.original_world_binding().is_some() == world);
        let original_carrier =
            crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(
                &original,
            )?;
        // Alter configuration, preference and source SHA. Resume must use the
        // originally issued context and both complete original roles.
        fs::write(
            home.join("config.yaml"),
            config.replace(
                original_model,
                if claude {
                    "claude-sonnet-4.5"
                } else {
                    "codex-mini-latest"
                },
            ),
        )?;
        for placement in ["host", "world"] {
            inventory["config"]["placements"][placement]["selection"]["model"] = if claude {
                "claude-sonnet-4.5"
            } else {
                "codex-mini-latest"
            }
            .into();
        }
        fs::write(&inventory_path, serde_yaml::to_string(&inventory)?)?;
        let public_backend = if claude {
            "cli:claude_code-host"
        } else if world {
            "cli:codex-world"
        } else {
            "cli:codex-host"
        };
        for turn in 2..=3 {
            let prompt = format!("synthetic host turn {turn}");
            let mut command = public_command(
                &encoded,
                world,
                &[
                    "agent",
                    "turn",
                    "--session",
                    session,
                    "--backend",
                    public_backend,
                    "--prompt",
                    &prompt,
                    "--json",
                ],
            );
            command
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());
            let mut accepted_world_turn = None;
            let mut last_observation = Instant::now() - Duration::from_secs(1);
            let mut join_world_terminal = || -> Result<()> {
                if turn != 3
                    || workspace
                        .join("public-world-child-terminal-joined")
                        .exists()
                    || !workspace.join("public-world-child-receipt-ready").is_file()
                    || last_observation.elapsed() < Duration::from_millis(500)
                {
                    return Ok(());
                }
                last_observation = Instant::now();
                if accepted_world_turn.is_none() {
                    use crate::execution::agent_runtime::dispatch_policy_commitment::{
                        resolve_accepted_work_receipt_material_from_snapshot,
                        AcceptedWorkReceiptMaterialResolutionV1,
                        DispatchPolicyCommitmentLookupKeyV1, DispatchPolicyCommitmentSubjectKeyV1,
                    };
                    use crate::execution::agent_runtime::state_store::{
                        authenticate_persisted_world_work_acceptance,
                        WorldWorkAcceptanceLookupKeyV1,
                    };
                    let receipt: crate::execution::agent_runtime::dispatch_contract::ActiveRetainedTurnReceiptV1 =
                        serde_json::from_slice(&fs::read(workspace.join("public-world-child-resume.json"))?)?;
                    let current = verifier.authenticate(session)?;
                    let acceptance = {
                        let snapshot = crate::execution::agent_runtime::host_session_authority::store::read_existing_accepted_work_authority_snapshot(current.held_authority())
                            .map_err(|error| anyhow::anyhow!("{error:?}"))?;
                        let witness = authenticate_persisted_world_work_acceptance(
                            &snapshot,
                            &WorldWorkAcceptanceLookupKeyV1::new(
                                current.authority_store_id(),
                                &receipt.acceptance_record_id,
                            ),
                        )
                        .map_err(|error| anyhow::anyhow!("{error:?}"))?;
                        let acceptance = witness.record().clone();
                        ensure!(
                            acceptance.caller_participant_id == current.current_participant_id()
                                && acceptance.orchestration_session_id == session
                        );
                        let key = DispatchPolicyCommitmentLookupKeyV1 {
                            authority_store_id: acceptance.authority_store_id.clone(),
                            orchestration_session_id: acceptance.orchestration_session_id.clone(),
                            request_id: receipt.request_id.clone(),
                            subject: DispatchPolicyCommitmentSubjectKeyV1::RetainedWorkerTurn {
                                active_run_id: receipt.active_run_id.clone(),
                                message_id: receipt.message_id.clone(),
                            },
                        };
                        let AcceptedWorkReceiptMaterialResolutionV1::Resolved(material) =
                            resolve_accepted_work_receipt_material_from_snapshot(
                                &snapshot,
                                &key,
                                &acceptance,
                            )
                            .map_err(|error| anyhow::anyhow!("{error:?}"))?
                        else {
                            anyhow::bail!("foreground proof requires exact accepted E2 material");
                        };
                        ensure!(crate::execution::orchestrator_world_dispatch::project_active_retained_turn_receipt_for_test(&material)? == receipt,
                            "foreground receipt differs from actual B1/E2 material");
                        acceptance
                    };
                    accepted_world_turn = Some((current, acceptance));
                }
                let native = workspace.join("world-observations/world-native.jsonl");
                if fs::read_to_string(&native)?.lines().count() < 3 {
                    return Ok(());
                }
                let (current, acceptance) = accepted_world_turn.as_ref().unwrap();
                let observation = current
                    .held_authority()
                    .observe_existing_ordinary_world_work_v1(acceptance)?;
                if let Some(terminal) = &observation.terminal {
                    ensure!(
                        terminal.exit_code == 0 && observation.interruption.is_none(),
                        "World turn did not complete cleanly: {observation:?}"
                    );
                    let frame = observation
                        .journal
                        .last()
                        .context("World terminal journal absent")?
                        .decode_for_c1(&observation.claim)?;
                    ensure!(
                        matches!(frame, transport_api_types::ExecuteStreamFrame::Exit {
                        exit: 0, ref frame_identity, ref event_identity, ref terminal_identity, ref span_id, ..
                    } if frame_identity == &terminal.frame_identity && event_identity == &terminal.event_identity
                        && terminal_identity == &terminal.terminal_identity && span_id == &terminal.span_id),
                        "World terminal differs from its exact Root journal frame"
                    );
                    private_file(
                        &workspace.join("public-world-child-terminal-joined"),
                        &serde_json::to_vec(terminal)?,
                    )?;
                }
                Ok(())
            };
            let output = if world_work {
                wait_public_command_with_budget(
                    command.spawn()?,
                    workspace,
                    turn,
                    Duration::from_secs(300),
                    Some(&mut join_world_terminal),
                )?
            } else {
                wait_public_command(command.spawn()?, workspace, turn)?
            };
            ensure!(
                output.status.success(),
                "public Resume {turn} failed: status={}; stderr={}; stdout={}",
                output.status,
                String::from_utf8_lossy(&output.stderr),
                String::from_utf8_lossy(&output.stdout)
            );
            checked_public_completion(
                &output.stdout,
                PublicPromptAction::Turn,
                public_backend,
                scope,
                Some(session),
            )?;
            let current = verifier.authenticate(session)?;
            controller_participants.push(current.current_participant_id().to_string());
            ensure!(
                crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(
                    &current
                )? == original_carrier
            );
        }
        let rows: Vec<serde_json::Value> = fs::read_to_string(workspace.join("host-native.jsonl"))?
            .lines()
            .map(serde_json::from_str)
            .collect::<std::result::Result<_, _>>()?;
        ensure!(rows.len() == 3, "public helper redelivered accepted input");
        for row in &rows {
            ensure!(
                row["home_device"] == rows[0]["home_device"]
                    && row["home_inode"] == rows[0]["home_inode"]
            );
            ensure!(
                row["history_device"] == rows[0]["history_device"]
                    && row["history_inode"] == rows[0]["history_inode"]
            );
            for key in ["pid", "parent_pid"] {
                let pid = row[key].as_u64().context("native PID missing")?;
                let deadline = Instant::now() + Duration::from_secs(10);
                while Path::new(&format!("/proc/{pid}")).exists() {
                    ensure!(
                        Instant::now() < deadline,
                        "observed native/helper PID {pid} remained present"
                    );
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
        let current = verifier.authenticate(session)?;
        let root = current
            .held_authority()
            .resolve_current_exact(session, None)?;
        ensure!(matches!(
            root.authority.lifecycle_posture,
            HostSessionPostureV1::ParkedResumable | HostSessionPostureV1::AwaitingAttention
        ));

        if world_work {
            use crate::execution::agent_runtime::tool_invocation_contract::{
                HostToolForkWorldWorkerReceiptV1, HostToolSpawnWorldWorkerReceiptV1,
            };
            let spawn: HostToolSpawnWorldWorkerReceiptV1 =
                serde_json::from_slice(&fs::read(workspace.join("public-world-source.json"))?)?;
            let fork: HostToolForkWorldWorkerReceiptV1 =
                serde_json::from_slice(&fs::read(workspace.join("public-world-child.json"))?)?;
            let resumed: crate::execution::agent_runtime::dispatch_contract::ActiveRetainedTurnReceiptV1 = serde_json::from_slice(&fs::read(
                workspace.join("public-world-child-resume.json"),
            )?)?;
            ensure!(
                controller_participants
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    == 3
            );
            ensure!(
                [
                    &spawn.orchestrator_participant_id,
                    &fork.orchestrator_participant_id,
                    &resumed.orchestrator_participant_id
                ] == [
                    &controller_participants[0],
                    &controller_participants[1],
                    &controller_participants[2]
                ]
            );
            ensure!(
                spawn.orchestration_session_id == session
                    && fork.orchestration_session_id == session
                    && resumed.orchestration_session_id == session
            );
            ensure!(
                fork.source_participant_id == spawn.participant_id
                    && fork.participant_id != spawn.participant_id
                    && resumed.target_participant_id == fork.participant_id
            );
            ensure!(
                spawn.world_id == fork.world_id
                    && fork.world_id == resumed.world_id
                    && [
                        spawn.world_generation,
                        fork.world_generation,
                        resumed.world_generation
                    ] == [0, 0, 0]
            );
            ensure!(root.authority.retained_worker_refs.len() == 2);
            ensure!(
                root.bound_state_store.load_session(session)?.is_none(),
                "canonical World work must not create a legacy session projection"
            );
            for participant in [&spawn.participant_id, &fork.participant_id] {
                ensure!(
                    root.bound_state_store
                        .load_participant(participant)?
                        .is_none(),
                    "canonical World worker must not create a legacy participant projection"
                );
            }
            let world_rows: Vec<serde_json::Value> =
                fs::read_to_string(workspace.join("world-observations/world-native.jsonl"))?
                    .lines()
                    .map(serde_json::from_str)
                    .collect::<std::result::Result<_, _>>()?;
            ensure!(
                world_rows.len() == 3
                    && world_rows[0]["kind"] == "start"
                    && world_rows[1]["kind"] == "fork"
                    && world_rows[2]["kind"] == "resume_child"
            );
            for row in &world_rows {
                ensure!(
                    row["home_device"] == world_rows[0]["home_device"]
                        && row["home_inode"] == world_rows[0]["home_inode"]
                        && row["malformed_body_refusal"] == true
                );
                let pid = row["pid"].as_u64().context("native World PID missing")?;
                ensure!(
                    row["cgroup_target"]
                        == format!("/sys/fs/cgroup/substrate/{}/cgroup.procs", spawn.world_id)
                        && row["cgroup_self"] == format!("0::/substrate/{}\n", spawn.world_id)
                        && row["cgroup_members"]
                            .as_array()
                            .is_some_and(|members| members
                                .iter()
                                .any(|value| value.as_u64() == Some(pid))),
                    "native placement does not join the genuine receipt World: {row}"
                );
            }
            ensure!(
                world_rows[1]["thread_id"] == world_rows[2]["thread_id"]
                    && world_rows[0]["thread_id"] != world_rows[1]["thread_id"]
                    && world_rows[1]["state_device"] == world_rows[2]["state_device"]
                    && world_rows[1]["state_inode"] == world_rows[2]["state_inode"]
                    && world_rows[1]["turns"] == 1
                    && world_rows[2]["turns"] == 2
            );
            checkpoint("complete", None)?;
            return Ok(());
        }

        if issued_negative {
            let raw_prompt = "must not deliver issued-helper negative";
            let input = raw_prompt.to_string();
            let mut request_guard = acquire_public_successor_request_guard(
                home,
                session,
                HostSessionTransitionModeV1::ResumeOneTurn,
                &public_start_digest(
                    "substrate.public-successor-request.v1",
                    &[
                        session.as_bytes(),
                        public_backend.as_bytes(),
                        raw_prompt.as_bytes(),
                    ],
                ),
            )?;
            mark_public_successor_request_in_flight(&mut request_guard)?;
            let source = Arc::new(verifier.authenticate(session)?);
            let mut plan = build_ordinary_public_resumed_turn_launch_plan(
                &source,
                session,
                &input,
                &request_guard,
            )?;
            let listener = register_durable_start_startup_prompt_listener(
                plan.helper_plan.orchestration_session_id(),
                plan.helper_plan.participant_id(),
            )?;
            let listener_path = listener.path().to_path_buf();
            let listener_identity = fs::symlink_metadata(&listener_path)?;
            plan.helper_plan.startup_prompt = Some(HiddenOwnerHelperStartupPromptPlan {
                prompt_text: input,
                stream_path: listener_path.clone(),
                request_key_sha256: None,
                prompt_sha256: None,
                public_backend_id: None,
                public_scope: None,
            });
            let current = Arc::new(verifier.authenticate(session)?);
            let result = crate::execution::agent_runtime::control::verify_ordinary_issued_helper_refusals_fixture(
                &current, &carrier, &plan, world,
            );
            drop(listener);
            let retained = fs::symlink_metadata(&listener_path)?;
            ensure!(
                retained.dev() == listener_identity.dev()
                    && retained.ino() == listener_identity.ino(),
                "issued-negative listener custody changed"
            );
            fs::remove_file(&listener_path)?;
            result?;
            checkpoint("issued-negatives-complete", None)?;
            return Ok(());
        }
        let native_entries = native_entry_count(workspace)?;
        ensure!(
            native_entries == 3,
            "normal public helper native entry count changed"
        );
        // This malformed-shape negative is supplemental only. Genuine issued
        // stripped/repacked and bound-field tests await the actual HSA comparison API.
        // Repacking ordinary helper data cannot instantiate generic Existing,
        // even when it names a real current root actor.
        let raw = serde_json::json!({
            "mode":"resume_one_turn",
            "descriptor":{
                "agent_id":root.caller.descriptor.agent_id,
                "backend_id":root.caller.descriptor.backend_id,
                "backend_kind":if claude {"claude_code"}else{"codex"},"protocol":root.caller.descriptor.protocol,
                "execution_scope":"host","binary_path":root.caller.descriptor.binary_path,
            },
            "session":{
                "orchestration_session_id":session,
                "shell_trace_session_id":root.authority.shell_trace_session_id,
                "workspace_root":workspace,"world_id":root.authority.world_binding.as_ref().map(|w|w.world_id.clone()),
                "world_generation":root.authority.world_binding.as_ref().map(|w|w.world_generation),
            },
            "participant":{
                "participant_id":root.caller.participant_id,"lease_token":"stripped",
                "run_id":"wrong-current-run","resumed_from_participant_id":"stripped","internal_uaa_session_id":"stripped",
            },
            "source_orchestration_session_id":null,
        });
        let path = workspace.join("stripped-public-helper.json");
        private_file(&path, &serde_json::to_vec(&raw)?)?;
        let denied = public_command(
            &encoded,
            world,
            &[
                "agent",
                "__owner-helper",
                "--plan-file",
                path.to_str().unwrap(),
            ],
        )
        .output()?;
        ensure!(
            !denied.status.success(),
            "repacked ordinary helper entered generic Existing: status={}; stderr={}; stdout={}",
            denied.status,
            String::from_utf8_lossy(&denied.stderr),
            String::from_utf8_lossy(&denied.stdout)
        );
        let context_path = home
            .join("run/agent-hub/ordinary-start-contexts")
            .join(format!("{}.json", current.original_start_intent_id()));
        let held_context = context_path.with_extension("held");
        fs::rename(&context_path, &held_context)?;
        let denied = public_command(
            &encoded,
            world,
            &[
                "agent",
                "turn",
                "--session",
                session,
                "--backend",
                public_backend,
                "--prompt",
                "must not deliver stripped source",
                "--json",
            ],
        )
        .output();
        fs::rename(&held_context, &context_path)?;
        let denied = denied?;
        ensure!(
            !denied.status.success(),
            "missing original issuer entered ordinary fallback: status={}; stderr={}; stdout={}",
            denied.status,
            String::from_utf8_lossy(&denied.stderr),
            String::from_utf8_lossy(&denied.stdout)
        );
        let fork = public_command(
            &encoded,
            world,
            &["agent", "fork", "--session", session, "--json"],
        )
        .output()?;
        ensure!(
            fork.status.success(),
            "genuine initial child allocation failed: status={}; stderr={}; stdout={}",
            fork.status,
            String::from_utf8_lossy(&fork.stderr),
            String::from_utf8_lossy(&fork.stdout)
        );
        let child: serde_json::Value = serde_json::from_slice(&fork.stdout)?;
        ensure!(
            child["action"] == "fork"
                && child["source_orchestration_session_id"] == session
                && child["backend_id"] == original.native_selection().backend_id
                && child["scope"] == "host"
                && child["state"] == "parked_resumable",
            "public fork allocation identity changed"
        );
        // The retained Start->Fork->original-Resume failure needs its original
        // preconditions and separate no-additional-native/provider evidence.
        // This Start+2Resume fixture does not claim equivalence to that failure.
        let child_id = child["orchestration_session_id"]
            .as_str()
            .context("fork child missing")?;
        let denied = public_command(
            &encoded,
            world,
            &[
                "agent",
                "turn",
                "--session",
                child_id,
                "--backend",
                public_backend,
                "--prompt",
                "must not deliver unsupported child",
                "--json",
            ],
        )
        .output()?;
        ensure!(
            !denied.status.success(),
            "unsupported child Resume was enabled: status={}; stderr={}; stdout={}",
            denied.status,
            String::from_utf8_lossy(&denied.stderr),
            String::from_utf8_lossy(&denied.stdout)
        );
        ensure!(
            fs::read_to_string(workspace.join("host-native.jsonl"))?
                .lines()
                .count()
                == 3,
            "a refused source/child/repacked request added native traffic"
        );
        ensure!(
            native_entry_count(workspace)? == native_entries,
            "a refused child/repacked request entered native"
        );
        checkpoint("complete", None)?;
        println!("public configured {scope} Start+2Resume: exact public envelopes and original roles/source carrier; public completion held until native fixture release. PID absence is supporting observation. SDK native wait/stdio ACK and owned gateway/helper cleanup require separately attributed receipt/source-order evidence. World startup binding only; downstream member proof remains separate.");
        Ok(())
    }

    #[test]
    #[ignore = "requires exact candidate CLI/service/gateway in the reviewed isolated two-identity fixture"]
    fn test_public_configured_host_start_and_two_resumes() -> Result<()> {
        configured_public_helper_fixture(false, false, false, false)
    }

    #[test]
    #[ignore = "requires exact candidate CLI/service/gateway and private backend overlay in the reviewed fixture"]
    fn test_public_configured_world_start_and_two_resumes() -> Result<()> {
        configured_public_helper_fixture(true, false, false, false)
    }

    #[test]
    #[ignore = "owned normal-service public controller/Spawn/Fork/child-Resume composition fixture"]
    fn test_public_configured_world_spawn_fork_and_child_resume() -> Result<()> {
        configured_public_helper_fixture_with_world_work(true, false, false, false, true, false)
    }

    mod genuine_current_effects {
        use super::*;
        use crate::execution::agent_runtime::{
            dispatch_contract::*, native_launch_context::*, state_store::*,
        };
        use transport_api_types::{ExecuteStreamFrame, OrdinaryMemberTurnSubmitRequestV1};

        struct AcceptedTurn {
            selection: Arc<AuthenticatedOrdinarySelectionV1>,
            request: OrdinaryMemberTurnSubmitRequestV1,
            record: WorldWorkAcceptanceRecordV1,
        }

        fn accepted(
            verifier: &OrdinarySelectionAuthorityV1,
            workspace: &Path,
            name: &str,
        ) -> Result<AcceptedTurn> {
            let receipt: ActiveRetainedTurnReceiptV1 =
                serde_json::from_slice(&fs::read(workspace.join(name))?)?;
            let selection = Arc::new(verifier.authenticate(&receipt.orchestration_session_id)?);
            ensure!(selection.current_participant_id() == receipt.orchestrator_participant_id);
            let store = AgentRuntimeStateStore::new()?;
            ensure!(store.substrate_home() == Path::new("/home/fixture/.substrate"));
            let (request, _, record) = crate::execution::orchestrator_world_dispatch::reconstruct_ordinary_accepted_turn_for_test(
                &store, &selection, &receipt.request_id)?;
            ensure!(record.acceptance_record_id == receipt.acceptance_record_id);
            let AcceptedWorldWorkIdentityV1::RetainedTurn {
                active_run_id,
                message_id,
                target_participant_id,
            } = &record.work_identity
            else {
                anyhow::bail!("current-effects fixture needs an actual retained acceptance");
            };
            ensure!(
                active_run_id == &receipt.active_run_id
                    && message_id == &receipt.message_id
                    && target_participant_id == &receipt.target_participant_id
            );
            Ok(AcceptedTurn {
                selection,
                request,
                record,
            })
        }

        fn terminal(turn: &AcceptedTurn, exit: i32) -> Result<Option<Vec<serde_json::Value>>> {
            let observation = turn
                .selection
                .held_authority()
                .observe_existing_ordinary_world_work_v1(&turn.record)?;
            let Some(terminal) = &observation.terminal else {
                return Ok(None);
            };
            ensure!(
                terminal.exit_code == exit && observation.interruption.is_none(),
                "genuine current-effects terminal mismatch: {observation:?}"
            );
            let frames = observation
                .journal
                .iter()
                .map(|row| {
                    row.decode_for_c1(&observation.claim)
                        .and_then(|frame| Ok(serde_json::to_value(frame)?))
                })
                .collect::<Result<Vec<_>>>()?;
            let frame: ExecuteStreamFrame =
                serde_json::from_value(frames.last().context("missing terminal journal")?.clone())?;
            ensure!(matches!(frame, ExecuteStreamFrame::Exit {
                exit: actual, ref frame_identity, ref event_identity, ref terminal_identity, ref span_id, ..
            } if actual == exit && frame_identity == &terminal.frame_identity && event_identity == &terminal.event_identity
                && terminal_identity == &terminal.terminal_identity && span_id == &terminal.span_id));
            Ok(Some(frames))
        }

        fn root_replay(
            runtime: &tokio::runtime::Runtime,
            turn: &AcceptedTurn,
        ) -> Result<Vec<serde_json::Value>> {
            use http_body_util::BodyExt as _;
            runtime
                .block_on(async {
                    tokio::time::timeout(Duration::from_secs(45), async {
                        let response =
                            transport_api_client::AgentClient::unix_socket("/run/substrate.sock")?
                                .submit_ordinary_member_turn_stream(turn.request.clone())
                                .await?;
                        let bytes = response.into_body().collect().await?.to_bytes();
                        ensure!(bytes.ends_with(b"\n"), "Root replay lost NDJSON terminator");
                        std::str::from_utf8(&bytes)?
                            .lines()
                            .map(|line| {
                                let frame: ExecuteStreamFrame = serde_json::from_str(line)?;
                                let value: serde_json::Value = serde_json::from_str(line)?;
                                ensure!(
                                    serde_json::to_value(&frame)? == value,
                                    "unknown Root replay frame fields"
                                );
                                Ok(value)
                            })
                            .collect::<Result<Vec<_>>>()
                    })
                    .await
                })
                .context("Root replay method deadline")?
        }

        fn root_refusal(runtime: &tokio::runtime::Runtime, turn: &AcceptedTurn) -> Result<String> {
            let result = runtime
                .block_on(async {
                    tokio::time::timeout(Duration::from_secs(30), async {
                        transport_api_client::AgentClient::unix_socket("/run/substrate.sock")?
                            .submit_ordinary_member_turn_stream(turn.request.clone())
                            .await
                    })
                    .await
                })
                .context("Root refusal method deadline")?;
            let error = match result {
                Ok(_) => anyhow::bail!("Root accepted revoked original request"),
                Err(error) => error,
            };
            let message = error.to_string();
            // Current authority/account errors propagate before join and the
            // ordinary handler maps them to this explicit generic 500 response.
            // Keep its actual status; transport failures are not a refusal.
            let early_authority_refusal = message
                == "HTTP 500 Internal Server Error error: {\"error\":\"member_turn_internal_error\"}";
            ensure!(
                message.starts_with("HTTP 409 Conflict error: ") || early_authority_refusal,
                "revocation did not return a supported Root refusal response: {message}"
            );
            Ok(message)
        }

        fn gateway_cut(phase: &str, label: &str) -> Result<()> {
            checkpoint(phase, Some(label))?;
            proceed(label)
        }

        fn replay_without_delivery(
            runtime: &tokio::runtime::Runtime,
            turn: &AcceptedTurn,
            frames: &[serde_json::Value],
            workspace: &Path,
            label: &str,
        ) -> Result<()> {
            let native = fs::read(workspace.join("world-observations/world-native.jsonl"))?;
            gateway_cut("effects-zero-begin", label)?;
            let replay = root_replay(runtime, turn);
            gateway_cut("effects-zero-end", label)?;
            ensure!(
                replay? == frames,
                "Root replay differs from the exact B2 journal"
            );
            ensure!(
                fs::read(workspace.join("world-observations/world-native.jsonl"))? == native,
                "exact Root retry redelivered native input"
            );
            private_file(
                &workspace.join(format!("effects-{label}.json")),
                &serde_json::to_vec(frames)?,
            )?;
            Ok(())
        }

        pub(super) fn run(
            mut start: Command,
            workspace: &Path,
            held: Arc<config_projection::OrdinaryConfiguredAcceptedHomeAuthorityV1>,
        ) -> Result<()> {
            let verifier =
                OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(held)?;
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            let mut completed = None;
            let mut completed_frames = None;
            let mut active_turn = None;
            let mut restored = false;
            let mut finished = false;
            let mut last = Instant::now() - Duration::from_secs(1);
            let mut progress = || -> Result<()> {
                if finished || last.elapsed() < Duration::from_millis(500) {
                    return Ok(());
                }
                last = Instant::now();
                if completed.is_none() {
                    if !workspace
                        .join("current-effects-completed-receipt.json.ready")
                        .is_file()
                    {
                        return Ok(());
                    }
                    completed = Some(accepted(
                        &verifier,
                        workspace,
                        "current-effects-completed-receipt.json",
                    )?);
                }
                let turn = completed.as_ref().unwrap();
                if !restored {
                    let Some(frames) = terminal(turn, 0)? else {
                        return Ok(());
                    };
                    let native_path = workspace.join("world-observations/world-native.jsonl");
                    ensure!(fs::read_to_string(&native_path)?.lines().count() == 2);
                    let recipes = crate::execution::orchestrator_world_dispatch::ordinary_current_effects_parent_recipes_for_test(&turn.selection)?;
                    private_file(
                        &workspace.join("current-effects-parent-recipes.json"),
                        &serde_json::to_vec(&recipes)?,
                    )?;
                    replay_without_delivery(
                        &runtime,
                        turn,
                        &frames,
                        workspace,
                        "completed-replay",
                    )?;
                    for fault in ["account", "policy", "source"] {
                        let native = fs::read(&native_path)?;
                        gateway_cut("effects-zero-begin", fault)?;
                        let home = Path::new("/home/fixture/.substrate");
                        let path = match fault {
                            "account" => PathBuf::from("/home/fixture/.codex/auth.json"),
                            "policy" => home.join("policy.yaml"),
                            "source" => {
                                home.join("run/agent-hub/ordinary-start-contexts")
                                    .join(format!(
                                        "{}.json",
                                        turn.selection.original_start_intent_id()
                                    ))
                            }
                            _ => unreachable!(),
                        };
                        let original = fs::read(&path)?;
                        let metadata = fs::metadata(&path)?;
                        let held_path = path.with_extension("effects-held");
                        match fault {
                            "account" => {
                                let mut auth: serde_json::Value =
                                    serde_json::from_slice(&original)?;
                                auth["tokens"]["account_id"] = "synthetic-revoked-account".into();
                                fs::write(&path, serde_json::to_vec(&auth)?)?;
                            }
                            "policy" => {
                                let policy = String::from_utf8(original.clone())?;
                                ensure!(policy.contains(
                                    "allowed_backends: [cli:codex-host, cli:codex-world]"
                                ));
                                fs::write(
                                    &path,
                                    policy.replace(
                                        "allowed_backends: [cli:codex-host, cli:codex-world]",
                                        "allowed_backends: [cli:codex-host]",
                                    ),
                                )?;
                            }
                            "source" => fs::rename(&path, &held_path)?,
                            _ => unreachable!(),
                        }
                        let refusal = root_refusal(&runtime, turn);
                        // Restore even when an unexpected refusal/transport result is reported.
                        if fault == "source" {
                            fs::rename(&held_path, &path)?
                        } else {
                            fs::write(&path, &original)?
                        }
                        let after = fs::metadata(&path)?;
                        ensure!(
                            after.dev() == metadata.dev()
                                && after.ino() == metadata.ino()
                                && fs::read(&path)? == original,
                            "revocation fixture did not restore its original inode/bytes"
                        );
                        gateway_cut("effects-zero-end", fault)?;
                        ensure!(
                            fs::read(&native_path)? == native,
                            "revoked input reached native"
                        );
                        private_file(
                            &workspace.join(format!("effects-{fault}-refusal.json")),
                            &serde_json::to_vec(
                                &serde_json::json!({"root_refusal":refusal?,"same_original_inode_restored":true,"new_native_deliveries":0}),
                            )?,
                        )?;
                        replay_without_delivery(
                            &runtime,
                            turn,
                            &frames,
                            workspace,
                            &format!("{fault}-restored"),
                        )?;
                    }
                    completed_frames = Some(frames);
                    private_file(&workspace.join("current-effects-restored"), b"restored\n")?;
                    restored = true;
                    return Ok(());
                }
                if !workspace
                    .join("current-effects-active-receipt.json.ready")
                    .is_file()
                {
                    return Ok(());
                }
                let rows: Vec<serde_json::Value> =
                    fs::read_to_string(workspace.join("world-observations/world-native.jsonl"))?
                        .lines()
                        .map(serde_json::from_str)
                        .collect::<std::result::Result<_, _>>()?;
                if rows.len() < 3 {
                    return Ok(());
                }
                ensure!(
                    rows.len() == 3 && rows[2]["kind"] == "resume_active" && rows[2]["turns"] == 3
                );
                if active_turn.is_none() {
                    let active =
                        accepted(&verifier, workspace, "current-effects-active-receipt.json")?;
                    ensure!(
                        active.selection.current_participant_id()
                            == turn.selection.current_participant_id()
                            && active.record.caller_participant_id
                                == turn.record.caller_participant_id
                    );
                    ensure!(
                        terminal(&active, 130)?.is_none(),
                        "active native had already terminated"
                    );
                    // The owning helper must issue cancellation through its normal
                    // toolbox route. This separate observer cannot acquire its lease.
                    private_file(
                        &workspace.join("current-effects-cancel-ready"),
                        b"native owner may cancel its accepted turn\n",
                    )?;
                    active_turn = Some(active);
                    return Ok(());
                }
                if !workspace
                    .join("current-effects-cancel-receipt.json.ready")
                    .is_file()
                {
                    return Ok(());
                }
                let active = active_turn.as_ref().unwrap();
                let AcceptedWorldWorkIdentityV1::RetainedTurn {
                    active_run_id,
                    message_id,
                    target_participant_id,
                } = &active.record.work_identity
                else {
                    unreachable!()
                };
                let target = WorldDispatchControlTargetV1::AcceptedRetainedTurn {
                    acceptance_record_id: active.record.acceptance_record_id.clone(),
                    active_run_id: active_run_id.clone(),
                    message_id: message_id.clone(),
                    target_participant_id: target_participant_id.clone(),
                };
                let cancelled: crate::execution::agent_runtime::tool_invocation_contract::HostToolCancelWorldWorkOutcomeV1 =
                    serde_json::from_slice(&fs::read(workspace.join("current-effects-cancel-receipt.json"))?)?;
                private_file(
                    &workspace.join("effects-supported-cancellation-return.json"),
                    &serde_json::to_vec(&cancelled)?,
                )?;
                ensure!(
                    matches!(
                        cancelled.state,
                        CancelWorldWorkTerminalStateV1::CancelAcceptedPendingCloseout
                            | CancelWorldWorkTerminalStateV1::CancelledViaLiveTransport
                    ),
                    "unexpected supported native-owner cancel outcome: {cancelled:?}"
                );
                ensure!(
                    cancelled.exact_target.as_ref() == Some(&target)
                        && cancelled.cancel_request_id.is_some()
                        && cancelled.orchestration_session_id
                            == active.record.orchestration_session_id
                        && cancelled.orchestrator_participant_id
                            == active.record.caller_participant_id
                        && cancelled.target_backend_id == active.record.target_backend_id
                        && cancelled.world_id == active.record.world_id
                        && cancelled.world_generation == active.record.world_generation
                        && cancelled.action == WorldDispatchActionV1::CancelWorldWork
                        && cancelled.mode == WorldDispatchModeV1::Retained
                );
                let active_frames = root_replay(&runtime, active)?;
                let journal = terminal(active, 130)?
                    .context("cancelled native did not publish checked B2 terminal")?;
                ensure!(
                    active_frames == journal,
                    "Root cancellation terminal differs from exact B2 journal"
                );
                ensure!(
                    fs::read_to_string(workspace.join("world-observations/world-native.jsonl"))?
                        .lines()
                        .count()
                        == 3
                );
                private_file(
                    &workspace.join("effects-supported-cancellation.json"),
                    &serde_json::to_vec(
                        &serde_json::json!({"native":rows[2],"cancel":cancelled,"root_frames":active_frames,"b2_journal":journal,"signal_marker_claimed":false,"raw_sdk_wait_exported":false}),
                    )?,
                )?;
                replay_without_delivery(
                    &runtime,
                    turn,
                    completed_frames.as_ref().unwrap(),
                    workspace,
                    "post-cancel-completed-replay",
                )?;
                checkpoint("effects-backend-replace", None)?;
                proceed("effects-backend-replaced")?;
                let native = fs::read(workspace.join("world-observations/world-native.jsonl"))?;
                gateway_cut("effects-zero-begin", "backend")?;
                let refusal = root_refusal(&runtime, turn);
                gateway_cut("effects-zero-end", "backend")?;
                ensure!(
                    fs::read(workspace.join("world-observations/world-native.jsonl"))? == native
                );
                private_file(
                    &workspace.join("effects-backend-refusal.json"),
                    &serde_json::to_vec(
                        &serde_json::json!({"root_refusal":refusal?,"new_native_deliveries":0}),
                    )?,
                )?;
                private_file(&workspace.join("current-effects-finished"), b"finished\n")?;
                finished = true;
                Ok(())
            };
            let output = wait_public_command_with_budget(
                start.spawn()?,
                workspace,
                1,
                Duration::from_secs(300),
                Some(&mut progress),
            )?;
            ensure!(
                finished,
                "current-effects controller terminated before all required cases"
            );
            ensure!(
                output.status.success(),
                "current-effects public Start failed: status={}; stderr={}; stdout={}",
                output.status,
                String::from_utf8_lossy(&output.stderr),
                String::from_utf8_lossy(&output.stdout)
            );
            checked_public_completion(
                &output.stdout,
                PublicPromptAction::Start,
                "cli:codex-world",
                "world",
                None,
            )?;
            ensure!(
                fs::read_to_string(workspace.join("host-native.jsonl"))?
                    .lines()
                    .count()
                    == 1
            );
            Ok(())
        }

        #[test]
        #[ignore = "Root-only one-shot backend replacement within owned current-effects namespace"]
        fn replace_backend() -> Result<()> {
            use world_api::WorldBackend as _;
            ensure!(unsafe { libc::geteuid() } == 0 && !Path::new("/home/spenser").exists());
            let (parent, persistent, expected): (
                world_api::WorldSpec,
                world_api::WorldSpec,
                world_api::SharedWorldBindingSnapshot,
            ) = serde_json::from_slice(&fs::read(
                "/home/fixture/workspace/current-effects-parent-recipes.json",
            )?)?;
            let backend = world::LinuxLocalBackend::new();
            let mut recovered = None;
            for spec in [&parent, &persistent] {
                if let Some(handle) = backend.find_compatible_session(spec)? {
                    ensure!(
                        handle.shared_binding.as_ref() == Some(&expected)
                            && handle.id == expected.world_id
                    );
                    recovered = Some(handle);
                    break;
                }
            }
            ensure!(
                recovered.is_some(),
                "actual persisted backend producer could not be recovered"
            );
            let spec = backend.with_existing_custody_nonreconciling_v1(
                &expected.orchestration_session_id,
                &expected.world_id,
                expected.world_generation,
                &parent.project_dir,
                |held| {
                    let (spec, handle, path, cgroup) =
                        held.world_placement_for_specs_v1(&[parent.clone(), persistent.clone()])?;
                    ensure!(handle.shared_binding.as_ref() == Some(&expected));
                    ensure!(
                        fs::read(path.join("cgroup.procs"))?.is_empty(),
                        "old genuine World still has live processes"
                    );
                    ensure!(cgroup.metadata()?.ino() == fs::metadata(&path)?.ino());
                    Ok(spec)
                },
            )?;
            let mut replacement = spec;
            replacement.reuse_mode =
                world_api::WorldReuseMode::SharedOrchestration(world_api::SharedWorldOwnerSpec {
                    orchestration_session_id: expected.orchestration_session_id.clone(),
                    action: world_api::SharedWorldOwnerAction::ReplaceExpectedGeneration {
                        expected_generation: expected.world_generation,
                        reason: "owned genuine current-effects backend revocation proof".into(),
                    },
                });
            let actual = backend.ensure_session(&replacement)?;
            let binding = actual
                .shared_binding
                .as_ref()
                .context("replacement lost shared binding")?;
            ensure!(
                binding.world_id != expected.world_id
                    && binding.world_generation == expected.world_generation + 1
            );
            let readback = world::LinuxLocalBackend::new()
                .find_compatible_session(&replacement)?
                .context("fresh replacement readback missing")?;
            ensure!(readback.shared_binding == actual.shared_binding && readback.id == actual.id);
            println!(
                "GENUINE_BACKEND_REPLACEMENT {}",
                serde_json::to_string(
                    &serde_json::json!({"old":expected,"new":binding,"fresh_readback":readback.shared_binding,"old_cgroup_empty_before_replace":true})
                )?
            );
            Ok(())
        }
    }

    #[test]
    #[ignore = "owned genuine Root/service revocation, exact replay and supported running-turn cancellation proof"]
    fn test_public_configured_world_current_effects_and_exact_retry() -> Result<()> {
        configured_public_helper_fixture_with_world_work(true, false, false, false, true, true)
    }
    #[test]
    #[ignore = "requires exact candidate CLI/Claude SDK/service/gateway and synthetic API key in the reviewed fixture"]
    fn test_public_configured_claude_host_start_and_two_resumes() -> Result<()> {
        configured_public_helper_fixture(false, true, false, false)
    }

    #[test]
    #[ignore = "pending actual reviewed HSA helper comparison and exact isolated public fixture"]
    fn test_public_configured_host_issued_helper_refusals() -> Result<()> {
        configured_public_helper_fixture(false, false, true, false)
    }

    #[test]
    #[ignore = "pending actual reviewed HSA helper comparison and exact isolated public World fixture"]
    fn test_public_configured_world_issued_helper_refusals() -> Result<()> {
        configured_public_helper_fixture(true, false, true, false)
    }

    #[test]
    #[ignore = "requires the exact isolated public CLI/service/gateway and controlled native failure"]
    fn test_public_configured_host_start_failure_is_one_terminal_and_not_replayed() -> Result<()> {
        configured_public_helper_fixture(false, false, false, true)
    }

    const PUBLIC_CLAUDE_NATIVE_FIXTURE: &str = r##"#!/usr/bin/python3
import json, os, pathlib, sys, time, urllib.request, urllib.error
args = sys.argv[1:]
if args and args[0] in ('--version','--help'):
    print('claude 1.2.3' if args[0] == '--version' else 'Usage: --print --output-format --resume --model')
    raise SystemExit(0)
root = pathlib.Path(__file__).resolve().parent
def stage(name):
    fd = os.open(root / 'host-native-stages.jsonl',
        os.O_WRONLY | os.O_CREAT | os.O_APPEND | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, 'a') as output: output.write(json.dumps(dict(stage=name,pid=os.getpid()))+'\n')
stage('native_entered')
log = root / 'host-native.jsonl'
prior = [json.loads(s) for s in log.read_text().splitlines()] if log.exists() else []
turn = len(prior) + 1
session = 'thread-public-claude-helper'
assert turn in (1,2,3)
assert '--model' in args and args[args.index('--model')+1] == 'claude-sonnet-4.5'
assert ('--resume' in args) == (turn > 1)
if turn > 1: assert args[args.index('--resume')+1] == session
assert '--fork-session' not in args
assert args[-1].encode('utf-8') == (root / ('expected-prompt-%s.txt' % turn)).read_bytes()
home = pathlib.Path(os.environ['CLAUDE_CONFIG_DIR'])
assert str(home).startswith('/proc/')
assert os.environ['HOME'] == str(home)
for name in ('ANTHROPIC_API_KEY','OPENAI_API_KEY','SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN','SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID'):
    assert not os.environ.get(name)
cap = os.environ['ANTHROPIC_AUTH_TOKEN']
assert len(cap) == 43 and cap != 'substrate-managed-gateway'
origin = os.environ['ANTHROPIC_BASE_URL']
assert origin.startswith('http://127.0.0.1:')
def refused(token):
    headers = {'Content-Type':'application/json'}
    if token is not None: headers['Authorization'] = 'Bearer ' + token
    request = urllib.request.Request(origin.rstrip('/')+'/v1/messages', data=b'{', headers=headers)
    try:
        urllib.request.urlopen(request,timeout=5)
        raise AssertionError('malformed request accepted')
    except urllib.error.HTTPError as error: return error.code
deadline=time.monotonic()+5
while True:
    try: missing=refused(None); break
    except urllib.error.URLError:
        assert time.monotonic()<deadline
        time.sleep(0.02)
assert [missing,refused('wrong'),refused(cap)] == [403,403,400]
history = home / 'component-host-history'
expected = session + ''.join('\nturn-%s' % n for n in range(1,turn))
if turn == 1:
    with history.open('x') as output: output.write(session)
else: assert history.read_text() == expected
before=history.stat()
with history.open('a') as output: output.write('\nturn-%s' % turn)
after=history.stat()
assert (before.st_dev,before.st_ino)==(after.st_dev,after.st_ino)
entry=dict(turn=turn,pid=os.getpid(),parent_pid=os.getppid(),home_device=home.stat().st_dev,home_inode=home.stat().st_ino,
    history_device=after.st_dev,history_inode=after.st_ino,history=history.read_text(),authenticated_body_refusal=True)
with log.open('a') as output: output.write(json.dumps(entry)+'\n')
print(json.dumps(dict(type='system',subtype='init',session_id=session)),flush=True)
print(json.dumps(dict(type='result',subtype='success',is_error=False,session_id=session,
    result='synthetic Claude output %s'%turn,duration_ms=1,duration_api_ms=0,num_turns=1,total_cost_usd=0)),flush=True)
stage('events_emitted')
deadline=time.monotonic()+15
while not (root/('host-release-%s'%turn)).exists():
    assert time.monotonic()<deadline
    time.sleep(0.01)
"##;
}
