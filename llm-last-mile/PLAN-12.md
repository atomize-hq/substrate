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

`PLAN-11_5` got close and still blocked on the wrong seam.

The runtime contract is no longer the problem. The accepted carryover already froze the
member-dispatch payload, the shell-side builder, and the test harnesses. The remaining
problem is that the shell lane still cannot legally *construct* the already-frozen request
through `crate::execution::*`.

This plan finishes the slice by doing three things:

1. explicitly thawing the crate-surface request-construction bridge that `PLAN-11_5`
   froze too aggressively,
2. authorizing the parent to choose the correct bridge shape without re-opening the
   runtime contract again,
3. then resuming the exact same world-agent, shell, and regression-wall work needed to
   satisfy the original SOW.

The required user outcome is unchanged:

- when a world-scoped member says it is live on generation `N` of world `W`,
- it is actually running inside `world-agent` on generation `N` of world `W`,
- with real remote cancel delivery, real replacement behavior, and real retained control,
- and `substrate agent status` plus trace rows do not lie about placement.

## Why `PLAN-11_5` Blocked Again

The blocked state is now concrete, not speculative.

1. `crates/shell/src/execution/routing.rs` already re-exports
   `build_agent_client_and_member_dispatch_request(...)`.
2. The builder input type,
   `MemberDispatchTransportRequest`, still lives only in
   [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs).
3. `async_repl.rs` therefore cannot consume the builder from `crate::execution::*`
   without either:
   - reaching into private routing modules, or
   - forcing a parent-owned export change outside the old allowed blast radius.
4. The old plan froze exactly the files that contain the missing bridge:
   - [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)
   - optionally [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)

That is the third block. Not runtime uncertainty. Not transport uncertainty. Plan authority
uncertainty.

## Imported Truth

The current accepted parent state is:

- Gate A carryover: accepted
- Gate B carryover: accepted
- Gate C: blocked
- Gate D: not reached

Accepted branch truth that remains valid:

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)
- [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)

Already-landed parent unblock that stays valid:

- [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs)
  now re-exports the builder function itself

Preserved but unintegrated evidence:

- the `world-agent` worker patch is preserved as reference implementation only
- the shell worker returned blocked with no accepted code changes
- no worker output beyond the parent-owned `routing.rs` change is branch truth today

## Explicit Authority Reset

This is the whole point of `PLAN-12`.

`PLAN-12` explicitly grants the parent authority to finish the crate-surface request bridge
before reopening worker lanes. No additional approval is required during execution if the work
stays within the bounds below.

### Parent-owned bridge decision

The parent must choose exactly one of these bridge shapes at Step 1:

#### Preferred bridge, default

Re-export `MemberDispatchTransportRequest` through the same crate-level surface that already
exports `build_agent_client_and_member_dispatch_request(...)`.

That means:

1. re-export the type from `dispatch/prelude.rs`,
2. re-export the type from `routing.rs`,
3. keep `world_ops.rs` behavior unchanged.

#### Authorized fallback bridge

If direct type export proves to be the wrong seam after reading the code carefully, the parent
is pre-authorized to add a crate-local adapter helper in `world_ops.rs`, then re-export that
adapter through `dispatch/prelude.rs` and `routing.rs`.

Hard rule:

- this fallback may change visibility and helper shape only,
- it may not change serialized payload fields,
- it may not change `MemberDispatchRequestV1`,
- it may not add a second request-construction path inside `async_repl.rs`.

### Why this is authorized

The old plan treated the request-construction surface as frozen contract. That was wrong.

The payload contract is frozen. The crate-surface bridge is not. `PLAN-12` separates those two
facts explicitly so the run cannot block again on a plan-boundary technicality.

### Revised file authority

