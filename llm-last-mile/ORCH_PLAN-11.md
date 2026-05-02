# ORCH_PLAN-11: In-World Member Dispatch Over Existing Host<->World Transport

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN-11.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-11.md)  
Reference style sources: [ORCH_PLAN-08.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-08.md), [ORCH_PLAN-09.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-09.md), [ORCH_PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-10.md)  
Execution type: shell/world-agent transport-placement cutover orchestration plan, Linux-first, no UI scope, strong runtime/status/trace scope

## Summary

This run executes `PLAN-11` on `feat/session-centric-state-store` with an exact active worker cap of `2`. The parent remains the only integrator, the only final branch writer, and the only agent allowed to mutate the frozen request/builder/event contract before the runtime cutover begins. The canonical run-state source of truth is `.runs/plan-11/run-state.json`.

This slice is not a new control plane, not a scheduler, not a docs-first cleanup, and not a platform-parity project. It is one bounded production cutover:

1. freeze the typed `ExecuteRequest.member_dispatch` contract and request-boundary validation in [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs),
2. freeze the shell transport builder and harness capture in [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs), [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs), and [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs),
3. implement the Linux-first in-world member runtime manager in `world-agent`,
4. cut the shell member path off the local `gateway.run_control(...)` path with an explicit local-vs-remote retained-control split and an explicit `PreparedMemberDispatch` launch shape,
5. close the final regression wall for cancel, replacement, status, and trace correctness,
6. touch docs only if shipped operator wording changed.

The honest concurrency shape is narrow. The typed request contract, the shell request builder, and the session-bearing ready event contract must freeze before any worker lane starts. After that freeze, exactly two runtime lanes are real:

1. lane `B1` for [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs) plus the new [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs),
2. lane `B2` for [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) plus its immediate shell test fallout,
3. parent-only integration and regression closure after both lanes merge.

Every child worker uses `GPT-5.4` with `reasoning_effort=high`.

Worktree set for the run:

- `../substrate-m11-world-agent-member-manager` on `codex/feat-session-centric-state-store-m11-world-agent-member-manager`
- `../substrate-m11-shell-remote-member-cutover` on `codex/feat-session-centric-state-store-m11-shell-remote-member-cutover`

The parent-owned critical path is:

1. `task/m11-a1-preflight`
2. `task/m11-a2-request-boundary-contract-freeze`
3. `task/m11-a3-shell-transport-harness-freeze`
4. `task/m11-c1-integrate-and-regression-wall`
5. `task/m11-c2-docs-review`
6. `task/m11-c3-closeout`

## Hard Guards

### Locked invariants

1. `PLAN-11` is a transport-backed placement-honesty slice only. No UI work is authorized.
2. `POST /v1/execute/stream` and `POST /v1/execute/cancel` remain the only transport seam for this run.
3. `ExecuteRequest.member_dispatch` is additive and typed. No magic command string is allowed.
4. Request-boundary validation happens at the type boundary in [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs), not ad hoc inside handlers.
5. For typed member dispatch, `cmd.trim().is_empty()` must be true and `pty` must be false. For ordinary process exec, `member_dispatch` must be `None`.
6. The top-level `agent_id` remains authoritative for budgets, traces, and diagnostics. `member_dispatch` must not duplicate it.
7. The shell remains the only authority for canonical session-root writes, persisted participant state, `status`, `doctor`, and toolbox surfaces.
8. `world-agent` owns in-world member execution, remote control retention, cancel delivery, and remote lifecycle streaming only.
9. The shell must represent retained control explicitly as local vs remote. Remote member control must not be stuffed into local `RetainedRunControl`.
10. The shell must use an explicit `PreparedMemberDispatch` or equivalent dedicated remote-prepared launch shape. Host orchestrator startup stays local-gateway shaped.
11. A world-scoped member launch fails closed if remote dispatch fails. Host fallback is forbidden.
12. The authoritative `world_id` and `world_generation` supplied by the shell must be validated remotely and rejected on mismatch.
13. `ExecuteStreamFrame::{Start,Event,Exit,Error}` remain the only stream families. No new frame family is authorized.
14. The shell persists a member participant as `Allocating` before remote launch and may advertise live only after session-handle evidence, retained ownership, active event stream, and completion observation all exist.
15. The remote event that unlocks readiness must remain compatible with `extract_session_handle_id(...)` in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) unless the parent explicitly refreezes the event contract before worker lanes open.
16. `/v1/execute/cancel` must be able to address both process-exec spans and member-dispatch spans.
17. `substrate agent status --json` and trace rows must remain producer-backed and participant-correct after cancel and replacement. Stale liveness must not be revived.
18. Linux-first is explicit. Non-Linux paths fail closed for member dispatch in this slice. No macOS or Windows parity work is authorized.
19. Docs are late and optional. No early doc edits are authorized.
20. Package-targeted cargo commands must use the real package names in this repo: `agent-api-types`, `world-agent`, and `shell`.
21. The parent is the only writer of `.runs/plan-11/*`.

