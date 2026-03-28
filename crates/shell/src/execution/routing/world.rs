//! World initialization flows for routing, including platform-gated defaults and agent bridging.

#[cfg(target_os = "macos")]
use crate::execution::policy_snapshot::bootstrap_world_spec;
#[cfg(all(test, any(target_os = "windows", target_os = "macos")))]
use crate::execution::world_env_guard;
use crate::execution::ShellConfig;

#[cfg(target_os = "linux")]
use super::dispatch::init_linux_world;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::execution::pw;

/// Check whether world support should be disabled based on resolved config.
pub(crate) fn world_disabled(config: &ShellConfig) -> bool {
    config.no_world
}

/// Initialize world support for the current platform when enabled.
pub(crate) fn initialize_world(config: &ShellConfig) {
    #[cfg(target_os = "windows")]
    init_windows_world(config);

    #[cfg(target_os = "macos")]
    init_macos_world(config);

    #[cfg(target_os = "linux")]
    init_linux_world_default(config);
}

#[cfg(target_os = "windows")]
fn init_windows_world(config: &ShellConfig) {
    if world_disabled(config) {
        return;
    }

    #[cfg(test)]
    let _env_guard = world_env_guard();

    // If the caller explicitly overrides the world socket, do not attempt platform detection
    // (which may provision/boot WSL). Downstream call-sites already honor the override.
    if std::env::var_os("SUBSTRATE_WORLD_SOCKET").is_some() {
        std::env::set_var("SUBSTRATE_WORLD", "enabled");
        return;
    }

    match pw::detect() {
        Ok(ctx) => {
            if (ctx.ensure_ready)().is_ok() {
                std::env::set_var("SUBSTRATE_WORLD", "enabled");
                if let Ok(handle) = ctx
                    .backend
                    .ensure_session(&pw::windows::bootstrap_world_spec())
                {
                    std::env::set_var("SUBSTRATE_WORLD_ID", handle.id);
                }
            }
            pw::store_context_globally(ctx);
        }
        Err(_e) => {}
    }
}

#[cfg(target_os = "macos")]
fn init_macos_world(config: &ShellConfig) {
    use substrate_broker::world_fs_mode;

    if world_disabled(config) {
        return;
    }

    #[cfg(test)]
    let _env_guard = world_env_guard();

    // If the caller explicitly overrides the world socket, do not attempt Lima detection/startup.
    // This keeps tests/fixtures hermetic and prevents accidental VM provisioning when users point
    // at a custom agent socket.
    if std::env::var_os("SUBSTRATE_WORLD_SOCKET").is_some() {
        std::env::set_var("SUBSTRATE_WORLD", "enabled");
        return;
    }

    match pw::detect() {
        Ok(ctx) => {
            if (ctx.ensure_ready)().is_ok() {
                // Set parity with Linux: world enabled + ID only
                std::env::set_var("SUBSTRATE_WORLD", "enabled");

                // Attempt to retrieve world id
                let spec =
                    bootstrap_world_spec(config.world_root.effective_root(), world_fs_mode());
                if let Ok(handle) = ctx.backend.ensure_session(&spec) {
                    std::env::set_var("SUBSTRATE_WORLD_ID", handle.id);
                }
            }
            pw::store_context_globally(ctx);
        }
        Err(_e) => {}
    }
}

#[cfg(target_os = "linux")]
fn init_linux_world_default(config: &ShellConfig) {
    let disabled = world_disabled(config);
    let _ = init_linux_world(disabled);
}
