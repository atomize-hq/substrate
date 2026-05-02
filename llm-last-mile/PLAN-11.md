<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/feat-session-centric-state-store-autoplan-restore-20260502-140843.md -->

# PLAN-11: In-World Member Dispatch Over Existing Host<->World Transport

Source file: [11-in-world-member-dispatch-over-existing-host-world-transport.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/11-in-world-member-dispatch-over-existing-host-world-transport.md)  
Branch: `feat/session-centric-state-store`  
Plan type: host<->world transport placement seam, no UI scope, strong DX scope  
Review posture: `/autoplan`-style scope tightening with `/plan-eng-review` structure and rigor  
Status: execution-ready after [PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-10.md), with outside voice skipped on 2026-05-02 because `claude` CLI auth is missing

## Objective

This slice is not a second agent hub.

It is the honesty pass on member placement.

After [PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-10.md),
the shell can already:

- select one world-scoped member backend,
- persist canonical parent and member participants,
- invalidate stale generations,
- create replacement lineage,
- and project that state through `substrate agent status`.

What it still does not do is actually run the member inside the world boundary it claims to be
bound to.

Today `start_member_runtime_with_prepared(...)` in
[crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
simply reuses `start_host_orchestrator_runtime_with_prepared(...)`, which means the member
ultimately still reaches `gateway.run_control(...)` inside the shell process.

`PLAN-11` fixes that with the smallest complete vertical slice:

1. extend the existing `ExecuteRequest` shape with one typed member-dispatch payload,
2. reuse `POST /v1/execute/stream` and `POST /v1/execute/cancel` as the only transport seam,
3. add one Linux-first internal member-runtime manager in `world-agent`,
4. switch the shell member runtime path from local `run_control(...)` to remote retained control,
5. keep canonical session-root state ownership in the shell,
6. prove placement, cancellation, replacement, and trace identity with real tests.

The user outcome is simple:

- when a world-scoped member says it is live on generation `N` of world `W`,
- it is actually running inside `world-agent` on generation `N` of world `W`,
- not just correlated to that world in metadata.

## Step 0: Scope Challenge

### 0A. Repo truth and why this slice exists

The SOW is right about the missing seam.

The repo already proves all the setup around it:

1. `prepare_member_runtime_startup_for_descriptor(...)`,
   `ensure_member_runtime_ready(...)`, and
   `reconcile_member_runtime_generation(...)` in
   [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
   already select the member, validate the parent session, bind it to the authoritative world,
   and preserve replacement lineage.
2. `AgentRuntimeParticipantRecord::new_member_participant(...)`,
   `new_replacement_participant(...)`,
   `can_advertise_live()`, and
   `is_authoritative_live()` in
   [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
   already encode the correct liveness bar.
3. `persist_participant(...)`,
   `resolve_live_orchestrator_participant(...)`,
   `list_live_participants_for_session(...)`, and
   `invalidate_stale_world_members_for_session(...)` in
   [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
   already give this slice the persistence and invalidation truth it needs.
4. `build_agent_client_and_request_with_trace_metadata(...)`,
   `stream_non_pty_via_agent(...)`, and
   `process_agent_stream_body(...)` in
   [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
   already prove the repo has one real host<->world execute-stream transport.
5. `WorldAgentService::execute_stream(...)` and `execute_cancel(...)` in
   [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
   already own the Linux-first NDJSON streaming and cancel loop.
6. `GatewayRuntimeManager` in
   [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs)
   already demonstrates the level of rigor the in-world manager should match for start, ready,
   cancel, and cleanup.
7. the shell and world-agent test harnesses already have the right leverage:
   - recorded `/v1/execute/stream` payloads in
     [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs)
   - persistent-session world-agent stubs in
     [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
   - streamed execute cancel coverage in
     [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)

What is still missing:

1. `ExecuteRequest` can only describe process execution today.
2. `world-agent` cannot start and retain a pure-agent member runtime today.
3. `/v1/execute/cancel` only knows about process exec spans today.
4. the shell member path still shares the local host-orchestrator startup function.
5. existing tests prove state semantics, but not actual world placement semantics.

### 0B. Premise challenge

Premise check, one by one:

1. **The existing execute-stream transport should stay the seam.**
   - Accepted.
   - The repo already has a stable request/stream/cancel path. Reusing it is the boring choice.

2. **Shell authority over canonical session-root state must remain intact.**
   - Accepted.
   - Moving persistence authority into `world-agent` would reopen [PLAN-09.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-09.md).

3. **The world-member launch contract needs typed transport data, not a magic shell command.**
   - Accepted.
   - Encoding "launch member" as `cmd="__member_dispatch__ ..."` is clever in the bad way.

4. **The first slice should stay one shell-owned orchestrator plus one active world-scoped member.**
   - Accepted.
   - This plan is about placement honesty, not scheduler design.

5. **We should build a new public `/v1/member/*` or `/v1/agents/*` API family now.**
   - Rejected.
   - That spends an innovation token on packaging before the underlying control path is even real.

6. **Host fallback is acceptable if the remote path fails.**
   - Rejected.
   - That would make the status surface lie in the exact failure mode this slice exists to close.

Premise gate posture:

- accepted as-is for this plan,
- with one hard constraint: the remote path fails closed, or it does not ship.

### 0C. Existing code to reuse

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
| streamed frame consumption | `process_agent_stream_body(...)` in [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs) | Reuse, extend for member frames only if needed |
| Linux execute-stream service | `execute_stream(...)` and `execute_cancel(...)` in [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs) | Reuse as outer transport shell |
| in-world lifecycle rigor | `GatewayRuntimeManager` in [gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs) | Reuse as implementation pattern, not as the member runtime itself |
| operator status projection | `build_status_report(...)` and `build_toolbox_status_report(...)` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | Consume unchanged unless the real transport path proves drift |

### 0D. Dream state and 12-month ideal

```text
CURRENT REPO
    │
    ├── shell owns orchestration session state correctly
    ├── member participants carry correct world_id/world_generation metadata
    ├── execute-stream transport exists for world commands
    ├── world-agent can stream and cancel process exec spans
    └── member run_control still starts inside the shell process
            │
            ▼
THIS PLAN
    │
    ├── shell still owns canonical participant/session persistence
    ├── member startup request crosses /v1/execute/stream with typed payload
    ├── world-agent starts and retains member run_control in-world
    ├── cancel and terminal handling reuse /v1/execute/cancel
    └── trace/status/replacement semantics stay participant-correct
            │
            ▼
12-MONTH IDEAL
    │
    ├── multiple world members can be routed intentionally
    ├── public control-plane UX can be layered later if needed
    ├── host/orchestrator vs world/member placement stays explicit
    ├── auth-bundle handoff is secret-safe in-world
    └── platform parity follows from one already-honest transport contract
```

### 0E. Implementation alternatives

| Approach | Summary | Effort | Risk | Decision |
| --- | --- | --- | --- | --- |
| A. Add `member_dispatch` to `ExecuteRequest` and branch inside existing execute-stream service | Smallest complete slice, reuses transport and cancel family | Medium | Low | **Accepted** |
| B. Replace `ExecuteRequest` with a tagged transport-target enum now | Cleaner long-term, bigger immediate blast radius across callers and tests | Medium-Large | Medium | Deferred |
| C. Encode member launch as a fake `cmd` string | Small diff, terrible contract clarity, easy to break silently | Small | High | Rejected |
| D. Add a new `/v1/member/*` public API family | Productizes too early, duplicates routing/cancel/stream concerns | Large | High | Rejected |

### 0F. Complexity, search, completeness, and distribution checks

The complexity check triggers on raw file count. That is expected here.

This seam spans:

1. transport types,
2. shell request construction and retained-control plumbing,
3. world-agent runtime ownership,
4. integration stubs,
5. contract tests.

That does **not** mean the plan is overbuilt.

The minimal production logic surface is still tight:

1. [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
2. [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
3. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
4. [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
5. one new internal manager module under `crates/world-agent/src/`

Everything else is test or glue.

`[Layer 1]` wins:

- reuse `/v1/execute/stream`,
- reuse `/v1/execute/cancel`,
- reuse `ExecuteStreamFrame`,
- reuse shell store/state helpers,
- reuse world-agent cancel-delivery pattern,
- reuse test stubs that already record execute-stream JSON.

Completeness check:

- the complete version is to move launch, cancel, replacement, and trace identity together,
- the shortcut is to move launch only and leave cancel or replacement semantics lying,
- with AI-assisted implementation, the shortcut saves almost nothing and leaves the most dangerous
  footguns alive.

Distribution check:

- no new binary, package, container image, or installer surface is introduced,
- distribution work is not applicable.

### 0G. What already exists

1. shell-owned orchestration session truth already exists,
2. member participant lineage and world binding already exist,
3. restart invalidation and replacement semantics already exist in shell state,
4. execute-stream and execute-cancel transport already exist,
5. world-agent already supports streamed process execution and cancel,
6. world-agent already has a runtime-manager pattern worth copying,
7. shell and world-agent tests already have stream-recording harnesses,
8. the remaining gap is placement, not state semantics.

### 0H. NOT in scope

- a new public `/v1/member/*` or `/v1/agents/*` API family
- broad `substrate agent start|resume|fork|stop` productization
- host-side fallback for world-scoped member launch
- changing the authoritative session-root store contract from [PLAN-09.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-09.md)
- changing the member-selection rule from [PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-10.md) except where typed transport serialization requires explicit fields
- moving the host orchestrator into `world-agent`
- auth-bundle carrier replacement for gateway secrets
- macOS and Windows parity beyond explicit fail-closed posture
- UI changes

## Architecture Contract

### V1 transport contract

Add one additive field to `ExecuteRequest` in
[crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs):

```text
ExecuteRequest
    ├── existing process exec fields
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
    ├── agent_id
    ├── backend_id
    ├── protocol
    ├── run_id
    ├── world_id
    └── world_generation
```

Rules:

1. `cmd` stays present for regular process execution and is ignored for typed member dispatch.
2. `agent_id` at the request top level remains the selected member agent id so budgets, traces,
   and diagnostics stay agent-correct.
3. `member_dispatch` and regular process execution are mutually exclusive in validation.
4. no magic command strings.

### Shell-to-world launch flow

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
    ├── surface real UAA session handle
    ├── mark runtime ownership retained
    ├── transition Allocating -> Ready -> Running
    ├── persist canonical participant snapshots
    └── publish canonical AgentEvent rows
```

### Cancel and terminal handling

```text
shell shutdown / ctrl-c / restart invalidation
    │
    └── POST /v1/execute/cancel { span_id, sig }
             │
             ├── process exec span? -> existing world::exec path
             └── member dispatch span? -> retained run_control cancel path
                          │
                          ├── delivered = true  -> shell converges to stopped/failed/invalidated
                          └── delivered = false -> shell must not keep advertising live forever
```

### Lifecycle state machine

```text
Allocating
    │
    ├── typed dispatch accepted, control retained, handle surfaced
    ▼
Ready
    │
    ├── first runtime event / active control loop
    ▼
Running
    │
    ├── clean cancel / clean terminal completion -> Stopped
    ├── bootstrap failure before readiness      -> Failed
    ├── stream closes before readiness          -> Failed
    ├── stream closes after readiness           -> Invalidated
    └── world generation rollover               -> Invalidated, then replacement Allocating
```

### Dependency graph

```text
agent-api-types::ExecuteRequest
        │
        ├── shell/world_ops request builder
        │       │
        │       └── shell/async_repl member launch + replacement
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

### Error & Rescue Registry

| Failure | Detection point | Required behavior | Rescue |
| --- | --- | --- | --- |
| `member_dispatch` payload missing required identity fields | request validation in `agent-api-types` and handler parse | reject before world launch | fix shell builder, no state mutation beyond existing allocating snapshot |
| authoritative world binding does not match active session world | shell preflight and world-agent recheck | fail closed, no local fallback | restart or rebind world first |
| in-world backend cannot start | world-agent member manager | emit terminal error, shell persists failed member | operator sees explicit failure, stale member stays dead |
| control stream closes before handle surfaces | shell retained-control consumer | mark failed, never live | retry only through normal next launch path |
| control stream closes after live ownership | shell retained-control consumer | invalidate, persist terminal reason, clear liveness | next command or restart can create replacement |
| cancel delivery misses span | `execute_cancel` response or timeout | shutdown surfaces failure and shell converges non-live | operator retries or stops REPL, but no fake liveness |
| replacement launch fails after stale invalidation | replacement path in `async_repl.rs` | honest absence, predecessor remains invalidated | next valid command attempts fresh launch |

## File Plan

### Primary implementation surfaces

#### 1. `crates/agent-api-types/src/lib.rs`

Deliver:

- `MemberDispatchRequestV1`
- additive `ExecuteRequest.member_dispatch`
- validation that regular exec and member dispatch are mutually exclusive
- round-trip tests for the new typed payload

#### 2. `crates/shell/src/execution/routing/dispatch/world_ops.rs`

Deliver:

- a dedicated member-dispatch request builder that reuses existing transport/client creation,
- request-level trace metadata and policy/world inputs,
- stream-consumption helpers that can drive retained-control semantics instead of stdout/stderr-only
  command output.

This file should stay transport-focused.

Do not move orchestration session ownership into it.

#### 3. `crates/shell/src/repl/async_repl.rs`

Deliver:

- split member remote startup from host orchestrator local startup,
- persist member participant in `Allocating` before remote launch,
- consume remote start/event/terminal/cancel outcomes through the same liveness rules already used
  for local runtimes,
- keep replacement lineage and invalidation behavior intact.

#### 4. `crates/world-agent/src/service.rs`

Deliver:

- branch `execute_stream(...)` on `member_dispatch`,
- route ordinary process exec exactly as before,
- route member dispatch into the new internal manager,
- extend `execute_cancel(...)` so a span can address either process exec or member dispatch.

#### 5. `crates/world-agent/src/member_runtime.rs`

New internal module.

Deliver:

- Linux-first member runtime manager,
- start context validation against world binding,
- `run_control(...)` startup in-world,
- cancel/event/completion retention,
- `ExecuteStreamFrame` emission,
- cleanup on completion, cancel, and abnormal stream loss.

#### 6. `crates/shell/tests/support/socket.rs`

Deliver:

- additive request-stub support for `member_dispatch`,
- assertions that recorded `/v1/execute/stream` payloads include the typed member contract when the
  shell launches a world member.

#### 7. `crates/shell/tests/support/repl_world_agent.rs`

Deliver:

- additive capture of typed member-dispatch payloads and their lineage fields,
- optional canned lifecycle scripting for ready, event, cancel, and terminal cases.

#### 8. Test owners

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

### Step 1. Add the typed transport contract in `agent-api-types`

Deliverables:

1. additive `MemberDispatchRequestV1`,
2. `ExecuteRequest.member_dispatch`,
3. request validation and round-trip tests,
4. no behavior change for ordinary process execution callers.

Acceptance gate:

- all existing transport callers still compile unchanged,
- new typed payload round-trips cleanly.

### Step 2. Extend shell test stubs and request builders before touching runtime behavior

Deliverables:

1. `socket.rs` can deserialize and record `member_dispatch`,
2. `repl_world_agent.rs` can capture and script typed member lifecycle responses,
3. `world_ops.rs` can build a typed member-dispatch request over existing transport plumbing.

Acceptance gate:

- tests can assert payload contents before the shell runtime switch lands.

### Step 3. Add the Linux-first world-agent member runtime manager

Deliverables:

1. new internal `member_runtime.rs`,
2. start/ready/event/completion ownership in-world,
3. stream-frame emission for lifecycle events,
4. span registration for cancel delivery,
5. strict world-binding validation against authoritative `world_id` and `world_generation`.

Acceptance gate:

- unit/integration tests can start and cancel a typed member dispatch without involving shell REPL
  logic yet.

### Step 4. Switch `async_repl.rs` member launch from local to remote retained control

Deliverables:

1. `prepare_member_runtime_startup_for_descriptor(...)` no longer builds a local gateway for the
   member path,
2. `start_member_runtime_with_prepared(...)` becomes the remote member transport path,
3. `ensure_member_runtime_ready(...)` and replacement startup both consume the new remote path,
4. host orchestrator startup remains unchanged.

Acceptance gate:

- first world-backed command launches the member through `/v1/execute/stream`,
- same-generation reuse still works,
- missing parent/binding/selection still fail before remote launch.

### Step 5. Wire cancel, abnormal terminal convergence, and restart replacement

Deliverables:

1. `/v1/execute/cancel` can reach world-member spans,
2. shutdown and restart paths converge member state to stopped/failed/invalidated honestly,
3. replacement launch on the new generation keeps fresh `participant_id` and correct
   `resumed_from_participant_id`.

Acceptance gate:

- restart with a live member yields either a live replacement or honest absence,
- stale generation never regains liveness.

### Step 6. Close the test wall before any opportunistic cleanup

Required test owners:

1. `agent-api-types` unit tests for transport typing,
2. `world-agent` tests for member dispatch startup/cancel/terminal cleanup,
3. `async_repl.rs` runtime tests for remote member startup and replacement,
4. shell integration/contract tests for status, trace, and replacement behavior.

Acceptance gate:

- targeted validation commands below are green,
- no docs or cleanup work starts first.

### Step 7. Only then touch docs if runtime wording changed

This is not a docs-first slice.

Touch docs only if the implementation changes operator-visible transport wording or trace examples.

## Code Quality Review

### Issue 1. Do not fork the retained-control state machine

The easy bad move is a second "member-specific" startup state machine in the shell that mostly
copies the local orchestrator path.

Recommendation:

- keep one retained-control lifecycle discipline,
- split only the launch transport and world-agent ownership details.

Why:

- DRY matters here because "live" is the whole product claim.

### Issue 2. Keep transport typing explicit, not clever

There are two viable shapes:

- additive `member_dispatch` field now,
- or a broader tagged execute-target enum.

Recommendation:

- use the additive field for this slice,
- leave the enum refactor for later if the transport family genuinely grows.

Why:

- minimal diff and explicit over clever.

### Issue 3. Keep `world_ops.rs` transport-focused

Do not let `world_ops.rs` become an orchestration-runtime brain.

Recommendation:

- request construction and stream framing in `world_ops.rs`,
- participant/store ownership stays in `async_repl.rs`.

Why:

- module boundaries stay boring and readable.

### Issue 4. Keep `service.rs` as the branch point, not the whole implementation

Jamming all member-runtime ownership code into `WorldAgentService::execute_stream(...)` would
create a 500-line switchboard nobody wants to maintain.

Recommendation:

- let `service.rs` validate and dispatch,
- let `member_runtime.rs` own the internal lifecycle.

Why:

- engineered enough, not under-engineered and not a new framework.

## Test Review

### Test framework detection

- Runtime: Rust
- Framework: `cargo test`
- Primary packages: `agent-api-types`, `world-agent`, `shell`
- No LLM prompt/eval suite expansion is required for this slice

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/agent-api-types/src/lib.rs
    │
    ├── ExecuteRequest.member_dispatch
    │   ├── [GAP]         Regular process exec request still validates
    │   ├── [GAP]         Member-dispatch request validates with full identity tuple
    │   ├── [GAP]         Missing participant/world fields fail closed
    │   └── [GAP]         Process exec + member_dispatch together fails closed
    │
    └── ExecuteStreamFrame compatibility
        └── [★★★ TESTED] Existing frame families already round-trip for process execution

[+] crates/world-agent/src/service.rs
    │
    ├── execute_stream()
    │   ├── [★★★ TESTED] Regular streamed process exec path already works
    │   ├── [GAP]         Branches typed member dispatch away from process exec
    │   ├── [GAP]         Rejects mismatched/non-authoritative world binding
    │   └── [GAP]         Registers member span for cancel delivery
    │
    └── execute_cancel()
        ├── [★★★ TESTED] Process-exec cancel path already works
        └── [GAP]         Member-dispatch cancel reaches retained control and converges terminally

[+] crates/world-agent/src/member_runtime.rs
    │
    ├── start()
    │   ├── [GAP]         Build backend in-world and start run_control(...)
    │   ├── [GAP]         Surface session handle before Ready
    │   ├── [GAP]         Emit lifecycle events with participant identity
    │   └── [GAP]         Bootstrap failure emits Error/terminal cleanup
    │
    └── abnormal termination
        ├── [GAP]         Stream closes before readiness -> failed
        └── [GAP]         Stream closes after readiness -> invalidated

[+] crates/shell/src/execution/routing/dispatch/world_ops.rs
    │
    ├── build_member_dispatch_request(...)
    │   ├── [GAP]         Reuses policy/world routing inputs
    │   ├── [GAP]         Carries participant/lineage/world fields correctly
    │   └── [GAP]         Preserves trace parent metadata
    │
    └── stream consumption
        ├── [GAP]         Ready/Status/Event frames advance shell liveness correctly
        └── [GAP]         Cancel failure does not leave fake liveness

[+] crates/shell/src/repl/async_repl.rs
    │
    ├── start_member_runtime_with_prepared()
    │   ├── [GAP] [->E2E] Uses remote member-dispatch transport, not local gateway reuse
    │   ├── [GAP]         Persists Allocating before remote ownership
    │   ├── [GAP]         Ready -> Running only after retained control is proven
    │   └── [GAP]         Bootstrap failure marks participant failed
    │
    ├── ensure_member_runtime_ready()
    │   ├── [GAP] [->E2E] First world-backed command launches member lazily over transport
    │   ├── [GAP]         Same-generation command reuses live remote member
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

### Missing tests to add to the plan

1. **`crates/agent-api-types/src/lib.rs`**
   - Add round-trip and validation tests for:
     - well-formed `member_dispatch`
     - missing required fields
     - illegal combination of process exec + member dispatch

2. **`crates/world-agent/tests/streamed_execute_cancel_v1.rs` or new focused world-agent tests**
   - Add Linux-first tests for:
     - successful member-dispatch startup
     - member-dispatch cancel delivery
     - bootstrap failure before readiness
     - abnormal post-ready termination cleanup

3. **`crates/shell/src/repl/async_repl.rs`**
   - Add runtime tests for:
     - remote member startup uses transport instead of local `gateway.run_control(...)`
     - Allocating -> Ready -> Running progression only after ownership is retained
     - missing parent/binding/selection fail before launch
     - replacement launch preserves `resumed_from_participant_id`

4. **`crates/shell/tests/repl_world_first_routing_v1.rs`**
   - Add integration cases for:
     - first world-backed command launches member through typed execute-stream payload
     - restart replacement uses typed execute-stream payload on the new generation
     - replacement failure leaves no authoritative-live member

5. **`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`**
   - Add contract cases for:
     - status remains correct for a real remotely launched member
     - cancel/restart terminal states stay non-live
     - toolbox remains anchored to the orchestrator session

6. **`crates/shell/tests/agent_hub_trace_persistence.rs`**
   - Add producer-backed cases for:
     - remote member Registered / Status / terminal event emission
     - replacement lineage staying top-level in trace rows

7. **`crates/shell/tests/support/socket.rs` and `crates/shell/tests/support/repl_world_agent.rs`**
   - Add harness assertions that typed member-dispatch payloads are actually recorded and inspectable.

### Test artifact

The eng-review QA artifact for this plan is:

[spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260502-140843.md](/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260502-140843.md)

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
| --- | --- | --- | --- | --- | --- |
| typed request construction | shell sends malformed lineage/world metadata and world-agent guesses | no | not yet | no | yes until transport validation lands |
| shell preflight | missing authoritative world binding still emits a remote launch attempt | no | partial | partial | yes until preflight-fails-before-transport test lands |
| world-agent startup | backend cannot start in-world but participant becomes live anyway | no | not yet | no | yes until bootstrap-failure coverage lands |
| remote readiness | session handle never surfaces but shell transitions Ready | no | partial | no | yes until Ready gating test lands |
| cancel delivery | `/v1/execute/cancel` misses the member span and shell keeps advertising live | no | partial | partial | yes until cancel convergence coverage lands |
| replacement launch | stale member invalidates, replacement fails, stale trace row looks live again | partial | partial | no | yes until replacement + trace suppression tests land |
| status/trace projection | remote producer emits wrong participant/world identity | no | partial | yes | yes until producer-backed status/trace tests land |

Critical gap rule:

If this slice can launch a member locally on fallback, advertise live before remote ownership is
retained, or leave stale liveness after cancel/restart failure, the implementation is not done.

## Performance Review

This is still correctness-first. The performance risk is architectural sprawl, not CPU.

Performance rules:

1. do not build a second client/transport stack for member dispatch,
2. do not re-run remote launch if a matching authoritative-live member already exists for the
   current generation,
3. do not add extra full-store scans to steady-state command execution if the current REPL already
   owns the live member handle,
4. do not make `execute_cancel` poll forever waiting for delivery.

Performance issues found:

- 0 throughput blockers
- 1 structural caution: keep cancel delivery bounded and reuse the existing span registry pattern

The bad optimization here would be inventing a scheduler or cache to "speed up" a single-member
placement seam. Don't.

## DX Review

This slice has no UI scope. It has strong developer/operator scope.

The user here is the maintainer asking:
"If `status` says my world member is live, can I trust that it is actually in the world?"

### Developer journey map

| Stage | What the developer is doing | Current friction | Target after this slice |
| --- | --- | --- | --- |
| 1 | enable one world-scoped member backend | low-medium | keep low |
| 2 | start REPL with shared world enabled | low | keep low |
| 3 | run first world-backed command | high, placement is still shell-local today | transport-backed lazy launch |
| 4 | inspect recorded execute-stream request during tests | high, no typed member payload exists | explicit typed payload visible in stubs |
| 5 | inspect `substrate agent status --json` | medium | live member row reflects actual remote runtime |
| 6 | cancel or stop the session | medium-high | same cancel seam as process exec, honest terminal state |
| 7 | restart the shared world | medium-high | replacement launches through the same remote seam |
| 8 | inspect trace rows | medium | producer-backed world/member identity stays top-level |
| 9 | extend later to more members/platforms | high | clear boundary around what v1 does and does not do |

### Developer empathy narrative

Right now the repo makes you do too much trust-based reasoning.

You can read the shell store, see a member participant with the right `world_id`, and still have
to remember that the actual `run_control(...)` turn is local. That is bad product for a developer
tool. It turns status into folklore.

After this slice, the mental model gets simpler. If the member is live, it crossed the world
transport and retained ownership there. If the remote path failed, you see honest absence or
explicit failure. Painful sometimes. But true.

### DX Scorecard

| Dimension | Score | Notes |
| --- | --- | --- |
| Getting started | 6/10 | world-member configuration exists, placement truth is still hidden today |
| API/CLI naming | 8/10 | execute-stream and cancel naming are already good |
| Error messages | 7/10 | fail-closed posture exists, but member-specific transport failures need first-class wording |
| Docs findability | 6/10 | planning docs are good, implementation truth still stops one seam short |
| Upgrade path safety | 8/10 | additive transport contract keeps existing callers stable |
| Observability | 8/10 | store + trace + recorded request stubs make this inspectable once the producer is real |
| Recovery guidance | 7/10 | restart invalidation is solid, remote replacement failure needs explicit terminal reasons |
| Escape hatches | 6/10 | fail-closed is right, but operator remediation needs to be obvious |

Overall DX score: **7/10**

### DX Implementation Checklist

- make typed member transport visible in tests and diagnostics,
- keep every remote failure message in problem + cause + fix style,
- keep `status` and `toolbox` consuming existing shell authority instead of remote guesses,
- preserve top-level `world_id` and `world_generation` in trace rows,
- document any new operator-visible error only if implementation wording changes.

### TTHW assessment

Current TTHW for "prove whether world-scoped member placement is real" is about **15 minutes**.

You have to read the seam doc, `async_repl.rs`, the transport service, and the tests to discover
that the member still launches locally.

Target after this slice: **under 8 minutes**.

That means a maintainer can:

1. read `PLAN-11.md`,
2. inspect one typed `ExecuteRequest`,
3. run the targeted tests,
4. trust that status and trace are describing the actual execution boundary.

## Cross-Phase Themes

These themes showed up across scope, engineering, and DX review:

1. **Keep one transport family.**
   - New public APIs here would be theater, not leverage.

2. **Keep shell authority separate from world execution.**
   - The whole design works because those jobs are different.

3. **Typed beats magical.**
   - A real transport contract is worth the extra struct.

4. **Fail closed on placement truth.**
   - Local fallback would make the product lie in its most important state transition.

5. **Tests must prove placement, not just metadata.**
   - The repo already proves the metadata story. This slice is about the last mile.

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Transport contract | `crates/agent-api-types/`, `crates/shell/tests/support/` | — |
| World-agent manager | `crates/world-agent/src/`, `crates/world-agent/tests/` | Transport contract |
| Shell runtime switch | `crates/shell/src/repl/`, `crates/shell/src/execution/routing/dispatch/`, `crates/shell/tests/` | Transport contract |
| Trace/status regression wall | `crates/shell/tests/`, maybe `crates/shell/src/execution/` | World-agent manager, Shell runtime switch |

### Parallel lanes

- Lane A: transport contract and stub capture support
- Lane B: world-agent member manager after Lane A
- Lane C: shell runtime switch after Lane A
- Lane D: integration/trace/status regression wall after B + C

### Execution order

1. Launch Lane A first.
2. After A merges cleanly, run Lane B and Lane C in parallel worktrees.
3. Merge B + C.
4. Run Lane D last because it verifies the full end-to-end seam.

### Conflict flags

- Lanes B and C are safe in parallel if both avoid editing `agent-api-types` after Lane A lands.
- Lane D touches the shared shell test surface and should stay last.

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
3. `/v1/execute/cancel` can cancel a live member-dispatch span,
4. abnormal stream loss or startup failure converges the member to a non-live terminal state,
5. replacement launch after world-generation rollover crosses the same transport and preserves
   `resumed_from_participant_id`,
6. `substrate agent status --json` and trace rows remain participant-correct and world-correct for
   the real remote producer,
7. no host fallback exists for world-scoped members,
8. targeted validation commands below are green.

### Recommended verification commands

```bash
cargo test -p agent-api-types -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_hub_trace_persistence -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
```

## Completion Summary

- Step 0: scope accepted as transport-placement slice, no new public API family
- Architecture Review: 4 issues found, all about keeping one honest retained-control and transport contract
- Code Quality Review: 4 issues found, all about avoiding split brains between shell, transport, and world-agent
- Test Review: diagram produced, 19 direct gaps identified
- Performance Review: 1 structural caution, 0 throughput blockers
- DX Review: 7/10 overall, TTHW 15 min to target under 8 min
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0, repo has no root `TODOS.md`, deferred work captured here
- Failure modes: 7 critical gaps flagged for the implementation to close through tests and fail-closed behavior
- Outside voice: skipped, `claude` CLI is installed but unauthenticated on 2026-05-02
- Parallelization: 4 steps, 1 real parallel window after the transport contract lands
- Lake Score: complete option chosen for every in-slice decision

<!-- AUTONOMOUS DECISION LOG -->
## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Scope | Keep `/v1/execute/stream` and `/v1/execute/cancel` as the only transport seam | Mechanical | Boring by default | Existing transport already solves routing, NDJSON streaming, and cancel delivery | New `/v1/member/*` family |
| 2 | Transport | Use additive `ExecuteRequest.member_dispatch` now | Taste | Explicit over clever | Smallest complete change that keeps old callers stable and makes the new intent readable | Immediate tagged target-enum refactor |
| 3 | Authority | Keep canonical session-root writes in the shell only | Mechanical | Systems over heroes | `world-agent` execution and shell authority are intentionally different jobs | Remote canonical-state writes |
| 4 | Failure posture | Deny world-member launch rather than falling back locally | Mechanical | Completeness | A fallback would make status and trace lie about placement | Host-side fallback |
| 5 | World-agent structure | Add one internal `member_runtime.rs` instead of bloating `service.rs` | Mechanical | Engineered enough | `service.rs` should branch, not become the lifecycle implementation | 500-line `execute_stream` branch |
| 6 | Reuse | Preserve existing liveness and lineage helpers in `session.rs` | Mechanical | DRY | The model already encodes the correct live bar and replacement semantics | Parallel remote-only state model |
| 7 | Tests | Extend existing shell and world-agent stubs instead of building a new harness | Mechanical | Pragmatic | The repo already records execute-stream payloads and scripted world-agent streams | New bespoke integration harness |
| 8 | Rollout | Land transport contract first, then runtime switch, then regression wall | Mechanical | Incremental over revolutionary | This keeps contract churn separate from lifecycle churn | Big-bang multi-crate rewrite |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
| --- | --- | --- | --- | --- | --- |
| CEO Review | `/plan-ceo-review` | Scope and strategy | 1 | CLEAR | Kept the slice narrow, rejected a new public API family, and froze the host-authority/world-execution split |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | SKIPPED | No separate outside-model review run, and `claude` CLI auth is missing for outside voice on this machine |
| Eng Review | `/plan-eng-review` | Architecture and tests (required) | 1 | CLEAR | Locked the typed transport contract, the fail-closed placement posture, and the full test wall for placement/cancel/replacement correctness |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | SKIPPED | No UI scope |

**UNRESOLVED:** 0 plan-level decision points remain. The remaining work is implementation of the
typed transport seam and the regression wall already listed.

**VERDICT:** CEO + ENG CLEARED. `PLAN-11` is ready to execute as the transport-backed placement
cutover that makes world-scoped member runtime truth match the canonical session model.
