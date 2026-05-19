# SOW: In-World Member Dispatch Over Existing Host<->World Transport

Status: implementation-oriented draft. This document defines the remaining member-runtime placement slice after the shell-side launch, persistence, status, and replacement semantics from [10-member-runtime-launch-seam.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/10-member-runtime-launch-seam.md) are already present in production code. It is intentionally bounded to moving world-scoped member execution from the shell process into `world-agent` over the existing host<->world execute/stream transport. It does not reopen shared-world ownership, world-generation authority, session-centric live-state authority, shell-side replacement semantics, or broad UAA adoption.

## Objective

Land one Linux-first production path where a world-scoped member runtime is started inside `world-agent`, not inside the shell process, while preserving the already-landed shell authority model:

- the shell remains the authoritative owner of orchestration session state,
- the shell still persists the canonical parent plus participant records,
- the shell still applies invalidation and replacement semantics,
- but the actual member `AgentWrapperGateway.run_control(...)` call moves into `world-agent`,
- and the shell drives that runtime through the existing `/v1/execute/stream` plus `/v1/execute/cancel` transport.

The required outcome is not a second agent hub. The required outcome is that the existing world-scoped member participant model becomes honest about placement: member execution actually runs inside the active world boundary that its participant record claims to be bound to.

## Why This Is Needed

The main launch/lifecycle seam from [10-member-runtime-launch-seam.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/10-member-runtime-launch-seam.md) is now effectively landed in the shell runtime:

- shell-owned UAA orchestrator startup is real,
- session-centric runtime state is real,
- authoritative `world_id` plus `world_generation` plumbing is real,
- stale-generation invalidation is real,
- replacement-member lineage across world rollover is real,
- and `substrate agent status` already consumes real member participants.

The remaining mismatch is narrower:

- `prepare_member_runtime_startup_for_descriptor(...)` and `start_member_runtime_with_prepared(...)` exist in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs),
- but `start_member_runtime_with_prepared(...)` still delegates to `start_host_orchestrator_runtime_with_prepared(...)`,
- which still calls `gateway.run_control(...)` inside the shell process,
- so the member is only correlated to a world binding in state,
- not actually launched inside `world-agent` over the host<->world transport.

That is now the last major placement gap called out in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md).

## Relationship To Landed Slices

This slice depends on, and must consume without redesign:

- [03-shared-world-ownership-linux-first.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/03-shared-world-ownership-linux-first.md)
- [04-thread-world-binding-into-runtime-state.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/04-thread-world-binding-into-runtime-state.md)
- [05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md)
- [07-world-replacement-ordering-rollback-atomic-metadata.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/07-world-replacement-ordering-rollback-atomic-metadata.md)
- [08-explicit-orchestration-authority-event-emission.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/08-explicit-orchestration-authority-event-emission.md)
- [09-live-state-authority-and-compatibility-cutover.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/09-live-state-authority-and-compatibility-cutover.md)
- [10-member-runtime-launch-seam.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/10-member-runtime-launch-seam.md)

Non-goal reminder:

- this slice does not re-decide member selection,
- does not re-decide replacement ordering,
- does not re-decide who owns `world_generation`,
- and does not broaden the architecture into a new public `agent` service surface.

## Current Repo Seams And Exact Code Surfaces

### Shell-side member runtime seam already exists

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  - `prepare_member_runtime_startup_for_descriptor(...)`
  - `start_member_runtime_with_prepared(...)`
  - `ensure_member_runtime_ready(...)`
  - `reconcile_member_runtime_generation(...)`
  - `shutdown_host_orchestrator_runtime(...)`
- The critical current-reality fact is that:
  - `start_member_runtime_with_prepared(...)` currently calls `start_host_orchestrator_runtime_with_prepared(...)`
  - and `start_host_orchestrator_runtime_with_prepared(...)` is where `gateway.run_control(...)` happens
  - so member launch still uses the shell-owned local UAA runtime path.

### Shell authority and state seams already exist

- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
  - `new_member_participant(...)`
  - `new_replacement_participant(...)`
  - `can_advertise_live()`
  - `is_authoritative_live()`
  - `mark_runtime_ownership_retained()`
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
  - `persist_participant(...)`
  - `persist_orchestration_session(...)`
  - `resolve_live_orchestrator_participant(...)`
  - `list_live_participants_for_session(...)`
  - `invalidate_stale_world_members_for_session(...)`
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
  - `build_status_report(...)`
  - `build_toolbox_status_report(...)`

These are not missing. This slice must keep them authoritative.

### Existing host<->world transport already exists

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
  - `ExecuteRequest`
  - `ExecuteStreamFrame`
  - `ExecuteCancelRequestV1`
  - `ExecuteCancelResponseV1`
- [crates/agent-api-client/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-client/src/lib.rs)
  - `execute_stream(...)`
  - `cancel_execute(...)`
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
  - `build_agent_client_and_request_with_trace_metadata(...)`
  - `stream_non_pty_via_agent(...)`
  - `process_agent_stream_body(...)`
- [crates/world-agent/src/handlers.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/handlers.rs)
  - `execute_stream(...)`
  - `execute_cancel(...)`
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
  - `execute_stream(...)`
  - `execute_cancel(...)`

This slice must reuse that transport rather than creating a separate top-level control API.

### World-agent already has one relevant runtime-management pattern

- [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs)
  - `GatewayRuntimeManager`
  - `GatewayRuntimeStartContext`
  - runtime manifest persistence
  - pid rediscovery
  - cgroup attach
  - ready/restart lifecycle
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
  - `prepare_gateway_runtime_request(...)`
  - `resolve_gateway_runtime_binding(...)`

That code is not the member-launch surface, but it is a strong implementation reference for Linux-first world-owned long-lived runtime management.

## In Scope

- moving production world-scoped member execution into `world-agent`
- reusing `/v1/execute/stream` plus `/v1/execute/cancel` as the transport seam
- adding the typed request metadata needed to tell `world-agent` to launch a member runtime instead of a shell command
- forwarding member lifecycle and wrapper events back to the shell over existing stream frames
- keeping the shell as the authoritative writer of session-root live state
- preserving already-landed replacement-member lineage and invalidation behavior while moving the actual replacement launch into the world transport path
- Linux-first implementation and validation

## Out Of Scope

- new shared-world ownership or rollback logic
- new session-root schema or new compatibility-cutover rules
- reworking the host orchestrator runtime into a world-owned service
- broad `substrate agent start|resume|fork|stop` CLI productization
- a new `/v1/member/*` or `/v1/agents/*` public API family
- toolbox redesign
- macOS and Windows parity beyond fail-closed posture
- changing the already-landed selection rule except where transport serialization needs a stable descriptor shape

## Blockers And Gaps

1. Member runtime placement is still wrong.
   - The shell allocates correct member participants, but the actual UAA control turn still lives in the shell process.

2. The transport has no typed member-dispatch request today.
   - `ExecuteRequest` only models process execution.
   - There is no explicit payload for "start this world-scoped member runtime under attached control."

3. `world-agent` has no member-runtime manager today.
   - There is no world-owned equivalent of the shell path that builds a UAA gateway, starts `run_control(...)`, retains cancel ownership, forwards events, and observes completion for a member session.

4. `/v1/execute/cancel` only knows how to cancel process exec spans today.
   - Member dispatch needs the same cancel endpoint to reach a retained world-owned UAA session, not just a process group.

5. The shell member path is structurally coupled to the local orchestrator launcher.
   - Reusing `start_host_orchestrator_runtime_with_prepared(...)` for member launch made sense for the first shell-only slice.
   - It is now the exact coupling that must be broken.

6. Current tests prove state semantics, not placement semantics.
   - Existing shell tests prove member records, replacement lineage, and status.
   - They do not yet prove that the member binary actually starts inside `world-agent` over the transport boundary.

## Required Semantics And Invariants

### 1. Shell authority remains authoritative

