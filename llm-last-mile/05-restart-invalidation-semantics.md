# SOW: Restart Invalidation Semantics for Live State

## Objective

Define and land the live-state and registry semantics that make shared-world restarts authoritative for world-scoped member sessions. When `world_generation` changes for an `orchestration_session_id`, every prior-generation world-scoped member session must stop being live in the registry immediately, and no status or toolbox surface may resurrect those stale sessions from trace history.

This prerequisite exists to close the gap between:

- the current authoritative-live manifest model in [`crates/shell/src/execution/agent_runtime/session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) and [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs),
- the current restart signaling in [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs),
- and the successor contract already written in [`docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md).

## Why This Is Needed

The repo already has most of the pieces, but they are not yet locked together for restart invalidation:

- `AgentRuntimeSessionState` already includes `Restarting` and `Invalidated`, and `is_authoritative_live()` already excludes invalidated manifests.
  - Source: [`crates/shell/src/execution/agent_runtime/session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
- `AgentRuntimeStateStore::list_live_manifests()` already treats persisted manifests as the authoritative live source and explicitly separates them from historical trace.
  - Source: [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- REPL world drift already increments `world_generation` and emits `world_restarted` or `world_restart_required`.
  - Source: [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- `substrate agent status` already merges authoritative live manifests with historical trace, preferring live manifests when present.
  - Source: [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- The successor draft already states that a world restart invalidates old world-scoped handles and that trace is historical, not authoritative live state.
  - Sources:
    - [`docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md)
    - [`docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md)
    - [`docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/policy-spec.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/policy-spec.md)

The missing prerequisite is the live-state rule that ties generation transitions to registry invalidation. Without that rule, a prior-generation member can remain selectable or be re-surfaced from trace fallback after the shared world has moved on.

## Current Repo Seams

### Runtime state and persistence

