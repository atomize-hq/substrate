<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/feat-restart-invalidation-semantics-autoplan-restore-20260430-121358.md -->

# PLAN-05: Restart Invalidation Semantics for Live State

Source file: [05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md)  
Branch: `feat/restart-invalidation-semantics`  
Plan type: backend-only, no UI scope  
Review posture: consolidation pass with `/plan-eng-review` structure and rigor  
Status: execution-ready after `PLAN-04`, before `PLAN-06`

## Objective

`PLAN-03` made shared-world ownership and generation backend-authoritative.

`PLAN-04` made the parent orchestration-session record the host-side authority for the active
`world_id` and `world_generation`.

`PLAN-05` is the next narrow slice. Its job is to make that authoritative generation transition
invalidate stale world-scoped member live state immediately and permanently for live surfaces.

After this plan lands:

1. when a parent `orchestration_session_id` advances from generation `G` to `G+1`, every older
   world-scoped member for that same session stops being authoritative-live,
2. invalidated member tombstones suppress trace fallback, so old rows stay historical only,
3. restart sequencing fails closed if replacements are not ready,
4. `status` and toolbox surfaces keep using the current live-state authority instead of
   reconstructing stale liveness from trace.

This plan does **not** redesign storage layout, trace schema, or session grouping. That remains
owned by [PLAN-06](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/06-session-centric-state-store.md).

## Step 0: Scope Challenge

### 0A. Repo truth that this plan must follow

The source SOW is directionally right, but the repository has moved since it was written. This
plan is locked to current repo truth:

1. authoritative participant storage is
   [`participants/*.json`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs),
   not `handles/*.json`; `handles/*.json` is legacy compatibility input,
2. the authoritative active generation already lives on
   [`OrchestrationSessionRecord.world_id/world_generation`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs),
