# SOW: Replace Process-Global Orchestration Identity with a Substrate-Owned Session Identity

## Objective

Replace the current process-global orchestration identity with a first-class Substrate-owned session identity that is created, persisted, resolved, and invalidated by the shell runtime. The new identity must become the authoritative source for:

- `orchestration_session_id` on pure-agent and toolbox-related trace records
- live orchestrator discovery for `substrate agent status` and `substrate agent toolbox *`
- shared-world correlation for world-scoped member agents
- lifecycle transitions across bootstrap, steady state, invalidation, shutdown, and future resume/fork work

This work is a prerequisite for reliable multi-agent orchestration. It is not a generic refactor of all session concepts in the repo.

## Problem Statement

Substrate already has two real runtime concepts:

- shell trace session identity via `ShellConfig.session_id`
- persisted orchestrator runtime manifests via `crates/shell/src/execution/agent_runtime/session.rs` and `crates/shell/src/execution/agent_runtime/state_store.rs`

But `orchestration_session_id` is still sourced from a process-global helper in `crates/shell/src/execution/agent_events.rs`:

- `static ORCHESTRATION_SESSION_ID: OnceLock<String> = OnceLock::new();`
- `orchestration_session_id() -> String` lazily returns one UUID per process

That is materially weaker than the surrounding runtime model:

- it is not persisted as a first-class session record
- it is not explicitly created at REPL/session bootstrap time
- it is not invalidated when the authoritative orchestrator runtime becomes invalid
- it can be emitted by code paths that are not actually attached to a live orchestrator session
- it conflates "this shell process has emitted agent-ish events" with "Substrate owns a live orchestration session"

The result is that the codebase already behaves as if a real session exists, while the actual identity source remains a convenience global.

## Current Repo Seams to Ground This Work

### Current identity and event seam

- `crates/shell/src/execution/agent_events.rs`
  - owns the process-global `ORCHESTRATION_SESSION_ID`
  - is used by shell/demo/runtime/world stream event producers
- `crates/common/src/agent_events.rs`
  - defines the canonical `AgentEvent` envelope
  - requires `orchestration_session_id` for agent-hub records
- `crates/shell/src/execution/routing/telemetry.rs`
  - persists `AgentEvent` rows into canonical trace and attaches the shell trace `session_id`

### Current authoritative-live runtime seam

- `crates/shell/src/execution/agent_runtime/session.rs`
  - defines `AgentRuntimeSessionManifest`
  - already stores `orchestration_session_id`, `session_handle_id`, runtime state, ownership flags, optional `world_id`, and `world_generation`
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - persists manifests under `~/.substrate/run/agent-hub/handles/*.json`
  - decides whether a manifest is authoritative-live
- `crates/shell/src/repl/async_repl.rs`
  - boots the shell-owned orchestrator runtime
  - currently creates an `AgentRuntimeSessionManifest` using `orchestration_session_id()`
  - treats retained attached-control ownership as the authoritative liveness boundary

### Current operator/status/toolbox seam

- `crates/shell/src/execution/agents_cmd.rs`
  - `substrate agent status` projects runtime state from trace plus live manifests
  - `substrate agent toolbox status|env` resolves the active toolbox endpoint from the selected live orchestrator manifest
  - UDS endpoint path is derived from `~/.substrate/run/agent-toolbox/<orchestration_session_id>.sock`
- `docs/TRACE.md`
  - already documents `orchestration_session_id` as a canonical cross-feature join key
  - already documents live manifest storage under `~/.substrate/run/agent-hub/handles/*.json`

### Current world seam

- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
  - emits stream chunks using `crate::execution::agent_events::orchestration_session_id()`
- `crates/shell/src/repl/async_repl.rs`
  - already emits world lifecycle alerts keyed by orchestration session and run id
- `docs/WORLD.md`
  - treats world reuse and world identity as operator-visible execution facts

## Why the Current Model is Blocked

1. The orchestration identity is owned by process lifetime, not session lifetime.
   A single long-lived shell process can outlive multiple orchestrator runtimes, invalidations, or future resume flows, but the current `OnceLock` never rotates.

2. The identity is not independently persisted.
   The only persisted objects are per-handle manifests. There is no authoritative top-level orchestration session record that says when a session began, why it ended, which shell trace session owns it, or which live handle currently represents it.

3. Event producers can emit an orchestration id before ownership is proven.
   `async_repl.rs` does real ownership gating for the orchestrator runtime, but helpers in `agent_events.rs` and `world_ops.rs` can still source an id from the global helper without resolving a real session object.

4. Toolbox and future internal MCP routing need stable session ownership.
   `agents_cmd.rs` already assumes a real current session when projecting `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT`, but there is no dedicated session store to back that assumption.

