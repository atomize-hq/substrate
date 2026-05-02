<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/feat-session-centric-state-store-autoplan-restore-20260502-140843.md -->

# PLAN-11: In-World Member Dispatch Over Existing Host<->World Transport

Source file: [11-in-world-member-dispatch-over-existing-host-world-transport.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/11-in-world-member-dispatch-over-existing-host-world-transport.md)  
Branch: `feat/session-centric-state-store`  
Plan type: host<->world transport placement seam, no UI scope, strong DX scope  
Review posture: `/autoplan`-style consolidation with `/plan-eng-review` rigor, tightened into one execution document  
Status: execution-ready after [PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-10.md), with outside voice skipped on 2026-05-02 because `claude` CLI auth is missing

## Objective

This slice is not a second agent hub.

It is the honesty pass on member placement.

After [PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-10.md), the shell can already:

- select one world-scoped member backend,
- persist canonical orchestrator and member participants,
- invalidate stale generations,
- create replacement lineage,
- and project that state through `substrate agent status`.

What it still cannot do honestly is run the member inside the world boundary it claims to be
bound to.

Today `start_member_runtime_with_prepared(...)` in
[crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
still delegates to `start_host_orchestrator_runtime_with_prepared(...)`, which means the
member's `gateway.run_control(...)` turn still starts inside the shell process.

`PLAN-11` closes that gap with the smallest complete vertical slice:

1. extend `ExecuteRequest` with one typed member-dispatch payload,
2. reuse `POST /v1/execute/stream` and `POST /v1/execute/cancel` as the only transport seam,
3. add one Linux-first internal member-runtime manager in `world-agent`,
4. switch the shell member path from local `run_control(...)` to remote retained control,
5. keep canonical session-root state authority in the shell,
6. prove placement, cancellation, replacement, status, and trace identity with real tests.

The user outcome is simple:

- when a world-scoped member says it is live on generation `N` of world `W`,
- it is actually running inside `world-agent` on generation `N` of world `W`,
- not just correlated to that world in metadata.

## Why This Slice Exists

The repo already has most of the hard state work:

- shell-owned orchestration session authority is real,
- world binding and generation plumbing are real,
- stale-generation invalidation is real,
- replacement lineage is real,
- status and trace projection already consume real participant records.

The remaining lie is narrow but important:

- `prepare_member_runtime_startup_for_descriptor(...)` and
  `ensure_member_runtime_ready(...)` already decide *which* member should exist,
- but `start_member_runtime_with_prepared(...)` still reuses the local host-orchestrator launch
  path,
- which means the member is only *described* as world-scoped,
- not actually *executed* in-world.

This plan fixes placement without reopening:

- member selection,
- replacement ordering,
- world-generation authority,
- session-root state ownership,
- or a new public `/v1/member/*` API family.

## Step 0: Scope Challenge

### 0A. Repo truth and why this slice exists

The repo already proves the prerequisites this slice needs:

1. `prepare_member_runtime_startup_for_descriptor(...)`,
   `ensure_member_runtime_ready(...)`, and
   `reconcile_member_runtime_generation(...)` in
   [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
   already select the member, validate the parent session, bind it to the authoritative world,
   and preserve replacement lineage.
2. `AgentRuntimeParticipantRecord::new_member_participant(...)`,
   `new_replacement_participant(...)`,
   `can_advertise_live()`, and
   `is_authoritative_live()` in
   [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
   already encode the correct liveness bar.
3. `persist_participant(...)`,
   `resolve_live_orchestrator_participant(...)`,
   `list_live_participants_for_session(...)`, and
   `invalidate_stale_world_members_for_session(...)` in
   [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
   already give this slice persistence and invalidation truth.
4. `build_agent_client_and_request_with_trace_metadata(...)`,
   `stream_non_pty_via_agent(...)`, and
   `process_agent_stream_body(...)` in
   [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
   already prove the repo has one real host<->world execute-stream transport.
5. `WorldAgentService::execute_stream(...)` and `execute_cancel(...)` in
   [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
   already own the Linux-first NDJSON streaming and cancel loop.
6. `GatewayRuntimeManager` in
   [gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs)
   already demonstrates the rigor level the in-world manager should match for start, ready,
   cancel, and cleanup.
7. the shell and world-agent test harnesses already have the right leverage:
   - recorded `/v1/execute/stream` payloads in
     [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs)
   - persistent-session world-agent stubs in
     [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
   - streamed execute cancel coverage in
     [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)

What is still missing:

1. `ExecuteRequest` cannot describe typed member dispatch today.
2. `world-agent` cannot start and retain a pure-agent member runtime today.
3. `/v1/execute/cancel` only knows about process exec spans today.
4. the shell member path still shares the local host-orchestrator startup function.
5. existing tests prove state semantics, but not actual world placement semantics.

### 0B. Accepted premises and rejected shortcuts

| Premise | Decision | Why |
| --- | --- | --- |
| Reuse existing execute-stream and execute-cancel transport | Accepted | This is the boring path. Routing, NDJSON streaming, and cancel semantics already exist. |
| Keep canonical session-root state authority in the shell | Accepted | Moving writes into `world-agent` would reopen [PLAN-09.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-09.md). |
| Use typed transport data, not a magic command string | Accepted | `cmd="__member_dispatch__ ..."` is clever in the bad way. |
| Keep v1 to one shell-owned orchestrator plus one active world-scoped member | Accepted | This slice is about placement honesty, not scheduler design. |
| Add a new public `/v1/member/*` or `/v1/agents/*` family now | Rejected | That spends an innovation token on packaging before the control path is even real. |
| Allow host fallback if remote member dispatch fails | Rejected | That would make status and trace lie in the exact failure mode this slice exists to close. |

Hard posture:

- the remote path fails closed,
- and a world-scoped member never silently falls back to local launch.

### 0C. What already exists

| Sub-problem | Existing code | Plan |
| --- | --- | --- |
| unique live parent lookup | `resolve_live_orchestrator_participant(...)` in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse directly |
| member descriptor selection | `select_member_runtime_descriptor(...)` + `validate_member_selection(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) and [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) | Reuse |
| authoritative world binding validation | `authoritative_member_world_binding(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse |
| member participant construction | `new_member_participant(...)` and `new_replacement_participant(...)` in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) | Reuse, no schema redesign |
| retained-control liveness rules | `mark_runtime_ownership_retained()`, `can_advertise_live()`, `is_authoritative_live()` in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) | Reuse exactly |
| store-owned persistence | `persist_participant(...)` and `persist_orchestration_session(...)` in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse exactly |
| stale-generation invalidation | `invalidate_stale_world_members_for_session(...)` in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse exactly |
| command transport client creation | `build_agent_client_and_request_with_trace_metadata(...)` in [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs) | Reuse pattern, add member-dispatch builder |
| Linux execute-stream service | `execute_stream(...)` and `execute_cancel(...)` in [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs) | Reuse as outer transport shell |
| in-world lifecycle rigor | `GatewayRuntimeManager` in [gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs) | Reuse as implementation pattern, not as the member runtime itself |
| operator status projection | `build_status_report(...)` and `build_toolbox_status_report(...)` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | Consume unchanged unless the real transport path proves drift |

### 0D. NOT in scope

- a new public `/v1/member/*` or `/v1/agents/*` API family
- broad `substrate agent start|resume|fork|stop` productization
- host-side fallback for world-scoped member launch
- changing the authoritative session-root store contract from [PLAN-09.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-09.md)
- changing the member-selection rule from [PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-10.md) except where typed transport serialization needs explicit fields
- moving the host orchestrator into `world-agent`
- auth-bundle carrier replacement for gateway secrets
- macOS and Windows parity beyond explicit fail-closed posture
- UI changes

### 0E. Complexity, completeness, and distribution checks

The raw file count will exceed 8. That is expected here.

This seam spans:

1. transport types,
2. shell request construction,
3. shell retained-control ownership,
4. world-agent runtime ownership,
5. contract and integration tests.

That does **not** mean the plan is overbuilt.

The minimal production logic surface is still tight:

1. [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
2. [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
3. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
4. [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
5. one new internal manager module under `crates/world-agent/src/`

Everything else is test or glue.

Completeness check:

- the complete version moves launch, cancel, replacement, and trace identity together,
- the shortcut moves launch only and leaves cancel or terminal convergence lying,
- with AI-assisted implementation, that shortcut saves almost nothing and keeps the dangerous
  footguns alive.

Distribution check:

- no new binary, package, container image, or installer surface is introduced,
- distribution work is not applicable for this slice.

## Architecture Contract

### Hard invariants

1. **Shell authority remains authoritative.**
   - `world-agent` owns in-world member execution and remote control retention.
   - the shell owns canonical `AgentRuntimeStateStore` writes and the status surface.

2. **No host-side fallback for world-scoped members.**
   - once `execution.scope=world` is selected, failure is explicit failure, not local launch.

3. **The active world binding remains authoritative.**
   - the shell dispatches the member against the already-authoritative `world_id` and
     `world_generation`,
   - and `world-agent` must reject a mismatch instead of normalizing it.

4. **The existing transport endpoints remain the seam.**
   - `POST /v1/execute/stream` for launch plus long-lived control streaming,
   - `POST /v1/execute/cancel` for cancellation,
   - existing `ExecuteStreamFrame::{Start,Event,Exit,Error}` families only.

5. **Readiness stays strict.**
   - a member may be persisted as `Allocating`,
   - but it may advertise live only after a real remote session handle exists, cancel ownership is
     retained, event streaming is active, completion observation is retained, and the shell has
     persisted the updated snapshot.

6. **Linux-first is explicit.**
   - non-Linux platforms remain unavailable for this slice and must fail closed with an explicit
     error, not a compatibility fallback.

### V1 request contract

Add one additive field to `ExecuteRequest` in
[crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs):

```text
ExecuteRequest
    ├── profile
    ├── cmd
    ├── cwd
    ├── env
    ├── pty
    ├── agent_id
    ├── budget
    ├── policy_snapshot
    ├── shared_world
    ├── world_network
    ├── world_fs_mode
    └── member_dispatch: Option<MemberDispatchRequestV1>
```

`MemberDispatchRequestV1` should be explicit, boring, and shell-authoritative:

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
    └── world_generation
```

Rules:

1. regular process exec is:
   - `member_dispatch=None`
   - `cmd.trim().is_empty()==false`
2. typed member dispatch is:
   - `member_dispatch=Some(...)`
   - `cmd.trim().is_empty()==true`
3. top-level `agent_id` remains authoritative for budgets, traces, and diagnostics, so
   `MemberDispatchRequestV1` does **not** duplicate `agent_id`.
4. `pty=true` is invalid for member dispatch.
5. no magic command strings.

Implementation rule:

- validate the contract at the type boundary, not ad hoc in the handler.
- mirror the existing `GatewayLifecycleRequestV1` pattern in `agent-api-types`:
  introduce an internal `ExecuteRequestDef` plus `TryFrom<ExecuteRequestDef>` validation so the
  mutual-exclusion rules fail during deserialize/parse, not deep inside `service.rs`.

### Stream and event contract

Do **not** invent a new stream frame family.

Use the existing `ExecuteStreamFrame` contract:

- `Start { span_id }` announces the cancellable remote control span.
- `Event { event }` carries member lifecycle events and status messages.
- `Exit { ... }` marks normal terminal completion.
- `Error { message }` marks startup or transport failure.

Required event rules:

1. the event that first exposes a real session handle is what allows the shell to call
   `set_uaa_session_id(...)`,
2. the shell flips `Allocating -> Ready` only after that session-handle event arrives **and**
   retained remote ownership is established,
3. subsequent lifecycle messages may move `Ready -> Running`,
4. stream loss before readiness becomes `Failed`,
5. stream loss after authoritative live ownership becomes `Invalidated`,
6. terminal success or explicit cancel becomes `Stopped`,
7. all forwarded `AgentEvent` rows must keep top-level participant lineage and world identity.

### Retained-control ownership model

This is the biggest structural clarification missing from the previous draft.

The shell cannot pretend a remote member runtime is the same thing as the local
`RetainedRunControl` carrier.

`AsyncReplAgentRuntime` currently stores a local UAA cancel handle plus event and completion
tasks. Remote member dispatch does not have that shape. It has:

- a remote `span_id`,
- a remote cancel path through `POST /v1/execute/cancel`,
- a long-lived stream task,
- and shell-owned manifest/state transitions driven by remote events.

Required design:

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

Rules:

1. keep the manifest, store, heartbeat, and liveness rules shared above this abstraction,
2. do not overload `RetainedRunControl` with fake remote values,
3. do not fork the state machine just because the cancel mechanism differs,
4. prefer one small enum over a trait hierarchy.

### Prepared-launch split

This is the second structural clarification missing from the previous draft.

`PreparedAgentRuntime` is local-gateway shaped today:

- it contains a gateway,
- it contains an agent kind resolved for local `run_control(...)`,
- and it assumes the shell will own the control turn.

Remote member dispatch does not need that.

Required design:

```text
PreparedHostRuntime
    ├── descriptor
    ├── gateway
    ├── agent_kind
    ├── startup_context
    ├── manifest
    └── run_id

PreparedMemberDispatch
    ├── descriptor
    ├── startup_context
    ├── manifest
    ├── run_id
    └── authoritative world binding metadata
```

Recommendation:

- introduce a dedicated `PreparedMemberDispatch` instead of stuffing `Option<gateway>` and
  `Option<agent_kind>` into the existing struct.

Why:

- explicit over clever,
- smaller blast radius than a generic "maybe local, maybe remote" mega-struct,
- easier for a tired engineer to read at 3am.

### Launch flow

```text
shell REPL
    │
    ├── ensure_member_runtime_ready()
    │       │
    │       ├── validate member descriptor
    │       ├── resolve live orchestrator parent
    │       ├── verify authoritative world binding
    │       ├── persist member participant as Allocating
    │       └── open /v1/execute/stream with member_dispatch payload
    │
    ▼
world-agent execute_stream(member_dispatch)
    │
    ├── validate typed payload
    ├── ensure active session world matches world_id/world_generation
    ├── build backend in-world
    ├── start run_control(...)
    ├── retain cancel + event + completion ownership
    ├── emit Start / Event / Exit / Error frames
    └── register span_id for /v1/execute/cancel
    │
    ▼
shell retained-control consumer
    │
    ├── store remote span_id in RemoteMemberControl
    ├── surface real UAA session handle
    ├── mark runtime ownership retained
    ├── transition Allocating -> Ready -> Running
    ├── persist canonical participant snapshots
    └── publish canonical AgentEvent rows
```

### Lifecycle state machine

```text
Allocating
    │
    ├── typed dispatch accepted, remote control retained, session handle surfaced
    ▼
Ready
    │
    ├── first steady-state lifecycle event / active control loop
    ▼
Running
    │
    ├── clean cancel / clean completion      -> Stopped
    ├── bootstrap failure before readiness   -> Failed
    ├── stream closes before readiness       -> Failed
    ├── stream closes after readiness        -> Invalidated
    └── world generation rollover            -> Invalidated, then replacement Allocating
```

### Dependency graph

```text
agent-api-types::ExecuteRequest
        │
        ├── shell/world_ops member-dispatch request builder
        │       │
        │       └── shell/async_repl remote member startup + replacement
        │
        └── world-agent/service execute_stream branch
                │
                └── world-agent/member_runtime manager
                        │
                        ├── UAA backend builder
                        ├── retained control ownership
                        ├── event forwarding
                        └── cancel registry integration
```

## File Plan

### 1. `crates/agent-api-types/src/lib.rs`

Deliver:

- `MemberDispatchRequestV1`
- additive `ExecuteRequest.member_dispatch`
- request validation via typed deserialize/parse rules
- round-trip and invalid-shape tests for the new contract

Do not:

- convert the whole transport family into a new target-enum abstraction in this slice.

### 2. `crates/shell/src/execution/routing/dispatch/world_ops.rs`

Deliver:

- a dedicated member-dispatch request builder that reuses existing transport/client creation,
- request-level trace metadata and policy/world inputs,
- additive NDJSON decode helpers only if `async_repl.rs` needs shared parsing support.

Do not:

- move participant persistence or liveness state transitions into this file.

This file stays transport-focused.

### 3. `crates/shell/src/repl/async_repl.rs`

Deliver:

- a dedicated `PreparedMemberDispatch` path,
- a transport-agnostic retained-control carrier (`LocalUaa` vs `RemoteMember`),
- member participant persistence in `Allocating` before remote launch,
- remote stream consumption that drives the existing liveness rules,
- shutdown and replacement logic that can cancel a remote span and converge honestly,
- host orchestrator startup unchanged.

Do not:

- fork the lifecycle state machine into a second member-only live model.

### 4. `crates/world-agent/src/service.rs`

Deliver:

- branch `execute_stream(...)` on `member_dispatch`,
- keep ordinary process exec exactly as before,
- route member dispatch into the new internal manager,
- extend `execute_cancel(...)` so a span can address either process exec or member dispatch.

Do not:

- jam the whole member-runtime lifecycle into `execute_stream(...)`.

This file is the branch point, not the framework.

### 5. `crates/world-agent/src/member_runtime.rs`

New internal module.

Deliver:

- Linux-first member runtime manager,
- start-context validation against world binding,
- `run_control(...)` startup in-world,
- cancel/event/completion retention,
- `ExecuteStreamFrame` emission,
- cleanup on completion, cancel, and abnormal stream loss.

Use `GatewayRuntimeManager` as the rigor reference, not as the implementation target.

### 6. `crates/shell/tests/support/socket.rs`

Deliver:

- additive request-stub support for `member_dispatch`,
- assertions that recorded `/v1/execute/stream` payloads include the typed member contract.

### 7. `crates/shell/tests/support/repl_world_agent.rs`

Deliver:

- additive capture of typed member-dispatch payloads and lineage fields,
- optional canned lifecycle scripting for ready, cancel, terminal success, and failure cases.

### 8. Primary test owners

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)

### Validation-first no-change gates

#### `crates/shell/src/execution/agent_runtime/session.rs`

Default: no schema redesign.

Only touch if the remote path truly needs one additive ownership marker. The existing liveness and
lineage model is already correct.

#### `crates/shell/src/execution/agent_runtime/state_store.rs`

Default: no new semantics.

The store already owns persistence and invalidation truth. This plan consumes that contract.

#### `crates/shell/src/execution/agents_cmd.rs`

Default: no logic change.

Touch only if a real launched member exposes status or doctor drift that existing tests do not
already pin.

#### `crates/world-agent/src/gateway_runtime.rs`

No logic change.

This is the rigor reference, not the implementation target for world members.

## Implementation Sequence

### Step 1. Freeze the typed transport contract

Deliverables:

1. additive `MemberDispatchRequestV1`,
2. additive `ExecuteRequest.member_dispatch`,
3. typed request validation,
4. round-trip and invalid-shape tests,
5. no behavior change for ordinary process execution callers.

Acceptance gate:

- all existing transport callers still compile unchanged,
- regular process exec still deserializes and executes unchanged,
- invalid `cmd` plus `member_dispatch` combinations fail at the request boundary.

### Step 2. Teach the shell stubs and request builders the new contract

Deliverables:

1. `socket.rs` can deserialize and record `member_dispatch`,
2. `repl_world_agent.rs` can capture and script typed member lifecycle responses,
3. `world_ops.rs` can build a typed member-dispatch request over existing transport plumbing.

Acceptance gate:

- tests can assert payload contents before the runtime switch lands,
- and the shell-side transport builder is pinned before lifecycle work begins.

### Step 3. Add the Linux-first world-agent member runtime manager

Deliverables:

1. new internal `member_runtime.rs`,
2. start/ready/event/completion ownership in-world,
3. span registration for cancel delivery,
4. explicit world-binding validation,
5. startup, cancel, and abnormal-termination tests at the world-agent layer.

Acceptance gate:

- world-agent tests can start and cancel a typed member dispatch without involving shell REPL
  logic yet.

### Step 4. Cut the shell member path off the local gateway path

Deliverables:

1. `PreparedMemberDispatch` or an equivalently explicit remote-prepared shape,
2. `RetainedRuntimeControl::{LocalUaa,RemoteMember}` or an equivalently explicit ownership split,
3. `start_member_runtime_with_prepared(...)` becomes the remote member transport path,
4. `ensure_member_runtime_ready(...)` and replacement startup consume the remote path,
5. host orchestrator startup remains unchanged.

Acceptance gate:

- first world-backed command launches the member through `/v1/execute/stream`,
- same-generation reuse still works,
- missing parent/binding/selection still fail before remote launch.

### Step 5. Land cancel, replacement, status, and trace regression closure

Deliverables:

1. `/v1/execute/cancel` reaches world-member spans,
2. shutdown and restart paths converge member state to stopped, failed, or invalidated honestly,
3. replacement launch on the new generation keeps fresh `participant_id` and correct
   `resumed_from_participant_id`,
4. status and trace stay producer-backed and participant-correct.

Acceptance gate:

- restart with a live member yields either a live replacement or honest absence,
- stale generation never regains liveness,
- trace rows come from the real remote producer, not from fixture-only paths.

### Step 6. Only then touch docs if wording changed

This is not a docs-first slice.

Touch docs only if implementation changes operator-visible transport wording or trace examples.

## Test Review

### Test framework detection

- Runtime: Rust
- Framework: `cargo test`
- Primary packages: `agent-api-types`, `world-agent`, `shell`
- No LLM prompt or eval suite expansion is required for this slice

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/agent-api-types/src/lib.rs
    │
    ├── ExecuteRequest.member_dispatch
    │   ├── [GAP]         Regular process exec request still validates
    │   ├── [GAP]         Member-dispatch request validates with full identity tuple
    │   ├── [GAP]         Empty cmd + member_dispatch is accepted only for typed dispatch
    │   └── [GAP]         Process exec + member_dispatch together fails closed
    │
    └── ExecuteStreamFrame compatibility
        └── [★★★ TESTED] Existing Start/Event/Exit/Error families already round-trip

[+] crates/world-agent/src/service.rs + member_runtime.rs
    │
    ├── execute_stream()
    │   ├── [★★★ TESTED] Regular streamed process exec path already works
    │   ├── [GAP]         Branches typed member dispatch away from process exec
    │   ├── [GAP]         Rejects mismatched/non-authoritative world binding
    │   └── [GAP]         Registers member span for cancel delivery
    │
    ├── execute_cancel()
    │   ├── [★★★ TESTED] Process-exec cancel path already works
    │   └── [GAP]         Member-dispatch cancel reaches retained control and converges terminally
    │
    └── member runtime lifecycle
        ├── [GAP]         In-world backend start emits Start + session-bearing Event
        ├── [GAP]         Bootstrap failure emits Error and cleanup
        ├── [GAP]         Stream closes before readiness -> failed
        └── [GAP]         Stream closes after readiness -> invalidated

[+] crates/shell/src/execution/routing/dispatch/world_ops.rs
    │
    ├── build_member_dispatch_request(...)
    │   ├── [GAP]         Reuses policy/world routing inputs
    │   ├── [GAP]         Carries participant/lineage/world fields correctly
    │   └── [GAP]         Preserves trace parent metadata
    │
    └── decode support
        └── [GAP]         Any shared frame parsing stays transport-only, not lifecycle-owning

[+] crates/shell/src/repl/async_repl.rs
    │
    ├── PreparedMemberDispatch / remote startup path
    │   ├── [GAP] [->E2E] Uses remote member-dispatch transport, not local gateway reuse
    │   ├── [GAP]         Persists Allocating before remote ownership
    │   ├── [GAP]         Ready -> Running only after retained ownership is proven
    │   └── [GAP]         Bootstrap failure marks participant failed
    │
    ├── retained control abstraction
    │   ├── [GAP]         Remote member cancel uses execute-cancel span path
    │   └── [GAP]         Shutdown path converges non-live if cancel delivery fails
    │
    ├── ensure_member_runtime_ready()
    │   ├── [GAP] [->E2E] First world-backed command launches member lazily over transport
    │   ├── [GAP]         Same-generation command reuses the live remote member
    │   └── [GAP]         Missing parent/binding fails before transport call
    │
    └── reconcile_member_runtime_generation()
        ├── [★★  TESTED] Stale-generation invalidation already exists
        ├── [GAP] [->E2E] Replacement launch crosses transport on the new generation
        └── [GAP] [->E2E] Replacement failure leaves honest absence

[+] shell integration and contract tests
    │
    ├── repl_world_first_routing_v1.rs
    │   ├── [★★  TESTED] Lazy member launch and replacement behavior already exist at the shell-state level
    │   └── [GAP] [->E2E] Recorded execute-stream payload proves actual member dispatch transport
    │
    ├── agent_successor_contract_ahcsitc0.rs
    │   ├── [★★★ TESTED] Status surfaces already keep member world fields top-level
    │   └── [GAP]         Real remote member launch remains status-correct after cancel/restart
    │
    └── agent_hub_trace_persistence.rs
        ├── [★★★ TESTED] Replacement lineage and world fields already persist in trace rows
        └── [GAP]         Remote member lifecycle producer emits those rows, not just fixtures

─────────────────────────────────
COVERAGE: 6/25 paths tested (~24%)
  Code paths: 5/20
  Integration/user flows: 1/5
QUALITY:  ★★★: 4  ★★: 2  ★: 0
GAPS: 19 paths need tests (5 need E2E/integration)
─────────────────────────────────
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] First world-backed command in REPL
    ├── [GAP] [->E2E] Shell persists Allocating member, then launches it through execute-stream
    ├── [GAP]         Second same-generation command reuses the live member
    └── [GAP]         Missing authoritative world binding fails before transport call

[+] World-member cancellation and shutdown
    ├── [GAP] [->E2E] Ctrl-C or shutdown calls /v1/execute/cancel for member span
    ├── [GAP]         Cancel delivery failure surfaces non-live terminal outcome
    └── [GAP]         Clean stop lands in Stopped, not Invalidated

[+] Shared-world restart with live member
    ├── [★★  TESTED] Shell invalidation semantics already exist
    ├── [GAP] [->E2E] Replacement launch crosses transport on the new generation
    └── [GAP] [->E2E] Replacement startup failure leaves honest absence

[+] Operator inspection
    ├── [GAP]         `substrate agent status --json` shows the remote member from runtime state
    ├── [GAP]         terminal trace rows stay auditable without reviving stale liveness
    └── [GAP]         doctor/toolbox remain honest and orchestrator-anchored
```

### Required tests to add to the plan

1. **`crates/agent-api-types/src/lib.rs`**
   - add round-trip and validation tests for:
     - well-formed `member_dispatch`
     - empty `cmd` accepted only with `member_dispatch`
     - missing required fields
     - illegal combination of process exec + member dispatch

2. **`crates/world-agent/tests/streamed_execute_cancel_v1.rs` or focused world-agent tests**
   - add Linux-first tests for:
     - successful member-dispatch startup
     - member-dispatch cancel delivery
     - bootstrap failure before readiness
     - abnormal post-ready termination cleanup

3. **`crates/shell/src/repl/async_repl.rs`**
   - add runtime tests for:
     - remote member startup uses transport instead of local `gateway.run_control(...)`
     - `Allocating -> Ready -> Running` progression only after remote ownership is retained
     - missing parent/binding/selection fail before launch
     - replacement launch preserves `resumed_from_participant_id`
     - remote cancel path converges the member non-live

4. **`crates/shell/tests/repl_world_first_routing_v1.rs`**
   - add integration cases for:
     - first world-backed command launches member through typed execute-stream payload
     - restart replacement uses typed execute-stream payload on the new generation
     - replacement failure leaves no authoritative-live member

5. **`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`**
   - add contract cases for:
     - status remains correct for a real remotely launched member
     - cancel and restart terminal states stay non-live
     - toolbox remains anchored to the orchestrator session

6. **`crates/shell/tests/agent_hub_trace_persistence.rs`**
   - add producer-backed cases for:
     - remote member lifecycle event emission
     - replacement lineage staying top-level in trace rows

7. **`crates/shell/tests/support/socket.rs` and `crates/shell/tests/support/repl_world_agent.rs`**
   - add harness assertions that typed member-dispatch payloads are actually recorded and
     inspectable.

### Test artifact

The eng-review QA artifact for this plan is:

[spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260502-140843.md](/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260502-140843.md)

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
| --- | --- | --- | --- | --- | --- |
| typed request construction | shell sends malformed lineage/world metadata and world-agent guesses | no | not yet | no | yes until request-boundary validation lands |
| shell preflight | missing authoritative world binding still emits a remote launch attempt | no | partial | partial | yes until preflight-fails-before-transport coverage lands |
| remote ownership carrier | shutdown path treats a remote member like a local cancel handle and leaves state live | no | not yet | no | yes until `RemoteMemberControl` convergence tests land |
| world-agent startup | backend cannot start in-world but participant becomes live anyway | no | not yet | no | yes until bootstrap-failure coverage lands |
| remote readiness | session handle never surfaces but shell transitions Ready | no | partial | no | yes until Ready-gating tests land |
| cancel delivery | `/v1/execute/cancel` misses the member span and shell keeps advertising live | no | partial | partial | yes until cancel convergence coverage lands |
| replacement launch | stale member invalidates, replacement fails, stale trace row looks live again | partial | partial | no | yes until replacement + trace suppression tests land |
| status and trace projection | remote producer emits wrong participant/world identity | no | partial | yes | yes until producer-backed status/trace tests land |

Critical gap rule:

If this slice can launch a member locally on fallback, advertise live before remote ownership is
retained, or leave stale liveness after cancel or restart failure, the implementation is not
done.

## Performance Review

This is correctness-first. The performance risk is architectural sprawl, not CPU.

Performance rules:

1. do not build a second client or transport stack for member dispatch,
2. do not re-run remote launch if a matching authoritative-live member already exists for the
   current generation,
3. do not add extra full-store scans to steady-state command execution if the current REPL already
   owns the live member handle,
4. do not make `execute_cancel` poll forever waiting for delivery,
5. do not invent a scheduler or cache to "optimize" a one-member placement seam.

Performance verdict:

- 0 throughput blockers,
- 1 structural caution: keep cancel delivery bounded and reuse the existing span-registry pattern.

## DX Guardrails

This slice has no UI scope. It has strong developer and operator scope.

Required DX posture:

- typed member transport must be visible in tests and diagnostics,
- every remote failure message must follow problem + cause + fix wording,
- `status` and `toolbox` must keep consuming shell authority instead of remote guesses,
- top-level `world_id` and `world_generation` must stay present in trace rows,
- docs only change if operator-visible wording changes,
- proving whether world-member placement is real should drop from roughly 15 minutes of code
  reading to under 8 minutes of plan + test inspection.

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| A. Transport contract freeze | `crates/agent-api-types/`, `crates/shell/tests/support/` | — |
| B. World-agent member manager | `crates/world-agent/src/`, `crates/world-agent/tests/` | A |
| C. Shell remote runtime carrier and launch switch | `crates/shell/src/repl/`, `crates/shell/src/execution/routing/dispatch/`, `crates/shell/tests/` | A |
| D. Status, trace, and replacement regression wall | `crates/shell/tests/`, maybe `crates/shell/src/execution/` | B + C |

### Parallel lanes

- Lane A: transport contract and stub capture support
- Lane B: world-agent member manager after Lane A
- Lane C: shell runtime-carrier split and remote launch cutover after Lane A
- Lane D: integration, status, and trace regression wall after B + C

### Execution order

1. Launch Lane A first and freeze the typed request contract.
2. After A merges cleanly, run Lane B and Lane C in parallel worktrees.
3. Merge B + C.
4. Run Lane D last because it verifies the full end-to-end seam.

### Conflict flags

- Lanes B and C are safe in parallel only if both avoid reopening `agent-api-types` after Lane A
  lands.
- Lane C and Lane D both touch shell test surfaces. Keep D last.
- If B needs to rename any event payload fields after C starts, stop and refreeze the contract.

### Parallelization verdict

Three execution phases, with one real parallel window:

- sequential contract freeze,
- parallel runtime implementation,
- sequential regression closure.

## Deferred Work

1. Replace additive `member_dispatch` with a tagged execute-target enum if more transport variants
   actually appear.
2. Extract shared backend-building helpers between shell and `world-agent` only if duplication
   becomes real and measured.
3. Land secret-safe auth-bundle carrier handoff into the world transport.
4. Add macOS and Windows parity for in-world member dispatch.
5. Consider public control-plane UX only after the transport seam is already boring in production.

## Definition of Done

This slice is done only when all of the following are true:

1. a world-scoped member launch uses typed `/v1/execute/stream` transport, not local
   shell-owned `gateway.run_control(...)`,
2. the shell persists the member as `Allocating` first and only advertises live after remote
   ownership is retained,
3. remote retained control is represented explicitly in the shell, not by pretending it is a
   local UAA cancel handle,
4. `/v1/execute/cancel` can cancel a live member-dispatch span,
5. abnormal stream loss or startup failure converges the member to a non-live terminal state,
6. replacement launch after world-generation rollover crosses the same transport and preserves
   `resumed_from_participant_id`,
7. `substrate agent status --json` and trace rows remain participant-correct and world-correct for
   the real remote producer,
8. no host fallback exists for world-scoped members,
9. targeted validation commands below are green,
10. the plan remains Linux-first and explicitly fail-closed elsewhere.

## Recommended verification commands

```bash
cargo test -p agent-api-types -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_hub_trace_persistence -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
```

## Completion Summary

- Step 0: scope accepted as transport-placement slice, no new public API family
- Architecture: tightened around one typed request, one remote control abstraction, and one
  explicit prepared-launch split
- Code Quality: no second lifecycle state machine, no fake remote-as-local carrier, no transport
  brain in `world_ops.rs`
- Test Review: diagrams produced, 19 direct gaps identified
- Performance Review: 1 structural caution, 0 throughput blockers
- DX Guardrails: explicit and fail-closed, TTHW target under 8 minutes
- NOT in scope: written
- What already exists: written
- Failure modes: 8 critical gaps flagged for implementation to close through tests and fail-closed
  behavior
- Outside voice: skipped, `claude` CLI is installed but unauthenticated on 2026-05-02
- Parallelization: 4 steps, 1 real parallel window after the transport contract lands
- Lake Score: complete option chosen for every in-slice decision

<!-- AUTONOMOUS DECISION LOG -->
## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Scope | Keep `/v1/execute/stream` and `/v1/execute/cancel` as the only transport seam | Mechanical | Boring by default | Existing transport already solves routing, NDJSON streaming, and cancel delivery | New `/v1/member/*` family |
| 2 | Transport | Use additive `ExecuteRequest.member_dispatch` now | Taste | Explicit over clever | Smallest complete change that keeps old callers stable and makes the new intent readable | Immediate tagged target-enum refactor |
| 3 | Validation | Validate member-dispatch exclusivity at the request type boundary | Mechanical | Systems over heroes | Failing at deserialize/parse is more reliable than hoping every handler remembers the rules | Ad hoc handler-only validation |
| 4 | Authority | Keep canonical session-root writes in the shell only | Mechanical | Systems over heroes | `world-agent` execution and shell authority are intentionally different jobs | Remote canonical-state writes |
| 5 | Ownership carrier | Split local and remote retained control with one small enum | Mechanical | Explicit over clever | Remote execute spans are not local UAA cancel handles, and pretending otherwise is a maintenance trap | Stuffing remote state into `RetainedRunControl` |
| 6 | Prepared launch | Introduce a dedicated remote-prepared shape for member dispatch | Taste | Minimal diff | A separate remote-prepared struct is clearer than optional gateway fields everywhere | One mega-struct with optional local-only fields |
| 7 | Failure posture | Deny world-member launch rather than falling back locally | Mechanical | Completeness | A fallback would make status and trace lie about placement | Host-side fallback |
| 8 | World-agent structure | Add one internal `member_runtime.rs` instead of bloating `service.rs` | Mechanical | Engineered enough | `service.rs` should branch, not become the lifecycle implementation | 500-line `execute_stream` branch |
| 9 | Reuse | Preserve existing liveness and lineage helpers in `session.rs` | Mechanical | DRY | The model already encodes the correct live bar and replacement semantics | Parallel remote-only state model |
| 10 | Rollout | Land transport contract first, then runtime switch, then regression wall | Mechanical | Incremental over revolutionary | This keeps contract churn separate from lifecycle churn | Big-bang multi-crate rewrite |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
| --- | --- | --- | --- | --- | --- |
| CEO Review | `/plan-ceo-review` | Scope and strategy | 1 | CLEAR | Kept the slice narrow, rejected a new public API family, and froze the host-authority/world-execution split |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | SKIPPED | No separate outside-model review run, and `claude` CLI auth is missing for outside voice on this machine |
| Eng Review | `/plan-eng-review` | Architecture and tests (required) | 1 | CLEAR | Locked the typed transport contract, the explicit local-vs-remote control split, the fail-closed placement posture, and the full test wall for placement/cancel/replacement correctness |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | SKIPPED | No UI scope |

**UNRESOLVED:** 0 plan-level decision points remain. The remaining work is implementation of the
typed transport seam and the regression wall already listed.

**VERDICT:** CEO + ENG CLEARED. `PLAN-11` is ready to execute as the transport-backed placement
cutover that makes world-scoped member runtime truth match the canonical session model.
