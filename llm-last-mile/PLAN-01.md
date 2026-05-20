<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/testing-autoplan-restore-20260429-101402.md -->
# PLAN-01: Orchestration Session Identity Cutover

Source file: `llm-last-mile/01-orchestration-session-identity.md`  
Branch: `testing`  
Plan type: backend-only, no UI scope  
Review posture: `/autoplan` consolidation pass with `/plan-eng-review` structure and rigor  
Status: execution-ready

## What This Plan Does

This slice replaces the current process-global `orchestration_session_id` allocator with a real Substrate-owned parent session record.

That matters because the repo already behaves like a real orchestration session exists, but the actual identity still comes from a shell-process `OnceLock` in [agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs:12). That is the wrong authority boundary for bootstrap, status, toolbox projection, world correlation, invalidation, and any future resume/fork work.

The goal is narrow and concrete:

1. Persist a parent orchestration session record under `~/.substrate/run/agent-hub/sessions/`.
2. Make bootstrap create that parent record before any live child handle can exist.
3. Make operator surfaces and runtime-owned agent events resolve through that parent record.
4. Remove process-global identity synthesis from production runtime paths.

This is the first prerequisite in the `llm-last-mile` packet. It is not the later grouped-store migration, not participant generalization, and not shared-world ownership.

## Scope Challenge

### Why this is the right first slice

The repo already has the right child-handle runtime seams and the right operator surfaces. What it does not have is a first-class parent session object that owns orchestration identity. If the team adds more multi-agent behavior before fixing that boundary, every later slice inherits cleanup work.

Fix the authority boundary first. Then build the later features on top of a real session model instead of a convenience UUID.

### What already exists

| Sub-problem | Existing code | Reuse or replace |
|---|---|---|
| Atomic JSON persistence | [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:36) `write_atomic_json()` | Reuse exactly |
| Per-handle runtime manifest | [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:31) | Reuse as the child record |
| Authoritative-live child liveness rules | [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:160) | Reuse, then parent-gate |
| Bootstrap lifecycle | [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1260) `start_host_orchestrator_runtime()` | Extend |
| Live-manifest discovery | [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:92) | Reuse as low-level child lookup only |
| Status and toolbox projection | [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:404), [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:913) | Extend with parent validation |
| Agent-event trace flattening | [routing/telemetry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/telemetry.rs:174) | Reuse unchanged |
| Canonical `AgentEvent` schema | [crates/common/src/agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs:66) | Reuse unchanged, do not weaken required fields |

### Minimum change that achieves the goal

Do this, and only this:

- add one new parent record type
- extend the existing runtime store with parent-session APIs
- make bootstrap create parent first, child second
- thread explicit orchestration context to runtime-owned agent-event producers
- parent-gate status and toolbox projection
- update docs and tests

Do not:

- reshape `handles/` into grouped session directories
- redesign shell trace `session_id`
- generalize the runtime model to member participants yet
- introduce a registry service, cache, or new long-lived daemon
- leave a fallback global allocator "just for convenience"

### Complexity check

The blast radius is real but still bounded. This slice touches:

- `crates/shell/src/execution/agent_runtime/`
- `crates/shell/src/repl/async_repl.rs`
- `crates/shell/src/execution/agent_events.rs`
- `crates/shell/src/execution/agents_cmd.rs`
- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
- `crates/shell/src/execution/routing/dispatch/exec.rs`
- targeted tests
- `docs/TRACE.md`

That is more than 8 files once tests and docs are counted. Normally that is a smell. Here it is acceptable because one authority seam crosses four already-existing operator/runtime surfaces. The constraint is: no new service objects, no new storage layout, no speculative abstractions.

### Hard non-goals

This slice must not absorb:

- `06-session-centric-state-store.md`
- participant-generalized runtime records from slice `02`
- shared-world owner binding from slice `03`
- generation invalidation semantics from slice `05`
- public multi-session selection or resume UX

## Architecture Contract

### No-ambiguity rules

These are hard rules, not suggestions:

1. Production runtime code must not call a process-global `orchestration_session_id()` helper.
2. A child `AgentRuntimeSessionManifest` must never be the thing that creates or legitimizes the parent session identity.
3. A parent session record must exist on disk before any live child can become discoverable.
4. A child is authoritative-live only when both:
   - the existing child ownership rules pass
   - the parent session exists and is `Active`
