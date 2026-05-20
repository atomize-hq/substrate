# SOW: Rework Agent Runtime Handle Store to Be Session-Centric

## Objective

Rework the current agent runtime handle store so lookup, enumeration, and consumer APIs are centered on `orchestration_session_id`, not on "latest live orchestrator handle for agent X".

This prerequisite exists because the current store shape and API are optimized for a single selected orchestrator lookup, while the rest of the emerging contract surface is already session-centric:

- `AgentSessionHandleV1` makes `orchestration_session_id` the grouping key for the orchestrator session and all member sessions:
  [docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md)
- toolbox socket placement is already keyed by `orchestration_session_id`:
  [docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md)
- `substrate agent status` and `substrate agent toolbox ...` already expose `active_orchestration_session_id` and session rows keyed by `orchestration_session_id`:
  [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

The prerequisite outcome is an implementation-ready store API that can answer session questions directly, preserves current manifest semantics, and provides a bounded migration path for existing flat-handle readers and tests.

## Scope

In scope:

- runtime state-store layout under `$SUBSTRATE_HOME/run/agent-hub`
- read/write APIs in the shell-owned runtime store
- migration/compatibility for existing flat handle files
- consumer updates needed for `substrate agent status` and `substrate agent toolbox status|env`
- test and validation expectations for the state-store cutover

Out of scope:

- changing the public `AgentRuntimeSessionHandle` field schema
- introducing toolbox mutating tools
- redesigning trace event semantics
- solving broader multi-session CLI selection UX beyond what is required to remove the orchestrator-centric store heuristic

## Current State

### Current storage layout

The authoritative runtime store is currently a flat directory:

- manifests: `$SUBSTRATE_HOME/run/agent-hub/handles/<session_handle_id>.json`
- leases: `$SUBSTRATE_HOME/run/agent-hub/handles/<session_handle_id>.lease`

Owned by:

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

### Current write path

The host orchestrator runtime in the async REPL allocates:

- `session_handle_id = ash_<uuid>`
- `lease_token = <uuid>`
- `orchestration_session_id = agent_events::orchestration_session_id()`

and persists the manifest immediately, then rewrites it across lifecycle transitions:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)

### Current read path

The store API is flat-file oriented:

- `list_manifests()`
- `list_live_manifests()`
- `find_live_orchestrator(agent_id)`

The last API is the core problem. It ignores session identity and returns the latest live manifest for a given `agent_id`, globally.

### Current consumer behavior

1. `substrate agent toolbox status` / `env`

- Select orchestrator from config/policy.
- Call `AgentRuntimeStateStore::find_live_orchestrator(&orchestrator.file.id)`.
- Use the returned manifest's `orchestration_session_id` to build the UDS endpoint.

Anchor:

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

2. `substrate agent status`

- Reads live manifests.
- Collapses live-manifest overlay by `agent_id`.
- Falls back to trace-derived session projections when no live manifest is found.

This means the live overlay is still effectively agent-centric even though the JSON surface is session-centric.

3. Tests and fixtures

Existing tests write fake runtime manifests directly into the flat `handles/` directory:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)

## Problem Statement

The current API shape is mismatched with the repo's session contract.

Concrete mismatches:

- The contract says `orchestration_session_id` is the grouping identity for orchestrator + members, but the store's convenience lookup is keyed by `agent_id`.
- Toolbox endpoint placement is session-keyed (`<orchestration_session_id>.sock`), but toolbox discovery is "latest live orchestrator for this agent".
- Status is supposed to present session rows, but the live-manifest overlay de-duplicates by `agent_id` instead of preserving multiple concurrent sessions.
- Future toolbox/session-history/list-sessions work in ADR-0026/ADR-0045 needs direct session lookup and session-local enumeration, not a global orchestrator scan.

Failure modes if unchanged:

- wrong toolbox endpoint chosen when the same orchestrator agent has more than one live orchestration session
- inability to reason about concurrent sessions cleanly
- more consumer-specific heuristics added on top of a store that already lost the primary grouping key
- status/toolbox drift where both surfaces reconstruct "current session" differently

## Target Outcome

Make `orchestration_session_id` the primary read boundary while keeping `session_handle_id` as the unique handle identity.

The store must support:

- persist by handle
- enumerate by session
- load one session and its handles directly
- list live sessions without collapsing them by agent
- resolve the live handle for an `(orchestration_session_id, agent_id)` pair
- optionally provide a transition shim for "single live session for orchestrator agent" that fails closed on ambiguity instead of silently choosing newest

## Proposed Storage Layout

### Canonical layout

Adopt a session-grouped runtime tree:

```text
$SUBSTRATE_HOME/run/agent-hub/
  sessions/
    <orchestration_session_id>/
      handles/
        <session_handle_id>.json
      leases/
        <session_handle_id>.lease
```

Notes:

- `session_handle_id` remains the stable per-handle filename.
- `orchestration_session_id` becomes the directory partition and primary lookup anchor.
- Keep manifest JSON payloads unchanged except for any additive metadata strictly required by migration bookkeeping.
- Keep lease JSON payload semantics unchanged unless a new field is needed to aid compatibility reads.

### Why this layout

- It matches the ownership semantics already stated in `AgentSessionHandleV1`.
- It makes "list all handles in session X" a direct filesystem operation instead of a full-store scan plus filter.
- It aligns the storage root with toolbox endpoint naming and future `list_sessions` / `get_session_history` style consumers.
- It avoids inventing a second derived index file until the implementation proves one is necessary.

## Proposed API Changes

Primary owner:

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

### New/updated types

Add a session-grouped projection type in the runtime-store layer:

```rust
pub(crate) struct AgentRuntimeSessionRecord {
    pub orchestration_session_id: String,
    pub manifests: Vec<AgentRuntimeSessionManifest>,
}
```

Required helpers on the record:

- `live_manifests()`
- `latest_manifest_for_agent(agent_id)`
- `live_manifest_for_agent(agent_id)`
- `live_orchestrator_manifest()`
- `last_updated_at()`

### Required store API

Minimum API surface:

```rust
impl AgentRuntimeStateStore {
    pub(crate) fn persist_manifest(&self, manifest: &AgentRuntimeSessionManifest) -> Result<()>;

    pub(crate) fn load_session(
        &self,
        orchestration_session_id: &str,
    ) -> Result<Option<AgentRuntimeSessionRecord>>;

    pub(crate) fn list_sessions(&self) -> Result<Vec<AgentRuntimeSessionRecord>>;

    pub(crate) fn list_live_sessions(&self) -> Result<Vec<AgentRuntimeSessionRecord>>;

    pub(crate) fn find_live_session_handle(
        &self,
        orchestration_session_id: &str,
        agent_id: &str,
    ) -> Result<Option<AgentRuntimeSessionManifest>>;

    pub(crate) fn resolve_single_live_session_for_agent(
        &self,
        agent_id: &str,
    ) -> Result<Option<AgentRuntimeSessionRecord>>;
}
```

Rules:

- `load_session` and `find_live_session_handle` are the new primary read APIs.
- `resolve_single_live_session_for_agent` is transitional only. It must fail closed when more than one live session exists for the same orchestrator agent.
- Remove or stop using `find_live_orchestrator(agent_id)` in production code.
- `list_live_manifests()` may remain as a low-level helper if still useful internally, but high-level consumers should not reconstruct sessions from a flat manifest list when session records are available.

## Consumer Changes

### 1. Toolbox status/env

Consumer:

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Required change:

- Stop treating "selected orchestrator agent id" as a sufficient lookup key.
- Switch to store APIs that return a session record or fail with ambiguity.

Expected behavior:

- `substrate agent toolbox status` still reports the endpoint template even when no session is live.
- When exactly one live session exists for the selected orchestrator, publish its concrete endpoint and `active_orchestration_session_id`.
- When more than one live session exists for the selected orchestrator and the CLI has no explicit selector, fail closed with `dependency_unavailable` or equivalent operator-readable ambiguity text. Do not silently pick the newest session.

This keeps the current CLI surface stable while removing the incorrect hidden heuristic.

### 2. Agent status

Consumer:

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Required change:

- Build the live overlay from `list_live_sessions()` or session records.
- Preserve distinct live rows per `(orchestration_session_id, agent_id)` instead of per `agent_id`.

Status-specific rule:

- Trace fallback remains useful for historical/nested correlation, but authoritative live runtime state must come from the session-centric store.
- The live overlay must not hide one live session just because the same `agent_id` is active in another orchestration session.

### 3. Async REPL runtime writer

Consumer/writer:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Required change:

- Persist into the session-grouped tree using the already-available `manifest.handle.orchestration_session_id`.
- No lifecycle semantics change is required for this prerequisite. This is a pathing/API change, not a state-machine redesign.

### 4. Test fixtures

Consumers:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- any direct runtime-store tests added under `crates/shell/src/execution/agent_runtime/` or `crates/shell/tests/`

Required change:

- Stop seeding only `run/agent-hub/handles/*.json`.
- Seed session-grouped fixtures at `run/agent-hub/sessions/<orchestration_session_id>/handles/*.json`.
- During migration coverage, add legacy flat fixtures too, so read compatibility is verified explicitly.

## Migration and Compatibility

### Compatibility requirement

This prerequisite should not strand existing manifests already written by current builds or test fixtures.

### Migration plan

Phase 1: additive reader support

- Add canonical session-grouped read/write support.
- Add legacy read compatibility for the current flat `run/agent-hub/handles/*.json` layout.
- Prefer canonical session-grouped data when both layouts exist for the same handle.

Phase 2: dual-write bridge

