# ORCH_PLAN: Linux Shared-World Replacement Ordering, Rollback, and Atomic Metadata Writes

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)  
Reference style source: [llm-last-mile/ORCH_PLAN-11.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-11.md)  
Execution type: Linux backend hardening orchestration plan, no UI scope, shared-world replacement and durability slice

## Summary

This run operationalizes the root `PLAN.md` on `feat/session-centric-state-store` with an exact
active worker cap of `2`, but the honest execution shape is narrower than the cap. The parent
remains the only integrator, the only final branch writer, and the only agent allowed to mutate the
core shared-world transaction seam in
[crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)
and
[crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)
before any parallel work starts. The canonical orchestration state surface is
`.runs/root-plan-linux-shared-world-replace/run-state.json`.

This slice is not a new authority model, not a shell-owned binding-store project, not a public API
expansion, and not a platform-parity effort. It is one bounded production hardening run:

1. freeze the shared-binding transition rules, recovery contract, and atomic `session.json` write
   path in `session.rs`,
2. freeze replacement ordering and same-owner serialization in `lib.rs`,
3. prove the unchanged downstream `binding_state=Active` contract through targeted world-agent and
   shell regressions,
4. correct the stale authority and replacement-order docs only after the runtime contract is
   already proven.

The concurrency shape must follow `PLAN.md` exactly:

1. zero worker lanes during the core `crates/world/src/` work,
2. one late proof-seam lane after the core world contract is frozen,
3. one late docs lane after the same freeze,
4. parent-only integration, validation, and closeout after both late lanes finish.

Worker-model policy for this run:

1. the parent is the only integrator and final branch writer,
2. every child worker uses `GPT-5.4` with `reasoning_effort=high`,
3. the active worker cap remains exactly `2`,
4. `.runs/root-plan-linux-shared-world-replace/*` remains parent-owned only.

Worktree set for the run:

- `../substrate-root-proof-seams`
  - `codex/feat-session-centric-state-store-root-proof-seams`
- `../substrate-root-docs-drift`
  - `codex/feat-session-centric-state-store-root-docs-drift`

The parent-owned critical path is:

1. `task/root-a1-preflight`
2. `task/root-a2-session-state-and-durability`
3. `task/root-a3-backend-transaction-and-serialization`
4. `task/root-c1-integrate-and-validation-wall`
5. `task/root-c2-closeout`

## Hard Guards

### Locked invariants

1. This run is Linux-only. macOS and Windows remain compile-compatible only.
2. Linux metadata in `crates/world` remains the sole authority for shared-world ownership. No
   shell-side binding store is authorized.
3. No new wire schema is authorized in
   [crates/world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs)
   or any `agent-api-*` crate.
4. `binding_state=Active` remains the only proof state exposed to world-agent or shell consumers.
5. `world_generation` increments exactly once at commit, on the new world only.
6. Failed replacement before commit must preserve the original `world_id`,
   `world_generation`, and `binding_state=Active`.
7. A successful replace must end with exactly one committed `Active` world for the owner at
   generation `N+1`.
8. Shared-owner recovery must fail closed on ambiguity and must not silently delete malformed
   owner-bearing metadata.
9. `session.json` durability is part of the slice. Returning success after a torn or truncated
   metadata write is forbidden.
10. The coarse shared-owner mutex is allowed; a new lock-file protocol is not.
11. Docs are late and depend on the final runtime truth. Early docs edits are not authorized.
12. The parent is the only writer of `.runs/root-plan-linux-shared-world-replace/*`.
13. Package-targeted cargo commands must use the real package names in this repo: `world`,
   `world-agent`, and `shell`.

### File-level boundaries

Parent-owned serialized production surfaces:

- [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)
- [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)

Worker-safe proof seam lane after the core freeze:

- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/src/pty.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs)
- [crates/shell/src/execution/repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs)
- [crates/shell/src/execution/routing/dispatch/tests/repl_persistent_session_client_fail_closed.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/tests/repl_persistent_session_client_fail_closed.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)

Worker-safe docs lane after the core freeze:

- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
- [llm-last-mile/PLAN-03.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-03.md)
- [llm-last-mile/03-shared-world-ownership-linux-first.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/03-shared-world-ownership-linux-first.md)

Read-for-truth only:

- [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)
- [llm-last-mile/ORCH_PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-10.md)
- [llm-last-mile/ORCH_PLAN-11.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-11.md)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

### Non-negotiable stop conditions

Stop the run and write `.runs/root-plan-linux-shared-world-replace/blocked.json` if any of these
occur:

1. A task requires a new shell-owned shared-world authority seam or any second persisted source of
   truth.
2. A task requires a new public schema or wire contract in `world-api` or `agent-api-*`.
3. A task requires widening proof acceptance beyond `binding_state=Active`.
4. A task requires parallel edits to `crates/world/src/session.rs` and `crates/world/src/lib.rs`
   before the core contract is frozen.
5. A task proves the coarse same-owner mutex is insufficient and needs a broader synchronization
   design to complete the slice.
6. A task requires non-Linux behavior changes rather than additive compatibility.
7. A worker lane needs to touch a parent-owned `crates/world/src/*` file to finish its assignment.
8. A worker lane needs early docs edits to describe behavior the code and tests do not yet prove.
9. A task silently deletes malformed owner-bearing metadata or guesses through ambiguous recovery.

### Blocked-run record contract

`blocked.json` is parent-written only, and it is written exactly once at the moment the parent
decides the run cannot advance within `PLAN.md` scope.

Required fields in `.runs/root-plan-linux-shared-world-replace/blocked.json`:

- `run_id`
- `branch`
- `plan_source`
- `timestamp`
- `current_task_id`
- `gate_state`
- `stop_condition_id`
- `summary`
- `blocking_files`
- `worker_lane` when applicable, otherwise `null`
- `accepted_sentinels`
- `rejected_or_quarantined_outputs`
- `next_required_parent_action`

Blocked-write rules:

1. the parent writes `blocked.json` before any later-phase sentinel can be created,
2. the parent also updates `run-state.json` to a blocked terminal state in the same decision
   window,
3. `session.log` records the triggering evidence, including file path or command context,
4. partial worker output is either explicitly rejected or explicitly quarantined in `session.log`,
5. `closeout.md` is not written on a blocked run.

## Orchestration State Surfaces

### Canonical run state

The only canonical source of truth for run orchestration state:

- `.runs/root-plan-linux-shared-world-replace/run-state.json`

Parent-only writes to this file. It tracks:

- current phase,
- active task IDs,
- branch and worktree assignment,
- gate status,
- frozen shared-binding transition contract,
- frozen recovery decision tree,
- frozen atomic-write contract,
- frozen replacement transaction ordering,
- accepted and rejected worker outputs,
- blocked or completed terminal state,
- final closeout pointer.

If a worker report conflicts with `run-state.json`, the parent treats `run-state.json` as
authoritative until the parent explicitly reconciles the discrepancy.

### Derived run artifacts

The parent may maintain these local artifacts:

- `.runs/root-plan-linux-shared-world-replace/queue.json`
- `.runs/root-plan-linux-shared-world-replace/session.log`
- `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-a1-preflight.ok`
- `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-a2-session-state-and-durability.ok`
- `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-a3-backend-transaction-and-serialization.ok`
- `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-b1-proof-seam-regressions.ok`
- `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-b2-docs-drift-correction.ok`
- `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-c1-integrate-and-validation-wall.ok`
- `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-c2-closeout.ok`
- `.runs/root-plan-linux-shared-world-replace/blocked.json`
- `.runs/root-plan-linux-shared-world-replace/closeout.md`

Sentinel rules:

1. `.ok` means the parent validated the task output and advanced the run.
2. Missing sentinel means the task is not accepted.
3. `blocked.json` is written only on blocked termination.
4. `closeout.md` is written only on successful completion.
5. Worker notes, local commits, and branch state never replace parent-written sentinels or
   `run-state.json`.

## Concurrency Policy

