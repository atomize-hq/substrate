# ORCH_PLAN: Root PLAN.md Execution Control Artifact

Branch: `feat/session-centric-state-store`  
Primary plan: [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)  
Refinement sources: [llm-last-mile/PLAN-07.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-07.md), [llm-last-mile/ORCH_PLAN-06.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-06.md)  
Execution type: orchestration-only control artifact for completing the Linux shared-world replacement hardening slice

## Summary

This document is the parent-run control artifact for executing [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) to completion on `feat/session-centric-state-store`.

The parent agent is:

1. the only integrator,
2. the only final branch writer,
3. the only authority for `.runs/root-plan-shared-world-replacement/*`,
4. the only lane allowed to mutate the core `crates/world` production seam.

This run is not allowed to treat `PLAN.md` as if it were greenfield. Current repo truth already contains large parts of the intended fix. The run therefore starts with a mandatory parent-owned delta audit that classifies every `PLAN.md` requirement as one of:

- `landed`
- `partial`
- `missing`
- `out_of_scope`

Already-landed items are completion status, not reasons to reopen the seam.

The honest dependency graph is mostly serialized. `crates/world/src/lib.rs` and `crates/world/src/session.rs` remain a single parent-owned choke point. Parallelism opens only after that contract is frozen, and even then only if the audit shows remaining proof-boundary or doc-drift work. Exact worker cap: `2`. Actual worker count may be `0`, `1`, or `2`.

## Execution Defaults

| Control | Default |
| --- | --- |
| parent lane | current checkout at `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` on `feat/session-centric-state-store` |
| worker model | `GPT-5.4` with `reasoning_effort=high` |
| exact worker cap | `2` |
| final integration authority | parent only |
| final branch writer | parent only |
| canonical orchestration state surfaces | `.runs/root-plan-shared-world-replacement/run-state.json`, `.runs/root-plan-shared-world-replacement/tasks.json`, `.runs/root-plan-shared-world-replacement/session.log` |

## Current Repo Truth That Changes The Run

The parent must treat these as observed branch facts at run start, then verify them more completely in `task-root-a1-delta-audit`.

| Area | Current repo truth | Evidence | Run implication |
| --- | --- | --- | --- |
| Shared binding transitions | `SessionWorld::set_shared_binding_state(...)` already exists | `crates/world/src/session.rs` | Do not re-plan this as missing production work |
| Atomic metadata writes | `SessionWorld::persist_metadata()` already uses temp-file write + rename + sync flow | `crates/world/src/session.rs` | Audit behavior and tests before changing implementation |
| Replacement ordering | `replace_shared_owner_session_from_root_with_creator(...)` already does pre-commit, rollback, and finalize phases | `crates/world/src/lib.rs` | Audit for remaining edge gaps instead of re-implementing the transaction from scratch |
| Same-owner serialization | `shared_owner_mutex` already gates shared-owner `ensure_session()` flow | `crates/world/src/lib.rs` | Treat concurrency control as landed unless the audit proves a gap |
| Recovery posture | shared-owner recovery already handles `Active`, `Replacing`, and ambiguity fail-closed behavior | `crates/world/src/session.rs` | Audit exact edge-case coverage and tests |
| Downstream proof boundary | world-agent already rejects non-`Active` proof snapshots | `crates/world-agent/src/service.rs` | Preserve boundary contract exactly |
| Authority docs | `docs/WORLD.md` and `llm-last-mile/03-shared-world-ownership-linux-first.md` already reflect large parts of the corrected seam | `docs/WORLD.md`, `llm-last-mile/03-shared-world-ownership-linux-first.md` | Docs lane is drift-correction, not first-authoring |
| Existing regression base | world/session tests already cover success, rollback, and recovery foundations | `crates/world/src/lib.rs`, `crates/world/src/session.rs` | The audit must distinguish test gaps from already-closed cases |

Parent rule: if `PLAN.md` says "implement X" but the branch already has X, the run records `X = landed` and moves on.

## Workstream Overview

| Workstream | Ownership | Shape | Contents |
| --- | --- | --- | --- |
| parent serialized phase A | parent only | serialized | `task-root-a0-packet-freeze`, `task-root-a1-delta-audit`, `task-root-a2-core-reconcile` |
| conditional late worker window | worker lanes if justified, parent fallback otherwise | at most 2 parallel lanes | `task-root-b1-proof-late-lane`, `task-root-b2-doc-drift-lane` |
| final integration and closeout | parent only | serialized | `task-root-c1-integrate-validate`, `task-root-c2-closeout` |

