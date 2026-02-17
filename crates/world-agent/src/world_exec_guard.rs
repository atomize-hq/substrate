//! Exec-time guardrails for host-mounted toolchain binaries in host-visible worlds.
//!
//! Spec: `docs/project_management/next/world-deps-host-visible-hardening/WDH2-spec.md`

pub(crate) use substrate_common::world_exec_guard::{check_command, deny_message};