1. The parent is the only integrator.
2. The parent is the only writer of final branch state on `feat/session-centric-state-store`.
3. Exact active worker cap: `2`.
4. There are zero worker lanes during `task/root-a1` through `task/root-a3`.
5. The only honest parallel window is after the core `crates/world/src/*` contract is frozen.
6. `task/root-b1` and `task/root-b2` are the only parallel tasks in this run.
7. No worker may edit `crates/world/src/session.rs` or `crates/world/src/lib.rs`.
8. The parent seeds both child worktrees from the exact post-`task/root-a3` tree.
9. If either worker discovers a required production change in a parent-owned file, that worker
   stops and hands the gap back to the parent instead of widening scope.
10. Worker coordination uses sentinels and long waits. Tight polling against branch or run-state
    is forbidden.

### Why the worker cap stays exactly `2`

The worker cap is exactly `2` because `PLAN.md` only authorizes late safe parallelism after the
core world contract lands:

1. `session.rs` and `lib.rs` are a single-lane choke point and must stay serialized,
2. the proof-seam work is independent only after the transaction and recovery contract is frozen,
3. the docs drift correction is independent only after the same wording is final,
4. there is no third honest lane before final validation because integration depends on both late
   outputs and one merged test wall.

## Approval And Gate Model

There are no human approval gates defined for this run.

Replacement control mechanism:

1. parent validation gates,
2. parent-written sentinels,
3. `session.log` for acceptance and rejection rationale,
4. `blocked.json` for hard-stop termination,
5. `closeout.md` for successful completion.

### Gate A: Scope And Topology Lock

Required before implementation starts:

- parent re-reads [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md),
  [llm-last-mile/ORCH_PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-10.md),
  and
  [llm-last-mile/ORCH_PLAN-11.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-11.md),
- parent records the exact execution shape: serialized world core, one proof lane, one docs lane,
  one final validation wall,
- parent records the exact worker cap of `2`,
- parent records parent-owned, worker-owned, and stop-back file boundaries.

### Gate B: Core World Contract Freeze

Required before worker lanes open:

- the shared-binding transition helper contract is frozen in `session.rs`,
- atomic `session.json` writes are frozen in `session.rs`,
- shared-owner recovery behavior is frozen in `session.rs`,
- replacement ordering and same-owner serialization are frozen in `lib.rs`,
- parent pre-worker validation proves the core world crate behavior before any worker lane starts,
- both child worktrees are seeded from the exact post-`task/root-a3` tree.

### Gate C: Late-Lane Integration

Required before final validation starts:

- the proof-seam lane and docs lane outputs are both reviewed and integrated by the parent,
- any production changes outside the planned late-lane surfaces are either rejected or rewritten by
  the parent with rationale recorded in `run-state.json`,
- docs wording reflects the actual committed runtime contract rather than speculative behavior.

### Gate D: Final Acceptance

Required before closeout:

- the integrated tree passes the exact validation command set from `PLAN.md`,
- rollback, lone-`Replacing`, `Active + Replacing`, concurrent same-owner race, and fail-closed
  downstream proof cases are all covered,
- docs are either updated minimally and truthfully or explicitly left unchanged,
- `run-state.json` records the final recovery, transaction, proof, and docs decisions.

## Workstream Plan

### Worktree topology

Parent checkout:

- current checkout on `feat/session-centric-state-store`

Child worktrees and branches:

- `../substrate-root-proof-seams`
  - `codex/feat-session-centric-state-store-root-proof-seams`
- `../substrate-root-docs-drift`
  - `codex/feat-session-centric-state-store-root-docs-drift`

Worktree creation commands:

```bash
git worktree add -b codex/feat-session-centric-state-store-root-proof-seams ../substrate-root-proof-seams feat/session-centric-state-store
git worktree add -b codex/feat-session-centric-state-store-root-docs-drift ../substrate-root-docs-drift feat/session-centric-state-store
```

### Task graph

Execution graph for the run:

1. `task/root-a1-preflight`
2. `task/root-a2-session-state-and-durability`
3. `task/root-a3-backend-transaction-and-serialization`
4. `task/root-b1-proof-seam-regressions` and `task/root-b2-docs-drift-correction` in parallel
5. `task/root-c1-integrate-and-validation-wall`
6. `task/root-c2-closeout`

Parent-only serialized tasks:

- `task/root-a1-preflight`
- `task/root-a2-session-state-and-durability`
- `task/root-a3-backend-transaction-and-serialization`
- `task/root-c1-integrate-and-validation-wall`
- `task/root-c2-closeout`

Worker-owned tasks:

- `task/root-b1-proof-seam-regressions`
- `task/root-b2-docs-drift-correction`

### task/root-a1-preflight

Ownership:

- parent only

Scope:

1. Re-read `PLAN.md` and the cited implementation seams.
2. Initialize `.runs/root-plan-linux-shared-world-replace/` state surfaces.
3. Record the exact frozen scope: Linux-only, no new authority seam, no public schema changes, no
   background reconciliation service.
4. Record the parent-owned choke point in `crates/world/src/*`.
5. Record the late worker window, worktree paths, and stop conditions.

Acceptance gate:

- Gate A passes,
- `run-state.json` contains the frozen topology and file boundaries,
- `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-a1-preflight.ok` exists.

### task/root-a2-session-state-and-durability

Ownership:

- parent only

Allowed files:

- [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)

Scope:

1. Replace one-way shared-binding mutation with a single internal transition helper.
2. Move `persist_metadata()` to atomic same-directory temp-file write semantics modeled on
   [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs).
3. Reconcile shared-owner recovery so lone `Replacing`, `Active + Replacing`, malformed
   owner-bearing metadata, and ambiguous multi-`Active` cases resolve exactly as defined in
   `PLAN.md`.
4. Add or extend world metadata tests required to prove the transition helper, atomic-write
   durability, and recovery rules.

Parent validation before sentinel:

1. `cargo test -p world -- --nocapture`

Acceptance gate:

- shared-binding state mutation is centralized,
- atomic write behavior is implemented and covered,
- recovery behavior is deterministic and covered,
- `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-a2-session-state-and-durability.ok`
  exists.

### task/root-a3-backend-transaction-and-serialization

Ownership:

- parent only

Allowed files:

- [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)
- [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)

Scope:

1. Rewrite `replace_shared_owner_session()` as explicit load, pre-commit, commit, finalize, and
   rollback phases.
2. Add the backend-local mutex guarding same-owner shared-world `AttachOrCreate` and
   `ReplaceExpectedGeneration`.
3. Extend backend tests for successful replace, generation mismatch, create failure rollback, and
   concurrent same-owner attach or replace races.
4. Freeze the final world-core contract before any late worker lane opens.

Parent pre-worker validation order:

1. `cargo test -p world -- --nocapture`
2. `cargo test -p world-agent --no-run`
3. `cargo test -p shell repl_persistent_session_client_fail_closed --no-run`
4. `cargo test -p shell --test repl_world_first_routing_v1 --no-run`

Acceptance gate:

- replacement ordering is explicit and rollback-safe,
- concurrent same-owner paths are serialized,
- Gate B passes,
- `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-a3-backend-transaction-and-serialization.ok`
  exists.

### task/root-b1-proof-seam-regressions

Ownership:

- worker lane `B1` only in `../substrate-root-proof-seams`
- worker model: `GPT-5.4`, `reasoning_effort=high`

Allowed files:

- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/src/pty.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs)
- [crates/shell/src/execution/repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs)
- [crates/shell/src/execution/routing/dispatch/tests/repl_persistent_session_client_fail_closed.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/tests/repl_persistent_session_client_fail_closed.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)

Forbidden touch surfaces:

- [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)
- [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
- [llm-last-mile/PLAN-03.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-03.md)
- [llm-last-mile/03-shared-world-ownership-linux-first.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/03-shared-world-ownership-linux-first.md)
- `.runs/root-plan-linux-shared-world-replace/*`

Scope:

1. Preserve the existing `Active`-only proof contract in world-agent and shell validators.
2. Add or extend proof-seam regressions so non-PTY and PTY flows surface only committed `Active`
   proofs.
3. Extend the shell stub harness so replace rollback and retry flows can prove the owner is not
   stranded after a failed replace.
4. Keep production behavior unchanged unless a minimal proof-preserving adjustment is required to
   make the regression coverage honest.

Minimum worker test commands before handoff:

1. `cargo test -p world-agent -- --nocapture`
2. `cargo test -p shell repl_persistent_session_client_fail_closed -- --nocapture`
3. `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`

Acceptance gate:

- the proof seams still fail closed for missing, stale, mismatched, or non-`Active` proofs,
- rollback and retry coverage proves the orchestration session remains recoverable,
- worker returns no request to reopen the frozen `crates/world/src/*` contract,
- `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-b1-proof-seam-regressions.ok`
  is written only after parent acceptance.

Stop-back conditions for lane `B1`:

- any need to weaken `Active`-only validation,
- any need to touch `crates/world/src/*`,
- any need to add new recovery semantics rather than proving the frozen contract.

### task/root-b2-docs-drift-correction

Ownership:

- worker lane `B2` only in `../substrate-root-docs-drift`
- worker model: `GPT-5.4`, `reasoning_effort=high`

Allowed files:

- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
- [llm-last-mile/PLAN-03.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-03.md)
- [llm-last-mile/03-shared-world-ownership-linux-first.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/03-shared-world-ownership-linux-first.md)

Forbidden touch surfaces:

- all `crates/world/src/*` files,
- all world-agent and shell production and test files,
- `.runs/root-plan-linux-shared-world-replace/*`

Scope:

1. Update `PLAN-03.md` only where replacement ordering no longer matches implementation.
2. Mark the shell-authoritative binding-store proposal in
   `03-shared-world-ownership-linux-first.md` as stale and historical only.
3. Update `docs/WORLD.md` to explain the Linux metadata authority, the `Replacing` crash window,
   and the recovery guarantees from the landed code.
4. Keep docs tightly aligned to the frozen world-core contract and avoid speculative future design
   language.

Minimum worker verification before handoff:

1. re-read the final accepted `PLAN.md` core contract summary from `run-state.json`,
2. provide a narrow diff summary showing each doc statement tied to a production behavior already
   frozen by `task/root-a2` and `task/root-a3`.

Acceptance gate:

- docs match the committed authority seam and replacement ordering,
- no docs edit tries to compensate for missing runtime proof,
- `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-b2-docs-drift-correction.ok`
  is written only after parent acceptance.

Stop-back conditions for lane `B2`:

- any need to reinterpret unfinished runtime behavior,
- any need to describe authority outside Linux metadata in `crates/world`,
- any need to widen scope into new operator features or future design work.

### task/root-c1-integrate-and-validation-wall

Ownership:

- parent only

Scope:

1. Integrate accepted outputs from lanes `B1` and `B2` in the parent checkout.
2. Resolve conflicts only in the parent branch.
3. Re-check that any proof-seam production change preserved `Active`-only behavior exactly.
4. Run the final validation wall in the merged tree.
5. Capture conditional repo-policy evidence if the final accepted diff changes shell or
   world-facing production behavior beyond tests and docs.

Required final validation order:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test -p world -- --nocapture`
4. `cargo test -p world-agent -- --nocapture`
5. `cargo test -p shell repl_persistent_session_client_fail_closed -- --nocapture`
6. `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`

Supplemental repo-policy evidence when applicable:

1. `substrate world doctor --json`
2. `substrate shim doctor --json`
3. `substrate health --json`

Acceptance gate:

- the exact `PLAN.md` validation stack is green,
- any supplemental doctor or health evidence required by repo policy is captured or explicitly
  noted as environment-blocked,
- Gate C and Gate D pass,
- `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-c1-integrate-and-validation-wall.ok`
  exists.

### task/root-c2-closeout

Ownership:

- parent only

Scope:

1. Confirm all required sentinels exist and `blocked.json` does not.
2. Finalize `run-state.json` with accepted decisions, worker dispositions, and validation results.
3. Write `.runs/root-plan-linux-shared-world-replace/closeout.md` with branch state, tests run,
   docs disposition, and any deferred work limited to the explicit out-of-scope list.
4. Confirm no unresolved scope creep remains into new schemas, shell-owned authority, non-Linux
   behavior, or background reconciliation services.

Acceptance gate:

- `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-c2-closeout.ok` exists,
- `.runs/root-plan-linux-shared-world-replace/closeout.md` exists,
- `run-state.json` records a completed terminal state.

## Context-Control Rules

1. The parent owns `.runs/root-plan-linux-shared-world-replace/*`. Workers do not edit
   `run-state.json`, sentinels, `queue.json`, `session.log`, `blocked.json`, or `closeout.md`.
2. The parent keeps only the following live in working context:
   - current task ID and gate state,
   - the frozen shared-binding transition rules,
   - the frozen atomic-write contract,
   - the frozen recovery decision tree,
   - the frozen replacement and rollback ordering,
   - exact allowed and forbidden files for each worker lane,
   - latest accepted worker summaries, diffs, blockers, and tests run.
3. Worker packets contain only:
   - task ID,
   - worktree path and branch name,
   - allowed files,
   - forbidden files,
   - frozen invariants,
   - stop-back conditions,
   - exact commands to run,
   - exact handoff format.
4. Each worker prompt must enumerate exact files it may and may not touch.
5. Workers stop immediately if they need to touch a parent-owned file or reinterpret the frozen
   recovery or transaction contract.
6. Each worker returns:
   - short result summary,
   - touched files,
   - exact commands run,
   - test outcomes,
   - blockers or unresolved assumptions,
   - narrow diff summary tied to touched files only,
   - explicit statement whether parent-owned follow-up is required.
7. Workers do not rebase, merge, integrate each other, or update parent run artifacts.
8. The parent reviews narrow diffs and validation evidence, not broad restatements of repo context.
9. If worker output conflicts with current parent truth, the parent re-derives the correct result
   locally instead of negotiating blended semantics across worktrees.

## Tests And Acceptance

### Pre-worker proof order

Run these parent-owned checks before dispatching the worker window:

1. `cargo test -p world -- --nocapture`
2. `cargo test -p world-agent --no-run`
3. `cargo test -p shell repl_persistent_session_client_fail_closed --no-run`
4. `cargo test -p shell --test repl_world_first_routing_v1 --no-run`

### Final validation order

Run these commands in this exact order during `task/root-c1-integrate-and-validation-wall`:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test -p world -- --nocapture`
4. `cargo test -p world-agent -- --nocapture`
5. `cargo test -p shell repl_persistent_session_client_fail_closed -- --nocapture`
6. `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`

### Acceptance checklist

#### World core

1. `set_shared_binding_state(...)` or equivalent is the only shared-binding mutation path.
2. `session.json` writes are atomic and preserve prior readable metadata on write failure.
3. Recovery restores a lone `Replacing` world to `Active`.
4. Recovery prefers the newer committed `Active` when `Active(g=N+1)` and `Replacing(g=N)` both
   exist.
5. Ambiguous multi-`Active` ownership fails closed.
6. Malformed owner-bearing metadata is warned on, ignored for reuse, and not silently deleted.

#### Backend transaction

1. replace ordering is explicit: load, pre-commit, commit, finalize, rollback,
2. creation failure before commit restores the original world to committed `Active`,
3. cleanup failure after rollback does not strand the old world,
4. concurrent same-owner attach or replace requests cannot create duplicate generation `0` worlds.

#### Proof seams

1. world-agent exposes only committed `Active` shared-world proof snapshots,
2. shell proof validation rejects missing, mismatched, stale-generation, or non-`Active` echoes,
3. replace rollback regressions prove the orchestration session remains recoverable,
4. proof-seam changes, if any, preserve fail-closed behavior exactly.

#### Docs and drift

1. `PLAN-03.md` matches the landed replacement ordering,
2. `03-shared-world-ownership-linux-first.md` explicitly marks the old shell-authoritative idea as
   historical only,
3. `docs/WORLD.md` documents the `Replacing` window and the recovery rules from the final code,
4. docs are updated only after runtime truth is proven.

#### Scope boundary

1. no new public API family is introduced,
2. no shell-owned authority seam is introduced,
3. no macOS or Windows behavior redesign is introduced,
4. `.runs/root-plan-linux-shared-world-replace/*` remains parent-owned only.

## Merge Refusal Rules

The parent refuses to merge a worker output if any of these are true:

1. The patch edits a file outside the task’s allowed file list.
2. The patch reopens the frozen `crates/world/src/*` contract after Gate B.
3. The patch weakens `binding_state=Active` as the only acceptable downstream proof state.
4. The patch adds or implies a second authority seam outside Linux metadata in `crates/world`.
5. The patch broadens scope into new schemas, non-Linux behavior, lock-file protocols, or
   background reconciliation work.
6. The patch changes docs to describe behavior that the parent has not yet accepted in code and
   tests.
7. The patch omits concrete command evidence for the behavior it claims to cover.
8. The patch requires coordinated edits to parent-owned files to become intelligible.
9. The patch silently resolves ambiguity instead of failing closed on ownership or proof state.

## Run Exit Criteria

### Successful run

The run is successful only if all of these are true:

1. `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-a1-preflight.ok` exists.
2. `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-a2-session-state-and-durability.ok`
   exists.
3. `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-a3-backend-transaction-and-serialization.ok`
   exists.
4. `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-b1-proof-seam-regressions.ok`
   exists.
5. `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-b2-docs-drift-correction.ok`
   exists.
6. `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-c1-integrate-and-validation-wall.ok`
   exists.
7. `.runs/root-plan-linux-shared-world-replace/sentinels/task-root-c2-closeout.ok` exists.
8. `.runs/root-plan-linux-shared-world-replace/run-state.json` exists and records a completed
   terminal state.
9. `.runs/root-plan-linux-shared-world-replace/queue.json` and
   `.runs/root-plan-linux-shared-world-replace/session.log` exist.
10. `.runs/root-plan-linux-shared-world-replace/closeout.md` exists and matches the final accepted
    branch state.
11. `.runs/root-plan-linux-shared-world-replace/blocked.json` does not exist.

### Blocked termination

The run terminates as blocked only if all of these are true:

1. `.runs/root-plan-linux-shared-world-replace/blocked.json` exists with the triggering
   `stop_condition_id`, `current_task_id`, evidence summary, and next required parent action.
2. `run-state.json` records a blocked terminal state in the same decision window.
3. No later-phase sentinel is written after the blocking condition is detected.
4. Partial worker output is either rejected or explicitly quarantined in `session.log`.
5. `.runs/root-plan-linux-shared-world-replace/closeout.md` does not exist.

## Closeout

At successful completion the parent writes and verifies exactly these artifacts:

1. updates `run-state.json` with final gate state, accepted worker outputs, command results, and
   docs disposition,
2. writes `closeout.md` with final branch state, validation commands run, environment-blocked
   checks if any, worker disposition, and explicit deferred items limited to `PLAN.md` out-of-scope
   work,
3. confirms every required sentinel exists and that `blocked.json` does not,
4. confirms the final tree still matches the honest topology: parent-owned world core, one late
   proof lane, one late docs lane, then parent-only integration and validation.

## Assumptions

1. `PLAN.md` is the authoritative implementation brief for this run and remains approved unless the
   parent explicitly reopens scope.
2. The branch for execution remains `feat/session-centric-state-store`.
3. The parent can stage the core `crates/world/src/*` work locally before dispatching late worker
   lanes.
4. The worker window does not open until the parent has a stable post-`task/root-a3` tree and
   pre-worker validation passes.
5. The required cargo targets in `PLAN.md` continue to exist with the package names `world`,
   `world-agent`, and `shell`.
6. Supplemental doctor and health commands may be environment-dependent; if they cannot run, the
   parent records that as evidence-blocked rather than silently skipping them.
7. `.runs/*` artifacts are local orchestration state only and are never treated as authored product
   output.
