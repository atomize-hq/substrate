# ORCH_PLAN: Linux Shared-World Replacement Hardening

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)  
Execution type: Linux backend hardening and documentation orchestration plan, no UI scope

## Summary

This run executes [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) on the current branch `feat/session-centric-state-store` with an exact worker cap of `2`. The parent is the only integrator, the only final branch writer, and the only agent allowed to accept or reject widened scope. The canonical orchestration state source of truth is `.runs/linux-shared-world-replace/run-state.json`.

The critical path is intentionally serialized through the shared-owner core semantics in [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:16) and [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs:148). No proof-seam or docs worker opens until the parent has frozen the replacement transaction, recovery, atomic persistence, and same-owner serialization contract in the integration lane.

Worktree set for the run:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/core` on `ws/lswr-core`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/proof` on `ws/lswr-proof`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/docs` on `ws/lswr-docs`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/int` on `ws/lswr-int`

The parent-owned critical path is:

1. `task/lswr-a1-preflight`
2. `task/lswr-a2-core-transaction-recovery`
3. `task/lswr-c1-integrate-and-validate`
4. `task/lswr-c2-closeout`

The worker lanes are late and bounded:

- `task/lswr-b1-proof-regressions`
- `task/lswr-b2-doc-drift-correction`

This is deliberate. The honest parallelism in this slice begins only after the core `crates/world` contract is stable. Earlier concurrency would create false throughput because the replacement ordering, recovery, and serialization work all converge in the same two production files.

## Orchestration State Surfaces

### Canonical run state

Single local source of truth for the run:

- `.runs/linux-shared-world-replace/run-state.json`

Parent-only writes to this file. It tracks:

- current phase,
- active task IDs,
- branch and worktree assignment,
- gate status,
- accepted and rejected worker outputs,
- blocked or completed terminal state,
- final closeout pointer,
- frozen core contract summary.

If a worker report conflicts with `run-state.json`, the parent trusts `run-state.json` until it explicitly reconciles the discrepancy.

### Derived run artifacts

These are orchestration control surfaces, not product source and not assumed tracked deliverables:

- `.runs/linux-shared-world-replace/queue.json`
- `.runs/linux-shared-world-replace/session.log`
- `.runs/linux-shared-world-replace/core-freeze.md`
- `.runs/linux-shared-world-replace/blocked.json`
- `.runs/linux-shared-world-replace/closeout.md`
- `.runs/linux-shared-world-replace/logs/`
- `.runs/linux-shared-world-replace/sentinels/task-lswr-a1-preflight.ok`
- `.runs/linux-shared-world-replace/sentinels/task-lswr-a2-core-transaction-recovery.ok`
- `.runs/linux-shared-world-replace/sentinels/task-lswr-b1-proof-regressions.ok`
- `.runs/linux-shared-world-replace/sentinels/task-lswr-b2-doc-drift-correction.ok`
- `.runs/linux-shared-world-replace/sentinels/task-lswr-c1-integrate-and-validate.ok`
- `.runs/linux-shared-world-replace/sentinels/task-lswr-c2-closeout.ok`

Sentinel rules:

1. `.ok` means the parent validated the task output and advanced the run.
2. Missing sentinel means the task is not accepted.
3. `blocked.json` is written only on blocked termination.
4. `closeout.md` is written only on successful completion.
5. Worker notes never replace parent-written sentinels or the canonical run-state file.

## Hard Guards

### Locked invariants

1. This is a Linux-only hardening and docs session. macOS and Windows behavior must continue compiling with additive compatibility only.
2. `binding_state=Active` remains the only proof state that world-agent and shell may surface or accept.
3. `world_generation` increments exactly once, only at the commit point for the new durable replacement world.
4. A failed replace before commit preserves the original `world_id`, `world_generation`, and `binding_state=Active`.
5. Shared-owner recovery must reconcile `Active` and `Replacing` deterministically and must never silently reset generation to `0`.
6. Malformed owner-bearing metadata must be warned on, retained on disk, and treated as non-reusable or ambiguous. It must not be silently deleted during shared-owner recovery.
7. Generic compatible reuse for non-owner flows remains unchanged.
8. No new authority seam is allowed. Linux metadata in `crates/world` remains the only authority for this slice.
9. No new persisted schema, no new crate, no new background service, no new lock-file protocol, and no shell-owned binding store are allowed.
10. [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) is the implementation source of truth and is read-only during this session.

