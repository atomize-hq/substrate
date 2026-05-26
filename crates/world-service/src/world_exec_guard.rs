//! Exec-time guardrails for host-mounted toolchain binaries in host-visible worlds.
//!
//! Reference: `docs/reference/config/world.md`

pub(crate) use substrate_common::world_exec_guard::{check_command, deny_message};
