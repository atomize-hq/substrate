# ORCH_PLAN-16: Execute PLAN-16 Through A Parent-Frozen Backend-Identity Contract And One Honest Two-Lane Parallel Window

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN-16.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-16.md)  
Style reference: [ORCH_PLAN-15.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-15.md)  
Structure reference: [M26 Orchestration Plan](/Users/spensermcconnell/__Active_Code/atomize-hq/spec/docs/m26_orchestration_kickoff_prompt.md)  
Gap matrix anchor: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Execution type: fresh orchestration plan, Linux-first, exact backend-id selection, retained world-member coexistence, shell + world-agent + tests/docs closeout, parent-frozen identity contract, parent-only integration and approval  
Worker model: GPT-5.4 workers with `reasoning_effort=high`  
Max concurrent code workers before integration: `2`

## Summary

This document is the execution controller for `PLAN-16`, not a restatement of it.

The run shape is fixed before any worker starts:

1. `task/m16-p1-parent-identity-freeze-and-run-init` is parent-only and freezes the retained-member identity contract, exact failure taxonomy, lane ownership, merge order, and validation wall.
2. `task/m16-g1-worker-launch-gate` is parent-only and is the only gate that may launch parallel code workers.
3. The only honest parallel window is exactly two code lanes after `p1`:
   - `task/m16-l1-shell-retained-backend-map`
   - `task/m16-l2-world-agent-retained-coexistence`
4. `task/m16-g2-code-lane-integration-gate` and `task/m16-p2-parent-code-lane-integration` are parent-only.
5. Tests, docs, and gap-matrix closeout do not run in parallel with the code lanes. They start only after `p2` lands final merged code truth.
6. `task/m16-l3-tests-docs-gap-closeout` is a single sequential worker lane on top of the integrated code.
7. `task/m16-g4-validation-wall-gate` and `task/m16-p3-parent-validation-wall-and-closeout` are parent-only and finish the run.

Canonical task IDs:

- `task/m16-p1-parent-identity-freeze-and-run-init`
- `task/m16-g1-worker-launch-gate`
- `task/m16-l1-shell-retained-backend-map`
- `task/m16-l2-world-agent-retained-coexistence`
- `task/m16-g2-code-lane-integration-gate`
- `task/m16-p2-parent-code-lane-integration`
- `task/m16-g3-closeout-launch-gate`
- `task/m16-l3-tests-docs-gap-closeout`
- `task/m16-g4-validation-wall-gate`
- `task/m16-p3-parent-validation-wall-and-closeout`

## Assumptions

1. [PLAN-16.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-16.md) remains the authoritative dependency graph for this run.
2. The authoritative checkout for integration remains on `feat/session-centric-state-store`.
3. The already-landed typed follow-up submit contract is sufficient and does not require new transport-surface work in `p1`.
4. Linux is the only platform that must go green for retained world-member coexistence in this run.
5. The current shell and world-agent tests named in `PLAN-16` remain the correct validation seams for coexistence, duplicate detection, targeted routing, and unchanged regressions.

## Immutable Run Shape

### Frozen contract truth

These are run-stopping invariants, not preferences:

1. `backend_id` remains the canonical selector for every explicit backend-targeted path.
2. The only targeted-turn grammar remains `::<backend_id> <prompt>`.
3. Exact backend selection continues to fail closed for missing, wrong-scope, denied, unrealizable, and duplicate-inventory cases.
4. Retained world-member reuse is keyed by `orchestration_session_id + world_generation + backend_id`.
5. Distinct backend ids may coexist as retained live world members in the same orchestration session and authoritative world generation.
6. Duplicate retained live members for the same backend key remain a hard error.
7. `MemberTurnSubmitRequestV1` remains the submit contract. No second submit transport is introduced.
8. No new CLI surface is introduced.
9. No grammar redesign is introduced.
10. No macOS parity work is introduced.
11. The parent agent is the only integrator and the only approval authority for worker outputs.
12. The tests/docs lane never runs concurrently with the two code lanes.

### Parent-only versus worker-owned authority

