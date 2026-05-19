# ORCH_PLAN-11: Refreeze Member Dispatch Runtime Descriptor And Complete In-World Cutover

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN-11.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-11.md)  
Reference style source: [ORCH_PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-10.md)  
Blocked-run evidence: [.runs/plan-11/run-state.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/run-state.json), [.runs/plan-11/blocked.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/blocked.json)  
Execution type: shell/world-agent transport-backed member-placement cutover orchestration plan, Linux-first, no UI scope, strong runtime/status/trace scope

## Summary

This run replaces the blocked `ORCH_PLAN-11` execution with a fresh parent-runner plan that starts
from the actual blocked state, explicitly invalidates the old Gate B freeze, refreezes
`MemberDispatchRequestV1` with the required `resolved_runtime` descriptor, refreezes the shell
builder and harness seam including the `dispatch/prelude.rs` export, then opens exactly one real
parallel runtime window with an exact active worker cap of `2`.

This slice is not a second resolver, not a new public API family, not a scheduler, not a
shared-crate extraction project, and not a docs-first cleanup run. It is one bounded production
cutover:

1. import the blocked-run truth and supersede the old frozen request contract,
2. refreeze the typed `ExecuteRequest.member_dispatch` contract in
   [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
   by adding required `resolved_runtime`,
3. refreeze the shell builder and harness seam in
   [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs),
   [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs),
   [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs),
   and
   [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs),
4. run one honest two-lane runtime window:
   - lane `B1` implements in-world UAA member startup in `world-agent`, including a direct
     `unified-agent-api` dependency,
   - lane `B2` cuts the shell member path over to explicit remote retained control and the frozen
     typed builder,
5. close the parent-only regression wall for cancel, replacement, status, and trace correctness,
6. review docs only after the runtime and test wall are green.

The prior run blocked because Gate B froze too early. The frozen request lacked
`resolved_runtime`, so `world-agent` could not honestly reconstruct the shell-owned UAA startup
path without inventing a second resolver, and the shell lane also lacked the parent-fixable
builder export seam through `dispatch/prelude.rs`. This plan prevents that failure mode by making
both fixes mandatory in the parent-only refreeze lane before any worker starts.

Parent-owned critical path:

1. `task/m11-a1-preflight-and-blocked-import`
2. `task/m11-a2-member-dispatch-contract-refreeze`
3. `task/m11-a3-builder-export-and-harness-refreeze`
4. `task/m11-c1-integrate-and-regression-wall`
5. `task/m11-c2-docs-review`
6. `task/m11-c3-closeout`

Every child worker uses `GPT-5.4` with `reasoning_effort=high`.

## Hard Guards

### Locked invariants

1. `PLAN-11` remains a transport-backed placement-honesty slice only. No UI work is authorized.
2. `POST /v1/execute/stream` and `POST /v1/execute/cancel` remain the only transport seam.
3. `ExecuteRequest.member_dispatch` remains additive, typed, and internal. No magic command
   string or new target enum is authorized.
4. The contract refreeze stays on `MemberDispatchRequestV1`. No `V2` rename is authorized for this
   unshipped internal seam.
5. `MemberDispatchRequestV1` must carry required
   `resolved_runtime: ResolvedMemberRuntimeDescriptorV1`.
6. `ResolvedMemberRuntimeDescriptorV1` contains exactly:
   - `backend_kind`
   - `binary_path`
7. `backend_kind` is explicit. No inference from `backend_id` is allowed.
8. `binary_path` is the already-resolved absolute path selected by the shell. `world-agent` must
   consume it and must not re-resolve from inventory, config, or a second selector.
9. Request-boundary validation lives in
   [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs),
   not in deep handler code.
10. For typed member dispatch, `cmd.trim().is_empty()` must be true and `pty` must be false. For
    ordinary process exec, `member_dispatch` must be `None`.
11. The top-level `agent_id` remains authoritative for budgets, traces, and diagnostics.
    `member_dispatch` must not duplicate it.
12. The shell remains the only authority for canonical session-root writes, persisted participant
    state, `status`, `doctor`, and toolbox surfaces.
13. `world-agent` owns in-world member execution, retained remote cancel delivery, remote event
    streaming, and completion observation only.
14. `world-agent` must take a direct `unified-agent-api` dependency in
    [crates/world-agent/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/Cargo.toml).
    No new shared crate extraction is authorized in this slice.
15. The shell must represent retained control explicitly as local vs remote. Remote member control
    must not be stored as a local `RetainedRunControl`.
16. The shell must use an explicit `PreparedMemberDispatch` or equivalent dedicated remote launch
    shape. Host orchestrator startup stays on the existing local path.
17. The frozen builder must be re-exported through
    [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)
    before the shell runtime lane opens.
18. A world-scoped member launch fails closed on remote dispatch failure. Host fallback is
    forbidden.
19. The authoritative `world_id` and `world_generation` supplied by the shell must be validated
    remotely and rejected on mismatch.
20. `ExecuteStreamFrame::{Start,Event,Exit,Error}` remain the only stream families.
21. Remote readiness remains gated by the existing session-handle event contract unless the parent
    explicitly refreezes it before worker lanes open.
22. A member persists as `Allocating` before remote launch and may advertise live only after
    session-handle evidence, retained ownership, active event stream, and completion observation
    all exist.
23. `substrate agent status --json` and trace rows must remain producer-backed and
    participant-correct after cancel and replacement. Stale liveness must not revive.
24. Linux-first is explicit. Non-Linux member dispatch paths fail closed in this slice.
25. Docs are late and optional only after the regression wall is green.
26. Package-targeted cargo commands use the actual package names in this repo:
    `agent-api-types`, `world-agent`, and `shell`.
27. The parent is the only writer of `.runs/plan-11/*`.

### File-level boundaries

Parent-owned serialized refreeze surfaces:

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)
- [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)