| Boundary | Files | Rule |
| --- | --- | --- |
| Frozen payload contract | `crates/agent-api-types/src/lib.rs` | No request-shape changes. |
| Parent-owned bridge seam | `crates/shell/src/execution/routing.rs`, `crates/shell/src/execution/routing/dispatch/prelude.rs` | Parent may export the type or adapter. |
| Parent-owned surface-only fallback | `crates/shell/src/execution/routing/dispatch/world_ops.rs` | Parent may add visibility/helper glue only if the preferred bridge is rejected after code review. No payload-schema edits. |
| Worker-safe world-agent lane | `crates/world-agent/Cargo.toml`, `crates/world-agent/src/lib.rs`, `crates/world-agent/src/service.rs`, `crates/world-agent/src/member_runtime.rs`, `crates/world-agent/tests/streamed_execute_cancel_v1.rs` | Lane B only. |
| Worker-safe shell lane | `crates/shell/src/repl/async_repl.rs`, `crates/shell/tests/repl_world_first_routing_v1.rs` | Lane C only. |
| Parent-owned regression wall | `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`, `crates/shell/tests/agent_hub_trace_persistence.rs` | Parent only after B and C integrate. |
| Escalation-only surfaces | `crates/shell/src/execution/agent_runtime/session.rs`, `crates/shell/src/execution/agent_runtime/state_store.rs`, `crates/shell/src/execution/agents_cmd.rs` | Touch only if the regression wall proves the current assumptions false. |

### Revised stop conditions

Stop the run and write blocked state again only if one of these becomes true:

1. the parent bridge step requires changing `crates/agent-api-types/src/lib.rs`
2. the parent bridge step requires changing the serialized `member_dispatch` payload
3. the shell lane still cannot construct the request through `crate::execution::*` after the
   parent bridge lands
4. either worker lane needs to edit the other lane's files
5. the world-agent lane needs a second runtime selector or backend inference from `backend_id`
6. status or trace truth requires unplanned production logic outside the escalation-only surfaces
7. a third independent worker lane becomes necessary

## Step 0: Scope Challenge

### 0A. What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| typed `member_dispatch` payload with resolved runtime | [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs), [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs) | Reuse exactly. Do not refreeze payload shape again. |
| builder function already visible at crate surface | [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs) | Keep. The missing piece is request construction, not function visibility. |
| request-construction type lives one layer too low | [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs) | Bridge it. Do not duplicate it. |
| shell-owned member lifecycle semantics | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse persistence, invalidation, and lineage semantics. Change placement only. |
| world-owned retained-control reference pattern | [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs) | Reuse as lifecycle pattern, not as a second owner model. |
| status and trace truth consumers | [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs), [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs) | Reuse. The regression wall proves the producer is now actually remote. |

### 0B. Minimum diff decision

The smallest honest diff is:

1. thaw the request-construction bridge only,
2. keep the payload contract frozen,
3. reseed the same two runtime lanes,
4. integrate the preserved `world-agent` lane work only after the shell lane is unblocked,
5. finish the status and trace wall.

Anything smaller is fake progress. Anything larger is unnecessary scope.

### 0C. Complexity check

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

That is still one slice. Not pretty. Necessary.

### 0D. Search-before-building result

- **[Layer 1]** Keep using the existing `crate::execution::*` export pattern instead of adding
  another import seam.
- **[Layer 1]** Keep using `/v1/execute/stream` and `/v1/execute/cancel` only.
- **[Layer 1]** Keep using the existing session-handle readiness event as the live gate.
- **[Layer 3]** Treat the bridge as crate-surface plumbing, not as contract refreeze. The
  payload already carries the authoritative runtime facts.

### 0E. Completeness check

The shortcut would be:

- private-module reach-in from `async_repl.rs`,
- local reconstruction of `MemberDispatchTransportRequest`,
- or a second ad hoc builder shape near the REPL.

Reject all of them.

That would save minutes and permanently preserve drift in one of the most fragile seams in the
slice.

### 0F. Distribution check

No new distributable artifact type is introduced here.

This slice changes runtime truth inside existing crates only. No new binary, package, container,
or publish pipeline is required.

### 0G. NOT in scope

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

`[P1] (confidence: 10/10) crates/shell/src/execution/routing.rs:40-41 + crates/shell/src/execution/routing/dispatch/prelude.rs:16 + crates/shell/src/execution/routing/dispatch/world_ops.rs:203,1074 — the crate-level surface exports the member-dispatch builder but not the request-construction surface needed to call it, so the shell cutover is blocked by module boundaries, not by runtime design.`

Recommendation:

- prefer direct type re-export through `prelude.rs` and `routing.rs`
- if that feels wrong after code review, add one sanctioned adapter in `world_ops.rs`
- do not reopen the payload contract

`[P1] (confidence: 10/10) crates/shell/src/repl/async_repl.rs:2624,2706,2763 — member startup still flows through prepare/start helpers that end at host-local orchestrator startup, so world-scoped members can still claim remote placement while running in the shell process.`

Recommendation:

- split host-orchestrator startup from remote member startup explicitly
- keep local startup for the orchestrator path only
- move member startup, cancel retention, and completion observation onto the remote path