Parent-only for the full run:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-16/*`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m16-*/**`
- worker launch, output classification, merge approval, quarantine, retry approval, and final validation
- any temporary shared-hotspot takeover if a lane boundary proves wrong

Parent-only during `p1`, then frozen by artifact:

- retained-member identity contract
- exact explicit-backend failure taxonomy
- worker cap and lane ownership map
- merge order
- validation-wall command list
- stop conditions

Worker-owned after `p1`:

- `L1`: shell retained-member collection, backend-aware reuse, exact shell-side failure preservation
- `L2`: world-agent coexistence, duplicate same-backend detection, exact world-side lookup and submit preservation
- `L3`: integrated tests, test support, docs, and gap-matrix truth

No worker may edit parent-owned run-state. No worker may edit another lane's owned files. No worker may reopen the frozen contract.

### Stop conditions

Stop the run, write `.runs/plan-16/blocked.json`, and do not advance if any of these occur:

1. The retained-member identity contract cannot be frozen without changing grammar, CLI surface, or transport shape.
2. Any implementation reintroduces `agent_id` or singleton-member heuristics as the reusable selector for explicit backend-targeted world reuse.
3. Any implementation allows duplicate same-backend retained members to coexist without failing closed.
4. Any worker changes the `::<backend_id> <prompt>` grammar.
5. Any worker introduces a new CLI surface.
6. Any worker broadens scope into macOS parity work.
7. Any worker edits another lane's owned files or any parent-owned run-state.
8. `L3` starts before `p2` accepts final code truth.
9. The parent would need to invent hybrid semantics during integration to make the two code lanes appear compatible.
10. The validation wall cannot prove coexistence of distinct backend ids, duplicate same-backend fail-closed behavior, unchanged exact targeted-turn routing, and unchanged existing regressions.
11. The gap matrix is marked closed before code, tests, and final docs agree on the same landed behavior.

## Fresh Worktrees And Branches

Fresh worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-16`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-16/shell-retained-backend-map`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-16/world-agent-retained-coexistence`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-16/tests-docs-gap-closeout`

Worker branches:

- `codex/feat-session-centric-state-store-m16-shell-retained-backend-map`
- `codex/feat-session-centric-state-store-m16-world-agent-retained-coexistence`
- `codex/feat-session-centric-state-store-m16-tests-docs-gap-closeout`

Exact setup commands:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-16
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-16/shell-retained-backend-map -b codex/feat-session-centric-state-store-m16-shell-retained-backend-map feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-16/world-agent-retained-coexistence -b codex/feat-session-centric-state-store-m16-world-agent-retained-coexistence feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-16/tests-docs-gap-closeout -b codex/feat-session-centric-state-store-m16-tests-docs-gap-closeout feat/session-centric-state-store
```

### Parent integration surface

The parent integrates on the authoritative checkout already on `feat/session-centric-state-store`.

No separate parent integration worktree is introduced because:

1. `PLAN-16` has one serialized integrator.
2. `.runs/plan-16/*` is parent-owned state and should stay co-located with the authoritative branch context.
3. The only true merge activity is parent-owned and serialized.

## Phase Graph And Concurrency

Parent checkout:

- current authoritative checkout on `feat/session-centric-state-store`

Concurrency rules:

1. Worker cap is `2` until `g2` completes.
2. `p1` must finish before any worker starts.
3. `g1` must accept before any worker starts.
4. The only real parallel window is:
   - `task/m16-l1-shell-retained-backend-map`
   - `task/m16-l2-world-agent-retained-coexistence`
5. `g2` classifies both code-lane outputs before integration.
6. `p2` integrates accepted code lanes in this order:
   - `L2` first
   - `L1` second
7. `g3` starts only after `p2` is green.
8. `L3` runs alone on the exact post-`p2` tree.
9. `g4` and `p3` are parent-only.
10. No third concurrent worker is honest because tests, support stubs, and docs must encode final merged coexistence semantics, not lane-local guesses.

### Why `L2` integrates before `L1`

1. `L2` lands the backend-aware retained-member truth inside `world-agent`, which the shell must reuse honestly.
2. Integrating shell second avoids landing outward coexistence behavior before the world registry can actually honor it.
3. If the combined state reveals semantic drift, the later shell lane is the first quarantine target rather than weakening world-side duplicate detection.