Operational reading:

1. The run starts in one parent-owned serialized lane.
2. The worker window opens only after Gate C and only if the delta audit proves real remaining proof or docs work.
3. The parent integrates all accepted late-lane output back into `feat/session-centric-state-store`.
4. If the delta audit and core reconcile close the slice without remaining late-lane work, the worker window stays closed.

## Hard Guards

### Locked invariants

1. [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) is the scope authority for this run.
2. Current code truth determines completion status. Stale plan prose does not justify churn.
3. Linux metadata in `crates/world` remains the only authority seam for shared-owner replacement in this slice.
4. `binding_state=Active` remains the only reusable downstream proof state.
5. `world_generation` increments exactly once per committed replacement world.
6. Replacement create failure before commit must preserve one recoverable old `Active` world.
7. Recovery remains deterministic and fail-closed on ambiguity.
8. No new wire schema, second authority store, shell-owned binding registry, lock-file protocol, background reconciler, or non-Linux redesign is allowed.
9. The parent remains the only final writer on `feat/session-centric-state-store`.
10. Worker lanes stop instead of widening into parent-owned `crates/world` files.
11. Docs are allowed to describe only behavior present in the final integrated tree.
12. Validation failure after a "no-op" delta audit is still a failed run.

### Parent-only seams

| Surface | Ownership rule | Why |
| --- | --- | --- |
| `crates/world/src/lib.rs` | parent only for the full run | core replacement transaction and backend-local serialization live here |
| `crates/world/src/session.rs` | parent only for the full run | binding state machine, recovery, and metadata durability live here |
| inline unit tests inside the two files above | parent only | they overlap the same production seam and must stay serialized |
| `.runs/root-plan-shared-world-replacement/*` | parent only | canonical run-state authority |

### Worker-safe late seams

These lanes open only after Gate C and only if the delta audit proves they still contain remaining work.

| Lane | Allowed files |
| --- | --- |
| `task-root-b1-proof-late-lane` | `crates/world-agent/src/service.rs`, `crates/world-agent/src/pty.rs`, `crates/shell/src/execution/repl_persistent_session.rs`, `crates/shell/src/execution/routing/dispatch/tests/repl_persistent_session_client_fail_closed.rs`, `crates/shell/tests/repl_world_first_routing_v1.rs`, `crates/shell/tests/support/repl_world_agent.rs` |
| `task-root-b2-doc-drift-lane` | `docs/WORLD.md`, `llm-last-mile/PLAN-03.md`, `llm-last-mile/03-shared-world-ownership-linux-first.md` |

### Read-only context sources

These are read-for-truth inputs during the run, not edit targets by default:

- `PLAN.md`
- `llm-last-mile/PLAN-07.md`
- `llm-last-mile/ORCH_PLAN-06.md`
- any run artifacts created under `.runs/root-plan-shared-world-replacement/`

### Non-negotiable stop conditions

Write `blocked.json` and stop the run if any of these occur:

1. A required fix needs schema or API work in `crates/world-api/**` or `crates/agent-api-*`.
2. A required fix needs a second authority store outside `crates/world`.
3. A required fix widens into macOS or Windows behavior redesign.
4. A worker needs to edit `crates/world/src/lib.rs` or `crates/world/src/session.rs`.
5. A proof lane requires downstream acceptance of non-`Active` binding proof.
6. A required concurrency fix needs something materially wider than the current backend-local shared-owner mutex.
7. The docs lane would have to document behavior not implemented in the integrated tree.
8. The delta audit shows the slice is already complete but the branch cannot be validated cleanly.

## Run-Control Packet

### Canonical packet path

All run-control state lives under:

- `.runs/root-plan-shared-world-replacement/`

### Canonical artifacts

| Artifact | Authority | Required contents |
| --- | --- | --- |
| `.runs/root-plan-shared-world-replacement/run-state.json` | parent only | phase, active task IDs, gate states, accepted outputs, blocked/completed terminal state, frozen contract summaries |
| `.runs/root-plan-shared-world-replacement/tasks.json` | parent only | task inventory, dependencies, owner, worktree path, branch, status, acceptance ref, sentinel path |
| `.runs/root-plan-shared-world-replacement/session.log` | parent only | append-only gate decisions, dispatch events, acceptance notes, blockers, closeout notes |

