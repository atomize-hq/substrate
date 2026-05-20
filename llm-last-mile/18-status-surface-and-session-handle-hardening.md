# SOW: Harden Agent Status and Session-Handle Contracts For Public Control Surfaces

Status: implementation-oriented follow-on draft. This SOW does not claim that `substrate agent status` or session persistence are missing. Live runtime state, canonical session/participant snapshots, participant-aware producer-side trace rows, and nested gateway tuple validation are already landed. The remaining gap is narrower and operator-facing: stop `agent status` from aborting wholesale on strict live-session selection failures, make trace-only fallback stop collapsing sibling participants, and clean up the local-vs-upstream session-handle contract enough that future public `start|resume|fork|stop` surfaces do not inherit ambiguous identifiers by default.

## Objective

Harden only the status/addressability layer needed before public control-plane work is safe:

- make `substrate agent status` degrade cleanly when parent-session selection is ambiguous or stale,
- make trace-only fallback participant-aware so same-agent sibling participants inside one orchestration session do not collapse,
- and lock one minimal public addressability contract:
  - public parent-session selectors use `orchestration_session_id`
  - `participant_id` remains a Substrate runtime/lineage identifier, not the primary public session selector
  - surfaced backend-native session ids remain internal implementation detail for now

This SOW does not add public `start|resume|fork|stop`. It only hardens the read-side and naming contract those later surfaces would rely on.

## Why This Is Needed

The repo already has the core primitives this slice should reuse rather than replace:

- the gap matrix now records live runtime persistence, trace production, and status/live fallback as real, not hypothetical, in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:76)
- the same matrix also calls out the remaining open rows this SOW should close:
  - `Session handles` in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:112)
  - `Status ambiguity handling` in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:125)
  - `Trace-only participant-aware fallback` in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:126)
- `AgentRuntimeStateStore` already has a permissive session-record model that can preserve warnings instead of panicking in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:858)
- live runtime status rows already preserve `participant_id` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1140)
- same-agent concurrent live world rows from store-owned session records are already supposed to remain visible, as pinned in [agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs:2528)

What is still wrong is a narrower mismatch between those landed primitives and the selected read-side/operator contract:

1. `build_status_report(...)` still hard-preflights `resolve_single_live_session_for_agent(...)` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:421), [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:446), and [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:433).
2. That strict helper is correct for active-control authorization, but it intentionally fails closed on:
   - multiple active parent sessions,
   - missing `active_session_handle_id`,
   - stale active-handle references,
   - inactive selected participants,
   - or live children without an active parent,
   as covered by tests in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1936) and [agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs:1468).
3. Reusing that same strict helper for `agent status` means a read-only surface aborts wholesale even though the store can already enumerate partial records and warnings.
4. The trace fallback path is still keyed too coarsely:
   - pure-agent trace rows use `(orchestration_session_id, agent_id)` in `pure_session_key(...)` at [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1252)
   - nested parent-run selection is keyed by `(orchestration_session_id, agent_id)` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:595)
   - fallback suppression is only `(orchestration_session_id, agent_id, execution.scope)` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1185)
5. That coarse key shape is enough for many rows, but it is not enough once one orchestration session can legitimately contain sibling participants for the same `agent_id`. The gap matrix already names this as open in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:126).
6. The persisted session-handle vocabulary is still overloaded:
   - `OrchestrationSessionRecord.active_session_handle_id` stores the selected participant id in [orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:35)
   - `AgentRuntimeParticipantHandle.session_handle_id` is skipped from serialization and mirrors `participant_id` in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:45)
   - deserialization still aliases legacy `session_handle_id`, `parent_session_handle_id`, and `resumed_from_session_handle_id` onto participant ids in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:528)
   - the real surfaced upstream backend handle lives separately as `internal.uaa_session_id` in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:75)

That identity mix is survivable for internal plumbing, but it is the wrong base contract for a public control plane.

## Relationship To Existing Slices

- This SOW is the narrow read-side follow-on to the current gap-matrix recommendation to "make `substrate agent status` degrade cleanly on ambiguity/stale parent linkage and make its trace fallback participant-aware" in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:179).
- It intentionally does not reopen backend targeting, turn submit/reuse, or the broader public caller-surface design. It only fixes the status/addressability substrate those later slices need.

## Current Relevant Code Surfaces

