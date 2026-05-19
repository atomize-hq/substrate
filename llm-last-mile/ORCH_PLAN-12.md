# ORCH_PLAN-12: Execute PLAN-12 From The Frozen Payload Contract And Parent-Owned Bridge Step

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN-12.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-12.md)  
Prior blocked-run evidence only: [.runs/plan-11_5/run-state.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11_5/run-state.json), [.runs/plan-11_5/blocked.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11_5/blocked.json), [.runs/task-m11_5-l2-shell-remote-member-cutover/blocked.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m11_5-l2-shell-remote-member-cutover/blocked.json)  
Execution type: fresh continuation orchestration plan, Linux-first, backend-only, status/trace truth required

## Summary

This document is the execution control artifact for `PLAN-12`.

The orchestration truth is fixed:

1. Step 1 is parent-only. The parent initializes the fresh run, lands the crate-surface bridge, and freezes the bridge seam before any worker starts.
2. The run then opens exactly one honest parallel window:
   - one `world-agent` lane,
   - one shell lane.
3. The parent remains the only integrator, the only writer of accepted branch truth on `feat/session-centric-state-store`, and the only writer of `.runs/plan-12/*`.
4. The parent performs the only integration pass, the only regression wall, and the only closeout decision after both worker lanes return.

Canonical task IDs:

- `task/m12-p1-parent-bridge-and-run-init`
- `task/m12-g1-worker-launch-gate`
- `task/m12-l1-world-agent-member-runtime`
- `task/m12-l2-shell-remote-member-cutover`
- `task/m12-g2-integration-gate`
- `task/m12-p2-parent-integration-and-regression-wall`
- `task/m12-g3-closeout-gate`
- `task/m12-p3-closeout`

Fresh run control lives under `.runs/plan-12/*`. The old `.runs/plan-11_5/*` artifacts remain read-only evidence and never become active truth for this run.

## Hard Guards

### Frozen bridge truth

This planning truth is mandatory and preserved exactly:

> the payload contract stays frozen, but the parent is now explicitly authorized to thaw the crate-surface request bridge before reopening lanes. The plan chooses a preferred fix, direct re-export of MemberDispatchTransportRequest through the allowed crate surface, and pre-authorizes one fallback, a sanctioned adapter helper, so the run cannot block again on the same boundary mistake.

Operational meaning:

1. `MemberDispatchRequestV1` remains frozen.
2. `resolved_runtime` remains frozen.
3. `resolved_runtime.binary_path` stays absolute.
4. `resolved_runtime.backend_kind` stays explicit.
5. The top-level `agent_id` remains authoritative.
6. `protocol` remains part of the transport identity contract.
7. Step 1 uses the preferred bridge first:
   - re-export `MemberDispatchTransportRequest` through [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)
   - re-export it through [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs)
8. If code review rejects direct type export, the parent may use one sanctioned adapter helper in [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs), then re-export that helper through `prelude.rs` and `routing.rs`.
9. No worker may reopen the bridge seam after Step 1 lands, regardless of whether the preferred bridge or the sanctioned fallback was used.

### Locked scope

1. This run executes only `PLAN-12`. It does not restart `PLAN-11_5`.
2. The parent-owned bridge seam is resolved in Step 1 only.
3. Workers may read bridge files for context but may not edit them after `task/m12-p1-parent-bridge-and-run-init` completes.
4. `POST /v1/execute/stream` and `POST /v1/execute/cancel` remain the only transport seam.
5. `ExecuteStreamFrame::{Start,Event,Exit,Error}` remain the only stream families.
6. `world-agent` owns in-world member execution, remote cancel delivery, event streaming, and completion observation only.
7. The shell remains the only canonical writer of orchestration session state, participant state, status, and trace-facing producer truth.
8. World-scoped member launch fails closed. Host fallback is forbidden.
9. `world-agent` must validate shell-supplied `world_id` and `world_generation` and reject mismatches.
10. The shell must represent retained control explicitly as local vs remote and must not model remote member control as local `RetainedRunControl`.
11. The shell must consume the sanctioned bridge through `crate::execution::*`; no private-module reach-in and no duplicate request assembly is allowed.
12. Linux-first remains explicit. Non-Linux member-dispatch behavior stays fail-closed in this slice.
13. No new API family, no `V2` request rename, no shared startup-crate extraction, no status/doctor redesign, and no third worker lane are allowed in this run.
14. Workers use `GPT-5.4` with `reasoning_effort=high`.

