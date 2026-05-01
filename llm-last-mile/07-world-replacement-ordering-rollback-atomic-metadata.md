# SOW: World Replacement Ordering, Rollback, and Atomic Metadata Writes

Status: implementation reference. This document records the exact backend hardening scope for Linux shared-world replacement ordering, rollback, and atomic metadata persistence so follow-on planning can rely on the actual contract that was required in code.

## Objective

Close the Linux shared-world replacement correctness gap so a shared-owner world restart:

- never leaves an orchestration session with zero valid active world because replacement creation failed mid-transition,
- persists owner-bearing `session.json` metadata atomically,
- rolls the old world back to `Active` when replacement creation fails after the old world has already entered a pre-commit transition state,
- cleans up partial replacement roots before returning failure, and
- preserves a single authoritative downstream proof state: `binding_state=Active`.

This scope is backend-first. It hardens the Linux shared-world authority layer in `crates/world` that later shell/runtime slices consume.

## Problem Statement

The explicit shared-world ownership contract was already landed at the world API and backend layer, but two correctness risks remained in the Linux backend:

1. replacement ordering could transition the old world out of `Active` before the replacement world was durably committed, creating a crash/failure window where no valid active world remained for the owner,
2. Linux `session.json` metadata persistence was weaker than the shell runtime store’s atomic-write posture, making owner-bearing metadata more exposed to partial-write corruption or torn replacement-state transitions.

This problem sits under the already-landed explicit shared-owner world model in:

- [crates/world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs)
- [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)
- [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/src/pty.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs)

It is not a shell-runtime projection problem. Projection of active binding into shell state belongs to later `llm-last-mile` slices.

## Current Repo Seams

### Shared-owner contract surface

- [crates/world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs)
  - `WorldReuseMode`
  - `SharedWorldOwnerSpec`
  - `SharedWorldOwnerAction`
  - `SharedWorldBindingSnapshot`
  - `SharedWorldBindingState`

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
  - `ExecuteRequest.shared_world`
  - `ExecuteResponse.shared_world`

### Linux backend replacement path

- [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)
  - `LinuxLocalBackend::ensure_shared_owner_session_from_root(...)`
  - `LinuxLocalBackend::replace_shared_owner_session_from_root(...)`
  - `LinuxLocalBackend::replace_shared_owner_session_from_root_with_creator(...)`
  - backend-local `shared_owner_mutex`

### Linux world metadata authority

- [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)
  - `SessionWorldMetadata`
  - `SessionWorld::to_metadata()`
  - `SessionWorld::persist_metadata()`
  - `SessionWorld::set_shared_binding_state(...)`
  - `SessionWorld::recover_shared_active_from_root(...)`

### Downstream proof consumers

- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
  - `build_world_spec(...)`
  - `resolve_shared_world_binding(...)`

- [crates/world-agent/src/pty.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs)
  - persistent-session ready-frame shared-world proof plumbing

- [crates/shell/src/execution/repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs)
  - fail-closed client proof validation

## In Scope

- Linux shared-owner replacement ordering in `crates/world`
- explicit pre-commit replacement state for the old world
- rollback to `Active` if replacement creation fails
- cleanup of partial replacement roots
- atomic `session.json` metadata persistence for Linux session worlds
- recovery rules for `Replacing` vs `Active` vs `Replaced`
- targeted backend tests for success, rollback, cleanup, and recovery

## Out of Scope

- shell runtime projection of `world_id` / `world_generation`
- participant invalidation or replacement-member creation
- toolbox/status selection logic
- non-Linux independent shared-world semantics
- broad trace/event schema work

## Required Data / State Contract

Linux session metadata for shared-owner worlds must remain owner-bearing and explicit:

- `owner_mode=shared_orchestration`
- `orchestration_session_id`
- `world_generation`
- `binding_state`
- `last_restart_reason`
- `policy_snapshot_hash`
- `world_fs_mode`

Only `binding_state=Active` is a valid downstream reusable proof state. `Replacing` is an internal transition state only.

## Required Replacement Ordering

`ReplaceExpectedGeneration` must follow this contract:

1. Resolve the currently active shared-owner world for the requested `orchestration_session_id`.
2. Verify the current world’s generation matches `expected_generation`.
3. Allocate a fresh replacement `world_id`.
4. Pre-commit the old world to `binding_state=Replacing` and persist that metadata atomically.
5. Create the replacement world with:
   - the same `orchestration_session_id`
   - `world_generation = expected_generation + 1`
   - `binding_state=Active`