5. Ordinary shell commands and host stream plumbing must not be retroactively stamped with orchestration identity if no active parent session exists.
6. If discovery sees ambiguity, missing parent state, or parent-child mismatch, it fails closed. No "latest wins".
7. `AgentEvent` remains unchanged. It still requires `orchestration_session_id`. The fix is to stop emitting `AgentEvent` when no legitimate orchestration session exists, not to weaken the schema.

### Target data model

Add a new module:

[`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)

Required record shape:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OrchestrationSessionState {
    Allocating,
    Active,
    Invalidated,
    Stopping,
    Stopped,
    Failed,
}
```

Persistence path:

```text
~/.substrate/run/agent-hub/sessions/<orchestration_session_id>.json
```

Store location decision:

- keep parent-session persistence in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1)
- export the new type from [mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mod.rs:1)
- do not introduce a separate store service just for this slice

### Parent-child lifecycle contract

Parent and child do not use the exact same enum, so the mapping must be explicit.

| Moment | Parent state | Child state | Notes |
|---|---|---|---|
| parent record created | `Allocating` | no child yet | parent must exist first |
| child manifest persisted | `Allocating` | `Allocating` | child references existing parent id |
| durable UAA session handle observed and ownership valid | `Active` | `Ready` | parent becomes active in the same transition block that first makes the child authoritative-live |
| graceful shutdown requested | `Stopping` | `Stopping` | both transition before teardown completes |
| graceful shutdown completed | `Stopped` | `Stopped` | record `closed_at` on parent |
| attached control exits after being active | `Invalidated` | `Invalidated` | record reason |
| bootstrap fails before durable ownership | `Failed` | `Failed` if child exists, otherwise no child | `Invalidated` is not used before a session has ever become active |

Two specific decisions remove ambiguity:

- Parent `Active` means "there is an attached, durable, authoritative child right now". It does not mean "we allocated a UUID".
- There is no parent `Restarting` state in this slice. Do not add one.

### Required store APIs

Extend [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1) with:

```rust
pub(crate) fn sessions_dir(&self) -> PathBuf;
pub(crate) fn persist_orchestration_session(
    &self,
    session: &OrchestrationSessionRecord,
) -> Result<()>;
pub(crate) fn load_orchestration_session(
    &self,
    orchestration_session_id: &str,
) -> Result<Option<OrchestrationSessionRecord>>;
pub(crate) fn list_orchestration_sessions(&self) -> Result<Vec<OrchestrationSessionRecord>>;
pub(crate) fn find_active_orchestration_session_for_pid(
    &self,
    pid: u32,
) -> Result<Option<OrchestrationSessionRecord>>;
pub(crate) fn resolve_live_orchestrator_session(
    &self,
    agent_id: &str,
) -> Result<Option<(OrchestrationSessionRecord, AgentRuntimeSessionManifest)>>;
```

Rules:

- all writes use existing `write_atomic_json()`
- reads tolerate a missing `sessions/` directory
- `find_active_orchestration_session_for_pid()` returns an error, not newest-wins, when more than one active parent exists for the same shell pid
- `resolve_live_orchestrator_session()` fails closed on:
  - missing parent
  - inactive parent
  - missing `active_session_handle_id`
  - mismatched `active_session_handle_id`
  - multiple active parent candidates

### Event emission contract

This is the most important cleanup because the current helper is hiding ownership bugs.

#### `agent_events.rs`

Current problem:

- [agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs:66) exposes `orchestration_session_id() -> String`
- [publish_command_completion()](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs:72) always emits an `AgentEvent`
- demo helpers also use the same global id

Required change:

```rust
pub(crate) fn publish_command_completion(
    orchestration_session_id: Option<&str>,
    command: &str,
    cmd_id: &str,
    status: &ExitStatus,
)
```

Behavior:

- `Some(id)` means emit the `AgentEvent`
- `None` means do not emit an `AgentEvent` at all
- ordinary shell command completions still have regular shell trace rows; they just stop pretending they belong to an orchestration session

Demo helper decision:

- remove the production global helper completely
- if `schedule_demo_events()` and `schedule_demo_burst()` remain, each helper mints its own local UUID inline and does not share it through any reusable global or runtime lookup path

