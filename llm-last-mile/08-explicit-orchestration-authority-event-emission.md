# SOW: Explicit Orchestration Authority for Event Emission

## Objective

Remove ambient PID and process-state lookup from shell-owned orchestration-scoped event emission paths, and replace it with explicit runtime-owned context threaded from session/bootstrap state into every emitter that can append `agent_event` rows.

The required outcome is narrow and implementation-oriented:

- no production event-emission path may recover `orchestration_session_id` by scanning runtime state for `shell_owner_pid`,
- shell-owned emitters must accept explicit runtime ownership/context from the caller,
- emitters that do not receive explicit orchestration context must suppress the orchestration-scoped `agent_event` row instead of reconstructing identity from ambient state,
- terminal stdout/stderr behavior and trace span emission must continue even when orchestration-scoped `agent_event` emission is suppressed.

This slice is about authority and plumbing on event emission paths. It is not a redesign of agent-event schema, runtime-store layout, or operator-facing status selection.

## Why This Is Needed

The repo already states the intended contract in [`docs/TRACE.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md):

- runtime-owned producers must emit a real `orchestration_session_id` or suppress the row,
- they must not synthesize a process-global fallback id,
- trace joins are supposed to use explicit keys, not heuristics.

Current production code still violates that contract on several shell-generated paths by doing live PID-based recovery at emit time:

- [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  - `resolve_active_orchestration_session_id()`
  - REPL command-completion call sites
- [`crates/shell/src/execution/routing/dispatch/exec.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs)
  - `resolve_active_orchestration_session_id()`
  - host external command stream emission