This slice does not move live-state authority into `world-agent`.

Required rule:

- `world-agent` owns in-world member process execution and in-world UAA control retention,
- the shell owns canonical `AgentRuntimeStateStore` writes,
- the shell remains the authority for participant state transitions exposed through `substrate agent status`,
- and the shell remains the owner of stale-generation invalidation and replacement decisions.

`world-agent` may emit events and terminal outcomes; it must not become the writer of canonical session-root state.

### 2. No host-side fallback for world-scoped members

Once a selected runtime resolves to `execution.scope=world`, the shell must not silently fall back to local `gateway.run_control(...)`.

Required rule:

- if world transport member dispatch cannot be established, member startup fails closed,
- the participant remains non-live or transitions terminally failed,
- and the shell must not "helpfully" launch the member locally.

### 3. Launch must consume the authoritative active world binding

The shell must dispatch against the already-authoritative active shared world from the orchestration session snapshot.

Required data carried into member dispatch:

- `orchestration_session_id`
- `participant_id`
- `orchestrator_participant_id`
- `resumed_from_participant_id` when replacement applies
- `agent_id`
- `backend_id`
- backend/runtime selector sufficient for `world-agent` to build the correct UAA backend
- authoritative `world_id`
- authoritative `world_generation`
- caller-owned `run_id`

The world side must consume those values. It must not invent a different `world_id`, `world_generation`, or lineage.

### 4. The existing transport endpoints remain the seam

Linux-first member dispatch must ride:

- `POST /v1/execute/stream` for launch plus attached-control streaming
- `POST /v1/execute/cancel` for cancellation
- existing `ExecuteStreamFrame::Start`, `Event`, `Exit`, and `Error` frames for control-plane feedback

The implementation may extend request payloads and may add additive event contents, but it must not require a new top-level world-agent endpoint for the first shipped slice.

### 5. Readiness must remain strict

The shell may persist a member participant in `allocating`, but it may only advertise it authoritative-live after all of the following have occurred:

- `world-agent` has successfully started the member backend inside the active world,
- a real UAA session handle has surfaced,
- remote cancel ownership is retained,
- remote event streaming is active,
- remote completion observation is retained,
- and the shell has persisted the updated participant snapshot.

This is the same live-authority bar already enforced by `can_advertise_live()` and `is_authoritative_live()`.

### 6. World replacement semantics stay the same; only placement changes

When `world_generation` rolls forward:

- stale member participants still invalidate in the shell store,
- the predecessor still remains auditable,
- the replacement participant still receives a fresh `participant_id`,
- `resumed_from_participant_id` still points at the predecessor,
- but the replacement runtime must now be launched through `world-agent`, not in the shell.

### 7. Event and trace identity must stay participant-correct

Remote member lifecycle events forwarded from `world-agent` must preserve:

- `orchestration_session_id`
- `participant_id`
- `parent_participant_id` when present
- `resumed_from_participant_id` when present
- `role=member`
- `agent_id`
- `backend_id`
- top-level `world_id`
- top-level `world_generation`
- caller-owned `run_id`

The transport must not flatten those into anonymous shell command rows.

### 8. Cancel and terminal handling must remain fail-closed

If the shell asks `world-agent` to cancel a member dispatch span and delivery fails:

- the shell must not keep advertising the participant as safely live forever,
- shutdown must surface failure,
- and the participant must converge to `stopped`, `failed`, or `invalidated` based on what is actually observed.

### 9. Linux-first posture is explicit

For the first shipped slice:

- Linux is the required implementation target,
- non-Linux world-scoped member dispatch may deny as unavailable,
- and non-Linux paths must fail closed instead of routing the member locally.

## Recommended Implementation Shape

### 1. Split local orchestrator launch from remote world-member launch

Keep the current shell-owned orchestrator path intact.

Change the member path so that:

- `prepare_member_runtime_startup_for_descriptor(...)` returns a member-dispatch preparation object, not a local `AgentWrapperGateway`,
- `start_member_runtime_with_prepared(...)` opens a long-lived world-agent execute stream,
- and the shell uses that stream as the retained attached-control boundary for the member runtime.