#### `world_ops.rs` and `exec.rs`

Current problem:

- [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1786) `emit_stream_chunk()` creates `AgentEvent::stream_chunk(...)`
- [exec.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs:1597) also calls `emit_stream_chunk()`
- today that helper reaches into the global allocator

Required signature change:

```rust
pub(super) fn emit_stream_chunk(
    orchestration_session_id: Option<&str>,
    agent_label: &str,
    run_id: &str,
    span_id: Option<&str>,
    data: &[u8],
    is_stderr: bool,
)
```

Behavior:

- world-backed and orchestrator-backed callers pass `Some(parent_session_id)`
- host exec callers that are not part of an orchestration session pass `None`
- `None` still writes bytes to stdout/stderr, but does not emit an `AgentEvent`

#### `async_repl.rs` world alerts

Current problem:

- [emit_world_restarted_alert()](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2251) and [build_world_restart_required_alert()](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2292) also call the global helper

Required change:

- thread orchestration context from the active runtime when it exists
- if no active parent session exists, do not emit an `AgentEvent` alert for these world events
- keep terminal/operator messaging; skip fake orchestration correlation

Exact rule:

- `emit_world_restarted_alert(...)` and `build_world_restart_required_alert(...)` must take `Option<&str>` and follow the same contract as `publish_command_completion(...)`: `Some(id)` emits an `AgentEvent`, `None` does not

### Operator discovery contract

`substrate agent status`, `substrate agent toolbox status`, and `substrate agent toolbox env` must all resolve through the same parent-aware rule set.

Required rule:

1. Start from the selected orchestrator agent id in config/inventory.
2. Resolve the live child manifest candidate.
3. Load the referenced parent session record by `orchestration_session_id`.
4. Accept the result only if:
   - child is authoritative-live
   - parent exists
   - parent state is `Active`
   - parent `active_session_handle_id` is set
   - parent `active_session_handle_id` matches the child handle id
5. If multiple active parents exist for the same orchestrator agent or shell pid, fail closed with an ambiguity error.

Concrete surface decisions:

- [build_status_report()](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:404) must stop trusting `list_live_manifests()` alone for active pure-agent session projection
- [run_toolbox_status()](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:883) and [run_toolbox_env()](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:969) must use the parent-aware resolver
- keep endpoint naming unchanged:

```text
~/.substrate/run/agent-toolbox/<orchestration_session_id>.sock
```

## Architecture Diagrams

### Bootstrap and lifecycle

```text
BOOTSTRAP / OWNERSHIP
=====================

ShellConfig.session_id
        |
        v
start_host_orchestrator_runtime()
        |
        +-- resolve config / policy / inventory / backend
        |
        +-- allocate orchestration_session_id locally
        |
        +-- persist parent session ------------------------------+
        |      sessions/<orchestration_session_id>.json         |
        |      state=Allocating                                 |
        |                                                       |
        +-- create child manifest ------------------------------+----+
        |      handles/<session_handle_id>.json                      |
        |      state=Allocating                                      |
        |                                                            |
        +-- run_control() / wait for durable UAA session handle      |
               |                                                     |
               +-- handle observed + ownership valid?                |
               |      |                                              |
               |      +-- yes -> child Ready                         |
               |                 parent Active                       |
               |                 parent.active_session_handle_id=child|
               |                                                     |
               +-- no  -> parent Failed                              |
                          child Failed                               |
```

### Status and toolbox lookup

```text
STATUS / TOOLBOX PROJECTION
===========================

selected orchestrator agent_id
        |
        +-- resolve live child candidate
        |
        +-- load parent by child.orchestration_session_id
        |
        +-- validate:
              child.is_authoritative_live()
              parent exists
              parent.state == Active
              parent.active_session_handle_id matches child (when set)
              no ambiguity among active parents
        |
        +-- success -> publish session + endpoint
        |
        +-- failure -> dependency_unavailable, or explicit ambiguity error if more than one active parent exists
```

## Implementation Plan

### Ordered execution checklist

Implement this plan in this order. Do not reshuffle it.

