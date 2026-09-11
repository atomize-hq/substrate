pub mod builtins;
pub mod execution;
pub mod repl;
pub mod scripts;

pub use execution::{
    compare_and_swap_head_v1, complete_mac_lima_stage_one_transition_v1,
    deliver_retained_publisher_bootstrap_authorization_v1, derive_lifecycle_capsule_locator_v1,
    load_manifest_v1, needs_shell, open_publisher_bootstrap_channel_v1,
    open_trusted_lifecycle_capsule_v1, publish_action_receipt_index_v1, publish_manifest_v1,
    resume_action_receipt_commit_v1, run_shell, run_shell_with_cli,
    submit_post_pm_managed_action_v1, submit_stage_one_absent_instance_create_v1,
    transition_manifest_v1, update_shared_claims_v1, validate_mapped_lifecycle_control_request_v1,
    validate_publisher_response_v1, AnchorModeArg, Cli, GraphAction, GraphCmd, HealthCmd,
    HostAction, HostCmd, LifecyclePublisherClientV1, ManagedLifecycleControlRequestV1,
    MappedLifecycleTagV1, ShellConfig, ShellMode, ShimAction, ShimCmd, WorldAction,
    WorldCleanupArgs, WorldCmd, WorldDepsAction, WorldDepsCmd, WorldEnableArgs, WorldVerifyArgs,
};
// The two closed direct-control seed joins are re-exported only so the top-level macOS control
// binary does not acquire an independent transport-api dependency or expose a new carrier.
pub use execution::{lock, manager_init, shim_deploy};
pub use transport_api_types::{
    InstallBootstrapContextCarrierV1, PlatformBootstrapMappingV1, PlatformPrincipalV1,
};

#[cfg(target_os = "linux")]
pub use execution::agent_runtime::host_session_authority::facade::OpenedConfigProjectionHsaAuthorityV1;
