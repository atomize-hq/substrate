# ORCH_PLAN-17: Execute PLAN-17 Through A Parent-Frozen Selected-Follow-Up Contract And One Honest Serialized Shell Lane

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN-17.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-17.md)  
Style reference: [ORCH_PLAN-15.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-15.md)  
Style reference: [ORCH_PLAN-16.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-16.md)  
Structure reference: [M26 Orchestration Plan](/Users/spensermcconnell/__Active_Code/atomize-hq/spec/docs/m26_orchestration_kickoff_prompt.md)  
Gap matrix anchor: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Execution type: fresh orchestration plan, Linux-first, REPL-targeted selected-member submit/reuse contract hardening, shell-centric helper extraction plus regression-floor completion, parent-frozen scope, parent-only integration and gates  
Worker model: GPT-5.4 workers with `reasoning_effort=high`  
Max concurrent code workers before integration: `1`

## Summary

This document is the execution controller for `PLAN-17`, not a restatement of it.

The current repo truth is narrower than `PLAN-17`'s initial parallelization sketch:

1. targeted follow-up submit already exists
2. exact backend-id selection already exists
3. retained world-member coexistence by distinct backend id already exists from `PLAN-16`
4. the real remaining work is contract centralization in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) plus regression-floor completion and repo-truth docs

That means this run is intentionally serialized:

1. `task/m17-p1-parent-contract-freeze-and-run-init` is parent-only and freezes the selected-follow-up contract, stop conditions, lane ownership, merge order, and validation wall.
2. `task/m17-g1-shell-lane-launch-gate` is parent-only and is the only gate that may launch the shell lane.
3. `task/m17-l1-shell-contract-centralization-and-regression-floor` is the single implementation lane. It owns the helper extraction, any selector-level pinning needed by that seam, and the REPL integration test floor in one coherent file set.
4. `task/m17-g2-shell-lane-integration-gate` and `task/m17-p2-parent-shell-lane-integration` are parent-only.
5. `task/m17-l2-docs-gap-matrix-closeout` starts only after `p2` lands merged code truth.
6. `task/m17-g4-validation-wall-gate` and `task/m17-p3-parent-validation-wall-and-closeout` are parent-only and finish the run.

Canonical task IDs:

- `task/m17-p1-parent-contract-freeze-and-run-init`
- `task/m17-g1-shell-lane-launch-gate`
- `task/m17-l1-shell-contract-centralization-and-regression-floor`
- `task/m17-g2-shell-lane-integration-gate`
- `task/m17-p2-parent-shell-lane-integration`
- `task/m17-g3-doc-closeout-launch-gate`
- `task/m17-l2-docs-gap-matrix-closeout`
- `task/m17-g4-validation-wall-gate`
- `task/m17-p3-parent-validation-wall-and-closeout`

## Assumptions

1. [PLAN-17.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-17.md) remains the authoritative dependency graph for this run.
2. The authoritative integration checkout remains the current workspace checkout on `feat/session-centric-state-store`.
3. `PLAN-16` already landed backend-id-scoped retained-member coexistence, so this run must not reopen `world-agent` cardinality or reintroduce singleton logic.
4. The remaining production hotspot is [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs); the remaining proof hotspot is [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) plus its stub support.
5. Linux is the only world-target platform that must go green in this run; non-Linux world follow-up behavior remains fail-closed.
6. Repo-truth docs must move last, after validated merged behavior exists.

## Immutable Run Shape

### Frozen contract truth

These are run-stopping invariants, not preferences:

1. `::<backend_id> <prompt>` remains the only targeted follow-up grammar.
2. Plain REPL input remains shell execution.
3. `validate_exact_backend_selection(...)` remains the canonical explicit selector.
4. Host follow-up turns may target only the active orchestrator backend for the current REPL session.
5. Linux world follow-up turns continue to submit through `MemberTurnSubmitRequestV1` and `/v1/member_turn/stream`.
6. World-member reuse remains keyed by the exact retained backend slot for the current orchestration session and authoritative `world_generation`.
7. Relaunch remains a readiness concern inside `ensure_member_runtime_ready_for_descriptor(...)`, not a submit concern.
8. Duplicate retained members for the same backend slot remain fail-closed. This run does not reopen `PLAN-16`.
9. `substrate -c` remains shell wrap mode.
10. This run may not widen into status/toolbox redesign, non-REPL caller surfaces, public control-plane productization, or macOS parity.
11. The parent agent is the only integrator and the only approval authority for worker outputs.