1. Add `orchestration_session.rs` and export the new types from `agent_runtime/mod.rs`.
2. Extend `state_store.rs` with parent-session persistence, load, list, and resolve APIs.
3. Update `async_repl.rs` bootstrap so parent persistence happens before child manifest creation.
4. Update `async_repl.rs` shutdown and invalidation paths so parent and child transition together.
5. Remove production `ORCHESTRATION_SESSION_ID` usage from `agent_events.rs`.
6. Update `publish_command_completion(...)` callsites in `async_repl.rs` to pass `None`.
7. Update `emit_stream_chunk(...)` signature in `world_ops.rs`, then fix all `world_ops.rs` and `exec.rs` callers.
8. Update world restart alert helpers in `async_repl.rs` to use `Option<&str>` and stop emitting fake orchestration events.
9. Update `agents_cmd.rs` production paths to resolve parent + child together.
10. Update tests.
11. Update `docs/TRACE.md`.

If an implementation step requires touching code outside that order, stop and justify it in the PR description. The default assumption is that out-of-order work is scope creep.

### Workstream 1: Parent session model and store

Files:

- [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs) (new)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agent_runtime/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mod.rs)

Required work:

1. Add `OrchestrationSessionRecord` and `OrchestrationSessionState`.
2. Add record helpers:
   - `transition_state(...)`
   - `touch_active(...)`
   - `mark_failed(...)`
   - `mark_invalidated(...)`
   - `mark_stopping(...)`
   - `mark_stopped(...)`
3. Add parent-session persistence and load/list/resolve helpers to `state_store.rs`.
4. Keep the storage root under `~/.substrate/run/agent-hub/`.
5. Do not change existing child manifest JSON shape in this slice.

Exit gate:

- a parent session can be created, loaded, listed, transitioned, and resolved without booting a live child handle

### Workstream 2: Parent-first bootstrap and lifecycle cutover

Files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Required work:

1. In [start_host_orchestrator_runtime()](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1260), allocate `orchestration_session_id` locally with `Uuid::now_v7().to_string()`.
2. Create and persist the parent session record first with `state=Allocating`.
3. Pass that id into `AgentRuntimeSessionManifest::new(...)`.
4. Persist the child manifest second.
5. When durable UAA ownership is established:
   - set child `uaa_session_id`
   - refresh child ownership validity
   - transition child to `Ready`
   - set parent `active_session_handle_id`
   - set parent `latest_run_id`
   - transition parent to `Active`
6. On startup failure before durable ownership:
   - parent `Failed`
   - child `Failed` if child exists
   - do not use `Invalidated` in this codepath
7. On attached-control exit after activation:
   - child `Invalidated`
   - parent `Invalidated`
   - record `invalidation_reason`
8. On graceful shutdown:
   - parent `Stopping` then `Stopped`
   - child `Stopping` then `Stopped`
   - set parent `closed_at`

Exit gate:

- no live child can exist unless the parent session was already persisted

### Workstream 3: Event emission cutover

Files:

- [crates/shell/src/execution/agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [crates/shell/src/execution/routing/dispatch/exec.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Required work:

1. Delete production use of `ORCHESTRATION_SESSION_ID`.
2. Update `publish_command_completion(...)` to accept `Option<&str>`.
3. Update every current async REPL shell command completion callsite to pass `None`.
4. Update `emit_stream_chunk(...)` to accept `Option<&str>`.
5. Update world-backed callers to pass `Some(parent_id)`.
6. Update [exec.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs:1597) host stream path to pass `None`.
7. Update world-restart alert helpers in `async_repl.rs` to take `Option<&str>` and emit no `AgentEvent` when the value is `None`.
8. Keep the sender singleton. Remove the identity singleton.

Exit gate:

- no production path can mint orchestration identity on demand

### Workstream 4: Status and toolbox parent-gating

Files:

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Required work:

1. Keep `find_live_orchestrator()` as a low-level child helper for tests and migration glue only. Production `status` and `toolbox` code must not call it after this slice lands.
2. Move `agent status` active-session projection to `resolve_live_orchestrator_session()`.
3. Move `agent toolbox status|env` to the same parent-aware resolver.
4. Preserve existing allowlist and protocol checks.
5. Preserve endpoint path shape.
6. Fail closed on:
   - missing parent
   - inactive parent
   - mismatched active child
   - multiple active parents

Exit gate:

- `status` and `toolbox` surfaces never advertise from a stale child whose parent is missing or inactive

### Workstream 5: Docs and regression tests

Files:

- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs)
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)