## PLAN-16 Step Mapping

| Orchestration task | PLAN-16 step alignment |
| --- | --- |
| `task/m16-p1-parent-identity-freeze-and-run-init` | Step 1: freeze the retained-member identity contract |
| `task/m16-l1-shell-retained-backend-map` | Step 2 and the shell-owned portion of Step 4 |
| `task/m16-l2-world-agent-retained-coexistence` | Step 3 |
| `task/m16-p2-parent-code-lane-integration` | End-to-end Step 4 enforcement across merged shell + world truth |
| `task/m16-l3-tests-docs-gap-closeout` | Step 5 |
| `task/m16-p3-parent-validation-wall-and-closeout` | Recommended verification commands plus Definition of Done enforcement |

## Parent-Owned Run-State Surface

Canonical parent-owned state under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-16/`:

- `run-state.json`
- `tasks.json`
- `session-log.md`
- `identity-freeze.json`
- `lane-ownership.json`
- `merge-order.json`
- `validation-wall.md`
- `blocked.json` on failure only
- `quarantine/`
- `sentinels/`

`identity-freeze.json` is the single source of truth for:

- `plan_id: "PLAN-16"`
- `plan_source: "llm-last-mile/PLAN-16.md"`
- `orchestration_plan_source: "llm-last-mile/ORCH_PLAN-16.md"`
- `branch: "feat/session-centric-state-store"`
- `selector_authority: "validate_exact_backend_selection"`
- `retained_member_key: "orchestration_session_id + world_generation + backend_id"`
- `allow_distinct_backend_coexistence: true`
- `duplicate_same_backend_policy: "fail_closed"`
- `grammar: "::<backend_id> <prompt>"`
- `new_cli_surface_allowed: false`
- `macos_parity_allowed: false`
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

Required sentinels:

- `.runs/plan-16/sentinels/task-m16-p1-parent-identity-freeze-and-run-init.ok`
- `.runs/plan-16/sentinels/task-m16-g1-worker-launch-gate.ok`
- `.runs/plan-16/sentinels/task-m16-l1-shell-retained-backend-map.ok`
- `.runs/plan-16/sentinels/task-m16-l2-world-agent-retained-coexistence.ok`
- `.runs/plan-16/sentinels/task-m16-g2-code-lane-integration-gate.ok`
- `.runs/plan-16/sentinels/task-m16-p2-parent-code-lane-integration.ok`
- `.runs/plan-16/sentinels/task-m16-g3-closeout-launch-gate.ok`
- `.runs/plan-16/sentinels/task-m16-l3-tests-docs-gap-closeout.ok`
- `.runs/plan-16/sentinels/task-m16-g4-validation-wall-gate.ok`
- `.runs/plan-16/sentinels/task-m16-p3-parent-validation-wall-and-closeout.ok`

Per-task artifact directories:

- `.runs/task-m16-p1-parent-identity-freeze-and-run-init/`
- `.runs/task-m16-g1-worker-launch-gate/`
- `.runs/task-m16-l1-shell-retained-backend-map/`
- `.runs/task-m16-l2-world-agent-retained-coexistence/`
- `.runs/task-m16-g2-code-lane-integration-gate/`
- `.runs/task-m16-p2-parent-code-lane-integration/`
- `.runs/task-m16-g3-closeout-launch-gate/`
- `.runs/task-m16-l3-tests-docs-gap-closeout/`
- `.runs/task-m16-g4-validation-wall-gate/`
- `.runs/task-m16-p3-parent-validation-wall-and-closeout/`

Each task directory contains:

- `task.json`
- `summary.md`
- `commands.txt`
- `artifacts/`
- `evidence-manifest.json`

Each worker task directory also contains:

- `worker-output.patch`
- `worker-report.md`
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

## Parent Phase Details

### `task/m16-p1-parent-identity-freeze-and-run-init`

Owner:

- parent only

Scope:

1. Initialize `.runs/plan-16/*` and `.runs/task-m16-*/**`.
2. Freeze the canonical retained-member key and exact explicit-backend failure taxonomy into `identity-freeze.json`.
3. Freeze worker file ownership, merge order, retry budget, and stop conditions.
4. Freeze the exact validation-wall commands the parent will later execute.
5. Seed all worker worktrees from the exact same post-`p1` tree.

Expected source files reviewed in `p1` but not lane-opened for shared mutation:

- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)

Command gates:

```bash
cargo test -p shell validate_exact_backend_selection -- --nocapture
cargo test -p world-agent member_runtime -- --nocapture
```

Acceptance:

1. `identity-freeze.json`, `lane-ownership.json`, `merge-order.json`, `run-state.json`, and `tasks.json` exist.
2. The frozen contract explicitly records exact backend-id selection, backend-aware coexistence, duplicate same-backend fail-closed behavior, and the no-new-surface constraints.
3. No production-file edits are integrated in `p1` unless the parent takes explicit hotspot ownership and reseeds all worker branches afterward.
4. The parent writes `.runs/plan-16/sentinels/task-m16-p1-parent-identity-freeze-and-run-init.ok`.

### `task/m16-g1-worker-launch-gate`

Owner:

- parent only

Checks:

1. `p1` is accepted.
2. All worker worktrees were seeded from the exact same post-`p1` tree.
3. Worker prompts name only owned files, forbidden files, command gates, sentinel, and retry budget.
4. Worker prompts explicitly repeat the frozen contract clauses that apply to each lane.
5. `run-state.json`, `tasks.json`, and `session-log.md` reflect launch state.

Acceptance:

1. No worker starts before this gate is green.
2. The parent writes `.runs/plan-16/sentinels/task-m16-g1-worker-launch-gate.ok`.

## Worker Lanes

### `task/m16-l1-shell-retained-backend-map`

Owner:

- worker only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-16/shell-retained-backend-map`

