# PLAN-18: Harden Agent Status Surfaces And Canonicalize Session Handle Semantics

Source SOW: [18-status-surface-and-session-handle-hardening.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/18-status-surface-and-session-handle-hardening.md)  
Gap matrix anchor: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Adjacent landed slices: [PLAN-15.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-15.md), [PLAN-16.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-16.md), [PLAN-17.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-17.md)  
Branch: `feat/session-centric-state-store`  
Base branch: `main`  
Plan type: read-side status/control-surface hardening, developer-facing shell/runtime behavior, no new control command family  
Review posture: `/autoplan` scope discipline with `/plan-eng-review` structure, rewritten as one cohesive execution plan  
Status: execution-ready planning pass on 2026-05-05  
Outside voice: not used for this document generation

## Objective

Finish the narrow slice that must be true before public `start|resume|fork|stop` surfaces are safe to expose.

This plan does three things and only three things:

1. make `substrate agent status` render degraded but intelligible output instead of aborting on strict parent-session selection failures,
2. make trace-only fallback participant-aware so same-agent sibling participants inside one orchestration session do not collapse when trace evidence can distinguish them,
3. freeze one public naming contract now so later control surfaces do not inherit the current local handle ambiguity.

This slice does not add new public control commands. It hardens the read-side and naming substrate those commands will rely on.

## Plan Summary

The repo is not missing agent runtime persistence, participant lineage, world-generation invalidation, or trace production. Those are already real:

- strict active-session selection is implemented in [`resolve_single_live_session_for_agent(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:433),
- permissive session-record construction with warning accumulation already exists in [`build_session_record(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:858),
- live status projections already preserve `participant_id` in [`live_participant_status_projection(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1140),
- producer-side trace rows already carry `participant_id`, `parent_participant_id`, and `resumed_from_participant_id` in [`AgentEvent`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs:74) and are populated by [`agent_events.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs:117).

What is still wrong is contract cohesion:

1. `build_status_report(...)` still preflights the strict control-plane selector and therefore aborts status wholesale on ambiguity or stale linkage, even though the store can already preserve warnings and partial truth.
2. trace fallback suppression and parent-run correlation are still keyed too coarsely by `(orchestration_session_id, agent_id)` or `(orchestration_session_id, agent_id, execution.scope)`, which is not enough once sibling participants for the same agent can coexist inside one orchestration session.
3. persisted and local model names still overload "session handle" to mean a participant lineage id in some places and an upstream UAA session id in others.

This slice does not invent new runtime behavior. It makes the already-landed behavior safe and legible for operators.

## Locked Starting State

### What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| Strict active control target resolution | [`resolve_single_live_session_for_agent(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:433) | Keep strict. Do not weaken it for doctor, toolbox, or future mutating controls. |
| Permissive session record construction with warning accumulation | [`build_session_record(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:858) | Reuse as the status-read foundation instead of inventing a parallel warning model. |
| Status surface rendering and trace fallback merge | [`build_status_report(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:421) | Refactor this seam. Do not redesign unrelated agent commands. |
| Live participant status rows already expose `participant_id` | [`live_participant_status_projection(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1140) | Preserve and extend with explicit provenance, not a new row model. |
| Producer-side trace rows already carry participant lineage | [`AgentEvent`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs:74), [`agent_events.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs:117) | Reuse. Do not widen producer schema unless a regression proves a missing field. |
| Authoritative parent session record with overloaded `active_session_handle_id` | [`OrchestrationSessionRecord`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:24) | Keep storage shape for now, but stop treating the field name as canonical public semantics. |
| Participant model with local `session_handle_id` mirror and legacy aliases | [`AgentRuntimeParticipantHandle`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:45), [`AgentRuntimeParticipantHandleWire`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:528) | Preserve compatibility reads. Canonicalize docs and status/report naming instead of doing a storage flag day. |
| Regression anchors for strict fail-closed operator surfaces and status fallback rules | [`agent_successor_contract_ahcsitc0.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs) | Extend this suite. Do not replace it with a fresh harness. |

### Exact remaining gap

The remaining gap is narrower than the design doc had to assume:

1. `agent status` still behaves like a control-surface selector gate because it calls the strict resolver up front in [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:446).
2. incomplete active parent linkage is already expressible as warnings in [`build_session_record(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:872), but that information is discarded before status rendering.
3. fallback suppression is still keyed by `(orchestration_session_id, agent_id, execution.scope)` in [`session_fallback_suppression_key(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1185), so one live or invalidated participant can suppress a sibling that should remain visible.
4. pure-agent trace selection still keys by `(orchestration_session_id, agent_id)` in [`pure_session_key(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1252), so trace-only sibling participants can collapse even though `participant_id` is present on the event.
5. nested parent-run correlation still groups selected parent runs by `(orchestration_session_id, agent_id)` in [`build_status_report(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:595), which is too coarse when multiple same-agent parents are plausible.
6. public/operator-facing naming still lacks one explicit rule separating `orchestration_session_id`, `participant_id`, and `internal.uaa_session_id`.

## Frozen Execution Contract

If implementation wants to do something else, revise this plan first. Do not freestyle past these rules.

### Non-negotiable invariants

1. `substrate agent status` becomes permissive in rendering, not permissive in authorization.
2. [`resolve_single_live_session_for_agent(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:433) stays fail-closed and remains the selector for doctor, toolbox, and future mutating control paths.
3. live authoritative participant/session state continues to beat trace fallback.
4. invalidated authoritative participant tombstones continue to suppress matching stale trace fallback rows.
5. participant-aware trace fallback uses `participant_id` when trace evidence supplies it. If trace evidence does not supply it, legacy coarse fallback behavior is allowed only for those legacy rows.
6. same-agent sibling participants in one orchestration session must remain distinct on the status surface when authoritative live state or trace lineage can distinguish them.
7. malformed selected tuples still fail closed. This slice does not relax world identity validation or nested tuple validation.
8. public parent-session selection uses `orchestration_session_id`.
9. `participant_id` remains a subordinate runtime/lineage identifier, not the canonical public session handle.
10. `internal.uaa_session_id` remains internal implementation detail and is never the default public selector.
11. compatibility reads for legacy `session_handle_id`, `parent_session_handle_id`, and `resumed_from_session_handle_id` remain supported.
12. new status/report surfaces must use canonical names even if storage still reads legacy aliases.

### Public naming contract

| Concept | Canonical name | Current source of truth | Rule after this slice |
| --- | --- | --- | --- |
| Parent orchestration selector | `orchestration_session_id` | [`OrchestrationSessionRecord`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:24) | This is the public control-plane session handle. |
| Runtime participant lineage id | `participant_id` | [`AgentRuntimeParticipantHandle`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:45) | This stays visible in status, lineage, nesting, and debugging only. |
| Upstream backend-native handle | `internal.uaa_session_id` | [`AgentRuntimeSessionInternal`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:75) | Internal only. Do not expose as the default operator target. |
| Legacy compatibility aliases | `session_handle_id`, `parent_session_handle_id`, `resumed_from_session_handle_id` | [`AgentRuntimeParticipantHandleWire`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:528) | Read-compatible only. Never present these as canonical public terminology in new docs or status fields. |

### Surface behavior split

| Surface | Selection behavior | Trace behavior | Required result |
| --- | --- | --- | --- |
| `substrate agent status` | enumerate and degrade | fallback allowed, but never authoritative over live/tombstone state | render valid rows, attach warnings, never authorize control |
| `substrate agent doctor` | strict single-live-session selector | trace may inform diagnostics, not authorization | fail closed on ambiguity or stale parent linkage |
| `substrate agent toolbox status|env` | strict single-live-session selector | trace must not authorize endpoint selection | fail closed on ambiguity or stale parent linkage |
| future `substrate agent start|resume|fork|stop` | strict explicit selector | trace must not authorize control | fail closed on ambiguity or stale parent linkage |

### Status report contract to freeze now

These are exact contract choices, not examples:

| Surface element | Exact contract | Why it is frozen now |
| --- | --- | --- |
| Status row provenance field | `StatusSessionJson.source_kind: "live_runtime" \| "trace_fallback"` | Operators need to see whether a row came from authoritative runtime state or from history. |
| Report warning field | `StatusReportJson.warnings: Vec<String>` | Degraded status is allowed only if the surface says what degraded. |
| Warning ordering | sorted, deduplicated, human-readable strings | Output must stay stable in tests and readable in operator screenshots. |
| Record-level warnings | copied from `AgentRuntimeSessionRecord.warnings` | Reuse the store's existing warning truth instead of duplicating it. |
| Set-level warnings | emitted by `build_status_report(...)` for cross-record ambiguity such as multiple active parent candidates | `build_session_record(...)` only sees one record at a time. |

### Participant-aware identity contract to freeze now

Use one private key family everywhere status decides whether two rows represent the same logical participant:

```text
StatusIdentityKey {
    orchestration_session_id: String,
    agent_id: String,
    execution_scope: "host" | "world",
    participant_id: Option<String>,
}
```

Rules:

1. if `participant_id` exists on the row, it participates in identity and equality
2. if `participant_id` is absent, fall back to the coarse legacy identity for that row only
3. the same key contract is used for pure-agent trace selection, live/tombstone fallback suppression, and selected-parent bucketing for nested correlation
4. this slice must not introduce a second "almost the same" tuple family

## Step 0: Scope Challenge

### 0A. What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| Session enumeration across canonical + compatibility snapshots | [`list_sessions()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:708) | Reuse. Add a status-specific filter, not a new storage scan path. |
| Active parent completeness checks and warnings | [`build_session_record(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:858) | Reuse as the degradation source of truth. |
| Status JSON rows and nested row shaping | [`StatusSessionJson`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:329), [`StatusReportJson`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:367) | Extend minimally with provenance and warnings. Do not create a second report format. |
| Trace event lineage fields | [`AgentEvent`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs:74) | Reuse. Do not change producer schema unless a regression proves a missing correlation field. |
| Legacy handle alias support | [`legacy_handle_upgrade`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:747) | Preserve. Canonicalize naming around it instead of deleting it. |
| Existing strict operator-surface failure tests | [`operator_surfaces_fail_closed_when_active_parent_omits_active_session_handle_id`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs:1514) and nearby tests | Keep green. This slice must not weaken them. |

### 0B. Minimum honest diff

The minimum honest implementation is:

1. add `list_status_sessions_for_agent(&self, orchestrator_agent_id: &str) -> Result<Vec<AgentRuntimeSessionRecord>>` in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) so status can enumerate incomplete-but-readable records without authorizing control,
2. rework [`build_status_report(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:421) to consume that permissive path instead of strict preflight and to emit sorted, deduplicated report warnings,
3. add one participant-aware `StatusIdentityKey` family in [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) and use it consistently for pure-agent trace selection, fallback suppression, and nested parent correlation,
4. extend status JSON/text output with the exact fields `source_kind` and `warnings`,
5. update tests and repo-truth docs so the public naming contract and degraded read-side contract are pinned.

Anything broader is scope creep. Anything smaller leaves the core ambiguity un-fixed.

### 0C. Complexity check

This plan stays under the smell threshold if executed honestly.

Expected production files:

1. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
2. [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
3. [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs) for comment/name clarification only if needed
4. [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) for naming comments and alias tests only

Expected tests/docs:

1. [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
2. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
3. [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md) if this index is used as current planning truth

Rejected expansions:

1. changing trace producer schema in [`crates/common/src/agent_events.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs) unless a concrete missing-field regression appears
2. changing world-member launch, follow-up submit, or replacement semantics
3. adding the public `substrate agent start|resume|fork|stop` family
4. redesigning toolbox as a mutation surface
5. changing on-disk layout or dropping compatibility dual-read behavior
6. broad docs rewrites outside the gap matrix and active planning index

### 0D. Search and completeness check

Search-before-building result, in practical terms:

- **[Layer 1]** reuse permissive warning construction in [`build_session_record(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:858)
- **[Layer 1]** reuse strict control-plane selection in [`resolve_single_live_session_for_agent(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:433)
- **[Layer 1]** reuse existing trace participant lineage fields instead of inventing new ones
- **[Layer 1]** reuse the existing status JSON surface and extend it minimally
- **[EUREKA]** the problem is not persistence or event emission anymore. The problem is that the read-side still throws away the permissive information the store already knows and then falls back with a key that is too coarse.
- **[EUREKA]** the cheapest complete version is not a new session model. It is one explicit split between strict control selection and permissive status rendering, plus participant-aware keys and regression proof.

### 0E. Distribution check

No new artifact type is introduced. No CI/CD or packaging work is required.

The real distribution requirement here is operator truth:

1. the CLI must say which rows are live and which are trace fallback,
2. it must not pretend a degraded surface is healthy when warnings exist,
3. it must not ask operators to learn the wrong handle names before public controls ship.

### 0F. NOT in scope

- adding `substrate agent start|resume|fork|stop`, because this slice is the read-side prerequisite for those controls, not the controls themselves
- redesigning `substrate -c`, because that is a caller-surface product decision unrelated to status correctness
- changing world-member follow-up submit or replacement behavior, because PLAN-17 already froze that runtime contract
- weakening doctor/toolbox authorization rules, because those surfaces must stay fail-closed even if status becomes permissive
- removing compatibility alias reads, because storage migration is not required to fix operator-facing naming
- changing trace producer schema unless a concrete missing-field regression appears, because the existing events already carry the lineage fields this slice needs
- replacing `active_session_handle_id` in storage with a new persisted field name, because that is a flag-day migration with no bearing on the immediate read-side bug
- macOS/Lima parity work, because this plan is about Linux-first status and naming cohesion, not cross-platform rollout

## Architecture Review

### Locked architecture decisions

1. Add one status-specific enumeration seam named `list_status_sessions_for_agent(...)` in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs).
2. Keep [`resolve_single_live_session_for_agent(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:433) untouched for strict control-plane selection.
3. Extend the existing status report shape instead of creating a second status schema.
4. Use one participant-aware `StatusIdentityKey` family consistently for pure-agent trace projection, live/tombstone fallback suppression, and nested parent correlation.
5. Keep compatibility alias support at the serde boundary only.
6. Update docs and comments so public/operator-facing language says `orchestration_session_id` and `participant_id`, not `session_handle_id`.

### Architecture findings resolved in-plan

**Issue 1. Status is still paying the control-plane tax.**

That is the main bug. A read-only surface should not die just because the authoritative parent is incomplete if it can still render truthful rows with warnings.

**Issue 2. The store already knows how to degrade, but status ignores it.**

This is the classic "we built the right primitive and then walked around it." [`build_session_record(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:858) already models incomplete active sessions and warnings. Status should consume that instead of re-imposing strictness.

**Issue 3. Participant-aware trace fields exist, but the fallback reader still groups like the old world.**

That is how sibling participants disappear. The trace producer is already richer than the trace consumer.

**Issue 4. Naming drift will poison the next public surface if not frozen now.**

If this slice leaves `session_handle_id` ambiguous, the later public control family will inherit a bad contract by default. That is the kind of bug that becomes documentation debt, then UX debt, then API debt.

### Architecture ASCII diagrams

### Status vs control selection split

```text
STRICT CONTROL SURFACES
=======================
doctor / toolbox / future start|resume|fork|stop
    |
    v
resolve_single_live_session_for_agent(...)
    |
    +--> one authoritative active parent + one live host orchestrator --> authorize
    |
    +--> ambiguity / stale linkage / missing active handle -----------> fail closed

PERMISSIVE STATUS SURFACE
=========================
agent status
    |
    v
list status candidate sessions
    |
    +--> complete authoritative records ------------------------------> project live rows
    |
    +--> incomplete authoritative records with live participants ----> project valid rows + warnings
    |
    +--> no live authoritative row ----------------------------------> allow trace fallback
    |
    \--> malformed selected tuple -----------------------------------> fail closed only for that malformed selected record class
```

### Status row precedence and fallback suppression

```text
STATUS ROW PRECEDENCE
=====================
authoritative live participant row
    beats
trace fallback row with same participant-aware identity

invalidated authoritative tombstone
    suppresses
trace fallback row with same participant-aware identity

trace fallback row
    only fills
gaps where no live row and no invalidated tombstone exists

identity key
============
(orchestration_session_id, agent_id, execution.scope, participant_id?)

rules:
- if participant_id exists, use it
- if participant_id is absent, fall back to legacy coarse identity for that row only
- never let one participant suppress an unrelated sibling when participant_id differs
```

### Nested parent correlation with participant awareness

```text
selected pure-agent parent rows
    |
    v
index by:
(orchestration_session_id, agent_id, participant_id?)
    |
    +--> if nested row has parent_participant_id
    |       require match inside that participant bucket
    |
    +--> else
    |       use legacy session+agent bucket
    |
    \--> parent_run_id must still match either:
            - the selected winning parent run in that bucket, or
            - a known historical pure-agent run in that bucket
```

### Public naming split that must remain unchanged

```text
public/operator-facing parent selector
    orchestration_session_id

runtime lineage / nesting / debugging
    participant_id

backend-native upstream runtime handle
    internal.uaa_session_id

legacy alias names
    serde-read compatibility only
```

## Code Quality Review

### Findings resolved in-plan

**Issue 1. The read-side contract should live in one obvious place.**

`build_status_report(...)` currently mixes projection, fallback merge, validation, and strict preflight assumptions. This slice should leave one readable story instead of another fossil dig.

**Issue 2. Minimal diff matters here.**

The repo already has the right data. This is not the moment for a new status module, new storage format, or a generic "identity resolver" framework.

**Issue 3. Storage comments and serde aliases need to stop teaching the wrong vocabulary.**

We do not need a schema migration. We do need comments, tests, and report fields that say the right thing.

**Issue 4. Warning handling must be deliberate, not incidental.**

If degraded status output exists, operators need to see why it degraded. Silent partial truth is just a different kind of lie.

### Allowed code shape

1. Prefer one small status-read helper named `list_status_sessions_for_agent(...)` in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) over a new state-store abstraction layer.
2. Prefer one small private `StatusIdentityKey` type or helper family in [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) over ad hoc tuple rewrites in three separate spots.
3. Extend [`StatusSessionJson`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:329) with the exact field `source_kind` and [`StatusReportJson`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:367) with the exact field `warnings`. Do not create a shadow JSON shape.
4. Keep legacy alias support in [`session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:528). Do not rename persisted fields in this slice.
5. Use comments to freeze the naming contract near [`OrchestrationSessionRecord`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:24) and [`AgentRuntimeParticipantHandle`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:45), because those are where future control-surface authors will look first.

## Test Review

### Test framework detection

This repo is Rust-first and the relevant review surface is `cargo test`.

Primary suites for this slice:

1. [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
2. unit tests in [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
3. unit tests in [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/shell/src/execution/agent_runtime/state_store.rs
    |
    ├── resolve_single_live_session_for_agent(...)
    │   ├── [★★★ TESTED] multiple active parent sessions fail closed
    │   ├── [★★★ TESTED] missing active parent fails closed
    │   ├── [★★★ TESTED] missing active_session_handle_id fails closed
    │   └── [★★★ TESTED] inactive/mismatched orchestrator participant fails closed
    |
    └── new status-specific session enumeration helper
        ├── [GAP]         incomplete active parent with live readable participants degrades to warnings, not error
        ├── [GAP]         multiple active parent candidates remain visible to status as warnings, not abort
        └── [GAP]         missing active_session_handle_id remains visible to status as warning, not abort

[+] crates/shell/src/execution/agents_cmd.rs
    |
    ├── build_status_report(...)
    │   ├── [★★★ TESTED] live manifest beats trace fallback for selected orchestrator
    │   ├── [★★★ TESTED] invalidated tombstone beats stale trace fallback
    │   ├── [★★★ TESTED] same-agent concurrent live rows from authoritative session records remain visible
    │   ├── [GAP]         status no longer hard-preflights strict live-session selection
    │   ├── [GAP]         status emits warnings when parent linkage is incomplete
    │   ├── [GAP]         status marks row provenance as live_runtime vs trace_fallback
    │   └── [GAP]         malformed selected tuples still fail closed after the refactor
    |
    ├── participant-aware pure-agent trace projection
    │   ├── [GAP]         trace-only sibling participants with distinct participant_id stay distinct
    │   └── [GAP]         legacy trace rows without participant_id still collapse only within their own coarse bucket
    |
    ├── fallback suppression
    │   ├── [★★★ TESTED] tombstone suppression works for coarse tuple today
    │   ├── [GAP]         live/tombstone suppression only suppresses the matching participant-aware row
    │   └── [GAP]         sibling participant fallback rows survive when only one sibling has a live or tombstoned match
    |
    └── nested parent correlation
        ├── [★★★ TESTED] malformed nested tuples fail closed on selected rows
        ├── [★★★ TESTED] malformed parent correlation fails closed today
        └── [GAP]         parent_participant_id-aware nested correlation chooses the right sibling parent bucket

[+] crates/shell/src/execution/agent_runtime/session.rs
    |
    ├── legacy alias deserialization
    │   └── [★★★ TESTED] legacy session_handle_id aliases deserialize into participant fields
    |
    └── canonical naming/documentation expectations
        └── [GAP]         tests/comments pin that canonical reporting uses participant/session terminology, not legacy alias names

---------------------------------
COVERAGE TARGET
- strict control-surface fail-closed tests stay green
- degraded status rendering paths gain direct regression proof
- participant-aware fallback and nested correlation get sibling-specific proof
- naming compatibility stays green while canonical terminology moves forward
---------------------------------
```

### Operator flow coverage

```text
OPERATOR FLOW COVERAGE
===========================
[+] Operator runs `substrate agent status --json` during healthy live runtime
    |
    ├── [★★★ TESTED] live authoritative rows win over trace fallback
    └── [GAP]         each row exposes provenance = live_runtime

[+] Operator runs `substrate agent status --json` with incomplete parent linkage
    |
    ├── [GAP] [->E2E] missing active_session_handle_id produces readable rows + warning
    ├── [GAP] [->E2E] multiple active parent candidates produce warnings instead of whole-surface abort
    └── [GAP] [->E2E] trace fallback fills only missing rows and is clearly labeled

[+] Operator runs `substrate agent status --scope world --json` after same-agent sibling activity
    |
    ├── [★★★ TESTED] authoritative same-agent concurrent sessions stay visible
    ├── [GAP] [->E2E] trace-only sibling participants stay visible when participant_id differs
    └── [GAP] [->E2E] tombstone/live suppression only suppresses the matching sibling

[+] Operator runs `substrate agent doctor --json` or `toolbox env --json` under ambiguous parent linkage
    |
    └── [★★★ TESTED] fail closed, no status-style degradation leaks into authorization paths
```

### Required tests to add or extend

1. Add status regressions proving that `substrate agent status --json` no longer aborts when the active parent omits `active_session_handle_id`, and instead returns successful output with warnings plus whatever authoritative live rows remain readable.
2. Add status regressions proving that multiple active parent-session candidates for the same orchestrator agent are warnings on `agent status`, not fatal selection errors, while doctor/toolbox remain fail closed.
3. Add a trace-only sibling-participant regression showing that two status events with the same `(orchestration_session_id, agent_id)` but distinct `participant_id` values remain distinct on the status surface.
4. Add a sibling-specific suppression regression proving that a live or invalidated participant suppresses only the matching trace fallback row, not every sibling row sharing the same agent id and orchestration session.
5. Add a nested parent-correlation regression proving that `parent_participant_id` narrows the selected-parent bucket when present and preserves the current fail-closed behavior when the nested tuple is malformed.
6. Preserve existing strict fail-closed tests for `agent doctor`, `toolbox env`, and the strict selector unit tests. Those are not optional.
7. Keep the existing legacy alias deserialization test green and add one small naming assertion if status/report output gains canonical naming metadata.

### QA-facing test artifact

During implementation, write a QA-facing artifact to:

```text
~/.gstack/projects/<slug>/<user>-feat-session-centric-state-store-eng-review-test-plan-<timestamp>.md
```

Required contents:

1. healthy `agent status --json` with live authoritative rows
2. degraded `agent status --json` with missing `active_session_handle_id`
3. degraded `agent status --json` with multiple active parent candidates
4. same-agent sibling participant visibility under trace-only fallback
5. doctor/toolbox fail-closed behavior under the same degraded parent states

This artifact is for `/qa` and `/qa-only`. Keep it operator-journey oriented.

### Regression rule for this slice

These tests are mandatory. No discussion:

1. doctor/toolbox strict fail-closed behavior stays green
2. live authoritative rows still beat trace fallback
3. tombstones still beat stale trace fallback
4. same-agent concurrent live authoritative rows stay visible
5. legacy `session_handle_id` aliases still deserialize
6. degraded status paths are directly tested, not inferred from helper behavior

## Failure Modes Registry

| Failure mode | Test required | Error handling exists | Operator sees clear result | Critical gap before this slice lands |
| --- | --- | --- | --- | --- |
| status still aborts when active parent omits `active_session_handle_id` | yes | partial today, but only as hard error | no | yes |
| status still aborts on multiple active parent candidates | yes | partial today, but only as hard error | no | yes |
| trace-only sibling participants collapse into one row | yes | no | no | yes |
| one sibling tombstone suppresses a different sibling's fallback row | yes | no | no | yes |
| nested row attaches to the wrong sibling parent when `parent_participant_id` is present | yes | no | no | yes |
| degraded status output omits why it degraded | yes | no | no | yes |
| doctor/toolbox accidentally inherit permissive status behavior | yes | yes today via strict selector | yes | yes |
| malformed selected tuple becomes silently tolerated during refactor | yes | yes today | yes | yes |
| canonical naming changes break legacy snapshot reads | yes | yes today via serde aliases | yes | no |

Critical gap rule for this plan:

No degraded status path is allowed to be both partially truthful and silent. If the surface degrades, it must say why.

## Performance Review

Performance is not the main risk here, but the read-side can still get sloppier than it needs to.

### Findings resolved in-plan

1. Do not add extra filesystem passes when the store already materializes session records in one place. Status should consume a status-specific list helper, not re-scan storage three different ways.
2. Use one small participant-aware key type and `BTreeMap`/`BTreeSet` lookups. Do not introduce nested O(n²) sibling matching across the whole report.
3. Keep trace ingestion as the current single pass over `trace.jsonl`. The change is key shape and warning merge, not a second trace-processing pipeline.
4. Status remains human-paced operator traffic. Correctness and debuggability matter more than shaving a few map allocations.

### Performance posture

- no new N+1 data-access pattern is acceptable
- no background cache or index is needed
- no new persistence file is needed
- participant-aware keying must stay in-memory and linearithmic at worst
- live authoritative precedence must remain the first choice, not a fallback after extra work

## DX Guardrails

This is a developer/operator surface even though it is backend code.

Required operator experience:

1. `agent status` must still succeed in degraded states where doctor/toolbox would fail closed.
2. the output must say which rows are `live_runtime` and which are `trace_fallback`.
3. warning text must name the broken parent/session condition in plain terms, not bury it in an internal stack trace.
4. new docs and report shapes must say `orchestration_session_id` and `participant_id`.
5. no surface in this slice should ask an operator to type or target `uaa_session_id`.

## Worktree Parallelization Strategy

This plan has limited but real parallelization opportunities once the naming and degradation contract is frozen.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Freeze degraded-status contract, naming contract, and participant-aware identity rules | `crates/shell/src/execution/`, `crates/shell/src/execution/agent_runtime/`, repo docs | — |
| Status-specific session enumeration and naming/comment cleanup | `crates/shell/src/execution/agent_runtime/` | Freeze degraded-status contract, naming contract, and participant-aware identity rules |
| Status rendering refactor, participant-aware fallback, and nested correlation tightening | `crates/shell/src/execution/` | Freeze degraded-status contract, naming contract, and participant-aware identity rules |
| Regression tests and repo-truth closeout | `crates/shell/tests/`, repo docs | Status-specific session enumeration and naming/comment cleanup, Status rendering refactor and participant-aware fallback |

### Parallel lanes

- Lane A: Status-specific session enumeration and naming/comment cleanup
  - sequential inside the lane because these steps share `crates/shell/src/execution/agent_runtime/`
- Lane B: Status rendering refactor, participant-aware fallback, and nested correlation tightening
  - sequential inside the lane because these steps share `crates/shell/src/execution/agents_cmd.rs`
- Lane C: Regression tests and repo-truth closeout
  - starts after A and B because the tests and docs depend on the frozen helper behavior and key contract

### Execution order

1. Freeze the degraded-status contract and participant-aware identity rules.
2. Launch Lane A and Lane B in parallel worktrees.
3. Merge A and B.
4. Run Lane C for degraded-status regressions, sibling-participant regressions, strict-surface non-regressions, and docs closeout.

### Conflict flags

- Lane A and Lane B are intentionally split at the module boundary, but they still share the naming contract. Freeze the rule set first or you will get two "obvious" implementations that disagree.
- Lane B owns [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs). Keep that ownership exclusive to avoid merge churn in the status refactor.
- Lane C owns [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs). Do not let implementation lanes opportunistically add tests there mid-flight.
- Docs move last. Updating the gap matrix before the regressions land is how truth drifts again.

### Parallelization verdict

Three workstreams, two parallel implementation lanes, one final integration lane.

## Implementation Sequence

### Step 1. Freeze the degraded-status and naming contract

Files:

1. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
2. [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
3. [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
4. [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)

Deliver:

1. decide and document that `agent status` is permissive and doctor/toolbox remain strict
2. freeze the exact JSON/report additions:
   - `StatusSessionJson.source_kind`
   - `StatusReportJson.warnings`
   - warning strings sorted and deduplicated before render
3. freeze the participant-aware key contract:
   - `(orchestration_session_id, agent_id, execution.scope, participant_id?)`
   - participant-specific when present
   - legacy coarse fallback only when participant identity is absent
4. freeze the naming contract:
   - `orchestration_session_id` public selector
   - `participant_id` runtime lineage id
   - `internal.uaa_session_id` internal only
5. freeze warning ownership:
   - record-local degradation comes from `AgentRuntimeSessionRecord.warnings`
   - cross-record ambiguity warnings are emitted by `build_status_report(...)`

Done means the contract is explicit before helper and projection work begins.

### Step 2. Add one status-specific session enumeration seam

Files:

1. [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Deliver:

1. add `list_status_sessions_for_agent(...)` that enumerates status-visible session records for the selected orchestrator agent without calling the strict single-live-session selector
2. preserve `warnings`, `complete`, and `has_authoritative_parent` from [`build_session_record(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:858)
3. include every record from `list_sessions()` whose authoritative parent targets the selected orchestrator agent, plus parentless live-host-orchestrator cases for that agent that status must render as degraded warnings
4. keep owner-pid liveness and authoritative session/participant validation rules where they already belong
5. do not add a second record-construction path; this helper must still route through `build_session_record(...)`
6. do not weaken [`resolve_single_live_session_for_agent(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:433)

Done means status can enumerate incomplete-but-readable active records without authorizing control.

### Step 3. Rewire status rendering onto the permissive seam

Files:

1. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Deliver:

1. remove the unconditional strict preflight from [`build_status_report(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:446)
2. consume `list_status_sessions_for_agent(...)`
3. extend [`StatusSessionJson`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:329) with the exact field `source_kind`
4. extend [`StatusReportJson`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:367) with `warnings: Vec<String>`
5. populate `warnings` as the sorted, deduplicated union of:
   - selected record warnings
   - cross-record ambiguity warnings such as multiple active parent candidates
6. render warnings in text mode before the `sessions` block and include them in JSON mode unchanged

Done means degraded status succeeds, is explicit about degradation, and stays schema-minimal.

### Step 4. Make pure-agent fallback and suppression participant-aware

Files:

1. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Deliver:

1. replace the current coarse pure-agent trace key in [`pure_session_key(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1252) with `StatusIdentityKey`
2. replace the current suppression keys in [`session_fallback_suppression_key(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1185) and [`participant_fallback_suppression_key(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1193) with the same `StatusIdentityKey`
3. preserve legacy coarse grouping only for rows that truly lack participant identity
4. keep live authoritative rows and invalidated tombstones winning over trace fallback

Done means sibling participants no longer disappear just because they share `agent_id`.

### Step 5. Make nested parent correlation participant-aware without weakening validation

Files:

1. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Deliver:

1. key selected parent runs by `(orchestration_session_id, agent_id, participant_id?)`
2. when a nested row has `parent_participant_id`, require correlation within that participant bucket before checking `parent_run_id`
3. when a nested row lacks `parent_participant_id`, preserve current legacy session+agent fallback behavior
4. keep malformed tuple and malformed parent-run validation fail closed

Done means the nested gateway surface attaches to the right parent sibling when lineage exists and still refuses malformed rows.

### Step 6. Freeze the contract with regression tests

Files:

1. [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
2. [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) unit tests
3. [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) unit tests

Deliver:

1. add degraded-status success-path tests for missing active handle and multiple active parents
2. add participant-aware trace-only sibling visibility tests
3. add participant-aware suppression specificity tests
4. add parent-participant-aware nested correlation tests
5. preserve strict doctor/toolbox fail-closed tests
6. preserve legacy alias deserialization tests

Done means the read-side and naming contract are proven, not implied.

### Step 7. Update repo-truth docs

Files:

1. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
2. [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md) if this plan index is used

Deliver:

1. mark status ambiguity handling as landed once regressions are green
2. narrow the session-handle gap language to remaining public control-surface productization instead of current local ambiguity
3. describe the canonical naming contract plainly
4. describe trace-only fallback as participant-aware once the tests prove it

Done means the repo says what the runtime actually does now.

## Recommended Verification Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
```

Manual spot checks after tests are green:

```bash
substrate agent status --json
substrate agent doctor --json
substrate agent toolbox env --json
```

## Definition of Done

1. `agent status` no longer aborts on strict parent-selection ambiguity or stale linkage that can be rendered truthfully with warnings.
2. doctor and toolbox surfaces remain strict and fail closed.
3. status rows explicitly identify whether they come from `live_runtime` or `trace_fallback`.
4. same-agent sibling participants remain distinct on the status surface when lineage evidence exists.
5. live/tombstone suppression only suppresses matching participant-aware fallback rows.
6. nested parent correlation respects `parent_participant_id` when present and still rejects malformed tuples.
7. canonical naming in docs and status/report surfaces uses `orchestration_session_id` and `participant_id`.
8. legacy alias reads remain green.
9. no new public `start|resume|fork|stop` surface is introduced.
10. repo-truth docs reflect the landed behavior.

## Deferred Work

- public `substrate agent start|resume|fork|stop`
- explicit non-REPL caller surfaces
- storage-level rename away from `active_session_handle_id`
- broader toolbox mutation surface
- any producer-side trace schema expansion beyond currently shipped lineage fields
- macOS/Lima parity and broader operator-surface productization

## Completion Summary

- Step 0: Scope Challenge, scope accepted as-is
- Architecture Review: 4 issues found, all resolved in-plan
- Code Quality Review: 4 issues found, all resolved in-plan
- Test Review: diagram produced, 7 concrete regression gaps identified
- Performance Review: 4 issues found, all resolved in-plan
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 items proposed, no `TODOS.md` exists in the repo today
- Failure modes: 6 critical gaps flagged until the new regression floor lands
- Outside voice: skipped for this document generation
- Parallelization: 3 lanes, 2 parallel / 1 sequential integration lane
- Lake Score: 9/9 recommendations chose the complete option

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Step 0 | Treat this as a read-side contract split, not a persistence or producer-schema slice | Mechanical | Pragmatic | The hard parts already landed; the bug is how the status surface consumes them | reopening storage or trace producer design |
| 2 | Architecture | Keep strict control selection in `resolve_single_live_session_for_agent(...)` | Mechanical | Explicit over clever | One helper should stay authoritative for control-plane safety | weakening the selector to make status easier |
| 3 | Architecture | Add a status-specific enumeration helper instead of teaching status to fake permissive selection itself | Mechanical | DRY | The store already knows how to build incomplete session records with warnings | duplicating permissive warning logic in `agents_cmd.rs` |
| 4 | Architecture | Extend the existing status report shape with provenance and warnings | Mechanical | Minimal diff | Operators need more truth, not a second schema | brand-new status JSON model |
| 5 | Architecture | Use one participant-aware identity key family everywhere fallback identity matters | Mechanical | Systems over heroes | One contract across selection, suppression, and nesting avoids sibling drift | three unrelated tuple rewrites |
| 6 | Code Quality | Keep compatibility aliases at serde-read boundaries only | Mechanical | Pragmatic | This avoids a storage flag day while still fixing public naming | renaming persisted fields in this slice |
| 7 | Test Review | Make degraded-status success cases mandatory regression tests | Mechanical | Completeness | The whole point of the slice is that status no longer dies on readable ambiguity | relying on unit helpers without CLI-surface proof |
| 8 | Test Review | Make sibling-specific suppression proof mandatory | Mechanical | Completeness | This is the main correctness edge once participant-aware fallback lands | assuming the key refactor is obviously right |
| 9 | Parallelization | Freeze naming and degradation rules first, then run state-store and status-surface lanes in parallel | Mechanical | Pragmatic | The module split is clean, but the contract is shared and easy to drift on | uncoordinated parallel edits before the rules are frozen |
