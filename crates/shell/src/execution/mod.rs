pub mod agent_events;
mod cli;
mod config_cmd;
pub(crate) mod config_model;
mod invocation;
pub mod lock;
mod manager;
pub mod manager_init;
mod platform;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod platform_world;
mod pty;
mod routing;
mod settings;
pub mod shim_deploy;
#[cfg(target_os = "linux")]
pub(crate) mod socket_activation; // Made public for integration tests
mod value_parse;
mod workspace;
mod workspace_cmd;

pub use cli::*;
pub(crate) use config_cmd::handle_config_command;
pub use invocation::{needs_shell, ShellConfig, ShellMode};
pub use routing::*;
pub(crate) use workspace_cmd::handle_workspace_command;

pub(crate) use manager::{
    configure_child_shell_env, configure_manager_init, current_platform, log_manager_init_event,
    manager_manifest_base_path, write_manager_env_script, write_manager_env_script_at,
};
pub(crate) use platform::{handle_health_command, handle_world_command, update_world_env};
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
