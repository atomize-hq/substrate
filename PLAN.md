<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/substrate/feat-session-centric-state-store-autoplan-restore-20260501-100941.md -->
# PLAN: Linux Shared-World Replacement Ordering, Rollback, and Atomic Metadata Writes

Source brief: user SOW dated `2026-05-01`  
Branch: `feat/session-centric-state-store`  
Plan type: Linux backend hardening slice, no UI scope  
Review posture: fresh plan written against current repo truth with `/autoplan` completeness and `/plan-eng-review` depth  
Status: execution-ready once approved

## Objective

The bug is not "replacement sometimes fails." The bug is that Linux shared-world replacement currently commits the old world out of service before the new one is durably real.

Right now:

- [`LinuxLocalBackend::replace_shared_owner_session()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs:195) marks the old world `Replaced` before creating the replacement.
- [`SessionWorld::mark_shared_binding_replaced()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:176) is one-way and also leaves `last_restart_reason` behind as if the restart succeeded.
- [`SessionWorld::persist_metadata()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:477) uses raw `fs::write`, so `session.json` can tear or truncate on crash.
- Shared-owner recovery in [`recover_shared_active_from_root()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:207) only accepts `Active`, deletes malformed metadata, and does not reconcile a replace window.
- Downstream consumers in [`crates/world-agent/src/service.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:2328) and [`crates/shell/src/execution/repl_persistent_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs:308) correctly fail closed unless the echoed proof is `binding_state=Active`.

That combination can strand an orchestration session without a reusable world, silently reset generation, or destroy the only durable owner proof. The fix is a Linux-only two-phase replace transaction plus atomic metadata writes. Nothing more. Nothing fancier.

## Execution Summary

This slice ships in one narrow vertical path:

1. harden the shared-binding state machine and `session.json` durability in [`crates/world/src/session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:16),
2. rewrite Linux replacement ordering and same-owner serialization in [`crates/world/src/lib.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs:148),
3. prove the unchanged `Active`-only contract through targeted world-agent and shell regressions,
4. correct the stale authority docs so the code and docs describe the same seam.

If any step tries to widen scope beyond those four items, it is out of plan.

## Scope Lock

### Repo truth this plan must follow

This plan is locked to current repository behavior, not stale intent docs:

1. Linux shared-owner reuse is already authoritative in `crates/world` through `WorldReuseMode::SharedOrchestration`, `SharedWorldBindingSnapshot`, and `session.json` owner metadata. There is no missing schema problem in [`crates/world-api/src/lib.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:9).
2. The replace bug is ordering, not model shape. The current bug is at [`crates/world/src/lib.rs:195`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs:195).
3. The persistence bug is write durability, not serialization shape. The current bug is at [`crates/world/src/session.rs:477`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:477).
4. `binding_state=Active` is already the only acceptable proof state on both world-agent and shell validation paths. The plan must preserve that contract exactly.
5. [`llm-last-mile/03-shared-world-ownership-linux-first.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/03-shared-world-ownership-linux-first.md:112) is stale where it still proposes a shell-authoritative binding store. Current repo truth is Linux metadata authority in `crates/world`.
6. This slice is Linux only. macOS and Windows must keep compiling with additive compatibility only. No behavior redesign there.

### What already exists

| Sub-problem | Existing code | Plan |
| --- | --- | --- |
| Shared owner request model | [crates/world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:9) | Reuse as-is |
| Shared owner allocation entrypoint | [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs:172) | Refactor ordering only |
| Shared binding metadata model | [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:16) | Reuse schema, improve durability and reconciliation |
| Active-only proof validation | [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:2328), [crates/shell/src/execution/repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs:308) | Reuse unchanged |
| Atomic JSON write precedent | [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:912) | Copy the pattern into `crates/world` |
| Shared-world shell stub harness | [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs:393) | Extend for replace rollback cases |

### NOT in scope

- any new wire schema in `crates/world-api` or `crates/agent-api-types`
- shell-side authoritative shared-world binding files
- runtime manifest redesign or status-store redesign
- platform behavior changes for macOS or Windows
- new cache/index files under `/tmp/substrate-worlds`
- optimistic cleanup daemons or background reconciliation services
- rollout of a generic reusable atomic write utility crate

