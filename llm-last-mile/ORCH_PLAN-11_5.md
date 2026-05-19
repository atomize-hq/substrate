# ORCH_PLAN-11_5: Execute PLAN-11_5 From The Accepted Gate A/B Carryover

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN-11_5.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-11_5.md)  
Prior blocked-run evidence only: [.runs/plan-11/run-state.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/run-state.json), [.runs/plan-11/blocked.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/blocked.json)  
Execution type: continuation orchestration plan, Linux-first, backend-only, status/trace truth required

## Summary

This document is the execution control artifact for `PLAN-11_5`.

The continuation truth is fixed:

1. Gate A and Gate B carryover are already accepted.
2. Those carryover files become read-only after the parent lands the missing crate-surface export in [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs).
3. Only then may the run open the single honest parallel window:
   - one `world-agent` lane,
   - one shell lane.
4. The parent then performs the only integration pass, the only regression wall, and the only closeout decision.

Fresh run control lives under `.runs/plan-11_5/*`. The old `.runs/plan-11/*` artifacts remain read-only evidence and never become active truth for this continuation.

Canonical task IDs:

- `task/m11_5-p0-carryover-import-and-unblock`
- `task/m11_5-l1-world-agent-member-runtime`
- `task/m11_5-l2-shell-remote-member-cutover`
- `task/m11_5-g1-worker-launch-gate`
- `task/m11_5-g2-integration-gate`
- `task/m11_5-p1-parent-integration-and-regression-wall`
- `task/m11_5-g3-closeout-gate`
- `task/m11_5-p2-closeout`

The parent remains the only integrator, the only writer of accepted branch state on `feat/session-centric-state-store`, and the only writer of `.runs/plan-11_5/*`.

## Hard Guards

### Locked scope