Branch:

- `codex/feat-session-centric-state-store-m16-shell-retained-backend-map`

Owned files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)

Forbidden files:

- all parent-owned files
- all `L2` owned files
- all `L3` owned files
- `.runs/plan-16/*`
- `.runs/task-m16-*/**`

Scope:

1. Replace shell singleton retained-member state with a backend-aware retained-member collection.
2. Replace shell reuse lookup keyed by `agent_id` or singleton-slot assumptions with exact lookup by `backend_id` within the frozen retained-member key.
3. Update readiness logic so:
   - exact retained backend match is reused
   - missing exact retained backend launches that backend
   - duplicate same-backend retained entries fail closed
4. Preserve exact backend-specific failure reporting for missing, wrong-scope, denied, unrealizable, and duplicate cases.
5. Preserve targeted-turn grammar and the existing exact route boundary.
6. Preserve world-generation rollover invalidation and widen it to all retained backends for the stale generation.

Command gates:

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p shell validate_exact_backend_selection -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 --no-run
```

Acceptance:

1. The lane touches only its owned files.
2. The shell can retain more than one world member by backend id without fallback to singleton replacement behavior.
3. Exact backend-targeted shell reuse is by `backend_id`, not `agent_id`.
4. Duplicate same-backend retained state fails closed.
5. No grammar, CLI, or macOS scope drift is introduced.
6. All command gates pass.
7. Acceptance evidence exists before the parent marks the lane green:
   - `worker-output.patch`
   - `worker-report.md`
   - `commands.txt` with exit codes
   - `evidence-manifest.json`

### `task/m16-l2-world-agent-retained-coexistence`

Owner:

- worker only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-16/world-agent-retained-coexistence`

Branch:

- `codex/feat-session-centric-state-store-m16-world-agent-retained-coexistence`

Owned files:

- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)

Forbidden files:

- all parent-owned files
- all `L1` owned files
- all `L3` owned files
- `.runs/plan-16/*`
- `.runs/task-m16-*/**`

Scope:

1. Remove the effective global single-retained-member restriction inside `world-agent`.
2. Add exact retained-member lookup by `orchestration_session_id + world_generation + backend_id`.
3. Allow distinct backend ids to coexist in one session and generation.
4. Reject duplicate same-backend retained members for the same session and generation.
5. Preserve strict submit validation against retained backend identity.
6. Preserve submitted-turn collision behavior and do not widen transport or CLI surface.

