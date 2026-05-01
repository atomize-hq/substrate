# ORCH_PLAN: PLAN.md Execution Control Artifact

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)  
Refined sibling reference: [llm-last-mile/PLAN-07.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-07.md)  
Execution type: Linux backend hardening, proof preservation, and drift-correction orchestration plan

## Summary

This document is the authoritative orchestration control artifact for executing [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) to completion on `feat/session-centric-state-store`. The parent agent is the only integrator, the only final branch writer, and the only agent allowed to own the overlapping `crates/world` production seam.

Current repo truth matters here. On this branch, the core mechanisms that [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) describes as missing are already at least partially landed:

- [`SessionWorld::set_shared_binding_state()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:212) exists.
- [`SessionWorld::persist_metadata()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:586) already uses an atomic temp-file rename flow.
- [`LinuxLocalBackend::replace_shared_owner_session_from_root_with_creator()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs:261) already expresses pre-commit, rollback, and finalize phases.
- [`LinuxLocalBackend::shared_owner_mutex`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs:29) already serializes the shared-owner branch.
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md) and [llm-last-mile/03-shared-world-ownership-linux-first.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/03-shared-world-ownership-linux-first.md) already reflect parts of the corrected authority seam.

Because of that, this run does not start by blindly reimplementing the root plan text. It starts with a mandatory parent-owned delta audit against [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md), [llm-last-mile/PLAN-07.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-07.md), and current code truth. If a `PLAN.md` requirement is already satisfied in code or docs, the parent records it as complete and does not churn the seam.

The honest parallelism is narrow. `crates/world/src/lib.rs` and `crates/world/src/session.rs` remain a single parent-owned choke point. Worker concurrency opens only after the parent freezes the core contract or explicitly records that no remaining core delta exists. Exact worker cap: `2`.

## Hard Guards

### Locked invariants

1. [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) is the scope authority for this run.
2. If [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) disagrees with current repo truth about already-landed mechanics, the parent follows `PLAN.md` for scope and current code for completion status. It does not reintroduce churn to make the tree match stale prose.
3. Linux metadata in `crates/world` remains the only authority seam for shared-owner replacement in this slice.
4. `binding_state=Active` remains the only downstream proof state that world-agent and shell may expose or accept.
5. `world_generation` increments exactly once, only on the committed replacement world.
6. A failed replacement before commit must preserve the prior reusable `Active` world.
7. Shared-owner recovery must remain deterministic and fail closed on ambiguity.
8. No new wire schema, no new authority store, no new lock-file protocol, no new background service, and no non-Linux behavior redesign are allowed.
9. The parent remains the sole integrator for the branch.
10. Worker lanes must stop instead of widening scope into parent-owned production seams.

### Parent-only surfaces

The parent exclusively owns these production choke points for the full run:

- [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)
- [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)

The parent also owns any world test additions that must live inline in those two files.

### Conditionally mutable proof seams

These surfaces are read-only by default and open for changes only if the delta audit shows real remaining gaps after the core freeze:

- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/src/pty.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs)
- [crates/shell/src/execution/repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs)

### Worker-safe late-phase surfaces

After the parent freezes the core contract, workers may own these non-overlapping late-phase seams:

- [crates/shell/src/execution/routing/dispatch/tests/repl_persistent_session_client_fail_closed.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/tests/repl_persistent_session_client_fail_closed.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
- [llm-last-mile/PLAN-03.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-03.md)
- [llm-last-mile/03-shared-world-ownership-linux-first.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/03-shared-world-ownership-linux-first.md)

### Non-negotiable stop conditions

Stop the run and write `blocked.json` if any of these occur:

1. A required fix needs new API or schema work in `crates/world-api/**` or `crates/agent-api-*`.
2. A required fix needs a second authority store outside `crates/world`.
3. A required fix widens into macOS or Windows shared-owner behavior redesign.
4. A worker needs to edit [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs) or [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs).
5. A proof lane requires downstream acceptance of non-`Active` binding states.
6. A fix requires concurrency design beyond the existing backend-local shared-owner mutex.
7. A docs lane would document behavior not implemented in the integrated tree.
8. The delta audit proves the slice is already complete but the branch is too dirty in parent-owned surfaces to validate safely.

## Orchestration State Surfaces

### Canonical run state

Single local source of truth for the run:

- `.runs/root-plan-shared-world-replace/run-state.json`

Parent-only writes to this file. It tracks:

- current phase,
- active task IDs,
- branch and worktree assignment,
- accepted and rejected worker outputs,
- gate status,
- blocked or completed terminal state,
- delta-audit results,
- frozen core contract summary,
- final closeout pointer.

If any worker report conflicts with `run-state.json`, the parent trusts `run-state.json` until it explicitly reconciles the discrepancy.

### Run-control packet

The parent keeps one sharp run-control packet under:

- `.runs/root-plan-shared-world-replace/`

Canonical control artifacts inside that packet:

- `.runs/root-plan-shared-world-replace/run-state.json`
  - current phase, active task IDs, gate states, accepted outputs, blocked/completed state
- `.runs/root-plan-shared-world-replace/tasks.json`
  - canonical task inventory, dependency graph, owner, worktree path, branch, status, and dispatch order
- `.runs/root-plan-shared-world-replace/session.log`
  - append-only parent log for gate decisions, worker dispatch, integration results, blockers, and closeout notes

Control packet rules:

1. `run-state.json` is the canonical live state surface.
2. `tasks.json` is the canonical queue and dispatch surface.
3. `session.log` is append-only and explanatory, never authoritative over `run-state.json` or `tasks.json`.
4. Workers do not write authoritative state into any of these files.
5. The parent updates `tasks.json` before dispatching or accepting any task transition.
6. Each `tasks.json` entry records at minimum:
   - `task_id`
   - `owner`
   - `status`
   - `depends_on`
   - `worktree_path`
   - `branch`
   - `acceptance_ref`
   - `sentinel_path`

### Derived run artifacts

The parent may maintain these local orchestration artifacts:

- `.runs/root-plan-shared-world-replace/tasks.json`
- `.runs/root-plan-shared-world-replace/session.log`
- `.runs/root-plan-shared-world-replace/delta-audit.md`
- `.runs/root-plan-shared-world-replace/core-freeze.md`
- `.runs/root-plan-shared-world-replace/validation-report.md`
- `.runs/root-plan-shared-world-replace/blocked.json`
- `.runs/root-plan-shared-world-replace/closeout.md`
- `.runs/root-plan-shared-world-replace/sentinels/task-root-a1-preflight-delta-audit.ok`
- `.runs/root-plan-shared-world-replace/sentinels/task-root-a2-core-contract-completion.ok`
- `.runs/root-plan-shared-world-replace/sentinels/task-root-b1-proof-regressions.ok`
- `.runs/root-plan-shared-world-replace/sentinels/task-root-b2-doc-drift-correction.ok`
- `.runs/root-plan-shared-world-replace/sentinels/task-root-c1-integrate-and-validate.ok`
- `.runs/root-plan-shared-world-replace/sentinels/task-root-c2-closeout.ok`

Sentinel rules:

1. `.ok` means the parent validated the task output and advanced the run.
2. Missing sentinel means the task is not accepted.
3. `blocked.json` exists only on blocked termination.
4. `closeout.md` exists only on successful completion.
5. Worker notes never replace parent-written sentinels.

## Concurrency Policy

1. The parent is the only integrator.
2. The parent is the only writer of final branch state on `feat/session-centric-state-store`.
3. Exact worker cap: `2`.
4. There are no production-code workers before the parent freezes the core `crates/world` contract.
5. `crates/world/src/session.rs` and `crates/world/src/lib.rs` are a single serialized lane for the entire run.
6. World test additions that live inline in those files stay in the parent lane with the production code.
7. The only honest worker window is late phase proof/docs work after Gate C.
8. If the delta audit shows the core contract is already complete, the parent may mark `task-root-a2-core-contract-completion` complete as a no-op and open the late worker window immediately after Gate B.
9. If a worker discovers a missing change in a parent-only seam, it stops and hands the gap back to the parent. It does not widen scope itself.

## Approval And Gate Model

There are no human approval gates inside the run. Control is through parent validation gates and parent-written sentinels.

### Gate A: Packet freeze

Required before any implementation task starts:

- parent confirms the branch is `feat/session-centric-state-store`,
- parent re-reads [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) and [llm-last-mile/PLAN-07.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-07.md),
- parent initializes `.runs/root-plan-shared-world-replace/`,
- parent records the invariant list and stop conditions in `run-state.json`,
- parent materializes `tasks.json` with the exact task graph, ownership, dependencies, worktree paths, and dispatch order for this run.

### Gate B: Delta freeze

Required before worker launch:

- `task-root-a1-preflight-delta-audit` is accepted,
- the parent has marked each `PLAN.md` requirement as `landed`, `partial`, or `missing`,
- the parent has either finished `task-root-a2-core-contract-completion` or recorded that it is a no-op because the core contract is already landed,
- `core-freeze.md` names the final parent-owned core truth for this run.

### Gate C: Worker launch

Required before proof/docs worktrees open:

- Gate B is green,
- no structural churn remains in the parent-owned core seam,
- worker prompts list exact allowed files, commands, acceptance rules, and stop conditions.

### Gate D: Final acceptance

Required before closeout:

- worker outputs are accepted or deliberately rejected and replaced by parent work,
- the validation stack defined below is green in the integrated tree,
- docs reflect the final merged contract and do not overclaim beyond current code truth.

## Workstream Plan

### Worktree topology

Parent integration lane:

- path: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- branch: `feat/session-centric-state-store`
- role: parent-owned integration lane and final branch writer

Dedicated integration worktree decision:

- The parent integrates in the current checkout at `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`.
- No dedicated integration worktree is created for this run.
- Reason: the core lane is fully serialized in parent-owned `crates/world` seams, and the final branch state must land on `feat/session-centric-state-store` itself. A second integration checkout would add one more merge hop and one more run-control surface without creating real concurrency or reducing contention.

Worker worktrees and branches:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/root-plan-shared-world-replace/proof`
  - `codex/feat-session-centric-state-store-root-plan-proof-regressions`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/root-plan-shared-world-replace/docs`
  - `codex/feat-session-centric-state-store-root-plan-doc-drift`

Worktree materialization rules:

1. Worker worktrees are created only after Gate C.
2. Both worker worktrees are seeded from the parent checkout after `core-freeze.md` is written.
3. The parent never dispatches workers from stale pre-freeze commits.
4. Workers never merge each other and never write directly to `feat/session-centric-state-store`.

Commands where useful:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/root-plan-shared-world-replace
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/root-plan-shared-world-replace/proof -b codex/feat-session-centric-state-store-root-plan-proof-regressions feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/root-plan-shared-world-replace/docs -b codex/feat-session-centric-state-store-root-plan-doc-drift feat/session-centric-state-store
```

### Task graph

```text
task-root-a1-preflight-delta-audit
    ->
task-root-a2-core-contract-completion
    ->
Gate C worker launch
    ->
{ task-root-b1-proof-regressions || task-root-b2-doc-drift-correction }
    ->
task-root-c1-integrate-and-validate
    ->
task-root-c2-closeout
```

### Parent-only serialized tasks

- `task-root-a1-preflight-delta-audit`
- `task-root-a2-core-contract-completion`
- `task-root-c1-integrate-and-validate`
- `task-root-c2-closeout`

### Worker-owned tasks

- `task-root-b1-proof-regressions`
- `task-root-b2-doc-drift-correction`

### task-root-a1-preflight-delta-audit

Ownership:

- parent only

Lane:

- worktree: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- branch: `feat/session-centric-state-store`

Scope:

1. Re-read [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) and [llm-last-mile/PLAN-07.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-07.md).
2. Audit current code and docs truth in:
   - [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)
   - [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)
   - [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
   - [crates/shell/src/execution/repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs)
   - [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
   - [llm-last-mile/PLAN-03.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-03.md)
   - [llm-last-mile/03-shared-world-ownership-linux-first.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/03-shared-world-ownership-linux-first.md)
3. Mark every root-plan requirement as `landed`, `partial`, or `missing`.
4. Freeze the exact target file list for the rest of the run.
5. Initialize the `.runs/root-plan-shared-world-replace/` control surfaces.

Owned surfaces:

- `.runs/root-plan-shared-world-replace/**`

Commands where useful:

```bash
git status --short
git rev-parse --abbrev-ref HEAD
rg -n "set_shared_binding_state|persist_metadata|recover_shared_active_from_root|replace_shared_owner_session|shared_owner_mutex" crates/world/src
rg -n "resolve_shared_world_binding|validate_shared_world_echo|binding_state" crates/world-agent/src crates/shell/src crates/shell/tests
rg -n "shell-authoritative|binding store|replacing|active" docs/WORLD.md llm-last-mile/PLAN-03.md llm-last-mile/03-shared-world-ownership-linux-first.md
```

Acceptance:

1. The branch is confirmed as `feat/session-centric-state-store`.
2. The parent can state which `PLAN.md` items are already landed and which still require execution.
3. `delta-audit.md` names the real remaining work without duplicating completed code.
4. `tasks.json` contains the concrete task graph, owner, branch, worktree, and dispatch order for the run.

Outputs and sentinels:

- `.runs/root-plan-shared-world-replace/delta-audit.md`
- `.runs/root-plan-shared-world-replace/sentinels/task-root-a1-preflight-delta-audit.ok`

### task-root-a2-core-contract-completion

Ownership:

- parent only

Lane:

- worktree: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- branch: `feat/session-centric-state-store`

Why serialized:

- [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs) and [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs) are the single core contract seam.
- Any world test additions that prove rollback, reconciliation, cleanup, or mutex behavior are inline in those same files.

Scope:

1. Execute only the remaining core deltas from `delta-audit.md`.
2. Keep the work limited to:
   - `set_shared_binding_state()` transition rigor,
   - `persist_metadata()` durability and cleanup behavior,
   - `recover_shared_active_from_root()` deterministic reconciliation,
   - replacement transaction ordering,
   - same-owner serialization,
   - inline world regressions required to prove those behaviors.
3. Do not mutate proof-seam production files unless the audit proves the core contract cannot be completed otherwise.
4. If the audit shows the core contract is already complete, record a no-op completion and freeze the existing implementation as authoritative for the run.

Owned surfaces:

- [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)
- [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)

Commands where useful:

```bash
cargo test -p world -- --nocapture
```

Acceptance:

1. The parent has either landed the remaining core deltas or proven no core delta remains.
2. `core-freeze.md` captures the exact transaction, recovery, persistence, and serialization contract to which worker lanes must conform.
3. No worker-owned seam is needed to reinterpret the core contract.

Outputs and sentinels:

- `.runs/root-plan-shared-world-replace/core-freeze.md`
- `.runs/root-plan-shared-world-replace/sentinels/task-root-a2-core-contract-completion.ok`

### task-root-b1-proof-regressions

Ownership:

- worker

Lane:

- worktree: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/root-plan-shared-world-replace/proof`
- branch: `codex/feat-session-centric-state-store-root-plan-proof-regressions`

Scope:

1. Close remaining proof and routing test gaps after the core freeze.
2. Keep production proof behavior fail-closed and `Active`-only.
3. Prefer test-only edits. Touch production proof files only when adding colocated unit coverage is required and the parent has explicitly opened that file.

Owned surfaces:

- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs) only for colocated unit tests if needed
- [crates/shell/src/execution/routing/dispatch/tests/repl_persistent_session_client_fail_closed.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/tests/repl_persistent_session_client_fail_closed.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)

Commands where useful:

```bash
cargo test -p world-agent -- --nocapture
cargo test -p shell repl_persistent_session_client_fail_closed -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

Acceptance:

1. Remaining proof tests prove that only committed `Active` proof escapes.
2. Replace rollback and retry flows in shell-facing harnesses never strand the orchestration session.
3. No worker edit leaks into parent-only `crates/world` files.

Outputs and sentinels:

- worker patch or branch for parent integration from `codex/feat-session-centric-state-store-root-plan-proof-regressions`
- `.runs/root-plan-shared-world-replace/sentinels/task-root-b1-proof-regressions.ok`

### task-root-b2-doc-drift-correction

Ownership:

- worker

Lane:

- worktree: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/root-plan-shared-world-replace/docs`
- branch: `codex/feat-session-centric-state-store-root-plan-doc-drift`

Scope:

1. Align authority docs to the frozen core contract.
2. Correct any remaining stale wording that implies shell-owned co-authority, incorrect replace ordering, or unsupported proof states.
3. Keep docs descriptive of the merged tree, not speculative.

Owned surfaces:

- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
- [llm-last-mile/PLAN-03.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-03.md)
- [llm-last-mile/03-shared-world-ownership-linux-first.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/03-shared-world-ownership-linux-first.md)

Commands where useful:

```bash
rg -n "shell-authoritative|binding store|replacing|replaced|active" docs/WORLD.md llm-last-mile/PLAN-03.md llm-last-mile/03-shared-world-ownership-linux-first.md
```

Acceptance:

1. Docs clearly state that Linux metadata in `crates/world` is authoritative for this slice.
2. Docs describe the two-phase replace window and deterministic recovery rules without widening scope.
3. Docs preserve the `Active`-only downstream proof contract.

Outputs and sentinels:

- worker patch or branch for parent integration from `codex/feat-session-centric-state-store-root-plan-doc-drift`
- `.runs/root-plan-shared-world-replace/sentinels/task-root-b2-doc-drift-correction.ok`

### task-root-c1-integrate-and-validate

Ownership:

- parent only

Lane:

- worktree: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- branch: `feat/session-centric-state-store`

Scope:

1. Integrate accepted worker outputs into the parent checkout.
2. Reject and replace any worker output that violates file ownership or scope.
3. Run the final validation stack in the integrated tree.
4. Record validation outcomes and any accepted deviations from the optimistic path.

Owned surfaces:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- `.runs/root-plan-shared-world-replace/validation-report.md`

Validation stack:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p world -- --nocapture
cargo test -p world-agent -- --nocapture
cargo test -p shell repl_persistent_session_client_fail_closed -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

Acceptance:

1. All required validation commands pass in the integrated tree.
2. The final diff still respects the hard guards and scope lock.
3. `validation-report.md` records what ran and what passed.

Outputs and sentinels:

- `.runs/root-plan-shared-world-replace/validation-report.md`
- `.runs/root-plan-shared-world-replace/sentinels/task-root-c1-integrate-and-validate.ok`

### task-root-c2-closeout

Ownership:

- parent only

Lane:

- worktree: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- branch: `feat/session-centric-state-store`

Scope:

1. Write the final closeout.
2. Record which root-plan requirements were already landed at preflight versus newly completed in this run.
3. Record any rejected worker output, blocked path, or no-op core completion.
4. Confirm all required sentinels are present before ending the session.

Owned surfaces:

- `.runs/root-plan-shared-world-replace/closeout.md`

Acceptance:

1. `closeout.md` accurately captures the final completion status of [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md).
2. The run ends either in a clean green state or an explicitly recorded blocked state.
3. No open question is hidden in worker notes.

Outputs and sentinels:

- `.runs/root-plan-shared-world-replace/closeout.md`
- `.runs/root-plan-shared-world-replace/sentinels/task-root-c2-closeout.ok`

## Parallel Window Rules

1. There is exactly one worker window in this run.
2. That window opens only after `task-root-a2-core-contract-completion` is accepted.
3. If `task-root-a2-core-contract-completion` is a no-op because the core contract is already landed, the worker window opens immediately after the parent records the freeze.
4. Proof and docs may run in parallel because they do not overlap when the core seam is frozen.
5. World inline tests do not become a worker lane because they overlap the parent-owned core files.
6. If proof work discovers a core gap, the worker stops, the parent closes the window, and the run returns to the serialized parent lane.

## Integration / merge rules

1. Workers never merge directly into `feat/session-centric-state-store`.
2. The parent integrates all worker output.
3. Proof work merges before docs whenever proof output changes the final phrasing of the contract.
4. Docs merge last by default so prose reflects the final integrated behavior and validations.
5. Any worker touching forbidden files is rejected unless the parent explicitly reassigns the task and updates `run-state.json`.
6. After each accepted worker integration, the parent reruns the smallest relevant targeted validation before moving to the full final stack.

## Validation stack

Validation is staged, not one-shot:

1. Parent preflight confirms the branch, the scope, and the live code seams.
2. Parent core work runs targeted `world` validation while the core seam is still moving.
3. Proof worker runs targeted `world-agent` and `shell` regressions in its own lane.
4. Parent final integration runs the full command set from [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md).
5. If the delta audit concludes the implementation is already complete, the run still must pass the final validation stack before closeout.

## Context-control / worker-prompt rules

### Parent live context

The parent keeps only these surfaces live as orchestration context:

- [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)
- [ORCH_PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/ORCH_PLAN.md)
- `.runs/root-plan-shared-world-replace/run-state.json`
- `.runs/root-plan-shared-world-replace/tasks.json`
- `.runs/root-plan-shared-world-replace/delta-audit.md`
- `.runs/root-plan-shared-world-replace/core-freeze.md`
- the current integrated diff summary from `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`

### Each worker prompt contains

1. The exact task section from this file.
2. The relevant delta-audit excerpt naming the remaining gap.
3. The exact allowed files and forbidden files.
4. The assigned worktree path and worker branch.
5. The exact commands to run.
6. The acceptance rules and stop conditions.
7. A reminder that the worker must not widen scope or reinterpret the core contract.

### Each worker returns

1. Exact files changed.
2. Commands run and exit status.
3. A short result summary against the task acceptance rules.
4. Blockers or unresolved assumptions.

Worker return rules:

1. Workers do not return full transcripts for the parent to ingest.
2. Workers do not widen scope on their own.
3. Workers do not reinterpret parent-owned requirements from scratch.
4. The parent consumes only the worker patch, file list, command results, and blocker summary.

## Validation / acceptance matrix

| Gate | Owner | Commands | Must be true before proceeding |
| --- | --- | --- | --- |
| Gate A packet freeze | parent | `git status --short`<br>`git rev-parse --abbrev-ref HEAD` | branch is `feat/session-centric-state-store`; run packet exists; `run-state.json` and `tasks.json` are initialized |
| Gate B delta freeze | parent | `rg -n "set_shared_binding_state|persist_metadata|recover_shared_active_from_root|replace_shared_owner_session|shared_owner_mutex" crates/world/src` | `delta-audit.md` classifies each root-plan requirement as `landed`, `partial`, or `missing`; parent-owned target seam is frozen |
| Core completion acceptance | parent | `cargo test -p world -- --nocapture` | remaining `crates/world` delta is landed or explicitly proven already complete; `core-freeze.md` captures the final contract |
| Gate C worker launch | parent | `git rev-parse HEAD` in parent checkout before creating worker worktrees | proof/docs workers are seeded from the frozen parent commit and the late parallel window is safe to open |
| Proof acceptance | worker + parent | `cargo test -p world-agent -- --nocapture`<br>`cargo test -p shell repl_persistent_session_client_fail_closed -- --nocapture`<br>`cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture` | only committed `Active` proof escapes; rollback/retry path does not strand the orchestration session; no forbidden file touches |
| Docs acceptance | worker + parent | `rg -n "shell-authoritative|binding store|replacing|replaced|active" docs/WORLD.md llm-last-mile/PLAN-03.md llm-last-mile/03-shared-world-ownership-linux-first.md` | docs match merged authority and recovery semantics and do not overclaim beyond current code truth |
| Gate D final acceptance | parent | `cargo fmt --all -- --check`<br>`cargo clippy --workspace --all-targets -- -D warnings`<br>`cargo test -p world -- --nocapture`<br>`cargo test -p world-agent -- --nocapture`<br>`cargo test -p shell repl_persistent_session_client_fail_closed -- --nocapture`<br>`cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture` | integrated tree is green, hard guards still hold, docs and proof surfaces match final behavior |

## Blocked-path behavior

1. On any hard-stop condition, the parent writes `.runs/root-plan-shared-world-replace/blocked.json`, updates `run-state.json` to `blocked`, appends `session.log`, and stops dispatch.
2. If the branch already satisfies [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) but cannot be validated because parent-owned surfaces are dirty or the environment is broken, that is a blocked run, not a green closeout.
3. If a worker reports a blocker in a parent-owned seam, the parent either absorbs the task back into the serialized lane or blocks the run. It does not let the worker widen scope.
4. If the only remaining issues are documentation mismatches after code is validated, the run stays open until docs are corrected or explicitly blocked.

## Closeout conditions

The run is complete only when all of these are true:

1. Every requirement in [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) is marked in `closeout.md` as either already landed before the run or completed during the run.
2. The parent-owned core contract is frozen and documented in `core-freeze.md`.
3. Proof regressions are closed or explicitly proven already green by the final validation evidence.
4. Authority docs match the final merged behavior.
5. The full validation stack passes in the integrated tree.
6. All required sentinels exist, `tasks.json` shows no open accepted tasks, and `run-state.json` ends in `completed`.

## Assumptions

1. The execution branch remains `feat/session-centric-state-store`.
2. The parent may create local worktrees and `.runs/root-plan-shared-world-replace/` control artifacts during the run.
3. `cargo`, `rustfmt`, and `clippy` are available in the execution environment.
4. [llm-last-mile/PLAN-07.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-07.md) is a refinement aid, not a replacement for [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md).
5. Any existing implementation already present on the branch should be preserved and validated, not rewritten for plan-text symmetry.
6. If unrelated user changes appear in parent-owned surfaces during the run, the parent pauses and re-audits before editing them.