3. [`list_live_manifests()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
   is already the right low-level live read and stays in place,
4. [`toolbox status --json`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
   already exposes `active_world_binding` from `PLAN-04`,
5. runtime aliases such as `session_handle_id` and `resumed_from_session_handle_id` still exist,
   but the persisted lineage field on disk is `resumed_from_participant_id`.

If the implementation or tests contradict any of the five rules above, the code is wrong, not the
plan.

### 0B. Existing code to reuse

| Sub-problem | Existing code | Plan |
| --- | --- | --- |
| Parent session already stores active world binding | [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs) | Reuse exactly |
| Parent binding persistence already exists | [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse exactly |
| Participant state already has `Invalidated` and authoritative-live checks | [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) | Reuse, add one explicit invalidation helper |
| Live participant reads already exclude invalidated rows | [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse, do not replace |
| Status already overlays live manifests over trace | [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | Reuse, narrow the suppression boundary and add tombstones |
| Restart alerts already prove parent binding persistence ordering | [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) | Reuse, extend for member invalidation ordering |
| Toolbox already proves live parent binding from the authoritative session record | [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | Reuse, do not add a second proof surface |

### 0C. Minimum viable slice

The smallest correct slice is:

1. one record-level invalidation helper on participants,
2. one session-local sweep in the runtime state store,
3. one status suppression fix that uses invalidated tombstones,
4. one restart-ordering rule that prefers fail-closed absence over stale liveness,
5. regression coverage for restart, tombstones, and cross-session same-agent collisions.

Everything else is extra.

### 0D. Complexity check

This plan touches a bounded set of surfaces:

- runtime state model: `execution/agent_runtime/`
- status surface: `execution/agents_cmd.rs`
- restart orchestration: `repl/async_repl.rs`
- regression suites: `crates/shell/tests/`
- contract docs: `docs/TRACE.md` and the successor protocol draft

That is one bounded runtime seam, not a rewrite:

- 0 new crates
- 0 new storage backends
- 0 new public JSON surfaces
- 0 new artifact types or distribution work

Scope reduction is **not** needed. The slice is already minimal.

### 0E. Search and built-in check

No framework or runtime built-in replaces this work. The missing behavior is local repository
policy:

- how participant invalidation is recorded,
- how live state is selected,
- how trace fallback is suppressed,
- how restart commit ordering avoids dual-live generations.

This is a repo seam, not a library shopping problem.

### 0F. What already exists

- `AgentRuntimeSessionState::Invalidated` already exists.
- `is_authoritative_live()` already excludes non-live participant rows.
- parent sessions already persist `world_id/world_generation`.
- `toolbox status --json` already reports `active_world_binding` when the parent session is live.
- restart alert tests already prove the parent binding persists before alert publication.
- toolbox env already fails closed on invalidated manifests.
- world-scoped member status rows already carry top-level `world_id/world_generation`.

### 0G. NOT in scope

- session-grouped registry migration under `run/agent-hub/sessions/<id>/...`
- new active-generation index files
- trace schema redesign
- new public `agent status` fields for restart placeholders
- host-orchestrator invalidation from member generation rollover
- toolbox mutating tools
- UI work

## Architecture Contract

### No-ambiguity rules

1. `participants/*.json` plus `sessions/*.json` are the authoritative runtime store.
2. Only `role=member` plus `execution.scope=world` participants are invalidated by shared-world
   generation rollover.
3. The active generation source is the parent
   `OrchestrationSessionRecord.world_generation` persisted by `PLAN-04`.
4. `PLAN-05` consumes that generation. It does not assign it.
5. A participant in `state=invalidated` is a tombstone for live surfaces. It stays on disk for
   audit and compatibility reads, but it may not re-authorize live state.
6. Trace fallback suppression identity is
   `(orchestration_session_id, agent_id, execution.scope)`. `(agent_id, role)` is too weak.
7. Persisted replacement lineage uses `resumed_from_participant_id`.
8. Restart ordering prefers fail-closed absence over stale presence.
9. `status` stays a live-session surface, not a historical ledger.
10. `PLAN-06` still owns registry regrouping and migration away from the flat participant tree.

### Current state to target state

```text
CURRENT
-------
parent session already persists the active world binding
        │
        ├── live participant reads exclude Invalidated
        ├── restart alerts publish authoritative generation
        └── status still suppresses trace by (agent_id, role)

TARGET
------
parent session advances to generation G+1
        │
        ├── state store invalidates generation < G+1 world members for that session
        ├── invalidated members remain on disk as tombstones
        ├── status suppresses stale trace rows by session-aware identity
        └── replacement members appear live only after old generation is already dead

LATER
-----
PLAN-06 groups the same parent + member lineage under session-centric storage
without changing the invalidation rule
```

### Restart ordering contract

This is the critical sequencing rule:

```text
backend accepts replacement world binding G+1
        │
        ▼
PLAN-04 persists parent session binding for G+1
        │
        ▼
PLAN-05 invalidates every older world-scoped member in that session
        │
        ▼
replacement member manifests persist for G+1
        │
        ▼
restart success reporting becomes visible
```

The order is strict because the bad crash window is "replacement looks live before stale
generation is dead."

Crash behavior must be:

- after invalidation, before replacement persistence: member is absent, which is safe
- after replacement persistence, before invalidation: two generations can look live, which is not safe

### Status and tombstone suppression contract

`substrate agent status --json` remains a live-state view. It must:

1. build live projections from authoritative live manifests first,
2. build a second suppression set from invalidated world-scoped member tombstones,
3. suppress trace fallback rows if either a live row or a tombstone owns the same
   `(orchestration_session_id, agent_id, execution.scope)` identity,
4. keep host-orchestrator behavior unchanged,
5. keep trace historical. Historical rows may remain on disk forever. They may not come back into
   `sessions[]` once a tombstone exists.

### Architecture diagrams

#### Generation rollover commit flow

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

#### Status selection and tombstone suppression

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

#### Cross-session same-agent isolation

```text
session A / agent codex / world / generation 7
session B / agent codex / world / generation 2
        │
        ├── invalidate session A generation < 8
        └── do not touch session B

suppression key = (orchestration_session_id, agent_id, execution.scope)
not               (agent_id, role)
```

## Concrete File Touch Plan

### 1. Participant invalidation primitive

Primary file:
[crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)

Add one explicit helper, for example:

```text
invalidate_for_generation_rollover(reason, error_bucket)
    1. transition_state(Invalidated)
    2. mark_terminal_state(reason)
    3. set last_error_bucket / last_error_message
    4. rely on terminal-state handling to clear ownership validity
```

Required behavior:

- uses existing `Invalidated`, not a new "superseded" state
- records terminal metadata consistently
- records failure reason in one place
- keeps persisted lineage vocabulary `participant_id`-first
- does not alter host/world invariants

Must not do:

- invent a new state enum variant
- mutate invalidation fields ad hoc in multiple call sites
- rename persisted lineage fields to match runtime aliases

### 2. Session-local generation sweep

Primary file:
[crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Add one bounded helper:

```text
invalidate_prior_world_members(
    orchestration_session_id,
    active_generation,
    reason
) -> Vec<participant_id>
```

Rules:

- iterate authoritative participant files, not just live projections
- mutate only rows that satisfy all of:
  - same `orchestration_session_id`
  - `role=member`
  - `execution.scope=world`
  - `world_generation < active_generation`
  - current state is still live
- skip host-scoped participants
- skip already-invalidated rows
- persist each mutated participant back to `participants/*.json`
- return invalidated participant ids for logging and test assertions
- remain idempotent when called twice

Must not do:

- add a session-level index file
- rewrite directory layout
- change `list_live_manifests()` semantics

### 3. Status suppression and trace fail-closed behavior

Primary file:
[crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Required changes:

- replace `session_fallback_suppression_key(...)`
- build suppression keys from live projections
- build suppression keys from invalidated world-member tombstones on disk
- suppress trace fallback on either match
- keep host-orchestrator selected-row behavior unchanged
- keep nested gateway correlation logic unchanged

Required suppression identity:

```text
(orchestration_session_id, agent_id, execution.scope)
```

Must not do:

- keep `(agent_id, role)` as the live/trace suppression boundary
- suppress session B because session A used the same agent id
- redesign the public `status` JSON shape

### 4. Restart ordering and replacement publication

Primary file:
[crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Required changes:

- keep the `PLAN-04` parent-binding barrier
- after generation `G+1` is accepted and persisted on the parent, invalidate stale generation
  `G` member participants before replacement publication is considered complete
- allow temporary absence
- forbid temporary stale-live visibility
- emit `world_restarted` only after:
  - parent binding persists,
  - old member generation is invalidated,
  - replacement publication succeeds, or the path intentionally remains fail closed

Must not do:

- publish replacement success before stale generation is dead
- keep the old generation live "for continuity"
- move authority back into trace

### 5. Tests and docs

Primary files:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- [docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md)

Required changes:

- new regression fixtures prefer `participants/*.json`
- keep at most one legacy `handles/*.json` compatibility test where it still matters
- update doc wording so tombstones beat trace for live-state selection
- update lineage wording so the persisted field name is `resumed_from_participant_id`

## Error & Rescue Registry

| Failure point | What goes wrong | Required rescue |
| --- | --- | --- |
| suppression key stays `(agent_id, role)` | one live session suppresses another unrelated concurrent session with the same member agent | change key to `(orchestration_session_id, agent_id, execution.scope)` |
| stale generation invalidates after replacement is surfaced | crash window can leave two generations live | reorder to invalidate old generation before replacement is considered live |
| trace fallback ignores tombstones | dead generation comes back into `sessions[]` | build tombstone suppression from invalidated world-member participants |
| tests still write only `handles/*.json` | regressions exercise the compatibility path, not the authority path | add `participants/*.json`-first fixtures for this slice |
| plan/tests use runtime alias field names | implementation targets the wrong serialized contract | name and assert `resumed_from_participant_id` |

## Test Review

100 percent new-path coverage is the target. This slice is mostly stale-state and restart-window
logic. Those are the paths that quietly poison operator trust if they are not tested.

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
    ├── [★★  TESTED] live-manifest reads already filter non-live rows
    ├── [GAP]        invalidate_prior_world_members(...) sweeps one session only
    ├── [GAP]        sweep skips host orchestrator rows
    ├── [GAP]        sweep is idempotent
    └── [GAP]        sweep and persisted parent generation stay in sync after restart

[+] crates/shell/src/execution/agents_cmd.rs
    │
    ├── [★★★ TESTED] live manifest already beats trace fallback for selected orchestrator cases
    ├── [★★  TESTED] invalidated toolbox env already fails closed
    ├── [GAP]        tombstone suppression beats trace fallback for invalidated members
    ├── [GAP]        suppression key uses orchestration_session_id
    └── [GAP]        same-agent concurrent sessions do not suppress each other

[+] crates/shell/src/repl/async_repl.rs
    │
    ├── [★★★ TESTED] parent binding persists before restart alerts publish
    ├── [GAP]        member invalidation runs before replacement publication is considered complete
    └── [GAP]        crash-window ordering is fail closed rather than dual-live

─────────────────────────────────
COVERAGE: 5/13 paths tested (38%)
QUALITY:  ★★★: 2  ★★: 3  ★: 0
GAPS: 8 code paths need tests
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
GAPS: 7 user flows need tests (2 need integration coverage)
─────────────────────────────────
```

### Required test additions by file

#### `crates/shell/src/execution/agent_runtime/session.rs`

Add unit coverage for:

- generation-rollover invalidation helper sets:
  - `state=Invalidated`
  - terminal metadata
  - `last_error_bucket`
  - `last_error_message`
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

- after restart binding persists, stale member generation invalidates before success publication
- crash or injected failure between invalidation and replacement leaves the member absent
- replacement generation becomes the only live generation after the full restart path

#### Docs

Add wording checks or examples for:

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

Primary QA handoff artifact:
[spensermcconnell-feat-restart-invalidation-semantics-eng-review-test-plan-20260430-115447.md](/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-feat-restart-invalidation-semantics-eng-review-test-plan-20260430-115447.md)

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
| --- | --- | --- | --- | --- | --- |
| session-local invalidation sweep | stale generation remains live beside the replacement generation | planned | planned | partial today | yes until fixed |
| tombstone-based status suppression | trace history resurrects invalidated member rows | planned | planned | no today | yes until fixed |
| same-agent concurrent sessions | one session suppresses another because `agent_id` matches | planned | no | no | yes until fixed |
| fail-closed no-replacement window | member keeps looking live even though replacement failed | planned | planned | no today | yes until fixed |
| persisted lineage naming | tests assert runtime alias fields instead of serialized participant lineage | planned | yes | yes | no, but contract drift |

Critical gap rule:

If generation rollover can still leave two live generations visible, or if invalidated tombstones
do not suppress trace fallback, this slice is not done.

## Performance Review

This is a correctness project, not a performance project.

Still, four rules matter:

1. the invalidation sweep is bounded to one orchestration session and runs only at restart boundaries,
2. no cache or index file is justified before measurement,
3. `agent status` can afford one extra participant read pass for tombstone suppression at current
   scale,
4. if scale later makes the sweep expensive, that is the trigger for `PLAN-06`, not for an ad hoc
   cache inside this slice.

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| A. Participant invalidation helper | `crates/shell/src/execution/agent_runtime/` | — |
| B. Status suppression key + tombstone pass | `crates/shell/src/execution/` | A |
| C. Restart ordering | `crates/shell/src/repl/` | A |
| D. Tests and docs | `crates/shell/tests/`, `docs/`, `llm-last-mile/` | B, C |

### Parallel lanes

- Lane A: step A
- Lane B: step B
- Lane C: step C
- Lane D: step D

### Execution order

1. launch Lane A first, because it defines the shared invalidation primitive and helper signature,
2. after Lane A lands, launch Lane B and Lane C in parallel worktrees,
3. run Lane D last, once status behavior and restart ordering are stable.

### Conflict flags

- Lanes B and C both depend on the final helper signature from Lane A. Do not start them early.
- Lane D will touch assertions for both status and restart behavior. Keep it last to reduce merge churn.

### Parallelization verdict

This plan has **one foundation lane**, **two safe parallel implementation lanes**, and **one final
validation lane**.

## Deferred Work

There is no `TODOS.md` in the repo root, so explicit deferrals stay here:

1. session-grouped registry layout under `run/agent-hub/sessions/<id>/...`
   - owned by [PLAN-06](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/06-session-centric-state-store.md)
2. explicit active-generation index file
   - defer unless sweep cost proves unacceptable before `PLAN-06`
3. new public status fields for restart placeholders
   - this slice keeps status live-only and uses absence plus alerts instead
4. broader member runtime ownership bootstrap productization
   - may need follow-on work once real member producers land beyond the current draft and test path
5. trace-schema widening for old/new participant correlation
   - useful later, not required for the invalidation contract itself

## Definition of Done

This slice is done only when all of the following are true:

1. parent session `world_generation` from `PLAN-04` is the active-generation source,
2. every older world-scoped member for that session becomes `Invalidated` after generation rollover,
3. `list_live_manifests()` returns zero stale-generation world members after invalidation commits,
4. `substrate agent status --json` suppresses stale trace fallback when a matching tombstone exists,
5. same-agent members in different orchestration sessions do not suppress each other,
6. replacement lineage is persisted and asserted with `resumed_from_participant_id`,
7. restart success reporting becomes visible only after parent binding and member invalidation commit,
8. no-replacement-yet windows remain fail closed and do not keep stale live rows visible,
9. `toolbox status --json` behavior from `PLAN-04` remains unchanged,
10. docs and fixtures use authoritative storage terminology and persisted field names.

## Completion Summary

- Step 0: scope accepted as-is after repo-state corrections
- Architecture Review: one bounded invalidation contract, no storage rewrite
- Code Quality Review: one helper, one sweep, one suppression fix, one restart-ordering rule
- Test Review: coverage diagrams produced, 15 concrete gaps or assertions identified
- Performance Review: correctness-first, no new caches or indexes justified
- NOT in scope: written
- What already exists: written
- Failure modes: 4 critical gaps remain until invalidation, suppression, and ordering land
- Parallelization: 4 steps, 2 safe parallel implementation lanes
- Lake Score: complete option chosen for every in-slice decision

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
| --- | --- | --- | --- | --- | --- |
| CEO Review | `/plan-ceo-review` | Scope and strategy | 0 | SKIPPED | Backend-only slice, no separate CEO pass run |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | SKIPPED | No separate Codex review run |
| Eng Review | `/plan-eng-review` | Architecture and tests (required) | 1 | CLEAR | Corrected authority-path terminology, tightened restart ordering, fixed suppression boundary, expanded regression matrix |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | SKIPPED | No UI scope |

**UNRESOLVED:** 0 blocking design decisions remain inside slice `05`. The main deferred decision is whether a future active-generation index is still needed once `PLAN-06` lands.

**VERDICT:** ENG CLEARED. `PLAN-05` is ready to implement after `PLAN-04` and before `PLAN-06`.