### File boundaries

Parent-only bridge files in Step 1, then frozen for the rest of the run:

- [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs)
- [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)

Parent-only, entire run:

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)
- `.runs/plan-12/*`
- `.runs/task-m12-*/**`

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

Stop the run, write `.runs/plan-12/blocked.json`, and do not advance if any of these occur:

1. Step 1 requires changing [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs).
2. Step 1 requires changing the serialized `member_dispatch` payload.
3. The shell lane still cannot construct the request through `crate::execution::*` after the Step 1 bridge lands.
4. Either worker lane needs to edit any bridge file after Step 1.
5. Either worker lane needs to edit the other lane's owned files.
6. The `world-agent` lane needs a second runtime selector, backend inference from `backend_id`, or host-side fallback.
7. Status or trace truth requires unplanned production edits outside the escalation-only surfaces.
8. A third independent worker lane appears necessary before the regression wall.
9. Integration would break the parent-only integrator rule.

## Workstream Plan

### Fresh-run state surfaces

Canonical parent-owned state:

- `.runs/plan-12/run-state.json`
- `.runs/plan-12/tasks.json`
- `.runs/plan-12/session-log.md`

`run-state.json` is the single source of truth for:

- `plan_id: "PLAN-12"`
- `plan_source: "llm-last-mile/PLAN-12.md"`
- `orchestration_plan_source: "llm-last-mile/ORCH_PLAN-12.md"`
- `branch: "feat/session-centric-state-store"`
- `current_phase`
- `active_task_ids`
- `worker_cap: 2`
- `terminal_state`
- `prior_run_evidence`
- `bridge_choice`
- `bridge_files_frozen_after_task`
- `gate_status`
- `worktrees`
- `accepted_worker_outputs`
- `rejected_worker_outputs`
- `blocked_worker_outputs`
- `quarantined_worker_outputs`
- `attempt_counts`
- `retry_budget_by_lane`
- `escalation_usage`
- `final_validation`
- `closeout_pointer`
- `read_only_seed_evidence`

`tasks.json` is the ordered execution registry for:

- task IDs
- owner
- worktree path
- branch
- allowed files
- forbidden files
- command gates
- expected artifacts
- sentinel name
- attempt number
- retry eligibility
- current status

`session-log.md` is the append-only parent log for:

- gate decisions
- bridge choice rationale
- prior-run evidence consulted
- worker launch and return records
- retry authorization or refusal rationale
- acceptance, rejection, quarantine, or blocked rationale
- merge refusal rationale
- final closeout or blocked termination summary

Required parent-owned sentinels and terminal artifacts:

- `.runs/plan-12/sentinels/task-m12-p1-parent-bridge-and-run-init.ok`
- `.runs/plan-12/sentinels/task-m12-g1-worker-launch-gate.ok`
- `.runs/plan-12/sentinels/task-m12-l1-world-agent-member-runtime.ok`
- `.runs/plan-12/sentinels/task-m12-l2-shell-remote-member-cutover.ok`
- `.runs/plan-12/sentinels/task-m12-g2-integration-gate.ok`
- `.runs/plan-12/sentinels/task-m12-p2-parent-integration-and-regression-wall.ok`
- `.runs/plan-12/sentinels/task-m12-g3-closeout-gate.ok`
- `.runs/plan-12/sentinels/task-m12-p3-closeout.ok`
- `.runs/plan-12/blocked.json`
- `.runs/plan-12/closeout.md`

Per-task artifact directories:

- `.runs/task-m12-p1-parent-bridge-and-run-init/`
- `.runs/task-m12-g1-worker-launch-gate/`
- `.runs/task-m12-l1-world-agent-member-runtime/`
- `.runs/task-m12-l2-shell-remote-member-cutover/`
- `.runs/task-m12-g2-integration-gate/`
- `.runs/task-m12-p2-parent-integration-and-regression-wall/`
- `.runs/task-m12-g3-closeout-gate/`
- `.runs/task-m12-p3-closeout/`

Each per-task directory contains:

- `task.json`
- `summary.md`
- `commands.txt`
- `artifacts/`

Worker task directories also contain:

- `worker-output.patch`
- `worker-report.md`

Accepted worker outputs additionally contain:

- `accepted.json`

Rejected worker outputs additionally contain:

- `rejected.json`
- `quarantine/`

Blocked worker outputs additionally contain:

- `blocked.json`

Sentinel semantics:

