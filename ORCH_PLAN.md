# ORCH_PLAN: Shared Dispatch Contract Closeout And Parity Hardening

Authoritative plan source: [PLAN.md](PLAN.md)  
Controller file: [ORCH_PLAN.md](ORCH_PLAN.md)  
Current branch: `feat/gateway-mediated-llm-fulfillment`  
Authoritative checkout: `/home/azureuser/__Active_Code/atomize-hq/substrate`  
Worktree root: `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-s29-5-shared-dispatch-closeout`  
Run id: `slice-29-5-shared-dispatch-closeout-parity-hardening`

## Execution Summary

This file is the execution controller for the current dirty working-tree [PLAN.md](PLAN.md). It is not a summary of the plan. It tells the parent exactly how to run the slice.

Operator facts:

| Item | Value |
| --- | --- |
| Current branch | `feat/gateway-mediated-llm-fulfillment` |
| Parent role | sole integrator, sole gate owner, sole writer of `.runs/**`, sole final acceptance authority |
| Worker model | `GPT-5.4 high` |
| Authoritative checkout | `/home/azureuser/__Active_Code/atomize-hq/substrate` |
| Worker worktree root | `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-s29-5-shared-dispatch-closeout` |
| Concurrency cap | `2` total live worker lanes |
| Exact safe parallel window | only after `G3`: `L4 retained member parity` plus `L4T frozen-surface regression tests` |
| Integration owner | parent only |
| Gate owner | parent only |
| `.runs/**` owner | parent only |

Execution shape:

1. `P0` parent freeze and source lock.
2. `T1/L1` durable attach truth first.
3. `G1` parent integration gate.
4. `T2/L2` policy overlay merge second.
5. `G2` parent integration gate.
6. `T3/L3` capability override closeout third.
7. `G3` parent integration gate and hotspot freeze.
8. `T4/L4` retained member parity plus `T4T/L4T` frozen-surface regression coverage in the only safe parallel window.
9. `G4` parent integration gate for the parallel window.
10. `T5/L5` docs truth sync last.
11. `G5` parent docs gate.
12. `P1` parent validation wall.
13. `G6` parent final acceptance.
14. `P2` parent closeout.

Integration and gate ownership:

| Step | Owner | Output |
| --- | --- | --- |
| `P0` | parent | source lock, lane map, branch map, hotspot freeze |
| `T1` | worker `L1` | Phase 1 code + handoff |
| `G1` | parent | integrated accepted tip for Phase 1 |
| `T2` | worker `L2` | Phase 2 code + handoff |
| `G2` | parent | integrated accepted tip for Phase 2 |
| `T3` | worker `L3` | Phase 3 code + handoff |
| `G3` | parent | integrated accepted tip for Phase 3 and shared-vocabulary freeze |
| `T4` | worker `L4` | retained parity code + parity tests |
| `T4T` | worker `L4T` | frozen-surface regression test additions only |
| `G4` | parent | integrated accepted tip for the only parallel window |
| `T5` | worker `L5` | docs truth sync |
| `G5` | parent | accepted docs tip |
| `P1` | parent | merged-tree validation wall |
| `G6` | parent | final acceptance decision |
| `P2` | parent | final run artifacts and completion sentinels |

## Summary

This run is complete only when the merged tree proves one truthful contract floor across the current slice:

1. durable attach truth is persisted from resolved launch truth;
2. inventory `policy_overlay` is merged into actual `effective_policy`;
3. bounded capability override narrowing works for the approved family and fails closed for all other fields;
4. retained member follow-up turns consume a shared-contract-derived subset instead of a hidden second dialect;
5. docs, tests, and final validation agree on the same shipped semantics.

## Hard Guards