### Strict live-session selection and the current read-side mismatch

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:433)
  - `resolve_single_live_session_for_agent(...)`
  - intentionally strict control-plane selector
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:858)
  - `build_session_record(...)`
  - already preserves warnings and incompleteness instead of crashing
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:421)
  - `build_status_report(...)`
  - currently aborts status rendering by preflighting the strict single-live-session selector

### Trace fallback and selected-row correlation

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:473)
  - pure-agent trace projection
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:595)
  - selected parent-run correlation for nested rows
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1185)
  - fallback suppression key
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs:2263)
  - invalidated tombstones already beat stale trace fallback
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs:2528)
  - store-owned same-agent concurrent rows already must remain visible

### Local identity, participant lineage, and surfaced upstream handles

- [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:24)
  - authoritative parent session record, including `orchestration_session_id` and `active_session_handle_id`
- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:45)
  - local participant handle fields
- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:75)
  - `internal.uaa_session_id`
- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:528)
  - legacy wire aliases that currently conflate "session handle" and participant lineage

## In Scope

- remove the strict single-live-session preflight from `substrate agent status`
- degrade ambiguous or stale parent linkage into readable status output instead of a whole-surface abort
- make trace fallback participant-aware for pure-agent selection, nested parent-run correlation, and fallback suppression
- define one minimal public addressability contract:
  - later public control surfaces address orchestrations by `orchestration_session_id`
  - `participant_id` remains visible for diagnostics/lineage
  - `internal.uaa_session_id` stays internal and is not the public control-plane handle
- preserve backward-compatible reads for legacy `session_handle_id` aliases
- add focused regression coverage for the above and nothing broader

## Out Of Scope

- adding `substrate agent start|resume|fork|stop`
- redesigning backend selection or targeted-turn grammar
- changing the runtime authority model for host orchestrator or world member ownership
- widening toolbox into a mutation surface
- changing macOS/Lima parity status
- rewriting trace schema producers beyond the minimal identity fields already present in `AgentEvent`
- replacing existing persistence directories or compatibility dual-write layout

## Required Semantics And Invariants

### 1. Read-only status must degrade; control selectors must stay strict

`substrate agent status` should no longer depend on `resolve_single_live_session_for_agent(...)` as a precondition for rendering any output.

At the same time:

- helpers used to authorize active control should keep failing closed on ambiguity or stale linkage
- `toolbox env`, future mutating agent commands, and any public `resume|fork|stop` path should continue to require explicit authoritative selection
- trace history must not back-authorize control operations, as already pinned in [agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs:4129)

### 2. Live runtime state stays authoritative over trace fallback

The current precedence remains correct and must stay true:

- live authoritative participant/session records win
- invalidated authoritative participant tombstones suppress stale trace rows
- trace fallback only fills gaps the store cannot fill

This is already the intended behavior in [agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs:2208) and [agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs:2263). This SOW should harden, not invert, that ordering.

### 3. Participant identity must survive trace-only fallback

Where producer-side trace rows already include `participant_id`, `parent_participant_id`, and `resumed_from_participant_id`, the fallback reader must use those fields as part of its identity model instead of collapsing by only `(orchestration_session_id, agent_id)`.

Required rule:

- two sibling participants for the same agent within one orchestration session must be able to remain distinct in fallback selection and suppression if the trace evidence distinguishes them.

### 4. Public addressability must stop conflating three identities

For public/operator-facing contracts after this slice:

- `orchestration_session_id` is the only required public control-plane selector
- `participant_id` is a subordinate runtime identity for status, lineage, and debugging
- surfaced backend session identity remains separate and internal, backed today by `internal.uaa_session_id`

Required rule:

- do not expose `participant_id` under the public name `session_handle_id`
- do not ask operators to target `uaa_session_id` directly on public control surfaces

### 5. Compatibility reads may stay; public docs and response shapes should use canonical names

The repo already supports reading older wire names through serde aliases in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:528).

That compatibility may remain, but:

- new public docs and new JSON/text status contracts should prefer canonical names
- later public control-plane surfaces should not require users to understand legacy alias names

### 6. Do not weaken malformed-row validation

This SOW is about ambiguity/linkage degradation, not about accepting corrupt selected rows.

Current fail-closed validation should remain for:

- malformed world identity on selected world-scoped status rows in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:544)
- malformed nested tuple fields in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:626)
- malformed nested parent correlation in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:668)

The degradation target is session-selection ambiguity and stale parent linkage, not malformed selected tuples.

## Recommended Implementation Shape

1. Split selection logic cleanly:
   - strict active-session selection remains for control/authorization
   - permissive enumeration/projection is used for `agent status`