`[P1] (confidence: 9/10) crates/world-agent/src/service.rs:1415 — execute_cancel only knows the ordinary process-exec span registry today, so remote member spans still need a world-owned retained-control registry and cancel path.`

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

If the implementation needs any of those, stop and escalate. That means the plan assumptions
were wrong.

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

This is still correctness-first.

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
2. remote launch failures must include `participant_id`, `world_id`, `world_generation`, and
   backend kind
3. cancel failures must say whether delivery failed before span registration or after remote
   startup
4. replacement failures must say whether stale generation was already invalidated and whether
   the successor ever reached remote readiness

## Worktree Parallelization Strategy

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

### Parallelization verdict

One real parallel window remains. Worker cap stays exactly `2`.

## Deferred Work

- shared startup-crate extraction, if a third consumer ever makes the duplication real
- non-Linux member-dispatch parity
- docs cleanup after the regression wall is green
- status and doctor UX polish after placement truth is proven

No new `TODOS.md` entry is required here. These are explicit non-goals, not forgotten work.

## Implementation Sequence

### Step 1. Parent crate-surface bridge

Files:

- [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs)
- [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)
- optional surface-only fallback in [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)

Deliverables:

1. choose the preferred type export or the sanctioned adapter
2. keep serialized payload behavior unchanged
3. prove the shell lane can construct the request through `crate::execution::*`
4. record the bridge choice in run-state and closeout notes

Acceptance:

- `async_repl.rs` no longer needs private-module reach-in
- no second payload-construction path exists
- `cargo test -p shell --lib -- --nocapture` passes before workers reopen

Stop condition:

- if Step 1 needs payload-schema edits, stop and reassess

### Step 2. World-agent lane

Files:

- [crates/world-agent/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/Cargo.toml)
- [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- new `crates/world-agent/src/member_runtime.rs`
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)

Deliverables:

1. add direct `agent_api` dependency
2. add Linux-only `member_runtime.rs`
3. route `member_dispatch` through the member runtime manager
4. register remote member spans for cancel delivery
5. fail closed on binding mismatch, missing binary, or unsupported backend

Acceptance:

- world-agent launches from the shell-resolved runtime descriptor
- member dispatch fails closed on world mismatch or missing runtime facts
- remote cancel reports truthfully against the member span registry

Use the preserved worker patch as logic reference only. Do not blindly apply it.

### Step 3. Shell remote-member lane

Files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

Deliverables:

1. split host-local orchestrator launch from remote member launch explicitly
2. consume only the sanctioned crate-surface bridge
3. wire startup, readiness, cancel, and replacement through the remote path
4. preserve same-generation reuse for already-live remote members

Acceptance:

- first world-backed member launch crosses typed execute-stream
- `Ready` and `Running` require session-handle evidence
- replacement preserves lineage and fails closed honestly

### Step 4. Parent regression wall

Files:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)

Deliverables:

1. add status truth assertions for real remote producer state
2. add trace truth assertions for launch, cancel, and replacement
3. prove stale generation never revives after failed replacement

Acceptance:

- all verification commands below are green in order
- remote producer truth is observable through status and trace, not inferred from optimistic state

### Step 5. Closeout

Deliverables:

1. update run-state with accepted worker outputs and final validation
2. leave quarantined but unused worker artifacts as evidence only
3. record whether the bridge choice was direct type export or adapter fallback

## Definition of Done

1. the crate surface exposes a sanctioned request-construction bridge for member dispatch
2. `async_repl.rs` launches world-scoped members through the remote path only
3. `world-agent` owns member retained control inside the active world
4. `execute_cancel` reaches both process-exec spans and member spans
5. status and trace show the real remote producer
6. same-generation reuse still avoids redundant relaunch
7. replacement preserves lineage and fails closed honestly
8. all targeted tests pass
9. the run cannot be blocked again by missing plan authority for the crate-surface bridge

## Recommended verification commands

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
cargo test -p shell --test repl_world_first_routing_v1 --no-run
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test agent_hub_trace_persistence -- --nocapture
```

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
| 7 | Regression wall | Keep status and trace proof parent-owned after both runtime lanes merge | Mechanical | Blast radius instinct | Producer truth crosses lane boundaries and needs one integrator | Letting a worker lane redefine status truth alone |

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
explicitly thaw the request bridge, resume the two runtime lanes, and finish the placement-truth
regression wall without reopening the payload contract.