1. The parent is the sole integrator, sole gate owner, sole writer of `.runs/**`, and sole final acceptance authority.
2. Workers must not edit `.runs/**`, `PLAN.md`, or `ORCH_PLAN.md`.
3. The authoritative checkout is already dirty because `PLAN.md` is modified; this run must not assume a clean tree and must not revert or normalize that drift.
4. The authoritative plan text is the current working-tree contents of `PLAN.md`, not the committed copy that may exist in clean worker worktrees.
5. Worker worktrees branch only from committed accepted gate tips; source-lock state stays in the dirty authoritative checkout.
6. `dispatch_contract.rs` is serialized through `T1`, `T2`, and `T3`; after `G3` it is frozen for the remainder of the run.
7. `orchestration_session.rs` is Phase-1-only and frozen after `G1`.
8. `policy_model.rs` and `agent_inventory.rs` are Phase-2-only and frozen after `G2`.
9. `state_store.rs`, `validator.rs`, and `agents_cmd.rs` are Phase-3 surfaces and frozen after `G3`.
10. `async_repl.rs` and additive `world_ops.rs` work belong only to retained parity after `G3`.
11. Docs stay last; runtime semantic work does not continue during the docs lane.
12. Honest parallelism only: two write lanes may overlap only when ownership is disjoint and all prerequisite hotspots are already frozen.
13. Any need to reopen a frozen hotspot after `G3` stops the run and forces re-planning.
14. Parent acceptance is against merged-tree behavior and the authoritative `PLAN.md` acceptance criteria, never lane-local green status alone.

## Blocked-Run Conditions

Stop the run and mark it blocked if any of the following becomes true:

1. `T1` cannot make `HostAttachContract` authoritative without introducing a second durable attach object.
2. `T2` cannot merge `policy_overlay` using shared resolver semantics and instead requires caller-specific patch logic.
3. `T3` cannot implement the approved narrowing-only override family without broadening scope, policy, or unsupported capability fields.
4. `T4` retained parity needs to reopen any frozen `G1` to `G3` hotspot.
5. `T4T` regression coverage needs to touch any frozen runtime hotspot instead of external tests only.
6. Equivalent human and orchestrator cold starts still resolve to different contract truth after `T1` to `T4`.
7. Persisted attach flows still regain permissive defaults from ambient runtime state after `T1` or `T3`.
8. Docs can only be made truthful by contradicting merged runtime behavior.
9. The validation wall cannot be satisfied on the same merged tree as the final docs.
10. `PLAN.md` changes materially after source lock and before `G6`.
11. A worker edits files outside its assigned lane ownership and the parent cannot quarantine the drift cleanly.

Blocked-path discipline:

1. Parent writes `.runs/<run-id>/blocked.json`.
2. Parent creates `.runs/<run-id>/sentinels/RUN_BLOCKED`.
3. Parent writes the current stop reason into the active gate directory under `decision.md`.
4. Parent records accepted lanes, rejected lanes, and unresolved files in `final-summary.md`.
5. Rejected worker branches remain unmerged.

## Fresh Worktree And Branch Protocol

All worker worktrees live under:

- `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-s29-5-shared-dispatch-closeout`

The parent integrates only in:

- `/home/azureuser/__Active_Code/atomize-hq/substrate`

Parent run-open commands:

```bash
RUN_ROOT="/home/azureuser/__Active_Code/atomize-hq/substrate/.runs/slice-29-5-shared-dispatch-closeout-parity-hardening"
WORKTREE_ROOT="/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-s29-5-shared-dispatch-closeout"
REPO_ROOT="/home/azureuser/__Active_Code/atomize-hq/substrate"

mkdir -p "$RUN_ROOT" "$WORKTREE_ROOT"
git -C "$REPO_ROOT" status --short
BASE_HEAD="$(git -C "$REPO_ROOT" rev-parse HEAD)"
```

Cut protocol rules:

1. The parent records `BASE_HEAD` in `branch-map.json`.
2. The parent snapshots dirty `PLAN.md` and current `ORCH_PLAN.md` into `.runs/**/source-lock/` before any worker starts.
3. Every worker worktree is cut from a committed accepted gate tip, never from the dirty authoritative checkout state.
4. The parent records each accepted gate tip SHA in `branch-map.json` before cutting the next lane.
5. The parent merges worker branches back into the authoritative checkout; workers never self-merge.

Lane creation commands:

### `T1/L1` durable attach truth from `BASE_HEAD`

```bash
git -C "$REPO_ROOT" worktree add \
  "$WORKTREE_ROOT/t1-durable-attach-truth" \
  -b codex/feat-gateway-mediated-llm-fulfillment-t1-durable-attach-truth \
  "$BASE_HEAD"
```

### `T2/L2` policy overlay merge from accepted `G1` tip

```bash
ACCEPTED_TIP_G1="$(git -C "$REPO_ROOT" rev-parse HEAD)"

git -C "$REPO_ROOT" worktree add \
  "$WORKTREE_ROOT/t2-policy-overlay-merge" \
  -b codex/feat-gateway-mediated-llm-fulfillment-t2-policy-overlay-merge \
  "$ACCEPTED_TIP_G1"
```

### `T3/L3` capability override closeout from accepted `G2` tip

```bash
ACCEPTED_TIP_G2="$(git -C "$REPO_ROOT" rev-parse HEAD)"

git -C "$REPO_ROOT" worktree add \
  "$WORKTREE_ROOT/t3-capability-override-closeout" \
  -b codex/feat-gateway-mediated-llm-fulfillment-t3-capability-override-closeout \
  "$ACCEPTED_TIP_G2"
```

### `T4/L4` retained member parity and `T4T/L4T` frozen-surface regression lane from accepted `G3` tip

```bash
ACCEPTED_TIP_G3="$(git -C "$REPO_ROOT" rev-parse HEAD)"

git -C "$REPO_ROOT" worktree add \
  "$WORKTREE_ROOT/t4-retained-member-parity" \
  -b codex/feat-gateway-mediated-llm-fulfillment-t4-retained-member-parity \
  "$ACCEPTED_TIP_G3"

git -C "$REPO_ROOT" worktree add \
  "$WORKTREE_ROOT/t4t-frozen-surface-regression-tests" \
  -b codex/feat-gateway-mediated-llm-fulfillment-t4t-frozen-surface-regression-tests \
  "$ACCEPTED_TIP_G3"
```

### `T5/L5` docs truth sync from accepted `G4` tip

```bash
ACCEPTED_TIP_G4="$(git -C "$REPO_ROOT" rev-parse HEAD)"

git -C "$REPO_ROOT" worktree add \
  "$WORKTREE_ROOT/t5-docs-truth-sync" \
  -b codex/feat-gateway-mediated-llm-fulfillment-t5-docs-truth-sync \
  "$ACCEPTED_TIP_G4"
```

## Parent-Owned Run-State Surface

Canonical run root:

- `/home/azureuser/__Active_Code/atomize-hq/substrate/.runs/slice-29-5-shared-dispatch-closeout-parity-hardening/`

Canonical artifact layout:

```text
.runs/slice-29-5-shared-dispatch-closeout-parity-hardening/
  run-state.json
  queue.json
  session-log.md
  branch-map.json
  lane-ownership.json
  hotspot-freeze.json
  merge-order.json
  validation-wall.md
  final-summary.md
  blocked.json                       # blocked runs only
  source-lock/
    PLAN.authoritative.md
    ORCH_PLAN.authoritative.md
    git-status.txt
    sha256.json
  sentinels/
    RUN_OPEN
    RUN_BLOCKED                      # blocked runs only
    RUN_COMPLETE                     # successful runs only
  tasks/
    P0-parent-freeze/
    T1-durable-attach-truth/
    G1-phase1-accept/
    T2-policy-overlay-merge/
    G2-phase2-accept/
    T3-capability-override-closeout/
    G3-phase3-accept/
    T4-retained-member-parity/
    T4T-frozen-surface-regression-tests/
    G4-parallel-window-accept/
    T5-docs-truth-sync/
    G5-docs-accept/
    P1-validation-wall/
    G6-final-accept/
    P2-parent-closeout/
  gates/
    G0-run-open/
    G1-phase1-accept/
    G2-phase2-accept/
    G3-phase3-accept/
    G4-parallel-window-accept/
    G5-docs-accept/
    G6-final-accept/
```