Required work:

1. Document the distinction between shell `session_id`, parent orchestration session, child runtime handle, and `world_id`.
2. Extend bootstrap tests to assert parent-first persistence and parent lifecycle transitions.
3. Extend toolbox/status tests to assert parent-child validation failures.
4. Add agent-event tests for `publish_command_completion(None, ...)` no-op behavior.
5. Add stream/event tests proving no production path can still reach a global allocator.

Exit gate:

- docs and tests describe the same runtime model the code implements

## Architecture Review Findings

### Finding 1

`[P1] (confidence: 9/10) crates/shell/src/execution/agent_events.rs:12-70 - process-global identity allocation is the wrong authority boundary`

Why it matters:

It lets the code emit records that look like live orchestration records before Substrate has actually proven live orchestration ownership.

Recommendation:

Remove it from production runtime paths entirely. No fallback allocator.

### Finding 2

`[P1] (confidence: 9/10) crates/shell/src/repl/async_repl.rs:1339-1347 - bootstrap creates the child manifest before a real parent session exists`

Why it matters:

That makes the child do double duty as both handle record and parent source of truth. The rest of the packet is trying to stop exactly that.

Recommendation:

Persist parent first. Always.

### Finding 3

`[P1] (confidence: 8/10) crates/shell/src/execution/agents_cmd.rs:404-430 and 913-924 - status/toolbox still trust child liveness without parent validation`

Why it matters:

These are operator control-plane surfaces. Wrong endpoint projection is worse than no endpoint projection.

Recommendation:

Use one parent-aware resolver for both surfaces and fail closed on ambiguity.

### Finding 4

`[P2] (confidence: 8/10) crates/shell/src/execution/routing/dispatch/world_ops.rs:1786-1814 and crates/shell/src/repl/async_repl.rs:2251-2302 - world-related agent events still reach for global orchestration identity`

Why it matters:

World correlation is one of the main reasons this parent session exists. If those events stay global, later shared-world work inherits a bad join key.

Recommendation:

Thread explicit orchestration context. If none exists, skip `AgentEvent` emission instead of inventing identity.

## Test Review

100% coverage is the target for changed behavior. The current tests are good at child-manifest fail-closed behavior. They are not yet sufficient for a parent-child session model.

### Code path coverage

```text
CODE PATH COVERAGE
==================
[+] crates/shell/src/repl/async_repl.rs
    |
    +-- start_host_orchestrator_runtime()
    |   +-- [GAP] parent session record persisted before child manifest
    |   +-- [GAP] parent becomes Active only after durable UAA ownership
    |   +-- [GAP] parent becomes Failed when durable ownership never surfaces
    |   +-- [GAP] parent becomes Invalidated on attached-control exit
    |   +-- [GAP] parent + child both transition through Stopping -> Stopped
    |
    +-- world restart alert helpers
        +-- [GAP] active orchestration context required for agent-event emission
        +-- [GAP] no fake orchestration alert emitted when runtime is absent

[+] crates/shell/src/execution/agent_events.rs
    |
    +-- publish_command_completion(Some(id), ...)
    |   +-- [GAP] emits runtime-owned task-end event with real orchestration id
    |
    +-- publish_command_completion(None, ...)
        +-- [GAP] emits nothing, leaves ordinary shell tracing unchanged

[+] crates/shell/src/execution/routing/dispatch/world_ops.rs
    |
    +-- emit_stream_chunk(Some(id), ...)
        +-- [GAP] world-backed stream chunks keep real parent session correlation

[+] crates/shell/src/execution/routing/dispatch/exec.rs
    |
    +-- emit_stream_chunk(None, ...)
        +-- [GAP] host stream output does not mint fake orchestration events

[+] crates/shell/src/execution/agents_cmd.rs
    |
    +-- build_status_report()
    |   +-- [GAP] inactive parent does not project active session
    |   +-- [GAP] missing parent does not project active session
    |
    +-- toolbox status/env
        +-- [TESTED] no child manifest -> dependency_unavailable
        +-- [TESTED] trace-only history -> fail closed
        +-- [TESTED] invalidated child not resurrected by trace
        +-- [GAP] live child + inactive parent -> fail closed
        +-- [GAP] live child + missing parent -> fail closed
        +-- [GAP] live child + mismatched parent handle id -> fail closed
        +-- [GAP] two active parents -> ambiguity / fail closed
```