### File-level boundaries

Parent-owned production choke points:

- [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:16)
- [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs:148)

Worker-safe proof and validation surfaces after core freeze:

- [crates/shell/src/execution/routing/dispatch/tests/repl_persistent_session_client_fail_closed.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/tests/repl_persistent_session_client_fail_closed.rs:1)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs:1)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs:393)

Read-only proof seams unless the parent explicitly reopens scope:

- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:2328)
- [crates/world-agent/src/pty.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs:211)
- [crates/shell/src/execution/repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs:308)

Worker-safe docs surfaces after core freeze:

- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
- [llm-last-mile/PLAN-03.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-03.md)
- [llm-last-mile/03-shared-world-ownership-linux-first.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/03-shared-world-ownership-linux-first.md)

### Non-negotiable stop conditions

Stop the run and write `.runs/linux-shared-world-replace/blocked.json` if any of these occur:

1. A lane requires edits in `crates/world-api/**`, `crates/agent-api-*`, or any new schema surface.
2. A lane requires a new shell-owned or cross-process authority seam for shared-world binding.
3. A lane widens scope into macOS or Windows behavior redesign instead of additive compile compatibility.
4. A worker lane needs to edit `crates/world/src/session.rs` or `crates/world/src/lib.rs` after Gate B without the parent absorbing the change.
5. A proof lane requires widening the proof contract beyond `Active`-only acceptance.
6. A fix requires a distributed lock-file protocol or any concurrency design beyond the backend-local shared-owner mutex already chosen in `PLAN.md`.
7. A docs lane would need to promise behavior not already implemented in the merged core lane or proven in the proof lane.
8. The integrated validation stack shows a regression that contradicts the frozen core contract and the parent cannot resolve it without widening scope.

## Concurrency Policy

1. The parent is the only integrator.
2. The parent is the only final branch writer.
3. Exact worker cap: `2`.
4. No worker opens during `task/lswr-a1-preflight` or `task/lswr-a2-core-transaction-recovery`.
5. The only honest parallel window is after the parent freezes the core `crates/world` contract.
6. Proof merges before docs because proof regressions are the executable confirmation that the frozen contract still exposes only committed `Active` truth.
7. Docs merge after proof so prose is aligned to the actually integrated semantics and test evidence, not to a speculative interpretation.
8. If a worker discovers missing support changes outside its owned files, it stops and returns the change to the parent instead of widening scope.

## Approval And Gate Model

There are no human product approval gates inside the session. The run is controlled by parent validation gates plus one final human review after the integrated validation stack is green.

### Gate A: Preflight freeze

Required before any code lane starts:

- parent re-reads [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md),
- parent records the invariant list and stop conditions in `run-state.json`,
- parent creates the `.runs/linux-shared-world-replace/` control surfaces,
- parent creates the integration worktree and records the starting commit.

### Gate B: Core freeze

Required before worker lanes launch:

- `task/lswr-a2-core-transaction-recovery` is merged into the integration lane,
- parent writes `.runs/linux-shared-world-replace/core-freeze.md`,
- parent records these frozen facts:
  - `set_shared_binding_state()` is the only shared-binding state transition path,
  - `persist_metadata()` writes atomically,
  - shared-owner recovery reconciles `Active` and `Replacing` deterministically,
  - `replace_shared_owner_session()` uses pre-commit, commit, rollback, finalize ordering,
  - same-owner shared-world `ensure_session()` paths are serialized in the Linux backend.

### Gate C: Worker launch

Required before proof and docs worktrees open:

- Gate B is green,
- the integration lane compiles logically enough for late validation work,
- the parent seeds both worker worktrees from the frozen integration state,
- worker prompts contain only lane-owned surfaces, exact commands, and stop conditions.

