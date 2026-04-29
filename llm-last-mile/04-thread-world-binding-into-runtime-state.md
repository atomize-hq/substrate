# SOW: Thread `world_id` and `world_generation` Into Runtime State

## Objective

Make the shell-owned agent runtime state authoritative for world binding by persisting the active
`world_id` and `world_generation` in live runtime state, not only in trace/event records.

This prerequisite is complete when the runtime manifest/state store tracks the currently bound
world identity across initial startup, drift-triggered restarts, and fail-closed
restart-required cases, and when downstream live-state consumers can trust the persisted runtime
record instead of reconstructing world binding from trace history.

## Why This Exists

The repo already has two parallel truths:

- Live runtime authority for orchestrator session ownership:
  - `crates/shell/src/execution/agent_runtime/session.rs`
  - `crates/shell/src/execution/agent_runtime/state_store.rs`
  - `crates/shell/src/execution/agents_cmd.rs`
- World identity authority for the active shared world:
  - in-memory `WorldSession` state in `crates/shell/src/repl/async_repl.rs`
  - persistent-session ready frames in `crates/shell/src/execution/repl_persistent_session.rs`
  - trace/event records written via `crates/shell/src/execution/agent_events.rs` and
    `crates/shell/src/execution/routing/telemetry.rs`

Today those truths are not joined. The runtime manifest schema already has top-level
`world_id` and `world_generation` fields, but the shell runtime never populates them.

## Current State

### Runtime manifest and state store

- `AgentRuntimeSessionHandle` in
  `crates/shell/src/execution/agent_runtime/session.rs` already defines:
  - `world_id: Option<String>`
  - `world_generation: Option<u64>`
- `AgentRuntimeSessionManifest::new(...)` initializes both to `None`.
- No existing manifest mutation path sets or updates those fields after startup.
- `AgentRuntimeStateStore::persist_manifest(...)` writes the full manifest JSON, but the lease
  sidecar in `persist_lease(...)` does not currently include world binding.

### World binding source of truth

- `WorldSession` in `crates/shell/src/repl/async_repl.rs` carries:
  - `world_id`
  - `world_generation`
  - restart sequencing
- `open_world_session(...)` gets `ready.world_id` from the persistent-session protocol.
- `start_world_session(...)` seeds `world_generation: 0`.
- `restart_world_session(...)` increments `world_generation` and emits a `world_restarted`
  alert.
- `handle_detected_world_drift(...)` emits `world_restart_required` alerts in fail-closed mode.

### Status and authorization surfaces

- `crates/shell/src/execution/agents_cmd.rs` uses
  `AgentRuntimeStateStore::list_live_manifests()` and `find_live_orchestrator(...)` as the
  authoritative live-state boundary for:
  - `substrate agent status`
  - `substrate agent toolbox status`
  - `substrate agent toolbox env`
- `live_manifest_status_session(...)` already projects `manifest.handle.world_id` and
  `manifest.handle.world_generation` if present.
- In practice those values are always absent for live manifests because the runtime never writes
  them.
- Trace fallback still carries world binding for world-scoped pure-agent projections because
  `AgentEvent` already supports top-level `world_id` and `world_generation`.

### Event producers

- Runtime/orchestrator events are built in `crates/shell/src/repl/async_repl.rs`:
  - `build_runtime_message_event(...)`
  - `translate_wrapper_event(...)`
  - `emit_world_restarted_alert(...)`
  - `build_world_restart_required_alert(...)`
- Events are published via `crates/shell/src/execution/agent_events.rs`.
- They are persisted to trace via
  `crates/shell/src/execution/routing/telemetry.rs::append_agent_event_to_trace(...)`.

## Current Blockers

1. `AgentRuntimeSessionManifest` has world-binding fields but no mutation API, so runtime state
   cannot reflect the active world even though the schema allows it.

2. The only durable place that tracks world generation changes today is trace/event history.
   `WorldSession` knows the active generation in memory, but live runtime manifests do not.

3. Drift restart alerts (`world_restarted`, `world_restart_required`) publish top-level
   world identity, but they do not synchronize the runtime manifest before or with the event.
   A consumer reading authoritative live state immediately after the alert still sees no binding.

4. The lease sidecar omits world binding entirely, so auxiliary readers that avoid the full
   manifest cannot observe the active world association.

5. The selected orchestrator is intentionally host-scoped today:
   `validate_orchestrator_selection(...)` in
   `crates/shell/src/execution/agent_runtime/validator.rs` rejects non-host orchestrators.
   This prerequisite must therefore persist world binding without accidentally redefining the
   existing host-scoped operator-facing status contract.

## Scope

This prerequisite is limited to the shell runtime state path.

### In scope

- Persisting active world binding into the shell runtime manifest and lease store.
- Seeding that binding from the active `WorldSession` at orchestrator runtime startup.
- Updating that binding when the world restarts and when a fail-closed restart-required posture
  is detected.
- Making runtime event producers consume the persisted binding so runtime state and emitted
  events stay aligned.
- Preserving current `agent status` and toolbox live-authority behavior while upgrading the
  underlying runtime truth.

### Out of scope

