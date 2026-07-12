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
#[cfg(any(target_os = "linux", test))]
pub(crate) mod host_inbox_materialization;
mod invocation;
pub mod lock;
mod manager;
pub mod manager_init;
pub(crate) mod orchestrator_world_dispatch;
mod platform;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod platform_world;
mod policy_cmd;
pub(crate) mod policy_model;
pub(crate) mod policy_snapshot;
pub(crate) mod prompt_fulfillment;
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

#[cfg(test)]
pub(crate) fn private_test_tempdir() -> tempfile::TempDir {
    #[cfg(target_os = "linux")]
    let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            // SAFETY: geteuid has no preconditions and does not mutate process state.
            std::path::PathBuf::from(format!("/run/user/{}", unsafe { libc::geteuid() }))
        });
    #[cfg(target_os = "macos")]
    let safe_parent =
        std::path::PathBuf::from(std::env::var_os("HOME").expect("macOS tests require HOME"))
            .join("Library/Caches");

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        std::fs::create_dir_all(&safe_parent).expect("create private test temp parent");
        let temp = tempfile::Builder::new()
            .prefix("st-")
            .tempdir_in(safe_parent)
            .expect("allocate private test tempdir");
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700))
            .expect("secure private test tempdir");
        temp
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    tempfile::Builder::new()
        .prefix("st-")
        .tempdir()
        .expect("allocate test tempdir")
}
