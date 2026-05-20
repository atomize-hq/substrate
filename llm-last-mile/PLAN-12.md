# PLAN-12: Thaw The Member-Dispatch Request Surface And Finish Remote Placement Truth

Source plan: [PLAN-11_5.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-11_5.md)  
Source SOW: [11-in-world-member-dispatch-over-existing-host-world-transport.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/11-in-world-member-dispatch-over-existing-host-world-transport.md)  
Supersedes: the blocked continuation in `PLAN-11_5` for the remaining unshipped work only  
Blocked-run evidence:
- [.runs/plan-11_5/run-state.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11_5/run-state.json)
- [.runs/plan-11_5/blocked.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11_5/blocked.json)
- [.runs/task-m11_5-l2-shell-remote-member-cutover/blocked.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m11_5-l2-shell-remote-member-cutover/blocked.json)
- [.runs/task-m11_5-l2-shell-remote-member-cutover/worker-report.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m11_5-l2-shell-remote-member-cutover/worker-report.md)
- [.runs/task-m11_5-l1-world-agent-member-runtime/worker-report.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m11_5-l1-world-agent-member-runtime/worker-report.md)
- [.runs/task-m11_5-l1-world-agent-member-runtime/worker-output.patch](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m11_5-l1-world-agent-member-runtime/worker-output.patch)

Branch: `feat/session-centric-state-store`  
Plan type: Linux-first continuation plan, backend-only, status/trace truth required  
Review posture: `/autoplan` scope discipline with `/plan-eng-review` completeness, collapsed into one execution document  
Status: execution-ready replacement for `PLAN-11_5` on 2026-05-02  
Outside voice: unavailable on 2026-05-02 because `claude` CLI is installed but unauthenticated

## Objective

This is the continuation plan. Not another design reset.

`PLAN-11_5` already froze the payload contract, preserved the right worker artifacts, and proved
that the remaining blocker is not runtime design. The remaining blocker is that the shell lane
still cannot legally construct the already-frozen member-dispatch request through the sanctioned
`crate::execution::*` surface.

This plan does exactly three things:

1. thaw the crate-surface request bridge without reopening the payload contract,
2. finish the real remote member-runtime cutover in `world-agent` and `async_repl.rs`,
3. prove status and trace truth so world-scoped members stop lying about where they run.

The user-visible outcome is unchanged:

- when a world-scoped member says it is live on generation `N` of world `W`,
- it is actually running inside `world-agent` on generation `N` of world `W`,
- with real remote cancel delivery, real replacement behavior, and real retained control,
- and `substrate agent status` plus trace rows do not lie about placement.

## Locked Starting State

### Accepted carryover

The accepted parent state entering `PLAN-12` is:

- Gate A carryover: accepted
- Gate B carryover: accepted
- Gate C: blocked
- Gate D: not reached