Command gates:

```bash
cargo test -p world-agent --lib -- --nocapture
cargo test -p world-agent member_runtime -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
cargo test -p world-agent --test member_runtime_world_placement_v1 -- --nocapture
```

Acceptance:

1. The lane touches only its owned files.
2. Distinct backend ids can coexist in one session and generation.
3. Duplicate same-backend retained members fail closed.
4. Submit validation remains exact about retained backend identity.
5. No grammar, CLI, or macOS scope drift is introduced.
6. All command gates pass.
7. Acceptance evidence exists before the parent marks the lane green:
   - `worker-output.patch`
   - `worker-report.md`
   - `commands.txt` with exit codes
   - `evidence-manifest.json`

## Parent Integration Phases

### `task/m16-g2-code-lane-integration-gate`

Owner:

- parent only

Checks:

1. `L1` and `L2` both returned.
2. Each output is classified as `accepted`, `rejected`, or `blocked` before integration.
3. No lane violated file ownership, contract freeze, or stop conditions.
4. `L2` proves coexistence for distinct backend ids without weakening duplicate detection.
5. `L1` proves backend-aware retained lookup and preserves exact failure handling.
6. `g2` goes green only if both code lanes are `accepted`.

Acceptance:

1. The parent writes `.runs/plan-16/sentinels/task-m16-g2-code-lane-integration-gate.ok` only if both code lanes are accepted.

### `task/m16-p2-parent-code-lane-integration`

Owner:

- parent only

Scope:

1. Integrate only accepted outputs.
2. Integrate `L2` first and rerun its lane-local gates on the parent tree.
3. Integrate `L1` second and rerun its lane-local gates on the parent tree.
4. If the shell lane depends on reopening the retained-member contract, quarantine it and bounce it back instead of editing around the freeze.
5. If the combined state allows coexistence but weakens duplicate same-backend failure, quarantine the later-integrated lane.
6. If the combined state preserves exact routing only by implicit singleton behavior, quarantine the shell lane.
7. Record accepted integration truth in `run-state.json` and `session-log.md`.
8. If an accepted worker patch merges mechanically but contradicts `identity-freeze.json`, refuse merge, write `merge-refusal.md`, quarantine the output, and either redrive or block.

Command gates:

```bash
cargo test -p world-agent --lib -- --nocapture
cargo test -p world-agent member_runtime -- --nocapture
cargo test -p shell --lib -- --nocapture
cargo test -p shell validate_exact_backend_selection -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 --no-run
```

Acceptance:

1. The parent remains the sole integrator.
2. The integrated parent tree still matches the frozen retained-member contract.
3. No hybrid truth was invented during integration.
4. The parent writes `.runs/plan-16/sentinels/task-m16-p2-parent-code-lane-integration.ok`.

## Sequential Closeout Lane

### `task/m16-g3-closeout-launch-gate`

Owner:

- parent only

Checks:

1. `p2` is accepted.
2. The closeout worktree is reseeded or rebased to the exact post-`p2` tree.
3. The worker prompt names only final tests, support stubs, and docs ownership.
4. The worker prompt explicitly forbids changing code-lane production files.

Acceptance:

1. The parent writes `.runs/plan-16/sentinels/task-m16-g3-closeout-launch-gate.ok`.

### `task/m16-l3-tests-docs-gap-closeout`

Owner:

- worker only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-16/tests-docs-gap-closeout`

Branch:

- `codex/feat-session-centric-state-store-m16-tests-docs-gap-closeout`

Owned files:

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
- [crates/shell/tests/agents_validate.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agents_validate.rs)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)
- [crates/world-agent/tests/member_runtime_world_placement_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/member_runtime_world_placement_v1.rs)
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)

Forbidden files:

- all parent-owned files
- all `L1` owned files
- all `L2` owned files
- `.runs/plan-16/*`
- `.runs/task-m16-*/**`

Scope:

1. Prove coexistence of `cli:codex` and `cli:claude-code` in one session and generation.
2. Prove exact backend reuse for each backend after both are live.
3. Prove duplicate same-backend retained state fails closed.
4. Preserve the current same-generation same-backend reuse proof.
5. Preserve exact targeted-turn routing and grammar regressions.
6. Extend shell/world-agent test support only as needed to model two retained backends honestly.
7. Update the gap matrix so repo truth matches landed behavior and still leaves broader productization out of scope.

Command gates:

```bash
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agents_validate -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p world-agent --test member_runtime_world_placement_v1 -- --nocapture
```

Acceptance:

1. The lane touches only its owned files.
2. Tests assert coexistence of distinct backend ids and fail-closed duplicate same-backend behavior on final merged code, not on lane-local assumptions.
3. Exact targeted-turn routing remains unchanged and explicitly covered.
4. Existing regressions remain green and explicit.
5. The gap-matrix row is not marked closed unless code, tests, and docs agree on the same shipped truth.
6. All command gates pass.
7. Acceptance evidence exists before the parent marks the lane green:
   - `worker-output.patch`
   - `worker-report.md`
   - `commands.txt` with exit codes
   - `evidence-manifest.json`

## Validation Wall And Final Closeout

### `task/m16-g4-validation-wall-gate`

Owner:

- parent only

Checks:

1. `L3` returned and is classified before final validation.
2. `L3` is `accepted`.
3. No quarantined or blocked output remains unresolved.
4. `validation-wall.md` names the exact final command order.
5. `validation-wall.md` includes contract assertions for:
   - exact backend-id selection
   - coexistence of distinct backend ids
   - duplicate same-backend fail-closed behavior
   - unchanged targeted-turn grammar and routing
   - no new CLI surface
   - no macOS parity broadening

Acceptance:

1. The parent writes `.runs/plan-16/sentinels/task-m16-g4-validation-wall-gate.ok`.

### `task/m16-p3-parent-validation-wall-and-closeout`

Owner:

- parent only

Scope:

1. Integrate only accepted `L3` output.
2. Run the full validation wall in exact order.
3. Record final command results and acceptance evidence in `.runs/task-m16-p3-parent-validation-wall-and-closeout/artifacts/`.
4. Mark the run complete only if the validation wall proves the frozen contract rather than merely compiling.

Required validation commands, executed in this order:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell --lib -- --nocapture
cargo test -p shell validate_exact_backend_selection -- --nocapture
cargo test -p shell --test agents_validate -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p world-agent --lib -- --nocapture
cargo test -p world-agent member_runtime -- --nocapture
cargo test -p world-agent --test member_runtime_world_placement_v1 -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
substrate world doctor --json
substrate shim doctor --json
substrate health --json
```

Required artifacts under `.runs/task-m16-p3-parent-validation-wall-and-closeout/artifacts/`:

- `fmt.txt`
- `clippy.txt`
- `shell-lib-tests.txt`
- `validate-exact-backend-selection.txt`
- `agents-validate.txt`
- `repl-world-first-routing.txt`
- `world-agent-lib-tests.txt`
- `world-agent-member-runtime.txt`
- `world-agent-member-runtime-placement.txt`
- `world-agent-streamed-execute-cancel.txt`
- `world-doctor.json`
- `shim-doctor.json`
- `health.json`
- `contract-audit.md`
- `closeout.md`

Required contract assertions in `contract-audit.md`:

1. Every explicit backend-targeted path still resolves by exact derived `backend_id`.
2. Wrong-scope, denied, unrealizable, duplicate, and missing backend failures remain exact and fail closed.
3. Distinct backend ids can coexist as retained live world members in one session and generation.
4. Duplicate same-backend retained live members still fail closed.
5. The shell reuses retained world members by exact backend id, not by `agent_id`.
6. Existing targeted-turn routing and grammar behavior remain unchanged.
7. Existing same-generation same-backend reuse remains green.
8. No new CLI surface was introduced.
9. No macOS parity work was introduced.
10. The gap matrix matches shipped reality.

Acceptance:

1. The full validation wall passes in order.
2. The doctor outputs are captured.
3. The artifact audit proves coexistence, duplicate-fail-closed behavior, unchanged exact routing, and unchanged regressions.
4. The parent writes `.runs/plan-16/sentinels/task-m16-p3-parent-validation-wall-and-closeout.ok`.