- [`crates/shell/src/execution/agent_runtime/session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
  - Owns `AgentRuntimeSessionManifest`, `world_id`, `world_generation`, lifecycle states, and authoritative-live checks.
- [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
  - Owns persistence under `~/.substrate/run/agent-hub/handles/*.json`.
  - Currently lists live manifests but does not expose generation-aware invalidation helpers or an active-generation index.

### Status and operator surfaces

- [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
  - `build_status_report()` merges trace-derived pure-agent session projections with live manifests.
  - `live_sessions_by_agent` is keyed only by `agent_id`, which is too weak for generation-aware member invalidation.
  - Trace fallback is suppressed only when a live manifest is present; there is no explicit stale-generation tombstone rule today.
- Same file also owns toolbox status/env behavior via `find_live_orchestrator()`, which already demonstrates the intended authoritative-live precedence for the orchestrator path.

### Restart signaling and world drift

- [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  - Owns `classify_world_restart_reason()`, `emit_world_restarted_alert()`, `build_world_restart_required_alert()`, and restart sequencing.
  - Already increments `world_generation` on auto-restart.
  - Already uses `Invalidated` for shell-owned runtime ownership loss, which is the closest existing state precedent.

### Existing contract and tests that this prerequisite must satisfy

- Draft contract:
  - [`docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md)
  - [`docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md)
- Gap framing:
  - [`AGENT_ORCHESTRATION_GAP_MATRIX.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
- Existing test anchors:
  - [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
  - [`crates/shell/tests/repl_world_first_routing_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

## In Scope

- Define the authoritative invalidation rule for world-scoped member session manifests when shared-world generation changes.
- Define the required registry semantics for marking old-generation members non-live.
- Define how `substrate agent status` must select, suppress, or omit stale member sessions.
- Define how trace/history must remain auditable without being allowed to re-authorize stale sessions.
- Define sequencing expectations for restart commit ordering.
- Define concrete acceptance criteria and tests.

## Out of Scope

- Shipping the full multi-member runtime if it does not already exist.
- Redesigning trace schema wholesale.
- Changing orchestrator selection rules.
- Replacing the current manifest storage layout under `~/.substrate/run/agent-hub/handles`.
- Adding a new top-level `crates/agent-hub` crate as part of this prerequisite.

## Observed Blockers

1. There is no explicit active-generation registry rule today.
   - `AgentRuntimeStateStore` can list manifests, but it does not provide helpers such as “invalidate all live world members from generation N for orchestration session X” or “what is the current active generation for orchestration session X”.

2. Status fallback is live-biased but not invalidation-aware.
   - `build_status_report()` in [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) falls back to trace when no live manifest wins.
   - That is correct for historical visibility, but insufficient once a session is explicitly invalidated due to restart.

3. Current manifest suppression keys are too coarse.
   - The status surface suppresses trace duplicates by `agent_id`, not by `(orchestration_session_id, agent_id, execution.scope, world_generation)`.
   - That creates room for stale same-agent rows to survive generation transitions incorrectly.

4. The current production runtime path is orchestrator-first.
   - The repo has real shell-owned orchestrator manifest plumbing today.
   - The world-scoped member contract exists primarily in the successor draft and tests, so this prerequisite must define semantics that future member producers will follow from day one.

5. There is no explicit “tombstone beats trace” rule for prior-generation members.
   - Toolbox env already follows “authoritative live manifests only” for the orchestrator path.
   - Member status needs the same fail-closed posture once generation invalidation exists.

## Proposed Semantics

### 1. Authoritative identity and generation boundary

For world-scoped member sessions, authoritative live identity is not just `agent_id`. It is:

- `orchestration_session_id`
- `agent_id`
- `role=member`
- `execution.scope=world`
- `world_generation`
- `session_handle_id`

Rule:

- Exactly one `world_generation` is active for a given `orchestration_session_id`.
- A world-scoped member handle from any non-active generation is not live, even if its persisted manifest still exists on disk.

### 2. Generation change is a hard invalidation barrier

When the shared world for an `orchestration_session_id` advances from generation `G` to `G+1`:

- every live world-scoped member manifest bound to generation `G` must be transitioned out of the live set,
- every such manifest must end in `state=invalidated` unless a stricter terminal failure state is required,
- every replacement handle must receive a fresh `session_handle_id`,
- every replacement handle must publish:
  - the same `orchestration_session_id`,
  - the same `agent_id`,
  - the same `backend_id`,
  - the same `role`,
  - the same `protocol`,
  - `world_generation=G+1`,
  - `resumed_from_session_handle_id=<old_handle_id>`.

Non-negotiable invariant:

- After the generation-change commit completes, `list_live_manifests()` must return zero world-scoped member manifests from generation `G` for that orchestration session.

### 3. Commit ordering

Restart handling must use commit ordering that never leaves the registry in an ambiguous live state.

Required sequence:

1. Detect drift or explicit restart need.
2. Allocate or prepare the replacement shared world.
3. Persist the new active world binding for the orchestration session.
4. Persist replacement member manifests for the new generation.
5. Mark prior-generation member manifests `invalidated`.
6. Only after steps 3-5 succeed, emit operator-facing success reporting such as `world_restarted`.

Implementation rule:

- The alert is historical reporting.
- The manifest/registry update is the live-state authority.
- If those disagree, the manifests win for current-state reads.

### 4. Fail-closed posture when replacement is not ready

If restart is required but replacement member handles are not ready yet:

- prior-generation world-scoped member handles must still leave the authoritative live set,
- status must not continue to show them as active,
- trace history must not resurrect them,
- policy evaluation for new world-scoped work must fail closed with a reason equivalent to the current successor draft:
  - “prior handle is invalidated and no replacement handle is ready”.

This applies both to:

- auto-restart windows where replacement startup failed or is incomplete,
- explicit fail-closed restart posture where new work must wait for a deliberate restart path.

### 5. Status and reporting semantics

`substrate agent status` remains a live-session surface, not a historical ledger.

Required behavior:

- `sessions[]` contains only authoritative-live sessions.
- Invalidated prior-generation member handles do not appear in `sessions[]`.
- A prior-generation trace status row must not be selected if the registry knows that the corresponding member handle was invalidated or superseded.
- If there is no live replacement handle yet, the session should be absent from live status rather than presented as still running.

Selection/suppression rule:

- Live manifest precedence must become generation-aware.
- Trace fallback is permitted only when there is no conflicting live or invalidated registry record for the same `(orchestration_session_id, agent_id, execution.scope)`.

Reporting rule:

- Restart history belongs in alert events and trace.
- Current liveness belongs in manifests and status output.
- Do not keep stale sessions visible in `status` merely to make restart history easier to read.

### 6. Trace vs audit distinction

The repo already states this separation in [`docs/TRACE.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md), and this prerequisite must preserve it:

- `~/.substrate/run/agent-hub/handles/*.json` is the authoritative live-state registry.
- `~/.substrate/trace.jsonl` is append-only audit/history.

Required distinction:

- Trace records may show:
  - old pure-agent status rows,
  - `world_restarted`,
  - `world_restart_required`,
  - nested gateway records,
  - historical world generations.
- Trace records may not be used to infer that a prior-generation world-scoped member is still live after the registry has invalidated it.

Practical consequence:

- The system may keep old-generation trace rows forever.
- The system may not select them into live status/toolbox surfaces once restart invalidation has occurred.

## Proposed Implementation Surfaces

### Primary code surfaces

- [`crates/shell/src/execution/agent_runtime/session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
  - Add helper semantics for generation supersession and invalidation transitions.
- [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
  - Add registry helpers for:
    - listing authoritative manifests with generation grouping,
    - invalidating prior-generation world-scoped member manifests,
    - suppressing stale live lookups when a newer generation exists.
- [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
  - Make status projection generation-aware.
  - Ensure invalidated/superseded registry state suppresses trace fallback for stale members.
- [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  - Ensure restart alert emission happens after live-state commit.
  - Provide the future restart handoff contract member runtimes will follow.

### Test and contract surfaces

- [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
  - Add or extend tests for:
    - stale prior-generation member suppression,
    - invalidated manifest beats trace fallback,
    - replacement handle publication rules.
- [`crates/shell/tests/repl_world_first_routing_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
  - Extend restart coverage to verify live-state commit and status/reporting parity.
- Contract docs to update when implementation lands:
  - [`docs/TRACE.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
  - [`docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md)
  - [`docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md)

## Sequencing

### Phase 1: Lock registry invariants

- Define the active-generation invariant in the runtime session/store layer.
- Add store APIs that can answer:
  - active world generation for an orchestration session,
  - live world-scoped members in that generation,
  - prior-generation world-scoped members that must be invalidated.

### Phase 2: Implement invalidation transition

- Add one bounded invalidation path that:
  - identifies prior-generation world-scoped member handles,
  - marks them `invalidated`,
  - stamps transition time and termination reason,
  - preserves them on disk for audit while removing them from authoritative live reads.

### Phase 3: Wire restart commit order

- Update restart handling so generation advancement, replacement-handle registration, and old-handle invalidation are committed before success alert publication.
- Ensure fail-closed posture does not leave stale prior-generation handles visible as live.

### Phase 4: Update status/reporting selection

- Change `substrate agent status` selection logic to suppress trace fallback when an invalidated or superseded registry record exists for the same world-scoped member identity.
- Preserve current omission rules for nested gateway-backed rows and trace/history separation.

### Phase 5: Add regression coverage

- Add tests that explicitly simulate:
  - generation `N` member manifest,
  - generation `N+1` restart,
  - prior-generation trace rows arriving after the restart,
  - no replacement handle yet,
  - replacement handle ready,
  - fail-closed restart posture.

## Acceptance Criteria

1. For a single `orchestration_session_id`, there is never more than one authoritative-live generation for world-scoped member sessions.
2. After a restart from generation `G` to `G+1`, no generation-`G` member handle is returned by authoritative live registry queries.
3. `substrate agent status --json` never shows a prior-generation world-scoped member as live once the registry has invalidated or superseded it.
4. Historical trace rows from generation `G` cannot reappear in status after generation `G+1` is committed.
5. Replacement handles use a new `session_handle_id` and preserve `resumed_from_session_handle_id=<old_handle_id>`.
6. `world_restarted` alert records report the new active generation at the top level and historical values only in `data.previous_*` fields.
7. If replacement handles are not ready, new world-scoped work fails closed and stale generation rows do not remain live.
8. Host-scoped orchestrator sessions are not incorrectly invalidated by member-generation rollover.
9. Existing trace/audit behavior remains additive and historical; no old trace is deleted to satisfy the live-state rule.

## Validation and Testing Suggestions

### Unit-level

- Add store tests around:
  - invalidating all prior-generation member manifests for one orchestration session,
  - ensuring host-scoped orchestrator manifests are untouched,
  - ensuring `list_live_manifests()` excludes invalidated generation `G` members once generation `G+1` is active.

### Status/reporting tests

- Extend [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs) with fixtures that write:
  - a live generation-`7` member manifest,
  - a generation-`8` replacement manifest,
  - an older generation-`7` trace row,
  - and verify only generation `8` is surfaced.
- Add a fixture where generation `7` is invalidated and generation `8` is not ready yet.
  - Expected: member absent from `sessions[]`; no trace resurrection.

### Restart-path integration tests

- Extend [`crates/shell/tests/repl_world_first_routing_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) to assert:
  - restart alert exists,
  - registry/live manifest state reflects the new generation before the alert is observed by readers,
  - stale-generation handles are no longer live.

### Manual validation

- Run `substrate agent status --json` before and after a forced restart/drift scenario and verify:
  - member `world_generation` advances,
  - old-generation members disappear from live status,
  - `trace.jsonl` still contains old-generation history plus restart alerts.
- If member-runtime producer code lands in the same implementation wave, also validate against:
  - `substrate agent doctor --json`
  - `substrate world doctor --json`
  - any targeted `cargo test -p shell -- --nocapture` subset covering agent successor and REPL restart flows.

## Risks and Open Questions

1. Registry shape vs minimal change
   - The smallest implementation may use manifest sweeps only.
   - The cleaner implementation may add an explicit active-world-generation index for each orchestration session.
   - Decision needed: do we keep this prerequisite manifest-only, or introduce a lightweight session-level registry file now?

2. Status fallback suppression granularity
   - Current suppression by `agent_id` is too coarse.
   - Decision needed: suppress by `(orchestration_session_id, agent_id)` or by full `(orchestration_session_id, agent_id, world_generation)` plus scope.
   - Recommendation: use `(orchestration_session_id, agent_id, execution.scope)` for stale suppression and keep `world_generation` as the active-generation discriminator.

3. Restart window semantics
   - The draft contract implies replacement handles should exist before more work dispatches.
   - Decision needed: whether status should expose an additive “restarting” placeholder row, or simply omit unavailable member sessions until replacement is ready.
   - Recommendation: keep `status` live-only and omit unavailable sessions; use alerts for restart history.

4. Event correlation depth
   - Trace `agent_event` rows do not currently carry `session_handle_id`.
   - This prerequisite does not require widening trace schema, but debugging could be easier if handle id becomes additive later.

5. Fail-closed drift timing
   - The successor draft allows invalidated or failed old handles when restart cannot proceed.
   - Decision needed: whether stale members are invalidated immediately on drift detection or only once an explicit restart path begins.
   - Recommendation: invalidate as soon as the system knows further work on that handle must not continue.

## Deliverable

A generation-aware live-state contract and implementation path that makes shared-world restart semantics deterministic for the registry, keeps status/toolbox surfaces fail-closed, and preserves trace as historical audit rather than live authorization.