Parent-only top-level files:

| File | Purpose |
| --- | --- |
| `run-state.json` | current gate, current task, run status, active lane count, accepted tip SHA |
| `queue.json` | ordered task queue, dependencies, ready/running/accepted/blocked states |
| `session-log.md` | chronological operator log of worker launches, merges, blockers, and gate decisions |
| `branch-map.json` | branch names, worktree paths, cut SHAs, merge SHAs, acceptance SHAs |
| `lane-ownership.json` | file ownership map and forbidden surfaces per lane |
| `hotspot-freeze.json` | hotspot files, freeze gate, reopen policy |
| `merge-order.json` | required merge order and actual merge order |
| `validation-wall.md` | final merged-tree proof record |
| `final-summary.md` | completion or blocked summary for the whole run |
| `blocked.json` | machine-readable stop artifact when blocked |

Required `source-lock/` files:

1. `PLAN.authoritative.md`
2. `ORCH_PLAN.authoritative.md`
3. `git-status.txt`
4. `sha256.json`

Required sentinel semantics:

1. `RUN_OPEN` exists after `G0`.
2. `RUN_BLOCKED` exists only when the run is stopped.
3. `RUN_COMPLETE` exists only after `G6` and `P2`.

Every task directory must contain:

1. `task.json`
2. `scope.md`
3. `owned-files.txt`
4. `forbidden-surfaces.txt`
5. `commands.txt`
6. `acceptance.md`
7. `status.json`
8. `handoff.md` when work has been returned

Every gate directory must contain:

1. `gate.json`
2. `checklist.md`
3. `diff-summary.md`
4. `decision.md`
5. `next-step.md`
6. `status.json`

Parent-only `.runs/**` rule:

1. Workers do not write these artifacts directly.
2. Workers return structured handoffs to the parent.
3. The parent writes the canonical record into `.runs/**`.
4. Full worker transcripts are not the source of truth; `.runs/**` artifacts are.

## Hotspot Ownership And Freeze Map

| Surface | Owner task | Freeze gate | Reopen policy |
| --- | --- | --- | --- |
| `crates/shell/src/execution/agent_runtime/orchestration_session.rs` | `T1` | `G1` | forbidden |
| `crates/shell/src/execution/agent_runtime/dispatch_contract.rs` | `T1`, `T2`, `T3` | `G3` | forbidden after `G3` |
| `crates/shell/src/execution/agents_cmd.rs` | `T1`, `T3` | `G3` | forbidden after `G3` |
| `crates/shell/src/execution/policy_model.rs` | `T2` | `G2` | forbidden |
| `crates/shell/src/execution/agent_inventory.rs` | `T2` | `G2` | forbidden |
| `crates/shell/src/execution/agent_runtime/state_store.rs` | `T3` | `G3` | forbidden after `G3` |
| `crates/shell/src/execution/agent_runtime/validator.rs` | `T3` | `G3` | forbidden after `G3` |
| `crates/shell/src/repl/async_repl.rs` | `T4` | `G4` | forbidden after `G4` |
| `crates/shell/src/execution/routing/dispatch/world_ops.rs` | `T4` if needed | `G4` | forbidden after `G4` |
| `crates/shell/tests/repl_world_first_routing_v1.rs` | `T4` | `G4` | forbidden after `G4` |
| `crates/shell/tests/agent_public_control_surface_v1.rs` | `T4T` | `G4` | test-only lane |
| `crates/shell/tests/agent_successor_contract_ahcsitc0.rs` | `T4T` | `G4` | test-only lane |
| `llm-last-mile/29*.md`, `30*.md`, `31*.md` | `T5` | `G5` | docs only |

Freeze rule:

1. If any later lane needs a frozen runtime hotspot, stop the run.
2. Do not silently shift frozen work into a later lane.
3. The safe parallel window exists only because `T4T` does not own runtime hotspots.

## Dependency And Parallel-Lanes Table