5. World reuse semantics need a parent session object.
   `world_id` and `world_generation` already live on the runtime manifest, but they conceptually belong to the orchestration session and will need to outlive any single backend-attached handle.

6. The current model does not clearly separate these identities:
   - shell trace session
   - orchestration session
   - UAA/backend session handle
   - world boundary identity

## Scope

### In scope

- introduce a real Substrate-owned orchestration session record and store
- replace process-global identity sourcing with runtime/session-owned resolution
- thread the new session identity through shell-owned orchestrator bootstrap, agent event emission, toolbox discovery, and world-related agent event producers
- update tests and operator docs that currently rely on implicit global identity behavior

### Out of scope

- redesigning the canonical shell trace `session_id`
- renaming `uaa.agent.session`
- full member-agent orchestration/runtime landing
- toolbox server implementation itself
- generalized cross-process session resume UX

## Proposed End State

Substrate must have one explicit orchestration session object per shell-owned orchestration lifecycle. That object is created by the shell when orchestration starts, persisted under `~/.substrate/run/agent-hub/sessions/`, and treated as the only legal source of `orchestration_session_id` for runtime-owned orchestration records.

The process-global `OnceLock<String>` must be removed from the runtime path.

## Proposed Data Model

Add a new top-level session record distinct from per-backend runtime handles.

### New session record

Suggested location:

- code: `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
- persistence: `crates/shell/src/execution/agent_runtime/state_store.rs` or a sibling store module
- on-disk directory: `~/.substrate/run/agent-hub/sessions/`

Suggested record shape:

```rust
pub(crate) struct OrchestrationSessionRecord {
    pub orchestration_session_id: String,
    pub shell_trace_session_id: String,
    pub workspace_root: String,
    pub shell_owner_pid: u32,
    pub state: OrchestrationSessionState,
    pub opened_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub orchestrator_agent_id: String,
    pub orchestrator_backend_id: String,
    pub orchestrator_protocol: String,
    pub active_session_handle_id: Option<String>,
    pub latest_run_id: Option<String>,
    pub world_id: Option<String>,
    pub world_generation: Option<u64>,
    pub invalidation_reason: Option<String>,
    pub closed_at: Option<DateTime<Utc>>,
}
```

Suggested state enum:

```rust
enum OrchestrationSessionState {
    Allocating,
    Active,
    Invalidated,
    Stopping,
    Stopped,
    Failed,
}
```

### Relationship to existing records

- `ShellConfig.session_id`
  - remains the canonical shell trace session id
  - is copied into the orchestration session record as `shell_trace_session_id`
- `AgentRuntimeSessionManifest`
  - remains per-backend-handle state
  - continues to store `session_handle_id`, runtime ownership flags, optional `uaa_session_id`
  - must reference a real orchestration session id that already exists
- `world_id` / `world_generation`
  - should be duplicated or promoted to the orchestration session record once world-scoped member flows exist
  - the session record becomes the cross-handle parent source of truth

### Resolution rules

The authoritative current orchestration session for a shell process should be:

1. a persisted orchestration session record in `Active` state
2. owned by the current shell process, or otherwise explicitly resumed by a future resume path
3. linked to an authoritative-live orchestrator manifest when the orchestrator backend is attached

Code must stop synthesizing orchestration ids on demand.

## Required Code Changes

### 1. Replace the global helper with a runtime-owned resolver

Primary files:

- `crates/shell/src/execution/agent_events.rs`
- `crates/shell/src/repl/async_repl.rs`

Required change:

- remove `static ORCHESTRATION_SESSION_ID: OnceLock<String>`
- replace `orchestration_session_id() -> String` with one of:
  - an explicit session context object passed to event producers, or
  - a resolver that loads the current orchestration session from runtime state, failing closed if none exists

Implementation requirement:

- event-producing code paths that are orchestration-specific must not silently invent an id
- event-producing code paths that may run before orchestration bootstrap must either:
  - remain shell-only and omit orchestration fields, or
  - require an explicit `OrchestrationSessionRecord` / `orchestration_session_id` argument

### 2. Create and persist a real orchestration session before manifest allocation

Primary files:

- `crates/shell/src/repl/async_repl.rs`
- `crates/shell/src/execution/agent_runtime/session.rs`
- `crates/shell/src/execution/agent_runtime/state_store.rs`

Required change:

- when `start_host_orchestrator_runtime` begins, allocate the top-level orchestration session record first
- persist it before the `AgentRuntimeSessionManifest`
- use that persisted id when constructing the manifest

Implementation requirement:

- if manifest persistence fails after session creation, mark the session `Failed`
- if attached control never surfaces a durable UAA session handle, the orchestration session must transition to `Failed` or `Invalidated`, not remain implicitly active

### 3. Make toolbox discovery resolve through the orchestration session record

Primary files:

- `crates/shell/src/execution/agents_cmd.rs`

Required change:

- `substrate agent toolbox status|env` should keep using live manifest state for "is there an attached orchestrator right now?"
- but endpoint/session projection should also validate that the referenced `orchestration_session_id` belongs to an active orchestration session record

Implementation requirement:

- no toolbox endpoint should be advertised from a stale handle whose parent orchestration session is already stopped or invalidated
- the endpoint path remains `~/.substrate/run/agent-toolbox/<orchestration_session_id>.sock`

### 4. Move world-related event producers off the global helper

Primary files:

- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
- `crates/shell/src/repl/async_repl.rs`

Required change:

- stream chunks and world lifecycle agent events must receive the real orchestration session id from the owning runtime/session context
- do not fetch it from a process-global singleton

Implementation requirement:

- world-scoped member events must remain joinable to the same parent orchestration session even if the orchestrator backend session handle changes later

### 5. Add explicit session store APIs

Primary files:

- `crates/shell/src/execution/agent_runtime/state_store.rs`
- `crates/shell/src/execution/agent_runtime/mod.rs`

Suggested APIs:

- `persist_orchestration_session(&OrchestrationSessionRecord)`
- `load_orchestration_session(&str)`
- `list_orchestration_sessions()`
- `find_active_orchestration_session_for_pid(u32)`
- `mark_orchestration_session_invalidated(...)`
- `mark_orchestration_session_stopped(...)`

Implementation requirement:

- writes must stay atomic, matching current manifest persistence style
- reads must tolerate missing directories

### 6. Update docs and contracts that currently imply manifests are enough

Primary files:

- `docs/TRACE.md`
- optionally `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- optionally `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`