- Relaxing host-only orchestrator selection.
- Adding a general multi-member live registry or `/v1/agents` service.
- Changing `substrate world gateway status --json` or the gateway status schema contracts.
- Redefining whether host-scoped selected orchestrator rows should display world binding in
  `substrate agent status --json`.

## Exact Change Areas

### 1. Runtime manifest model

Primary files:

- `crates/shell/src/execution/agent_runtime/session.rs`
- `crates/shell/src/execution/agent_runtime/state_store.rs`

Required changes:

- Add an explicit manifest-level mutation API for world binding.
  - Recommended shape: `set_world_binding(world_id, world_generation)`.
  - Optional companion: `clear_world_binding()`.
- Keep `world_id` and `world_generation` top-level on `AgentRuntimeSessionHandle`.
  Do not introduce a new nested object unless there is a strong serialization need; existing
  readers already know the top-level fields.
- Extend the lease payload written by `persist_lease(...)` to include:
  - `world_id`
  - `world_generation`
- Preserve the existing manifest and lease write path through `persist_manifest(...)` so both
  artifacts stay in sync.

### 2. Runtime bootstrap and world-session binding

Primary file:

- `crates/shell/src/repl/async_repl.rs`

Required changes:

- Thread the current `WorldSession` binding into `start_host_orchestrator_runtime(...)`.
  - `run_async_repl(...)` already starts the world session before starting the host orchestrator
    runtime; use that ordering.
- Seed the newly created runtime manifest with the active world binding before the runtime is
  considered ready/live.
- Persist the bound manifest before emitting runtime events that should reflect live world
  attachment.

Recommended implementation approach:

- Introduce a small helper in `async_repl.rs` that applies the `WorldSession` binding to the
  in-memory manifest and persists it through `AgentRuntimeStateStore`.
- Use that helper:
  - once during initial orchestrator runtime bootstrap if `world_session.is_some()`
  - again after any world-session replacement caused by `restart_world_session(...)`
  - again on fail-closed restart-required detection so the manifest keeps the last authoritative
    binding even when progress stops

### 3. World restart / drift synchronization

Primary file:

- `crates/shell/src/repl/async_repl.rs`

Required changes:

- Synchronize runtime manifest state whenever `WorldSession` changes.
- The ordering requirement is important:
  - update manifest binding
  - persist manifest
  - then emit the corresponding alert/event
- Ensure `world_generation` remains consistent with existing REPL semantics:
  - startup generation: `0`
  - each restart: `previous + 1`

Specific seams to touch:

- `start_world_session(...)`
- `restart_world_session(...)`
- `handle_detected_world_drift(...)`
- any call sites that replace `world_session` after drift handling

### 4. Event producers

Primary files:

- `crates/shell/src/repl/async_repl.rs`
- `crates/shell/src/execution/agent_events.rs`
- `crates/shell/src/execution/routing/telemetry.rs`
- `crates/common/src/agent_events.rs`

Required changes:

- Runtime-originated pure-agent events should derive world binding from the runtime manifest once
  it is available, not from ad hoc call-site copies.
- World lifecycle alerts should remain top-level-field compatible with the existing trace/event
  contract, but the persisted runtime state must be updated first.
- No event schema expansion is required for this prerequisite; use the existing top-level
  `world_id` and `world_generation` fields already supported by `AgentEvent`.

Recommended implementation detail:

- Update `build_runtime_message_event(...)` and `translate_wrapper_event(...)` to stamp
  `world_id/world_generation` from the manifest when present.
- Keep nested gateway-backed records unchanged; this prerequisite is about pure-agent runtime
  state, not the gateway tuple boundary.

### 5. Status and live-state consumers

Primary file:

- `crates/shell/src/execution/agents_cmd.rs`

Required changes:

- Preserve `AgentRuntimeStateStore` as the live authority boundary for toolbox and selected
  orchestrator status.
- Ensure any live-manifest projections read the stored world binding when applicable.
- Do not redefine host-scoped selected orchestrator output just because internal runtime state now
  knows the bound world.

Important contract guard:

- `agent status --json` currently has tests asserting that host-scoped selected orchestrator rows
  omit `world_id` and `world_generation`.
- This prerequisite should keep that behavior unless a later contract change explicitly decides
  otherwise.
- Internal state may know the binding even if the selected host-scoped status row continues to
  suppress it.

### 6. Precedent and alignment with existing runtime-state patterns

Reference file:

- `crates/world-agent/src/gateway_runtime.rs`

Why it matters:

- The gateway runtime manager already persists runtime manifests keyed by runtime/world identity
  and updates those manifests as lifecycle state changes.
- The shell agent runtime should follow the same principle: if world binding affects control-plane
  truth, it belongs in runtime state, not only in trace reconstruction.

This file is not necessarily in scope for code changes in this prerequisite, but it should be used
as the consistency model.

## Sequencing

1. Add manifest mutation support in `agent_runtime/session.rs` and include world binding in the
   lease payload in `state_store.rs`.

2. Thread optional world binding into `start_host_orchestrator_runtime(...)` from
   `run_async_repl(...)`.

3. Persist initial world binding onto the runtime manifest before the runtime is advertised as a
   live authoritative session.

