# Integration Map — World FS Granular Allow/Deny (V2) + Strict Deny

This document is a grounding index for ADR-0018 implementation. Line numbers are best-effort.

## Existing chokepoints (ground truth)
- Mount namespace wrapper + script:
  - `crates/world/src/exec.rs` (`PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT`)
- Helper entrypoint dispatch:
  - `crates/world-agent/src/main.rs` (argv[1] == `__substrate_world_landlock_exec`)
  - `crates/world-agent/src/internal_exec.rs` (`run_landlock_exec`)
- Host snapshot generation:
  - `crates/shell/src/execution/policy_snapshot.rs`
- REPL drift restart logic:
  - `crates/shell/src/repl/async_repl.rs` (`ensure_no_policy_drift`)
- Policy inputs resolution (snapshot ingestion + env injection):
  - `crates/world-agent/src/service.rs` (`resolve_policy_inputs`)
  - `crates/world-agent/src/pty.rs` (`prepare_persistent_world_context`)

## Planned touchpoints (V2)
- Broker schema update:
  - `crates/broker` policy structs and effective validation (reject legacy keys; validate patterns; enforce isolation constraints).
- Snapshot schema update:
  - `crates/agent-api-types/src/lib.rs` (add `PolicySnapshotV2`; update request types)
  - `crates/shell/src/execution/policy_snapshot.rs` (emit V2)
- World-agent snapshot consumption:
  - `crates/world-agent/src/service.rs` (accept V2 only; build helper env incl. enforcement plan)
  - `crates/world-agent/src/pty.rs` (accept V2 only for `start_session`)
- Helper enforcement:
  - `crates/world-agent/src/internal_exec.rs`:
    - apply deny mounts before Landlock and before `exec sh -c`
    - implement strict lockdown (cap drop + seccomp) before workload exec
- Landlock discover/read split:
  - `crates/world/src/landlock.rs` (support `READ_DIR` separate from `READ_FILE` as needed)