1. This run executes only the blocked remainder of `PLAN-11_5`. It does not restart `PLAN-11`.
2. Gate A/B carryover is accepted and frozen after `task/m11_5-p0-carryover-import-and-unblock`.
3. The parent-owned unblock seam is exactly [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs).
4. No worker may reopen the accepted Gate A/B files:
   - [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
   - [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
   - [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)
   - [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs)
   - [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
5. `POST /v1/execute/stream` and `POST /v1/execute/cancel` remain the only transport seam.
6. `ExecuteStreamFrame::{Start,Event,Exit,Error}` remain the only stream families.
7. `world-agent` owns in-world member execution, remote cancel delivery, event streaming, and completion observation only.
8. The shell remains the only canonical writer of orchestration session state, participant state, status, and trace-facing producer truth.
9. World-scoped member launch fails closed. Host fallback is forbidden.
10. `world-agent` must validate shell-supplied `world_id` and `world_generation` and reject mismatches.
11. The shell must represent retained control explicitly as local vs remote and must not treat remote member control as local `RetainedRunControl`.
12. The shell must use the exported member-dispatch builder through `crate::execution::*`; no private-module reach-in and no duplicate request assembly is allowed.
13. Linux-first remains explicit. Non-Linux member-dispatch behavior stays fail-closed in this slice.
14. No new API family, no `V2` request rename, no shared startup-crate extraction, no extra worker lane, and no status/doctor redesign are allowed in this run.
15. Workers use `GPT-5.4` with `reasoning_effort=high`.

### File boundaries

Parent-only:

- [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs)
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)
- `.runs/plan-11_5/*`
- `.runs/task-m11_5-*/**`

World-agent lane only:

- [crates/world-agent/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/Cargo.toml)
- [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)

Shell lane only:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

Escalation-only, parent-owned if the regression wall proves the planned blast radius false:

- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

### Stop conditions

Stop the run, write `.runs/plan-11_5/blocked.json`, and do not advance if any of these occur:

1. `task/m11_5-p0-carryover-import-and-unblock` needs any file besides `routing.rs` to land the visibility bridge.
2. Either worker lane needs to edit any frozen Gate A/B file.
3. Either worker lane needs to edit the other lane's owned files.
4. The shell lane cannot consume `build_agent_client_and_member_dispatch_request(...)` through `crate::execution::*`.
5. The `world-agent` lane needs a second runtime selector, backend inference from `backend_id`, or host-side fallback to finish.
6. Status or trace truth requires unplanned production changes outside the defined escalation surfaces.
7. A third independent worker lane appears necessary before the regression wall.
8. Integration cannot preserve the parent as the only integrator.

## Workstream Plan

### Fresh-run state surfaces

Canonical parent-owned state:

- `.runs/plan-11_5/run-state.json`
- `.runs/plan-11_5/tasks.json`
- `.runs/plan-11_5/session-log.md`

`run-state.json` is the single source of truth for:

- `plan_id: "PLAN-11_5"`
- `plan_source: "llm-last-mile/PLAN-11_5.md"`
- `orchestration_plan_source: "llm-last-mile/ORCH_PLAN-11_5.md"`
- `branch: "feat/session-centric-state-store"`
- `current_phase`
- `active_task_ids`
- `worker_cap: 2`
- `terminal_state`
- `prior_run_evidence`
- `gate_status`
- `frozen_file_boundaries`
- `worktrees`
- `accepted_worker_outputs`
- `rejected_worker_outputs`
- `blocked_worker_outputs`
- `escalation_usage`
- `final_validation`
- `closeout_pointer`

`tasks.json` is the ordered execution registry for:

- task IDs,
- owner,
- worktree path,
- branch,
- allowed files,
- command gates,
- expected artifacts,
- sentinel name,
- current status.

`session-log.md` is the append-only parent log for:

- gate decisions,
- worker launch and return records,
- acceptance or rejection rationale,
- quarantine rationale,
- merge refusal rationale,
- final closeout or blocked termination summary.

Required parent-owned sentinels and terminal artifacts:

- `.runs/plan-11_5/sentinels/task-m11_5-p0-carryover-import-and-unblock.ok`
- `.runs/plan-11_5/sentinels/task-m11_5-g1-worker-launch-gate.ok`
- `.runs/plan-11_5/sentinels/task-m11_5-l1-world-agent-member-runtime.ok`
- `.runs/plan-11_5/sentinels/task-m11_5-l2-shell-remote-member-cutover.ok`
- `.runs/plan-11_5/sentinels/task-m11_5-g2-integration-gate.ok`
- `.runs/plan-11_5/sentinels/task-m11_5-p1-parent-integration-and-regression-wall.ok`
- `.runs/plan-11_5/sentinels/task-m11_5-g3-closeout-gate.ok`
- `.runs/plan-11_5/sentinels/task-m11_5-p2-closeout.ok`
- `.runs/plan-11_5/blocked.json`
- `.runs/plan-11_5/closeout.md`

Per-task artifact directories:

- `.runs/task-m11_5-p0-carryover-import-and-unblock/`
- `.runs/task-m11_5-g1-worker-launch-gate/`
- `.runs/task-m11_5-l1-world-agent-member-runtime/`
- `.runs/task-m11_5-l2-shell-remote-member-cutover/`
- `.runs/task-m11_5-g2-integration-gate/`
- `.runs/task-m11_5-p1-parent-integration-and-regression-wall/`
- `.runs/task-m11_5-g3-closeout-gate/`
- `.runs/task-m11_5-p2-closeout/`

Each per-task directory contains:

- `task.json`
  - task metadata, command gates, status, and parent decision
- `summary.md`
  - concise execution notes and result
- `commands.txt`
  - exact commands run for that task
- `artifacts/`
  - captured test output snippets or generated evidence

Worker task directories also contain:

- `worker-output.patch`
  - the returned file-scoped patch
- `worker-report.md`
  - touched files, rationale, tests, blockers

Accepted worker outputs additionally contain:

- `accepted.json`
  - parent acceptance timestamp, integrated commit/tree reference, accepted files

Rejected worker outputs additionally contain:

- `rejected.json`
  - parent rejection reason, violated guard or mismatch, non-integrated status
- `quarantine/`
  - preserved patch and evidence retained without integration

Blocked worker outputs additionally contain:

- `blocked.json`
  - blocking condition, required escalation, and stop instruction

Sentinel semantics:

1. A `.ok` sentinel means the parent validated and accepted that task or gate.
2. Missing sentinel means the task is not accepted, even if code exists in a worker tree.
3. `.runs/plan-11/*` stays read-only evidence and is never rewritten by this continuation.
4. `blocked.json` exists only for blocked termination of the fresh run.
5. `closeout.md` exists only for successful completion.

### Worktrees and branches

Fresh worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-11_5`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-11_5/world-agent-member-runtime`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-11_5/shell-remote-member-cutover`

Worker branches:

- `codex/feat-session-centric-state-store-m11_5-world-agent-member-runtime`
- `codex/feat-session-centric-state-store-m11_5-shell-remote-member-cutover`

Exact setup commands:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-11_5
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-11_5/world-agent-member-runtime -b codex/feat-session-centric-state-store-m11_5-world-agent-member-runtime feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-11_5/shell-remote-member-cutover -b codex/feat-session-centric-state-store-m11_5-shell-remote-member-cutover feat/session-centric-state-store
```

### Topology and concurrency

Parent checkout:

- current checkout on `feat/session-centric-state-store`

Concurrency rules:

1. Exact worker cap is `2`.
2. `task/m11_5-p0-carryover-import-and-unblock` is parent-only and must finish first.
3. The only real parallel window is:
   - `task/m11_5-l1-world-agent-member-runtime`
   - `task/m11_5-l2-shell-remote-member-cutover`
4. `task/m11_5-p1-parent-integration-and-regression-wall` is parent-only after both worker lanes return.
5. `task/m11_5-p2-closeout` is parent-only.
6. No additional parallelism is honest in this continuation because status/trace truth depends on integrated behavior of the two runtime halves.

### Gate checkpoints

`task/m11_5-g1-worker-launch-gate`

- parent confirms `p0` is accepted,
- `routing.rs` export is landed,
- Gate A/B carryover files are still unchanged,
- both worker worktrees are seeded from the exact post-`p0` tree,
- `.runs/plan-11_5/sentinels/task-m11_5-g1-worker-launch-gate.ok` is written before any worker edits begin.

`task/m11_5-g2-integration-gate`

- parent confirms both worker lanes have returned,
- each worker output is classified as accepted, rejected, or blocked before merge,
- no frozen-file violation, cross-lane ownership violation, or escalation drift is unresolved,
- `.runs/plan-11_5/sentinels/task-m11_5-g2-integration-gate.ok` is written before parent integration starts.

`task/m11_5-g3-closeout-gate`

- parent confirms the full regression wall passed in order,
- final validation results are recorded in `run-state.json`,
- no active blocked artifact exists for the fresh run,
- `.runs/plan-11_5/sentinels/task-m11_5-g3-closeout-gate.ok` is written before closeout starts.

### Task order

#### `task/m11_5-p0-carryover-import-and-unblock`

Owner:

- parent only

Scope:

1. Initialize `.runs/plan-11_5/*` and the per-task artifact directories.
2. Import `.runs/plan-11/run-state.json` and `.runs/plan-11/blocked.json` as historical evidence only.
3. Freeze the accepted Gate A/B carryover files as read-only for the rest of the run.
4. Add `build_agent_client_and_member_dispatch_request` to the crate-surface re-export block in `routing.rs`.
5. Record the post-`routing.rs` tree state as the reseed point for both workers.

Command gate:

```bash
cargo test -p shell --lib -- --nocapture
```

Acceptance:

1. Only `routing.rs` changed in this task.
2. Gate A/B carryover files are unchanged and marked frozen in `run-state.json`.
3. The command gate passes.
4. Parent writes `.runs/plan-11_5/sentinels/task-m11_5-p0-carryover-import-and-unblock.ok`.

#### `task/m11_5-l1-world-agent-member-runtime`

Owner:

- worker only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-11_5/world-agent-member-runtime`

Branch:

- `codex/feat-session-centric-state-store-m11_5-world-agent-member-runtime`

Scope:

1. Add Linux-only member runtime management in `world-agent`.
2. Route typed `member_dispatch` requests from `service.rs`.
3. Register member spans for retained control and cancel targeting.
4. Extend cancel handling to reach member spans truthfully.
5. Fail closed on binding mismatch, unsupported backend, missing runtime facts, or missing binary.

Command gates:

```bash
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
```

Acceptance:

1. The lane touches only its allowed files.
2. Both command gates pass.
3. The worker returns `worker-output.patch`, `worker-report.md`, touched files, and exact command results.
4. The lane does not request edits to frozen Gate A/B files.

#### `task/m11_5-l2-shell-remote-member-cutover`

Owner:

- worker only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-11_5/shell-remote-member-cutover`

Branch:

- `codex/feat-session-centric-state-store-m11_5-shell-remote-member-cutover`

Scope:

1. Split host-orchestrator launch from remote-member launch in `async_repl.rs`.
2. Consume the exported builder through `crate::execution::*`.
3. Model remote retained control explicitly.
4. Route startup, readiness, cancel, reuse, and replacement through the remote member path.
5. Preserve same-generation reuse and fail-closed replacement behavior.

Command gates:

```bash
cargo test -p shell --test repl_world_first_routing_v1 --no-run
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

Acceptance:

1. The lane touches only its allowed files.
2. Ready/Running remains gated on session-handle evidence, not any arbitrary stream event.
3. Both command gates pass.
4. The worker returns `worker-output.patch`, `worker-report.md`, touched files, and exact command results.
5. The lane does not request edits to frozen Gate A/B files.

#### `task/m11_5-p1-parent-integration-and-regression-wall`

Owner:

- parent only

Scope:

1. Classify each returned worker output as accepted, rejected, or blocked.
2. Integrate accepted `l1` output first.
3. Integrate accepted `l2` output second.
4. Land status truth assertions in `agent_successor_contract_ahcsitc0.rs`.
5. Land trace truth assertions in `agent_hub_trace_persistence.rs`.
6. Prove stale generation never revives after a failed replacement.
7. Record any escalation usage explicitly in `.runs/plan-11_5/run-state.json`.

Command gates:

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
cargo test -p shell --test repl_world_first_routing_v1 --no-run
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test agent_hub_trace_persistence -- --nocapture
```

Acceptance:

1. Parent remains the only integrator.
2. Status and trace truth are green from producer-backed evidence, not inferred state alone.
3. All command gates pass in order.
4. Parent writes `.runs/plan-11_5/sentinels/task-m11_5-p1-parent-integration-and-regression-wall.ok`.

#### `task/m11_5-p2-closeout`

Owner:

- parent only

Scope:

1. Verify all required sentinels exist.
2. Verify final validation results are recorded in `run-state.json`.
3. Confirm `.runs/plan-11_5/blocked.json` is absent for the green path.
4. Write `.runs/plan-11_5/closeout.md`.
5. Mark `.runs/plan-11_5/run-state.json` as `completed`.

Command gate:

```bash
test -f .runs/plan-11_5/sentinels/task-m11_5-g3-closeout-gate.ok
```

Acceptance:

1. The run ends with either a complete closeout or an earlier blocked termination, never a silent partial state.
2. Parent writes `.runs/plan-11_5/sentinels/task-m11_5-p2-closeout.ok` only on green completion.

### Worker output handling

Accepted worker output:

1. Parent records the output in `accepted_worker_outputs` in `run-state.json`.
2. Parent writes the task sentinel.
3. Parent stores the integrated patch, evidence, and `accepted.json` in the task directory.
4. Only accepted output may be merged into the parent checkout.

Rejected worker output:

1. Parent records the output in `rejected_worker_outputs` in `run-state.json`.
2. Parent does not write the task sentinel.
3. Parent preserves the patch, report, and evidence under `quarantine/` in the task directory.
4. Rejected output is never integrated, even if parts look plausible.

Blocked worker output:

1. Parent records the output in `blocked_worker_outputs` in `run-state.json`.
2. Parent writes task-local `blocked.json` in the task directory.
3. Parent preserves the patch and report without integration.
4. If the block violates a hard guard, the full run terminates with `.runs/plan-11_5/blocked.json`.

### Merge refusal rules

The parent refuses merge and blocks the run if integrating either lane requires:

1. reopening any frozen Gate A/B carryover file,
2. cross-lane ownership violations,
3. edits to escalation-only surfaces that were not explicitly approved before merge,
4. reclassifying rejected or blocked output as accepted without new command-gate evidence,
5. any change that breaks the parent-only integrator rule.

## Context-Control Rules

1. The parent keeps only the active phase, frozen invariants, open blockers, and merge state in working memory.
2. Worker prompts must include the exact task ID, worktree, branch, allowed files, forbidden files, command gates, acceptance criteria, and sentinel name.
3. Every worker prompt must specify `model: GPT-5.4` and `reasoning_effort: high`.
4. Workers may read frozen files for context but may not edit them.
5. Workers do not update `.runs/plan-11_5/*` or `.runs/task-m11_5-*/**`.
6. Workers do not paste large file dumps; they return concise rationale and exact evidence only.
7. The parent updates `run-state.json`, `tasks.json`, and `session-log.md` at every gate transition, acceptance, rejection, quarantine, or blocked stop.
8. The parent re-reads [PLAN-11_5.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-11_5.md) before accepting any escalation outside the planned blast radius.
9. Tests may tighten assertions, but no lane may "fix" behavior by weakening status or trace truth claims.
10. The old `.runs/plan-11/*` artifacts are historical inputs only and never become active truth for the fresh run.

## Tests And Acceptance

### Task-scoped command gates

`task/m11_5-p0-carryover-import-and-unblock`

```bash
cargo test -p shell --lib -- --nocapture
```

`task/m11_5-l1-world-agent-member-runtime`

```bash
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
```

`task/m11_5-l2-shell-remote-member-cutover`

```bash
cargo test -p shell --test repl_world_first_routing_v1 --no-run
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

`task/m11_5-p1-parent-integration-and-regression-wall`

```bash
1. cargo test -p shell --lib -- --nocapture
2. cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
3. cargo test -p shell --test repl_world_first_routing_v1 --no-run
4. cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
5. cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
6. cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
7. cargo test -p shell --test agent_hub_trace_persistence -- --nocapture
```

### Acceptance matrix

| Gate | Required proof | Primary surfaces |
| --- | --- | --- |
| Carryover freeze gate | Gate A/B files remain unchanged after `p0` | `agent-api-types`, `world_ops.rs`, `prelude.rs`, socket/repl support tests |
| Worker launch gate | parent exported the builder, froze carryover, seeded both worktrees from the same post-`p0` tree | `routing.rs`, worktree records, `g1` sentinel |
| World-agent runtime gate | remote launch and cancel work from shell-resolved runtime facts | `world-agent` sources, `streamed_execute_cancel_v1.rs` |
| Shell cutover gate | world-backed member launch no longer uses the host-local member path | `async_repl.rs`, `repl_world_first_routing_v1.rs` |
| Integration gate | both worker outputs are classified and safe to integrate | task directories, `tasks.json`, `g2` sentinel |
| Status truth gate | `substrate agent status` reflects the real remote producer | `agent_successor_contract_ahcsitc0.rs` |
| Trace truth gate | trace rows remain participant-correct after remote launch, cancel, and replacement | `agent_hub_trace_persistence.rs` |
| Closeout gate | regression wall passed and no active blocked artifact remains | `run-state.json`, `g3` sentinel |

### Run exit criteria

Successful completion requires all of the following:

1. `routing.rs` exports the builder at the crate surface.
2. The frozen Gate A/B carryover files remain unchanged in this continuation.
3. `world-agent` owns remote member retained control and cancel for world-scoped members.
4. `async_repl.rs` launches world-scoped members through the remote path only.
5. Same-generation reuse still avoids redundant relaunch.
6. Replacement preserves lineage and fails closed honestly.
7. Status and trace show the real remote producer.
8. All required command gates pass in order.
9. `.runs/plan-11_5/closeout.md` exists and `.runs/plan-11_5/run-state.json` is `completed`.

Blocked completion requires all of the following:

1. `.runs/plan-11_5/blocked.json` exists.
2. `.runs/plan-11_5/run-state.json` is `blocked`.
3. `.runs/plan-11_5/session-log.md` records the exact violated guard and stopping task.
4. Rejected or blocked worker output remains preserved but not integrated.
5. No green closeout sentinel is written.

## Assumptions

1. `PLAN-11_5` is correct that Gate A and Gate B carryover are accepted and should not be reopened unless a new regression proves the continuation assumptions false.
2. The current implementation branch for this work remains `feat/session-centric-state-store`.
3. The missing crate-surface export in `routing.rs` is the only parent-owned unblock required before the worker window reopens.
4. The two honest runtime seams after that unblock are exactly the `world-agent` lane and the shell lane; no third independent lane exists before the regression wall.
5. The parent can seed both workers from the exact post-`p0` tree and remain the only integrator for the rest of the run.
6. The command gates listed above remain the correct execution wall for this continuation.