Required change:

- distinguish top-level orchestration session records from live per-handle manifests
- make it explicit that `orchestration_session_id` is Substrate-owned session state, not a process-local convenience id

## Sequencing

### Phase 1: Data model and store

Deliverables:

- new orchestration session record type
- session persistence directory under `~/.substrate/run/agent-hub/sessions/`
- state transitions for create, activate, invalidate, stop, fail

Exit condition:

- a shell-owned orchestration session can be created and inspected independently of any per-handle manifest

### Phase 2: Runtime bootstrap cutover

Deliverables:

- `async_repl.rs` creates session record first, then manifest
- `AgentRuntimeSessionManifest::new(...)` consumes a caller-supplied orchestration id from the session record
- no bootstrap path calls a global identity allocator

Exit condition:

- orchestrator bootstrap/shutdown/invalidation updates both the session record and the manifest consistently

### Phase 3: Event emission cutover

Deliverables:

- `agent_events.rs` no longer owns global orchestration identity state
- shell-owned orchestrator runtime events, shell completion events, and world stream events receive explicit orchestration context

Exit condition:

- all pure-agent orchestration records in trace are sourced from a real session record

### Phase 4: Status/toolbox integration

Deliverables:

- `substrate agent status` and `substrate agent toolbox *` can resolve active session state through the new session store
- stale or invalidated sessions are not projected as active even if old files remain on disk

Exit condition:

- toolbox env/status reports are backed by an active orchestration session plus an authoritative-live orchestrator handle

### Phase 5: Docs and regression coverage

Deliverables:

- tests for bootstrap, invalidation, shutdown, stale-state rejection, and toolbox/session projection
- doc updates to `docs/TRACE.md`

Exit condition:

- operator-facing docs and tests match the runtime model

## Acceptance Criteria

1. `orchestration_session_id` is never allocated from a process-global singleton in production runtime code.

2. A top-level orchestration session record exists on disk for each shell-owned orchestration lifecycle, separate from per-handle manifests.

3. `AgentRuntimeSessionManifest` creation fails closed unless it is linked to an already-created orchestration session record.

4. When the shell-owned orchestrator runtime becomes authoritative-live, the orchestration session transitions to `Active`.

5. When attached control exits unexpectedly, the orchestration session transitions to `Invalidated` and is no longer discoverable as active.

6. `substrate agent toolbox env` only reports a concrete endpoint when:
   - the orchestrator backend is allowlisted
   - a live orchestrator manifest exists
   - the parent orchestration session record is active

7. Agent and toolbox trace rows continue to carry:
   - shell `session_id`
   - `orchestration_session_id`
   - `run_id`
   without heuristic reconstruction

8. Existing shell trace session behavior remains unchanged for non-orchestration command spans.

## Validation and Testing Suggestions

### Targeted tests to add or update

- `crates/shell/src/repl/async_repl.rs`
  - extend existing runtime bootstrap tests to assert:
    - orchestration session record is created before manifest becomes live
    - session transitions on invalidation/shutdown match manifest transitions
