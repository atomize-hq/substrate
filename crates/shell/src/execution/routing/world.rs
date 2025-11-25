//! World initialization flows for routing, including platform-gated defaults and agent bridging.

use crate::execution::ShellConfig;
use std::env;

#[cfg(target_os = "linux")]
use super::dispatch::init_linux_world;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::execution::pw;

/// Check whether world support should be disabled based on environment or CLI flags.
pub(crate) fn world_disabled(config: &ShellConfig) -> bool {
    env::var("SUBSTRATE_WORLD")
        .map(|v| v == "disabled")
        .unwrap_or(false)
        || config.no_world
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

    match pw::detect() {
        Ok(ctx) => {
            if let Err(e) = (ctx.ensure_ready)() {
                eprintln!("substrate: windows world ensure_ready failed: {:#}", e);
            } else {
                env::set_var("SUBSTRATE_WORLD", "enabled");
                if let Ok(handle) = ctx.backend.ensure_session(&pw::windows::world_spec()) {
                    env::set_var("SUBSTRATE_WORLD_ID", handle.id);
                }
            }
            pw::store_context_globally(ctx);
        }
        Err(e) => {
            eprintln!("substrate: windows world detection failed: {}", e);
        }
    }
}

#[cfg(target_os = "macos")]
fn init_macos_world(config: &ShellConfig) {
    use substrate_broker::allowed_domains;
    use world_api::{ResourceLimits, WorldSpec};

    if world_disabled(config) {
        return;
    }

    match pw::detect() {
        Ok(ctx) => {
            if let Err(e) = (ctx.ensure_ready)() {
                // Degrade silently if ensure_ready fails
                eprintln!("substrate: mac world ensure_ready failed: {}", e);
            } else {
                // Set parity with Linux: world enabled + ID only
                env::set_var("SUBSTRATE_WORLD", "enabled");

                // Attempt to retrieve world id
                let spec = WorldSpec {
                    reuse_session: true,
                    isolate_network: true,
                    limits: ResourceLimits::default(),
                    enable_preload: false,
                    allowed_domains: allowed_domains(),
                    project_dir: config.world_root.effective_root(),
                    always_isolate: false,
                };
                if let Ok(handle) = ctx.backend.ensure_session(&spec) {
                    env::set_var("SUBSTRATE_WORLD_ID", handle.id);
                }
            }
            pw::store_context_globally(ctx);
        }
        Err(e) => {
            // Degrade silently on mac as well
            eprintln!("substrate: mac world detection failed: {}", e);
        }
    }
}

#[cfg(target_os = "linux")]
fn init_linux_world_default(config: &ShellConfig) {
    let disabled = world_disabled(config);
    let _ = init_linux_world(disabled);
}