### Chosen approach

| Approach | Summary | Effort | Risk | Decision |
| --- | --- | --- | --- | --- |
| A. Two-phase replace in `crates/world` + atomic `session.json` writes | Keep old world committed until new `Active` world is durable, then finalize old | Medium | Low | **Accepted** |
| B. Mark old `Replaced` first and try to recreate on failure | Current behavior | Small | High | Rejected |
| C. Add shell-owned binding store and reconcile to Linux metadata | Second authority, more drift, wrong seam | High | High | Rejected |
| D. Introduce per-owner persistent lock files | More moving parts than needed for the first hardening fix | Medium | Medium | Rejected |

The accepted path is the smallest correct slice. It fixes the real failure window without spending an innovation token on a new authority model.

## Step 0: Scope Challenge

### Minimum change set

The minimum complete fix is:

1. add an internal shared-binding state transition helper in `crates/world/src/session.rs`,
2. make replacement in `crates/world/src/lib.rs` explicit: pre-commit, commit, rollback, finalize,
3. make `session.json` writes atomic in `crates/world/src/session.rs`,
4. teach shared-owner recovery to reconcile `Active` plus `Replacing` worlds deterministically,
5. serialize shared-owner `ensure_session()` paths so concurrent `AttachOrCreate` cannot observe a half-transition,
6. add targeted tests in `world`, `world-agent`, and `shell`,
7. update the stale plan/doc surfaces that still describe the wrong authority seam.

### Complexity check

This slice is a little wider than the ideal 5-file bug fix, but still boring:

- production files: `crates/world/src/lib.rs`, `crates/world/src/session.rs`
- validation-only or docs: `crates/world-agent/src/service.rs`, `crates/world-agent/src/pty.rs`, `crates/shell/src/execution/repl_persistent_session.rs`, related tests, `docs/WORLD.md`, `llm-last-mile/03-shared-world-ownership-linux-first.md`

No new service. No new storage authority. No new public model. That is the whole game.

### File touch budget

Expected production edits are intentionally constrained:

- required production files: [`crates/world/src/session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:16), [`crates/world/src/lib.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs:148)
- validation seams that may change only if a regression test proves they must: [`crates/world-agent/src/service.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:2328), [`crates/world-agent/src/pty.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs:211), [`crates/shell/src/execution/repl_persistent_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs:308)
- required tests and docs: targeted files listed later in this plan

No new crate, no new public API surface, no new persisted schema, no new background worker.

### Completeness check

Shortcut versions of this fix are not acceptable:

- "just mark `Replacing` instead of `Replaced`" is incomplete without recovery logic,
- "just add retries" is incomplete without rollback semantics,
- "just stop deleting malformed metadata" is incomplete without atomic writes,
- "just add tests" is incomplete if the transaction ordering stays wrong.

This plan chooses the complete version because the extra cost is minutes, not weeks.

### Distribution check

No new artifact type is introduced. No CI/CD changes are required beyond normal test coverage.

## Architecture Contract

### Hard invariants

1. At most one committed `Active` world exists per `orchestration_session_id` after recovery.
2. `binding_state=Active` remains the only proof state surfaced to world-agent or shell callers.
3. `world_generation` increments exactly once, only at the commit point when the new world becomes durable.
4. A failed replace before commit preserves the original `world_id`, `world_generation`, and `binding_state=Active`.
5. Cleanup failure for a partially created replacement must never block rollback of the old world.
6. Shared-owner recovery must never silently reset generation to `0` when a prior owner chain exists.
7. Malformed owner-bearing metadata must be warned on and treated as non-reusable or ambiguous. It must not be silently deleted during shared-owner recovery.
8. Generic compatible reuse remains unchanged for non-owner flows.
9. Same-owner allocation and replacement must be serialized inside the Linux backend so concurrent requests cannot create duplicate generation `0` worlds.

### Architecture dependency graph

```text
SHARED-OWNER REPLACE DEPENDENCY GRAPH
=====================================
LinuxLocalBackend::ensure_session()
    |
    +--> shared-owner action gate
          |
          +--> AttachOrCreate
          |     |
          |     +--> recover_shared_active_from_root()
          |     \--> create_shared_owner_session()
          |
          \--> ReplaceExpectedGeneration
                |
                \--> replace_shared_owner_session()
                      |
                      +--> SessionWorld::set_shared_binding_state(Active -> Replacing)
                      +--> SessionWorld::persist_metadata() [atomic]
                      +--> create_shared_owner_session(generation + 1)
                      +--> SessionWorld::set_shared_binding_state(Replacing -> Replaced)
                      \--> rollback on failure: SessionWorld::set_shared_binding_state(Replacing -> Active)
```

All state transitions funnel through `SessionWorld`. All request serialization lives in `LinuxLocalBackend`. Downstream proof readers stay read-only and fail closed.

### Current unsafe flow

```text
CURRENT REPLACE FLOW
====================
old Active(g=N)
    |
    | mark old Replaced + persist
    v
old Replaced(g=N)
    |
    | create replacement world
    +--> success -> new Active(g=N+1)
    |
    +--> failure -> NO ACTIVE WORLD
```

That "failure -> NO ACTIVE WORLD" branch is the bug.

### Target transaction flow

```text
TARGET REPLACE FLOW
===================
old Active(g=N)
    |
    | pre-commit: set old Replacing(g=N), atomic persist
    v
old Replacing(g=N)
    |
    | create replacement root + persist new Active(g=N+1)
    +--> failure before commit
    |       |
    |       | rollback old to Active(g=N), then attempt cleanup of the new root
    |       v
    |   old Active(g=N)
    |
    +--> success at commit point
            |
            | finalize old to Replaced(g=N), atomic persist
            v
        new Active(g=N+1) + old Replaced(g=N)
```

### Recovery reconciliation contract

```text
RECOVERY DECISION TREE
======================
scan owner-matching session.json files
    |
    +--> exactly 1 Active, 0 Replacing
    |       -> return Active
    |
    +--> 0 Active, 1 Replacing
    |       -> restore Replacing -> Active, persist, return it
    |
    +--> 1 Active(g=N+1), 1 Replacing(g=N)
    |       -> return newer Active
    |       -> do not reuse older Replacing
    |
    +--> 2+ Active
    |       -> fail closed, do not guess
    |
    +--> malformed owner-bearing metadata
            -> warn, ignore for reuse, do not delete automatically
```

### Serialization strategy

Use one coarse backend-local mutex around shared-owner `ensure_session()` paths in [`LinuxLocalBackend::ensure_session()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs:329).

Why this choice:

- it is explicit,
- it is a minimal diff,
- it avoids duplicate generation `0` creation during concurrent `AttachOrCreate`,
- it keeps generic execution paths untouched,
- it is easy to replace later with a narrower per-owner lock if profiling ever justifies it.

Do not build a lock-file protocol. Not for this fix.

## Detailed Execution Plan

### 1. `crates/world/src/session.rs`

#### 1.1 Replace one-way mutation with a single internal transition helper

Replace [`mark_shared_binding_replaced()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:176) with this helper:

```rust
fn set_shared_binding_state(
    &mut self,
    binding_state: SharedWorldBindingState,
    last_restart_reason: Option<String>,
) -> Result<()>
```

Rules:

- allow transitions `Active -> Replacing`, `Replacing -> Active`, `Replacing -> Replaced`
- reject every other transition with an error that names the current and requested state
- keep `world_id` and `world_generation` unchanged during pre-commit and rollback
- set `last_restart_reason=Some(reason)` only when entering `Replacing` or finalizing `Replaced` after a committed replacement
- clear `last_restart_reason` on rollback to `Active`
- persist via the new atomic metadata writer every time
- no other function in `session.rs` may write `binding_state` or `last_restart_reason` directly for shared-owner worlds

Done when:

- `mark_shared_binding_replaced()` is gone,
- the helper is the only shared-binding state mutation path,
- tests cover all three allowed transitions and at least one rejected transition.

#### 1.2 Add an atomic metadata writer

Replace raw [`fs::write()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:484) with the same-directory temp-file pattern already used by [`write_atomic_json()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:912):

1. `create_dir_all(parent)`
2. `NamedTempFile::new_in(parent)`
3. `serde_json::to_writer_pretty(temp)`
4. `temp.sync_all()`
5. `persist()` into `session.json`
6. on Linux, attempt a parent-directory `sync_all()` after rename; warn on unsupported directory sync, but do not undo a successful rename

Durability contract on successful return:

- readers see old full JSON or new full JSON,
- never torn bytes,
- parent directory metadata is flushed when supported.

Done when:

- `persist_metadata()` still owns the public write contract,
- the implementation no longer calls `fs::write()` for `session.json`,
- a forced write failure test proves the prior readable metadata remains intact.

#### 1.3 Reconcile shared-owner recovery

Refactor [`recover_shared_active_from_root()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:207) so it:

- collects all owner-matching `Shared` candidates,
- distinguishes `Active`, `Replacing`, `Replaced`, `Abandoned`,
- restores a lone `Replacing` world to `Active`,
- prefers the newer committed `Active` when both `Active` and older `Replacing` exist,
- fails closed on ambiguous multi-`Active` cases,
- never auto-deletes malformed owner-bearing metadata during shared-owner scan.

Generic recovery may keep its current cleanup behavior for purely generic metadata. Shared-owner recovery cannot.

Deterministic resolution order:

1. parse all owner-matching shared candidates,
2. separate malformed owner-bearing metadata from valid candidates,
3. partition valid candidates by `binding_state`,
4. resolve exactly one of the cases in the recovery decision tree,
5. if no valid reusable candidate exists, return `Ok(None)`,
6. if the valid set is ambiguous, return an error and leave files on disk for inspection.

Done when:

- shared-owner recovery never deletes malformed owner-bearing metadata,
- ambiguous multi-`Active` ownership fails closed,
- lone `Replacing` rollback recovery and `Active + Replacing` reconciliation both have regression tests.

### 2. `crates/world/src/lib.rs`

#### 2.1 Rewrite replace as an explicit transaction

Refactor [`replace_shared_owner_session()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs:195) into four phases:

1. **Load and validate current active world**
   - find authoritative active proof
   - reject generation mismatch
2. **Pre-commit**
   - set old world `Active -> Replacing`
   - atomically persist old metadata
3. **Commit**
   - create replacement world with `world_generation = expected_generation + 1`
   - replacement world persists immediately as `Active`
4. **Finalize**
   - set old world `Replacing -> Replaced`
   - atomically persist old metadata

Rollback path:

- if replacement creation fails before the commit point, restore old world `Replacing -> Active`,
- clear stale restart reason on rollback,
- attempt cleanup of any partially created replacement root after the old world is back to `Active`,
- return an error that preserves the original creation failure and also mentions rollback or cleanup failure if either secondary step fails.

Additional implementation rules:

- do not call `create_shared_owner_session()` until pre-commit metadata persistence succeeds,
- do not finalize the old world until the new world is created, persisted, and inserted into the backend cache,
- `world_generation` increments only on the new committed world,
- the old world keeps generation `N` in both `Replacing` and `Replaced`,
- the rollback path must execute in the same request before returning an error.

Done when:

- a successful replace leaves exactly one committed `Active` world at generation `N+1`,
- a failed replace leaves the original world committed as `Active` at generation `N`,
- no code path returns with zero recoverable world for the owner.

#### 2.2 Serialize same-owner shared-world ensure paths

Guard the shared-owner branch in [`ensure_session()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs:329) with a backend-local mutex. Apply it to both:

- `AttachOrCreate`
- `ReplaceExpectedGeneration`

That closes the window where:

1. request A marks old `Replacing`,
2. request B scans and sees no `Active`,
3. request B creates a second generation `0` world.

That race is unacceptable.

Implementation requirement:

- use one backend-local mutex guarding only the shared-owner branch,
- hold it across lookup, pre-commit, create, rollback, and finalize,
- do not widen the lock to generic reuse flows.

Done when:

- concurrent same-owner attach and replace tests cannot create duplicate generation `0` worlds,
- generic non-owner flows still execute without taking the shared-owner lock.

### 3. Downstream proof seams

Production behavior stays unchanged in:

- [`resolve_shared_world_binding()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:2328)
- [`PersistentServerMessage::Ready.shared_world`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs:211)
- [`validate_shared_world_echo()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs:308)

This plan deliberately preserves their fail-closed contract:

- world-agent must only surface committed `Active` proofs,
- shell must reject missing, mismatched, stale-generation, or non-`Active` echoes.