### User flow coverage

```text
USER FLOW COVERAGE
==================
[+] Start async REPL with agents enabled
    |
    +-- [GAP] parent session file exists before child becomes discoverable
    +-- [GAP] status/toolbox report the same active orchestration session id

[+] Orchestrator exits unexpectedly
    |
    +-- [GAP] active session disappears from discovery
    +-- [GAP] trace remains historical only

[+] Operator asks for toolbox env after stale state remains on disk
    |
    +-- [GAP] stale child cannot publish endpoint if parent is dead

[+] Ordinary shell command completes before any orchestrator runtime exists
    |
    +-- [GAP] no fake orchestration task-end event is emitted

[+] World stream output occurs in non-orchestrator host exec flow
    |
    +-- [GAP] stdout/stderr still print, but no orchestration-scoped agent event is emitted
```

### Required test additions by file

#### `crates/shell/src/repl/async_repl.rs`

Extend:

- [start_host_orchestrator_runtime_bootstraps_and_persists_a_live_manifest()](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3650)
- [start_host_orchestrator_runtime_invalidates_when_attached_control_exits()](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3730)
- [start_host_orchestrator_runtime_does_not_persist_live_manifest_without_session_handle()](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3821)
- [shutdown_host_orchestrator_runtime_waits_for_cancel_completion_before_stopping()](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3901)

Add assertions for:

- session file exists before child becomes live
- parent transitions `Allocating -> Active`
- parent transitions to `Invalidated`
- parent transitions `Stopping -> Stopped`
- parent transitions to `Failed` when durable ownership never materializes

#### `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

Keep existing tests for:

- no active session -> `dependency_unavailable`
- trace-only history does not authorize
- invalidated child is not resurrected

Add:

- live child + missing parent -> fail closed
- live child + inactive parent -> fail closed
- live child + mismatched `active_session_handle_id` -> fail closed
- multiple active parents for one orchestrator agent -> ambiguity / fail closed
- `agent status --json` does not project active pure-agent session from child-only state

#### `crates/shell/tests/agent_hub_trace_persistence.rs`

Keep:

- flattened join keys survive
- `parent_run_id` survives when present

Add:

- runtime-owned rows still keep both shell `session_id` and real `orchestration_session_id`
- no fake orchestration session is emitted from a no-context shell completion path

#### `crates/shell/src/execution/agent_events.rs`

Add:

- `publish_command_completion(None, ...)` emits no `AgentEvent`
- `publish_command_completion(Some(id), ...)` still emits the expected task-end payload

#### `crates/shell/src/execution/routing/dispatch/world_ops.rs` and `exec.rs`

Add:

- unit coverage that `emit_stream_chunk(Some(id), ...)` emits an orchestration-scoped `AgentEvent`
- host exec stream path with `None` does not emit orchestration-scoped `AgentEvent`

### Test commands

Run at minimum:

```bash
cargo test -p substrate-shell start_host_orchestrator_runtime -- --nocapture
cargo test -p substrate-shell agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p substrate-shell agent_hub_trace_persistence -- --nocapture
cargo test -p substrate-shell repl_persistent_session -- --nocapture
cargo test -p substrate-shell publish_command_completion -- --nocapture
```

Then run:

```bash
cargo test -p substrate-shell -- --nocapture
```

### QA artifact

Primary QA artifact for follow-up verification:

[spensermcconnell-testing-eng-review-test-plan-20260429-101402.md](/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-testing-eng-review-test-plan-20260429-101402.md)

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
|---|---|---|---|---|---|
| parent session creation | disk write failure under `$SUBSTRATE_HOME` | planned | yes | yes | no |
| child manifest creation after parent | manifest write failure leaves orphan parent | planned | yes | yes | no |
| durable ownership establishment | wrapper exits before UAA session handle surfaces | planned | yes | yes | no |
| parent-child invalidation | stale child remains on disk after control loss | planned | yes | yes | no |
| status projection | parent missing but child remains live-shaped | planned | yes | yes | no |
| toolbox publication | active-looking child references inactive parent | planned | yes | yes | no |
| shell command completion | no active runtime but helper tries to emit `AgentEvent` | planned | yes | yes | no |
| host exec stream output | non-orchestrator stream path mints orchestration event | planned | yes | yes | no |
| multiple active parents | wrong endpoint or session chosen silently | planned | yes | yes | no |

