# PLAN-11_5: Refreeze Member Dispatch Runtime Descriptor And Complete In-World Cutover

Supersedes: the blocked remainder of [PLAN-11.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-11.md) only.  
Depends on accepted carryover artifacts from the blocked `PLAN-11` run:
- [.runs/plan-11/run-state.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/run-state.json)
- [.runs/plan-11/blocked.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/blocked.json)
- [.runs/plan-11/session.log](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-11/session.log)

Branch: `feat/session-centric-state-store`  
Plan type: continuation and unblock plan, Linux-first, no UI scope, strong runtime and DX scope  
Review posture: `/autoplan`-style consolidation with `/plan-eng-review` rigor  
Outside voice: unavailable on 2026-05-02 because `claude` CLI is installed but unauthenticated on this machine

## Objective

`PLAN-11` found a real seam and stopped honestly.

The accepted freeze work already proved three things:

1. the shell can serialize typed member dispatch over the existing `/v1/execute/stream` seam,
2. the shell test harnesses can capture and script that traffic,
3. the old freeze was still missing the one fact `world-agent` actually needs to start the member with the same UAA retained-control path the shell uses today.

`PLAN-11_5` does not start over.

It resumes from the accepted Gate A and Gate B carryover, reopens the unshipped internal request contract once, adds the resolved runtime descriptor that the blocked run proved is mandatory, fixes the missing builder export seam, then completes the original runtime cutover and regression wall.

The required user outcome remains unchanged:

- when a world-scoped member says it is live on generation `N` of world `W`,
- it is actually running inside `world-agent` on generation `N` of world `W`,
- with real remote retained control, real remote cancel delivery, real replacement behavior,
- and producer-backed `status` plus trace rows that do not lie about placement.

## Why PLAN-11 Blocked

The blocker is not hypothetical. It is captured in the run artifacts above.

### Accepted carryover from the blocked run

The blocked parent branch still contains accepted freeze work in:

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)

Those surfaces already prove:

1. deserialize-time `ExecuteRequest` boundary validation exists,
2. typed `member_dispatch` exists,
3. ready-event capture is pinned to the existing session-handle `AgentEvent` contract,
4. the transport seam is inspectable in shell tests.

### The two real missing decisions

The blocked run exposed two real gaps:

1. **Parent-fixable shell seam**
   - [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs) does not re-export the frozen member-dispatch builder, so the shell lane could not consume the already-frozen transport helper from [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).
2. **Actual hard stop**
   - the frozen `member_dispatch` payload carries lineage, backend identity, protocol, run id, and world identity,
   - but it does **not** carry the resolved runtime descriptor from [RuntimeSelectionDescriptor](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs),
   - and `world-agent` cannot honestly rebuild the same UAA startup path from `backend_id` plus `protocol` alone without inventing a second resolver.

That second gap is the whole game.

## Step 0: Scope Challenge