The work here is test coverage, not production redesign.

### 4. Docs and drift correction

Update:

- [`llm-last-mile/PLAN-03.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-03.md) only where it still describes replace ordering that no longer matches implementation
- [`llm-last-mile/03-shared-world-ownership-linux-first.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/03-shared-world-ownership-linux-first.md) to mark the shell-authoritative binding-store proposal as stale
- [`docs/WORLD.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md) to document the two-phase replace window and recovery guarantees for Linux metadata authority

Doc update contract:

- `PLAN-03` must match the final replacement ordering,
- `03-shared-world-ownership-linux-first.md` must explicitly say Linux metadata is authoritative now and the shell-side binding-store idea is historical only,
- `docs/WORLD.md` must document the `Replacing` crash window and the recovery rules from this plan.

## Implementation Sequence

Execute the work in this order. No later step starts before the prior step satisfies its done-when checks.

1. **State machine and persistence core**
   - implement `set_shared_binding_state()`
   - switch `persist_metadata()` to atomic writes
   - make shared-owner recovery deterministic
2. **Backend transaction and serialization**
   - rewrite `replace_shared_owner_session()`
   - add the shared-owner mutex in `ensure_session()`
3. **Proof seam regressions**
   - extend `world`, `world-agent`, and `shell` coverage to prove only committed `Active` proof escapes
4. **Docs and drift correction**
   - update `PLAN-03`, `03-shared-world-ownership-linux-first.md`, and `docs/WORLD.md`
5. **Validation**
   - run the exact command set in `Validation commands`

The only allowed parallelism is the one described later in `Worktree Parallelization Strategy`.

## State Machine

```text
SHARED BINDING STATES
=====================
Active
  |
  | replace requested, pre-commit persisted
  v
Replacing
  | \
  |  \ rollback before commit
  |   \
  |    -> Active
  |
  | new Active(g+1) durably committed
  v
Replaced

Abandoned
  reserved, not required for this slice
```

Rules:

- `AttachOrCreate` reuses only `Active`
- `ReplaceExpectedGeneration` starts only from `Active`
- recovery may promote a lone `Replacing` back to `Active`
- callers never receive `Replacing`, `Replaced`, or `Abandoned`

## Failure Modes Registry

| Failure mode | Where it happens | Planned handling | Test required | User-visible risk if unhandled |
| --- | --- | --- | --- | --- |
| replacement create fails before commit | `create_shared_owner_session()` | rollback old to `Active`, preserve generation and world_id | yes | owner loses reusable world |
| crash after old marked `Replacing`, before new commit | `replace_shared_owner_session()` window | recovery restores lone `Replacing` -> `Active` | yes | restart resets generation or strands owner |
| crash after new commit, before old finalize | finalize window | recovery prefers newer `Active`, ignores/finalizes older `Replacing` | yes | duplicate active interpretation |
| torn `session.json` write | `persist_metadata()` | atomic temp-file write + rename + sync | yes | metadata unreadable, proof lost |
| malformed owner-bearing metadata | shared-owner recovery scan | warn, ignore for reuse, do not delete silently | yes | durable proof erased |
| concurrent `AttachOrCreate` races | `ensure_session()` | serialize shared-owner path with mutex | yes | duplicate generation `0` worlds |
| stale echoed proof from server | PTY/non-PTY response validation | existing fail-closed validators reject it | already covered, extend | shell adopts wrong world |

Any row that would otherwise produce "no test + no error handling + silent failure" is a critical gap. This plan closes all of them.

## Test Review

### Code path coverage

```text
CODE PATH COVERAGE
==================
[+] crates/world/src/lib.rs
    |
    ├── replace_shared_owner_session()
    │   ├── [GAP] successful replace: old Active -> Replacing, new Active(g+1), old Replaced
    │   ├── [GAP] generation mismatch rejected before mutation
    │   ├── [GAP] create failure before commit rolls old back to Active
    │   └── [GAP] same-owner concurrent ensure path serializes correctly
    |
    └── ensure_session() shared-owner branch
        ├── [EXISTS] AttachOrCreate request plumbing
        └── [GAP] serialized replace/attach race regression

[+] crates/world/src/session.rs
    |
    ├── set_shared_binding_state()
    │   ├── [GAP] Active -> Replacing persists atomically
    │   ├── [GAP] Replacing -> Active clears restart reason on rollback
    │   └── [GAP] Replacing -> Replaced preserves generation/world_id
    |
    ├── persist_metadata()
    │   ├── [GAP] atomic write keeps prior file on failure
    │   └── [GAP] round-trip still succeeds for shared metadata
    |
    └── recover_shared_active_from_root()
        ├── [GAP] lone Replacing restored to Active
        ├── [GAP] Active(g=N+1)+Replacing(g=N) reconciles to newer Active
        ├── [GAP] malformed owner-bearing metadata is ignored, not deleted
        └── [EXISTS] ownerless legacy metadata rejected for shared-owner reuse

[+] world-agent / shell proof seams
    |
    ├── [GAP] non-PTY replace response never exposes non-Active proof
    ├── [EXISTS] PTY missing shared_world proof fails closed
    ├── [EXISTS] stale generation echo fails closed
    └── [GAP] stubbed restart rollback path still only echoes committed Active proof

─────────────────────────────────
COVERAGE: existing baseline is good on proof validation, weak on transaction windows
PRIORITY: replacement ordering + recovery + atomic persistence
─────────────────────────────────
```

### Required test additions

#### `crates/world/src/lib.rs`

Add or extend backend tests for:

1. successful replace commits new `Active(g+1)` and finalizes old `Replaced(g)`
2. generation conflict rejects before any metadata mutation
3. replacement create failure rolls old world back to original `Active`
4. concurrent same-owner `AttachOrCreate` or replace paths do not create duplicate generation `0` worlds

These tests are the regression floor. The implementation is not done if only the happy path passes.

#### `crates/world/src/session.rs`

Extend shared metadata tests for:

1. lone `Replacing` recovery promotes back to `Active`
2. `Active + Replacing` reconciliation selects the committed newer `Active`
3. malformed owner-bearing metadata is warned on and retained, not deleted
4. atomic write failure preserves prior readable metadata file
5. rollback clears `last_restart_reason`

#### `crates/world-agent/src/service.rs`

Extend unit coverage around [`resolve_shared_world_binding()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:2328) so replace flows still surface only committed `Active` proof snapshots.