- `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
  - add status/toolbox projection tests that reject stale manifests whose parent session is inactive
- `crates/shell/tests/agent_hub_trace_persistence.rs`
  - assert trace rows continue to contain both shell `session_id` and real `orchestration_session_id`
- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
  - add unit coverage proving stream chunk events require explicit orchestration context rather than calling a global allocator

### Suggested command validation

Run at minimum:

```bash
cargo test -p substrate-shell start_host_orchestrator_runtime -- --nocapture
cargo test -p substrate-shell agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p substrate-shell agent_hub_trace_persistence -- --nocapture
cargo test -p substrate-shell repl_persistent_session -- --nocapture
```

Then run broader shell validation:

```bash
cargo test -p substrate-shell -- --nocapture
```

### Suggested operator verification

After landing the implementation:

1. Start an async REPL with agents enabled and a host-scoped orchestrator configured.
2. Verify both directories exist:
   - `~/.substrate/run/agent-hub/sessions/`
   - `~/.substrate/run/agent-hub/handles/`
3. Run `substrate agent status --json` and confirm the projected active session id matches:
   - the session record id
   - the handle manifest parent id
   - toolbox endpoint derivation if enabled
4. Force orchestrator invalidation and confirm:
   - active session disappears from discovery
   - trace still retains historical rows
   - session record state changes to `Invalidated`

## Linux-First Implications

This prerequisite should be implemented Linux-first, then generalized.

### Why Linux first

- Linux is the repo's primary always-in-world environment in `docs/WORLD.md`
- authoritative world identity and world reuse semantics matter most there first
- current runtime liveness checks in `crates/shell/src/execution/agent_runtime/state_store.rs` already use Unix process semantics (`kill(pid, 0)`)
- toolbox UDS endpoint projection is naturally aligned with Unix host paths

### Concrete Linux-first requirements

1. The first implementation should make Linux host orchestrator lifecycle authoritative.
   The orchestration session record must be created, updated, and invalidated correctly on Linux before pursuing parity polish elsewhere.

2. World-scoped member identity must remain parented by orchestration session.
   On Linux, `world_id` and `world_generation` are meaningful operator-visible boundary facts, so the parent session record should be ready to own them.

3. UDS-based toolbox identity must stay deterministic.
   The Linux path `~/.substrate/run/agent-toolbox/<orchestration_session_id>.sock` should remain the source of projected endpoint truth.

4. Do not block on cross-platform parity before defining the session model.
   macOS can reuse the same host-side session identity even while the world backend is Lima-backed. Windows can follow with explicit degraded owner-liveness rules if needed.

### Cross-platform notes to capture during implementation

- `owner_process_is_alive` is currently Unix-real and non-Unix-degraded in `state_store.rs`
- UDS assumptions in toolbox projection are Unix-native today
- session ownership semantics should be documented so later Windows/TCP support does not reintroduce implicit identity synthesis

## Risks

1. Dual-source-of-truth drift between the new orchestration session record and existing handle manifests.
   Mitigation: make session transitions explicit and update both objects in the same runtime state transition blocks.

2. Event producers outside the REPL runtime may become hard to thread.
   Mitigation: prefer explicit context plumbing over fallback globals; fail closed where runtime ownership is required.

3. Stale files under `~/.substrate/run/agent-hub/` may confuse discovery during rollout.
   Mitigation: version the new session schema and make active discovery require both valid session state and valid live handle state.

4. Tests that currently assume one process equals one orchestration session may start failing.
   Mitigation: update fixtures to create explicit session records and assert lifecycle semantics directly.

## Open Questions

1. Should a shell process be allowed to host multiple orchestration sessions sequentially in one lifetime, or should the first runtime remain the only supported v1 model?
   Recommended v1 answer: sequential support is allowed by the data model, but only one active session per shell process at a time.

2. Should shell-originated completion events emitted before orchestrator bootstrap carry `orchestration_session_id`?
   Recommended v1 answer: no. Keep them as shell events unless they are explicitly tied to an active orchestration session.

3. Should the top-level orchestration session record own `world_id` / `world_generation` immediately, or only once member-agent world reuse lands?
   Recommended v1 answer: include nullable fields now so the parent model is stable before member execution expands.

4. Should session discovery for CLI surfaces prioritize the session store or the handle store?
   Recommended v1 answer: active session record first, authoritative-live handle second.

## Definition of Done

- process-global orchestration identity allocation is removed from the runtime path
- real orchestration session records exist and are persisted separately from handle manifests
- orchestrator bootstrap, invalidation, and shutdown transition both record families consistently
- toolbox/status/world event seams resolve through the real session identity
- targeted tests and `docs/TRACE.md` are updated to match the new contract