### 0A. What already exists and must be reused

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| runtime selection and resolved binary path | [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) | Reuse. This remains the only authoritative selector. |
| gateway-backed UAA startup in the shell | [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs), [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse the startup model, not the location. |
| session authority, liveness, lineage, invalidation | [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs), [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse exactly. No new store semantics. |
| typed execute transport and capture harness | [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs), [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs), [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs), [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs) | Reuse, but refreeze with the resolved descriptor added. |
| Linux-first runtime-management rigor in world-agent | [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs) | Reuse as the lifecycle rigor reference. |
| ordinary execute-stream and execute-cancel routing | [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs) | Reuse as the branch point only. |

### 0B. Decision: choose contract refreeze, reject shared-crate extraction

The blocked run left two plausible unblock paths:

| Approach | Decision | Why |
| --- | --- | --- |
| Carry the resolved runtime descriptor over `member_dispatch` and let `world-agent` consume it directly | **Accepted** | Smallest honest diff. It removes the second resolver and gives `world-agent` the exact launch facts the shell already resolved. |
| Extract a new shared gateway/UAA startup crate first, then make both shell and `world-agent` consume it | **Rejected for this slice** | Overbuilt. It spends an innovation token, widens the blast radius, and still does not remove the need to carry the resolved launch facts over transport. |

This is the key architecture decision in `PLAN-11_5`.

### 0C. Exact refreeze decision

Keep the existing internal type name `MemberDispatchRequestV1`.

Do **not** pay a fake `V2` tax for an unshipped internal seam. The previous freeze was never released outside this branch. Rewriting the same internal `V1` before the runtime cutover is the minimal diff.

Refreeze `member_dispatch` by adding a required nested resolved-runtime payload:

```text
MemberDispatchRequestV1
    ├── schema_version
    ├── orchestration_session_id
    ├── participant_id
    ├── orchestrator_participant_id
    ├── parent_participant_id
    ├── resumed_from_participant_id
    ├── backend_id
    ├── protocol
    ├── run_id
    ├── world_id
    ├── world_generation
    └── resolved_runtime: ResolvedMemberRuntimeDescriptorV1

ResolvedMemberRuntimeDescriptorV1
    ├── backend_kind
    └── binary_path
```

Rules:

1. `backend_kind` is explicit and boring. No backend-kind inference from `backend_id`.
2. `binary_path` is the already-resolved absolute path from the shell selector.
3. the top-level `agent_id` remains authoritative for traces, budgets, and diagnostics.
4. `protocol` remains part of the transport identity contract even though `world-agent` uses `resolved_runtime.backend_kind` for UAA backend construction.
5. validation remains at the deserialize/parse boundary in [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs).

### 0D. Complexity check

The plan still touches more than 8 files. That is acceptable and still the minimal complete version.

The tight production path is:

1. [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
2. [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
3. [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)
4. [crates/world-agent/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/Cargo.toml)
5. [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
6. new `crates/world-agent/src/member_runtime.rs`
7. [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
8. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Everything else is regression wall and harness proof.

### 0E. Completeness check

The shortcut path would be:

- keep the old contract,
- special-case local launch or backend-specific CLI startup in `world-agent`,
- and try to infer readiness from something weaker than the existing session-handle event contract.

That shortcut is bad software.

It would save almost nothing with CC+gstack and it would preserve the exact lie this slice exists to remove.

### 0F. NOT in scope

- a new public `/v1/member/*` or `/v1/agents/*` API family
- a new shared internal crate for gateway/UAA startup
- shell store/schema redesign
- status/doctor/toolbox redesign
- macOS or Windows member-dispatch parity
- auth-bundle redesign beyond continuing to use the existing execute request environment carrier
- broad backend expansion beyond the backends already supported by the shell-owned UAA path
- UI work

## Architecture Contract

### Hard invariants

1. The shell remains the only canonical writer of orchestration session state and participant state.
2. `world-agent` owns in-world member execution, remote cancel delivery, event streaming, and completion observation only.
3. World-scoped member launch fails closed. No host fallback.
4. `/v1/execute/stream` and `/v1/execute/cancel` remain the only transport seam.
5. `ExecuteStreamFrame::{Start,Event,Exit,Error}` remain the only stream families.
6. Remote readiness still depends on the existing session-handle event contract unless the parent explicitly refreezes it before worker lanes open.
7. The shell must represent retained control explicitly as local vs remote.
8. The shell must use an explicit remote-prepared launch shape for members.
9. Linux-first remains explicit. Non-Linux member dispatch fails closed.

### New contract detail: resolved runtime descriptor

`world-agent` must not perform runtime selection from agent inventory, effective config, or host-side shell-only helpers.

The shell already did that work in [validate_runtime_realizability(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs).

`PLAN-11_5` makes that result transport-visible instead of forcing `world-agent` to guess.

### New dependency decision

Add direct `unified-agent-api` dependency to [crates/world-agent/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/Cargo.toml).

Do not extract a new shared crate in this slice.

Rationale:

1. `world-agent` must actually call the same UAA retained-control path the shell already trusts.
2. the only new cross-crate drift risk that matters here is the launch descriptor, and the transport contract now pins that explicitly.
3. duplicating a tiny backend registration helper is acceptable in-slice; extracting a new crate is deferred until a third backend or a second consumer makes the duplication real.

### Builder export decision

Re-export `build_agent_client_and_member_dispatch_request(...)` through:

- [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)

This is parent-owned pre-worker glue. Do not let the shell lane reopen transport modules ad hoc.

### Data flow

```text
shell selector
    │
    ├── validate_runtime_realizability(...)
    │       └── resolves backend_kind + absolute binary_path
    │
    ├── build typed ExecuteRequest.member_dispatch
    │       └── includes resolved_runtime
    │
    ▼
world-agent execute_stream(member_dispatch)
    │
    ├── validate request boundary
    ├── validate authoritative world binding
    ├── build UAA gateway from resolved_runtime
    ├── start run_control(...) inside world-agent
    ├── retain remote cancel + events + completion
    └── emit Start / Event / Exit / Error
    │
    ▼
shell retained-control consumer
    │
    ├── records remote span_id
    ├── waits for session-handle Event
    ├── marks retained ownership
    ├── persists Allocating -> Ready -> Running honestly
    └── publishes status and trace from canonical shell state
```

### Retained-control model

`PLAN-11` was already right about the shell-side abstraction. `PLAN-11_5` keeps it.

```text
RetainedRuntimeControl
    ├── LocalUaa(RetainedRunControl)
    └── RemoteMember(RemoteMemberControl)

RemoteMemberControl
    ├── span_id
    ├── stream_task
    ├── cancel_path
    └── completion_observer
```

What changes is that the remote member path now has the data needed to exist for real.

## File Plan

### 1. `crates/agent-api-types/src/lib.rs`

Deliver:

- `ResolvedMemberRuntimeDescriptorV1`
- required `MemberDispatchRequestV1.resolved_runtime`
- parse-boundary validation for `backend_kind` and `binary_path`
- updated round-trip and invalid-shape tests

Do not:

- rename the transport family to `V2`
- add a generic execute target enum

### 2. `crates/shell/src/execution/routing/dispatch/world_ops.rs`

Deliver:

- updated member-dispatch builder carrying `resolved_runtime`
- keep existing world env and trace plumbing

Do not:

- move lifecycle logic into this file

### 3. `crates/shell/src/execution/routing/dispatch/prelude.rs`

Deliver:

- re-export of `build_agent_client_and_member_dispatch_request(...)`

Do not:

- widen the public routing surface any further than the one helper the shell lane actually needs

### 4. `crates/shell/tests/support/socket.rs`

Deliver:

- updated request capture asserting nested `resolved_runtime`

### 5. `crates/shell/tests/support/repl_world_agent.rs`

Deliver:

- updated typed request capture including nested `resolved_runtime`
- keep the accepted ready-event and cancel scripting contracts unchanged

### 6. `crates/world-agent/Cargo.toml`

Deliver:

- direct `unified-agent-api` dependency needed for real UAA startup in-world

### 7. `crates/world-agent/src/member_runtime.rs`

New internal module.

Deliver:

- resolved-runtime validation
- small backend registration helper using `backend_kind` plus `binary_path`
- in-world `run_control(...)` startup
- remote cancel/event/completion retention
- terminal cleanup

### 8. `crates/world-agent/src/service.rs`

Deliver:

- `execute_stream(...)` branch for typed member dispatch
- `execute_cancel(...)` delivery to member-dispatch spans

Do not:

- turn `service.rs` into the runtime implementation

### 9. `crates/world-agent/src/lib.rs`

Deliver:

- module exposure only as needed for the new internal manager

### 10. `crates/shell/src/repl/async_repl.rs`

Deliver:

- explicit `PreparedMemberDispatch`
- explicit `RetainedRuntimeControl`
- remote member startup through exported typed builder
- same-generation reuse
- honest cancel and replacement convergence

Do not:

- change host orchestrator startup behavior
- invent a second lifecycle model

### 11. Late regression wall

Primary regression files remain:

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)

## Implementation Sequence

### Step 1. Import carryover state and explicitly invalidate the old freeze

Deliverables:

1. carry forward the accepted A1/A2/A3 work from the blocked run,
2. mark the old `member_dispatch` freeze as superseded inside the new run state,
3. keep the accepted ready-event scripting and harness capture work.

Acceptance:

- no worker lane opens before the new contract refreeze lands

### Step 2. Refreeze the request contract and builder seam

Deliverables:

1. add `resolved_runtime` to `MemberDispatchRequestV1`
2. update builder and harness capture
3. re-export the builder through `dispatch/prelude.rs`
4. update compile and contract tests

Acceptance:

- the shell lane can consume the frozen builder through the normal routing seam
- the world-agent lane receives the full resolved launch facts without local re-resolution

### Step 3. Implement the world-agent runtime lane

Deliverables:

1. add direct `unified-agent-api` dependency
2. build the UAA gateway from `resolved_runtime`
3. start run control inside `world-agent`
4. retain cancel/event/completion and emit the existing stream family

Acceptance:

- no backend-specific stdout parsing
- no local re-resolution from inventory
- no shell-private helper dependency

### Step 4. Implement the shell runtime lane

Deliverables:

1. consume the exported builder
2. add explicit remote-prepared launch shape
3. add explicit remote retained-control carrier
4. wire cancel, startup, and replacement through the remote path

Acceptance:

- first world-backed command launches over typed execute-stream
- same-generation reuse still works
- failed preflight still stops before transport

### Step 5. Land the regression wall

Deliverables:

1. cancel reaches member-dispatch spans
2. replacement launch carries fresh `participant_id` and correct `resumed_from_participant_id`
3. `status` and trace remain participant-correct for the real remote producer
4. stale liveness never revives

Acceptance:

- all targeted tests below are green in order

## Test Review

### Test framework detection

- Runtime: Rust
- Framework: `cargo test`
- Packages: `agent-api-types`, `world-agent`, `shell`
- No separate LLM eval suite is required for this slice

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/agent-api-types/src/lib.rs
    │
    ├── member_dispatch request boundary
    │   ├── [GAP]         resolved_runtime.backend_kind validates
    │   ├── [GAP]         resolved_runtime.binary_path validates as non-empty absolute path
    │   ├── [GAP]         old invalid mixed cmd/member_dispatch shapes still fail
    │   └── [GAP]         ordinary process exec remains unchanged
    │
    └── transport round trip
        └── [GAP]         nested resolved_runtime survives serde round trip

[+] crates/shell/src/execution/routing/dispatch/world_ops.rs + prelude.rs
    │
    ├── builder export seam
    │   ├── [GAP]         async_repl can import the builder through dispatch prelude
    │   └── [GAP]         builder carries backend_kind + binary_path correctly
    │
    └── harness capture
        └── [GAP]         typed request capture asserts resolved_runtime

[+] crates/world-agent/src/member_runtime.rs + service.rs
    │
    ├── in-world UAA startup
    │   ├── [GAP]         resolved runtime builds the correct backend
    │   ├── [GAP]         unsupported backend_kind fails closed
    │   ├── [GAP]         missing binary path fails before startup
    │   └── [GAP]         session-bearing Event still surfaces readiness
    │
    ├── cancel path
    │   └── [GAP]         execute-cancel reaches member span and converges terminally
    │
    └── abnormal paths
        ├── [GAP]         bootstrap failure before readiness -> Failed
        ├── [GAP]         stream closes before readiness -> Failed
        └── [GAP]         stream closes after readiness -> Invalidated

[+] crates/shell/src/repl/async_repl.rs
    │
    ├── remote prepared launch
    │   ├── [GAP] [->E2E] first world-backed command uses typed member dispatch with resolved runtime
    │   ├── [GAP]         Allocating persists before remote ownership
    │   ├── [GAP]         Ready waits for the real session-handle event
    │   └── [GAP]         Running requires retained ownership, not just any Event
    │
    ├── remote retained control
    │   ├── [GAP]         shutdown uses execute-cancel for member spans
    │   └── [GAP]         cancel failure converges the member non-live
    │
    └── replacement flow
        ├── [GAP] [->E2E] world rollover replacement crosses the same transport seam
        └── [GAP] [->E2E] replacement failure leaves honest absence

[+] shell status and trace wall
    │
    ├── [GAP]         status shows a real remotely launched member
    ├── [GAP]         trace rows are emitted by the real remote producer
    └── [GAP]         stale generation never revives live appearance after failure

─────────────────────────────────
COVERAGE: 0/21 new unblock paths tested yet
GAPS: 21 paths require coverage before the cutover is accepted
─────────────────────────────────
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] First world-backed REPL command
    ├── [GAP] [->E2E] Shell selects the world member, serializes resolved runtime, and launches remotely
    ├── [GAP]         Missing parent or world binding fails before transport
    └── [GAP]         Same-generation reuse does not relaunch

[+] Live member cancel and shutdown
    ├── [GAP] [->E2E] Shutdown issues execute-cancel against the remote span
    ├── [GAP]         Cancel delivery failure leaves non-live terminal state
    └── [GAP]         Clean cancel becomes Stopped, not Invalidated

[+] Shared-world rollover with live member
    ├── [GAP] [->E2E] Replacement launch includes resolved runtime and preserved lineage
    ├── [GAP]         Replacement bootstrap failure leaves no authoritative-live member
    └── [GAP]         Old generation never regains liveness

[+] Operator inspection
    ├── [GAP]         substrate agent status --json reflects the real remote producer
    └── [GAP]         trace rows remain participant-correct after cancel and replacement
```

### Required tests to add

1. `crates/agent-api-types/src/lib.rs`
   - add request-boundary tests for nested `resolved_runtime`
2. `crates/shell/tests/support/socket.rs`
   - assert captured requests expose `resolved_runtime`
3. `crates/shell/tests/support/repl_world_agent.rs`
   - assert member-dispatch scripts receive `resolved_runtime`
4. `crates/world-agent/tests/streamed_execute_cancel_v1.rs`
   - add success, unsupported backend, missing binary, cancel, and abnormal termination cases
5. `crates/shell/tests/repl_world_first_routing_v1.rs`
   - add first launch, same-generation reuse, replacement launch, and replacement failure cases
6. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
   - add status correctness for real remote member launch
7. `crates/shell/tests/agent_hub_trace_persistence.rs`
   - add producer-backed trace correctness for remote member lifecycle

### Test artifact

The eng-review QA artifact for this unblock plan is:

- [/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260502-154419.md](/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260502-154419.md)

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
| --- | --- | --- | --- | --- | --- |
| request refreeze | shell emits typed member dispatch without `resolved_runtime` and world-agent guesses | no | no | no | yes |
| builder export seam | async_repl bypasses the frozen builder and hand-rolls request JSON | no | no | no | yes |
| resolved binary path | shell resolves a binary path that is missing or non-executable inside the Linux world | no | not yet | partial | yes |
| backend-kind startup | world-agent builds the wrong backend for the declared participant | no | not yet | no | yes |
| retained-control split | shell treats remote state like a local cancel handle and leaves live state behind | no | not yet | no | yes |
| remote readiness | readiness advances before the session-handle event arrives | no | partial | no | yes |
| cancel delivery | execute-cancel misses the member span and status keeps advertising live | no | partial | partial | yes |
| replacement wall | replacement launch fails and stale lineage still looks live in status or trace | no | partial | no | yes |

Critical gap rule:

If any path can still advertise a world member live without a real remote session handle, a real remote cancel path, and real remote completion observation, the slice is not done.

## Performance Review

This is still correctness-first.

Performance cautions:

1. do not introduce a second transport client stack for member dispatch
2. do not re-run member startup if the same-generation remote member is already authoritative-live
3. keep execute-cancel bounded
4. do not add any new full-store scan on the steady-state happy path just to support the refreeze

Performance verdict:

- 0 throughput blockers
- 1 structural caution: remote cancel bookkeeping must stay bounded and span-indexed

## DX Guardrails

This slice is for developers and operators, not end-users.

Required DX posture:

1. the resolved runtime descriptor must be inspectable in tests, not hidden behind opaque helper state
2. failure messages must name the problem and the fix
3. the unblock plan must reduce "why did PLAN-11 block?" from code archaeology to one plan section and one blocked artifact link
4. proving real placement should remain under 10 minutes by reading plan plus tests

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| A. Carryover import and contract refreeze | `crates/agent-api-types/`, `crates/shell/src/execution/routing/dispatch/`, `crates/shell/tests/support/` | — |
| B. World-agent in-world UAA runtime | `crates/world-agent/` | A |
| C. Shell remote member cutover | `crates/shell/src/repl/`, `crates/shell/tests/` | A |
| D. Regression wall | `crates/shell/tests/`, `crates/world-agent/tests/` | B + C |

### Parallel lanes

- Lane A: parent-only contract refreeze
- Lane B: world-agent runtime lane after A
- Lane C: shell remote-cutover lane after A
- Lane D: parent-only regression wall after B + C

### Execution order

1. Run Lane A first.
2. After A lands, launch B and C in parallel.
3. Merge B and C into the parent branch.
4. Run D last.

### Conflict flags

- B and C both depend on the refrozen request shape. If either lane requests another contract change, stop and refreeze in the parent.
- C and D both touch shell tests. D stays last.
- The accepted ready-event contract remains shared risk. Do not let either worker rename it independently.

### Parallelization verdict

Same worker cap as the blocked run: exactly `2`.

That is still the honest cap. There are still only two real runtime seams after the parent-owned refreeze.

## Deferred Work

1. Extract a shared gateway/UAA startup helper only if a third backend or a second consumer makes the duplication real.
2. Revisit explicit wire version bump only if the internal member-dispatch seam becomes externally consumed.
3. Revisit secret-safe auth-bundle transport only after this placement seam is boring in production.
4. Add macOS and Windows parity later.

## Definition of Done

This continuation slice is done only when all of the following are true:

1. the member-dispatch request carries the resolved runtime descriptor
2. async_repl consumes the exported builder, not a hand-rolled request
3. world-agent starts the member with the real UAA retained-control path inside the world boundary
4. the shell uses explicit remote retained control for members
5. cancel reaches live member spans through `/v1/execute/cancel`
6. failure before readiness becomes `Failed`
7. stream loss after readiness becomes `Invalidated`
8. replacement after world rollover crosses the same transport seam with correct lineage
9. `status` and trace remain producer-backed and participant-correct
10. no host fallback exists

## Recommended verification commands

```bash
cargo test -p agent-api-types -- --nocapture
cargo test -p shell --lib -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test agent_hub_trace_persistence -- --nocapture
```

Compile-only gates must use Cargo's real form:

```bash
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
cargo test -p shell --test repl_world_first_routing_v1 --no-run
```

Not:

```bash
cargo test -p shell --test repl_world_first_routing_v1 -- --no-run
```

## Completion Summary

- Step 0: scope accepted as a continuation unblock, not a restart
- Architecture Review: 4 issues found and resolved in-plan
- Code Quality Review: 2 issues found and resolved in-plan
- Test Review: diagram produced, 21 unblock gaps identified
- Performance Review: 1 structural caution, 0 throughput blockers
- NOT in scope: written
- What already exists: written
- Failure modes: 8 critical gaps called out for the implementation wall
- Outside voice: unavailable because `claude` CLI auth is missing
- Parallelization: 4 execution phases, 1 real parallel window, worker cap stays `2`
- Lake Score: complete option chosen for the unblock instead of shortcut fallback or overbuilt extraction

<!-- AUTONOMOUS DECISION LOG -->
## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Scope | Treat accepted Gate A and Gate B work as carryover, not throwaway | Mechanical | Minimal diff | The blocked run already proved and landed useful freeze work | Restarting `PLAN-11` from zero |
| 2 | Contract | Reopen `MemberDispatchRequestV1` and add `resolved_runtime` | Taste | Completeness | The blocked run proved this fact is mandatory for honest remote startup | Keeping the old freeze and guessing remotely |
| 3 | Versioning | Keep the internal type name `V1` | Taste | Minimal diff | The old freeze was unshipped internal branch state, not a released public protocol | Renaming the whole seam to `V2` |
| 4 | Runtime startup | Add direct `unified-agent-api` dependency to `world-agent` | Mechanical | Explicit over clever | The world side must call the real retained-control path, not fake it | Backend-specific CLI stdout parsing |
| 5 | Reuse | Do not add a new shared startup crate in this slice | Mechanical | Engineered enough | A new crate widens scope and still does not remove transport facts that must be carried | Shared-crate extraction before proving the cutover |
| 6 | Builder seam | Re-export the frozen member-dispatch builder through `dispatch/prelude.rs` | Mechanical | DRY | The shell lane should consume the same builder the transport freeze already pinned | Hand-built request assembly in `async_repl.rs` |
| 7 | Backend semantics | Carry explicit `backend_kind` instead of inferring it from `backend_id` | Mechanical | Explicit over clever | Hidden inference is exactly the wrong move in a seam that already blocked once | Parsing `backend_id` for runtime meaning |
| 8 | State authority | Keep canonical store semantics unchanged | Mechanical | Systems over heroes | The state model is already correct; the problem is placement honesty | Remote canonical-state writes |
| 9 | Parallelism | Keep exact worker cap at `2` | Mechanical | Blast radius instinct | There are still only two honest runtime lanes after the parent-owned refreeze | Artificial third lane for status or docs |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
| --- | --- | --- | --- | --- | --- |
| CEO Review | `/plan-ceo-review` | Scope and strategy | 5 | CLEAR | Kept the slice narrow, reused the accepted freeze carryover, and chose contract refreeze over shared-crate extraction |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | SKIPPED | No separate outside-model review run, and `claude` CLI auth is missing for outside voice |
| Eng Review | `/plan-eng-review` | Architecture and tests (required) | 5 | CLEAR | Locked the resolved-runtime refreeze, the builder export seam, the direct world-agent UAA startup path, and the full regression wall for cancel/replacement/status/trace truth |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | SKIPPED | No UI scope |

**UNRESOLVED:** 0 plan-level decision points remain. The remaining work is implementation and verification only.

**VERDICT:** CEO + ENG CLEARED. `PLAN-11_5` is the honest continuation plan that picks up from the blocked `PLAN-11` state, fixes the missing contract facts, and finishes the original in-world placement cutover without widening scope.