### File-level boundaries

Parent-owned serialized freeze surfaces:

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)

Worker-safe world-agent lane after the freeze:

- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
- new [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)

Worker-safe shell runtime lane after the freeze:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

Parent-owned late regression and drift-closure surfaces:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)

Escalation-only production surfaces:

- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Read-for-truth only:

- [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs)
- [llm-last-mile/PLAN-11.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-11.md)
- [llm-last-mile/ORCH_PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-10.md)

Optional parent-owned docs surface only after the regression wall is green:

- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)

### Non-negotiable stop conditions

Stop the run and write `.runs/plan-11/blocked.json` if any of these occur:

1. A task requires reopening the frozen `member_dispatch` contract in `agent-api-types` after worker lanes `B1` or `B2` have started.
2. A task requires a new `/v1/member/*`, `/v1/agents/*`, scheduler surface, or non-Linux implementation to finish `PLAN-11`.
3. A task falls back to host launch after a remote world-member launch failure.
4. A task keeps the shell member path on local `gateway.run_control(...)` while claiming world placement.
5. A task treats remote control state as if it were a local `RetainedRunControl`.
6. A task introduces a second lifecycle model instead of reusing the existing manifest/store/liveness contract.
7. A worker lane needs to touch any parent-owned freeze file or any late regression file to finish its assignment.
8. The world-agent lane needs to rename or redesign the session-bearing `Event` payload after the shell lane starts.
9. A task requires new semantics in [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) rather than consuming the existing persistence and invalidation contract.
10. A task proves `status` or trace correctness requires heuristic trace-first liveness recovery.
11. A task requires early docs edits to explain behavior that runtime and tests still do not prove.

## Orchestration State Surfaces

### Canonical run state

The only canonical source of truth for run orchestration state:

- `.runs/plan-11/run-state.json`

Parent-only writes to this file. It tracks:

- current phase,
- active task IDs,
- branch and worktree assignment,
- gate status,
- frozen `member_dispatch` request contract,
- frozen request-boundary validation rules,
- frozen session-handle event extraction contract,
- frozen local-vs-remote retained-control contract,
- frozen `PreparedMemberDispatch` launch contract,
- accepted and rejected worker outputs,
- escalation-file usage,
- blocked or completed terminal state,
- final closeout pointer.

If a worker report conflicts with `run-state.json`, the parent treats `run-state.json` as authoritative until it explicitly reconciles the discrepancy.

### Derived run artifacts

The parent may maintain these local artifacts:

- `.runs/plan-11/queue.json`
- `.runs/plan-11/session.log`
- `.runs/plan-11/sentinels/task-m11-a1-preflight.ok`
- `.runs/plan-11/sentinels/task-m11-a2-request-boundary-contract-freeze.ok`
- `.runs/plan-11/sentinels/task-m11-a3-shell-transport-harness-freeze.ok`
- `.runs/plan-11/sentinels/task-m11-b1-world-agent-member-manager.ok`
- `.runs/plan-11/sentinels/task-m11-b2-shell-remote-member-cutover.ok`
- `.runs/plan-11/sentinels/task-m11-c1-integrate-and-regression-wall.ok`
- `.runs/plan-11/sentinels/task-m11-c2-docs-review.ok`
- `.runs/plan-11/sentinels/task-m11-c3-closeout.ok`
- `.runs/plan-11/blocked.json`
- `.runs/plan-11/closeout.md`

