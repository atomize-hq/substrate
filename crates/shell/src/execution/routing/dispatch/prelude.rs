//! Dispatch exports, keeping the module root thin.

pub(crate) use super::exec::execute_command;
#[allow(unused_imports)]
pub(crate) use super::registry::{
    container_wants_pty, git_wants_pty, has_top_level_shell_meta, is_force_pty_command,
    is_interactive_shell, is_pty_disabled, looks_like_repl, needs_pty, parse_demo_burst_command,
    peel_wrappers, sudo_wants_pty, wants_debugger_pty,
};
#[allow(unused_imports)]
pub(crate) use super::shim_ops::wrap_with_anchor_guard;
#[cfg(target_os = "linux")]
pub(crate) use super::world_ops::init_linux_world;
#[allow(unused_imports)]
pub(crate) use super::world_ops::{
    build_agent_client_and_request, consume_agent_stream_buffer, stream_non_pty_via_agent,
    AgentStreamOutcome,
};
#[cfg(all(test, target_os = "linux"))]
pub(crate) use super::world_ops::{init_linux_world_with_probe, LinuxWorldInit};