### Parent-only versus worker-owned authority

Parent-only for the full run:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-17/*`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m17-*/**`
- worker launch, output classification, merge approval, quarantine, retry approval, and final validation
- any decision to stop the run instead of inventing cross-lane or cross-scope semantics

Parent-only during `p1`, then frozen by artifact:

- selected-follow-up failure taxonomy
- helper-seam ownership contract
- lane ownership map
- merge order
- validation-wall command list
- stop conditions

Worker-owned after `p1`:

- `L1`: shell helper extraction, selector-dependency pinning, REPL integration tests, stub support
- `L2`: docs and planning-index closeout only

No worker may edit parent-owned run-state. No worker may edit another lane's owned files. No worker may reopen the frozen scope boundary.

### Stop conditions

Stop the run, write `.runs/plan-17/blocked.json`, and do not advance if any of these occur:

1. The selected-follow-up contract cannot be frozen without redesigning grammar, caller surface, or transport shape.
2. Any implementation requires edits in [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs) to rework retained-member cardinality rather than merely validating current behavior.
3. Any worker broadens scope into `state_store.rs`, `agents_cmd.rs`, CLI command-mode semantics, toolbox surfaces, or macOS/Lima parity.
4. Any worker changes the `::<backend_id> <prompt>` grammar.
5. Any worker attempts to add or imply a new non-REPL caller surface.
6. Any worker edits another lane's owned files or any parent-owned run-state.
7. `L2` starts before `p2` accepts final merged code truth.
8. The parent would need to invent hybrid semantics during integration to make test expectations or doc truth fit behavior that the merged code does not actually implement.
9. The validation wall cannot prove helper centralization, positive host follow-up proof, stale-or-missing world relaunch proof, unchanged exact selector behavior, and unchanged fail-closed non-Linux behavior.
10. The gap matrix is marked closed beyond what the integrated tests and validation wall prove.

## Fresh Worktrees And Branches

Fresh worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-17`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-17/shell-contract-centralization-and-regression-floor`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-17/docs-gap-matrix-closeout`

Worker branches:

- `codex/feat-session-centric-state-store-m17-shell-contract-centralization-and-regression-floor`
- `codex/feat-session-centric-state-store-m17-docs-gap-matrix-closeout`