6. If replacement creation fails:
   - roll the old world back to `binding_state=Active`,
   - persist rollback metadata atomically,
   - remove any partial replacement root,
   - return a single failure that includes create, rollback, and cleanup errors when present.
7. If replacement creation succeeds:
   - finalize the old world to `binding_state=Replaced`,
   - persist finalization metadata atomically,
   - return the new active world handle even if old-world finalization logs a warning.

This ordering keeps the only unsafe window on the side of “old world temporarily marked `Replacing` but still recoverable,” not “old world gone before replacement exists.”

## Failure / Rollback Semantics

### Create failure after pre-commit

If the backend has already moved the old world to `Replacing` and replacement creation fails:

- the old world must be restored to `Active`,
- recovery after the failure must still find the original world as the active reusable world,
- a partial replacement root must not remain reusable or recoverable,
- the returned error must preserve root cause detail.

### Finalize-old-world failure after replacement commit

If replacement creation succeeds but old-world finalization to `Replaced` fails:

- the new world remains the committed active world,
- the backend may log a warning,
- recovery must prefer the new active world,
- downstream consumers must still only observe the new `Active` binding as reusable.

### Recovery posture

Recovery from persisted metadata must satisfy:

- a lone `Replacing` world reconciles back to `Active`,
- an `Active` replacement outranks an older `Replacing` predecessor for the same owner,
- `Replaced` and `Abandoned` worlds are never reused,
- malformed partial owner metadata is ignored rather than adopted,
- ambiguous same-owner active states fail closed.

## Required Atomic Metadata Persistence

`SessionWorld::persist_metadata()` must use an atomic-write posture comparable to the shell runtime store:

1. serialize metadata to a temp file inside the destination world directory,
2. `sync_all()` the temp file,
3. rename temp file onto `session.json`,
4. on Unix, best-effort `sync_all()` the containing directory,
5. remove temp files on failure paths.

Target implementation seam:

- [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)

This must apply to:

- initial shared-owner world creation,
- pre-commit `Replacing`,
- rollback to `Active`,
- finalize to `Replaced`,
- generic reusable world metadata where persistence is enabled.

## Exact Code Areas To Touch

### Primary implementation

- [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)
  - `replace_shared_owner_session_from_root(...)`
  - `replace_shared_owner_session_from_root_with_creator(...)`
  - `cleanup_partial_shared_world_root(...)`
  - `ensure_shared_owner_session_from_root(...)`

- [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)
  - `persist_metadata()`
  - `set_shared_binding_state(...)`
  - `recover_shared_active_from_root(...)`
  - metadata read/write helpers

### Proof-surface validation context

- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/src/pty.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs)
- [crates/shell/src/execution/repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs)

These are not the main edit surface, but they are required context because downstream consumers must continue to accept only `binding_state=Active`.

## Testing Requirements

At minimum, land targeted tests for:

1. successful replacement commits a new `Active` world and finalizes the old world to `Replaced`
2. replacement creation failure rolls the old world back to `Active`
3. replacement creation failure cleans any partial replacement root
4. recovery prefers the committed new `Active` world over an older `Replacing` world
5. a lone `Replacing` world reconciles back to `Active`
6. malformed or partial owner metadata is rejected during recovery
7. atomic metadata persistence does not leave stray temp files after normal success

Primary test anchors:

- [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)
- [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)

Recommended verification commands:

```bash
cargo test -p world-api shared_world_contract_round_trips_with_canonical_shape
cargo test -p agent-api-types execute_response_shared_world_round_trip
cargo test -p world
cargo test -p world-agent resolve_shared_world_binding_rejects_mismatched_owner_proof
```

## Acceptance Criteria

- shared-owner replacement never leaves an owner with zero recoverable active world because replacement creation failed,
- replacement create failure restores the original world to `binding_state=Active`,
- partial replacement roots are cleaned on failure,
- `session.json` writes for Linux worlds are atomic,
- downstream proof consumers continue to observe only `binding_state=Active`,
- recovery semantics are deterministic and fail closed on ambiguity,
- targeted tests cover success, rollback, cleanup, and recovery.

## Relationship To Other `llm-last-mile` Work

This scope hardens the backend authority layer that later slices consume:

- it is upstream of runtime-state projection work in [04-thread-world-binding-into-runtime-state.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/04-thread-world-binding-into-runtime-state.md),
- upstream of member invalidation work in [05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md),
- and upstream of session-centric runtime-store work in [06-session-centric-state-store.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/06-session-centric-state-store.md).

It must not absorb participant replacement, shell registry mutation, or live status/toolbox UX changes.