### Derived artifacts

The parent may create these supporting artifacts:

- `.runs/root-plan-shared-world-replacement/delta-audit.md`
- `.runs/root-plan-shared-world-replacement/core-freeze.md`
- `.runs/root-plan-shared-world-replacement/validation-report.md`
- `.runs/root-plan-shared-world-replacement/closeout.md`
- `.runs/root-plan-shared-world-replacement/blocked.json`
- `.runs/root-plan-shared-world-replacement/sentinels/task-root-a0-packet-freeze.ok`
- `.runs/root-plan-shared-world-replacement/sentinels/task-root-a1-delta-audit.ok`
- `.runs/root-plan-shared-world-replacement/sentinels/task-root-a2-core-reconcile.ok`
- `.runs/root-plan-shared-world-replacement/sentinels/task-root-b1-proof-late-lane.ok`
- `.runs/root-plan-shared-world-replacement/sentinels/task-root-b2-doc-drift-lane.ok`
- `.runs/root-plan-shared-world-replacement/sentinels/task-root-c1-integrate-validate.ok`
- `.runs/root-plan-shared-world-replacement/sentinels/task-root-c2-closeout.ok`

### Packet rules

1. `run-state.json` is the live source of truth.
2. `tasks.json` is the canonical dispatch queue.
3. `session.log` is explanatory only.
4. Workers never write authoritative state into any packet file.
5. Missing `.ok` sentinel means "not accepted".
6. `blocked.json` exists only on blocked termination.
7. `closeout.md` exists only on successful completion.

## Worktree And Branch Topology

### Integration lane

| Role | Path | Branch | Notes |
| --- | --- | --- | --- |
| parent integration lane | `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` | `feat/session-centric-state-store` | the only final branch writer |

No extra parent integration worktree is created for this run. The serialized core seam already lives in the current checkout, and a second parent worktree would add merge overhead without adding truthful concurrency.

### Conditional worker worktrees

| Lane | Path | Branch | Opens at |
| --- | --- | --- | --- |
| proof lane | `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/root-plan-shared-world-replacement/proof` | `codex/feat-session-centric-state-store-root-plan-proof` | Gate C only |
| docs lane | `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/root-plan-shared-world-replacement/docs` | `codex/feat-session-centric-state-store-root-plan-docs` | Gate C only |

Suggested materialization commands:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/root-plan-shared-world-replacement
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/root-plan-shared-world-replacement/proof -b codex/feat-session-centric-state-store-root-plan-proof feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/root-plan-shared-world-replacement/docs -b codex/feat-session-centric-state-store-root-plan-docs feat/session-centric-state-store
```

Worktree rules:

1. Worker worktrees are created only after the parent writes `core-freeze.md`.
2. Worker worktrees are seeded from the exact frozen parent commit recorded at Gate C.
3. Workers do not merge each other and do not write to `feat/session-centric-state-store`.

## Dependency Graph And Parallelism Truth

### Task graph

```text
task-root-a0-packet-freeze
    ->
task-root-a1-delta-audit
    ->
task-root-a2-core-reconcile
    ->
Gate C worker decision
    ->
{ task-root-b1-proof-late-lane || task-root-b2-doc-drift-lane || skip }
    ->
task-root-c1-integrate-validate
    ->