Exact setup commands:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-17
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-17/shell-contract-centralization-and-regression-floor -b codex/feat-session-centric-state-store-m17-shell-contract-centralization-and-regression-floor feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-17/docs-gap-matrix-closeout -b codex/feat-session-centric-state-store-m17-docs-gap-matrix-closeout feat/session-centric-state-store
```

### Parent integration surface

The parent integrates on the authoritative checkout already on `feat/session-centric-state-store`.

No separate parent integration worktree is introduced because:

1. the run has one serialized integrator
2. `.runs/plan-17/*` is parent-owned state and should stay co-located with the authoritative branch context
3. there is no honest concurrent parent merge activity in this plan

## Parent-Owned Run-State Surface

Canonical parent-owned state under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-17/`:

- `run-state.json`
- `tasks.json`
- `session-log.md`
- `contract-freeze.json`
- `lane-ownership.json`
- `merge-order.json`
- `validation-wall.md`
- `blocked.json` on failure only
- `quarantine/`
- `sentinels/`

`contract-freeze.json` is the single source of truth for:

- `plan_id: "PLAN-17"`
- `plan_source: "llm-last-mile/PLAN-17.md"`
- `orchestration_plan_source: "llm-last-mile/ORCH_PLAN-17.md"`
- `branch: "feat/session-centric-state-store"`
- `selector_authority: "validate_exact_backend_selection"`
- `targeted_grammar: "::<backend_id> <prompt>"`
- `host_target_rule: "active_orchestrator_backend_only"`
- `world_target_rule: "linux_first_member_turn_submit_only"`
- `reuse_rule: "exact_backend_slot_for_current_world_generation"`
- `relaunch_rule: "readiness_only_when_missing_invalidated_or_stale"`
- `plan_16_cardinality_reopened: false`
- `new_caller_surface_allowed: false`
- `validation_commands`
- `stop_conditions`

`lane-ownership.json` is the single source of truth for:

- task ID
- owner
- worktree path
- branch
- allowed files
- forbidden files
- command gates
- retry budget
- merge order
- sentinel filename

`run-state.json` is the single source of truth for:

- `current_phase`
- `active_task_ids`
- `worker_cap`
- `contract_freeze_status`
- `lane_status`
- `accepted_outputs`
- `rejected_outputs`
- `quarantined_outputs`
- `blocked_outputs`
- `retry_budget_by_lane`
- `attempt_counts`
- `integration_order`
- `gate_status`
- `validation_wall_status`
- `termination_reason`
- `terminal_state`

### Kickoff initialization order

The parent initializes the run in this exact order before any worker prompt is written:

1. Create `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-17/`, `.runs/plan-17/sentinels/`, `.runs/plan-17/quarantine/`, and every `.runs/task-m17-*/` directory.
2. Write `tasks.json` as the canonical queue and ledger for the entire run.
3. Write `run-state.json` with:
   - `current_phase: "kickoff"`
   - `worker_cap: 1`
   - every task id in `pending`
   - empty accepted/rejected/quarantined output arrays
4. Write `contract-freeze.json`, `lane-ownership.json`, `merge-order.json`, and `validation-wall.md`.
5. Write `session-log.md` with the kickoff timestamp, authoritative branch, worktree roots, and the explicit statement that max honest worker concurrency is `1`.
6. Write each per-task `task.json`, `commands.txt`, and `summary.md` stub before launch packets are generated.
7. Seed worker worktrees only after the above artifacts exist and `task-m17-p1-parent-contract-freeze-and-run-init` is marked complete.

`tasks.json` is both the launch queue and the execution ledger. Each row must include at least:

- `task_id`
- `title`
- `owner`
- `status`
- `phase_class`
- `depends_on`
- `worktree_path` when applicable
- `branch` when applicable
- `allowed_files`
- `forbidden_files`
- `command_gates`
- `acceptance_checks`
- `sentinel_path`
- `artifact_dir`

Gate-proof artifacts are parent-owned and mandatory:

- each gate task directory contains `gate-checklist.md`
- each gate task directory contains `gate-result.json`
- each gate result records:
  - checked prerequisites
  - artifact paths inspected
  - pass/fail classification
  - approval timestamp
  - approver: `parent`

Required sentinels:

- `.runs/plan-17/sentinels/task-m17-p1-parent-contract-freeze-and-run-init.ok`
- `.runs/plan-17/sentinels/task-m17-g1-shell-lane-launch-gate.ok`
- `.runs/plan-17/sentinels/task-m17-l1-shell-contract-centralization-and-regression-floor.ok`
- `.runs/plan-17/sentinels/task-m17-g2-shell-lane-integration-gate.ok`
- `.runs/plan-17/sentinels/task-m17-p2-parent-shell-lane-integration.ok`
- `.runs/plan-17/sentinels/task-m17-g3-doc-closeout-launch-gate.ok`
- `.runs/plan-17/sentinels/task-m17-l2-docs-gap-matrix-closeout.ok`
- `.runs/plan-17/sentinels/task-m17-g4-validation-wall-gate.ok`
- `.runs/plan-17/sentinels/task-m17-p3-parent-validation-wall-and-closeout.ok`

Per-task artifact directories:

- `.runs/task-m17-p1-parent-contract-freeze-and-run-init/`
- `.runs/task-m17-g1-shell-lane-launch-gate/`
- `.runs/task-m17-l1-shell-contract-centralization-and-regression-floor/`
- `.runs/task-m17-g2-shell-lane-integration-gate/`
- `.runs/task-m17-p2-parent-shell-lane-integration/`
- `.runs/task-m17-g3-doc-closeout-launch-gate/`
- `.runs/task-m17-l2-docs-gap-matrix-closeout/`
- `.runs/task-m17-g4-validation-wall-gate/`
- `.runs/task-m17-p3-parent-validation-wall-and-closeout/`

Each task directory contains:

- `task.json`
- `summary.md`
- `commands.txt`
- `artifacts/`
- `evidence-manifest.json`

Each gate task directory also contains:

- `gate-checklist.md`
- `gate-result.json`

Each worker task directory also contains:

- `worker-output.patch`
- `worker-report.md`
- `touched-files.txt`
- `command-results.md`
- `rejected.json` on rejection only
- `blocked.json` on blocked lane return only
- `quarantine/` when the parent quarantines the output

Blocked termination minimum contents:

1. task or gate where execution stopped
2. classification: `rejected`, `blocked`, or `merge_refused`
3. exact contract clause or ownership rule that stopped the run
4. whether retry remained available
5. artifact paths for patch, report, command output, and semantic-drift evidence
6. explicit statement that no blocked output was integrated

## Phase Graph And Concurrency

Parent checkout:

- current authoritative checkout on `feat/session-centric-state-store`

Concurrency rules:

1. Worker cap is `1` for the entire run.
2. `p1` must finish before any worker starts.
3. `g1` must accept before `L1` starts.
4. `g2` classifies `L1` before any docs work starts.
5. `p2` integrates `L1` on the authoritative checkout.
6. `g3` starts only after `p2` is green.
7. `L2` runs alone on the exact post-`p2` tree.
8. `g4` and `p3` are parent-only.
9. No parallel worker window is allowed because the real remaining scope converges on one shared shell/test seam and docs must encode merged, validated truth rather than lane-local guesses.

### Why this run is serialized

1. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) is the production hotspot for contract centralization and the only honest place to own any follow-on readiness clarification.
2. [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) and [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs) depend on the exact merged helper behavior, so splitting them into a concurrent lane would create patch ping-pong rather than speed.
3. Docs can only close after code and tests agree on the same narrowed contract.
4. The earlier `PLAN-17` “two parallel implementation lanes” sketch is not honest against the current repo truth and should not drive execution.

## Kickoff And Execution Hygiene

### Parent working context

The parent keeps only the minimum live context needed to run the controller accurately:

- [PLAN-17.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-17.md)
- [ORCH_PLAN-17.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-17.md)
- `.runs/plan-17/tasks.json`
- `.runs/plan-17/run-state.json`
- `.runs/plan-17/contract-freeze.json`
- the current gate's `gate-checklist.md` and latest accepted `gate-result.json`
- the latest integration diff summary once `p2` begins

The parent should not keep broad exploratory repo context live after `p1`. If a later decision depends on fresh repo evidence outside the frozen contract, the parent re-reads that evidence deliberately and records it in the relevant task artifact directory.

### Worker prompt contents

Every worker launch packet must be written by the parent under the worker task artifact directory before the worker starts. Each launch packet must contain:

- the exact task id and worker branch/worktree path
- the owned file set
- the forbidden file set
- the exact `PLAN-17` step excerpt relevant to that lane
- the frozen contract clauses that lane must preserve
- the command gates the worker must run
- the acceptance checks the worker must satisfy
- the sentinel path the worker is expected to cause the parent to mark after acceptance
- the retry budget
- the explicit blocked-run rule for out-of-scope discoveries

### Worker return contract

Every worker must return a bounded artifact package, not freeform commentary. The required return surface is:

- `worker-output.patch`
- `worker-report.md`
- `touched-files.txt`
- `command-results.md`
- `evidence-manifest.json`

`worker-report.md` must contain:

- summary of what changed
- exact commands run
- pass/fail results
- unresolved risks
- explicit statement whether the lane stayed within scope
- explicit statement whether any blocked condition was discovered

`command-results.md` must include enough output detail for the parent to classify the lane without rerunning guesswork.

### Prompt and artifact hygiene rules

1. Worker prompts may quote only the narrow `PLAN-17` and `ORCH_PLAN-17` excerpts needed for that lane.
2. Worker prompts may not paste unrelated repo docs or reopen already-frozen scope debates.
3. The parent is the only actor that updates `tasks.json`, `run-state.json`, sentinels, gate artifacts, or quarantine directories.
4. Workers return patches and evidence only. They do not self-approve gates.
5. If a worker finds a genuine out-of-scope blocker, it returns `blocked.json` and stops instead of proposing a broadened implementation.

## PLAN-17 Step Mapping

| Orchestration task | PLAN-17 step alignment |
| --- | --- |
| `task/m17-p1-parent-contract-freeze-and-run-init` | Step 1: freeze the targeted-follow-up contract |
| `task/m17-l1-shell-contract-centralization-and-regression-floor` | Step 2, Step 3, Step 4, and Step 5 |
| `task/m17-l2-docs-gap-matrix-closeout` | Step 6 |
| `task/m17-p3-parent-validation-wall-and-closeout` | Recommended verification commands plus Definition of Done enforcement |

## Lane Ownership By File Set

| Lane | Allowed files | Forbidden escalation surfaces |
| --- | --- | --- |
| `L1` | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs), [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs), [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs), [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs) | any `crates/world-agent/**` production code, `crates/agent-api-types/**`, `crates/shell/src/execution/agent_runtime/state_store.rs`, `crates/shell/src/execution/agents_cmd.rs`, CLI command-mode files, docs |
| `L2` | [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md), [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md) if needed | every Rust file and every runtime/control-plane surface |

## Parent Phase Details

### `task/m17-p1-parent-contract-freeze-and-run-init`

Owner:

- parent only

Scope:

1. Initialize `.runs/plan-17/*` and `.runs/task-m17-*/**`.
2. Freeze the exact failure taxonomy, helper-seam responsibility, no-new-surface boundary, and no-`PLAN-16`-redo rule into `contract-freeze.json`.
3. Freeze worker file ownership, merge order, retry budget, and stop conditions.
4. Freeze the exact validation-wall commands the parent will later execute.
5. Seed all worker worktrees from the exact same post-`p1` tree.

Expected source files reviewed in `p1` but not lane-opened for shared mutation:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)

Command gates:

```bash
cargo test -p shell validate_exact_backend_selection -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