#### `crates/shell/src/execution/routing/dispatch/tests/repl_persistent_session_client_fail_closed.rs`

Keep current fail-closed cases and add coverage that replace responses remain invalid unless the echoed proof is:

- `binding_state=Active`
- `world_generation > expected_generation`

#### `crates/shell/tests/repl_world_first_routing_v1.rs` and stub harness

Extend the stub in [`crates/shell/tests/support/repl_world_agent.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs:393) so an end-to-end rollback case can prove:

- replace failure does not strand the orchestration session,
- subsequent attach or retry still observes a valid committed `Active` proof.

### Validation evidence required in the implementation PR

The implementation PR description must include:

1. the exact `cargo` commands run from the list below,
2. a short note confirming whether any proof-seam production files changed,
3. the observed result for the rollback regression, the lone-`Replacing` recovery regression, and the concurrent same-owner race regression.

### Validation commands

Run at minimum:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p world -- --nocapture
cargo test -p world-agent -- --nocapture
cargo test -p shell repl_persistent_session_client_fail_closed -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

## Operator and DX Notes

This repo is a developer tool, so DX still matters even with no dedicated UI review:

- error text for shared-owner ambiguity must stay explicit and owner-scoped,
- recovery must prefer "preserve prior usable world" over "fail with no state",
- docs must explain why only committed `Active` proofs are surfaced,
- stale-doc cleanup is part of the fix because bad authority docs cause real future bugs.

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| A. state machine and atomic persistence | `crates/world/src/` | — |
| B. backend transaction and serialization | `crates/world/src/` | A |
| C. proof seam regression tests | `crates/world-agent/src/`, `crates/shell/src/execution/`, `crates/shell/tests/` | B |
| D. docs drift correction | `docs/`, `llm-last-mile/` | B |

### Parallel lanes

- Lane A: Step A -> Step B. Sequential, both touch `crates/world/src/`.
- Lane B: Step C after Step B. Independent from docs once the core proof contract is frozen.
- Lane C: Step D after Step B. Independent from tests once the state machine wording is final.

### Execution order

1. Launch Lane A first and finish both core steps.
2. After Lane A lands locally, launch Lane B and Lane C in parallel worktrees.
3. Merge Lane B and Lane C back into the main worktree.
4. Run the full validation stack in the merged tree.

### Conflict flags

- `crates/world/src/lib.rs` and `crates/world/src/session.rs` are a single-lane choke point.
- no parallel work starts before the `crates/world/src/` contract is frozen,
- proof tests and docs can run in parallel only because they touch different module directories.

## Deferred and TODO Disposition

No new `TODOS.md` file is created in this slice.

Reason:

- every real deferral is already captured in `NOT in scope`,
- none of those deferred items blocks the hardening fix,
- adding vague TODOs here would preserve less context than the explicit deferred list already does.

If follow-on work is needed later, the candidates are:

1. extract a shared atomic JSON writer once both `state_store` and `crates/world` prove stable on the pattern
2. narrow the coarse shared-owner mutex to a per-owner lock only if real contention appears
3. add operator-facing diagnostics for ignored malformed owner-bearing metadata if support needs that visibility

## Acceptance Criteria

This slice is complete only when all of these are true:

1. replacement create failure never leaves the owner with zero recoverable world
2. a pre-commit failure preserves old `world_id`, `world_generation`, and `binding_state=Active`
3. a successful replace returns only the new committed `Active` proof with `world_generation = expected_generation + 1`
4. `session.json` writes are atomic and never expose torn bytes on successful return
5. recovery from every replace crash window is deterministic and generation-safe
6. world-agent and shell proof validators still reject non-`Active` proof states
7. concurrent same-owner attach or replace requests cannot create duplicate generation `0` worlds
8. malformed owner-bearing metadata is never silently deleted by shared-owner recovery
9. stale authority docs are corrected so future work does not reintroduce the wrong seam

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | CEO | Treat the supplied SOW as the source-of-truth design input | mechanical | bias toward action | The SOW is already more concrete than a missing branch design doc | blocking on `/office-hours` |
| 2 | CEO | Keep Linux `session.json` as the only authority seam for this fix | mechanical | DRY | A shell-side binding store would create dual truth | shell binding file authority |
| 3 | Eng | Use a coarse shared-owner mutex instead of a new per-owner lock system | taste, resolved | explicit over clever | Minimal diff, low risk, closes the race now | lock files, distributed owner registry |
| 4 | Eng | Preserve world-agent and shell proof contracts unchanged | mechanical | systems over heroes | Active-only fail-closed validation is already correct | widening proof acceptance |
| 5 | Eng | Require full rollback and recovery coverage, not partial ordering fixes | mechanical | boil the lake | Cheap enough to do right now, expensive to debug later | mark-`Replacing` only |

## Completion Summary

- Step 0: Scope Challenge, scope accepted as-is with stale-authority correction
- Architecture Review: 4 issues found, all resolved in-plan
- Code Quality Review: 3 issues found, all resolved in-plan
- Test Review: diagram produced, 11 concrete gaps identified
- Performance Review: 1 issue found, resolved via shared-owner serialization
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 items proposed, explicit no-TODO disposition recorded
- Failure modes: 0 unresolved critical gaps after planned fixes
- Outside voice: skipped, Claude CLI auth unavailable
- Parallelization: 3 lanes total, 1 core sequential lane then 2 late parallel lanes
- Lake Score: 5/5 decisions chose the complete option

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
| --- | --- | --- | --- | --- | --- |
| CEO Review | `/autoplan` | Scope and authority seam | 1 | CLEAR | accepted Linux metadata authority, rejected stale shell-binding-store design |
| Codex Review | `n/a` | Independent 2nd opinion | 0 | — | outside voice unavailable, Claude CLI has no auth configured |
| Eng Review | `/plan-eng-review` | Architecture and tests | 1 | CLEAR | transaction ordering, recovery windows, and atomic writes fully specified |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | skipped | no UI scope |

**CROSS-MODEL:** not run, outside voice degraded because local Claude auth is missing.  
**UNRESOLVED:** 0  
**VERDICT:** CEO + ENG CLEARED, ready to implement.
