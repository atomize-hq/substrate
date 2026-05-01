# SOW: Production Member Runtime Launch Seam

Status: implementation-oriented draft. This document defines the missing production seam for launching and tracking world-scoped member runtimes inside an existing orchestration session. It is intentionally bounded to launch and lifecycle plumbing. It does not reopen shared-world authority, replacement ordering, or cleanup/cutover design that belongs to earlier `llm-last-mile` slices.

## Objective

Land one real production launch seam for world-scoped member participants so the shell can:

- select an allowed member backend,
- launch it against the already-authoritative active shared world,
- persist a real live `member` participant record,
- stream and observe its lifecycle with the same rigor as the host orchestrator path,
- and surface that member through existing runtime store and status machinery.

The required outcome is not "member-shaped JSON exists." The required outcome is that production code in the shell actually creates, starts, persists, updates, and retires member participants under a live `orchestration_session_id`.

## Why This Is Needed

The repository now has strong runtime modeling, but the launch seam is still missing.

What already exists:

- participant-capable runtime records in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
- session-centric runtime storage in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- shared-world authority and generation plumbing from [PLAN-03.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-03.md), [04-thread-world-binding-into-runtime-state.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/04-thread-world-binding-into-runtime-state.md), and [07-world-replacement-ordering-rollback-atomic-metadata.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/07-world-replacement-ordering-rollback-atomic-metadata.md)
- stale-generation invalidation rules in [05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md)
- live status/toolbox projections that can already consume member participants once real producers exist in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

What is still missing:

- the only production runtime bootstrap path today is the host orchestrator path in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- the only production `AgentWrapperGateway.run_control(...)` call today is also in that host orchestrator path
- `new_member_participant(...)` and `new_replacement_participant(...)` exist, but there is no production caller outside tests and store fixtures
- invalidation and status suppression logic therefore consume a future contract rather than a live producer

That mismatch is now the bottleneck. The repo can describe member participants honestly, but it still cannot launch one in production.

## Current Repo Seams

### Production runtime path that exists today

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  - `prepare_host_orchestrator_runtime_startup(...)`
  - `start_host_orchestrator_runtime_with_prepared(...)`
  - `shutdown_host_orchestrator_runtime(...)`
  - `persist_world_binding_authority(...)`
  - `invalidate_stale_world_members_after_binding(...)`
- That file proves the current runtime is orchestrator-first:
  - it allocates one host-scoped orchestrator participant,
  - persists one parent orchestration session,
  - and calls `gateway.run_control(...)` only for that host runtime.

### Runtime descriptor and registry surfaces

- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
  - validates only the selected orchestrator path today
  - `validate_runtime_realizability(...)` is written around "selected orchestrator" and the first caller path
- [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs)
  - builds a gateway from one `RuntimeSelectionDescriptor`
  - registers exactly one backend implementation for that descriptor
- [crates/shell/src/execution/agent_runtime/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mod.rs)
  - exports orchestrator-oriented helpers today
  - does not yet expose a production member-launch entrypoint

### Runtime state that is ready to consume a real member producer

- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
  - already models `role=member`
  - already models required `orchestrator_participant_id`
  - already models `world_id` and `world_generation`
  - already models replacement lineage through `resumed_from_participant_id`
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
  - already persists participants under the session-centric runtime tree
  - already groups participants into `AgentRuntimeSessionRecord`
  - already has `invalidate_stale_world_members_for_session(...)`
  - already suppresses invalidated participants in live reads

### Status and toolbox consumer posture

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
  - `build_status_report(...)` already merges live runtime participants with trace-derived history
  - invalidated world members already suppress stale trace fallback
  - `build_toolbox_status_report(...)` already anchors toolbox selection to the live orchestrator session, not a member runtime
- This means status/toolbox mostly need producer validation, not a redesign.

