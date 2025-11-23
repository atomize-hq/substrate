pub mod agent_events;
mod cli;
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
pub mod shim_deploy; // Made public for integration tests

pub use cli::*;
pub use invocation::{needs_shell, ShellConfig, ShellMode};
pub use routing::*;

pub(crate) use manager::{
    configure_child_shell_env, configure_manager_init, current_platform, log_manager_init_event,
    manager_manifest_base_path, write_manager_env_script,
};
pub(crate) use platform::{handle_health_command, handle_world_command, update_world_env};
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) use platform_world as pw;