| Task | Description | Depends on | Safe to run in parallel with |
| --- | --- | --- | --- |
| `P0` | parent freeze and source lock | - | none |
| `T1` | durable attach truth | `P0`, `G0` | none |
| `T2` | policy overlay merge | `G1` | none |
| `T3` | capability override closeout | `G2` | none |
| `T4` | retained member parity | `G3` | `T4T` only |
| `T4T` | frozen-surface regression tests | `G3` | `T4` only |
| `T5` | docs truth sync | `G4` | none |
| `P1` | validation wall | `G5` | none |
| `P2` | closeout | `G6` | none |

Parallel-lane rationale:

1. `T1`, `T2`, and `T3` are serialized because they share the contract and durable-state hotspots.
2. `T4` is safe only after `G3` freezes shared contract vocabulary and runtime hotspots.
3. `T4T` is safe only because it is limited to external regression tests that validate already-frozen behavior.
4. If `T4T` needs any frozen runtime hotspot, the parallel window closes immediately and the parent absorbs follow-up work after `G4`.

## Task-Level Orchestration

### `P0` Parent Freeze

Goal:

1. lock the dirty working-tree `PLAN.md` as authoritative;
2. lock this controller version;
3. publish lane ownership, hotspot freeze, branch protocol, and queue state.

Required parent actions:

1. snapshot `PLAN.md`, `ORCH_PLAN.md`, and `git status --short` into `.runs/**/source-lock/`;
2. initialize `queue.json`, `branch-map.json`, `lane-ownership.json`, `hotspot-freeze.json`, and `merge-order.json`;
3. create all task and gate directories;
4. set `run-state.json` to `status=open-pending-g0`.

Acceptance:

1. source lock exists;
2. dirty-tree state is recorded;
3. workers can be launched without relying on mutable ambient context.

### `G0` Run-Open Gate

Parent verifies:

1. authoritative branch is `feat/gateway-mediated-llm-fulfillment`;
2. current dirty `PLAN.md` snapshot is captured;
3. worker ownership and forbidden surfaces are recorded;
4. `RUN_OPEN` sentinel can be created safely.

Advance only if:

1. no required run artifacts are missing;
2. there is no unresolved ambiguity about the ordered execution shape.

### `T1 / L1` Durable Attach Truth First

Scope:

1. derive `HostAttachContract` from resolved host launch truth;
2. persist attach-relevant capabilities and attach knobs from resolved truth;
3. keep sync continuity-only;
4. keep successor copy as generalized truth plus cleared continuity only.

Owned files:

1. `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
2. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
3. `crates/shell/src/execution/agents_cmd.rs`

Forbidden surfaces:

1. `.runs/**`
2. `PLAN.md`
3. `ORCH_PLAN.md`
4. `policy_model.rs`
5. `agent_inventory.rs`
6. `state_store.rs`
7. `validator.rs`
8. `async_repl.rs`
9. `world_ops.rs`
10. docs

Required commands/tests:

```bash
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell agent_successor_contract_ahcsitc0 -- --nocapture
```

Acceptance criteria:

1. birth-time host attach truth comes from resolved contract truth;
2. persisted attach planning no longer reconstructs permissive baseline truth from ambient participant state;
3. successor copy preserves generalized truth and clears only continuity-specific state;
4. handoff includes exact files changed, tests run, tests not run, and unresolved concerns.

### `G1` Phase 1 Gate

Parent verifies:

1. only owned files changed;
2. `HostAttachContract` remains the only durable attach object;
3. no ambient-state fallback path survived in the touched attach flow;
4. committed accepted tip is recorded in `branch-map.json`.

Advance only if:

1. durable attach schema is frozen enough for later phases;
2. no `T2` or `T3` assumptions are required to explain `T1`.

### `T2 / L2` Policy Overlay Merge Second

Scope:

1. expose or reuse one shared policy patch helper;
2. apply validated inventory `policy_overlay` into `ResolvedLaunchContract.effective_policy`;
3. return exact policy diagnostics and provenance.

Owned files:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/policy_model.rs`
3. `crates/shell/src/execution/agent_inventory.rs`

Forbidden surfaces:

1. `.runs/**`
2. `PLAN.md`
3. `ORCH_PLAN.md`
4. `orchestration_session.rs`
5. `state_store.rs`
6. `validator.rs`
7. `agents_cmd.rs`
8. `async_repl.rs`
9. `world_ops.rs`
10. docs

Required commands/tests:

```bash
cargo test -p shell dispatch_contract -- --nocapture
```

Acceptance criteria:

1. overlay-backed resolution returns materially narrower `effective_policy` where appropriate;
2. overlay-free resolution stays unchanged;
3. diagnostics explain overlay acceptance or denial at the policy layer;
4. no parallel policy semantics are introduced outside the shared resolver path.

### `G2` Phase 2 Gate

Parent verifies:

1. only owned files changed;
2. one merge semantics source is used;
3. overlay behavior remains narrowing-only;
4. accepted `G2` tip SHA is recorded before cutting `T3`.

Advance only if:

1. policy truth is frozen enough to support capability closeout and later parity;
2. no caller-specific overlay behavior survives.

### `T3 / L3` Capability Override Closeout Third

Scope:

1. implement field-by-field override handling;
2. support only narrowing-only overrides for:
   `session_resume`, `session_fork`, `session_stop`, `status_snapshot`, `event_stream`;
3. keep `session_start`, `llm`, and `mcp_client` rejected with field-scoped fail-closed diagnostics;
4. keep persisted attach launches rejecting dispatch-time capability overrides;
5. ensure persisted narrowed truth drives later state-store gates.

Owned files:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
3. `crates/shell/src/execution/agent_runtime/validator.rs`
4. `crates/shell/src/execution/agents_cmd.rs`

Forbidden surfaces:

1. `.runs/**`
2. `PLAN.md`
3. `ORCH_PLAN.md`
4. `orchestration_session.rs`
5. `policy_model.rs`
6. `agent_inventory.rs`
7. `async_repl.rs`
8. `world_ops.rs`
9. docs

Required commands/tests:

```bash
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell agent_public_control_surface_v1 -- --nocapture
```

Acceptance criteria:

1. approved override family works as narrowing-only;
2. unsupported fields fail closed with exact field names and bounded reasons;
3. later attach and control paths observe persisted narrowed truth;
4. shared contract vocabulary is frozen enough for retained parity.

### `G3` Phase 3 Gate

Parent verifies:

1. only owned files changed;
2. approved support matrix matches the authoritative plan;
3. denial wording is field-scoped and truthful;
4. `dispatch_contract.rs`, `state_store.rs`, `validator.rs`, and `agents_cmd.rs` can be frozen;
5. accepted `G3` tip SHA is recorded before cutting the parallel window.

Advance only if:

1. shared contract vocabulary is stable;
2. persisted narrowed truth is already real;
3. any need to reopen these hotspots would be a blocker.

### `T4 / L4` Retained Member Parity

Scope:

1. build the shared-contract-derived retained-turn subset;
2. feed typed transport from that subset;
3. keep retained follow-up turns off inventory and config re-resolution;
4. own parity-specific integration coverage.

Owned files:

1. `crates/shell/src/repl/async_repl.rs`
2. `crates/shell/src/execution/routing/dispatch/world_ops.rs` if additive wiring is required
3. `crates/shell/tests/repl_world_first_routing_v1.rs`

Forbidden surfaces:

1. `.runs/**`
2. `PLAN.md`
3. `ORCH_PLAN.md`
4. all frozen `G1` to `G3` runtime hotspots
5. docs

Required commands/tests:

```bash
cargo test -p shell repl_world_first_routing_v1 -- --nocapture
```

Acceptance criteria:

1. retained follow-up turns no longer own launch selection semantics;
2. retained follow-up turns use shared-contract-derived subset truth;
3. typed transport remains intact;
4. no frozen hotspot was reopened.

### `T4T / L4T` Frozen-Surface Regression Tests

Scope:

1. add or extend tests for durable attach truth, overlay merge, override persistence, and successor truth on already-frozen behavior;
2. validate the merged contract floor from outside the frozen runtime hotspots.

Owned files:

1. `crates/shell/tests/agent_public_control_surface_v1.rs`
2. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
3. additional external shell test files only if the parent pre-approves them in `lane-ownership.json`

Forbidden surfaces:

1. every frozen runtime hotspot from `G1` to `G3`
2. `async_repl.rs`
3. `world_ops.rs`
4. docs
5. `.runs/**`
6. `PLAN.md`
7. `ORCH_PLAN.md`

Why this lane is safe:

1. it branches from the accepted `G3` tip;
2. it touches only external regression test files;
3. it validates already-frozen semantics rather than defining new runtime behavior.

If this lane needs a frozen runtime hotspot:

1. stop the lane immediately;
2. mark the issue in `G4` as a blocker;
3. collapse the work back to parent-owned post-merge follow-up or re-plan the run.

Required commands/tests:

```bash
cargo test -p shell agent_public_control_surface_v1 -- --nocapture
cargo test -p shell agent_successor_contract_ahcsitc0 -- --nocapture
```

Acceptance criteria:

1. only approved test files changed;
2. no frozen runtime hotspot changed;
3. test coverage closes Phase-1-to-3 proof gaps without mutating frozen semantics.

### `G4` Parallel Window Gate

Parent verifies:

1. `T4` and `T4T` stayed inside disjoint ownership boundaries;
2. `T4` did not reopen frozen `G1` to `G3` hotspots;
3. `T4T` touched only approved external test files;
4. retained parity lands first if merge order matters;
5. compatible regression tests are replayed or rebased onto the accepted `G4` tip if necessary.

Advance only if:

1. the only safe parallel window remained honest;
2. retained parity truth and frozen-surface regression coverage can coexist on one accepted tree.

### `T5 / L5` Docs Truth Sync Last

Scope:

1. update slice docs so 29, 29.5, 30, and 31 agree on shipped semantics;
2. update nearby runtime comments or ASCII diagrams only if merged code made them stale;
3. do not change runtime semantics in this lane.

Owned files:

1. `llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md`
2. `llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md`
3. `llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md`
4. nearby comments or diagrams only if parent explicitly authorizes them

Forbidden surfaces:

1. all runtime files
2. `.runs/**`
3. `PLAN.md`
4. `ORCH_PLAN.md`

Required checks:

1. manual cross-check against accepted merged runtime semantics;
2. no runtime file drift.

Acceptance criteria:

1. docs describe shipped semantics, not aspirational semantics;
2. downstream slices can cite 29.5 as the truthful floor;
3. no runtime hotspot is reopened.

### `G5` Docs Gate

Parent verifies:

1. only docs or explicitly approved nearby comment surfaces changed;
2. docs match the merged runtime behavior from `G4`;
3. there is no new semantic claim unsupported by tests or code.

Advance only if:

1. docs are last and truthful;
2. the validation wall can run on the exact merged tree.

### `P1` Parent Validation Wall

Goal:

1. prove the authoritative `PLAN.md` acceptance criteria on the merged tree;
2. record commands, results, and manual proof notes in `validation-wall.md`.

Required commands:

```bash
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell agent_public_control_surface_v1 -- --nocapture
cargo test -p shell repl_world_first_routing_v1 -- --nocapture
cargo test -p shell agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell -- --nocapture
cargo clippy -p shell --all-targets -- -D warnings
cargo fmt --all -- --check
```

Parent proof record must include:

1. command results;
2. manual checks;
3. any skipped coverage with reason;
4. exact merged commit SHA used for validation.

### `G6` Final Acceptance Gate

Parent verifies the merged tree against the final acceptance matrix below and then either:

1. accepts the run and creates `RUN_COMPLETE`; or
2. blocks the run and creates `RUN_BLOCKED`.

### `P2` Parent Closeout

Parent actions:

1. finalize `run-state.json`, `queue.json`, `session-log.md`, `validation-wall.md`, and `final-summary.md`;
2. record accepted merge SHAs and final merged tip SHA;
3. create `RUN_COMPLETE` only if `G6` passed;
4. leave rejected or blocked worker branches unmerged.

## Context-Control Rules

Parent live context:

1. authoritative dirty `PLAN.md` snapshot;
2. current controller snapshot;
3. current accepted gate tip SHA;
4. lane ownership map;
5. hotspot freeze map;
6. queue state and current blocker state.

Every worker prompt should contain:

1. task ID and lane ID;
2. goal and acceptance criteria;
3. owned files;
4. forbidden surfaces;
5. authoritative source-lock reminder;
6. required tests or commands;
7. explicit stop conditions.

Every worker must return:

1. files changed;
2. tests run;
3. tests not run;
4. assumptions made;
5. blockers encountered;
6. any evidence that the frozen gate assumptions are no longer valid.

Context discipline:

1. Workers get only the context needed for their lane.
2. Workers treat all frozen hotspots outside their ownership as read-only.
3. Workers stop on unexpected pre-existing edits in owned files.
4. Workers do not broaden scope, normalize unrelated formatting, or rewrite docs unless assigned.
5. Full worker transcripts are not the run ledger; parent-authored `.runs/**` artifacts are.

## Validation Wall And Final Acceptance Matrix

The merged tree must satisfy all of the following before the run is complete.

| Final check | Mapped PLAN requirement |
| --- | --- |
| `FA1` host-scoped resolved launch truth persists into `HostAttachContract` | Acceptance criteria 1 |
| `FA2` persisted attach resolution reuses persisted capabilities and attach knobs instead of permissive defaults | Acceptance criteria 2 |
| `FA3` inventory-backed resolution merges validated `policy_overlay` into `effective_policy` as narrowing truth | Acceptance criteria 3 |
| `FA4` approved capability override family works as narrowing-only; unsupported fields fail closed with exact diagnostics | Acceptance criteria 4 |
| `FA5` retained member turns consume a shared-contract-derived subset and do not reconstruct hidden launch semantics | Acceptance criteria 5 |
| `FA6` equivalent human and orchestrator cold starts yield equivalent contract truth for backend, scope, capabilities, attach knobs, and policy | Acceptance criteria 6 |
| `FA7` merged contract floor is sufficient for slices 30 and 31 without inventing a second override or attach-truth model | Acceptance criteria 7 and 8 |
| `FA8` targeted shell tests, full `cargo test -p shell`, `clippy`, and `fmt --check` are green on the merged tree | Definition of done 8 |
| `FA9` docs for 29, 29.5, 30, and 31 match shipped semantics on the merged tree | Definition of done 9 |

Final acceptance checklist:

1. `FA1` through `FA9` are all satisfied on one merged tree.
2. No frozen hotspot was reopened after `G3`.
3. The only safe parallel window remained within disjoint ownership.
4. `PLAN.md` source lock remained authoritative for the duration of the run.
5. `validation-wall.md` and `final-summary.md` both point at the same merged tip SHA.

## Assumptions

1. The current working-tree rewrite of `PLAN.md` is authoritative for this run.
2. Parent integration remains on `feat/gateway-mediated-llm-fulfillment`.
3. Worker worktrees can be created under `/home/azureuser/__Active_Code/atomize-hq/.worktrees/`.
4. The slice stays limited to shared dispatch contract closeout and parity hardening.
5. There is no public scope expansion or lazy-attach product work in this run.
6. Parent can preserve the dirty authoritative checkout while cutting clean worker worktrees from committed gate tips.

## Completion Behavior

1. Parent merges accepted lanes in gate order only.
2. Parent runs the validation wall only after docs are accepted.
3. Parent marks the run complete only after `G6`.
4. If blocked, parent writes `blocked.json`, leaves rejected worker branches unmerged, and records the exact stop condition in both `session-log.md` and `final-summary.md`.
5. Workers never self-certify completion, never write `.runs/**`, and never own final acceptance.

## Completion Target

The target outcome is one merged tree where durable attach truth, overlay truth, override truth, retained member parity, and downstream docs all agree, with no hidden second contract dialect left for slices 30 or 31 to rediscover.