- [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
  - `resolve_active_orchestration_session_id()`
  - non-PTY world stream emission

That lookup shape is too weak for an authority boundary:

- it depends on mutable runtime-store state and liveness scans during emission,
- it couples event identity to `shell_owner_pid` instead of the already-allocated orchestration session,
- it fails only after the emitter has already chosen to reach for ambient state,
- it is duplicated in multiple modules, so drift is likely,
- it is inconsistent with shell runtime paths that already have explicit manifest/session ownership in hand.

The repo already has the correct source of truth for most shell-owned orchestrator activity. This SOW exists to require that the emitters consume it directly.

## Current Repo Seams

### Explicit runtime/session authority already exists

The async REPL bootstrap path already allocates and persists explicit orchestration ownership:

- [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  - `RuntimeOrchestrationContext`
  - `PreparedHostOrchestratorRuntime`
  - `start_host_orchestrator_runtime_with_prepared(...)`
  - `translate_wrapper_event(...)`
  - `build_runtime_message_event(...)`
- [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
  - `OrchestrationSessionRecord`
- [`crates/shell/src/execution/agent_runtime/session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
  - `AgentRuntimeParticipantRecord`
  - participant lineage fields
- [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
  - canonical session-root persistence under `run/agent-hub/sessions/<orchestration_session_id>/...`

Those paths already prove the repo does not need ambient PID lookup to know who owns the orchestration session. They already have that information in memory.

### Event helpers are currently too weak

[`crates/shell/src/execution/agent_events.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs) currently exposes low-level helpers:

- `publish_agent_event(...)`
- `publish_command_completion(orchestration_session_id: Option<&str>, ...)`

That API surface only accepts an optional session id string. It does not model runtime-owned orchestration authority as a first-class input, so callers are free to recover it ad hoc.

### Ambient lookup is still used in production emitters

1. REPL command completion

- [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  - host escape completion
  - world PTY completion
  - world line completion
  - host-only completion

All currently call `publish_command_completion(...)` after `resolve_active_orchestration_session_id()`.

2. Host external command stream chunks

- [`crates/shell/src/execution/routing/dispatch/exec.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs)
  - `execute_external(...)`
  - `spawn_host_stream_thread(...)`

This path resolves orchestration identity once per process launch from PID-owned runtime state, then emits stream rows from background threads using that recovered value.

3. World dispatch shell-generated stream chunks

- [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
  - `stream_non_pty_via_agent(...)`
  - `process_agent_stream_body(...)`
  - `emit_stream_chunk(...)`

This path also recovers orchestration identity via PID/state-store lookup before emitting shell-owned chunk rows.

### Restart alert paths are partially explicit today

[`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) already threads `startup_context` into:

- `build_world_restart_required_alert(...)`
- `emit_world_restarted_alert(...)`
- `handle_detected_world_drift(...)`
- `restart_world_session(...)`

Those paths already suppress rows when orchestration context is absent. They are the right behavioral model for the rest of this slice.

## In Scope

- introducing one explicit shell-local event-emission context model for orchestration-scoped shell events,
- threading that context through REPL host/world command execution paths,
- updating shell-owned command-completion and stream-chunk emission to consume explicit context,
- removing production use of PID-based orchestration lookup on event emission paths,
- tightening docs/tests so the explicit-context requirement is enforced,
- leaving pass-through wrapper/world-agent events untouched unless a shell-owned event wrapper is still deriving identity ambiently.

## Out of Scope

- redesigning `AgentEvent` wire schema,
- changing `substrate agent status` or toolbox session selection behavior,
- changing session-root or participant-root filesystem layout,
- changing Linux shared-world ownership semantics,
- broad trace replay or router/workflow schema changes,
- reworking agent wrapper event translation in paths that already have explicit manifest/session authority,
- changing historical trace rows beyond additive clarification in docs/tests.

## Current Blockers and Gaps

1. Production emitters can still reach for ambient authority.

- `find_active_orchestration_session_for_pid(std::process::id())` is currently used as an event-time fallback in multiple modules.
- That is the wrong dependency direction: emission should consume already-owned context, not rediscover it from persisted state.

2. Helper signatures do not force correct call-site behavior.

- `publish_command_completion(...)` takes `Option<&str>`.
- `emit_stream_chunk(...)` takes raw optional strings plus caller-chosen run ids.
- Neither API makes “this came from the active shell-owned orchestration runtime” explicit.

3. Host command execution does not currently accept runtime context.

- [`execute_command(...)` in `dispatch/exec.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs) only accepts `config`, `command`, `cmd_id`, and `running_child_pid`.
- That forces the callee to rediscover orchestration identity itself.

4. Some shell-generated event paths still synthesize weak per-event correlation.

- host stream threads currently emit with `run_id="unknown"`,
- world dispatch stream helpers still permit `run_id` values that are not explicitly tied to a caller-owned context object.

This SOW does not need to redesign all run semantics, but it must stop orchestration-scoped shell rows from being emitted with ambiently recovered session identity plus synthetic fallback correlation.

5. The state store currently exposes the exact ambient authority helper this slice wants to retire from production code.

- [`AgentRuntimeStateStore::find_active_orchestration_session_for_pid(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

That method may remain for diagnostics/tests if still needed, but it must stop being part of production event-emission control flow.

## Required Semantics and Invariants

### 1. Explicit authority only

Every shell-owned emitter that appends an orchestration-scoped `AgentEvent` must receive explicit orchestration authority from its caller.

Minimum required authority payload:

- `orchestration_session_id`
- event producer identity for the shell-owned row (`agent_id`, `role`, `backend_id`) when the helper sets those fields
- participant lineage when the row is logically attached to a persisted participant
- `world_id` and `world_generation` when the caller already has authoritative values

Exact Rust type names are flexible, but the implementation must use one dedicated context type rather than more unrelated `Option<&str>` parameters.

### 2. No PID-based recovery on emit paths

The following production pattern must be eliminated from event-emission control flow:

```rust
AgentRuntimeStateStore::new()?
    .find_active_orchestration_session_for_pid(std::process::id())
```

Specifically:

- no REPL command-completion path may do this,
- no host external command stream path may do this,
- no world dispatch stream path may do this,
- no restart alert helper may reintroduce it as a fallback.

### 3. Suppress, do not guess

If an emitter does not receive explicit orchestration authority:

- it must still write terminal stdout/stderr normally,
- it must still write ordinary command spans/trace telemetry normally,
- it must not append an orchestration-scoped `agent_event`,
- it must not scan runtime state, shell globals, PID tables, environment variables, or trace history to guess the orchestration session.

This is the same fail-closed posture already used by:

- [`publish_command_completion(None, ...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs)
- [`build_world_restart_required_alert(None, ...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [`emit_stream_chunk(..., None, ...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)

### 4. Caller-owned run/cmd correlation

For shell-owned orchestration-scoped rows emitted during command execution, the caller must also provide the per-command correlation needed for that row.

At minimum:

- `cmd_id` for command-completion rows,
- `run_id` for stream-chunk rows,
- `span_id` when the event is tied to a known command span.

Required rule:

- orchestration-scoped shell rows must not emit with synthetic fallback values like `"unknown"` once explicit context plumbing is available.
- if the caller cannot provide a real run identity for an orchestration-scoped stream row, suppress the row instead of inventing one.

### 5. Existing explicit runtime translation remains the model

Paths that already translate wrapper/runtime events from explicit manifest/session context must remain explicit and unchanged in principle:

- `translate_wrapper_event(...)`
- `build_runtime_message_event(...)`
- world-restart alert construction from `startup_context`

This slice should align the weaker host/world shell emitters with that existing model rather than invent a second authority path.

### 6. Trace doc contract must become implementation contract

[`docs/TRACE.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md) must continue to say:

- runtime-owned producers emit a real `orchestration_session_id` or suppress the row,
- no process-global fallback id is allowed.

After this slice, production code must actually satisfy that statement across shell-owned event emitters.

## Exact Code Areas

### Primary implementation

- [`crates/shell/src/execution/agent_events.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs)
  - introduce the explicit event-emission context type
  - update `publish_command_completion(...)` to consume explicit context rather than a bare optional session id
  - keep suppression semantics when context is absent

- [`crates/shell/src/execution/routing/dispatch/exec.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs)
  - remove `resolve_active_orchestration_session_id()`
  - thread optional explicit emission context into `execute_command(...)`
  - thread command/run context into `execute_external(...)`
  - update `spawn_host_stream_thread(...)` so it receives caller-owned context instead of ambiently recovered session identity

- [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
  - remove `resolve_active_orchestration_session_id()`
  - thread explicit emission context into `stream_non_pty_via_agent(...)`
  - update `process_agent_stream_body(...)` and `emit_stream_chunk(...)` so shell-generated chunk rows use caller-owned context only

- [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  - stop calling `resolve_active_orchestration_session_id()` for command completion
  - derive event-emission context from `RuntimeOrchestrationContext` / active manifest snapshots
  - pass that context into host/world execution helpers
  - preserve current explicit restart-alert behavior

### Related authority and compatibility seams

- [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
  - production emitters must stop depending on `find_active_orchestration_session_for_pid(...)`
  - keeping the helper for tests/diagnostics is acceptable if production paths no longer call it

- [`crates/common/src/agent_events.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs)
  - no schema redesign required, but this file is the field contract the new explicit shell-local context must populate correctly

- [`docs/TRACE.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
  - update wording if needed so shell-owned command/stream emitters are explicitly covered by the no-fallback rule

## Testing Requirements

At minimum, land or update tests for:

1. command completion emits only when explicit orchestration context is supplied

- anchor:
  - [`crates/shell/src/execution/agent_events.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs)

2. host stream chunk emission emits only when explicit orchestration context is supplied, and does not emit an orchestration-scoped row when context is absent

- anchor:
  - [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)

3. REPL host and world command-completion paths no longer read orchestration identity from PID-owned runtime state

- anchors:
  - [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  - [`crates/shell/src/execution/routing/dispatch/exec.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs)

4. shell-owned stream rows do not emit with synthetic fallback run identity once explicit context plumbing exists

- anchors:
  - [`crates/shell/src/execution/routing/dispatch/exec.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs)
  - [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)

5. world restart alert behavior remains explicit-context-only

- anchor:
  - [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

6. repo-wide guard that production code no longer uses PID lookup for event emission

- acceptable forms:
  - direct tests for the new context plumbing,
  - or a bounded repo assertion that only tests/diagnostics still reference `find_active_orchestration_session_for_pid(...)`.

Recommended verification commands:

```bash
cargo test -p shell agent_events
cargo test -p shell async_repl
cargo test -p shell routing::dispatch::world_ops
cargo test -p shell routing::dispatch::tests::host_replay
```

If module-filtered targets are not stable in this crate layout, run:

```bash
cargo test -p shell -- --nocapture
```

## Acceptance Criteria

- no production event-emission path in `async_repl`, `dispatch/exec`, or `dispatch/world_ops` looks up orchestration identity from `shell_owner_pid`,
- shell-owned command-completion events consume explicit caller-provided orchestration context,
- shell-owned stream-chunk events consume explicit caller-provided orchestration and run context,
- missing orchestration context suppresses the orchestration-scoped `agent_event` row without suppressing stdout/stderr or command spans,
- existing explicit runtime event translation paths remain explicit and continue to work,
- docs and tests reflect the “real id or suppress” contract,
- the remaining state-store PID lookup helper, if retained, is no longer part of production emission control flow.

## Recommended Execution Order

1. Define the explicit shell event-emission context type and update `publish_command_completion(...)`.

2. Thread optional context from `RuntimeOrchestrationContext` / active manifest state through REPL command execution entry points.

3. Update `dispatch/exec.rs` host execution helpers to accept that context and remove ambient PID lookup.

4. Update `dispatch/world_ops.rs` non-PTY/stream helpers to accept explicit context and remove ambient PID lookup.

5. Tighten tests around suppression, explicit context, and no-synthetic-fallback behavior.

6. Update [`docs/TRACE.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md) so the documented contract matches the landed implementation.

## Relationship To Existing `llm-last-mile` Work

This slice is downstream of the earlier identity/runtime-state work:

- [`01-orchestration-session-identity.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/01-orchestration-session-identity.md)
- [`02-session-participant-record.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/02-session-participant-record.md)
- [`06-session-centric-state-store.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/06-session-centric-state-store.md)

It relies on those slices having created real session/participant authority that can be threaded directly.

It should stay separate from:

- shared-world backend correctness work in [`07-world-replacement-ordering-rollback-atomic-metadata.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/07-world-replacement-ordering-rollback-atomic-metadata.md),
- operator surface selection and status aggregation work,
- any larger trace-schema or workflow-router/toolbox redesign.