2. Rework `build_status_report(...)` around:
   - `list_live_sessions()`
   - session-record warnings/incompleteness
   - authoritative live participants
   - participant-aware trace fallback

3. Freeze the public naming contract now:
   - public parent-session handle: `orchestration_session_id`
   - diagnostic/runtime row id: `participant_id`
   - backend-native session id: internal only

4. Keep compatibility at the serde boundary only.
   - Read legacy `session_handle_id` aliases.
   - Stop treating them as canonical semantics in new reporting/control code.

## Concrete Work Breakdown

### 1. Separate status rendering from active-control preflight

- Remove the unconditional `resolve_single_live_session_for_agent(...)` preflight from `build_status_report(...)` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:446).
- Replace it with status-specific logic that:
  - lists authoritative live sessions and participants,
  - preserves rows that are internally valid,
  - and degrades broken parent-selection state into non-fatal warnings or omitted affected rows instead of aborting all status output.
- Keep the strict helper in place for `toolbox env`, future start/resume/fork/stop surfaces, and any other operation that needs to authorize a live control target.

### 2. Make trace fallback participant-aware

- Change the pure-agent fallback key in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1252) so it incorporates `participant_id` when trace evidence supplies it.
- Change fallback suppression keys in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1185) so one live or invalidated participant suppresses only its matching fallback row, not every sibling row for the same `(orchestration_session_id, agent_id, execution.scope)`.
- Make nested parent-run selection in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:595) participant-aware when selected parent rows carry participant identity, while preserving current run-id-based fail-closed validation.

### 3. Clean up the public addressability contract

- Canonicalize the runtime/status vocabulary around:
  - `orchestration_session_id`
  - `participant_id`
  - internal backend-native session id
- Stop treating `active_session_handle_id` and `session_handle_id` as public-facing semantics when they are really storing participant identity today.
- Keep backward-compatible reads for:
  - `session_handle_id`
  - `parent_session_handle_id`
  - `resumed_from_session_handle_id`
  but shift comments, docs, and any new response shapes toward canonical names.

### 4. Add the minimum status/read-side reporting needed for degraded cases

- Extend `agent status` output so degraded records are still intelligible.
- The exact text/JSON warning shape is implementation-defined, but the surface must make it clear when:
  - a row came from authoritative live runtime state
  - a row came from trace fallback
  - or some candidate rows were omitted due to stale/incomplete parent linkage

### 5. Lock the contract with focused regression tests

- Add status tests proving that ambiguity or stale active-parent linkage no longer aborts the entire `agent status` surface.
- Add trace-only tests proving that sibling participants for the same `agent_id` inside one orchestration session stay distinct when participant-aware trace fields are present.
- Add compatibility tests proving legacy `session_handle_id` aliases still deserialize, while canonical reporting uses participant/session terminology instead.

## Validation Expectations

At minimum, this slice should ship with targeted coverage in:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs:2208)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1936)
- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:564)

Expected assertions:

- `substrate agent status --json` still prefers live authoritative runtime rows over trace fallback.
- Invalidated authoritative tombstones still suppress stale trace rows.
- Same-agent concurrent live rows from authoritative session records remain visible.
- Trace-only fallback no longer collapses same-agent sibling participants when participant-aware lineage is available.
- Ambiguous or stale active-parent selection no longer aborts all `agent status` output.
- strict active-session selector tests still fail closed on ambiguity or stale linkage.
- Legacy `session_handle_id`-style snapshots still deserialize.
- Canonical status/control reporting treats `orchestration_session_id` as the public handle and `participant_id` as subordinate runtime identity.

## Explicit Non-Goals

- Do not use this slice to sneak in public `start|resume|fork|stop`.
- Do not relax active-control authorization by allowing trace-only history to pick a control target.
- Do not redesign on-disk storage layout or remove compatibility aliases yet.
- Do not broaden this into toolbox mutation work, submit/reuse behavior, or non-interactive invocation design.

## Open Risks

- The biggest risk is relaxing the wrong helper. If `resolve_single_live_session_for_agent(...)` is weakened instead of factoring status onto a different read path, future control surfaces will inherit unsafe ambiguity.
- The second risk is only partially fixing trace fallback, for example making pure-agent projections participant-aware but leaving suppression or nested-parent correlation keyed too coarsely.
- The session-handle cleanup must avoid a flag day. The repo already reads legacy names, and the safe path is compatibility reads plus canonical public naming, not storage churn for its own sake.