Critical gap rule:

If any production path can emit or authorize using `orchestration_session_id` without a persisted active parent session, the slice is not done.

## Performance Review

No major performance risk if the implementation stays boring.

Hard rules:

- do not broad-scan the entire runtime store on hot event paths
- prefer `load_orchestration_session(<id>)` when a child already knows the parent id
- keep parent session files metadata-only
- do not add an in-memory cache to hide slow filesystem lookup. The correct answer here is direct lookup, not cache invalidation theater

One footgun to avoid:

If `agent status` or `toolbox` start rediscovering parents by scanning every file on every request, correctness survives but the implementation gets sloppy. Use direct parent lookup whenever the child already carries the session id.

## Worktree Parallelization Strategy

This plan has limited but real parallelization opportunity. The foundation has to land first. After that, there are two bounded lanes and one cleanup lane.

### Dependency table

| Step | Modules touched | Depends on |
|---|---|---|
| Parent session model and store APIs | `crates/shell/src/execution/agent_runtime/` | - |
| Bootstrap and lifecycle cutover | `crates/shell/src/repl/`, `crates/shell/src/execution/agent_runtime/` | parent session model |
| Event emission cutover | `crates/shell/src/execution/agent_events.rs`, `crates/shell/src/execution/routing/dispatch/`, `crates/shell/src/repl/` | parent session model |
| Status/toolbox parent gating | `crates/shell/src/execution/agents_cmd.rs`, `crates/shell/src/execution/agent_runtime/` | parent session model |
| Tests and docs | `crates/shell/tests/`, `docs/`, touched runtime modules | bootstrap + events + status/toolbox |

### Parallel lanes

Lane A: parent session model and store APIs -> bootstrap and lifecycle cutover  
Why: `async_repl.rs` depends on the new record type and store helpers, and both steps share `agent_runtime/`.

Lane B: event emission cutover  
Why: once the parent-session API exists, event plumbing can proceed mostly independently. It still has one shared seam in `async_repl.rs`.

Lane C: status/toolbox parent gating  
Why: this can start as soon as the parent-session read helpers are stable. It does not need bootstrap logic to finish if helper names and record shape are already fixed.

Lane D: tests and docs  
Why: runs after A + B + C settle so it can assert the final contract instead of chasing moving signatures.

### Execution order

1. Launch Lane A first. This is the foundation and it owns the shared type definitions.
2. Once Lane A lands the parent-session API surface, launch Lane B and Lane C in parallel worktrees.
3. Merge B and C.
4. Run Lane D last to update tests and docs against the final seam.

### Conflict flags

- Lane A and Lane B both touch `crates/shell/src/repl/async_repl.rs`
- Lane A and Lane C both touch `crates/shell/src/execution/agent_runtime/state_store.rs`

That means the safe split is:

- Lane A lands first
- Lane B and Lane C branch after Lane A, not before

This is not a "start four workers immediately" plan. It is a "foundation first, then two bounded follow-on lanes" plan.

## Deferred Work

There is no `TODOS.md` in this repo root today, so deferred items are captured here explicitly.

1. Session-centric handle-store layout and grouped APIs  
Why: needed for later concurrent session reasoning, but not needed to fix identity authority now.

2. Participant-generalized runtime record model  
Why: needed for member sessions and lineage, but this slice only needs one parent session plus one current orchestrator child.

3. Shared-world owner binding and generation invalidation  
Why: depends on having a real parent session first.

4. Public multi-session selection, resume, or fork UX  
Why: not required to make current runtime identity honest.

## NOT in Scope

- redesigning shell trace `session_id`
- renaming `uaa.agent.session`
- changing the `handles/` directory layout
- member runtime orchestration
- mutating toolbox tools or toolbox server work
- cross-process resume/fork productization
- cross-platform parity polish beyond carrying the additive parent-session model

## Definition of Done

The slice is done when all of these are true:

1. `orchestration_session_id` is not allocated from a process-global singleton anywhere in production runtime code.
2. A top-level orchestration session record is persisted separately from child handle manifests.
3. Parent session creation happens before child manifest creation in bootstrap.
4. A child manifest cannot be authoritative-live unless its parent session exists and is `Active`.
5. Attached-control loss invalidates both parent and child.
6. `substrate agent status` and `substrate agent toolbox *` only project active state when both parent and child validate.
7. Ordinary shell-only command and host-stream paths do not emit fake orchestration `AgentEvent` rows.
8. Trace rows for real orchestration work still carry both shell `session_id` and real `orchestration_session_id`.
9. Docs and regression tests lock the new contract.

## Completion Summary

- Step 0: Scope Challenge - scope accepted as-is
- Architecture Review: 4 material findings
- Code Quality Review: folded into architecture contract and implementation rules
- Test Review: coverage diagram produced, 15 concrete assertions/gaps identified
- Performance Review: 0 major issues, 3 direct-lookup/no-cache rules
- NOT in scope: written
- What already exists: written
- TODOS.md updates: deferred scope captured in-plan because no `TODOS.md` exists
- Failure modes: 0 unresolved critical gaps if the explicit-context rule is followed
- Outside voice: codex-grounded consolidation pass
- Parallelization: foundation first, then 2 bounded parallel lanes, then test/doc cleanup
- Lake Score: complete version chosen for every in-slice decision

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
|---|---|---|---|---|---|---|
| 1 | Scope | Add a first-class parent orchestration session record | Mechanical | Completeness | This slice exists to make identity owned, persisted, and lifecycle-aware | Opportunistic persistence of the global helper |
| 2 | Scope | Keep shell `session_id` unchanged | Mechanical | Minimal diff | The bug is orchestration authority, not shell trace identity | Trace-session redesign |
| 3 | Scope | Keep grouped store migration out of this slice | Mechanical | Blast radius | Identity authority and session-centric storage are separate problems | Pulling in `06` now |
| 4 | Architecture | Keep parent persistence in `state_store.rs` | Mechanical | DRY | The repo already has the atomic JSON helper and storage root | New store service |
| 5 | Lifecycle | Parent becomes `Active` only after durable ownership exists | Mechanical | Systems over heroes | Prevents ghost sessions that never really came alive | Mark parent active at allocation time |
| 6 | Events | `AgentEvent` schema stays strict; no weakening to optional orchestration id | Mechanical | Explicit over clever | Fake events are worse than absent events | Making `orchestration_session_id` optional on `AgentEvent` |
| 7 | Events | `publish_command_completion(None, ...)` emits nothing | Mechanical | Honest boundaries | Shell-only commands must not pretend to be orchestration work | Auto-minting or fake placeholder ids |
| 8 | Events | `emit_stream_chunk(None, ...)` still prints bytes but emits no `AgentEvent` | Mechanical | Honest boundaries | Host exec streaming is not automatically orchestration-scoped | Reaching into a global allocator |
| 9 | Operator surfaces | Status and toolbox share one parent-aware resolver | Mechanical | DRY | One validation contract is safer than two almost-identical ones | Surface-specific liveness logic |
| 10 | Operator surfaces | Ambiguity between active parents fails closed | Mechanical | Boring by default | Silent newest-wins routing is dangerous on control-plane surfaces | Implicit latest-session selection |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/autoplan` | Scope & strategy | 1 | CLEAR | Correct first slice, explicit non-goals, no absorption of later packet work |
| Codex Review | `/codex review` | Independent 2nd opinion | 1 | CLEAR | Parent-first bootstrap, explicit event context, parent-gated operator surfaces, no fallback allocator |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 1 | CLEAR | Architecture contract tightened, hidden event callers named, regression and QA obligations made explicit |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | SKIPPED | No UI scope |

**CODEX:** The real cleanup was not adding more structure. It was removing every place the plan still relied on implied authority, implied caller context, or implied operator truth.

**UNRESOLVED:** 0 blocking design decisions remain inside this slice. Deferred items are intentionally postponed to later `llm-last-mile` packet files.

**VERDICT:** ENG CLEARED - ready to implement `PLAN-01` as the first prerequisite before moving on to participant, world, or grouped-store follow-ons.
