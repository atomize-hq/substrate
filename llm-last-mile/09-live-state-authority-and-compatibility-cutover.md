# SOW: Live-State Authority Boundary and Compatibility Cutover

Status: implementation-oriented scope. This document freezes the production contract for live-state authority across canonical session-root records, flat compatibility files, and trace fallback so the remaining cleanup work does not reintroduce silent heuristics or split authority.

## Objective

Make one boundary authoritative for live agent-hub state:

- canonical parent session records under `sessions/<orchestration_session_id>/session.json`,
- canonical participant records under `sessions/<orchestration_session_id>/participants/<participant_id>.json`,
- store-owned live-session resolution for operator surfaces,
- bounded compatibility reads and writes only where required during cutover,
- trace fallback limited to historical gap-filling and never elevated into control-plane authority.

This SOW exists to remove the last ambiguity around "what is truth" for:

- `substrate agent status`
- `substrate agent toolbox status`
- `substrate agent toolbox env`
- the cutover posture for flat compatibility files and legacy `handles/*.json`

## Why Needed

The repo now contains the right pieces, but the final authority contract is still easy to misread because it is spread across runtime code, tests, and docs:

- the store already prefers canonical session-root objects and degrades torn roots with warnings:
  [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- `status` already overlays live session records ahead of trace fallback and suppresses stale trace rows behind tombstones:
  [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- `toolbox status|env` already resolves one live session through the store and fails closed on ambiguity or broken parent/child linkage:
  [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- docs now state that canonical session-root records outrank flat compatibility files and trace, but the write-side retirement decision is not yet captured in one execution-ready SOW:
  [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
  [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)

Without one explicit cutover contract, follow-on cleanup work can easily regress into one of these bad states:

- flat compatibility files regaining live-state authority,
- trace rows being used to authorize current toolbox sessions,
- ambiguous multi-session orchestrator state silently picking "latest",
- dual-write being removed too early without proving no compatibility readers still need it,
- torn-root crash windows turning into accidental operator-facing false positives.

## Current Repo Seams

### Live-state store and precedence

Primary owner:

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Key seams:

- `load_authoritative_session(...)`
- `build_session_record(...)`
- `list_sessions()`
- `list_live_sessions()`
- `resolve_single_live_session_for_agent(...)`
- `list_invalidated_participants_across_sources()`
- `persist_orchestration_session(...)`
- `persist_participant(...)`
- `persist_parent_session_snapshot(...)`

This file already encodes the real precedence rule:

1. canonical session-root parent record,
2. flat compatibility parent record only if canonical parent is absent,
3. canonical participant record,
4. flat compatibility participant record,
5. legacy `handles/*.json` only as last-resort compatibility input.

### Parent session model

- [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)

This file owns the parent-session state used by live discovery:

- `state`
- `active_session_handle_id`
- `world_id`
- `world_generation`
- `shell_owner_pid`

### Operator surfaces

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Key seams:

- `build_status_report(...)`
- `live_session_status_projections(...)`
- `build_toolbox_status_report(...)`
- `build_toolbox_env_report(...)`
- trace fallback suppression helpers

### Runtime writer choke point

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Key seam:

- `persist_runtime_snapshots(...)`

This is where parent and participant snapshots enter the store and where any write-side compatibility bridge must remain centralized.

### Existing contract tests

Primary regression anchor:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)

This suite already locks most of the intended behavior for:

- canonical-over-flat precedence,
- flat fallback when canonical roots are incomplete,
- trace fallback suppression,
- tombstone suppression,
- fail-closed ambiguity and parent/child mismatch handling,
- doc wording around live-state authority.

## In Scope

- freezing the live-state authority order across canonical records, flat compatibility files, and trace fallback
- deciding whether dual-write remains intentional during the cutover and where it is allowed to live
- defining exact ambiguity and torn-root behavior for `status` and `toolbox`
- tightening doc cleanup so active docs describe the same authority boundary as runtime code
- preserving `PLAN-05` tombstone suppression while cleaning up compatibility behavior
- defining the removal gates for flat compatibility writes and legacy `handles/*.json` reads

## Out of Scope

- redesigning the parent session schema
- changing the public JSON fields on `substrate agent status` or `substrate agent toolbox status`
- changing trace schema or adding new trace families
- introducing a new transactional runtime registry
- reworking world replacement semantics from [05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md)
- removing compatibility reads or writes in the same step that merely documents the contract

## Blockers And Gaps

### 1. Flat-file retirement is not yet proven safe

The repo still intentionally reads:

- `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>.json`
- `~/.substrate/run/agent-hub/participants/*.json`
- `~/.substrate/run/agent-hub/handles/*.json`

That means immediate compatibility-write removal is not justified until:

- in-repo fixtures stop depending on flat fallback except where explicitly testing migration behavior,
- active docs stop describing flat files as anything other than compatibility input,
- an owner confirms there are no required off-repo consumers of flat session or participant files.

### 2. Operator semantics are spread across tests and code paths

The runtime already fails closed on the right ambiguity classes, but the rules currently live in:

- store error strings,
- `toolbox env` exit-code routing,
- status suppression logic,
- individual contract tests.

This SOW must unify those into one execution order so later cleanup does not relax them accidentally.

### 3. Torn-root behavior is intentionally degraded, not transactional

Current code tolerates:

- parent-only canonical roots,
- participant-only canonical roots,
- flat compatibility participant fallback when the canonical parent exists but the canonical child is missing.

That is the correct current posture, but it must stay explicit. The cutover does not authorize pretending these states are impossible.

## Required Semantics And Invariants

### 1. Authority order

For live-state operator surfaces, truth is ordered exactly like this:

1. canonical parent session record:
   `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/session.json`
2. canonical participant record:
   `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/participants/<participant_id>.json`
3. flat compatibility parent and participant files only when needed to fill an incomplete canonical root on read
4. legacy `handles/*.json` only as compatibility input of last resort
5. trace fallback only for historical status projection gaps, never for current-session authority

Non-negotiable rules:

- canonical objects outrank conflicting flat compatibility objects
- flat compatibility input may fill a torn-root gap but may not overrule a present canonical object
- legacy `handles/*.json` is never authoritative live-state input
- trace never outranks canonical or flat runtime state

### 2. Dual-write decision

Dual-write remains intentional during this cutover, but only in a bounded form:

- it is allowed only inside [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- it may write flat compatibility copies for:
  - `sessions/<orchestration_session_id>.json`
  - `participants/<participant_id>.json`
  - flat lease files
- it must not be duplicated in callers
- it must not reintroduce new write ownership for `handles/*.json`

Required posture:

- canonical session-root writes are the real write target
- flat compatibility writes are a temporary migration bridge only
- `handles/*.json` remains read-only compatibility input and should be removed last

Removal gate:

- remove flat dual-write only after compatibility reads and fixtures are proven unnecessary outside explicit migration tests

### 3. `substrate agent status` semantics

`status` must:

- build authoritative live rows from `list_live_sessions()`
- preserve one row per live participant, not one row per `agent_id`
- use trace fallback only for tuples not already covered by a live record or invalidated tombstone
- suppress stale trace fallback when `list_invalidated_participants_across_sources()` finds an invalidated tuple for the same `(orchestration_session_id, agent_id, execution.scope)`

`status` must not:

- silently collapse concurrent sessions for the same orchestrator agent
- resurrect invalidated world members from trace
- treat historical trace rows as proof that a toolbox/control-plane session is currently live

### 4. `substrate agent toolbox status|env` semantics

`toolbox status` and `toolbox env` must resolve through `resolve_single_live_session_for_agent(...)`.

Required behavior:

- exactly one valid live host-scoped orchestrator session:
  - `toolbox status` reports concrete endpoint, `active_orchestration_session_id`, and optional parent `world_id` / `world_generation`
  - `toolbox env` succeeds
- zero valid live host-scoped orchestrator sessions:
  - `toolbox status` returns `dependency_unavailable`
  - `toolbox env` exits `3`
- more than one valid candidate or any broken parent/child linkage:
  - fail closed
  - keep the error operator-readable
  - never pick the newest candidate heuristically

Ambiguity or corruption classes that must fail closed:

- multiple active parent sessions for one orchestrator agent
- multiple live host-scoped orchestrator participants for one orchestrator agent
- live child exists without an active parent
- active parent missing `active_session_handle_id`
- active parent points at a missing participant
- active parent points at an inactive participant
- parent and participant disagree on `orchestration_session_id`
- parent points at a non-host or non-orchestrator participant

### 5. Torn-root behavior

Torn roots must degrade, not self-authorize:

- participant-only roots may be discovered by `list_sessions()`, but remain incomplete and non-live
- parent-only roots may be discovered by `load_session()`, but remain incomplete and non-live
- warnings must be preserved on incomplete session records
- incomplete roots must not be promoted by `list_live_sessions()`
- flat compatibility participant fallback remains allowed when the canonical parent exists but the canonical child is absent
- trace fallback must not hide torn-root warnings by pretending current liveness exists

### 6. Trace boundary

Trace remains the historical event log, not the live registry.

Required rules:

- trace may fill status gaps only after live runtime state and tombstones have had first priority
- trace must not authorize `toolbox env`
- trace must not authorize orchestrator control-plane health or selection
- trace must not resurrect invalidated participants

## Exact Code And Doc Areas

### Primary implementation surfaces

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
  - precedence
  - torn-root completeness rules
  - dual-write ownership
  - legacy-read retirement sequencing
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
  - status live-row selection
  - trace fallback suppression
  - toolbox ambiguity handling
  - exit-code mapping for unavailable versus unsupported versus denied
- [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
  - parent fields that define active-session authority
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  - `persist_runtime_snapshots(...)`
  - ensuring store-owned canonical writes remain the only write choke point

### Primary docs to align

- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
  - canonical live-state authority wording
  - trace fallback and tombstone suppression wording
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
  - operator-facing live discovery wording
  - toolbox `dependency_unavailable` posture

### Secondary doc audit if active wording still exists

- [docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/compatibility-spec.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/compatibility-spec.md)
- [docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/manual_testing_playbook.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/manual_testing_playbook.md)

These are not the primary authority files for runtime discovery, but they should not contradict the cutover decision.

## Testing Requirements

At minimum, preserve or extend regression coverage for:

1. canonical session-root records outrank conflicting flat compatibility files
2. flat compatibility participant fallback remains readable when the canonical parent exists but the canonical child is missing
3. participant-only and parent-only torn roots stay incomplete and never become live
4. `toolbox env` fails closed when trace is the only evidence of liveness
5. `toolbox status|env` fail closed on all ambiguity and parent/child mismatch classes
6. `status` prefers live runtime state over newer trace fallback
7. invalidated tombstones suppress stale trace rows for the same tuple
8. dual-write continues to produce canonical roots plus flat compatibility copies while the bridge remains intentional
9. no new test or fixture depends on `handles/*.json` as the primary write target

Primary anchors:

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
  - `persist_writes_canonical_and_flat_compatibility_layouts`
  - `list_sessions_discovers_participant_only_roots_and_excludes_them_from_live_results`
  - `parent_only_torn_roots_degrade_with_warnings_instead_of_failing_discovery`
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
  - `agent_toolbox_env_trace_history_does_not_authorize_active_session`
  - `agent_toolbox_surfaces_prefer_canonical_session_roots_over_flat_compatibility_files`
  - `agent_toolbox_surfaces_fall_back_to_flat_participant_when_canonical_root_is_incomplete`
  - `operator_surfaces_fail_closed_when_multiple_active_parent_candidates_exist`
  - `agent_status_prefers_live_manifest_over_trace_fallback_for_selected_orchestrator`
  - `agent_status_tombstone_suppression_beats_stale_trace_fallback_for_world_member`

Recommended verification commands:

```bash
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell agent_successor_contract_ahcsitc0 -- --nocapture
```

## Acceptance Criteria

- canonical session-root parent and participant records are the documented and implemented live-state authority
- flat `sessions/*.json`, `participants/*.json`, and flat lease files are explicitly temporary compatibility input/output only during the bridge
- `handles/*.json` is never described or treated as the authoritative live-state registry
- dual-write, while it exists, is store-owned only and does not leak into callers
- `substrate agent status` keeps concurrent sessions distinct and does not resurrect invalidated tuples from trace
- `substrate agent toolbox status|env` fail closed on ambiguity, missing parent linkage, or inactive selected participants
- torn roots remain discoverable with warnings but do not become live
- active docs describe the same authority boundary as code and tests

## Recommended Execution Order

### 1. Freeze the boundary first

Lock the precedence and dual-write decision in the store-owned contract:

- canonical root is truth
- flat compatibility data is bounded bridge input/output
- `handles/*.json` is legacy input only
- trace is historical fallback only

### 2. Keep operator surfaces strict

Audit `status` and `toolbox` against that boundary:

- no caller-owned regrouping
- no heuristic "latest" selection
- no trace-based control-plane authorization
- no promotion of incomplete roots into live discovery

### 3. Finish doc cleanup while behavior is still fresh

Update active docs so operators see the same rule set that tests enforce:

- one authority statement in `TRACE.md`
- one operator statement in `USAGE.md`
- secondary compatibility-pack wording only if it contradicts the runtime contract

### 4. Retire the bridge last

Only after the above is green:

- stop relying on flat compatibility reads except in explicit migration tests
- remove flat dual-write
- remove `handles/*.json` reads last

This sequence keeps the repo safe under concurrent development because it removes stale wording and hidden heuristics before it removes compatibility scaffolding.

## Relationship To Other `llm-last-mile` Work

This scope is downstream of:

- [05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md)
- [06-session-centric-state-store.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/06-session-centric-state-store.md)

It should be read as the cutover and cleanup contract on top of those earlier slices:

- `05` defines invalidation and tombstone meaning
- `06` defines session-centric store structure
- this file defines which sources remain authoritative during the migration bridge and how operator surfaces must behave when those sources disagree or are incomplete