### Gate D: Final acceptance

Required before closeout:

- proof lane is accepted or deliberately rejected and replaced by parent work,
- docs lane is accepted or deliberately rejected and replaced by parent work,
- the exact final validation stack from `PLAN.md` passes in the integration lane,
- the final human review sees the command results and acceptance summary.

## Workstream Plan

### Worktree topology

Parent integration worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/int`
- branch: `ws/lswr-int`

Child worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/core`
  - `ws/lswr-core`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/proof`
  - `ws/lswr-proof`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/docs`
  - `ws/lswr-docs`

The parent integrates only in `ws/lswr-int`. Worker lanes never merge each other and never merge directly into `feat/session-centric-state-store`.

### Task graph

```text
task/lswr-a1-preflight
    ->
task/lswr-a2-core-transaction-recovery
    ->
Gate B core freeze
    ->
{ task/lswr-b1-proof-regressions || task/lswr-b2-doc-drift-correction }
    ->
task/lswr-c1-integrate-and-validate
    ->
Gate D final acceptance
    ->
task/lswr-c2-closeout
```

### `task/lswr-a1-preflight` — parent

Owner:

- parent only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/int`

Owned surfaces:

- `.runs/linux-shared-world-replace/**`

Required commands:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/int -b ws/lswr-int feat/session-centric-state-store
mkdir -p .runs/linux-shared-world-replace/{logs,sentinels}
git status --short > .runs/linux-shared-world-replace/start-status.txt
git rev-parse HEAD > .runs/linux-shared-world-replace/start-head.txt
cat > .runs/linux-shared-world-replace/run-state.json <<'EOF'
{
  "plan": "linux-shared-world-replace",
  "branch": "feat/session-centric-state-store",
  "phase": "preflight",
  "active_tasks": ["task/lswr-a1-preflight"],
  "gates": {
    "gate_a_preflight": "in_progress",
    "gate_b_core_freeze": "pending",
    "gate_c_worker_launch": "pending",
    "gate_d_final_acceptance": "pending"
  }
}
EOF
```

Acceptance:

- integration worktree exists on `ws/lswr-int`,
- canonical run-state file exists and records the invariant list,
- run-artifact surfaces exist,
- no source file changed yet.

Halt conditions:

- worktree creation fails,
- repo state is too broken to capture a deterministic starting point.

### `task/lswr-a2-core-transaction-recovery` — parent

Owner:

- parent only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/core`
- branch: `ws/lswr-core`

Owns exactly:

- [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:16)
- [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs:148)

Scope:

- replace one-way shared-binding mutation with the single transition helper,
- switch `session.json` persistence to atomic writes,
- reconcile shared-owner recovery across `Active` and `Replacing`,
- rewrite Linux replacement ordering into pre-commit, commit, rollback, finalize,
- serialize same-owner shared-world `ensure_session()` paths.

Required commands:

```bash
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/core -b ws/lswr-core feat/session-centric-state-store
cargo test -p world -- --nocapture | tee .runs/linux-shared-world-replace/logs/core-world.log
git -C /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/int merge --no-ff ws/lswr-core
```

Acceptance:

- successful replace leaves exactly one committed `Active` world at generation `N+1`,
- failed replace preserves the original `Active` world, generation, and `world_id`,
- lone `Replacing` recovery and `Active + Replacing` reconciliation are covered,
- no file outside the two owned `crates/world/src` files changes.

Halt conditions:

- fix appears to require `crates/world-api`, `agent-api`, or non-Linux backend edits,
- shared-owner mutex must widen beyond the shared-owner branch,
- implementation cannot preserve the `Active`-only downstream proof contract.

### `task/lswr-b1-proof-regressions` — worker 1

Owner:

- one worker, late phase only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/proof`
- branch: `ws/lswr-proof`

Starts only after Gate B.

Owns exactly:

- [crates/shell/src/execution/routing/dispatch/tests/repl_persistent_session_client_fail_closed.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/tests/repl_persistent_session_client_fail_closed.rs:1)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs:1)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs:393)

Read-only unless the parent explicitly reopens scope:

- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:2328)
- [crates/world-agent/src/pty.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs:211)
- [crates/shell/src/execution/repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs:308)

Scope:

- prove world-agent still surfaces only committed `Active` proof,
- prove shell fail-closed validation still rejects missing, stale, mismatched, or non-`Active` proof,
- prove replace failure does not strand a later attach or retry.

Required commands:

```bash
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/proof -b ws/lswr-proof ws/lswr-int
cargo test -p world-agent -- --nocapture | tee .runs/linux-shared-world-replace/logs/proof-world-agent.log
cargo test -p shell repl_persistent_session_client_fail_closed -- --nocapture | tee .runs/linux-shared-world-replace/logs/proof-shell-fail-closed.log
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture | tee .runs/linux-shared-world-replace/logs/proof-shell-routing.log
```

Acceptance:

- rollback paths still expose only committed `Active` proof,
- stale generation echo remains rejected,
- stubbed end-to-end rollback proves the orchestration session is still recoverable.

Halt conditions:

- any production change in `service.rs`, `pty.rs`, or `repl_persistent_session.rs` appears necessary,
- tests imply the core lane widened semantics beyond the frozen `Active`-only proof contract.

### `task/lswr-b2-doc-drift-correction` — worker 2

Owner:

- one worker, late phase only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/docs`
- branch: `ws/lswr-docs`

Starts only after Gate B.

Owns exactly:

- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
- [llm-last-mile/PLAN-03.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-03.md)
- [llm-last-mile/03-shared-world-ownership-linux-first.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/03-shared-world-ownership-linux-first.md)

Scope:

- document Linux metadata authority in `crates/world`,
- document the two-phase replace window and recovery guarantees,
- mark the shell-authoritative binding-store proposal as historical and stale,
- align plan prose to the final merged replacement ordering.

Required commands:

```bash
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/docs -b ws/lswr-docs ws/lswr-int
```

No cargo command is required before merge if this lane remains docs-only.

Acceptance:

- `docs/WORLD.md` explains `Replacing`, commit/rollback windows, and recovery rules,
- `PLAN-03.md` matches the final replacement ordering now implemented in `crates/world`,
- `03-shared-world-ownership-linux-first.md` explicitly marks shell-owned binding-store authority as stale.

Halt conditions:

- core semantics are still moving,
- docs would need to promise behavior not validated by the core or proof lanes.

### `task/lswr-c1-integrate-and-validate` — parent

Owner:

- parent only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/int`
- branch: `ws/lswr-int`

Required merge order:

1. confirm `ws/lswr-core` is already merged into `ws/lswr-int`,
2. confirm `core-freeze.md` matches the merged core semantics,
3. merge `ws/lswr-proof` into `ws/lswr-int`,
4. merge `ws/lswr-docs` into `ws/lswr-int`,
5. run the integrated validation stack.

Required commands:

```bash
git merge --no-ff ws/lswr-proof
git merge --no-ff ws/lswr-docs
cargo fmt --all -- --check | tee .runs/linux-shared-world-replace/logs/fmt.log
cargo clippy --workspace --all-targets -- -D warnings | tee .runs/linux-shared-world-replace/logs/clippy.log
cargo test -p world -- --nocapture | tee .runs/linux-shared-world-replace/logs/world.log
cargo test -p world-agent -- --nocapture | tee .runs/linux-shared-world-replace/logs/world-agent.log
cargo test -p shell repl_persistent_session_client_fail_closed -- --nocapture | tee .runs/linux-shared-world-replace/logs/shell-fail-closed.log
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture | tee .runs/linux-shared-world-replace/logs/shell-repl-world-first.log
```

Acceptance:

- all lane sentinels exist,
- final validation stack is green,
- final diff still respects the locked invariants from `PLAN.md`,
- parent records whether any proof-seam production files changed,
- parent records the observed results for rollback regression, lone-`Replacing` recovery, and concurrent same-owner race regression.

Halt conditions:

- integrated validation contradicts the frozen core contract,
- a merge conflict requires creative redesign instead of straightforward integration,
- final diff widens scope beyond the owned surfaces and stop conditions.

### `task/lswr-c2-closeout` — parent

Owner:

- parent only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/linux-shared-world-replace/int`

Scope:

- write `closeout.md`,
- update `run-state.json` to completed,
- leave the worker worktrees intact until final human review is complete,
- only after approval, remove worktrees or keep them for audit as needed.

Acceptance:

- final human review has the integrated command logs,
- `closeout.md` summarizes final acceptance against `PLAN.md`,
- the run is either completed or blocked with a durable reason.

## Context-Control Rules

1. The parent keeps only four live artifacts in active working context:
   - [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)
   - [ORCH_PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/ORCH_PLAN.md)
   - `.runs/linux-shared-world-replace/run-state.json`
   - the latest integration diff summary
2. Each worker prompt contains only:
   - its lane section from this file,
   - the exact relevant excerpt from `PLAN.md`,
   - the current branch name,
   - its owned surfaces,
   - required commands,
   - stop conditions.
3. Workers return only:
   - changed files,
   - commands run with exit codes,
   - blockers or unresolved assumptions.
4. Workers do not widen scope on their own.
5. Workers do not ingest each other’s transcripts.
6. The parent is the only agent allowed to interpret cross-lane results, change merge order, or declare a gate green.
7. Every gate decision is recorded in `.runs/linux-shared-world-replace/` before the next lane starts.

## Tests And Acceptance

| Gate | Owner | Commands | Must be true before proceeding |
| --- | --- | --- | --- |
| Gate A preflight freeze | parent | worktree and run-state bootstrap commands | integration lane exists, run-state exists, invariants and stop conditions are recorded |
| Gate B core freeze | parent + `task/lswr-a2-core-transaction-recovery` | `cargo test -p world -- --nocapture` | replace rollback preserves the old world; successful replace yields one committed new `Active`; recovery is deterministic for `Replacing` windows |
| Gate C worker launch | parent | parent review of merged core diff + `core-freeze.md` | proof and docs lanes are seeded from the frozen core contract and no overlapping production churn remains |
| Proof acceptance | `task/lswr-b1-proof-regressions` + parent | `cargo test -p world-agent -- --nocapture`<br>`cargo test -p shell repl_persistent_session_client_fail_closed -- --nocapture`<br>`cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture` | only committed `Active` proof escapes; rollback does not strand the session; stale proof remains rejected |
| Docs acceptance | `task/lswr-b2-doc-drift-correction` + parent | parent semantic diff review only | docs match merged semantics and authority seam |
| Gate D final acceptance | parent | `cargo fmt --all -- --check`<br>`cargo clippy --workspace --all-targets -- -D warnings`<br>`cargo test -p world -- --nocapture`<br>`cargo test -p world-agent -- --nocapture`<br>`cargo test -p shell repl_persistent_session_client_fail_closed -- --nocapture`<br>`cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture` | the full session is shippable on this branch and the final acceptance list below is true |

Final acceptance for the session:

- replacement create failure never leaves the owner with zero recoverable world,
- pre-commit failure preserves old `world_id`, `world_generation`, and `binding_state=Active`,
- successful replace returns only the committed new `Active` proof with `world_generation = expected_generation + 1`,
- `session.json` writes are atomic and never expose torn bytes on successful return,
- recovery from each replace crash window is deterministic and generation-safe,
- world-agent and shell still reject non-`Active` proof states,
- concurrent same-owner attach or replace requests cannot create duplicate generation `0` worlds,
- malformed owner-bearing metadata is never silently deleted by shared-owner recovery,
- docs no longer describe a shell-authoritative binding store as active design,
- final review bundle includes the validation evidence required by `PLAN.md`.

## Assumptions

- [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) already reflects current branch truth and remains the governing implementation plan for this run.
- The repository may already be dirty outside this slice. The session must tolerate that state and avoid reverting unrelated edits.
- Worktree creation under `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/` is available.
- Package names used by validation remain `world`, `world-agent`, and `shell`.
- No human approval is needed between internal gates unless a hard guard is tripped; the only planned human pause is the final review after integrated validation.