The following branch truth already exists and remains authoritative unless a later proof fails:

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)
- [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
- [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs), which already re-exports `build_agent_client_and_member_dispatch_request(...)`

The preserved worker artifacts are evidence and logic references only. They are not accepted
branch truth until the parent reopens the correct lanes and integrates them.

### Exact blocker

The blocked state is concrete:

1. [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs) re-exports
   `build_agent_client_and_member_dispatch_request(...)` at lines 40-43.
2. [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)
   re-exports the same builder at lines 15-19.
3. [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
   still keeps `MemberDispatchTransportRequest` crate-private at lines 203-216.
4. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
   therefore cannot consume the builder through `crate::execution::*` without either reaching
   into private modules or recreating the request shape locally.

That is the block. Not runtime uncertainty. Not transport uncertainty. Not test ambiguity.

## Frozen Execution Contract

This section is the part implementers are not allowed to reinterpret.

### Non-negotiable invariants

1. The shell remains the only canonical writer of orchestration session state and participant state.
2. `world-agent` owns in-world member execution, remote cancel delivery, event streaming, and completion observation only.
3. World-scoped member launch fails closed. Host fallback is forbidden.
4. `POST /v1/execute/stream` and `POST /v1/execute/cancel` remain the only transport seam.
5. `ExecuteStreamFrame::{Start,Event,Exit,Error}` remain the only stream families.
6. Remote readiness still depends on the existing session-handle `AgentEvent` contract.
7. The shell must represent retained control explicitly as local vs remote.
8. The shell must use an explicit remote prepared-launch shape for members. It must not reuse the host-local orchestrator prepared-launch path as a shortcut.
9. `world-agent` must validate the shell-supplied `world_id` and `world_generation` and reject mismatches.
10. Linux-first remains explicit. Non-Linux member-dispatch paths fail closed in this slice.

### Frozen payload, thawed crate-surface bridge

The payload contract stays frozen, but the parent is now explicitly authorized to thaw the
crate-surface request bridge before reopening lanes. The plan chooses a preferred fix, direct
re-export of `MemberDispatchTransportRequest` through the allowed crate surface, and
pre-authorizes one fallback, a sanctioned adapter helper, so the run cannot block again on the
same boundary mistake.

That means:

- `MemberDispatchRequestV1` does not change.
- `resolved_runtime` does not change.
- `resolved_runtime.binary_path` stays absolute.
- `resolved_runtime.backend_kind` stays explicit. No inference from `backend_id`.
- top-level `agent_id` remains authoritative for traces, budgets, and diagnostics.
- `protocol` remains part of the transport identity contract.

#### Preferred bridge, default

Re-export `MemberDispatchTransportRequest` through the same crate-level surface that already
exports `build_agent_client_and_member_dispatch_request(...)`.

Required shape:

1. re-export the type from `dispatch/prelude.rs`,
2. re-export the type from `routing.rs`,
3. leave request serialization behavior in `world_ops.rs` unchanged.

#### Authorized fallback bridge

If direct type export proves to be the wrong seam after code review, the parent is pre-authorized
to add one crate-local adapter helper in `world_ops.rs`, then re-export that helper through
`dispatch/prelude.rs` and `routing.rs`.

Hard rules:

- the fallback may change visibility and helper shape only,
- it may not change serialized payload fields,
- it may not change `MemberDispatchRequestV1`,
- it may not add a second request-construction path inside `async_repl.rs`,
- it may not move runtime selection into `world-agent`.

### File authority and escalation boundary

| Boundary | Files | Rule |
| --- | --- | --- |
| Frozen payload contract | `crates/agent-api-types/src/lib.rs` | No request-shape changes. |
| Parent-owned bridge seam | `crates/shell/src/execution/routing.rs`, `crates/shell/src/execution/routing/dispatch/prelude.rs` | Parent may export the type or adapter. |
| Parent-owned surface-only fallback | `crates/shell/src/execution/routing/dispatch/world_ops.rs` | Parent may add visibility/helper glue only if the preferred bridge is rejected after code review. No payload-schema edits. |
| Worker-safe world-agent lane | `crates/world-agent/Cargo.toml`, `crates/world-agent/src/lib.rs`, `crates/world-agent/src/service.rs`, `crates/world-agent/src/member_runtime.rs`, `crates/world-agent/tests/streamed_execute_cancel_v1.rs` | Lane B only. |
| Worker-safe shell lane | `crates/shell/src/repl/async_repl.rs`, `crates/shell/tests/repl_world_first_routing_v1.rs` | Lane C only. |
| Parent-owned regression wall | `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`, `crates/shell/tests/agent_hub_trace_persistence.rs` | Parent only after B and C integrate. |
| Escalation-only surfaces | `crates/shell/src/execution/agent_runtime/session.rs`, `crates/shell/src/execution/agent_runtime/state_store.rs`, `crates/shell/src/execution/agents_cmd.rs` | Touch only if the regression wall proves current assumptions false. |

### Stop and escalate conditions

Stop the run and write blocked state again only if one of these becomes true:

1. the parent bridge step requires changing `crates/agent-api-types/src/lib.rs`,
2. the parent bridge step requires changing the serialized `member_dispatch` payload,
3. the shell lane still cannot construct the request through `crate::execution::*` after the parent bridge lands,
4. either worker lane needs to edit the other lane's files,
5. the world-agent lane needs a second runtime selector or backend inference from `backend_id`,
6. status or trace truth requires unplanned production logic outside the escalation-only surfaces,
7. a third independent worker lane becomes necessary.

## Step 0: Scope Challenge

### 0A. What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| typed `member_dispatch` payload with resolved runtime | [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs), [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs) | Reuse exactly. Do not reopen payload shape. |
| builder function already visible at crate surface | [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs) | Keep. The missing piece is request construction, not function visibility. |
| request-construction type lives one layer too low | [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs) | Bridge it. Do not duplicate it. |
| shell-owned member lifecycle semantics | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse persistence, invalidation, lineage, and status semantics. Change placement only. |
| world-owned retained-control lifecycle pattern | [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs) | Reuse as a lifecycle pattern, not as a second owner model. |
| status and trace truth consumers | [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs), [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs) | Reuse. The regression wall proves the producer is now actually remote. |

### 0B. Minimum diff decision

The smallest honest diff is:

1. thaw the request-construction bridge only,
2. keep the payload contract frozen,
3. reopen the same two runtime lanes,
4. integrate the preserved `world-agent` lane work only after the shell lane is unblocked,
5. finish the status and trace regression wall.

Anything smaller is fake progress. Anything larger is unnecessary scope.

### 0C. Complexity, completeness, and search check

This continuation still touches more than 8 files. That is justified and still minimal.

The production path is:

1. `crates/shell/src/execution/routing.rs`
2. `crates/shell/src/execution/routing/dispatch/prelude.rs`
3. optional surface-only bridge helper in `crates/shell/src/execution/routing/dispatch/world_ops.rs`
4. `crates/world-agent/Cargo.toml`
5. `crates/world-agent/src/lib.rs`
6. `crates/world-agent/src/service.rs`
7. `crates/world-agent/src/member_runtime.rs`
8. `crates/shell/src/repl/async_repl.rs`
9. `crates/world-agent/tests/streamed_execute_cancel_v1.rs`
10. `crates/shell/tests/repl_world_first_routing_v1.rs`
11. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
12. `crates/shell/tests/agent_hub_trace_persistence.rs`

Search-before-building result:

- **[Layer 1]** Keep using the existing `crate::execution::*` export pattern instead of adding another import seam.
- **[Layer 1]** Keep using `/v1/execute/stream` and `/v1/execute/cancel` only.
- **[Layer 1]** Keep using the existing session-handle readiness event as the live gate.
- **[Layer 3]** Treat the bridge as crate-surface plumbing, not as contract refreeze. The payload already carries the authoritative runtime facts.

Completeness decision:

- private-module reach-in from `async_repl.rs`: rejected
- local reconstruction of `MemberDispatchTransportRequest`: rejected
- second ad hoc builder near the REPL: rejected

Those shortcuts save minutes and permanently preserve drift in one of the most fragile seams in
the slice.

### 0D. Distribution check

No new distributable artifact type is introduced here.

This slice changes runtime truth inside existing crates only. No new binary, package, container,
or publish pipeline is required.

### 0E. NOT in scope

- another `MemberDispatchRequestV1` shape change
- a `V2` rename for an unshipped internal seam
- a new public `/v1/member/*` or `/v1/agents/*` API family
- a new shared startup crate
- shell state-store redesign
- macOS or Windows member-dispatch parity
- status or doctor UX redesign
- docs cleanup before the regression wall is green

## Architecture Review

### Findings

`[P1] (confidence: 10/10) crates/shell/src/execution/routing.rs:40-43 + crates/shell/src/execution/routing/dispatch/prelude.rs:15-19 + crates/shell/src/execution/routing/dispatch/world_ops.rs:203-216 — the crate-level surface exports the member-dispatch builder but not the request-construction surface needed to call it, so the shell cutover is blocked by module boundaries, not by runtime design.`

Recommendation:

- prefer direct type re-export through `prelude.rs` and `routing.rs`
- if that feels wrong after code review, add one sanctioned adapter in `world_ops.rs`
- do not reopen the payload contract

`[P1] (confidence: 10/10) crates/shell/src/repl/async_repl.rs:2624-2819 — member startup still flows through preparation helpers that end at host-local runtime launch and shutdown helpers, so world-scoped members can still claim remote placement while control remains rooted in the shell process.`

Recommendation:

- split host-orchestrator startup from remote member startup explicitly
- keep local startup for the orchestrator path only
- move member startup, cancel retention, and completion observation onto the remote path

`[P1] (confidence: 9/10) crates/world-agent/src/service.rs:1415-1438 — execute_cancel only knows the ordinary process-exec span registry today, so remote member spans still need a world-owned retained-control registry and cancel path.`

Recommendation:

- add Linux-only `member_runtime.rs`
- register member spans for cancel delivery
- route remote member shutdown through the same truthy terminal path used for process exec

### Ownership split

| Concern | Owner | Why |
| --- | --- | --- |
| runtime selection, backend kind, binary path | shell | The shell already resolved them. Re-resolving inside `world-agent` would create drift. |
| request payload construction | shell routing layer | The transport contract already lives there. |
| in-world `run_control(...)`, cancel delivery, completion observation | `world-agent` | That is the placement move this slice exists to land. |
| participant persistence, status truth, stale-generation invalidation | shell | Canonical state stays in the shell. |
| replacement decisions and lineage | shell | Already landed. Must remain authoritative. |
| remote event forwarding and terminal observation | `world-agent` | Required to make placement observable rather than inferred. |

### Architecture ASCII diagram

```text
CURRENT BLOCKED STATE
=====================
crate::execution::*
    │
    ├── build_agent_client_and_member_dispatch_request(...)
    │
    └── cannot expose the request-construction surface needed by async_repl.rs
            │
            ▼
async_repl.rs
    ├── can prepare member runtime state
    ├── cannot legally construct the transport request
    └── still delegates member startup to host-local runtime launch

TARGET PLAN-12 STATE
====================
crate::execution::*
    ├── build_agent_client_and_member_dispatch_request(...)
    └── MemberDispatchTransportRequest OR one sanctioned adapter
            │
            ▼
async_repl.rs
    ├── prepare remote member dispatch request
    ├── launch over /v1/execute/stream
    ├── retain remote cancel + event + completion ownership
    └── persist live-state transitions only after remote evidence
            │
            ▼
world-agent member_runtime.rs
    ├── validate authoritative world binding
    ├── build gateway from resolved_runtime
    ├── run_control(...) inside the world
    ├── register span_id for execute-cancel
    ├── forward Start / Event / Exit / Error
    └── close terminally and honestly on cancel or failure
```

### Liveness boundary

```text
Allocating
    │
    ├── request not constructible at crate surface
    │       └── stop before launch, non-live
    │
    ├── remote stream established, no session-handle event yet
    │       └── remain Allocating
    │
    └── remote stream + retained control + session-handle event
            └── Ready / Running may be advertised

Ready / Running
    │
    ├── execute-cancel delivered and remote terminal observed
    │       └── Stopped / terminal non-live
    │
    └── replacement generation selected
            └── stale generation invalidated, successor restarts the proof chain
```

## Code Quality Review

### Findings

`[P2] (confidence: 9/10) crates/shell/src/execution/routing/dispatch/world_ops.rs:203-265 — MemberDispatchTransportRequest already centralizes the transport shape, so rebuilding those fields in async_repl.rs would create the exact duplication the carryover refreeze was supposed to remove.`

Recommendation:

- keep request assembly single-sourced near the builder surface
- let `async_repl.rs` consume the sanctioned surface only
- do not create a second local request-mapping path

`[P2] (confidence: 8/10) .runs/task-m11_5-l1-world-agent-member-runtime/worker-output.patch — the preserved world-agent worker patch is useful, but blindly applying it before the shell lane is unblocked risks integrating code against a still-wrong request-construction boundary.`

Recommendation:

- mine logic, not hunks
- reseed both lanes from the post-bridge parent commit
- integrate only after the shell lane proves the crate-surface bridge is sufficient

### Allowed code shape

1. No new shared crate.
2. No second transport client.
3. No duplicate member-dispatch payload construction in `async_repl.rs`.
4. No new status reconstruction path.
5. No silent reopening of the payload schema.
6. No worker edits to the bridge files after Step 1 lands.

If implementation needs any of those, stop and escalate. That means the plan assumptions were
wrong.

## Test Review

### Test framework detection

- Runtime: Rust
- Framework: `cargo test`
- Packages: `shell`, `world-agent`
- No LLM eval suite is required for this slice

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crate-surface bridge
    │
    ├── [GAP]         direct type export works through crate::execution::*
    ├── [GAP]         fallback adapter works through crate::execution::* if chosen
    └── [GAP]         neighboring exports remain unchanged

[+] crates/world-agent/src/member_runtime.rs + service.rs
    │
    ├── [GAP]         success path launches from resolved_runtime inside the world
    ├── [GAP]         authoritative world mismatch fails before startup
    ├── [GAP]         unsupported backend or missing binary fails closed
    ├── [GAP]         execute-cancel reaches member spans and converges terminally
    └── [GAP]         pre-ready failure and post-ready terminal close produce distinct honest states

[+] crates/shell/src/repl/async_repl.rs
    │
    ├── [GAP] [->E2E] first world-backed command uses the sanctioned crate-surface bridge
    ├── [GAP]         Allocating persists before remote ownership is retained
    ├── [GAP]         Ready/Running require session-handle evidence, not any Event
    ├── [GAP]         same-generation reuse does not relaunch
    ├── [GAP]         shutdown uses execute-cancel for remote member spans
    └── [GAP] [->E2E] replacement launch preserves fresh participant_id + resumed_from lineage

[+] shell status and trace wall
    │
    ├── [GAP]         status reports the real remote producer
    ├── [GAP]         trace rows remain producer-correct after cancel and replacement
    └── [GAP]         failed replacement never revives stale generation liveness

─────────────────────────────────
COVERAGE: 0/17 remaining PLAN-12 paths proven
GAPS: 17 paths require coverage before closeout
─────────────────────────────────
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] First world-backed REPL command
    ├── [GAP] [->E2E] shell selects the member and launches it remotely through the sanctioned bridge
    ├── [GAP]         missing parent or world binding fails before transport
    └── [GAP]         same-generation reuse does not relaunch

[+] Live member cancel and shutdown
    ├── [GAP] [->E2E] shutdown issues execute-cancel against the remote span
    ├── [GAP]         cancel delivery failure leaves non-live terminal state
    └── [GAP]         clean cancel becomes Stopped, not Invalidated

[+] Shared-world rollover with live member
    ├── [GAP] [->E2E] replacement launch crosses the same transport seam with lineage preserved
    ├── [GAP]         replacement bootstrap failure leaves no authoritative-live member
    └── [GAP]         old generation never regains liveness

[+] Operator inspection
    ├── [GAP]         substrate agent status --json reflects the remote producer
    └── [GAP]         trace rows remain participant-correct after replacement and cancel
```

### Required tests to add

1. `crates/world-agent/tests/streamed_execute_cancel_v1.rs`
   - add member-dispatch success, authoritative-world mismatch, missing binary,
     unsupported backend, cancel, and abnormal terminal cases
   - assert cancel delivery toggles `delivered=true` only after member span registration
2. `crates/shell/tests/repl_world_first_routing_v1.rs`
   - add first launch, same-generation reuse, remote readiness, replacement launch, and
     replacement failure cases
   - assert `Ready` and `Running` cannot appear before session-handle evidence
3. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
   - add status truth checks for a real remotely launched member
   - assert stale trace fallback does not beat live remote runtime state
4. `crates/shell/tests/agent_hub_trace_persistence.rs`
   - add trace truth checks for remote member launch, cancel, and replacement
   - assert lineage survives replacement while terminal predecessors stay auditable

### Test artifact

The eng-review QA artifact for this continuation is:

- [spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260502-195956.md](/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260502-195956.md)

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
| --- | --- | --- | --- | --- | --- |
| crate-surface bridge | builder is exported but request still cannot be constructed from `crate::execution::*` | no | no | no | yes |
| fallback adapter | helper leaks a second request-construction contract and drifts from payload truth | no | no | no | yes |
| remote launch split | shell persists a world-scoped member as live while startup still runs locally | no | no | no | yes |
| authoritative world binding | world-agent launches against stale or mismatched `world_id` or `world_generation` | no | not yet | partial | yes |
| resolved runtime path | shell provides a binary path missing or unusable inside the Linux world | no | partial | partial | yes |
| remote readiness gate | readiness advances before the session-handle event arrives | no | partial | no | yes |
| cancel path | execute-cancel misses the remote member span and status keeps advertising live | no | partial | partial | yes |
| replacement wall | replacement fails and stale generation still appears live in status or trace | no | partial | no | yes |

Critical gap rule:

If any path can advertise a live world member without a real remote session handle, a real
remote cancel path, and a real remote terminal observer, the slice is not done.

## Performance Review

This is correctness-first.

Performance cautions:

1. keep using the existing `AgentClient` transport stack
2. keep remote stream handling incremental, not buffered
3. preserve same-generation reuse so steady state does not relaunch on every world-backed command

There is no new throughput blocker beyond that. The complexity is being spent on truth, not on
speculative optimization.

## DX Guardrails

This is a developer tool. Failure messages matter.

Required error-message posture:

1. bridge failures must name the missing crate surface explicitly
2. remote launch failures must include `participant_id`, `world_id`, `world_generation`, and backend kind
3. cancel failures must say whether delivery failed before span registration or after remote startup
4. replacement failures must say whether stale generation was already invalidated and whether the successor ever reached remote readiness

## Worktree Parallelization Strategy

This slice has one real parallel window and no more. The bridge must land first. The regression
wall must land last.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| P0. Parent crate-surface bridge | `crates/shell/src/execution/` | — |
| L1. World-agent member runtime lane | `crates/world-agent/` | P0 |
| L2. Shell remote-member lane | `crates/shell/src/repl/`, `crates/shell/tests/` | P0 |
| P1. Parent regression wall | `crates/shell/tests/` | L1, L2 |

### Parallel lanes

- Lane A: `P0`, sequential, parent-owned
- Lane B: `L1`, independent after `P0`
- Lane C: `L2`, independent after `P0`
- Lane D: `P1`, sequential after B and C integrate

### Suggested worktree ownership

- Parent worktree: bridge + integration + regression wall
- `wt/plan12-world-agent`: Lane B only
- `wt/plan12-shell`: Lane C only

No worker gets bridge-file ownership after Step 1. That is what keeps the merge cheap.

### Execution order

1. Parent lands the crate-surface bridge.
2. Parent reruns the shell library gate.
3. Reseed both worker lanes from that exact parent commit.
4. Reopen the `world-agent` lane and the shell lane in parallel.
5. Parent integrates accepted outputs.
6. Parent runs the regression wall.

### Conflict flags

- Lane B and Lane C stay parallel only if neither reopens the bridge files after `P0`
- if either lane needs `crates/agent-api-types/src/lib.rs`, stop
- if the shell lane needs `session.rs`, `state_store.rs`, or `agents_cmd.rs`, escalate before editing
- if the world-agent lane starts inferring runtime from `backend_id`, stop immediately

### Parallelization verdict

One real parallel window remains. Worker cap stays exactly `2`.

## Implementation Sequence

This is the execution contract. Steps are ordered. Do not skip a gate because later work seems
obvious.

### Step 1. Parent crate-surface bridge

Files:

- [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs)
- [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)
- optional surface-only fallback in [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)

Work:

1. choose the preferred type export or the sanctioned adapter
2. keep serialized payload behavior unchanged
3. prove the shell lane can construct the request through `crate::execution::*`
4. record the bridge choice in run-state and closeout notes

Validation gate:

- `async_repl.rs` no longer needs private-module reach-in
- no second payload-construction path exists
- `cargo test -p shell --lib -- --nocapture` passes before workers reopen

Escalate if:

- Step 1 needs payload-schema edits
- neighboring exports regress
- the sanctioned surface still does not unblock the shell lane

### Step 2. World-agent lane

Files:

- [crates/world-agent/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/Cargo.toml)
- [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- new `crates/world-agent/src/member_runtime.rs`
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)

Work:

1. add direct `agent_api` dependency
2. add Linux-only `member_runtime.rs`
3. route `member_dispatch` through the member runtime manager
4. register remote member spans for cancel delivery
5. fail closed on binding mismatch, missing binary, or unsupported backend

Validation gate:

- world-agent launches from the shell-resolved runtime descriptor
- member dispatch fails closed on world mismatch or missing runtime facts
- remote cancel reports truthfully against the member span registry
- `cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture` passes

Use the preserved worker patch as logic reference only. Do not blindly apply it.

### Step 3. Shell remote-member lane

Files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

Work:

1. split host-local orchestrator launch from remote member launch explicitly
2. consume only the sanctioned crate-surface bridge
3. wire startup, readiness, cancel, and replacement through the remote path
4. preserve same-generation reuse for already-live remote members

Validation gate:

- first world-backed member launch crosses typed execute-stream
- `Ready` and `Running` require session-handle evidence
- replacement preserves lineage and fails closed honestly
- `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture` passes

Escalate if:

- the shell lane needs to edit bridge files after Step 1
- remote readiness cannot be proven without reopening the readiness contract
- status truth starts depending on local optimistic state again

### Step 4. Parent regression wall

Files:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)

Work:

1. add status truth assertions for real remote producer state
2. add trace truth assertions for launch, cancel, and replacement
3. prove stale generation never revives after failed replacement

Validation gate:

- `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture` passes
- `cargo test -p shell --test agent_hub_trace_persistence -- --nocapture` passes
- remote producer truth is observable through status and trace, not inferred from optimistic state

### Step 5. Closeout

Work:

1. update run-state with accepted worker outputs and final validation
2. leave quarantined but unused worker artifacts as evidence only
3. record whether the bridge choice was direct type export or adapter fallback
4. capture the final ordered verification transcript

Closeout is not done until the bridge choice, lane outputs, and proof commands are all recorded.

## Definition of Done

1. the crate surface exposes one sanctioned request-construction bridge for member dispatch
2. `async_repl.rs` launches world-scoped members through the remote path only
3. `world-agent` owns member retained control inside the active world
4. `execute_cancel` reaches both process-exec spans and member spans
5. status and trace show the real remote producer
6. same-generation reuse still avoids redundant relaunch
7. replacement preserves lineage and fails closed honestly
8. all targeted tests pass in order
9. no worker lane reopened the payload contract
10. the run cannot block again on missing plan authority for the crate-surface bridge

## Recommended Verification Commands

Run in this order. Do not skip forward.

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
cargo test -p shell --test repl_world_first_routing_v1 --no-run
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test agent_hub_trace_persistence -- --nocapture
```

## Deferred Work

- shared startup-crate extraction, if a third consumer ever makes the duplication real
- non-Linux member-dispatch parity
- docs cleanup after the regression wall is green
- status and doctor UX polish after placement truth is proven

No new `TODOS.md` entry is required here. These are explicit non-goals, not forgotten work.

## Completion Summary

- Step 0: scope accepted as a continuation, not a restart
- Architecture Review: 3 issues found and resolved in-plan
- Code Quality Review: 2 issues found and resolved in-plan
- Test Review: diagram produced, 17 remaining continuation gaps identified
- Performance Review: 3 cautions, 0 throughput blockers
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 durable TODOs proposed
- Failure modes: 8 critical gaps called out for implementation
- Outside voice: unavailable because `claude` CLI auth is missing
- Parallelization: 4 execution phases, 1 real parallel window, worker cap stays `2`
- Lake Score: complete option chosen over private-module reach-in or another contract refreeze

<!-- AUTONOMOUS DECISION LOG -->
## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Scope | Continue from `PLAN-11_5` blocked truth instead of restarting `PLAN-11` | Mechanical | Minimal diff | The payload contract and harness work are already landed | Restarting transport design from zero |
| 2 | Authority | Thaw the crate-surface bridge while keeping the payload contract frozen | Mechanical | Explicit over clever | The block is plan authority, not request schema | Treating bridge visibility as contract churn |
| 3 | Bridge shape | Prefer direct type re-export, authorize one adapter fallback | Taste | Engineered enough | Direct export is smaller, but one sanctioned adapter is allowed if the code shows a better seam | Private-module reach-in or second local builder |
| 4 | Boundaries | Expand parent-owned bridge files to `routing.rs` + `prelude.rs` and optionally surface-only `world_ops.rs` | Mechanical | Blast radius instinct | That is the smallest authority expansion that removes the false blocker | Freezing the bridge seam again |
| 5 | Runtime ownership | Keep runtime selection in the shell and retained control in `world-agent` | Mechanical | Systems over heroes | Two selectors would drift, two state writers would lie | Letting `world-agent` infer runtime from `backend_id` |
| 6 | Shell cutover | Split local orchestrator control from remote member control explicitly | Mechanical | Explicit over clever | The current coupling is the remaining placement lie | Reusing host-local runtime launch for members |
| 7 | Parallelization | Keep the worker cap at `2` with parent-owned bridge first and regression wall last | Mechanical | Minimal diff | That is the only merge-cheap parallel window in this slice | Opening a third lane or parallelizing the bridge |
| 8 | Regression wall | Keep status and trace proof parent-owned after both runtime lanes merge | Mechanical | Blast radius instinct | Producer truth crosses lane boundaries and needs one integrator | Letting a worker lane redefine status truth alone |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
| --- | --- | --- | --- | --- | --- |
| CEO Review | `/plan-ceo-review` | Scope and strategy | 1 | CLEAR | Kept the continuation narrow: fix plan authority at the crate surface, then finish the original runtime cutover |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | SKIPPED | No separate outside-model review run, and `claude` CLI auth is missing for outside voice |
| Eng Review | `/plan-eng-review` | Architecture and tests (required) | 1 | CLEAR | Locked the real blocker to the request-construction bridge, froze the payload contract, and defined the regression wall for remote launch, cancel, replacement, status, and trace truth |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | SKIPPED | No UI scope |

**UNRESOLVED:** 0 plan-level decisions remain. The remaining work is implementation plus
verification only.

**VERDICT:** ENG CLEARED. `PLAN-12` replaces `PLAN-11_5` as the honest continuation plan:
thaw the request bridge, reopen exactly two runtime lanes, and finish the placement-truth
regression wall without reopening the payload contract.