Acceptance:

1. `contract-freeze.json`, `lane-ownership.json`, `merge-order.json`, `run-state.json`, and `tasks.json` exist.
2. The frozen contract explicitly records that this is a contract-centralization and regression-floor run, not a transport or retained-cardinality run.
3. No production-file edits are integrated in `p1` unless the parent takes explicit hotspot ownership and reseeds all worker branches afterward.
4. The parent writes `.runs/plan-17/sentinels/task-m17-p1-parent-contract-freeze-and-run-init.ok`.

### `task/m17-g1-shell-lane-launch-gate`

Owner:

- parent only

Checks:

1. `p1` is accepted.
2. The `L1` worktree was seeded from the exact same post-`p1` tree the parent will integrate against.
3. The `L1` prompt names only owned files, forbidden surfaces, command gates, sentinel, and retry budget.
4. The prompt explicitly repeats that `PLAN-16` retained-member coexistence is already landed and may not be reopened.

Acceptance:

1. No worker starts before this gate is green.
2. The parent writes `.runs/plan-17/sentinels/task-m17-g1-shell-lane-launch-gate.ok`.

## Worker Lanes

### `task/m17-l1-shell-contract-centralization-and-regression-floor`

Owner:

- single worker on `codex/feat-session-centric-state-store-m17-shell-contract-centralization-and-regression-floor`