1. A `.ok` sentinel means the parent validated and accepted that task or gate.
2. Missing sentinel means the task is not accepted, even if code exists in a worker tree.
3. `.runs/plan-11_5/*` stays read-only evidence and is never rewritten by this run.
4. `blocked.json` exists only for blocked termination of the fresh run.
5. `closeout.md` exists only for successful completion.

Read-only seed evidence policy:

1. The parent and workers may consult preserved `m11_5` artifacts as read-only seed context only.
2. Allowed read-only seed inputs include:
   - [.runs/task-m11_5-l1-world-agent-member-runtime/worker-output.patch](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m11_5-l1-world-agent-member-runtime/worker-output.patch)
   - [.runs/task-m11_5-l1-world-agent-member-runtime/worker-report.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m11_5-l1-world-agent-member-runtime/worker-report.md)
   - [.runs/task-m11_5-l2-shell-remote-member-cutover/worker-report.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m11_5-l2-shell-remote-member-cutover/worker-report.md)
   - [.runs/task-m11_5-l2-shell-remote-member-cutover/blocked.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m11_5-l2-shell-remote-member-cutover/blocked.json)
3. These artifacts are never accepted branch truth.
4. These artifacts are never applied blindly.
5. `PLAN-12` uses them to mine logic, not hunks.

### Worktrees and branches

Fresh worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-12`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-12/world-agent-member-runtime`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-12/shell-remote-member-cutover`

Worker branches:

- `codex/feat-session-centric-state-store-m12-world-agent-member-runtime`
- `codex/feat-session-centric-state-store-m12-shell-remote-member-cutover`

Exact setup commands:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-12
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-12/world-agent-member-runtime -b codex/feat-session-centric-state-store-m12-world-agent-member-runtime feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-12/shell-remote-member-cutover -b codex/feat-session-centric-state-store-m12-shell-remote-member-cutover feat/session-centric-state-store
```

### Topology and concurrency

Parent checkout:

- current checkout on `feat/session-centric-state-store`

Concurrency rules:

1. Exact worker cap is `2`.
2. `task/m12-p1-parent-bridge-and-run-init` is parent-only and must finish first.
3. After `g1`, the only real parallel window is:
   - `task/m12-l1-world-agent-member-runtime`
   - `task/m12-l2-shell-remote-member-cutover`
4. `task/m12-p2-parent-integration-and-regression-wall` is parent-only and starts only after both worker lanes are classified at `g2`.
5. `task/m12-p3-closeout` is parent-only.
6. No extra parallelism is honest because status and trace truth depend on integrated behavior of both runtime halves.

## Parent Task Breakdown

### `task/m12-p1-parent-bridge-and-run-init`

Owner:

- parent only

Scope:

1. Initialize `.runs/plan-12/*` and the per-task artifact directories.
2. Import `.runs/plan-11_5/*` evidence into `run-state.json` as historical inputs only.
3. Freeze the payload contract and bridge seam ownership in `run-state.json`.
4. Land the preferred direct re-export of `MemberDispatchTransportRequest` through `prelude.rs` and `routing.rs`.
5. If code review rejects the direct re-export, land exactly one sanctioned adapter helper in `world_ops.rs`, then re-export it through `prelude.rs` and `routing.rs`.
6. Record the chosen bridge mode in `bridge_choice`.
7. Record the post-bridge tree state as the reseed point for both workers.
8. Freeze [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs), [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs), and [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs) for the rest of the run.

Command gate:

```bash
cargo test -p shell --lib -- --nocapture
```

Acceptance:

1. Production-code edits in this task are limited to the parent-owned bridge seam files only.
2. `.runs/plan-12/*` and `.runs/task-m12-p1-parent-bridge-and-run-init/*` are expected parent-owned run artifacts for this task and do not count as production-code scope expansion.
3. The payload contract file remains unchanged.
4. `bridge_choice` is recorded as either `direct-re-export` or `sanctioned-adapter-helper`.
5. The command gate passes.
6. The parent writes `.runs/plan-12/sentinels/task-m12-p1-parent-bridge-and-run-init.ok`.

### `task/m12-g1-worker-launch-gate`

Owner:

- parent only

Checks:

1. `p1` is accepted.
2. The bridge seam is frozen and recorded.
3. Both worker worktrees are seeded from the exact post-`p1` tree.
4. Worker prompts name the frozen bridge files as forbidden edits.
5. Worker prompts name the allowed read-only `m11_5` evidence files they may consult.
6. `run-state.json`, `tasks.json`, and `session-log.md` reflect the launch state.

