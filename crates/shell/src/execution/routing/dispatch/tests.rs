//! Dispatch routing tests covering PTY heuristics, registry helpers, telemetry, and world init flows.

use super::*;

#[path = "tests/support.rs"]
mod support;
pub(crate) use support::with_test_mode;

#[path = "tests/pty.rs"]
mod pty;
#[path = "tests/registry.rs"]
mod registry;
#[path = "tests/telemetry.rs"]
mod telemetry;
#[cfg(target_os = "linux")]
#[path = "tests/linux_world.rs"]
mod linux_world;