Sentinel rules:

1. `.ok` means the parent validated the task output and advanced the run.
2. Missing sentinel means the task is not accepted.
3. `blocked.json` is written only on blocked termination.
4. `closeout.md` is written only on successful completion.
5. Worker notes, local commits, or branch state never replace parent-written sentinels or `run-state.json`.

## Concurrency Policy

1. The parent is the only integrator.
2. The parent is the only writer of final branch state on `feat/session-centric-state-store`.
3. Exact active worker cap: `2`.
4. There are zero worker lanes during `task/m11-a1` through `task/m11-a3`.
5. The only honest parallel window is `task/m11-b1` plus `task/m11-b2` after the transport contract and harness freeze lands.
6. No worker may edit [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs), [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs), [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs), or [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs) once Gate B passes.
7. No worker may edit [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs), [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs), docs, or any `.runs/plan-11/*` artifact.
8. Both worker worktrees are seeded from the exact post-`task/m11-a3` tree.
9. If either runtime lane discovers a missing contract change in a frozen parent-owned file, that lane stops and hands the gap back to the parent instead of widening scope.
10. Worker coordination uses sentinels and long waits. Tight polling loops against git state or run-state are forbidden.

### Why the worker cap stays exactly `2`

The worker cap is exactly `2` because there are only two honest runtime seams after the freeze and no third honest lane before the regression wall:

1. there is no safe parallelism across the frozen request/builder/event seam in `agent-api-types`, `world_ops.rs`, `socket.rs`, and `repl_world_agent.rs`,
2. there is one real `world-agent` runtime lane after that freeze,
3. there is one real shell remote-cutover lane after that freeze,
4. there is no third independent lane because status, trace, replacement, and operator-surface correctness all depend on the integrated behavior of both runtime halves,
5. opening a third worker before `task/m11-c1` would create fake parallelism, contract churn, and merge noise rather than throughput.

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

- parent re-reads [PLAN-11.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-11.md) and [ORCH_PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-10.md),
- parent records the exact execution shape: contract freeze, harness freeze, one parallel runtime window, one final regression wall,
- parent records the exact worker cap of `2`,
- parent records the package-name normalization: `agent-api-types`, `world-agent`, `shell`.

### Gate B: Transport And Event Contract Freeze

Required before worker lanes open:

- `ExecuteRequest.member_dispatch` shape is frozen,
- request-boundary mutual-exclusion rules are frozen,
- the session-bearing remote `Event` shape is frozen well enough for `extract_session_handle_id(...)` or an explicitly parent-approved equivalent,
- `world_ops.rs` request building is frozen,
- shell harness capture in `socket.rs` and `repl_world_agent.rs` is frozen,
- the parent has completed the pre-worker proof checks defined below,
- both child worktrees are seeded from the exact post-`task/m11-a3` tree.

### Gate C: Runtime Seam Integration

Required before the final regression wall starts:

- both worker outputs are integrated by the parent,
- any approved escalation into [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs), [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs), or [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) is already complete and justified in `run-state.json`,
- host orchestrator startup remains unchanged,
- no production path still routes world-member startup through local `gateway.run_control(...)`.

### Gate D: Final Acceptance

Required before closeout:

- the integrated runtime uses typed execute-stream member dispatch,
- cancel, replacement, status, and trace regression tests are green in the required order,
- docs review is complete and either produced aligned late edits or an explicit no-change decision,
- `run-state.json` records the final request, transport, remote-control, replacement, and regression contracts.

## Workstream Plan

### Worktree topology

Parent checkout:

- current checkout on `feat/session-centric-state-store`

Child worktrees and branches:

- `../substrate-m11-world-agent-member-manager`
  - `codex/feat-session-centric-state-store-m11-world-agent-member-manager`
- `../substrate-m11-shell-remote-member-cutover`
  - `codex/feat-session-centric-state-store-m11-shell-remote-member-cutover`

Worker model:

- every child worker uses `GPT-5.4` with `reasoning_effort=high`,
- workers return patches or local commits, touched files, tests run, and blockers,
- workers do not merge each other and do not write the parent branch.

### Task graph

Execution graph for the run:

1. `task/m11-a1-preflight`
2. `task/m11-a2-request-boundary-contract-freeze`
3. `task/m11-a3-shell-transport-harness-freeze`
4. `task/m11-b1-world-agent-member-manager` and `task/m11-b2-shell-remote-member-cutover` in parallel
5. `task/m11-c1-integrate-and-regression-wall`
6. `task/m11-c2-docs-review`
7. `task/m11-c3-closeout`

Parent-only serialized tasks:

- `task/m11-a1-preflight`
- `task/m11-a2-request-boundary-contract-freeze`
- `task/m11-a3-shell-transport-harness-freeze`
- `task/m11-c1-integrate-and-regression-wall`
- `task/m11-c2-docs-review`
- `task/m11-c3-closeout`

Worker-owned tasks:

- `task/m11-b1-world-agent-member-manager`
- `task/m11-b2-shell-remote-member-cutover`

## Parallel Window B

This is the only worker window in the run.

It opens only after Gate B passes. Before that point, the contract is too coupled to split safely:

- `agent-api-types` defines the typed request boundary,
- `world_ops.rs` defines the serialized outbound request shape,
- `socket.rs` and `repl_world_agent.rs` define what tests can actually assert,
- the shell and world-agent lanes both depend on the same event payload compatibility for remote readiness.

After that freeze, the runtime work splits honestly:

- lane `B1` owns `world-agent` execution, span registration, cancel delivery, and streaming,
- lane `B2` owns shell retained-control representation, remote launch cutover, and replacement convergence,
- neither lane should need to reopen the typed request or builder contract.

### task/m11-a1-preflight

Ownership:

- parent only

Scope:

1. Re-read [PLAN-11.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-11.md), [ORCH_PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-10.md), and the cited code seams.
2. Confirm the run executes on `feat/session-centric-state-store`.
3. Create `.runs/plan-11/` artifacts and initialize `run-state.json`.
4. Record exact parent-owned, worker-owned, and escalation-only file boundaries.
5. Record the no-fallback, shell-authority, Linux-first, and docs-late posture.

Acceptance gate:

- Gate A passes,
- `run-state.json` contains the frozen topology and package list,
- `.runs/plan-11/sentinels/task-m11-a1-preflight.ok` exists.

### task/m11-a2-request-boundary-contract-freeze

Ownership:

- parent only

Scope:

1. Freeze additive `MemberDispatchRequestV1` and `ExecuteRequest.member_dispatch` in [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs).
2. Implement request-boundary validation with an internal deserialize/parse boundary, not handler-local checks.
3. Freeze the exact exclusivity rules for `cmd`, `pty`, and `member_dispatch`.
4. Freeze the authoritative identity tuple fields carried by member dispatch: orchestration session, participant lineage, backend, protocol, run, world id, and world generation.
5. Add transport-level round-trip and invalid-shape tests so ordinary process exec remains unchanged and member-dispatch failure is rejected at parse time.

Parent validation before sentinel:

1. `cargo test -p agent-api-types -- --nocapture`
2. `cargo test -p world-agent --test streamed_execute_cancel_v1 -- --no-run`
3. `cargo test -p shell --test repl_world_first_routing_v1 -- --no-run`
4. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --no-run`
5. `cargo test -p shell --test agent_hub_trace_persistence -- --no-run`

Acceptance gate:

- typed request rules compile without changing old ordinary-exec callers,
- invalid `cmd` plus `member_dispatch` shapes fail at the request boundary,
- downstream packages still compile against the additive request shape,
- the parent records the exact accepted request schema in `run-state.json`,
- `.runs/plan-11/sentinels/task-m11-a2-request-boundary-contract-freeze.ok` exists.

### task/m11-a3-shell-transport-harness-freeze

Ownership:

- parent only

Scope:

1. Freeze the dedicated member-dispatch request builder in [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs).
2. Extend [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs) so recorded execute-stream payloads can deserialize and assert `member_dispatch`.
3. Extend [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs) so typed member-dispatch payloads and lineage fields are capturable and lifecycle scripting is possible.
4. Freeze the remote-ready event compatibility required by `extract_session_handle_id(...)` in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).
5. Prove that the shell can assert payload correctness before the runtime switch lands.

Parent pre-worker validation order:

1. `cargo test -p agent-api-types -- --nocapture`
2. `cargo test -p shell --lib -- --nocapture`
3. `cargo test -p world-agent --test streamed_execute_cancel_v1 -- --no-run`
4. `cargo test -p shell --test repl_world_first_routing_v1 -- --no-run`
5. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --no-run`
6. `cargo test -p shell --test agent_hub_trace_persistence -- --no-run`

Acceptance gate:

- tests can inspect typed member-dispatch payloads before runtime cutover,
- the builder contract is frozen for both worker lanes,
- the event payload shape required for remote readiness is frozen,
- Gate B passes,
- `.runs/plan-11/sentinels/task-m11-a3-shell-transport-harness-freeze.ok` exists.

### task/m11-b1-world-agent-member-manager

Ownership:

- worker lane `B1` only in `../substrate-m11-world-agent-member-manager`
- worker model: `GPT-5.4`, `reasoning_effort=high`

Allowed files:

- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
- new [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)

Forbidden touch surfaces:

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)
- docs and `.runs/plan-11/*`

Scope:

1. Branch [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs) on `member_dispatch` while preserving ordinary process execution unchanged.
2. Add new internal [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs) for Linux-first member runtime ownership.
3. Wire module exposure in [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs) only as needed.
4. Validate authoritative world binding remotely before startup.
5. Start in-world `run_control(...)`, retain cancel/event/completion ownership, emit `Start`, session-bearing `Event`, terminal `Exit` or `Error`, and register the span for cancel delivery.
6. Extend `/v1/execute/cancel` so it can deliver to member-dispatch spans with bounded wait behavior.
7. Add or extend [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs) for startup, cancel, bootstrap failure, and abnormal termination coverage.

Minimum worker test commands before handoff:

1. `cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture`
2. `cargo test -p world-agent -- --no-run`

Acceptance gate:

- typed member-dispatch startup and cancel work at the `world-agent` layer without the shell REPL in the loop,
- mismatched world binding fails closed,
- ordinary process streaming remains green,
- worker returns no contract change requests against frozen parent-owned files,
- `.runs/plan-11/sentinels/task-m11-b1-world-agent-member-manager.ok` is written only after parent acceptance.

Stop-back conditions for lane `B1`:

- any required rename of the ready-bearing event payload,
- any required change to `agent-api-types`, `world_ops.rs`, or shell harness files,
- any attempt to move canonical session-state writes into `world-agent`.

### task/m11-b2-shell-remote-member-cutover

Ownership:

- worker lane `B2` only in `../substrate-m11-shell-remote-member-cutover`
- worker model: `GPT-5.4`, `reasoning_effort=high`

Allowed files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

Forbidden touch surfaces:

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)
- docs and `.runs/plan-11/*`

Scope:

1. Replace the member launch path in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) so `start_member_runtime_with_prepared(...)` becomes the remote member-dispatch path instead of delegating to host orchestrator startup.
2. Introduce explicit `PreparedMemberDispatch` and explicit local-vs-remote retained-control representation.
3. Keep host orchestrator startup unchanged.
4. Persist member participants in `Allocating` before remote launch, then drive `Ready` and `Running` from remote stream events and retained ownership.
5. Route remote cancel through `/v1/execute/cancel` using the retained remote span id.
6. Reuse existing manifest, store, replacement, invalidation, and liveness helpers rather than inventing a second state machine.
7. Extend shell runtime tests in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) and integration tests in [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) for lazy first launch, same-generation reuse, preflight failure before transport, replacement launch, and non-live convergence on cancel or failure.