## Gates

Gate sequencing is serialized and parent-owned:

1. `task/m16-p1-parent-identity-freeze-and-run-init`
2. `task/m16-g1-worker-launch-gate`
3. parallel launch of `task/m16-l1-shell-retained-backend-map` and `task/m16-l2-world-agent-retained-coexistence`
4. `task/m16-g2-code-lane-integration-gate`
5. `task/m16-p2-parent-code-lane-integration`
6. `task/m16-g3-closeout-launch-gate`
7. `task/m16-l3-tests-docs-gap-closeout`
8. `task/m16-g4-validation-wall-gate`
9. `task/m16-p3-parent-validation-wall-and-closeout`

Gate rules:

1. The parent updates `run-state.json`, `tasks.json`, and `session-log.md` at every gate transition, worker launch, worker return, retry authorization, quarantine decision, merge refusal, and terminal closeout.
2. A gate is green only when all prerequisite sentinels and evidence for the prior phase exist.
3. A gate cannot go green on narrative progress alone. It requires command evidence, artifact presence, and ownership compliance.
4. `g2` and `g4` are classification gates, not integration gates. They may accept, reject, quarantine, or block. They do not mutate production files.

### Retry and redrive policy

1. Each worker lane has retry budget `1`.
2. The parent must classify the first attempt before authorizing any retry.
3. A retry is allowed only for lane-local failure inside the lane's owned files.
4. A retry is allowed only if the first attempt did not violate a hard guard, frozen contract clause, or cross-lane ownership rule.
5. Grammar drift, CLI-surface broadening, macOS scope broadening, duplicate-same-backend permissiveness, or selector-key regression are non-retryable.
6. If a lane exhausts retry budget without acceptance, the run blocks.

### Merge refusal rules

1. The parent refuses merge for any lane whose patch requires parent-only or cross-lane file edits.
2. The parent refuses merge for any lane that is locally green but semantically contradicts `identity-freeze.json`.
3. The parent refuses merge for any lane that encodes intermediate behavior as final repo truth.
4. Merge refusal writes:
   - `.runs/task-<task-id>/artifacts/merge-refusal.md`
   - `.runs/task-<task-id>/rejected.json`
   - `.runs/plan-16/quarantine/<task-id>/`

## Worker Interfaces

### Worker prompt contract

Every worker prompt sent by the parent must include exactly:

1. task ID
2. attempt number
3. worktree path
4. branch name
5. owned files
6. forbidden files
7. exact frozen contract clauses relevant to that lane
8. command gates
9. retry budget
10. required return artifacts
11. sentinel name

The parent must not send:

1. discretionary broad goals without file boundaries
2. permission to edit parent-owned or cross-lane files
3. instructions that reopen `p1` contract decisions
4. instructions that imply hidden concurrency beyond the one honest two-lane window

### Worker return contract

Every worker must return exactly:

1. changed files list
2. commands run, each with exit code
3. explicit attempt classification: `clean`, `retryable`, or `blocked`
4. unresolved assumptions or blockers
5. `worker-output.patch`
6. `worker-report.md`
7. `evidence-manifest.json`

For a return to be eligible for acceptance, the parent must be able to verify:

1. the patch stays within owned files
2. command gates actually ran
3. exit codes are present
4. the attempt classification matches the evidence
5. no frozen contract clause was reopened

## Tests And Acceptance

### Task-scoped command gates

`task/m16-p1-parent-identity-freeze-and-run-init`

```bash
cargo test -p shell validate_exact_backend_selection -- --nocapture
cargo test -p world-agent member_runtime -- --nocapture
```

