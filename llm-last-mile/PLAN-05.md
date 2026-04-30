# PLAN-05: Restart Invalidation Semantics for Live State

Source file: [05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md)  
Branch: `feat/restart-invalidation-semantics`  
Plan type: backend-only, no UI scope  
Review posture: `/autoplan`-style consolidation pass with `/plan-eng-review` rigor plus one Claude outside-voice challenge  
Status: execution-ready after repo-state corrections

## What This Plan Does

`PLAN-03` made shared-world ownership and generation backend-authoritative. `PLAN-04` made the
parent orchestration-session record the shell-owned bridge for the active `world_id` and
`world_generation`.

This plan does the next narrow job only:

1. make a shared-world generation change invalidate every older world-scoped **member**
   participant for the same `orchestration_session_id`,
2. make invalidated registry records suppress trace fallback for stale member rows,
3. make restart sequencing fail closed if replacement members are not ready,
4. preserve trace as historical audit instead of letting it re-authorize stale live state.

The repo is already ahead of the source SOW in a few places, so `PLAN-05` starts by correcting
those facts:

- authoritative runtime participant storage is now `participants/*.json`, not `handles/*.json`,
- `toolbox status --json` already has `active_world_binding` from `PLAN-04`,
- `session_handle_id` / `resumed_from_session_handle_id` are runtime aliases, while the persisted
  on-disk lineage field is `resumed_from_participant_id`,
- `list_live_manifests()` is already the right low-level live read and does not need to be
  replaced for this slice.

That means the smallest correct slice is not "new registry layout" and not "new status surface."
It is:

- one targeted invalidation sweep in the runtime state store,
- one generation-aware trace-suppression fix in `agent status`,
- one restart commit-order rule that prefers fail-closed absence over stale liveness.

## Scope Challenge

### 0A. Premise Challenge

These are the premises this plan accepts after repository review and outside-voice challenge.

1. **`PLAN-04` parent-session binding is the active-generation authority for `PLAN-05`.**
   - Accepted.
   - `OrchestrationSessionRecord.world_id/world_generation` already exists in
     [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:21)
     and is persisted through
     [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:280).

2. **Trace must never be allowed to re-promote a stale world-scoped member after invalidation.**
   - Accepted.
   - This is the whole point of the slice. If trace can still resurrect generation `G`, the
     generation contract is fake.

3. **This slice should not introduce a new session index file or grouped registry layout.**
   - Accepted.
   - `PLAN-06` already owns the session-centric regrouping. Doing it here spends effort on the
     wrong seam.

4. **Replacement lineage must be described with persisted participant ids, not skipped runtime aliases.**
   - Accepted.
   - On disk the field is `resumed_from_participant_id`, even though runtime aliases expose
     `resumed_from_session_handle_id`.

5. **Fail-closed absence is better than stale presence.**
   - Accepted.
   - If replacement startup fails, the correct user experience is "member is unavailable" rather
     than "member still looks live on the old world."

### 0B. Existing Code Leverage