4. Add a single synchronization helper in `async_repl.rs` for world-binding changes and call it
   from every world-session replacement path.

5. Update runtime event builders to read binding from the manifest instead of duplicating
   world-binding logic at each event site.

6. Re-run status/toolbox projections against the new persisted manifest state and confirm current
   host-scoped output contracts still hold.

7. Add/adjust tests for startup, restart, fail-closed drift, and live-manifest consumption.

## Acceptance Criteria

### Runtime state

- A live runtime manifest under `~/.substrate/run/agent-hub/handles/*.json` persists the active
  `world_id` and `world_generation` whenever the REPL has an active world session.
- The paired `*.lease` file also includes the same world-binding fields.
- On world restart, the live manifest updates from the old binding to the new binding with the
  incremented generation.
- On fail-closed restart-required drift, the manifest still records the current authoritative
  binding instead of dropping back to `None`.

### Event/state parity

- Runtime-originated pure-agent events and world lifecycle alerts remain consistent with the
  manifest’s binding.
- There is no path where a newly emitted world lifecycle alert reflects a new binding while the
  authoritative runtime manifest still reflects the old binding or no binding.

### Status/toolbox behavior

- `substrate agent toolbox env` and `substrate agent toolbox status` continue using authoritative
  live manifests, not trace fallback, for active-session authorization.
- `substrate agent status --json` continues to prefer live manifests over trace fallback for the
  selected orchestrator.
- Existing host-scoped selected orchestrator rows do not regress by unexpectedly publishing
  `world_id/world_generation` unless a separate contract change intentionally does that.

### Failure posture

- Binding persistence for authoritative state changes is fail-closed.
- If the runtime cannot persist a new authoritative world binding, it must not silently continue
  with stale live state.

## Validation and Testing Suggestions

### Unit / focused tests

- `crates/shell/src/repl/async_repl.rs`
  - add or extend tests around `start_host_orchestrator_runtime(...)`
  - verify manifest world binding is seeded when a world session exists
  - verify binding updates on restart paths
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - verify lease payload includes `world_id` and `world_generation`
- `crates/shell/src/execution/agent_runtime/session.rs`
  - verify binding mutation helpers preserve the rest of manifest state

### REPL/world integration tests

- `crates/shell/tests/repl_world_first_routing_v1.rs`
  - extend existing `world_restarted` and `world_restart_required` coverage to also inspect the
    authoritative runtime manifest after the event
  - verify generation increments stay aligned between trace and manifest

### Status/toolbox contract tests

- `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
  - add fixture coverage where a live manifest carries world binding
  - confirm live-manifest authority is preserved
  - confirm selected host-scoped orchestrator rows still omit world fields
  - confirm trace fallback does not override live manifest binding

### Suggested command set

At minimum, the implementer should expect to run:

```bash
cargo test -p substrate-shell start_host_orchestrator_runtime -- --nocapture
cargo test -p substrate-shell repl_world_first_routing_v1 -- --nocapture
cargo test -p substrate-shell agent_successor_contract_ahcsitc0 -- --nocapture
```

If the crate name differs in the local workspace layout, use the existing shell crate/package name
that owns:

- `crates/shell/src/repl/async_repl.rs`
- `crates/shell/tests/repl_world_first_routing_v1.rs`
- `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

## Risks

1. Silent state/event drift
   - If binding persistence stays best-effort while alerts remain fire-and-forget, the repo will
     continue to have conflicting sources of truth.

2. Contract regression on selected orchestrator status
   - `agent status` currently treats the selected orchestrator as host-scoped by contract.
     Accidentally surfacing stored world binding there would be a behavior change.

3. Partial updates across manifest and lease files
   - The manifest and lease sidecar must move together, or downstream consumers will see split
     truth depending on which file they read.

4. Startup ordering bugs
   - The world session starts before the orchestrator runtime today. Any future refactor that
     changes that order could reintroduce `None` world binding unless the dependency is explicit.

5. Future member-session expansion
   - This prerequisite should not hard-code assumptions that only the selected host orchestrator
     can have a runtime manifest forever. The binding plumbing should be reusable by later
     world-scoped member runtime records.

## Open Questions

1. Should host-scoped selected orchestrator status rows continue suppressing world binding even
   after the manifest stores it?
   - Recommendation for this prerequisite: yes, preserve current status behavior and treat the
     stored binding as internal/live-authority data only.

2. Should binding persistence failures during mid-session world restart invalidate the orchestrator
   runtime, terminate the REPL, or both?
   - Recommendation: fail closed rather than continue with stale authoritative state.

3. Is the lease sidecar consumed anywhere outside the current shell crate?
   - If yes, verify those readers tolerate the additive fields without further coordination.

4. Should future world-scoped member sessions reuse `AgentRuntimeSessionManifest`, or should this
   prerequisite stay strictly orchestrator-only?
   - Recommendation: implement the binding mutation API generically enough that member-session
     follow-on work can reuse it.

## Deliverable

Implement the runtime-state plumbing so that world binding is persisted in live shell runtime
state and remains aligned with event emission, while preserving the current host-scoped selected
orchestrator status contract.