`task/m16-l1-shell-retained-backend-map`

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p shell validate_exact_backend_selection -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 --no-run
```

`task/m16-l2-world-agent-retained-coexistence`

```bash
cargo test -p world-agent --lib -- --nocapture
cargo test -p world-agent member_runtime -- --nocapture
cargo test -p world-agent --test member_runtime_world_placement_v1 -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
```

`task/m16-p2-parent-code-lane-integration`

```bash
cargo test -p world-agent --lib -- --nocapture
cargo test -p world-agent member_runtime -- --nocapture
cargo test -p shell --lib -- --nocapture
cargo test -p shell validate_exact_backend_selection -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 --no-run
```

`task/m16-l3-tests-docs-gap-closeout`

```bash
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agents_validate -- --nocapture
cargo test -p world-agent --test member_runtime_world_placement_v1 -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
```

`task/m16-p3-parent-validation-wall-and-closeout`

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell --lib -- --nocapture
cargo test -p shell validate_exact_backend_selection -- --nocapture
cargo test -p shell --test agents_validate -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p world-agent --lib -- --nocapture
cargo test -p world-agent member_runtime -- --nocapture
cargo test -p world-agent --test member_runtime_world_placement_v1 -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
substrate world doctor --json
substrate shim doctor --json
substrate health --json
```

### Acceptance matrix

| Phase | Required proof | Refuse / block trigger |
| --- | --- | --- |
| Parent freeze `p1` | retained-member key, exact failure taxonomy, worker ownership, and validation wall are frozen in artifacts | contract freeze requires grammar, CLI, or macOS scope changes |
| `L1` shell lane | backend-aware retained-member collection, exact backend-id reuse, duplicate fail-closed, exact selector preserved | singleton fallback remains, `agent_id` remains reuse key, or grammar drifts |
| `L2` world lane | distinct backend coexistence lands, duplicate same-backend fail-closed lands, exact submit validation preserved | world-agent still acts singleton, duplicate same-backend is permissive, or transport surface widens |
| Parent integration `p2` | accepted `L2` then accepted `L1`, no hybrid truth, frozen contract preserved | merge requires parent hand-edit around contradiction or combined state weakens duplicate detection |
| `L3` closeout lane | coexistence proof, duplicate-fail-closed proof, unchanged routing regressions, truthful gap matrix | tests encode lane-local assumptions or docs overclaim landed behavior |
| Final validation wall `p3` | full command wall green, doctor evidence captured, contract audit proves coexistence and unchanged regressions | any final assertion fails or audit cannot prove the frozen truth |

## Acceptance Criteria

The run is complete only if all of these are true:

1. The parent froze the retained-member identity contract before worker launch and never reopened it.
2. The only parallel window was `L1` plus `L2`.
3. `L3` started only after code-lane integration was complete.
4. No file ownership overlap occurred outside parent integration and final closeout takeover.
5. Distinct backend ids coexist as retained live world members in one session and generation.
6. Duplicate same-backend retained live members still fail closed.
7. Shell reuse is by exact backend id, not by `agent_id`.
8. Existing exact targeted-turn routing and grammar behavior remain green.
9. Existing same-generation same-backend reuse remains green.
10. No new CLI surface was introduced.
11. No macOS parity work was introduced.
12. The gap matrix is updated to match shipped reality.

## Final State

Success requires all of:

1. every required sentinel exists
2. no blocked artifact exists under `.runs/plan-16/`
3. accepted outputs are integrated in the prescribed order only
4. final validation commands pass
5. `contract-audit.md` proves exact backend-id coexistence, duplicate same-backend fail-closed behavior, unchanged exact targeted-turn routing, and unchanged existing regressions

Blocked termination requires any of:

1. hard-guard violation
2. non-retryable rejection
3. exhausted retry budget
4. merge refusal with no legal redrive path
5. failed validation wall or doctor evidence

On blocked termination the parent must write:

1. `.runs/plan-16/blocked.json`
2. terminal state and rationale in `run-state.json`
3. gate and failure summary in `session-log.md`
4. preserved evidence under `.runs/plan-16/quarantine/` and the relevant task artifact directory

## Context-Control Rules

1. The parent keeps only frozen contract artifacts, active task state, worker reports, narrow diffs, blockers, and validation status in live context.
2. The parent does not keep full worker transcripts in live context.
3. Each worker prompt contains only:
   - its owned file set
   - the exact relevant `PLAN-16` excerpt
   - required commands
   - forbidden touch surfaces
   - the frozen contract clauses it must honor
4. Workers do not write `.runs/plan-16/*`.
5. If a worker discovers it needs an unowned file, it must stop and report that need instead of silently expanding scope.
6. Close each worker immediately after merge or rejection.