Worker-safe world-agent lane after the refreeze:

- [crates/world-agent/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/Cargo.toml)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
- new [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)

Worker-safe shell runtime lane after the refreeze:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

Parent-owned late regression and drift-closure surfaces:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)

Parent-owned escalation-only production surfaces:

- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Read-for-truth only:

- [PLAN-11.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-11.md)
- [ORCH_PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-10.md)
- [.runs/plan-11/run-state.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/run-state.json)
- [.runs/plan-11/blocked.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/blocked.json)

Optional parent-owned docs surface only after the regression wall is green:

- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)

### Non-negotiable stop conditions

Stop the run and write `.runs/plan-11/blocked.json` if any of these occur:

1. A worker lane opens before the parent lands the `resolved_runtime` refreeze and the
   `dispatch/prelude.rs` builder export.
2. Any task requires reopening the refrozen request contract after either worker lane has started.
3. Any task requires `world-agent` to infer runtime selection from `backend_id`, protocol, agent
   inventory, or shell-private selectors instead of consuming `resolved_runtime`.
4. Any task requires extracting a new shared startup crate to finish `PLAN-11`.
5. Any task falls back to host launch after remote world-member dispatch fails.
6. Any task keeps the shell member path on local `gateway.run_control(...)` while claiming world
   placement.
7. Any task treats remote control state as if it were a local `RetainedRunControl`.
8. Any task introduces a second lifecycle model instead of reusing the existing
   manifest/store/liveness contract.
9. Any worker lane needs to touch a parent-owned refreeze file, late regression file, docs, or
   `.runs/plan-11/*` to finish its assignment.
10. The world-agent lane needs to rename or redesign the session-bearing readiness `Event`
    contract after the shell lane starts.
11. Any task proves `status` or trace correctness depends on heuristic trace-first liveness
    recovery.
12. Any task requires early docs edits to explain behavior that runtime and tests still do not
    prove.

## Orchestration State Surfaces

### Canonical run state

The only canonical source of truth for run orchestration state:

- `.runs/plan-11/run-state.json`

Parent-only writes to this file. It tracks:

- current phase,
- active task IDs,
- branch and worktree assignment,
- gate status,
- the prior blocked freeze as superseded evidence,
- the refrozen `member_dispatch` request contract,
- the refrozen builder-export and harness contract,
- the refrozen session-handle event contract,
- the refrozen local-vs-remote retained-control contract,
- accepted and rejected worker outputs,
- escalation-file usage,
- blocked or completed terminal state,
- final closeout pointer.

If a worker report conflicts with `run-state.json`, the parent treats `run-state.json` as
authoritative until it explicitly reconciles the discrepancy.

### Derived run artifacts

The parent may maintain these local artifacts:

- `.runs/plan-11/queue.json`
- `.runs/plan-11/session.log`
- `.runs/plan-11/sentinels/task-m11-a1-preflight-and-blocked-import.ok`
- `.runs/plan-11/sentinels/task-m11-a2-member-dispatch-contract-refreeze.ok`
- `.runs/plan-11/sentinels/task-m11-a3-builder-export-and-harness-refreeze.ok`
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
5. Worker notes, commits, or branch state never replace parent-written sentinels or
   `run-state.json`.

## Concurrency Policy

1. The parent is the only integrator.
2. The parent is the only writer of final branch state on `feat/session-centric-state-store`.
3. Exact active worker cap: `2`.
4. There are zero worker lanes during the refreeze lane:
   - `task/m11-a1-preflight-and-blocked-import`
   - `task/m11-a2-member-dispatch-contract-refreeze`
   - `task/m11-a3-builder-export-and-harness-refreeze`