task-root-c2-closeout
```

### Parallelism verdict

This run is mostly serialized.

1. `task-root-a0`, `task-root-a1`, and `task-root-a2` are parent-only.
2. The worker window is late and conditional.
3. The docs lane can run in parallel with the proof lane only after the parent freezes the core contract.
4. If the delta audit shows no remaining proof or docs gaps, the worker window stays closed and the run proceeds directly to parent validation and closeout.

## Gate Model

| Gate | Must be true | Evidence |
| --- | --- | --- |
| Gate A: packet freeze | branch, scope, run packet, task graph, guards, and stop conditions are frozen | `run-state.json`, `tasks.json`, sentinel `task-root-a0-packet-freeze.ok` |
| Gate B: delta freeze | every `PLAN.md` requirement is classified `landed`, `partial`, `missing`, or `out_of_scope` with code/doc/test evidence | `delta-audit.md`, sentinel `task-root-a1-delta-audit.ok` |
| Gate C: core freeze / worker decision | all remaining parent-only `crates/world` delta is landed or proven already complete; frozen contract is written | `core-freeze.md`, sentinel `task-root-a2-core-reconcile.ok` |
| Gate D: final acceptance | integrated tree passes required validation and closeout maps every plan requirement to final status | `validation-report.md`, `closeout.md`, sentinels `task-root-c1-integrate-validate.ok` and `task-root-c2-closeout.ok` |

## Task Inventory

| Task ID | Owner | Depends on | Worktree | Branch |
| --- | --- | --- | --- | --- |
| `task-root-a0-packet-freeze` | parent | — | parent checkout | `feat/session-centric-state-store` |
| `task-root-a1-delta-audit` | parent | `task-root-a0-packet-freeze` | parent checkout | `feat/session-centric-state-store` |
| `task-root-a2-core-reconcile` | parent | `task-root-a1-delta-audit` | parent checkout | `feat/session-centric-state-store` |
| `task-root-b1-proof-late-lane` | worker or parent fallback | `task-root-a2-core-reconcile` | proof worktree | `codex/feat-session-centric-state-store-root-plan-proof` |
| `task-root-b2-doc-drift-lane` | worker or parent fallback | `task-root-a2-core-reconcile` | docs worktree | `codex/feat-session-centric-state-store-root-plan-docs` |
| `task-root-c1-integrate-validate` | parent | all accepted late lanes or explicit skips | parent checkout | `feat/session-centric-state-store` |
| `task-root-c2-closeout` | parent | `task-root-c1-integrate-validate` | parent checkout | `feat/session-centric-state-store` |

## Task Packets

### `task-root-a0-packet-freeze`

Owner: parent only

Scope:

1. Confirm the checkout branch is `feat/session-centric-state-store`.
2. Re-read `PLAN.md`, `ORCH_PLAN.md`, `llm-last-mile/PLAN-07.md`, and `llm-last-mile/ORCH_PLAN-06.md`.
3. Initialize `.runs/root-plan-shared-world-replacement/`.
4. Write the initial `run-state.json`, `tasks.json`, and `session.log`.
5. Freeze the hard guards, stop conditions, and task graph.

Commands:

```bash
git rev-parse --abbrev-ref HEAD
git status --short
mkdir -p .runs/root-plan-shared-world-replacement/sentinels
```

Done when:

1. `run-state.json` exists and names Gate A as complete.
2. `tasks.json` exists with the exact task IDs in this file.
3. `task-root-a0-packet-freeze.ok` exists.

Stop if:

1. the branch is not `feat/session-centric-state-store`,
2. the parent cannot create the run packet,
3. the repo state is too inconsistent to freeze a truthful task graph.

### `task-root-a1-delta-audit`

Owner: parent only

Scope:

1. Audit every requirement in `PLAN.md` against live branch truth.
2. Audit the required surfaces listed in the table below.
3. Classify each `PLAN.md` requirement as `landed`, `partial`, `missing`, or `out_of_scope`.
4. Record exact evidence references in `delta-audit.md`.
5. Convert already-landed plan text into completion status instead of fresh implementation work.

Required audit surfaces:

| Area | Required files |
| --- | --- |
| core replacement seam | `crates/world/src/lib.rs`, `crates/world/src/session.rs` |
| proof boundary seam | `crates/world-agent/src/service.rs`, `crates/world-agent/src/pty.rs`, `crates/shell/src/execution/repl_persistent_session.rs` |
| late regression seam | `crates/shell/src/execution/routing/dispatch/tests/repl_persistent_session_client_fail_closed.rs`, `crates/shell/tests/repl_world_first_routing_v1.rs`, `crates/shell/tests/support/repl_world_agent.rs` |
| authority docs seam | `docs/WORLD.md`, `llm-last-mile/PLAN-03.md`, `llm-last-mile/03-shared-world-ownership-linux-first.md` |

Minimum audit matrix categories:

1. core state machine,
2. atomic metadata durability,
3. replacement ordering,
4. shared-owner serialization,
5. shared-owner recovery,
6. downstream proof validation,
7. targeted regression coverage,
8. authority docs.

Commands:

```bash
rg -n "set_shared_binding_state|persist_metadata|recover_shared_active_from_root|replace_shared_owner_session|shared_owner_mutex" crates/world/src
rg -n "resolve_shared_world_binding|binding_state" crates/world-agent/src crates/shell/src/execution
rg -n "shared-world|binding_state|world_generation|replacing|replaced" docs/WORLD.md llm-last-mile/PLAN-03.md llm-last-mile/03-shared-world-ownership-linux-first.md
```

Done when:

1. `delta-audit.md` maps every root-plan requirement to a status with evidence.
2. `run-state.json` records the audit outcome.
3. `task-root-a1-delta-audit.ok` exists.

Stop if:

1. the audit shows required work outside run scope,
2. current branch truth is too inconsistent to classify safely,
3. a required seam cannot be validated from repo truth.

### `task-root-a2-core-reconcile`

Owner: parent only

Scope:

1. Land any remaining delta in `crates/world/src/lib.rs` and `crates/world/src/session.rs`.
2. Keep all `crates/world` edits serialized in the parent checkout.
3. If the delta audit shows the parent-only seam is already complete, record this explicitly as a no-op completion instead of opening the seam.
4. Freeze the final parent-owned core contract in `core-freeze.md`.
5. Record whether the late worker window is justified.

Allowed files:

- `crates/world/src/lib.rs`
- `crates/world/src/session.rs`
- inline tests inside those files

Required output:

1. either a bounded parent diff in the files above,
2. or a no-op core completion record with evidence that the required behavior is already landed.

Commands:

```bash
cargo test -p world -- --nocapture
```

Done when:

1. all remaining `crates/world` delta is landed or proven already complete,
2. `core-freeze.md` exists and states the final core truth for this run,
3. `run-state.json` records whether `task-root-b1` and `task-root-b2` are needed, optional, or skipped,
4. `task-root-a2-core-reconcile.ok` exists.

Stop if:

1. a required core fix widens outside the parent-owned seam,
2. a required fix needs forbidden new architecture,
3. the parent cannot get `cargo test -p world` green after core edits.

### `task-root-b1-proof-late-lane`

Owner: worker only if Gate C opens this lane; otherwise parent may mark it skipped

Scope:

1. Land only the remaining proof-boundary and late regression gaps identified in `delta-audit.md`.
2. Preserve `binding_state=Active` as the only downstream proof.
3. Extend late proof coverage without reopening `crates/world`.

Allowed files:

- `crates/world-agent/src/service.rs`
- `crates/world-agent/src/pty.rs`
- `crates/shell/src/execution/repl_persistent_session.rs`
- `crates/shell/src/execution/routing/dispatch/tests/repl_persistent_session_client_fail_closed.rs`
- `crates/shell/tests/repl_world_first_routing_v1.rs`
- `crates/shell/tests/support/repl_world_agent.rs`

Forbidden files:

- `crates/world/src/lib.rs`
- `crates/world/src/session.rs`
- all `.runs/root-plan-shared-world-replacement/*`

Commands:

```bash
cargo test -p world-agent -- --nocapture
cargo test -p shell repl_persistent_session_client_fail_closed -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

Done when:

1. the proof lane closes every remaining audit item assigned to it,
2. only committed `Active` proof escapes all touched paths,
3. touched files stay inside the allowed list,
4. the worker returns file list, commands run, results, and blockers,
5. the parent accepts the lane and writes `task-root-b1-proof-late-lane.ok`.

Stop if:

1. the lane needs `crates/world/src/*`,
2. the lane needs doc edits beyond its scope,
3. the lane would weaken the proof boundary or widen accepted proof states.

### `task-root-b2-doc-drift-lane`

Owner: worker only if Gate C opens this lane; otherwise parent may mark it skipped

Scope:

1. Correct only the doc drift that still remains after the integrated code truth is known.
2. Keep docs aligned with the authority seam already implemented.
3. Remove any stale shell-authoritative or dual-authority wording if it still exists.

Allowed files:

- `docs/WORLD.md`
- `llm-last-mile/PLAN-03.md`
- `llm-last-mile/03-shared-world-ownership-linux-first.md`

Forbidden files:

- any production code
- `PLAN.md`
- all `.runs/root-plan-shared-world-replacement/*`

Commands:

```bash
rg -n "shell-owned|binding store|authoritative|binding_state|world_generation|replacing|replaced" docs/WORLD.md llm-last-mile/PLAN-03.md llm-last-mile/03-shared-world-ownership-linux-first.md
```

Done when:

1. the docs describe the final integrated authority and recovery semantics,
2. the docs do not overclaim beyond actual code behavior,
3. touched files stay inside the allowed list,
4. the parent accepts the lane and writes `task-root-b2-doc-drift-lane.ok`.

Stop if:

1. the correct doc text depends on unresolved production behavior,
2. the lane needs to modify proof or core code,
3. the lane would preserve stale authority prose for convenience.

### `task-root-c1-integrate-validate`

Owner: parent only

Scope:

1. Integrate accepted late-lane outputs into `feat/session-centric-state-store`.
2. Reject or rewrite any worker output that violates lane boundaries.
3. Run the final validation stack in the integrated tree.
4. Record exact results in `validation-report.md`.

Integration rules:

1. proof lane integrates before docs lane if proof results change contract wording,
2. docs lane integrates last by default,
3. the parent reruns the smallest relevant targeted validation after each accepted worker patch,
4. the parent may absorb a skipped or rejected worker task back into the parent lane only if that does not violate the hard guards.

Final validation order:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p world -- --nocapture
cargo test -p world-agent -- --nocapture
cargo test -p shell repl_persistent_session_client_fail_closed -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

Conditional validation:

1. If `crates/shell/src/execution/repl_persistent_session.rs` changes, also run `cargo test -p shell --tests --no-run`.
2. If `crates/world-agent/src/pty.rs` changes, the proof lane must show PTY-path evidence, not just the service helper unit tests.

Done when:

1. all accepted late-lane work is integrated,
2. the final validation order passes,
3. `validation-report.md` exists,
4. `task-root-c1-integrate-validate.ok` exists.

Stop if:

1. final validations fail,
2. worker output conflicts with the frozen core contract and cannot be cleanly reconciled,
3. docs still contradict the integrated tree after the last integration pass.

### `task-root-c2-closeout`

Owner: parent only

Scope:

1. Write `closeout.md`.
2. Map every `PLAN.md` requirement to one final state in `{ landed_before_run, completed_during_run, validated_no_change, blocked }`.
3. Record the exact validation commands run and outcomes.
4. Record whether each late lane was executed, skipped, or absorbed by the parent.
5. Mark the run terminal state in `run-state.json`.

Done when:

1. `closeout.md` exists,
2. `run-state.json` records `completed`,
3. all required accepted task sentinels exist or are explicitly recorded as skipped in `run-state.json`,
4. `task-root-c2-closeout.ok` exists.

Stop if:

1. the parent cannot produce a truthful requirement-by-requirement closeout,
2. validation evidence is incomplete,
3. the run is actually blocked rather than complete.

## Context-Control Rules

### Parent live context

The parent keeps only these surfaces live as orchestration context:

- `PLAN.md`
- `ORCH_PLAN.md`
- `llm-last-mile/PLAN-07.md`
- `llm-last-mile/ORCH_PLAN-06.md`
- `.runs/root-plan-shared-world-replacement/run-state.json`
- `.runs/root-plan-shared-world-replacement/tasks.json`
- `.runs/root-plan-shared-world-replacement/delta-audit.md`
- `.runs/root-plan-shared-world-replacement/core-freeze.md`
- the current integrated diff summary

### Worker prompt contract

The parent dispatches each worker with one exact, self-sufficient packet. That packet must contain:

1. the exact task ID and verbatim task section from this file,
2. the exact `delta-audit.md` excerpt that defines the remaining gap,
3. the exact allowed file list,
4. the exact forbidden file list,
5. the exact assigned worktree path and branch,
6. the exact commands to run,
7. the exact stop conditions,
8. the requirement that the worker must stop instead of widening scope.

Dispatch rule:

1. The parent does not send workers a paraphrased objective.
2. The parent does not rely on implied file boundaries.
3. The parent does not rely on workers to rediscover the remaining delta from repo context.
4. The worker packet is the complete operating envelope for the lane.

### Worker return contract

Every worker return must contain:

1. exact files changed,
2. commands run and exit codes,
3. a short result summary against the task acceptance rules,
4. blockers or unresolved assumptions.

Workers do not return orchestration decisions. The parent makes those.

## Merge Refusal Rules

The parent refuses to integrate a worker output if any of these are true:

1. it edits a file outside the allowed list,
2. it assumes a second authority seam,
3. it reopens `crates/world` through an indirect helper edit,
4. it weakens the `Active`-only proof boundary,
5. it omits test evidence for behavior it claims to cover,
6. it leaves docs contradicting the integrated code truth.

## Validation And Acceptance Matrix

| Stage | Owner | Required checks | Acceptance rule |
| --- | --- | --- | --- |
| packet freeze | parent | `git rev-parse --abbrev-ref HEAD`, `git status --short` | branch and packet truth are frozen |
| delta audit | parent | targeted `rg` against core, proof, and docs seams | every root-plan requirement is classified with evidence |
| core reconcile | parent | `cargo test -p world -- --nocapture` | parent-owned `crates/world` seam is stable or proven already complete |
| proof lane | worker + parent | `cargo test -p world-agent -- --nocapture`, `cargo test -p shell repl_persistent_session_client_fail_closed -- --nocapture`, `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture` | only committed `Active` proof escapes; no forbidden touches |
| docs lane | worker + parent | targeted `rg` against final doc files | docs match final integrated behavior |
| final acceptance | parent | final validation order from `task-root-c1` | integrated tree is green and closeout is truthful |

## Tests And Acceptance

Green-path acceptance requires all of these truth surfaces to hold at once:

| Truth surface | What must be true |
| --- | --- |
| delta audit truth | `delta-audit.md` classifies every root-plan requirement with evidence, and already-landed mechanics are recorded as completion status rather than reopened work |
| core seam truth | the final `crates/world/src/lib.rs` and `crates/world/src/session.rs` state is either newly reconciled or explicitly validated as already landed, with `core-freeze.md` capturing the final contract |
| proof boundary truth | only committed `binding_state=Active` proof escapes downstream, and all remaining proof-lane gaps identified by the audit are closed or explicitly proven already green |
| docs drift truth | `docs/WORLD.md`, `llm-last-mile/PLAN-03.md`, and `llm-last-mile/03-shared-world-ownership-linux-first.md` describe the final integrated authority and recovery semantics without speculative claims |
| workspace / branch / worktree truth | the parent run stays anchored to `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` on `feat/session-centric-state-store`, worker worktrees if any are seeded from the frozen Gate C commit, and workers never write directly to the parent branch |
| closeout truth | `validation-report.md`, `closeout.md`, `run-state.json`, `tasks.json`, and required sentinels agree on the final run outcome with no hidden open work |

Required green-path command stack:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p world -- --nocapture
cargo test -p world-agent -- --nocapture
cargo test -p shell repl_persistent_session_client_fail_closed -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

Acceptance rule:

1. A "mostly landed already" audit is not sufficient by itself.
2. A docs-only green path is not sufficient by itself.
3. The run is green only when audit truth, core truth, proof truth, docs truth, workspace truth, and closeout truth all line up in the integrated tree.

## Blocked Path Behavior

If the run blocks:

1. write `.runs/root-plan-shared-world-replacement/blocked.json`,
2. update `run-state.json` to terminal state `blocked`,
3. append `session.log` with the blocking reason and last safe gate,
4. do not write downstream acceptance sentinels after the blocking point,
5. do not write `closeout.md`.

## Run Exit Criteria

### Successful run

The run is successful only if all of these are true:

1. `task-root-a0-packet-freeze.ok` exists.
2. `task-root-a1-delta-audit.ok` exists.
3. `task-root-a2-core-reconcile.ok` exists.
4. Any executed late lane has a matching `.ok` sentinel.
5. `task-root-c1-integrate-validate.ok` exists.
6. `task-root-c2-closeout.ok` exists.
7. `run-state.json` records terminal state `completed`.
8. `closeout.md` exists.
9. `blocked.json` does not exist.

### Blocked termination

The run is blocked only if all of these are true:

1. `blocked.json` exists.
2. `run-state.json` records terminal state `blocked`.
3. `closeout.md` does not exist.
4. no later sentinel exists beyond the blocking point.

## Assumptions

1. The execution branch remains `feat/session-centric-state-store`.
2. The parent may create local worktrees and `.runs/root-plan-shared-world-replacement/*` artifacts during execution.
3. `cargo`, `rustfmt`, and `clippy` are available.
4. `llm-last-mile/PLAN-07.md` is a refinement aid, not a replacement for `PLAN.md`.
5. The most likely outcome of the delta audit is "core production seam mostly landed, remaining work concentrated in proof gaps, tests, or doc drift", but the run must verify that rather than assume it.
6. If unrelated user edits appear in parent-owned core seams during execution, the parent pauses and re-audits before editing them.