Owned files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)

Scope:

1. Extract one explicit targeted-follow-up dispatch seam in `async_repl.rs`.
2. Keep `submit_host_targeted_turn(...)` and `submit_world_targeted_turn(...)` as distinct transport implementations.
3. Keep reuse-vs-relaunch inside `ensure_member_runtime_ready_for_descriptor(...)`.
4. Add positive host follow-up proof for the active orchestrator backend.
5. Add stale-or-missing world-runtime proof that exact-backend relaunch happens before submit.
6. Add selector-level tests only where the new seam depends on exactness not already pinned.

Command gates:

```bash
cargo test -p shell validate_exact_backend_selection -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

Acceptance:

1. One obvious helper seam owns exact backend selection result handling, host-vs-world classification, host activity check, Linux-only world gate, readiness call, and final submit choice.
2. The positive host targeted-follow-up path is proven, not inferred.
3. Same-generation world-member reuse remains proven and unchanged.
4. Stale-or-missing retained world state is proven to relaunch the exact backend slot before submit rather than fall back to sibling or shell behavior.
5. Any selector exactness relied on by the seam is pinned in `validator.rs` tests.
6. No owned patch broadens into status/toolbox, CLI caller surfaces, macOS parity, or `PLAN-16` cardinality work.
7. The worker writes `.runs/plan-17/sentinels/task-m17-l1-shell-contract-centralization-and-regression-floor.ok`.

### `task/m17-g2-shell-lane-integration-gate`

Owner:

- parent only

Checks:

1. `L1` returned a patch, report, command transcript, and evidence manifest.
2. Every touched file is inside the `L1` ownership boundary.
3. No patch reopens `world-agent` retained-member ownership, selector grammar, or caller-surface shape.
4. The parent can describe the merged selected-follow-up contract without inventing semantics not present in code and tests.

Acceptance:

1. Accepted `L1` output is marked in `run-state.json`.
2. Rejected or quarantined output is recorded explicitly if `L1` drifted.
3. The parent writes `.runs/plan-17/sentinels/task-m17-g2-shell-lane-integration-gate.ok`.

### `task/m17-p2-parent-shell-lane-integration`

Owner:

- parent only

Scope:

1. Integrate the accepted `L1` patch onto the authoritative checkout.
2. Resolve only straightforward merge mechanics within `L1`-owned surfaces.
3. Re-run the `L1` command gates on the authoritative checkout.
4. Freeze the merged code truth before any doc edits begin.

Acceptance:

1. The authoritative checkout now contains the helper seam and the merged regression floor.
2. No post-merge edit widens scope beyond `L1` ownership without stopping and re-planning.
3. The parent writes `.runs/plan-17/sentinels/task-m17-p2-parent-shell-lane-integration.ok`.

### `task/m17-g3-doc-closeout-launch-gate`

Owner:

- parent only

Checks:

1. `p2` is green.
2. The doc worker is seeded from the exact post-`p2` tree.
3. The doc prompt names only the allowed doc files and repeats that docs must describe the runtime as Linux-first and REPL-first, without implying broader caller-surface completion.

Acceptance:

1. No doc worker starts before this gate is green.
2. The parent writes `.runs/plan-17/sentinels/task-m17-g3-doc-closeout-launch-gate.ok`.

### `task/m17-l2-docs-gap-matrix-closeout`

Owner:

- single worker on `codex/feat-session-centric-state-store-m17-docs-gap-matrix-closeout`

Owned files:

- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
- [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md) if the README still needs an index update after merged truth review

Scope:

1. Describe targeted follow-up submit and reuse as landed, Linux-first, and REPL-first.
2. Describe remaining open work as broader non-REPL caller surface, status/toolbox productization, and macOS parity.
3. Remove stale language that still implies selected-member follow-up submit is broadly unimplemented.

Command gates:

```bash
rg -n "REPL-first|Linux-first|targeted follow-up|member_turn|substrate -c" AGENT_ORCHESTRATION_GAP_MATRIX.md llm-last-mile/README.md
```

Acceptance:

1. The docs do not claim a new caller surface or any macOS parity that the merged code does not prove.
2. The gap matrix and optional README index agree with the merged shell/test truth.
3. The worker writes `.runs/plan-17/sentinels/task-m17-l2-docs-gap-matrix-closeout.ok`.

## Quarantine, Retry, And Blocked-Run Posture

1. `L1` gets one retry only for mechanical defects inside its owned files.
2. If `L1` returns evidence that a correct fix would require `world-agent` production changes, state-store/status changes, CLI caller-surface changes, or macOS work, the parent does not repurpose the lane. The run stops as blocked.
3. `L2` gets one retry only for stale phrasing or overclaiming inside owned docs.
4. If `L2` can only make docs “pass” by broadening claims beyond the validated runtime behavior, the parent rejects the lane and stops the run instead of publishing optimistic docs.
5. The parent never hand-merges a hybrid contract out of conflicting worker guesses. When semantics drift, quarantine the output and either retry within the same owned file set or stop.

## Tests And Acceptance

This is the parent’s operator checklist for deciding whether the run can advance and whether the final state is acceptable.

### A. Contract-surface acceptance

- The targeted-follow-up grammar remains exactly `::<backend_id> <prompt>`.
Proof source:
- merged [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`