| Sub-problem | Existing code | Reuse or replace |
| --- | --- | --- |
| World-scoped member state already has `Invalidated` and replacement lineage | [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:15) | Reuse, add one explicit invalidation helper |
| Parent session already stores the active generation | [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:34) | Reuse exactly |
| Parent binding persistence already exists | [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:280) | Reuse exactly |
| Live participant filtering already excludes `Invalidated` | [session.rs `is_authoritative_live()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:493) and [state_store.rs `list_live_participants()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:138) | Reuse, do not replace |
| Status already overlays live manifests over trace | [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:417) | Reuse, but fix suppression key and add tombstone suppression |
| Current suppression key is too coarse | [session_fallback_suppression_key(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1181) | Replace |
| Restart alert ordering already waits for persisted parent binding | [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs:1107) | Reuse as precedent, extend for member invalidation |
| Toolbox already proves live binding from the authoritative parent session | [toolbox_active_world_binding(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1061) | Reuse, do not add a new proof surface |
| Invalidated manifests already fail closed for toolbox env | [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs:1259) | Reuse as contract anchor |

### 0C. Dream State Mapping

```text
CURRENT
-------
PLAN-04 persists the active world binding on the parent session
        │
        ├── live participant reads exclude Invalidated
        ├── restart alerts publish the authoritative generation
        └── agent status still suppresses trace by (agent_id, role) only

THIS PLAN
---------
authoritative parent world_generation advances
        │
        ├── state_store sweeps older world-scoped members in that session
        ├── older members become Invalidated tombstones on disk
        ├── agent status suppresses stale trace rows by session-aware identity
        └── replacement members appear only after the old generation is already dead

12-MONTH IDEAL
--------------
session-centric registry from PLAN-06
        │
        ├── one session group contains parent + all member lineage
        ├── generation invalidation is queryable without sweeps
        ├── status, toolbox, and future control-plane APIs read the same grouped record
        └── trace stays historical only
```

### 0C-bis. Implementation Alternatives

| Approach | What it does | Effort | Risk | Recommendation |
| --- | --- | ---: | ---: | --- |
| A. Keep current trace/live overlay and rely on newer timestamps | Leaves stale generations suppressible only by luck | S | Critical | Reject |
| B. Add a new active-generation index file now | Cleaner long-term query path, bigger slice | M | Medium | Defer to `PLAN-06` unless sweeps prove too slow |
| C. Add one manifest sweep, one tombstone suppression pass, and one restart-order rule | Smallest correct fix on current architecture | M | Low | Recommended |
| D. Collapse `PLAN-05` and `PLAN-06` into one session-registry rewrite | Better end-state, wrong slice boundary | L | High | Reject for this packet |

Recommendation: **C**.

What is interesting here is that the repo already has the live-state and parent-binding machinery.
The missing seam is not storage capability. It is the policy that says stale generations lose,
immediately, even when history still exists.

### 0D. Mode-Specific Analysis

Mode: `SELECTIVE EXPANSION`

Scope held:

- no new grouped session registry
- no new toolbox/status surface beyond existing fields
- no trace schema rewrite
- no new daemon or background registry service
- no cross-platform world-ownership redesign

Expansion accepted inside the blast radius:

- one participant invalidation helper in the runtime state store
- one explicit invalidation helper on participant records to avoid handwritten state mutations
- one new status-suppression pass that reads invalidated tombstones from disk
- one set of tests that simulate cross-session same-agent collisions and restart crash windows

### 0E. Temporal Interrogation

**Hour 1:** world generation advances from `7` to `8`; every world-scoped member from generation
`7` disappears from authoritative live reads before replacement work is considered live.

**Hour 6:** replacement startup fails halfway through; `substrate agent status --json` shows the
member absent, not stale, and trace still contains the historical `world_restarted` /
`world_restart_required` story.

**Month 3:** `PLAN-06` groups the same parent + member files under session directories without
changing the invalidation rule.

**Year 1:** restart invalidation is boring infrastructure. No one is debugging "why did the dead
generation come back from trace?" at 2am.

### 0F. Mode Selection

`SELECTIVE EXPANSION` is the right mode because the SOW needs repo-state correction, not a rewrite:

- authoritative storage already exists,
- authoritative parent generation already exists,
- restart alerts already have the right top-level generation fields,
- the missing contract is the sweep + suppression rule, not a new architecture.

## What Already Exists

- `AgentRuntimeSessionState::Invalidated` already exists for participant manifests.
- `OrchestrationSessionState::Invalidated` already exists for parent sessions.
- `OrchestrationSessionRecord` already persists `world_id/world_generation`.
- `toolbox status --json` already publishes optional `active_world_binding`.
- restart alert tests already prove parent-binding persistence ordering.
- toolbox env already fails closed when the manifest is invalidated.
- world-scoped member status rows already publish top-level `world_id/world_generation`.

## Architecture Contract

### No-ambiguity rules

1. `participants/*.json` plus `sessions/*.json` are the authoritative runtime store. `handles/*.json` is legacy compatibility input only.
2. Only `role=member` plus `execution.scope=world` participants are invalidated by shared-world generation rollover. Host orchestrators are never invalidated by this rule.
3. The active generation comes from the parent `OrchestrationSessionRecord.world_generation` already persisted by `PLAN-04`. `PLAN-05` consumes that value. It does not assign it.
4. A participant record in `state=invalidated` is a **tombstone** for live surfaces. It stays on disk for audit, but it must suppress trace fallback.
5. Status-suppression identity becomes `(orchestration_session_id, agent_id, execution.scope)`. `(agent_id, role)` is too weak and breaks concurrent sessions that use the same member agent.
6. Replacement lineage is persisted with `resumed_from_participant_id`. Runtime alias fields such as `resumed_from_session_handle_id` may still exist in memory, but the plan and tests must name the persisted field.
7. Restart sequencing prefers fail-closed safety over brief availability. Old generation invalidates before the new generation is considered live.
8. If replacement members are not ready, `substrate agent status --json` omits them from `sessions[]`. It never keeps the stale generation visible "for continuity."
9. Trace remains historical audit. Trace rows may continue to exist forever. They may not regain live authority once a tombstone exists on disk.
10. `PLAN-06` still owns registry regrouping and migration away from the flat participant tree. `PLAN-05` must not spend an innovation token on that rewrite.

### Target state model

Keep the current store layout and add the smallest possible contract helpers.

Add one participant helper in
[crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs):

```text
invalidate_for_generation_rollover(reason, bucket)
    1. transition_state(Invalidated)
    2. mark_terminal_state(reason)
    3. set last_error_bucket / last_error_message
    4. clear ownership_valid through the existing terminal-state path
```

Add one store sweep in
[crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs):

```text
invalidate_prior_world_members(
    orchestration_session_id,
    active_generation,
    reason
) -> Vec<participant_id>
```

Rules:

- sweep all participants, not just live manifests, because tombstones must remain readable,
- only mutate `member + world + generation < active_generation + state.is_live()`,
- skip host-scoped participants and already-invalidated rows,
- persist each mutated participant back to `participants/*.json`,
- return invalidated participant ids for restart telemetry / debugging.

Do **not** add a session-level active-generation index file in this slice.

### Restart ordering contract

The dangerous bug is "replacement written, crash happens, old generation still looks live."
So the ordering must be explicitly fail-closed:

```text
backend accepts generation G+1 binding
        │
        ├── PLAN-04 persists parent session binding world_generation=G+1
        ├── PLAN-05 invalidates generation<G+1 world members for that session
        ├── replacement member manifests are persisted
        └── success alert / status readers observe the new state
```

That is intentionally stricter than the original SOW wording.

If the process crashes:

- **after invalidation, before replacement persistence**: member is absent, which is safe.
- **after replacement persistence, before invalidation**: two generations can look live, which is not safe.

This is the whole game.

### Status and suppression contract

`substrate agent status --json` stays a live-session surface.

Selection rules:

1. Build live projections from authoritative live manifests exactly as today.
2. Build a second suppression set from **invalidated world-scoped participant tombstones**, using
   `(orchestration_session_id, agent_id, execution.scope)`.
3. When considering trace fallback rows, suppress them if:
   - a live manifest already owns the same suppression identity, or
   - an invalidated tombstone already owns the same suppression identity.
4. World-identity validation still applies only to world-scoped selected rows. Host rows do not inherit world validation because a trace row happened to carry world fields.

Practical consequence:

- generation `7` trace rows can remain in `trace.jsonl`,
- generation `7` tombstones remain on disk,
- generation `7` never comes back into `sessions[]` after generation `8` commits.

### Runtime-mode matrix

| Runtime condition | Parent session binding | Member invalidation rule | Status result |
| --- | --- | --- | --- |
| First world-scoped member on generation `G` | already persisted by `PLAN-04` | none | live member row can appear normally |
| Auto-restart to generation `G+1` | parent binding persisted first | invalidate every session-local world member where `generation < G+1` | only replacement rows may appear live |
| Fail-closed restart required, replacement not ready | parent binding remains authoritative | invalidate stale generation immediately | member absent from `sessions[]`, trace still historical |
| Concurrent second orchestration session uses same `agent_id` | separate parent session id | never cross-session invalidate | both sessions may surface independently |

## Architecture Diagrams

### Generation rollover commit flow

```text
world restart accepted by backend
    │
    ▼
parent session binding persists generation G+1
    │
    ▼
invalidate_prior_world_members(session, G+1)
    │
    ├── member_a G  -> Invalidated tombstone
    ├── member_b G  -> Invalidated tombstone
    └── host orchestrator untouched
    │
    ▼
replacement member manifests persist with generation G+1
    │
    ▼
world_restarted alert publishes
```

### Status selection and tombstone suppression

```text
participants/*.json + sessions/*.json
        │
        ├── live manifests ----------┐
        ├── invalidated tombstones --┼── suppression set
        └── trace.jsonl history -----┘
                    │
                    ▼
      build_status_report(...)
          ├── live rows selected first
          ├── trace rows suppressed by live identity
          ├── trace rows suppressed by tombstone identity
          └── only remaining historical rows may fall back into sessions[]
```

### Cross-session same-agent isolation

```text
session A / agent codex / world / generation 7
session B / agent codex / world / generation 2
        │
        ├── invalidate session A generation<8
        └── do not touch session B

suppression key = (orchestration_session_id, agent_id, execution.scope)
not               (agent_id, role)
```

## Concrete File Touch Plan

| File / module | Required change | Must not change |
| --- | --- | --- |
| [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) | Add one explicit helper for generation-rollover invalidation and keep lineage terminology honest around `participant_id` vs runtime aliases | Do not widen host/world invariants or invent new states |
| [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Add one sweep helper that invalidates older world members for one orchestration session | Do not add a new index file or rewrite directory layout |
| [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | Replace the suppression key and add tombstone-based trace suppression | Do not redesign the public `status` JSON shape |
| [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reorder restart commit so old generation is dead before replacement is considered live | Do not republish `active_world_binding` or move authority back into trace |
| [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs) | Add status/tombstone/cross-session regression coverage, preferably using `participants/` fixtures as the primary path | Do not rely only on legacy `handles/` fixtures for new coverage |
| [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) | Extend restart ordering coverage to include member invalidation and replacement ordering | Do not duplicate PLAN-04 parent-binding tests without new invalidation assertions |
| [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md) and [agent-hub-session-protocol-spec.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md) | Update terminology and invalidation wording to match shipped behavior | Do not rewrite whole specs |

## Implementation Plan

### Ordered implementation sequence

1. **Correct the record-level invalidation primitive**
   - add one helper that makes a participant invalidation write explicit and consistent.

2. **Add the store sweep**
   - sweep one orchestration session for older world-scoped member generations and persist tombstones.

3. **Fix status suppression**
   - replace `(agent_id, role)` suppression with `(orchestration_session_id, agent_id, execution.scope)`,
   - add tombstone suppression before trace fallback rows are appended.

4. **Wire restart ordering**
   - after `PLAN-04` persists the new parent binding, invalidate stale members before replacement publication and before success alert emission.

5. **Land the regression matrix**
   - same-agent cross-session case,
   - stale trace resurrection case,
   - no-replacement-yet fail-closed case,
   - idempotent sweep case,
   - crash-window ordering case.

### Step-by-step acceptance gates

| Step | Acceptance gate |
| --- | --- |
| 1 | invalidation uses one helper, not ad hoc field mutation in multiple places |
| 2 | older world members for one session become `Invalidated` and disappear from `list_live_manifests()` |
| 3 | trace fallback can no longer resurrect invalidated member rows |
| 4 | restart success alert can only publish after parent binding + member invalidation commit succeed |
| 5 | concurrent sessions that use the same member `agent_id` no longer suppress each other incorrectly |

### Workstream 1: Participant invalidation primitive

Primary file:

- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)

Tasks:

- add `invalidate_for_generation_rollover(...)` or equivalent,
- use `Invalidated` as the terminal state,
- stamp `last_error_bucket` and `last_error_message`,
- keep on-disk lineage expressed as `participant_id` fields,
- do not invent a separate "superseded" runtime state for this slice.

### Workstream 2: Session-local generation sweep

Primary file:

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Tasks:

- add `invalidate_prior_world_members(orchestration_session_id, active_generation, reason)`,
- iterate authoritative participant files,
- filter by:
  - same `orchestration_session_id`
  - `role=member`
  - `execution.scope=world`
  - `world_generation < active_generation`
  - current state still live
- persist each invalidated participant back to `participants/`,
- keep the method idempotent when called twice.

### Workstream 3: Status suppression and trace fail-closed behavior

Primary file:

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Tasks:

- replace `session_fallback_suppression_key(...)`,
- collect suppression keys from live projections,
- collect suppression keys from invalidated world-member tombstones on disk,
- suppress trace fallback on either match,
- preserve host/orchestrator selected-row behavior exactly as it exists today,
- keep nested gateway correlation rules unchanged.

### Workstream 4: Restart ordering and replacement publication

Primary file:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Tasks:

- after `restart_world_session(...)` receives generation `G+1`, keep the parent-binding barrier from `PLAN-04`,
- invalidate stale generation `G` members before replacement publication is treated as complete,
- emit `world_restarted` only after:
  - parent binding persists,
  - old member generation is invalidated,
  - replacement registration succeeds or the code intentionally stays fail closed.

Important:

- replacement may be absent temporarily,
- stale live rows may not remain temporarily.

### Workstream 5: Tests and docs

Primary files:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- [docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md)

Tasks:

- prefer new fixtures that write `participants/*.json` for PLAN-05 cases,
- retain one legacy `handles/*.json` compat read test only where it still matters,
- update spec wording from `session_handle_id` lineage to persisted `participant_id` lineage,
- update TRACE wording so readers know tombstones beat trace fallback for member liveness.

## Architecture Review

### Locked architecture decisions

1. **No new index file.**
   - The parent session already holds the active generation. Spend zero extra machinery until
     `PLAN-06` proves we need it.

2. **Invalidate first, then consider replacement live.**
   - Brief unavailability is acceptable. Dual-live generations are not.

3. **Tombstones suppress trace.**
   - This keeps history and liveness separate without deleting history.

4. **Suppression identity is session-aware.**
   - Same-agent members across concurrent sessions are real. `(agent_id, role)` is not enough.

5. **Persisted lineage names the participant id.**
   - The plan must not make promises about fields that do not serialize.

### Architecture acceptance gates

1. **Generation gate**
   - parent session says generation `G+1`; every generation `< G+1` world member in that session
     is non-live immediately after the commit.

2. **Status gate**
   - invalidated generation `G` member rows do not reappear through trace fallback.

3. **Cross-session gate**
   - session A invalidation never suppresses session B when both use the same member `agent_id`.

4. **Crash-window gate**
   - crashing before replacement persistence leaves absence, not stale liveness.

5. **Contract gate**
   - host/orchestrator status behavior and toolbox proof behavior from `PLAN-04` remain intact.

## Code Quality Review

### Implementation guardrails

1. One helper owns participant invalidation semantics.
2. One store method owns the older-generation sweep.
3. Status suppression reads invalidated tombstones explicitly. It does not infer them from trace.
4. New fixtures should prefer `participants/` as the authority path.
5. Persisted lineage vocabulary stays `participant_id`-first.
6. No slice-local performance cache lands without measurement.

### Minimal-diff rules

- reuse `OrchestrationSessionRecord.world_generation` as the active-generation source,
- reuse `Invalidated` as the terminal state,
- reuse `list_participants()` / `list_live_participants()` rather than inventing a second read path,
- keep `toolbox status --json` unchanged,
- avoid touching CLI schema unless a test proves it is required.

## Error & Rescue Registry

| Failure point | What goes wrong | Expected rescue / fail-closed behavior |
| --- | --- | --- |
| suppression key stays `(agent_id, role)` | one live session suppresses another concurrent session that uses the same member agent | change key to `(orchestration_session_id, agent_id, execution.scope)` |
| stale member invalidates after replacement write | crash window leaves both generations looking live | reorder: invalidate old generation before replacement is considered live |
| trace fallback ignores tombstones | dead generation reappears in `sessions[]` | collect tombstone suppression keys from invalidated world-member participants |
| tests still write only `handles/*.json` | new coverage exercises the compatibility path, not the authority path | add `participants/`-first fixtures for PLAN-05 cases |
| lineage acceptance names skipped runtime alias fields | plan and tests assert the wrong serialized contract | name and assert `resumed_from_participant_id` |
| replacement member never becomes authoritative-live | ownership flags are never established for the first real member producer | define and test the ownership bootstrap contract in the implementation wave |

## Test Review

100% new-path coverage is the goal. This slice is mostly about stale-state edge cases. Those are
the ones that quietly poison operator trust.

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/shell/src/execution/agent_runtime/session.rs
    │
    ├── [★★  TESTED] Invalidated already exists and is excluded from live reads
    └── [GAP]        generation-rollover invalidation helper stamps state + terminal metadata

[+] crates/shell/src/execution/agent_runtime/state_store.rs
    │
    ├── [★★  TESTED] list_live_participants/list_live_manifests already filter non-live rows
    ├── [GAP]        invalidate_prior_world_members(...) sweeps one session only
    ├── [GAP]        sweep skips host orchestrator rows
    ├── [GAP]        sweep is idempotent
    └── [GAP]        sweep + persisted parent generation agree after restart

[+] crates/shell/src/execution/agents_cmd.rs
    │
    ├── [★★★ TESTED] live manifest already beats trace fallback for selected orchestrator cases
    ├── [★★  TESTED] invalidated toolbox env already fails closed
    ├── [GAP]        tombstone suppression beats trace fallback for invalidated members
    ├── [GAP]        suppression key uses orchestration_session_id
    └── [GAP]        same-agent concurrent sessions do not suppress each other

[+] crates/shell/src/repl/async_repl.rs
    │
    ├── [★★★ TESTED] parent binding persists before world_restarted/world_restart_required alerts
    ├── [GAP]        member invalidation runs before replacement publication is considered complete
    └── [GAP]        crash-window ordering is fail-closed rather than dual-live

─────────────────────────────────
COVERAGE: 5/13 paths tested (38%)
  Code paths: 5/13 (38%)
QUALITY:  ★★★: 2  ★★: 3  ★: 0
GAPS: 8 paths need tests
─────────────────────────────────
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] Auto-restart with replacement ready
    │
    ├── [★★  TESTED] parent binding persists before restart alert publishes
    ├── [GAP] [→E2E] old generation members disappear before replacement is surfaced live
    └── [GAP]         replacement lineage persists via resumed_from_participant_id

[+] Fail-closed restart, replacement not ready
    │
    ├── [GAP] [→E2E] member is absent from status, not stale-live
    └── [GAP]         trace rows remain visible only as history

[+] Concurrent sessions using the same member agent
    │
    ├── [GAP]         invalidating session A does not suppress session B
    └── [GAP]         world-filtered status shows both sessions when both are truly live

[+] Crash recovery / rerun behavior
    │
    ├── [GAP]         rerunning the invalidation sweep is idempotent
    └── [GAP]         partial replacement persistence cannot leave two live generations

─────────────────────────────────
COVERAGE: 1/8 flows tested (12%)
  User flows: 1/8 (12%)
GAPS: 7 flows need tests (2 need integration coverage)
─────────────────────────────────
```

### Required test additions by file

#### `crates/shell/src/execution/agent_runtime/session.rs`

Add unit coverage for:

- generation-rollover invalidation helper sets:
  - `state=Invalidated`
  - terminal metadata
  - error bucket / message
- replacement constructor assertions continue to validate world lineage correctly

#### `crates/shell/src/execution/agent_runtime/state_store.rs`

Add unit coverage for:

- `invalidate_prior_world_members("sess_a", 8, "...")` only touches session A
- host orchestrator and host-scoped rows remain untouched
- calling the sweep twice is harmless
- already-invalidated rows remain stable

#### `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

Add contract coverage for:

- invalidated world-member tombstone suppresses stale trace fallback
- generation `8` live replacement wins over generation `7` trace history
- no-replacement-yet case omits the member row entirely
- two concurrent sessions that both use `agent_id=codex` remain independently visible
- persisted lineage is asserted with `resumed_from_participant_id`

#### `crates/shell/tests/repl_world_first_routing_v1.rs`

Add integration coverage for:

- after restart binding persists, the stale member generation invalidates before success publish
- crash or injected failure between invalidation and replacement leaves the member absent
- replacement generation becomes the only live generation after the full restart path

#### Docs

Add wording checks / examples for:

- `participants/*.json` as the authority path
- tombstone-beats-trace wording in `docs/TRACE.md`
- persisted lineage field naming in the successor protocol doc

### Test commands

Run at minimum:

```bash
cargo test -p substrate-shell agent_runtime::state_store -- --nocapture
cargo test -p substrate-shell agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p substrate-shell repl_world_first_routing_v1 -- --nocapture
```

Then run:

```bash
cargo test -p substrate-shell -- --nocapture
cargo test --workspace -- --nocapture
```

### QA artifact

Primary QA artifact for follow-up verification:

[spensermcconnell-feat-restart-invalidation-semantics-eng-review-test-plan-20260430-115447.md](/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-feat-restart-invalidation-semantics-eng-review-test-plan-20260430-115447.md)

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
| --- | --- | --- | --- | --- | --- |
| session-local invalidation sweep | stale generation remains live beside the replacement generation | planned | planned | partial today | yes until fixed |
| tombstone-based status suppression | trace history resurrects invalidated member rows | planned | planned | no today | yes until fixed |
| same-agent concurrent sessions | one session suppresses another because `agent_id` matches | planned | no | no | yes until fixed |
| fail-closed no-replacement window | member keeps looking live even though replacement failed | planned | planned | no today | yes until fixed |
| persisted lineage naming | tests assert skipped runtime alias fields instead of serialized participant lineage | planned | yes | yes | no, but contract drift |
| replacement ownership bootstrap | replacement manifest never qualifies as authoritative-live | planned | partial | partial | yes until producer contract is explicit |

Critical gap rule:

If a generation change can still leave two live generations visible, or if invalidated tombstones
do not suppress trace fallback, this slice is not done.

## Performance Review

This is not a performance project. It is a correctness project.

Still, a few rules matter:

1. The invalidation sweep is bounded to one orchestration session and runs only at restart
   boundaries, not per command.
2. No extra cache or index file is justified before measurement.
3. `agent status` may read full participant state once more for tombstone suppression. That is
   acceptable at current scale and much cheaper than wrong live state.
4. If future scale makes the sweep expensive, that is the signal to land the grouped session store
   in `PLAN-06`, not to bolt on an ad hoc cache here.

## Cross-Phase Themes

These concerns kept repeating and are now locked into the implementation contract:

1. **Fail closed beats stale confidence**
   - The plan intentionally prefers absence over stale presence during restart windows.

2. **Session identity is the real suppression boundary**
   - The same member agent can appear in more than one orchestration session. Status logic has to
     respect that.

3. **Parent session remains the authority bridge**
   - This slice consumes `PLAN-04`. It does not replace it.

4. **Terminology discipline matters**
   - The repo already has runtime aliases and persisted fields that differ. If the doc uses the
     wrong name, the tests will target the wrong contract.

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Participant invalidation helper | `crates/shell/src/execution/agent_runtime/` | — |
| Status suppression key + tombstone pass | `crates/shell/src/execution/` | participant invalidation helper |
| Restart ordering | `crates/shell/src/repl/` | participant invalidation helper |
| Tests and docs | `crates/shell/tests/`, `docs/`, `llm-last-mile/` | status suppression key + tombstone pass, restart ordering |

### Parallel lanes

Lane A: participant invalidation helper  
Lane B: status suppression key + tombstone pass  
Lane C: restart ordering  
Lane D: tests and docs

### Execution order

1. Launch Lane A first. It defines the shared invalidation primitive.
2. Once Lane A lands, launch Lane B and Lane C in parallel worktrees.
3. Run Lane D last after behavior settles.

### Conflict flags

- Lane B and Lane C both depend on the final invalidation helper signature. Do not start them
  until Lane A is merged.
- Lane D will touch assertions for both status and restart behavior. Keep it last to avoid merge churn.

## Deferred Work

There is no `TODOS.md` in the repo root, so explicit deferrals stay here.

1. Session-grouped registry layout under `run/agent-hub/sessions/<id>/...`  
   Why: owned by [06-session-centric-state-store.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/06-session-centric-state-store.md)

2. Explicit active-generation index file  
   Why: defer unless sweep cost proves unacceptable before `PLAN-06`

3. New public status fields for restart placeholders  
   Why: this slice keeps status live-only and uses absence + alerts instead

4. Broader member runtime ownership bootstrap productization  
   Why: real member producers may need follow-on work once they land beyond the current draft/test path

5. Trace-schema widening for `session_handle_id` correlation  
   Why: helpful later, not required for the invalidation contract itself

## NOT in Scope

- grouped session-centric registry migration
- toolbox mutating tools
- new public `agent status` schema
- trace schema redesign
- generic multi-session selection UX
- host orchestrator invalidation from world-member generation rollover
- UI work

## Definition of Done

This slice is done when all of these are true:

1. Parent session `world_generation` from `PLAN-04` is consumed as the active generation source.
2. Every older world-scoped member for that session becomes `Invalidated` after generation rollover.
3. `list_live_manifests()` returns zero stale-generation world members after the invalidation commit.
4. `substrate agent status --json` suppresses stale trace fallback when a matching tombstone exists.
5. Same-agent members in different orchestration sessions do not suppress each other.
6. Replacement lineage is persisted and asserted with `resumed_from_participant_id`.
7. Restart success alert publishes only after parent binding and member invalidation are committed.
8. No-replacement-yet windows remain fail closed and do not keep stale live rows visible.
9. `toolbox status --json` behavior from `PLAN-04` remains unchanged.
10. Docs and fixtures use authoritative storage terminology and field names.

## Completion Summary

- Step 0: Scope Challenge - scope accepted with three repo-state corrections: `participants/` is authoritative, `active_world_binding` already exists, and persisted lineage is `participant_id`-based
- Architecture Review: 5 locked decisions, 5 acceptance gates
- Code Quality Review: 6 guardrails, 5 minimal-diff rules
- Test Review: coverage diagrams produced, 15 concrete gaps/assertions identified
- Performance Review: 0 major performance issues, 4 correctness-first rules
- Cross-Phase Themes: 4 recurring concerns locked into the implementation contract
- Error & Rescue Registry: written
- NOT in scope: written
- What already exists: written
- Dream state delta: written
- TODOS.md updates: deferred scope captured in-plan because no `TODOS.md` exists
- Failure modes: 5 critical gaps flagged until session-local invalidation, tombstone suppression, and restart ordering land
- Outside voice: Claude CLI review completed and incorporated
- Parallelization: 4 lanes, 1 foundation lane first, 2 implementation lanes in parallel, 1 validation lane last
- Lake Score: complete option chosen for every in-slice decision

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Premise | Treat `participants/*.json` as the authoritative participant path | Mechanical | Explicit over clever | That is how the repo works today | Continuing to write the plan around `handles/*.json` |
| 2 | Authority | Use parent `world_generation` from `PLAN-04` as the active-generation source | Mechanical | DRY | Avoids a duplicate generation database | Adding a new index file now |
| 3 | Ordering | Invalidate stale generation before replacement is considered live | Mechanical | Systems over heroes | Favors fail-closed absence over dual-live ambiguity | Replacement-first ordering |
| 4 | Status | Suppression key becomes `(orchestration_session_id, agent_id, execution.scope)` | Mechanical | Completeness | Fixes concurrent same-agent sessions cleanly | Keeping `(agent_id, role)` |
| 5 | Trace | Invalidated tombstones suppress trace fallback | Mechanical | Fail closed | History must not regain authority | Letting trace fill the live-state gap |
| 6 | Lineage | Persisted replacement lineage is asserted via `resumed_from_participant_id` | Mechanical | Explicit over clever | Runtime aliases are not serialized | Naming skipped alias fields in acceptance criteria |
| 7 | Scope | Defer grouped registry reshaping to `PLAN-06` | Mechanical | Minimal diff | Correctness first, layout migration later | Folding `PLAN-06` into this slice |
| 8 | Tests | Prefer `participants/` fixtures for new PLAN-05 regressions | Taste | Pragmatic | New tests should exercise the authority path | Adding only more legacy `handles/` fixtures |

## Outside Voice Summary

Claude CLI review changed four important details:

1. corrected storage terminology from `handles/` to `participants/`,
2. corrected persisted lineage naming from `session_handle_id` aliases to `participant_id` fields,
3. identified `(agent_id, role)` suppression as unsafe for concurrent same-agent sessions,
4. pushed restart ordering to invalidate stale generations before replacement is considered live.

That review agreed with the repo reading. It also sharpened the test plan. Good signal.

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
| --- | --- | --- | --- | --- | --- |
| CEO Review | `/plan-ceo-review` | Scope & strategy | 0 | SKIPPED | No separate CEO pass run for this backend-only slice |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | SKIPPED | No separate Codex review run |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 1 | CLEAR | Corrected authority-path terminology, tightened restart ordering, fixed status suppression boundary, and expanded the regression matrix |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | SKIPPED | No UI scope |

**OUTSIDE VOICE:** Claude CLI review completed. It forced the most important corrections in this plan: authoritative storage path, persisted lineage naming, session-aware suppression keys, and invalidate-before-replacement ordering.

**UNRESOLVED:** 0 blocking design decisions remain inside slice `05`. The main deferred decision is whether a future session index should exist before or inside `PLAN-06`.

**VERDICT:** ENG CLEARED. `PLAN-05` is ready to implement after `PLAN-04` and before `PLAN-06`.