- Update `persist_manifest` to write the new canonical session-grouped files.
- Optionally dual-write the legacy flat files for one bounded transition window if needed to keep parallel in-flight work or existing tests stable.
- If dual-write is used, keep the legacy write path inside the store, not duplicated in callers.

Phase 3: consumer cutover

- Move toolbox/status/runtime tests to the new store API.
- Remove all production reliance on `find_live_orchestrator(agent_id)` or equivalent "latest for agent" scans.

Phase 4: legacy retirement

- After in-repo consumers and tests are fully cut over, remove legacy flat-write behavior.
- Keep legacy flat-read support only if there is a real upgrade requirement for existing `~/.substrate` state; otherwise remove it in the same bounded change once acceptance coverage is green.

### Compatibility invariants

- `session_handle_id` remains globally unique and unchanged.
- `orchestration_session_id` meaning does not change.
- manifest lifecycle fields and ownership checks stay authoritative
- ownership validity still gates "live" exactly as today
- no change to toolbox endpoint naming: it remains `<orchestration_session_id>.sock`

## Sequencing

Recommended execution order:

1. Refactor store internals and add session-grouped path helpers in
   [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs).
2. Add session record type and session-centric tests at the store layer.
3. Update the async REPL writer in
   [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
   to write canonical session paths.
4. Cut `substrate agent toolbox status|env` over to the new read API and ambiguity handling.
5. Cut `substrate agent status` live overlay over to session records.
6. Update fixtures in
   [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
   and any new store tests.
7. Remove legacy orchestrator-centric helpers after all call sites are gone.

## Acceptance Criteria

### Functional

- Runtime manifests are stored canonically under `run/agent-hub/sessions/<orchestration_session_id>/...`.
- The store can load all handles for one `orchestration_session_id` without scanning unrelated sessions.
- Production consumers no longer use a global "latest live manifest for agent_id" lookup as their primary session-resolution mechanism.
- `substrate agent status` can represent multiple concurrent live sessions for the same `agent_id` when they have different `orchestration_session_id` values.
- `substrate agent toolbox status` and `env` derive their endpoint from a resolved session record, not from a global latest-by-agent heuristic.

### Safety / fail-closed

- If two live sessions exist for the selected orchestrator and no explicit session selector exists, toolbox lookup fails closed instead of choosing one implicitly.
- Legacy flat files do not override newer canonical session-grouped files when both exist.
- Ownership checks (`is_authoritative_live` plus owner-process validity) remain in force before any session is treated as live.

### Code-shape

- `find_live_orchestrator(agent_id)` is removed or clearly demoted to a compatibility shim with ambiguity detection.
- Session grouping lives in the store layer, not reimplemented independently in each consumer.

## Validation and Testing Suggestions

### Unit / focused tests

Add or extend store-level tests to cover:

- persisting a manifest into the new session-grouped layout
- loading one session with multiple handles
- listing live sessions when multiple sessions share the same `agent_id`
- ambiguity detection for `resolve_single_live_session_for_agent`
- canonical-path preference when both canonical and legacy files exist
- legacy flat-read compatibility during the migration window

### Integration tests

Update/add coverage in:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs) if live/runtime persistence expectations move

Important scenarios:

- toolbox status with no live session still returns endpoint template
- toolbox env prefers canonical live session state over trace history
- toolbox env fails closed when a live session is invalidated
- toolbox lookup fails closed when two live orchestration sessions exist for the same orchestrator agent
- agent status emits two live rows for the same agent across two orchestration sessions

### Suggested commands

Use targeted shell crate checks first:

```bash
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test agent_hub_trace_persistence -- --nocapture
cargo test -p shell agent_runtime -- --nocapture
```

Then run broader guardrails if the implementation touches shared status rendering:

```bash
cargo test -p shell -- --nocapture
cargo clippy -p shell --all-targets -- -D warnings
```

## Risks

- Multi-session ambiguity is real now that the store is being made honest; consumers that previously assumed one live orchestrator per agent may need explicit fail-closed behavior.
- Dual-write periods can drift if canonical and legacy writes are not kept inside one store method.
- Status ordering may change once live sessions are no longer collapsed by `agent_id`; tests must assert the contract's documented sort order, not the old incidental behavior.
- There may be hidden direct readers of `run/agent-hub/handles/` outside the obvious shell tests; repo-wide search should be repeated immediately before removing legacy support.

## Open Questions

1. Should toolbox commands gain an explicit `--orchestration-session-id` selector in a follow-on slice, or is fail-closed ambiguity sufficient for v1?
2. Is legacy flat-read support needed for real on-disk upgrade compatibility, or only for in-repo tests during the cutover?
3. Should session-level summary/index files be added later for performance, or should the first implementation stay directory-scan based until there is evidence it is needed?

## Definition of Done

This prerequisite is complete when the runtime store is session-centric by API and layout, toolbox/status no longer depend on a global latest-by-orchestrator lookup, migration behavior is explicit, and the targeted shell tests cover canonical layout, compatibility, and ambiguity handling.