The shell should still allocate and persist the participant record before remote launch so the existing authority model remains intact.

### 2. Extend `ExecuteRequest` with a typed member-dispatch payload

Recommended transport shape:

- keep `/v1/execute/stream`,
- keep `ExecuteStreamFrame`,
- add one additive typed request payload such as `member_dispatch: Option<MemberDispatchRequestV1>` under `ExecuteRequest`,
- and branch inside `world-agent` on that field.

Do not encode member launch as a magic shell command string.

Minimum member-dispatch payload should include:

- orchestration and participant identity
- member backend selection data
- lineage data
- run correlation data
- authoritative world binding data

The top-level `ExecuteRequest` can continue to carry:

- `cwd`
- `env`
- `policy_snapshot`
- `shared_world`
- `world_network`

so the request still uses the existing world-routing and policy-input machinery.

### 3. Implement a Linux-first world-agent member runtime manager

Add a dedicated world-agent runtime manager modeled after the rigor of `gateway_runtime.rs`, but specialized for pure-agent member control:

- build the UAA backend inside `world-agent`
- start `run_control(...)`
- retain cancel ownership
- forward wrapper events as `ExecuteStreamFrame::Event`
- emit terminal `Exit` or `Error`
- keep cancellation addressable by the transport `span_id`
- and tear down cleanly on completion or cancel

This manager should be internal to `world-agent`. It does not need a public status API in the first slice.

### 4. Reuse the existing shell lifecycle helpers wherever possible

The shell already knows how to:

- persist `allocating`
- mark retained ownership
- translate runtime events into canonical `AgentEvent`
- publish ready/status/invalidated alerts
- and persist heartbeat snapshots

Do not duplicate that logic in a second shell-local state machine. Parameterize or extract the generic pieces from [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) so both:

- the local orchestrator path, and
- the remote in-world member path

share the same persistence and event-translation discipline.

### 5. Reuse `/v1/execute/cancel` rather than inventing a second cancel plane

The world-agent side should register member dispatch spans in a cancel registry that `/v1/execute/cancel` can reach.

Recommended shape:

- keep process-exec cancellation behavior as-is,
- add a member-dispatch cancellation branch keyed by the same `span_id`,
- and let the shell continue treating cancel as one transport operation.

### 6. Keep the first slice intentionally narrow

The first landed slice should support:

- one shell-owned orchestrator
- one active world-scoped member per selected descriptor per active world generation
- replacement on generation rollover
- cancel/stop through the same transport

Do not combine this with multi-member scheduling or public session-control UX work.

## Exact Code Areas To Change

### Shell

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  - stop routing member launch through `start_host_orchestrator_runtime_with_prepared(...)`
  - introduce a remote member dispatch startup path
  - adapt shutdown and replacement handling to the remote transport-backed control boundary
  - reuse existing lifecycle persistence and runtime-event helpers

- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
  - add a request-builder/helper for member dispatch over existing world transport
  - reuse existing world client creation, policy snapshot injection, and shared-world routing inputs
  - add any stream-consumption helper needed for non-command member lifecycle frames

- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
  - only if needed for additive remote-control metadata
  - do not redesign role, lineage, or world-binding fields

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
  - validate that status/doctor output remains honest for the remote member path
  - add no new selection semantics here

### Transport

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
  - add the typed member-dispatch request payload carried over `ExecuteRequest`
  - add any additive validation helpers needed for that payload
  - keep `ExecuteStreamFrame` compatible with existing process-exec callers

- [crates/agent-api-client/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-client/src/lib.rs)
  - only additive request-construction helpers if needed
  - no new transport family required

### World agent

- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
  - branch `execute_stream(...)` for typed member dispatch
  - wire the member runtime manager
  - integrate cancel delivery for remote member spans

- [crates/world-agent/src/handlers.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/handlers.rs)
  - input validation only as required by the additive request schema

