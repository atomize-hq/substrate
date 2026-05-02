# PLAN-11_5: Resume PLAN-11 From The Blocked Gate C State

Source plan: [PLAN-11.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-11.md)  
Source SOW: [11-in-world-member-dispatch-over-existing-host-world-transport.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/11-in-world-member-dispatch-over-existing-host-world-transport.md)  
Blocked-run evidence:
- [.runs/plan-11/run-state.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/run-state.json)
- [.runs/plan-11/blocked.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/blocked.json)
- [.runs/plan-11/session.log](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/session.log)
- [.runs/plan-11/quarantined-parent-b1.patch](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/quarantined-parent-b1.patch)
- [.runs/plan-11/quarantined-parent-b2.patch](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/quarantined-parent-b2.patch)

Branch: `feat/session-centric-state-store`  
Plan type: continuation and unblock plan, Linux-first, no UI scope, strong runtime and DX scope  
Review posture: `/autoplan` scope discipline with `/plan-eng-review` depth  
Outside voice: unavailable on 2026-05-02 because `claude` CLI is installed but unauthenticated

## Objective

`PLAN-11` already did the hard honesty work.

The request contract refreeze landed. The harness surfaces landed. The blocked run stopped
because the shell worker could not legally consume the frozen builder from
[async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
through the actual public crate surface.

`PLAN-11_5` does not reopen the transport contract again.

It resumes from the accepted Gate A and Gate B carryover, adds the missing parent-owned
visibility bridge in
[crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs),
then reopens exactly one honest two-lane runtime window to finish:

1. the `world-agent` member runtime manager,
2. the shell remote-member cutover in `async_repl.rs`,
3. the status and trace regression wall that makes the placement claim true.

The required user outcome is unchanged:

- when a world-scoped member says it is live on generation `N` of world `W`,
- it is actually running inside `world-agent` on generation `N` of world `W`,
- with real remote cancel delivery, real replacement behavior, and real retained control,
- and `substrate agent status` plus trace rows do not lie about placement.

## Imported Stop Point

The current accepted parent state is:

- Gate A: passed
- Gate B: passed
- Gate C: blocked
- Gate D: blocked

The accepted carryover files are now read-only unless a new regression proves drift:

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)
- [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)

The quarantined worker patches are evidence and implementation hints only.

They are not accepted branch truth. The parent must selectively mine them only after the
missing export bridge lands.

## Step 0: Scope Challenge

### 0A. What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| typed member-dispatch contract with nested resolved runtime | [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs) | Reuse exactly. Do not reopen `MemberDispatchRequestV1` again in this continuation. |
| world request builder and transport payload shaping | [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs) | Reuse exactly. The builder is already public at the dispatch layer. |
| dispatch-module thin export seam | [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs) | Reuse exactly. The missing hop is above this file now. |
| crate-level routing re-export pattern | [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs) | Extend. This is the actual missing visibility bridge. |
| shell-owned member lifecycle semantics | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse startup, persistence, invalidation, and status semantics. Change placement only. |
| world-owned long-lived runtime management reference | [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs) | Reuse as the lifecycle pattern, not as the exact API surface. |
| canonical status and trace consumers | [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs), [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs), [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs) | Reuse. The regression wall only proves the producer is now remote. |

### 0B. Minimum diff decision

The smallest honest continuation is:

1. add the missing member-dispatch builder re-export in `routing.rs`,
2. reopen the same two runtime lanes from `PLAN-11`,
3. keep Gate A and Gate B carryover files frozen,
4. reject any shell-side workaround that reaches into private dispatch modules or hand-rolls
   request assembly again.

This matters because the blocked shell worker already demonstrated the failure mode. The
contract is not the problem anymore. The visibility chain is.

### 0C. Complexity check

This continuation still touches more than 8 files, but that is the minimal complete version.

The production path is:

1. [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs)
2. [crates/world-agent/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/Cargo.toml)
3. new `crates/world-agent/src/member_runtime.rs`
4. [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
5. [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
6. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
7. [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)
8. [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
9. [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
10. [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)

### 0D. Search-before-building result

- **[Layer 1]** Reuse the existing `routing.rs` re-export pattern. It already surfaces
  `build_agent_client_and_request(...)` and `build_agent_client_and_pending_diff_request(...)`.
- **[Layer 1]** Reuse the existing `/v1/execute/stream` and `/v1/execute/cancel` transport seam.
- **[Layer 3]** Do not let `world-agent` invent a second runtime selector. The shell already
  resolved the authoritative runtime facts, and the request contract now carries them.

### 0E. Completeness check

The shortcut would be to let `async_repl.rs` reach into
`crate::execution::routing::dispatch::world_ops::MemberDispatchTransportRequest` directly or
manually rebuild the transport request again.

Reject that shortcut.

It is a code-smell version of the exact drift the Gate B refreeze was supposed to eliminate.

### 0F. NOT in scope

- another `MemberDispatchRequestV1` shape change
- a `V2` rename for an unshipped internal seam
- a new public `/v1/member/*` API family
- a new shared startup crate
- shell state-store redesign
- macOS or Windows member-dispatch parity
- status/doctor UX redesign
- docs cleanup before the regression wall is green

## Architecture Review

### Findings

`[P1] (confidence: 10/10) crates/shell/src/execution/routing.rs:40-42 — the crate-level routing surface re-exports the ordinary world request builders, but not build_agent_client_and_member_dispatch_request(...), so async_repl cannot consume the frozen builder without breaking module boundaries.`

Recommendation:

- add `build_agent_client_and_member_dispatch_request` to the existing `pub(crate) use dispatch::{...}` block in `routing.rs`
- continue consuming it from `crate::execution::*`, the same way `async_repl.rs` already imports other routing helpers

`[P1] (confidence: 10/10) crates/shell/src/repl/async_repl.rs:2685-2711 — member startup still builds a local gateway and delegates to start_host_orchestrator_runtime_with_prepared(...), so the participant can claim world placement while execution still happens in the shell process.`

Recommendation:

- split member launch from host orchestrator launch
- keep local startup for the orchestrator path
- move member startup onto a remote prepared-launch carrier plus remote retained-control carrier

`[P1] (confidence: 9/10) crates/world-agent/src/service.rs:1223-1438 — execute_stream and execute_cancel only know how to run or signal ordinary process-exec spans today, so world-agent still lacks a world-owned retained-control registry for member dispatch spans.`

Recommendation:

- add a Linux-only `member_runtime.rs`
- register active member runtimes by `span_id`
- route `member_dispatch` requests through that manager and teach `execute_cancel` to target both process exec spans and member spans

### Architecture ASCII Diagram

```text
CURRENT BLOCKED STATE
=====================
async_repl.rs
    │
    ├── prepare_member_runtime_startup_for_descriptor(...)
    ├── start_member_runtime_with_prepared(...)
    │       └── still delegates to local host startup
    │
    └── cannot legally call the frozen transport builder
            because routing.rs does not re-export it

TARGET CONTINUATION STATE
=========================
crate::execution::routing.rs
    └── re-export build_agent_client_and_member_dispatch_request(...)
            │
            ▼
async_repl.rs
    ├── prepare remote member dispatch payload
    ├── call frozen builder through crate::execution::*
    ├── retain remote cancel + stream + completion ownership
    └── persist live-state transitions only after remote evidence
            │
            ▼
world-agent member_runtime.rs
    ├── validate authoritative world binding
    ├── build gateway from resolved_runtime
    ├── run_control(...) inside the world
    ├── forward Start/Event/Exit/Error
    └── service execute_cancel() reaches member spans
```

## Code Quality Review

### Findings

`[P2] (confidence: 9/10) crates/shell/src/execution/routing/dispatch/world_ops.rs:203-215 — MemberDispatchTransportRequest already centralizes the transport shape, so rebuilding those fields ad hoc in async_repl would reintroduce duplication across one of the most fragile seams in the slice.`

Recommendation:

- keep the mapping from shell lifecycle state to transport request in one helper path
- if `async_repl.rs` needs a convenience adapter, add it next to the exported builder surface, not as a local struct-construction fork

`[P2] (confidence: 8/10) .runs/plan-11/quarantined-parent-b2.patch — the blocked shell patch reached into a private routing path because the public export hop was missing, which means blindly applying the patch now would preserve the wrong dependency direction even if it compiles after edits.`

Recommendation:

- treat both quarantined patches as reference implementations
- cherry-pick logic, not file hunks
- re-read every changed import and helper boundary after the `routing.rs` export lands

## Test Review

### Test framework detection

- Runtime: Rust
- Framework: `cargo test`
- Packages: `world-agent`, `shell`
- No LLM eval suite is required for this slice

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/shell/src/execution/routing.rs
    │
    ├── [GAP]         crate::execution::* exposes build_agent_client_and_member_dispatch_request(...)
    └── [GAP]         neighboring world-request exports remain unchanged

[+] crates/world-agent/src/member_runtime.rs + service.rs
    │
    ├── [GAP]         success path launches from resolved_runtime inside the world
    ├── [GAP]         authoritative world mismatch fails before startup
    ├── [GAP]         unsupported backend or missing binary fails closed
    ├── [GAP]         execute-cancel reaches member spans and converges terminally
    └── [GAP]         pre-ready failure and post-ready terminal close produce distinct honest states

[+] crates/shell/src/repl/async_repl.rs
    │
    ├── [GAP] [->E2E] first world-backed command uses typed member dispatch through crate::execution::*
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
COVERAGE: 0/16 remaining continuation paths proven
GAPS: 16 paths require coverage before Gate D can pass
─────────────────────────────────
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] First world-backed REPL command
    ├── [GAP] [->E2E] shell selects the member, serializes resolved runtime, and launches remotely
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
   - add member-dispatch success, authoritative-world mismatch, missing binary, unsupported backend, cancel, and abnormal terminal cases
2. `crates/shell/tests/repl_world_first_routing_v1.rs`
   - add first launch, same-generation reuse, remote readiness, replacement launch, and replacement failure cases
3. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
   - add status truth checks for a real remotely launched member
4. `crates/shell/tests/agent_hub_trace_persistence.rs`
   - add trace truth checks for remote member launch, cancel, and replacement

### Test artifact

The eng-review QA artifact for this continuation is:

- [spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260502-184500.md](/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260502-184500.md)

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
| --- | --- | --- | --- | --- | --- |
| routing export bridge | `async_repl.rs` reaches into private routing modules again and drifts away from the frozen builder surface | no | no | no | yes |
| remote launch split | shell persists a world-scoped member as live while startup still runs locally | no | no | no | yes |
| authoritative world binding | world-agent launches against a stale or mismatched world binding | no | not yet | partial | yes |
| resolved runtime path | shell provides a binary path that is missing or not executable inside the Linux world | no | partial | partial | yes |
| remote readiness gate | readiness advances before the session-handle event arrives | no | partial | no | yes |
| cancel path | execute-cancel misses the member span and status keeps advertising live | no | partial | partial | yes |
| replacement wall | replacement fails and stale generation still appears live in status or trace | no | partial | no | yes |

Critical gap rule:

If any path can advertise a live world member without a real remote session handle, a real
remote cancel path, and a real remote terminal observer, Gate D fails.

## Performance Review

This is still correctness-first.

Performance cautions:

1. reuse the existing `AgentClient` transport stack. Do not add a second member-specific client.
2. keep remote stream handling incremental. Do not buffer whole member sessions in memory.
3. preserve same-generation reuse so the steady-state path does not relaunch on every world-backed command.

There is no new throughput blocker beyond that. The slice is spending complexity on honesty, not
on speculative optimization.

## DX Guardrails

This is a developer tool. Failure messages matter.

Required error-message posture:

1. parent-side export failures must name the missing surface, not just emit a generic visibility error
2. remote launch failures must include `participant_id`, `world_id`, `world_generation`, and backend kind
3. cancel failures must say whether delivery failed before span registration or after remote startup

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| P0. Parent visibility bridge | `crates/shell/src/execution/` | — |
| L1. World-agent member runtime lane | `crates/world-agent/` | P0 |
| L2. Shell remote-member lane | `crates/shell/src/repl/`, `crates/shell/tests/` | P0 |
| P1. Parent regression wall | `crates/shell/tests/` | L1, L2 |

### Parallel lanes

- Lane A: `P0` only, sequential, parent-owned
- Lane B: `L1`, independent after `P0`
- Lane C: `L2`, independent after `P0`
- Lane D: `P1`, sequential after B + C merge

### Execution order

1. Land `P0` in the parent checkout.
2. Run `cargo test -p shell --lib -- --nocapture` to prove the new export surface is valid.
3. Reopen exactly one two-lane worker window: `L1` + `L2`.
4. Merge or manually integrate accepted outputs from both lanes.
5. Run the parent-owned regression wall in `P1`.

### Conflict flags

- `L1` and `L2` are parallel only if neither reopens the frozen carryover files from Gate A/B.
- If either lane needs to edit `agent-api-types`, `dispatch/world_ops.rs`, `dispatch/prelude.rs`, or `routing.rs` after `P0`, stop the run again. That means the unblock assumptions were wrong.

### Parallelization verdict

One real parallel window remains. Worker cap stays exactly `2`.

## Deferred Work

- shared startup-crate extraction, if a third consumer ever makes the duplication real
- non-Linux member-dispatch parity
- docs cleanup after the regression wall is green
- status/doctor UX polish after placement truth is proven

No new `TODOS.md` entry is required in this continuation. Every deferred item above is an explicit
non-goal of the slice, not a forgotten follow-up.

## Implementation Sequence

### Step 1. Parent visibility bridge

Deliverables:

1. add `build_agent_client_and_member_dispatch_request` to the `routing.rs` re-export block
2. keep Gate A/B carryover files unchanged
3. prove the shell crate can see the builder through `crate::execution::*`

Acceptance:

- `async_repl.rs` no longer needs to reference private routing internals
- `cargo test -p shell --lib -- --nocapture` passes before reopening workers

### Step 2. Reopen the world-agent lane

Deliverables:

1. add `agent_api` to `crates/world-agent/Cargo.toml`
2. add Linux-only `member_runtime.rs`
3. route `member_dispatch` requests from `service.rs`
4. register member runtimes by `span_id`
5. extend `execute_cancel` to target member spans

Acceptance:

- `world-agent` launches the selected backend from the shell-resolved runtime descriptor
- member dispatch fails closed on binding mismatch or missing runtime facts

### Step 3. Reopen the shell runtime lane

Deliverables:

1. split host-orchestrator and remote-member prepared launch shapes
2. consume the exported builder through `crate::execution::*`
3. add explicit remote retained-control state
4. wire startup, readiness, cancel, and replacement through the remote path

Acceptance:

- first world-backed member launch crosses typed execute-stream
- same-generation reuse still works
- replacement launches preserve lineage and fail closed honestly

### Step 4. Parent regression wall

Deliverables:

1. land status truth assertions
2. land trace truth assertions
3. verify stale generation never revives after failed replacement

Acceptance:

- all verification commands below are green in order

## Definition of Done

1. `routing.rs` exports the frozen member-dispatch builder at the crate surface
2. `async_repl.rs` launches world-scoped members through the remote path only
3. `world-agent` owns member retained control inside the world
4. `execute_cancel` reaches both process-exec spans and member spans
5. status and trace show the real remote producer
6. same-generation reuse still avoids redundant relaunch
7. replacement preserves lineage and fails closed honestly
8. all targeted tests pass
9. `.runs/plan-11/blocked.json` remains historical evidence, not active truth for the continuation

## Recommended verification commands

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test agent_hub_trace_persistence -- --nocapture
```

Compile-only gates still use Cargo's real form:

```bash
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
cargo test -p shell --test repl_world_first_routing_v1 --no-run
```

## Completion Summary

- Step 0: scope accepted as a Gate C/Gate D continuation, not a restart
- Architecture Review: 3 issues found and resolved in-plan
- Code Quality Review: 2 issues found and resolved in-plan
- Test Review: diagram produced, 16 remaining continuation gaps identified
- Performance Review: 3 cautions, 0 throughput blockers
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 durable TODOs proposed
- Failure modes: 7 critical gaps called out for the implementation wall
- Outside voice: unavailable because `claude` CLI auth is missing
- Parallelization: 4 execution phases, 1 real parallel window, worker cap stays `2`
- Lake Score: complete option chosen over a private-module shortcut or a new shared crate

<!-- AUTONOMOUS DECISION LOG -->
## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Scope | Treat Gate A/B as accepted carryover and plan only the blocked remainder | Mechanical | Minimal diff | The transport contract and harness proof already landed | Restarting `PLAN-11` from zero |
| 2 | Export seam | Add the missing visibility bridge in `crates/shell/src/execution/routing.rs` | Mechanical | Explicit over clever | `async_repl.rs` consumes `crate::execution::*`, not private dispatch modules | Reaching into `routing::dispatch::*` directly |
| 3 | Contract | Keep `MemberDispatchRequestV1` frozen as-is in this continuation | Mechanical | DRY | Reopening the request shape again would create churn without addressing the actual blocker | Another contract refreeze |
| 4 | Worker outputs | Use quarantined B1/B2 patches as references only | Mechanical | Systems over heroes | The patches were produced against an incomplete visibility surface | Blind patch application |
| 5 | Runtime startup | Keep direct `unified-agent-api` usage in `world-agent` | Mechanical | Engineered enough | The world side must call the real retained-control path now | Shared startup-crate extraction |
| 6 | Shell cutover | Split host-local orchestrator control from remote member control explicitly | Taste | Explicit over clever | The current coupling is exactly what preserves the placement lie | Reusing the host-local runtime path for members |
| 7 | Parallelism | Keep one worker window with cap `2` after the parent refreeze | Mechanical | Blast radius instinct | `world-agent` and `async_repl` are separable only after `routing.rs` lands | Opening more lanes or keeping everything parent-only |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
| --- | --- | --- | --- | --- | --- |
| CEO Review | `/plan-ceo-review` | Scope and strategy | 1 | CLEAR | Kept the continuation narrow: fix the export hop, finish the runtime cutover, do not reopen the contract or widen scope |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | SKIPPED | No separate outside-model review run, and `claude` CLI auth is missing for outside voice |
| Eng Review | `/plan-eng-review` | Architecture and tests (required) | 1 | CLEAR | Locked the real blocker to `routing.rs`, preserved Gate A/B carryover, and defined the exact regression wall for remote launch, cancel, replacement, status, and trace truth |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | SKIPPED | No UI scope |

**UNRESOLVED:** 0 plan-level decisions remain. The remaining work is implementation plus
verification only.

**VERDICT:** CEO + ENG CLEARED. `PLAN-11_5` is the honest continuation plan for the blocked
`PLAN-11` run: land the missing crate-surface export, reopen one two-lane runtime window, and
finish the placement-truth regression wall.
