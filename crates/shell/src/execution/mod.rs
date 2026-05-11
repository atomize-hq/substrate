#[doc(hidden)]
pub mod agent_dev_support;
pub mod agent_events;
pub(crate) mod agent_inventory;
pub(crate) mod agent_runtime;
mod agents_cmd;
mod auto_sync;
mod cli;
mod config_cmd;
pub(crate) mod config_model;
mod env_scripts;
mod home_bootstrap;
mod invocation;
pub mod lock;
mod manager;
pub mod manager_init;
mod platform;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod platform_world;
mod policy_cmd;
pub(crate) mod policy_model;
pub(crate) mod policy_snapshot;
mod pty;
pub mod repl_persistent_session;
mod routing;
mod settings;
pub(crate) use settings::resolve_world_root;
pub(crate) use settings::WorldRootSettings;
pub mod shim_deploy;
#[cfg(target_os = "linux")]
pub(crate) mod socket_activation; // Made public for integration tests
mod value_parse;
mod workspace;
mod workspace_cmd;

pub(crate) use agents_cmd::handle_agent_command;
pub(crate) use agents_cmd::handle_agents_command;
pub(crate) use auto_sync::run_auto_sync_if_enabled;
pub use cli::*;
pub(crate) use config_cmd::handle_config_command;
pub(crate) use env_scripts::{
    env_sh_path, export_runtime_config_env, write_env_sh, write_env_sh_at,
};
pub use invocation::{needs_shell, ShellConfig, ShellMode};
pub(crate) use policy_cmd::handle_policy_command;
#[cfg(unix)]
pub(crate) use pty::get_terminal_size;
pub(crate) use pty::MinimalTerminalGuard;
pub use routing::*;
pub(crate) use workspace::find_workspace_root;
pub(crate) use workspace_cmd::handle_workspace_command;

pub(crate) use manager::{
    configure_child_shell_env, configure_manager_init, current_platform, log_manager_init_event,
    manager_manifest_base_path, write_manager_env_script,
};
pub(crate) use platform::{
    handle_health_command, handle_host_command, handle_world_command, update_world_env,
};
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) use platform_world as pw;

#[cfg(test)]
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
#[cfg(test)]
use std::sync::OnceLock;

#[cfg(test)]
static WORLD_ENV_LOCK: OnceLock<ReentrantMutex<()>> = OnceLock::new();

#[cfg(test)]
pub(crate) fn world_env_guard() -> ReentrantMutexGuard<'static, ()> {
    WORLD_ENV_LOCK
        .get_or_init(|| ReentrantMutex::new(()))
        .lock()
}