5. The only honest parallel window is:
   - `task/m11-b1-world-agent-member-manager`
   - `task/m11-b2-shell-remote-member-cutover`
6. The regression wall is parent-only again after both runtime lanes return.
7. No worker may edit any parent-owned refreeze file, any late regression file, docs, or
   `.runs/plan-11/*`.
8. Both worker worktrees are seeded from the exact post-`task/m11-a3` tree.
9. If either worker proves a missing contract change is required in a frozen parent-owned file,
   that worker stops and hands the gap back to the parent.
10. Worker coordination uses sentinels and long waits. Tight polling loops against git state or
    run-state are forbidden.

### Why the worker cap remains exactly `2`

The worker cap remains exactly `2` because `PLAN-11` has only two honest runtime seams after the
parent-owned refreeze and no third independent lane before the regression wall:

1. the request contract, builder export, and harness/event seam are one coupled parent-owned
   freeze,
2. there is one real `world-agent` runtime lane after that freeze,
3. there is one real shell remote-cutover lane after that freeze,
4. status, trace, replacement, and operator-surface correctness depend on integrated behavior of
   those two runtime halves,
5. a third worker before `task/m11-c1` would only create contract churn, merge noise, and a
   higher chance of repeating the blocked refreeze failure.

## Approval And Gate Model

There are no human approval gates defined for this run.

Replacement control mechanism:

1. parent validation gates,
2. parent-written sentinels,
3. `session.log` for acceptance and rejection rationale,
4. `blocked.json` for hard-stop termination,
5. `closeout.md` for successful completion.

### Gate A: Blocked-State Import And Topology Lock

Required before implementation starts:

- parent re-reads [PLAN-11.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-11.md),
  [ORCH_PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-10.md),
  [.runs/plan-11/run-state.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/run-state.json),
  and [.runs/plan-11/blocked.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/blocked.json),
- parent records the exact execution shape: parent-only refreeze lane, one real parallel runtime
  window, parent-only regression wall,
- parent records the exact worker cap of `2`,
- parent records the actual blocked reason: missing `resolved_runtime` and missing prelude export
  seam,
- parent records the package-name normalization: `agent-api-types`, `world-agent`, `shell`.

### Gate B: Request, Builder, And Harness Refreeze

Required before either worker lane opens:

- `MemberDispatchRequestV1` is refrozen with required `resolved_runtime`,
- request-boundary validation rules are refrozen,
- `world_ops.rs` carries the refrozen `resolved_runtime` fields,
- `dispatch/prelude.rs` re-exports the frozen builder,
- `socket.rs` and `repl_world_agent.rs` assert the refrozen payload shape,
- the session-bearing remote readiness event remains frozen well enough for
  `extract_session_handle_id(...)` or an explicitly parent-approved equivalent,
- the parent has completed the pre-worker proof order,
- both worker worktrees are seeded from the exact post-`task/m11-a3` tree.

### Gate C: Runtime Seam Integration

Required before the final regression wall starts:

- both worker outputs are integrated by the parent,
- any approved escalation into
  [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs),
  [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs),
  or
  [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
  is already complete and justified in `run-state.json`,
- host orchestrator startup remains unchanged,
- no production path still routes a world-member startup through local `gateway.run_control(...)`,
- `world-agent` consumes the transport-carried `resolved_runtime` instead of any second resolver.

### Gate D: Final Acceptance

Required before closeout:

- the integrated runtime uses typed execute-stream member dispatch with `resolved_runtime`,
- cancel, replacement, status, and trace regression tests are green in the required order,
- docs review is complete and either produced aligned late edits or an explicit no-change
  decision,
- `run-state.json` records the final request, builder, remote-control, replacement, and
  regression contracts.

## Workstream Plan

### Worktree topology

Parent checkout:

- current checkout on `feat/session-centric-state-store`

Child worktrees and branches:

- `../substrate-m11-world-agent-member-manager`
  - `codex/feat-session-centric-state-store-m11-world-agent-member-manager`
- `../substrate-m11-shell-remote-member-cutover`
  - `codex/feat-session-centric-state-store-m11-shell-remote-member-cutover`

Suggested creation commands:

```bash
git worktree add ../substrate-m11-world-agent-member-manager \
  -b codex/feat-session-centric-state-store-m11-world-agent-member-manager \
  feat/session-centric-state-store
git worktree add ../substrate-m11-shell-remote-member-cutover \
  -b codex/feat-session-centric-state-store-m11-shell-remote-member-cutover \
  feat/session-centric-state-store
```

Execution topology:

1. The parent checkout serves every refreeze, integration, and final validation task.
2. Exactly two child worktrees exist because exactly two worker-safe runtime seams exist after the
   refreeze.
3. No third child worktree is authorized because no earlier seam can be parallelized honestly
   without repeating the blocked contract failure.

Subagents do not merge each other’s work. They return only changed files, exact commands run, test
results, and blockers to the parent.

### Task graph

Execution graph for the run:

1. `task/m11-a1-preflight-and-blocked-import`
2. `task/m11-a2-member-dispatch-contract-refreeze`
3. `task/m11-a3-builder-export-and-harness-refreeze`
4. `task/m11-b1-world-agent-member-manager` and `task/m11-b2-shell-remote-member-cutover` in
   parallel
5. `task/m11-c1-integrate-and-regression-wall`
6. `task/m11-c2-docs-review`
7. `task/m11-c3-closeout`

Parent-only serialized tasks:

- `task/m11-a1-preflight-and-blocked-import`
- `task/m11-a2-member-dispatch-contract-refreeze`
- `task/m11-a3-builder-export-and-harness-refreeze`
- `task/m11-c1-integrate-and-regression-wall`
- `task/m11-c2-docs-review`
- `task/m11-c3-closeout`

Worker-owned tasks:

- `task/m11-b1-world-agent-member-manager`
- `task/m11-b2-shell-remote-member-cutover`

## Parallel Window B

This is the only worker window in the run.

It opens only after Gate B passes and after the parent has landed the refreeze that the prior run
was missing. There is no earlier honest worker window because:

1. [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
   defines the request contract the runtime lanes share,
2. [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
   and
   [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)
   define the only allowed builder seam for the shell lane,
3. [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs)
   and
   [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
   define what the runtime lanes can safely assume about captured payloads and readiness events,
4. the blocked run already proved that splitting before this refreeze creates fake progress and a
   guaranteed re-open.

After the refreeze, the runtime work splits honestly:

- lane `B1` owns `world-agent` execution, UAA startup, span registration, cancel delivery, and
  streaming,
- lane `B2` owns shell retained-control representation, remote launch cutover, same-generation
  reuse, and replacement convergence,
- neither lane may reopen the typed request, builder, or readiness-event contract.

### task/m11-a1-preflight-and-blocked-import

Ownership:

- parent only

Scope:

1. Re-read [PLAN-11.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-11.md),
   [ORCH_PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-10.md),
   [.runs/plan-11/run-state.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/run-state.json),
   and [.runs/plan-11/blocked.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/blocked.json).
2. Confirm the run executes on `feat/session-centric-state-store`.
3. Import the blocked-run truth into the new run packet and mark the old Gate B freeze as
   superseded, not reusable.
4. Initialize or refresh:
   - `.runs/plan-11/run-state.json`
   - `.runs/plan-11/queue.json`
   - `.runs/plan-11/session.log`
5. Record the no-fallback, shell-authority, Linux-first, docs-late, and exact `2`-worker posture.
6. Record the parent-owned, worker-owned, and escalation-only file boundaries.

Commands:

1. `git branch --show-current`
2. `mkdir -p .runs/plan-11/sentinels`

Acceptance:

1. The parent can restate the exact blocked reason without ambiguity:
   - missing `resolved_runtime`
   - missing `dispatch/prelude.rs` export seam
2. `run-state.json` records that the old request freeze is superseded and Gate B must be rerun.
3. The parent can name the only honest worker window: after `task/m11-a3`.
4. Gate A passes.

Green-path output:

- `.runs/plan-11/sentinels/task-m11-a1-preflight-and-blocked-import.ok`

Blocked-path output:

- `.runs/plan-11/blocked.json`

### Parent validation gate A

Required before `task/m11-a2-member-dispatch-contract-refreeze` starts:

1. No invariant contradiction remains unresolved.
2. The parent can explain why the prior Gate B freeze is invalid and cannot be reused.
3. The parent can explain why docs stay deferred until after the runtime and regression wall.

### task/m11-a2-member-dispatch-contract-refreeze

Ownership:

- parent only

Allowed files:

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)

Scope:

1. Refreeze additive `MemberDispatchRequestV1` and `ExecuteRequest.member_dispatch` by adding:
   - required `resolved_runtime: ResolvedMemberRuntimeDescriptorV1`
   - `ResolvedMemberRuntimeDescriptorV1.backend_kind`
   - `ResolvedMemberRuntimeDescriptorV1.binary_path`
2. Keep the internal type name `MemberDispatchRequestV1`.
3. Implement parse-boundary validation for `backend_kind`, `binary_path`, and the existing
   mutually exclusive `cmd` / `pty` / `member_dispatch` rules.
4. Preserve the authoritative identity tuple carried by member dispatch:
   - orchestration session,
   - participant lineage,
   - backend identity,
   - protocol,
   - run id,
   - world id,
   - world generation,
   - resolved runtime.
5. Update round-trip and invalid-shape tests so ordinary process exec remains unchanged and missing
   `resolved_runtime` or invalid mixed shapes fail at parse time.

Must not do:

1. No `V2` transport family.
2. No generic execute target enum.
3. No shell or `world-agent` handler logic in this task.

Commands:

1. `cargo test -p agent-api-types -- --nocapture`
2. `cargo test -p world-agent --test streamed_execute_cancel_v1 -- --no-run`
3. `cargo test -p shell --test repl_world_first_routing_v1 -- --no-run`
4. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --no-run`
5. `cargo test -p shell --test agent_hub_trace_persistence -- --no-run`

Acceptance:

1. The request contract now carries the exact resolved launch facts `world-agent` needs.
2. Invalid `cmd` plus `member_dispatch` shapes fail at the request boundary.
3. Missing or malformed `resolved_runtime` fails at the request boundary.
4. Downstream packages still compile against the additive request shape.
5. The parent records the accepted refrozen schema in `run-state.json`.

Green-path output:

- `.runs/plan-11/sentinels/task-m11-a2-member-dispatch-contract-refreeze.ok`

Blocked-path output:

- `.runs/plan-11/blocked.json`

### Parent validation gate B1

Required before `task/m11-a3-builder-export-and-harness-refreeze` starts:

1. `cargo test -p agent-api-types -- --nocapture` passes.
2. `run-state.json` records the refrozen `resolved_runtime` contract.
3. No downstream task may remove or re-interpret `resolved_runtime`.

### task/m11-a3-builder-export-and-harness-refreeze

Ownership:

- parent only

Allowed files:

- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)
- [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)

Scope:

1. Refreeze the dedicated member-dispatch builder in
   [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
   so it carries `resolved_runtime` and keeps existing env and trace plumbing.
2. Re-export `build_agent_client_and_member_dispatch_request(...)` through
   [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs).
3. Extend [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs)
   so recorded execute-stream payloads deserialize and assert nested `resolved_runtime`.
4. Extend
   [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
   so typed member-dispatch payloads capture nested `resolved_runtime` while keeping accepted
   ready, cancel, success, and failure scripting contracts.
5. Freeze the remote-ready event compatibility required by
   `extract_session_handle_id(...)` in
   [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).

Must not do:

1. No runtime lifecycle logic in `world_ops.rs`.
2. No widening of the public routing surface beyond the one builder export the shell lane needs.
3. No change to the ready-event schema unless the parent explicitly records a replacement
   contract before worker lanes open.

Commands:

1. `cargo test -p agent-api-types -- --nocapture`
2. `cargo test -p shell --lib -- --nocapture`
3. `cargo test -p world-agent --test streamed_execute_cancel_v1 -- --no-run`
4. `cargo test -p shell --test repl_world_first_routing_v1 -- --no-run`
5. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --no-run`
6. `cargo test -p shell --test agent_hub_trace_persistence -- --no-run`

Acceptance:

1. The shell lane can import the frozen builder through the normal routing seam.
2. Tests can inspect typed member-dispatch payloads with nested `resolved_runtime`.
3. The builder contract is frozen for both worker lanes.
4. The ready-event contract remains frozen and testable.
5. Gate B passes.

Green-path output:

- `.runs/plan-11/sentinels/task-m11-a3-builder-export-and-harness-refreeze.ok`

Blocked-path output:

- `.runs/plan-11/blocked.json`

### Parent validation gate B2

Required before the worker window opens:

1. The full pre-worker proof order passes.
2. `run-state.json` records the refrozen request, builder export, and readiness-event contracts.
3. The parent seeds both worker worktrees from the exact post-`task/m11-a3` tree.

### task/m11-b1-world-agent-member-manager

Ownership:

- worker lane `B1` only in `../substrate-m11-world-agent-member-manager`
- worker model: `GPT-5.4`, `reasoning_effort=high`

Allowed files:

- [crates/world-agent/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/Cargo.toml)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
- new [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)

Forbidden touch surfaces:

- every parent-owned refreeze file,
- every shell runtime-lane file,
- every parent-owned late regression file,
- docs and `.runs/plan-11/*`.

Scope:

1. Add direct `unified-agent-api` dependency in
   [crates/world-agent/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/Cargo.toml).
2. Branch
   [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
   on `member_dispatch` while preserving ordinary process execution unchanged.
3. Add new internal
   [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
   that:
   - validates `resolved_runtime`,
   - validates authoritative world binding,
   - builds the UAA backend from `backend_kind` plus `binary_path`,
   - starts in-world retained `run_control(...)`,
   - retains cancel, event, and completion ownership,
   - emits `Start`, session-bearing `Event`, and terminal `Exit` or `Error`,
   - cleans up terminal state honestly.
4. Wire minimal module exposure in
   [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs).
5. Extend `/v1/execute/cancel` so it can deliver to member-dispatch spans with bounded wait
   behavior.
6. Extend
   [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)
   for:
   - success,
   - unsupported backend kind,
   - missing binary path,
   - binding mismatch,
   - cancel,
   - bootstrap failure,
   - abnormal termination.

Must not do:

1. No backend-specific stdout parsing as readiness truth.
2. No runtime re-resolution from inventory or config.
3. No shell-private helper dependency.
4. No canonical session-state writes in `world-agent`.

Minimum worker test commands before handoff:

1. `cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture`
2. `cargo test -p world-agent -- --no-run`

Acceptance:

1. Typed member-dispatch startup and cancel work at the `world-agent` layer without the shell REPL
   in the loop.
2. `world-agent` consumes the transport-carried `resolved_runtime` and does not invent a second
   resolver.
3. Mismatched world binding and unsupported backend fail closed.
4. Ordinary process streaming remains green.
5. Worker returns no contract-change request against a frozen parent-owned file.

Stop-back conditions for lane `B1`:

1. Any required change to `agent-api-types`, `world_ops.rs`, `dispatch/prelude.rs`, `socket.rs`,
   or `repl_world_agent.rs`.
2. Any required rename of the readiness-bearing `Event` contract.
3. Any need to extract a shared startup crate in this slice.
4. Any need to move canonical session-state writes into `world-agent`.

Green-path output:

- `.runs/plan-11/sentinels/task-m11-b1-world-agent-member-manager.ok` after parent acceptance

### task/m11-b2-shell-remote-member-cutover

Ownership:

- worker lane `B2` only in `../substrate-m11-shell-remote-member-cutover`
- worker model: `GPT-5.4`, `reasoning_effort=high`

Allowed files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

Forbidden touch surfaces:

- every parent-owned refreeze file,
- every world-agent lane file,
- every parent-owned late regression file,
- docs and `.runs/plan-11/*`.

Scope:

1. Replace the member launch path in
   [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
   so the member startup path uses the refrozen typed builder through `dispatch/prelude.rs`
   instead of host orchestrator startup.
2. Introduce explicit `PreparedMemberDispatch` and explicit local-vs-remote retained-control
   representation.
3. Keep host orchestrator startup unchanged.
4. Persist member participants in `Allocating` before remote launch, then drive `Ready` and
   `Running` from remote stream events plus retained ownership.
5. Route remote cancel through `/v1/execute/cancel` using the retained remote span id.
6. Reuse existing manifest, store, replacement, invalidation, and liveness helpers rather than
   inventing a second state machine.
7. Extend shell tests in
   [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
   and
   [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
   for:
   - first world-backed launch over typed execute-stream,
   - same-generation reuse,
   - preflight failure before transport,
   - replacement launch,
   - failed replacement leaving honest absence,
   - non-live convergence on cancel or failure.

Must not do:

1. No local `gateway.run_control(...)` for world-member startup.
2. No treating remote control as a local cancel handle.
3. No redesign of store semantics.
4. No broadening into doctor/toolbox work before the regression wall.

Minimum worker test commands before handoff:

1. `cargo test -p shell async_repl -- --nocapture`
2. `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
3. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --no-run`

Acceptance:

1. The first world-backed command launches the member through typed `/v1/execute/stream`.
2. Same-generation reuse remains intact.
3. Missing parent, binding, or selection still fails before any transport call.
4. Replacement launch crosses the same remote transport seam.
5. Worker returns no contract-change request against a frozen parent-owned file.

Stop-back conditions for lane `B2`:

1. Any need to reopen `agent-api-types`, `world_ops.rs`, `dispatch/prelude.rs`, `socket.rs`, or
   `repl_world_agent.rs`.
2. Any need to redesign store semantics.
3. Any need to treat remote control as a local cancel handle.
4. Any need to broaden scope into doctor/toolbox behavior before the regression wall.

Green-path output:

- `.runs/plan-11/sentinels/task-m11-b2-shell-remote-member-cutover.ok` after parent acceptance

### task/m11-c1-integrate-and-regression-wall

Ownership:

- parent only

Scope:

1. Integrate accepted outputs from lanes `B1` and `B2` in the parent checkout.
2. Resolve conflicts only in the parent branch.
3. Land the final status/trace/replacement regression wall in:
   - [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
   - [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)
4. Touch
   [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
   only if a real remotely launched member exposes operator-surface drift not already covered by
   existing logic.
5. Touch
   [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
   only if one additive ownership marker is truly required by the remote-control split.
6. Keep
   [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
   no-semantics-change by default.

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

Acceptance:

1. Cancel reaches real member-dispatch spans.
2. Replacement launch preserves fresh `participant_id` and correct
   `resumed_from_participant_id`.
3. Status remains correct for a real remotely launched member.
4. Trace rows remain participant-correct and world-correct without reviving stale liveness.
5. Gate C passes.

Green-path output:

- `.runs/plan-11/sentinels/task-m11-c1-integrate-and-regression-wall.ok`

Blocked-path output:

- `.runs/plan-11/blocked.json`

### task/m11-c2-docs-review

Ownership:

- parent only

Scope:

1. Review [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
   and [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
   only after the full regression wall is green.
2. Apply minimal wording updates only if shipped operator-visible transport behavior or trace
   examples changed.
3. Otherwise record an explicit no-change decision in `run-state.json` and `session.log`.

Acceptance:

1. Docs are either updated minimally and truthfully or explicitly left unchanged.
2. No docs edit compensates for missing runtime or test proof.

Green-path output:

- `.runs/plan-11/sentinels/task-m11-c2-docs-review.ok`

Blocked-path output:

- `.runs/plan-11/blocked.json`

### task/m11-c3-closeout

Ownership:

- parent only

Scope:

1. Confirm all required sentinels exist and `blocked.json` does not.
2. Finalize `run-state.json` with accepted decisions, superseded blocked-run notes, escalation
   notes, and validation outcomes.
3. Write `.runs/plan-11/closeout.md` with branch state, worker-output disposition, tests run, and
   remaining deferred work limited to post-`PLAN-11` items.
4. Confirm no unresolved scope creep remains into shared-crate extraction, scheduler work, public
   control-plane APIs, auth-bundle redesign, or non-Linux parity.

Acceptance:

1. Gate D passes.
2. `.runs/plan-11/closeout.md` exists.
3. The final run-state records the refrozen contract that avoided the prior blocked failure.

Green-path output:

- `.runs/plan-11/sentinels/task-m11-c3-closeout.ok`

## Context-Control Rules

1. The parent owns `.runs/plan-11/*`. Workers do not edit `run-state.json`, sentinels,
   `queue.json`, `session.log`, `blocked.json`, or `closeout.md`.
2. The parent keeps only the following live in working context:
   - current task ID and gate state,
   - the refrozen `member_dispatch` request contract,
   - the refrozen builder-export contract,
   - the refrozen readiness-event contract used by `extract_session_handle_id(...)`,
   - the exact allowed and forbidden files for each worker lane,
   - the latest accepted worker summaries, narrow diffs, and blockers,
   - the required pre-worker and final validation orders.
3. Worker packets contain only:
   - task ID,
   - worktree path and branch name,
   - allowed files,
   - forbidden files,
   - frozen invariants,
   - stop-back conditions,
   - exact commands to run,
   - exact handoff format.
4. Each worker prompt must enumerate the exact files it may touch and the exact files it must not
   touch.
5. Workers must stop immediately if they need to touch any file outside their allowed list or if
   they need to reinterpret the frozen request, builder, or event contract.
6. Each worker must return:
   - short result summary,
   - touched files,
   - exact commands run,
   - test outcomes,
   - blockers or unresolved assumptions,
   - narrow diff summary tied only to the touched files,
   - explicit statement of whether parent follow-up is required.
7. Workers do not rebase, merge, integrate each other, or update parent run artifacts.
8. The parent reviews worker summaries plus narrow diffs, not broad repo restatements.
9. The parent merges accepted work locally, reruns required validations, and then closes the lane.
10. If worker outputs conflict with current parent truth, the parent re-derives the correct result
    from production code and rewrites the patch locally instead of negotiating blended semantics
    across worktrees.

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

#### Refrozen request contract

1. `ExecuteRequest.member_dispatch` remains additive, typed, and validated at the request
   boundary.
2. `MemberDispatchRequestV1` now requires `resolved_runtime`.
3. `ResolvedMemberRuntimeDescriptorV1` carries explicit `backend_kind` and absolute
   `binary_path`.
4. Ordinary process exec still uses non-empty `cmd` with `member_dispatch=None`.
5. Typed member dispatch requires empty `cmd`, `pty=false`, and a fully formed identity plus
   resolved-runtime tuple.
6. Invalid mixed shapes and invalid `resolved_runtime` fail at parse time rather than in deep
   runtime handlers.
7. Top-level `agent_id` remains authoritative and is not duplicated inside `member_dispatch`.

#### Builder export and harness refreeze

1. `world_ops.rs` builds the typed member-dispatch request without inventing a second transport
   stack.
2. `dispatch/prelude.rs` re-exports the frozen builder required by the shell lane.
3. `socket.rs` can deserialize and inspect recorded `member_dispatch` payloads including nested
   `resolved_runtime`.
4. `repl_world_agent.rs` can capture typed payloads and script ready, cancel, success, and
   failure flows.
5. Worker lanes begin only after these conditions are proven by the pre-worker proof order.

#### World-agent runtime lane

1. `world-agent` branches `execute_stream(...)` on `member_dispatch` while preserving ordinary
   process streaming.
2. `world-agent` depends directly on `unified-agent-api`.
3. The new member runtime manager validates binding and `resolved_runtime` before startup.
4. Remote member startup emits `Start`, a session-bearing `Event`, and terminal `Exit` or
   `Error`.
5. `/v1/execute/cancel` can deliver to a live member-dispatch span.
6. Bootstrap failure and abnormal termination clean up honestly and do not leave a live claim
   behind.

#### Shell remote cutover lane

1. The member path in `async_repl.rs` no longer reuses local host orchestrator startup.
2. The shell uses an explicit local-vs-remote retained-control split.
3. The shell uses an explicit `PreparedMemberDispatch` or equivalent dedicated remote-prepared
   shape.
4. The shell persists `Allocating` before launch and only transitions to `Ready` and `Running`
   after remote ownership is retained.
5. Same-generation reuse remains intact and preflight failures still occur before any transport
   call.

#### Regression wall

1. Cancel, replacement, status, and trace correctness are proven after both runtime halves are
   integrated.
2. Replacement launch on world-generation rollover uses the same typed transport seam.
3. Replacement preserves fresh `participant_id` and correct `resumed_from_participant_id`.
4. Failed replacement leaves honest absence rather than stale liveness.
5. Trace remains auditable without becoming current liveness authority.

#### Workspace and scope boundary

1. No new public API family is introduced.
2. No scheduler, selector UX, shared-crate extraction, auth-bundle redesign, or non-Linux
   implementation is introduced.
3. Docs remain late and optional.
4. `.runs/plan-11/*` remains parent-owned only.
5. All work stays inside the `PLAN-11` architecture contract and the listed file boundaries.

## Merge Refusal Rules

The parent refuses to merge a worker output if any of these are true:

1. The patch edits a file outside the task’s allowed file list.
2. The patch reopens the refrozen request, builder, or readiness-event contract.
3. The patch authorizes host fallback for a world-scoped member.
4. The patch treats remote control state as if it were a local cancel handle.
5. The patch reintroduces a second runtime resolver or avoids the direct `unified-agent-api`
   dependency by falling back to backend-specific CLIs.
6. The patch weakens fail-closed behavior for binding mismatch, missing parent, cancel failure, or
   replacement failure.
7. The patch requires concurrent edits to parent-owned refreeze files or late regression files to
   become intelligible.
8. The patch omits test evidence for the behavior it claims to cover.
9. The patch broadens scope into shared-crate extraction, public APIs, scheduler work, early docs
   work, or non-Linux parity.

## Assumptions

1. The branch already contains the accepted carryover transport and harness work referenced in
   [PLAN-11.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-11.md),
   even though the old request freeze is superseded.
2. The existing readiness contract based on the session-bearing `Event` remains acceptable for
   this slice; no new stream frame family is required.
3. The resolved binary path chosen by the shell is valid input for Linux-world startup, and
   failure to use it in-world is a runtime error to surface and test, not a reason to add a
   second selector.
4. The current branch can accept a direct `unified-agent-api` dependency in `world-agent` without
   forcing a larger dependency architecture change.
5. Parent-owned `.runs/plan-11/*` artifacts remain the execution truth even if the repo has
   unrelated concurrent work elsewhere.
6. Operator surfaces should need at most additive parent-owned adjustments in
   [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs);
   a broader redesign would be out of scope and should block the run.

## Run Exit Criteria

### Successful run

The run is successful only if all of these are true:

1. `.runs/plan-11/sentinels/task-m11-a1-preflight-and-blocked-import.ok` exists.
2. `.runs/plan-11/sentinels/task-m11-a2-member-dispatch-contract-refreeze.ok` exists.
3. `.runs/plan-11/sentinels/task-m11-a3-builder-export-and-harness-refreeze.ok` exists.
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

1. `.runs/plan-11/blocked.json` exists with the triggering stop condition and current task ID.
2. No later-phase sentinel is written after the blocking condition is detected.
3. `run-state.json` records the blocked terminal state.
4. Partial worker outputs are either rejected or explicitly quarantined in `session.log`.
5. No fake completion signal is written to `.runs/plan-11/closeout.md`.