- `validate_exact_backend_selection(...)` remains the canonical selector.
Proof source:
- merged [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
- `cargo test -p shell validate_exact_backend_selection -- --nocapture`

- No new non-REPL caller surface is introduced.
Proof source:
- touched-file manifest for `L1`
- absence of edits in CLI/status/toolbox surfaces

### B. Shell implementation acceptance

- One explicit helper seam now owns route classification, host/world gating, readiness dispatch, and final submit choice.
Proof source:
- merged [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- `L1` report plus parent integration review

- `submit_host_targeted_turn(...)` and `submit_world_targeted_turn(...)` remain separate transport implementations.
Proof source:
- merged [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

- `ensure_member_runtime_ready_for_descriptor(...)` remains the relaunch authority.
Proof source:
- merged [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- stale-or-missing world-runtime regression

### C. Host-path acceptance

- Positive host targeted follow-up for the active orchestrator backend is proven.
Proof source:
- merged [repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`

- Non-active host backend rejection remains fail-closed with no shell fallback and no world-member launch.
Proof source:
- merged [repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- same integration test command

### D. World-path acceptance

- Same-generation exact world-member reuse stays green.
Proof source:
- merged [repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- same integration test command

- Stale-or-missing retained world state is proven to relaunch the exact backend slot before submit.
Proof source:
- merged [repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- any required stub support in [repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
- same integration test command

- Non-Linux world-target behavior remains explicit fail-closed rather than silently routing elsewhere.
Proof source:
- merged [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- validation-wall review and unchanged `world-agent` tests

### E. Regression-floor acceptance

- `cargo fmt --all -- --check` is green.
- `cargo clippy --workspace --all-targets -- -D warnings` is green.
- `cargo test -p shell validate_exact_backend_selection -- --nocapture` is green.
- `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture` is green.
- `cargo test -p world-agent member_runtime -- --nocapture` is green.

### F. Docs-truth acceptance

- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md) says selected-member follow-up submit/reuse is landed, Linux-first, and REPL-first.
- [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md) is updated only if needed to keep the packet index truthful.
- Docs do not claim status/toolbox productization, non-REPL caller completion, world-agent cardinality redesign, or macOS parity.

### G. Run-control acceptance

- Every completed task has its sentinel.
- Every gate has a `gate-result.json`.
- `tasks.json` and `run-state.json` match the actual accepted/rejected/quarantined outcomes.
- No quarantined or blocked output was partially integrated.

## Validation Wall

Parent-owned validation commands, executed only after `L2` is integrated:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell validate_exact_backend_selection -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p world-agent member_runtime -- --nocapture
```

Validation-wall requirements:

1. formatting and clippy stay green
2. exact backend selector regressions stay green
3. REPL integration tests prove malformed syntax rejection, positive host follow-up, host mismatch rejection, same-generation world reuse, and stale-or-missing world relaunch behavior
4. `world-agent` member-runtime tests stay green to prove no regression in the already-landed retained-member contract
5. docs do not overclaim beyond what the green runtime/tests prove

### `task/m17-g4-validation-wall-gate`

Owner:

- parent only

Checks:

1. `L2` has been integrated or explicitly skipped because no README index change was needed.
2. All required sentinels through `L2` exist.
3. The parent can enumerate every definition-of-done clause and the command or artifact that proves it.

Acceptance:

1. The validation wall is permitted to run exactly once on the final merged tree.
2. The parent writes `.runs/plan-17/sentinels/task-m17-g4-validation-wall-gate.ok`.

### `task/m17-p3-parent-validation-wall-and-closeout`

Owner:

- parent only

Scope:

1. Run the full validation wall on the final merged tree.
2. Record command outputs and artifact paths in the final task directory.
3. Confirm the gap matrix and optional README index match the validated runtime truth.
4. Mark the run complete only if every definition-of-done clause is proven.

Acceptance:

1. All validation commands succeed on the authoritative checkout.
2. `run-state.json` records a successful terminal state.
3. The parent writes `.runs/plan-17/sentinels/task-m17-p3-parent-validation-wall-and-closeout.ok`.

## Definition-Of-Done Enforcement

The run is complete only if all of these are true:

1. `::<backend_id> <prompt>` remains the only targeted follow-up grammar.
2. exact backend selection still resolves through `validate_exact_backend_selection(...)`.
3. one explicit helper seam in `async_repl.rs` owns the selected-follow-up orchestration contract.
4. host targeted follow-up turns resume only the active orchestrator backend and now have positive proof.
5. Linux world targeted follow-up turns still submit through `MemberTurnSubmitRequestV1`.
6. same-generation exact world-member reuse remains green.
7. stale or missing retained world state causes exact-backend relaunch before submit, not sibling fallback.
8. missing-active-host-runtime and non-Linux world-targeting failures remain explicit and fail closed.
9. no new non-REPL caller surface is introduced.
10. repo docs say REPL-first and Linux-first plainly without implying broader completion.

## Execution-To-Completion Notes

This controller is intentionally end-to-end, not a kickoff memo:

1. the parent freezes the contract before workers start
2. one worker lands the helper seam plus regression floor
3. the parent integrates and rechecks that code truth
4. one worker updates repo-truth docs after the merged code truth exists
5. the parent runs the full validation wall and closes the run only on green proof

No additional parallel lane should be introduced mid-run unless the parent rewrites this controller first.