- [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
  - register any new internal module wiring only

- recommended new module:
  - [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
  - purpose:
    - UAA backend build/start
    - retained cancel/event/completion ownership
    - stream-frame emission
    - cancel integration
    - Linux-first lifecycle cleanup

### Tests

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  - update unit coverage for member startup and replacement to assert remote transport usage rather than local gateway reuse

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
  - assert member launch occurs through world transport on Linux-first world-backed paths

- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)
  - assert remote member rows still preserve top-level world identity and replacement lineage

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
  - assert status/doctor behavior remains fail-closed and participant-aware

- Linux-first world-agent tests:
  - [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
  - [crates/world-agent/tests/](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/)
  - add focused tests for member dispatch startup, event forwarding, cancellation, and terminal cleanup

## Testing, Acceptance Criteria, And Failure Cases

### Acceptance criteria

1. With one live shell-owned orchestrator and one eligible world-scoped member, the first world-backed REPL command causes the member to be launched through `world-agent`, not through local shell `gateway.run_control(...)`.
2. The member participant is first persisted in `allocating`, then becomes authoritative-live only after a real UAA session handle is surfaced from the world-owned runtime.
3. `substrate agent status` shows the live member with correct `participant_id`, `role=member`, `world_id`, and `world_generation`.
4. After world-generation rollover, the prior member becomes invalidated and a replacement member is launched through the same world transport with correct `resumed_from_participant_id`.
5. Shutting down the REPL or cancelling the member dispatch uses `/v1/execute/cancel` and leaves the member in a terminal non-live state.
6. Trace rows and canonical `AgentEvent` rows for the remote member preserve top-level `world_id` and `world_generation`.

### Required failure cases

1. Ambiguous or missing eligible member selection must fail before any transport call is issued.
2. Backend deny by policy must fail before any transport call is issued.
3. Missing authoritative world binding must fail before any transport call is issued.
4. If the world-agent-side UAA backend cannot start, the participant must end failed and must never be advertised live.
5. If the remote stream closes before ownership is established, the participant must end failed, not live.
6. If the remote stream closes after readiness but before clean completion, the participant must end invalidated unless a normal stop path already won the race.
7. If cancel delivery cannot be made, shutdown must surface failure and the participant must not remain authoritatively live indefinitely.
8. On non-Linux or unavailable world transport, world-scoped member startup must fail closed rather than running locally.

### Validation commands

Minimum validation for implementation work on this slice should include:

```bash
cargo test -p world-agent -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_hub_trace_persistence -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
```

If the implementation touches shared-world restart behavior or cancellation internals, add the relevant Linux doctor/smoke evidence already required by the repo guidelines.

## Sequencing And Rollout Notes

1. Land transport typing first.
   - Add the typed member-dispatch payload and its validation without changing the orchestrator path.

2. Land Linux-first world-agent member runtime management next.
   - Keep it internal to `world-agent`.
   - Reuse existing execute-stream and cancel entrypoints.

3. Switch the shell member-launch path after the world-agent path exists.
   - The shell should stop building a local member gateway at that point.

4. Keep the host orchestrator path unchanged in this slice.
   - The orchestrator remains shell-owned.

5. Treat macOS and Windows as follow-on parity work.
   - The first slice may deny world-scoped member dispatch there if the in-world runtime path is not yet implemented.

## Open Choices Left By Current Repo Reality

1. Whether to factor UAA backend registration into a shared helper used by both shell and `world-agent`, or to keep two thin backend builders with identical supported-backend coverage.
   - Shared extraction is cleaner.
   - A duplicated first slice may still be acceptable if tightly bounded and tested.

2. Whether the member-dispatch extension hangs directly off `ExecuteRequest` or uses a tagged execute-target enum.
   - The first option is smaller.
   - The second is structurally cleaner if more transport variants are expected soon.

3. Whether the world-agent-side cancel registry should extend the existing exec-span registry or live beside it as a transport-span multiplexer.
   - Either is valid as long as `/v1/execute/cancel` remains the only caller-facing cancel seam for this slice.