### Existing tests that define the seam boundary

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  - current unit coverage is for host orchestrator startup/shutdown and invalidation
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
  - contract tests for status, toolbox, inventory, and runtime-state surfaces
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
  - shared-world startup, restart, and persisted binding behavior
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)
  - persisted runtime and trace evidence for the current orchestrator path

### Important non-seam

- [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs)
  - owns nested LLM gateway lifecycle
  - is not the member runtime launch surface
  - must not be repurposed as the source of truth for member liveness or session identity

## In Scope

- one production launch path for a world-scoped member runtime under an existing live orchestration session
- member descriptor selection, runtime realizability checks, and backend allowlist enforcement
- member participant allocation, persistence, and lifecycle transitions
- integration with already-authoritative world binding and already-defined stale-generation invalidation
- member runtime event publication and trace/state alignment
- status/reporting updates required to validate the new live producer
- targeted runtime, status, and world-restart tests

## Out of Scope

- redefining shared-world owner authority, generation assignment, or Linux reuse behavior already owned by [PLAN-03.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-03.md)
- replacement ordering, rollback, or backend atomic metadata work already owned by [07-world-replacement-ordering-rollback-atomic-metadata.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/07-world-replacement-ordering-rollback-atomic-metadata.md)
- broader authority/cutover cleanup slices
- a generalized `/v1/agents` service or new top-level agent-hub daemon
- public dispatch UX, scheduler design, or speculative multi-member productization
- repurposing toolbox or gateway lifecycle as a substitute for member session ownership
- host-scoped member product work unless strictly needed for shared helper reuse

## Blockers And Gaps

1. There is no production member selection or realizability seam.
   - `validate_orchestrator_selection(...)` and `validate_runtime_realizability(...)` are orchestrator-oriented today.
   - The doctor surface already checks whether required world-scoped member backends are allowlisted and whether a world boundary exists, but runtime startup does not yet consume that posture as a member launch preflight.

2. There is no production runtime registry shape for member launches.
   - `build_gateway_for_descriptor(...)` can build a gateway for one descriptor, but there is no shell-owned member runtime handle object, gateway cache, or launch helper that binds descriptor plus world placement plus orchestration lineage together.

3. Constructors are not a production launch seam.
   - `new_member_participant(...)` and `new_replacement_participant(...)` in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) are valuable model helpers, but they currently prove only that JSON can be shaped correctly.
   - They do not start a backend, own an event stream, persist lifecycle transitions, or produce authoritative live state by themselves.

4. `async_repl.rs` has no sibling member bootstrap path.
   - There is no `prepare_member_runtime_startup(...)`, `start_member_runtime(...)`, or `shutdown_member_runtime(...)`.
   - The only long-lived runtime control loop today is the host orchestrator control turn.

5. The status/store contract is ahead of the real runtime producer.
   - `list_live_sessions()`, invalidation helpers, and status fallback suppression already assume world-scoped member participants can be created and retired authoritatively.
   - Until production launch exists, those paths are only partially exercised through tests and fixtures.

6. Restart invalidation exists, but replacement launch does not.
   - [05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md) defines how stale members must stop being live.
   - This slice must supply the missing replacement producer that launches the successor participant on the current generation.

## Required Semantics And Invariants

### 1. Launch preconditions

A world-scoped member launch must fail closed unless all of the following are true:

- an active `OrchestrationSessionRecord` exists
- the parent live orchestrator participant exists and is authoritative-live
- the parent session has authoritative `world_id` plus `world_generation`
- the selected member inventory item resolves to `execution.scope=world`
- the selected member inventory item resolves to `protocol=uaa.agent.session`
- the selected member runtime is realizable for the current platform/build
- the selected member derived `backend_id` is allowlisted by effective policy

Missing world binding, missing live parent orchestrator, or denied backend must prevent launch. No synthetic fallback to host execution is allowed.

### 2. Member identity is session-local and world-bound

Every launched member participant must publish:

- the same `orchestration_session_id` as its parent session
- a fresh hub-assigned `participant_id`
- `role=member`
- `execution.scope=world`
- the selected member `agent_id`
- the selected member `backend_id`
- `orchestrator_participant_id=<active host orchestrator participant_id>`
- `world_id` plus `world_generation` copied from the already-authoritative parent session binding
- `ownership_mode=MemberRuntime`

For replacement after restart:

- the replacement participant must use a fresh `participant_id`
- `resumed_from_participant_id` must point at the retired participant
- the replacement participant must bind to the new active `world_generation`

### 3. Constructor output is not live authority

Creating a `member` participant record is not enough to advertise liveness.

Required rule:

- a member participant may be persisted in `allocating`, but it does not become authoritative-live until the runtime has acquired a usable backend session handle, retained ownership, and started the event/completion observers required by the runtime contract

This must match the rigor already used by the host orchestrator path.

### 4. Lifecycle parity with the host runtime

The member seam must own the same core lifecycle responsibilities already implemented for the host orchestrator:

- state transition from `allocating` to `ready`
- transition to `running` while work is active if the member protocol needs it
- heartbeat and last-event updates
- terminal `failed`, `stopped`, or `invalidated` transitions
- persisted snapshots on every meaningful lifecycle edge
- publication of runtime events that reflect the persisted participant state

This does not require the member path to duplicate the host orchestrator UX. It does require the same persistence and state-machine discipline.

### 5. Shared-world generation is consumed, not invented

This slice must not generate or reinterpret `world_generation`.

Required rule:

- member launch always consumes the current authoritative binding already persisted on the orchestration session
- member replacement after a restart consumes the latest binding after the world replacement is already committed
- stale-generation invalidation continues to use the existing state-store invalidation helper and restart semantics already defined upstream

### 6. Status and trace contracts

Once a real member runtime exists:

- live member participants must appear in `substrate agent status` through the runtime store, not only through trace replay
- world-scoped member rows must publish top-level `world_id` plus `world_generation`
- invalidated members must not be resurrected through trace fallback
- toolbox selection remains anchored to the live orchestrator session and does not switch to a member runtime

### 7. Gateway boundary remains separate

Nested gateway lifecycle and member runtime lifecycle are different contracts.

Required rule:

- `substrate_gateway` records remain nested runtime evidence
- they do not establish member participant liveness
- a launched member runtime must emit and persist its own pure-agent participant identity regardless of whether nested gateway usage occurs later

## Recommended Implementation Shape

The first landed production slice should be a bounded shell-owned launch primitive, not a scheduler.

Recommended shape:

1. Add a member-selection helper next to existing runtime validation in [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs).
2. Add a shell-owned member runtime launch helper in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) or a narrowly scoped sibling module under `crates/shell/src/execution/agent_runtime/`.
3. Reuse the existing Unified Agent API control contract already used by the host orchestrator path.
   - The member seam should launch through the same gateway/control abstraction family as the orchestrator seam.
   - Do not invent a second member-only backend protocol.
4. Make that helper accept:
   - a `RuntimeOrchestrationContext`
   - the selected member `RuntimeSelectionDescriptor`
   - the live parent orchestrator participant identity
   - the active authoritative world binding
5. Make the helper return a member runtime handle object that owns:
   - participant record mutex or snapshot path
   - cancel handle
   - event stream task
   - completion observer task
   - enough metadata to support restart replacement and explicit shutdown

This keeps the seam concrete without forcing a speculative agent-hub service or public dispatch API in the same slice.

## Exact Code Areas

### Primary implementation surfaces

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  - add the production member runtime bootstrap and teardown seam
  - wire it to the existing world-binding and restart flow
  - ensure replacement launches consume the already-updated parent binding

- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
  - add member-specific realizability and capability validation
  - keep orchestrator validation separate from member validation instead of overloading one function

- [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs)
  - extend gateway/registry creation so production member launch can resolve a backend from a member descriptor, not only the selected host orchestrator

- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
  - keep `member` constructor and replacement helpers authoritative for record shape
  - add any small lifecycle helpers needed by the new runtime seam
  - remove the practical dead-code status by giving these constructors a real production caller

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
  - add any lookup helpers needed by the member launch path
  - preserve session-centric persistence as the live authority boundary
  - keep invalidation session-local and generation-aware

### Consumer validation surfaces

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
  - confirm live status projections render real member participants correctly
  - confirm toolbox remains orchestrator-local
  - keep doctor checks aligned with the real launch preconditions

### Context surfaces that should not absorb this slice

- [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs)
  - do not use as the member runtime lifecycle owner
- [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/src/pty.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs)
  - these remain upstream authority/provider seams; this SOW consumes them rather than redesigning them

## Testing Requirements

At minimum, land tests that prove real production member launch rather than fixture-only shape.

### Runtime/unit coverage

- add `async_repl.rs` unit coverage for:
  - successful member bootstrap under an active orchestrator session
  - failed member bootstrap when no world binding exists
  - failed member bootstrap when the parent orchestrator is missing or not live
  - replacement member bootstrap after generation change using fresh lineage
  - member shutdown or unexpected completion persisting terminal state correctly

### Store/status integration coverage

- extend [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs) so that:
  - a real live member participant shows in `agent status`
  - world-scoped member rows publish `world_id` plus `world_generation`
  - invalidated member tombstones still suppress trace fallback
  - toolbox still resolves only through the orchestrator session

### World/restart integration coverage

- extend [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) so that:
  - a startup path with shared-world binding can also launch a member runtime
  - world restart produces a replacement member bound to the new generation
  - old-generation members are not left live after replacement
  - fail-closed restart posture refuses new member launch when replacement is not ready

### Trace persistence coverage

- extend [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs) so that:
  - member runtime events persist with pure-agent identity
  - member rows preserve world binding and participant lineage
  - member terminal transitions remain auditable without becoming authoritative-live again

### Recommended verification commands

```bash
cargo test -p shell agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell repl_world_first_routing_v1 -- --nocapture
cargo test -p shell agent_hub_trace_persistence -- --nocapture
cargo test -p shell async_repl -- --nocapture
```

## Acceptance Criteria

- production shell code can launch at least one world-scoped member runtime under a live orchestration session
- launch requires a live host orchestrator participant plus authoritative parent world binding
- the launched member writes an authoritative participant record under the session-centric runtime store
- member liveness depends on a real runtime ownership boundary, not only a constructor call
- `substrate agent status` can surface the live member from runtime state with `world_id` plus `world_generation`
- invalidated old-generation members remain non-live after replacement
- toolbox behavior stays orchestrator-scoped and unchanged except for consuming the same parent session authority
- no part of the implementation redefines world ownership, restart ordering, or backend rollback semantics already owned by earlier slices

## Recommended Staged Execution Order

1. Member preflight and descriptor resolution.
   - Add member-specific validation for scope, protocol, runtime realizability, and policy allowlisting.

2. Runtime launch abstraction.
   - Introduce the shell-owned member runtime handle and gateway/descriptor plumbing needed to start a real backend.

3. Participant bootstrap and persistence.
   - Allocate the member participant, persist `allocating`, bind it to the active orchestrator and world generation, and transition to authoritative-live only after ownership is retained.

4. Lifecycle and replacement integration.
   - Add shutdown, unexpected-exit, and restart-replacement handling.
   - Consume existing invalidation helpers rather than redefining cutover semantics here.

5. Status and trace validation.
   - Prove that live status, tombstone suppression, and trace persistence all work with a real member producer.

6. Integration hardening.
   - Add world-restart coverage, replacement lineage coverage, and fail-closed launch coverage.

## Main Contract Decisions

- The repo currently launches only the host orchestrator runtime in production; member constructors and store helpers do not count as a real launch seam.
- The first required capability expansion is one bounded shell-owned world-member launch primitive, not a generalized orchestration service.
- Member launch must consume already-authoritative world/session state and must stay isolated from authority, rollback, and cleanup/cutover slices.