Minimum worker test commands before handoff:

1. `cargo test -p shell async_repl -- --nocapture`
2. `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
3. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --no-run`

Acceptance gate:

- first world-backed command launches the member through typed `/v1/execute/stream`,
- same-generation reuse remains intact,
- missing parent, binding, or selection still fails before any transport call,
- replacement launch crosses the same remote transport seam,
- worker returns no contract change requests against frozen parent-owned files,
- `.runs/plan-11/sentinels/task-m11-b2-shell-remote-member-cutover.ok` is written only after parent acceptance.

Stop-back conditions for lane `B2`:

- any need to reopen `agent-api-types`, `world_ops.rs`, `socket.rs`, or `repl_world_agent.rs`,
- any need to redesign store semantics,
- any need to treat remote control as a local cancel handle,
- any need to broaden scope into doctor/toolbox behavior before the regression wall.

### task/m11-c1-integrate-and-regression-wall

Ownership:

- parent only

Scope:

1. Integrate accepted outputs from lanes `B1` and `B2` in the parent checkout.
2. Resolve conflicts only in the parent branch.
3. Land the final status/trace/replacement regression wall in [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs) and [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs).
4. Touch [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) only if a real remotely launched member exposes operator-surface drift not already covered by existing logic.
5. Touch [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) only if one additive ownership marker is truly required by the remote-control split.
6. Keep [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) no-semantics-change by default.

Required final validation order:

1. `cargo test -p agent-api-types -- --nocapture`
2. `cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture`
3. `cargo test -p shell async_repl -- --nocapture`
4. `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
5. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
6. `cargo test -p shell --test agent_hub_trace_persistence -- --nocapture`
7. `cargo test -p world-agent -- --nocapture`
8. `cargo test -p shell -- --nocapture`
9. `cargo fmt --all -- --check`
10. `cargo clippy --workspace --all-targets -- -D warnings`

Acceptance gate:

- cancel reaches real member-dispatch spans,
- replacement launch preserves new `participant_id` and correct `resumed_from_participant_id`,
- status remains correct for a real remotely launched member,
- trace rows remain participant-correct and world-correct without reviving stale liveness,
- `.runs/plan-11/sentinels/task-m11-c1-integrate-and-regression-wall.ok` exists.

### task/m11-c2-docs-review

Ownership:

- parent only

Scope:

1. Review [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md) and [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md) only after the full regression wall is green.
2. Apply minimal wording updates only if shipped operator-visible transport behavior or trace examples changed.
3. Otherwise record an explicit no-change decision in `run-state.json` and `session.log`.

Acceptance gate:

- docs are either updated minimally and truthfully or explicitly left unchanged,
- no docs edit compensates for missing runtime or test proof,
- `.runs/plan-11/sentinels/task-m11-c2-docs-review.ok` exists.

### task/m11-c3-closeout

Ownership:

- parent only

Scope:

1. Confirm all required sentinels exist and `blocked.json` does not.
2. Finalize `run-state.json` with accepted decisions, escalation notes, and validation outcomes.
3. Write `.runs/plan-11/closeout.md` with branch state, worker-output disposition, tests run, and remaining deferred work limited to post-`PLAN-11` items.
4. Confirm no unresolved scope creep remains into scheduler, public control-plane APIs, auth-bundle redesign, or non-Linux parity.

Acceptance gate:

- Gate D passes,
- `.runs/plan-11/sentinels/task-m11-c3-closeout.ok` exists,
- `.runs/plan-11/closeout.md` exists.

## Context-Control Rules

1. The parent owns `.runs/plan-11/*`. Workers do not edit `run-state.json`, sentinels, `queue.json`, `session.log`, `blocked.json`, or `closeout.md`.
2. The parent keeps only the following live in working context:
   - the current task ID and gate state,
   - the frozen `member_dispatch` request contract,
   - the frozen `world_ops.rs` builder contract,
   - the frozen ready-bearing event contract used by `extract_session_handle_id(...)`,
   - the exact allowed and forbidden files for each worker lane,
   - the latest accepted worker summaries, narrow diffs, and blockers,
   - the required pre-worker and final validation orders.
3. Worker packets contain only:
   - the task ID,
   - the worktree path and branch name,
   - allowed files,
   - forbidden files,
   - frozen invariants,
   - stop-back conditions,
   - exact commands to run,
   - exact handoff format.
4. Each worker prompt must include the exact files it may touch and the exact files it must not touch. “Stay in your lane” is not implied; it is enumerated.
5. Workers must stop immediately if they need to touch any file outside their allowed list or if they need to reinterpret the frozen request, builder, or event contract.
6. Each worker must return:
   - a short result summary,
   - touched files,
   - exact commands run,
   - test outcomes,
   - blockers or unresolved assumptions,
   - a narrow diff summary tied to the touched files only,
   - an explicit statement of whether parent-owned follow-up is required.
7. Workers do not rebase, merge, integrate each other, or update parent run artifacts.
8. The parent reviews worker summaries plus narrow diffs, not broad restatements of repo context.
9. The parent merges accepted work locally, reruns required validations, and then closes the worker lane. Rejected or superseded worker outputs are closed without negotiation across worktrees.
10. If worker outputs conflict with current parent truth, the parent re-derives the correct result from production code and rewrites the patch locally instead of negotiating blended semantics across worktrees.

## Tests And Acceptance

### Pre-worker proof order

Run these parent-owned checks before dispatching the worker window:

1. `cargo test -p agent-api-types -- --nocapture`
2. `cargo test -p shell --lib -- --nocapture`
3. `cargo test -p world-agent --test streamed_execute_cancel_v1 -- --no-run`
4. `cargo test -p shell --test repl_world_first_routing_v1 -- --no-run`
5. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --no-run`
6. `cargo test -p shell --test agent_hub_trace_persistence -- --no-run`

### Final validation order

Run these commands in this exact order during `task/m11-c1-integrate-and-regression-wall`:

1. `cargo test -p agent-api-types -- --nocapture`
2. `cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture`
3. `cargo test -p shell async_repl -- --nocapture`
4. `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
5. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
6. `cargo test -p shell --test agent_hub_trace_persistence -- --nocapture`
7. `cargo test -p world-agent -- --nocapture`
8. `cargo test -p shell -- --nocapture`
9. `cargo fmt --all -- --check`
10. `cargo clippy --workspace --all-targets -- -D warnings`

### Acceptance checklist

#### Request contract

1. `ExecuteRequest.member_dispatch` is additive, typed, and validated at the request boundary.
2. Ordinary process exec still uses non-empty `cmd` with `member_dispatch=None`.
3. Typed member dispatch requires empty `cmd`, `pty=false`, and a fully formed identity tuple.
4. Invalid mixed shapes fail at parse/deserialize time rather than in deep runtime handlers.
5. Top-level `agent_id` remains authoritative and is not duplicated in the member-dispatch payload.

#### Shell transport and harness freeze

1. `world_ops.rs` builds the typed member-dispatch request without inventing a second transport stack.
2. `socket.rs` can deserialize and inspect recorded `member_dispatch` payloads.
3. `repl_world_agent.rs` can capture typed payloads and script ready, cancel, success, and failure flows.
4. The ready-bearing remote `Event` contract is frozen well enough for shell readiness gating.
5. Worker lanes begin only after these conditions are proven by the pre-worker proof order.

#### World-agent manager

1. `world-agent` branches `execute_stream(...)` on `member_dispatch` while preserving ordinary process streaming.
2. The new member runtime manager validates world binding before startup.
3. Remote member startup emits `Start`, a session-bearing `Event`, and terminal `Exit` or `Error`.
4. `/v1/execute/cancel` can deliver to a live member-dispatch span.
5. Bootstrap failure and abnormal termination clean up honestly and do not leave a live claim behind.

#### Shell remote cutover

1. The member path in `async_repl.rs` no longer reuses local host orchestrator startup.
2. The shell uses an explicit local-vs-remote retained-control split.
3. The shell uses an explicit `PreparedMemberDispatch` or equivalent dedicated remote-prepared shape.
4. The shell persists `Allocating` before launch and only transitions to `Ready` and `Running` after remote ownership is retained.
5. Same-generation reuse remains intact and preflight failures still occur before any transport call.

#### Regression wall

1. Cancel, replacement, status, and trace correctness are proven after both runtime halves are integrated.
2. Replacement launch on world-generation rollover uses the same typed transport seam.
3. Replacement preserves fresh `participant_id` and correct `resumed_from_participant_id`.
4. Failed replacement leaves honest absence rather than stale liveness.
5. Trace remains auditable without becoming current liveness authority.

#### Operator surfaces

1. `substrate agent status --json` reflects the real remotely launched member from shell authority.
2. `status` retains top-level `world_id` and `world_generation`.
3. `toolbox` remains orchestrator-anchored in this slice.
4. `doctor` and status behavior do not guess remote truth from trace or unmanaged remote state.
5. Any operator-surface code change stays minimal and parent-owned.

#### Workspace and scope boundary

1. No new public API family is introduced.
2. No scheduler, selector UX, auth-bundle redesign, or non-Linux implementation is introduced.
3. Docs remain late and optional.
4. `.runs/plan-11/*` remains parent-owned only.
5. All work stays inside the `PLAN-11` architecture contract and the listed file boundaries.

## Merge Refusal Rules

The parent refuses to merge a worker output if any of these are true:

1. The patch edits a file outside the task’s allowed file list.
2. The patch reopens the frozen request, builder, or ready-bearing event contract without parent re-freeze.
3. The patch authorizes host fallback for a world-scoped member.
4. The patch treats remote control state as if it were a local cancel handle.
5. The patch weakens fail-closed behavior for binding mismatch, missing parent, cancel failure, or replacement failure.
6. The patch requires concurrent edits to parent-owned freeze files or late regression files to become intelligible.
7. The patch omits test evidence for the behavior it claims to cover.
8. The patch broadens scope into new public APIs, scheduler work, early docs work, or non-Linux parity.

## Run Exit Criteria

### Successful run

The run is successful only if all of these are true:

1. `.runs/plan-11/sentinels/task-m11-a1-preflight.ok` exists.
2. `.runs/plan-11/sentinels/task-m11-a2-request-boundary-contract-freeze.ok` exists.
3. `.runs/plan-11/sentinels/task-m11-a3-shell-transport-harness-freeze.ok` exists.
4. `.runs/plan-11/sentinels/task-m11-b1-world-agent-member-manager.ok` exists.
5. `.runs/plan-11/sentinels/task-m11-b2-shell-remote-member-cutover.ok` exists.
6. `.runs/plan-11/sentinels/task-m11-c1-integrate-and-regression-wall.ok` exists.
7. `.runs/plan-11/sentinels/task-m11-c2-docs-review.ok` exists.
8. `.runs/plan-11/sentinels/task-m11-c3-closeout.ok` exists.
9. `.runs/plan-11/run-state.json` exists and records a completed terminal state.
10. `.runs/plan-11/queue.json` and `.runs/plan-11/session.log` exist.
11. `.runs/plan-11/closeout.md` exists and matches the final accepted branch state.
12. `.runs/plan-11/blocked.json` does not exist.

### Blocked termination

The run terminates as blocked only if all of these are true:

1. `.runs/plan-11/blocked.json` exists with the triggering stop condition and the current task ID.
2. no later-phase sentinel is written after the blocking condition is detected,
3. `run-state.json` records the blocked terminal state,
4. partial worker outputs are either rejected or explicitly quarantined in `session.log`,
5. no fake completion signal is written to `.runs/plan-11/closeout.md`.