Acceptance:

1. No worker starts before the gate is accepted.
2. The parent writes `.runs/plan-12/sentinels/task-m12-g1-worker-launch-gate.ok`.

### `task/m12-g2-integration-gate`

Owner:

- parent only

Checks:

1. Both worker lanes returned.
2. Each worker output is classified as `accepted`, `rejected`, or `blocked` before integration.
3. No bridge-file reopen, cross-lane ownership violation, or escalation drift is unresolved.
4. Any rejected output is quarantined before the parent integrates anything.
5. Any blocked output is preserved and may terminate the run.
6. `g2` may go green only if both worker lanes are classified `accepted`.

Acceptance:

1. The parent writes `.runs/plan-12/sentinels/task-m12-g2-integration-gate.ok` only if both worker lanes are accepted and the run can legally proceed to parent integration.

### `task/m12-p2-parent-integration-and-regression-wall`

Owner:

- parent only

Scope:

1. Integrate accepted `l1` output first.
2. Integrate accepted `l2` output second.
3. Land status truth assertions in [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs).
4. Land trace truth assertions in [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs).
5. Prove stale generation never revives after failed replacement.
6. Record any escalation usage explicitly in `.runs/plan-12/run-state.json`.

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

1. The parent remains the only integrator.
2. Status and trace truth are green from producer-backed evidence, not inferred state alone.
3. All command gates pass in order.
4. The parent writes `.runs/plan-12/sentinels/task-m12-p2-parent-integration-and-regression-wall.ok`.

### `task/m12-g3-closeout-gate`

Owner:

- parent only

Checks:

1. The full regression wall passed in order.
2. Final validation results are recorded in `run-state.json`.
3. No active blocked artifact exists for the fresh run.

Acceptance:

1. The parent writes `.runs/plan-12/sentinels/task-m12-g3-closeout-gate.ok` before closeout starts.

### `task/m12-p3-closeout`

Owner:

- parent only

Scope:

1. Verify all required sentinels exist.
2. Verify final validation results are recorded in `run-state.json`.
3. Confirm `.runs/plan-12/blocked.json` is absent for the green path.
4. Write `.runs/plan-12/closeout.md`.
5. Mark `.runs/plan-12/run-state.json` as `completed`.

Command gate:

```bash
test -f .runs/plan-12/sentinels/task-m12-g3-closeout-gate.ok
```

Acceptance:

1. The run ends with either a complete closeout or an earlier blocked termination, never a silent partial state.
2. The parent writes `.runs/plan-12/sentinels/task-m12-p3-closeout.ok` only on green completion.

## Worker Task Breakdown

### `task/m12-l1-world-agent-member-runtime`

Owner:

- worker only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-12/world-agent-member-runtime`

Branch:

- `codex/feat-session-centric-state-store-m12-world-agent-member-runtime`

Scope:

1. Add Linux-only member runtime management in `world-agent`.
2. Route typed `member_dispatch` requests from `service.rs`.
3. Register member spans for retained control and cancel targeting.
4. Extend cancel handling to reach member spans truthfully.
5. Fail closed on binding mismatch, unsupported backend, missing runtime facts, or missing binary.

Forbidden edits:

- all Step 1 bridge files
- all shell-lane files
- all `.runs/plan-12/*` and `.runs/task-m12-*/**`

Command gates:

```bash
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
```

Acceptance:

1. The lane touches only its allowed files.
2. Both command gates pass.
3. The worker may consult the allowed `m11_5` evidence files read-only and may not apply preserved hunks blindly.
4. The worker returns `worker-output.patch`, `worker-report.md`, touched files, and exact command results.
5. The lane does not request edits to frozen payload or bridge files.

### `task/m12-l2-shell-remote-member-cutover`

Owner:

- worker only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-12/shell-remote-member-cutover`

Branch:

- `codex/feat-session-centric-state-store-m12-shell-remote-member-cutover`

Scope:

1. Split host-orchestrator launch from remote-member launch in `async_repl.rs`.
2. Consume the sanctioned bridge through `crate::execution::*`.
3. Model remote retained control explicitly.
4. Route startup, readiness, cancel, reuse, and replacement through the remote member path.
5. Preserve same-generation reuse and fail-closed replacement behavior.

Forbidden edits:

- all Step 1 bridge files
- all world-agent lane files
- all `.runs/plan-12/*` and `.runs/task-m12-*/**`

Command gates:

```bash
cargo test -p shell --test repl_world_first_routing_v1 --no-run
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

Acceptance:

1. The lane touches only its allowed files.
2. `Ready` and `Running` remain gated on session-handle evidence, not any arbitrary stream event.
3. Both command gates pass.
4. The worker may consult the allowed `m11_5` evidence files read-only and may not apply preserved hunks blindly.
5. The worker returns `worker-output.patch`, `worker-report.md`, touched files, and exact command results.
6. The lane does not request edits to frozen payload or bridge files.

## Gates

### Gate sequencing

1. `task/m12-p1-parent-bridge-and-run-init`
2. `task/m12-g1-worker-launch-gate`
3. `task/m12-l1-world-agent-member-runtime` and `task/m12-l2-shell-remote-member-cutover` in parallel
4. `task/m12-g2-integration-gate`
5. `task/m12-p2-parent-integration-and-regression-wall`
6. `task/m12-g3-closeout-gate`
7. `task/m12-p3-closeout`

### Retry and redrive policy

1. Each worker lane has a retry budget of `1` additional attempt after the first attempt.
2. The exact worker cap remains `2`; retries reuse the same lane slot and do not open a third lane.
3. The parent must classify the first attempt as `accepted`, `rejected`, or `blocked` before authorizing any retry.
4. A retry is allowed only for a lane-local failure that stayed within the lane's owned file boundary.
5. A retry is allowed only if the first attempt did not violate a hard guard.
6. A retry is allowed only if the first attempt did not drift into cross-lane ownership.
7. A retry is allowed only if the first attempt did not require new blast-radius authority.
8. Retries must stay inside the same owned file boundary and the same lane contract.
9. The parent records each attempt in `run-state.json`, `tasks.json`, and `session-log.md` with attempt count and retry rationale.
10. A retry may reuse the same task ID with `attempt=2` metadata, or use a retry suffix recorded by the parent, but the lane identity does not change.
11. If a lane exhausts its retry budget without an accepted result, the run blocks.
12. Hard-guard violation, cross-lane ownership drift, bridge-file reopen, or need for new blast-radius authority makes the lane immediately non-retryable and may block the run at once.

### Worker prompt contract

Each worker prompt must include:

1. exact task ID
2. attempt number and whether it is first attempt or retry
3. worktree and branch
4. allowed files
5. forbidden files
6. command gates
7. acceptance criteria
8. the specific prior-run evidence files the worker may consult read-only

### Worker return contract

Each worker return must include:

1. changed files
2. commands run with exit codes
3. blocker or unresolved assumption list
4. whether the attempt is `clean`, `retryable`, or `blocked`
5. `worker-output.patch` and `worker-report.md`

Accepted worker output:

1. The parent records the output in `accepted_worker_outputs` in `run-state.json`.
2. The parent writes the task sentinel.
3. The parent stores the integrated patch, evidence, and `accepted.json` in the task directory.
4. Only accepted output may be merged into the parent checkout.

Rejected worker output:

1. The parent records the output in `rejected_worker_outputs` and `quarantined_worker_outputs`.
2. The parent does not write the task sentinel.
3. The parent preserves the patch, report, and evidence under `quarantine/` in the task directory.
4. Rejected output is never integrated.
5. The parent classifies the rejection as retryable or non-retryable before any redrive decision.
6. A retryable rejection prevents `g2` from going green until the parent-authorized retry returns.
7. A non-retryable rejection, hard-guard violation, or exhausted retry budget blocks the full run.

Blocked worker output:

1. The parent records the output in `blocked_worker_outputs`.
2. The parent writes task-local `blocked.json`.
3. The parent preserves the patch and report without integration.
4. If the block violates a hard guard or prevents the regression wall from starting, the full run terminates with `.runs/plan-12/blocked.json`.

### Merge refusal rules

The parent refuses merge and blocks the run if integrating either lane requires:

1. reopening any frozen payload or bridge file,
2. cross-lane ownership violations,
3. edits to escalation-only surfaces that were not explicitly approved before merge,
4. reclassifying rejected or blocked output as accepted without new command-gate evidence,
5. weakening status or trace truth assertions to force green,
6. breaking the parent-only integrator rule.

## Context-Control Rules

1. The parent keeps only the active phase, frozen invariants, bridge choice, open blockers, and merge state in working memory.
2. Worker prompts must include the exact task ID, attempt number, worktree, branch, allowed files, forbidden files, command gates, acceptance criteria, sentinel name, and the specific read-only prior-run evidence files allowed for that attempt.
3. Every worker prompt must specify `model: GPT-5.4` and `reasoning_effort: high`.
4. Workers may read frozen files for context but may not edit them.
5. Workers do not update `.runs/plan-12/*` or `.runs/task-m12-*/**`.
6. Workers may consult the allowed `m11_5` patch and blocked-run artifacts read-only as seed context; they are never accepted truth and never applied blindly.
7. Workers do not paste large file dumps; they return concise rationale and exact evidence only.
8. The parent updates `run-state.json`, `tasks.json`, and `session-log.md` at every gate transition, attempt launch, attempt return, acceptance, rejection, retry authorization, quarantine, or blocked stop.
9. The parent re-reads [PLAN-12.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-12.md) before approving any escalation outside the planned blast radius.
10. Tests may tighten assertions, but no lane may "fix" behavior by weakening status or trace truth claims.
11. The blocked `.runs/plan-11_5/*` artifacts remain historical inputs only and never become active truth for this run.

## Tests And Acceptance

### Task-scoped command gates

`task/m12-p1-parent-bridge-and-run-init`

```bash
cargo test -p shell --lib -- --nocapture
```

`task/m12-l1-world-agent-member-runtime`

```bash
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
```

`task/m12-l2-shell-remote-member-cutover`

```bash
cargo test -p shell --test repl_world_first_routing_v1 --no-run
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

`task/m12-p2-parent-integration-and-regression-wall`

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
| Parent bridge gate | the sanctioned crate-surface bridge is landed without payload changes | `routing.rs`, `prelude.rs`, optional `world_ops.rs`, `p1` sentinel |
| Worker launch gate | bridge files are frozen and both workers are seeded from the same post-`p1` tree | worktree records, `run-state.json`, `g1` sentinel |
| World-agent runtime gate | remote launch and cancel work from shell-resolved runtime facts | `world-agent` sources, `streamed_execute_cancel_v1.rs` |
| Shell cutover gate | world-backed member launch no longer uses the host-local member path and consumes the sanctioned bridge only | `async_repl.rs`, `repl_world_first_routing_v1.rs` |
| Integration gate | both worker outputs are classified and safe to integrate | task directories, `tasks.json`, `g2` sentinel |
| Status truth gate | `substrate agent status` reflects the real remote producer | `agent_successor_contract_ahcsitc0.rs` |
| Trace truth gate | trace rows remain participant-correct after remote launch, cancel, and replacement | `agent_hub_trace_persistence.rs` |
| Closeout gate | regression wall passed and no active blocked artifact remains | `run-state.json`, `g3` sentinel |

### Run exit criteria

Successful completion requires all of the following:

1. The parent landed a sanctioned Step 1 bridge without changing the payload contract.
2. The bridge files remain frozen after Step 1.
3. `world-agent` owns remote member retained control and cancel for world-scoped members.
4. `async_repl.rs` launches world-scoped members through the remote path only.
5. Same-generation reuse still avoids redundant relaunch.
6. Replacement preserves lineage and fails closed honestly.
7. Status and trace show the real remote producer.
8. All required command gates pass in order.
9. `.runs/plan-12/closeout.md` exists and `.runs/plan-12/run-state.json` is `completed`.

Blocked completion requires all of the following:

1. `.runs/plan-12/blocked.json` exists.
2. `.runs/plan-12/run-state.json` is `blocked`.
3. `.runs/plan-12/session-log.md` records the exact violated guard and stopping task.
4. Rejected or blocked worker output remains preserved and quarantined without integration.
5. No green closeout sentinel is written.

## Assumptions

1. [PLAN-12.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-12.md) is the authoritative execution contract for this run.
2. The current implementation branch for this work remains `feat/session-centric-state-store`.
3. The only honest Step 1 parent-owned unblock is the sanctioned crate-surface bridge described in `PLAN-12`.
4. The two honest worker lanes after Step 1 are exactly the `world-agent` lane and the shell lane.
5. The parent can seed both workers from the exact post-bridge tree and remain the only integrator for the rest of the run.
6. The preserved `m11_5` worker patch and blocked shell worker artifacts are valid read-only seed context for parent and workers, but never accepted branch truth and never blindly applied.
7. The command gates listed above remain the correct execution wall for this run.
