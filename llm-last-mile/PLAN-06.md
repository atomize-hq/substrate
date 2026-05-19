<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/feat-session-centric-state-store-autoplan-restore-20260430-173741.md -->

# PLAN-06: Session-Centric Runtime State Store

Source file: [06-session-centric-state-store.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/06-session-centric-state-store.md)  
Branch: `feat/session-centric-state-store`  
Plan type: backend and operator-CLI slice, no UI scope  
Review posture: prior `/autoplan` and `/plan-eng-review` findings are now folded into one execution contract  
Status: execution-ready after [PLAN-05](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-05.md)

## Objective

The problem is not directory aesthetics. The problem is that live runtime truth is still partly
reconstructed by callers instead of being served directly by the store.

The repository already has the right identity boundary:

- parent orchestration truth in `sessions/*.json`
- participant truth in `participants/*.json`
- compatibility tail in `handles/*.json`
- toolbox socket naming keyed by `orchestration_session_id`
- status suppression keyed by `(orchestration_session_id, agent_id, execution.scope)`

What is still wrong is the read model. `status`, `toolbox`, and invalidation logic still rely on
flat scans or caller-owned regrouping in places where the store should already be authoritative.

`PLAN-06` fixes that in one bounded slice:

1. add a store-owned session projection over the current runtime sources,
2. move every participant enumerator and mutator onto one shared source walker,
3. cut `agent status` and `toolbox status|env` over to session-record reads,
4. then move canonical persistence under one session-root layout so disk shape matches API shape,
5. keep compatibility reads inside the store until upgrade confidence is green.

After this plan lands:

1. the store can answer `load_session(<id>)` and `list_live_sessions()` directly,
2. `substrate agent status` builds live rows from store-owned session records, not flat manifest regrouping,
3. `substrate agent toolbox status|env` resolves exactly one live session record and fails closed on ambiguity,
4. `PLAN-05` invalidation, tombstones, and same-agent concurrent-session visibility still hold,
5. canonical live runtime state writes under one session root, with flat compatibility reads bounded inside the store.

This slice does not add explicit selector UX such as `--orchestration-session-id`. Fail-closed
ambiguity is sufficient for this step.

## Scope Lock

### Repo truth this plan must follow

The source SOW is directionally right, but some of its current-state description is stale. This
plan is locked to current repository truth:

1. authoritative live runtime state is already split between flat parent session snapshots in
   [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
   and flat participant records in the same module,
2. `handles/*.json` is already legacy compatibility input, not the canonical write target,
3. [`OrchestrationSessionRecord`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
   already owns session-level truth such as `active_session_handle_id`, `world_id`, and `world_generation`,
4. [`resolve_live_orchestrator_session()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
   already fails closed on ambiguity and is the correct behavioral baseline,
5. [`build_status_report()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
   still reconstructs the live overlay from `list_live_manifests()` plus trace fallback suppression,
6. `PLAN-05` helpers such as `invalidate_stale_world_members_for_session()` and
   `list_invalidated_participants()` still enumerate only flat `participants/*.json`,
7. parent-session writes and participant writes are separate atomic rewrites today, so torn roots are a normal runtime condition, not an edge case,
8. test helpers and docs still hardcode flat paths in several places, including
   `agent_successor_contract_ahcsitc0.rs`, `agent_hub_trace_persistence.rs`,
   `repl_world_first_routing_v1.rs`, `docs/TRACE.md`, and `docs/USAGE.md`.

If the implementation or tests contradict any of those eight points, the code is wrong, not the
plan.

### What already exists

| Sub-problem | Existing code | Plan |
| --- | --- | --- |
| Session-level parent truth | [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs) | Reuse |
| Participant validation, liveness, lineage | [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) | Reuse |
| Fail-closed live orchestrator resolution | [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Promote into session-record API |
| Parent world-binding authority | [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs) | Reuse |
| `PLAN-05` tombstone invalidation | [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Preserve unchanged, move to unified walkers |
| Toolbox JSON shape | [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | Keep stable |
| REPL writer already has `orchestration_session_id` | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse, no lifecycle redesign |

### NOT in scope

- public schema changes to `AgentRuntimeParticipantRecord`
- trace event schema redesign
- new toolbox mutating tools
- new CLI selector UX such as `--orchestration-session-id`
- new cache or index files
- replay or doctor contract redesign
- UI work

### Chosen approach

| Approach | Summary | Effort | Risk | Decision |
| --- | --- | --- | --- | --- |
| A. Projection first, then canonical layout in the same slice | Cut consumers onto store-owned session projections first, then align disk layout | Medium | Medium | **Accepted** |
| B. Layout first | Move files first, then repair consumers | Medium | High | Rejected |
| C. Stay flat forever | Add session projection but never move canonical layout | Small | Medium | Rejected as end state |
| D. Add an index file | Keep flat manifests and add derived session index | Medium | High | Rejected |

The accepted path is the smallest correct slice. It solves the operator problem first, keeps the
diff boring, and avoids inventing a second source of truth.

## Execution Contract

### Problem statement

The store has to become session-centric by read contract before it becomes session-centric by disk
shape.

That means:

- session grouping belongs in the store layer, not in `agents_cmd.rs`,
- compatibility burden belongs in the store layer, not in tests or callers,
- parent and participant precedence must be resolved per object, not per directory root,
- torn writes must be modeled explicitly instead of hand-waved away,
- `PLAN-05` invalidation semantics are part of the acceptance gate, not an adjacent concern.

### Canonical target layout

End-state canonical runtime store root:

```text
$SUBSTRATE_HOME/run/agent-hub/
  sessions/
    <orchestration_session_id>/
      session.json
      participants/
        <participant_id>.json
      leases/
        <participant_id>.lease
```

Compatibility inputs during migration:

```text
$SUBSTRATE_HOME/run/agent-hub/
  sessions/<orchestration_session_id>.json
  participants/<participant_id>.json
  handles/<participant_id>.json
```

### Required store record

Add one store-layer session projection type:

```rust
pub(crate) struct AgentRuntimeSessionRecord {
    pub session: OrchestrationSessionRecord,
    pub participants: Vec<AgentRuntimeParticipantRecord>,
    pub warnings: Vec<String>,
}
```

Required helpers:

- `orchestration_session_id()`
- `is_complete()`
- `live_participants()`
- `live_orchestrator()`
- `live_participant_for_agent(agent_id, scope, role)`
- `invalidated_world_members()`
- `last_updated_at()`

`warnings` is the explicit answer to torn writes. This record is internal only. No public JSON
surface changes are required.

### Required store API

```rust
impl AgentRuntimeStateStore {
    pub(crate) fn load_session(
        &self,
        orchestration_session_id: &str,
    ) -> Result<Option<AgentRuntimeSessionRecord>>;

    pub(crate) fn list_sessions(&self) -> Result<Vec<AgentRuntimeSessionRecord>>;

    pub(crate) fn list_live_sessions(&self) -> Result<Vec<AgentRuntimeSessionRecord>>;

    pub(crate) fn resolve_single_live_session_for_agent(
        &self,
        orchestrator_agent_id: &str,
    ) -> Result<Option<AgentRuntimeSessionRecord>>;

    pub(crate) fn list_participants_across_sources(
        &self,
    ) -> Result<Vec<AgentRuntimeParticipantRecord>>;

    pub(crate) fn list_invalidated_participants_across_sources(
        &self,
    ) -> Result<Vec<AgentRuntimeParticipantRecord>>;

    pub(crate) fn invalidate_stale_world_members_for_session(
        &self,
        orchestration_session_id: &str,
        active_generation: u64,
    ) -> Result<Vec<String>>;
}
```

Hard rules:

1. `list_participants_across_sources()` is the one shared participant source walker.
2. `list_invalidated_participants_across_sources()` is the tombstone source for status suppression.
3. `invalidate_stale_world_members_for_session()` must stop assuming only flat `participants/*.json` exists once canonical roots are introduced.
4. `resolve_live_orchestrator_session()` may remain as a compatibility wrapper over `resolve_single_live_session_for_agent()` if that keeps the diff smaller.

### Object-level precedence

The precedence contract is per object, not per root:

```text
Session object:
  canonical sessions/<id>/session.json
    > flat sessions/<id>.json

Participant object:
  canonical sessions/<id>/participants/<participant_id>.json
    > flat participants/<participant_id>.json
    > legacy handles/<participant_id>.json
```

Missing canonical objects do not hide compatibility objects of the same class. That yields the
correct migration behavior:

- missing canonical parent still allows flat parent fallback,
- missing canonical participant still allows flat or legacy participant fallback,
- canonical parent does not automatically erase flat participants,
- canonical participant does not depend on canonical parent already being present.

### Torn-root read model

The runtime already writes parent and participant objects separately. Reads must tolerate all four
states:

```text
runtime writes parent and participant separately
        │
        ├── session.json only ----------------> record + warning
        ├── participant only -----------------> record + warning
        ├── stale active handle --------------> warning, not whole-command failure
        └── complete live pair ---------------> complete session record
```

`list_live_sessions()` only returns complete live records. `load_session()` and `list_sessions()`
may return incomplete records with warnings.

### Session discovery rule

`list_sessions()` must discover candidate session ids from the union of:

1. canonical `sessions/<id>/` directories,
2. flat `sessions/<id>.json` parent files,
3. merged participant records' `orchestration_session_id` values.

This is what makes participant-only torn roots observable instead of invisible.

### Path hardening

Every canonical session root and nested file is treated as untrusted local input:

- ignore symlinked session roots,
- ignore non-regular `session.json` entries,
- ignore non-regular participant files,
- ignore unexpected nested entry types,
- do not follow symlinks under canonical roots.

This is a local trust-boundary hardening requirement, not optional cleanup.

## File-by-File Contract

### 1. `crates/shell/src/execution/agent_runtime/state_store.rs`

This file owns the real change.

Required changes:

1. add helpers for canonical session roots, canonical participant paths, and canonical lease paths,
2. add one participant walker that merges canonical, flat-current, and legacy participant objects,
3. add parent-session loading that prefers canonical `session.json` over flat `sessions/<id>.json`,
4. add `load_session()`, `list_sessions()`, `list_live_sessions()`, and `resolve_single_live_session_for_agent()`,
5. move invalidation and invalidated-participant reads onto the unified walker,
6. add torn-root warnings and incomplete-root handling,
7. add path hardening with `symlink_metadata`,
8. keep the implementation in this file unless a tiny internal helper extraction makes the diff materially clearer.

Anti-goals:

- no new crate,
- no new cache or index file,
- no parallel store implementation for canonical and compatibility reads,
- no caller-owned merging logic.

### 2. `crates/shell/src/execution/agents_cmd.rs`

This file owns the consumer cutover.

Required changes:

1. `build_status_report()` must build authoritative live rows from `list_live_sessions()`,
2. `build_status_report()` must preserve one row per live participant, not one row per `agent_id`,
3. tombstone suppression must use `list_invalidated_participants_across_sources()`,
4. trace fallback may only fill gaps not already covered by a live record or tombstone,
5. `build_toolbox_status_report()` and `build_toolbox_env_report()` must resolve one live session record via `resolve_single_live_session_for_agent()`,
6. `toolbox status` keeps the current JSON fields:
   - `active_orchestration_session_id`
   - `active_world_binding`
   - `dependency_unavailable`
7. ambiguity remains fail closed and operator-readable.

Important repo-truth note:

`docs/USAGE.md` still says live session discovery is backed by `~/.substrate/run/agent-hub/handles/`.
That wording is stale today and must be corrected as part of this slice.

### 3. `crates/shell/src/repl/async_repl.rs`

This file owns writer cutover, not lifecycle redesign.

Required changes:

1. keep the current bootstrap and lifecycle ordering semantics,
2. once canonical pathing is added, move parent, participant, and lease writes onto store-owned canonical helpers,
3. keep any temporary dual-write inside the store only,
4. keep `persist_runtime_snapshots()` as the write choke point,
5. add crash-window tests for:
   - parent written, participant missing,
   - participant written, parent missing,
   - active handle points at stale participant.

This plan does not authorize inventing a new transactional layer.

### 4. Tests, fixtures, and docs

Primary touchset:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- helper readers in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)

Required changes:

1. fixture writers seed canonical session-root trees,
2. flat fixtures remain only where compatibility behavior is being tested explicitly,
3. helper readers stop assuming only `participants/*.json` exists,
4. docs stop claiming `handles/` or flat participant scans are the live authority,
5. test assertions stop depending on incidental pre-cutover path layout.

## Execution Plan

### Phase A: Store projection cutover over current roots

Deliver the store contract before moving writers.

Scope:

- `AgentRuntimeSessionRecord`
- canonical path helpers
- merged participant walker
- parent-session loading with precedence
- `load_session()`, `list_sessions()`, `list_live_sessions()`
- torn-root warnings
- path hardening

Exit criteria:

1. unit tests prove canonical and compatibility reads merge correctly by object,
2. `list_sessions()` can discover participant-only torn roots,
3. `list_live_sessions()` excludes incomplete roots,
4. no production caller still needs to regroup flat participant lists into sessions.

### Phase B: Consumer and invalidation cutover

Move every reader that matters onto the new store contract.

Scope:

- `agent status`
- `toolbox status`
- `toolbox env`
- `list_invalidated_participants_across_sources()`
- `invalidate_stale_world_members_for_session()` over the shared participant walker

Exit criteria:

1. same-agent concurrent sessions remain distinct in `agent status`,
2. ambiguity in `toolbox status|env` fails closed using session-record resolution,
3. `PLAN-05` tombstone suppression still holds after regrouping,
4. no invalidation or tombstone logic still reads only flat `participants/*.json`.

### Phase C: Canonical write cutover

Only after Phase A and Phase B are green:

- canonical `sessions/<id>/session.json` write
- canonical `sessions/<id>/participants/<participant_id>.json` write
- canonical `sessions/<id>/leases/<participant_id>.lease` write
- bounded dual-write only if needed, only inside the store

Exit criteria:

1. runtime writer still tolerates crash windows,
2. store reads remain correct when canonical and flat files coexist,
3. canonical write path is boring and centralized.

### Phase D: Fixture, doc, and compatibility cleanup

This is the consolidation pass:

- fixture updates
- helper-reader updates
- doc corrections
- flat-write removal if the migration bridge is already proven safe
- legacy `handles/*.json` reads removed last, not first

Exit criteria:

1. docs reflect current truth,
2. fixtures default to canonical layout,
3. compatibility behavior is either removed or explicitly justified,
4. no stale path assumptions remain in tests.

### Phase-order diagram

```text
PHASE A
-------
store-owned session projection over current roots
        │
        ├── load_session()
        ├── list_sessions()
        ├── list_live_sessions()
        └── unified participant walker

PHASE B
-------
consumer + invalidation cutover
        │
        ├── status uses live session records
        ├── toolbox resolves one session record
        └── tombstones/invalidation use same source walker

PHASE C
-------
canonical writer cutover
        │
        ├── session.json write
        ├── session-local participant write
        └── session-local lease write

PHASE D
-------
fixture/doc cleanup + compatibility retirement
```

## Architecture Diagrams

### Read-side composition

```text
load_session("sess_a")
    │
    ├── session object
    │     canonical sessions/sess_a/session.json
    │         > flat sessions/sess_a.json
    │
    └── participant objects
          canonical sessions/sess_a/participants/ash_123.json
              > flat participants/ash_123.json
              > legacy handles/ash_123.json
```

### Consumer ownership

```text
AgentRuntimeStateStore
        │
        ├── list_live_sessions() -----------------> agent status
        ├── resolve_single_live_session_for_agent() -> toolbox status/env
        ├── list_invalidated_participants_across_sources() -> status suppression
        └── invalidate_stale_world_members_for_session() -> PLAN-05 invalidation path
```

### Acceptance boundary

```text
CURRENT
-------
callers regroup flat manifests + flat tombstone scans

TARGET
------
store groups by session
callers consume session records
canonical layout follows once reads are green
```

## Test Review

### Runtime and test framework

- Runtime: Rust
- Package: `shell`
- Test framework: `cargo test` with unit and integration suites under `crates/shell/tests/`

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/shell/src/execution/agent_runtime/state_store.rs
    │
    ├── list_participants_across_sources()
    │   ├── [GAP] canonical + flat + legacy merge by participant_id
    │   ├── [GAP] canonical participant beats flat participant
    │   ├── [GAP] flat participant beats legacy handle
    │   └── [GAP] symlinked or non-regular files are ignored
    │
    ├── load_session()
    │   ├── [GAP] canonical parent + canonical participants
    │   ├── [GAP] flat parent + flat participants compatibility load
    │   ├── [GAP] parent-only torn root returns warning, not failure
    │   ├── [GAP] participant-only torn root returns warning, not failure
    │   └── [GAP] stale active handle returns warning, not whole-command failure
    │
    ├── list_live_sessions()
    │   ├── [GAP] incomplete roots are skipped
    │   ├── [GAP] two live sessions with same agent stay distinct
    │   └── [GAP] invalidated tombstones stay attached to the right session
    │
    ├── list_invalidated_participants_across_sources()
    │   └── [GAP] canonical session-root tombstones are visible after flat writes stop
    │
    └── invalidate_stale_world_members_for_session()
        ├── [GAP] canonical session-root participants are invalidated
        ├── [GAP] flat compatibility participants are still covered during migration
        └── [GAP] idempotence survives the new source walker

[+] crates/shell/src/execution/agents_cmd.rs
    │
    ├── build_toolbox_status_report()
    │   ├── [★★ TESTED] current ambiguity already fails closed
    │   ├── [GAP] session-record resolution keeps the same behavior
    │   └── [GAP] incomplete session records are not promoted live
    │
    └── build_status_report()
        ├── [GAP] live rows come from session records, not flat live manifests
        ├── [GAP] same-agent concurrent sessions stay visible
        └── [GAP] tombstone suppression still wins after regrouping

[+] crates/shell/src/repl/async_repl.rs
    │
    ├── bootstrap persistence
    │   ├── [GAP] canonical session-root parent write
    │   └── [GAP] parent-only crash window remains tolerated by reads
    │
    └── lifecycle rewrites
        ├── [GAP] canonical session-local participant write
        ├── [GAP] participant-only crash window remains tolerated by reads
        └── [GAP] invalidation helper still sees moved participants

─────────────────────────────────
COVERAGE: 1/22 paths partly covered today (5%)
GAPS: 21 code paths need direct tests
─────────────────────────────────
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] One live orchestrator session
    │
    ├── [GAP] toolbox status resolves one live session record
    └── [GAP] toolbox env exports the same endpoint

[+] Two live orchestrator sessions for the same agent
    │
    ├── [★★ TESTED] current ambiguity already fails closed
    └── [GAP] session-record ambiguity path keeps the same fail-closed contract

[+] Agent status with concurrent same-agent world sessions
    │
    ├── [GAP] both live rows remain visible
    └── [GAP] one tombstone suppresses only its own session

[+] Upgrade / compatibility window
    │
    ├── [GAP] canonical parent beats flat parent
    ├── [GAP] canonical participant beats flat participant
    ├── [GAP] flat participant beats legacy handle
    └── [GAP] missing canonical participant does not hide flat compatibility participant

[+] Runtime crash windows
    │
    ├── [GAP] session.json only does not break status/toolbox
    ├── [GAP] participant only does not break status/toolbox
    └── [GAP] stale active handle degrades safely

─────────────────────────────────
COVERAGE: 1/13 flows partly covered today (8%)
GAPS: 12 user flows need tests
─────────────────────────────────
```

### Required test additions by file

#### `crates/shell/src/execution/agent_runtime/state_store.rs`

Add unit coverage for:

- object-level precedence across canonical, flat-current, and legacy participant files
- canonical parent vs flat parent precedence
- torn-root warnings:
  - parent only
  - participant only
  - stale active handle
- unified invalidation walker across canonical and flat sources
- symlink and non-regular file rejection

#### `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

Add contract coverage for:

- toolbox status and env resolving from session records
- ambiguity remaining fail closed with operator-readable text
- status showing two live rows for one `agent_id` across two session ids
- canonical participants overriding flat compatibility participants
- flat compatibility participants still appearing if canonical root is incomplete during migration

#### `crates/shell/tests/agent_hub_trace_persistence.rs`

Add integration coverage for:

- canonical session-root persistence retaining the authoritative orchestration session id
- parent and participant rewrites remaining aligned across lifecycle changes
- incomplete roots degrading safely in reads

#### `crates/shell/tests/repl_world_first_routing_v1.rs`

Add integration coverage for:

- `PLAN-05` invalidation still working after participant location changes
- tombstone suppression still working after flat writes stop
- cross-session same-agent world rows remaining isolated after regrouping

#### `crates/shell/src/repl/async_repl.rs`

Add or update helper coverage for:

- helper readers that currently scan `participants/*.json`
- bootstrap and lifecycle crash windows under the new store contract

#### Docs

Update examples or wording in:

- `docs/TRACE.md`
- `docs/USAGE.md`
- successor protocol draft as needed

### Test commands

Run at minimum:

```bash
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test agent_hub_trace_persistence -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

Then run:

```bash
cargo test -p shell -- --nocapture
cargo clippy -p shell --all-targets -- -D warnings
```

### QA artifact

Primary QA handoff artifact:
[spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260430-162706.md](/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260430-162706.md)

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
| --- | --- | --- | --- | --- | --- |
| unified participant source walker | invalidation or tombstones stop seeing moved participants | planned | planned | no today | yes until fixed |
| torn-root tolerant reads | normal runtime transitions start failing status/toolbox | planned | planned | partial today | yes until fixed |
| object-level precedence | incomplete canonical roots hide still-valid compatibility objects | planned | planned | no today | yes until fixed |
| status projection cutover | same-agent concurrent sessions collapse back into one row | planned | no today | no today | yes until fixed |
| toolbox session resolution | CLI silently promotes incomplete or ambiguous session state | partly today | yes | yes | yes until session-record path is covered |
| nested directory traversal | symlinked session roots or files poison operator reads | planned | no today | no today | yes until hardened |

Critical gap rule:

If invalidation or tombstone suppression can miss canonical session-root participants, or if normal
parent/participant torn writes can now break `status` or `toolbox`, this slice is not done.

## Performance Review

This is a correctness slice, not a performance slice.

Hard rules:

1. no index file lands before measurement,
2. projection and consumer truth come before micro-optimization,
3. one top-level scan per command is acceptable at current scale,
4. if canonical scans later prove too slow, measure first and add an index later.

The performance footgun is keeping the flat global scan forever because it already exists. That is
the smaller diff now and the more expensive platform forever.

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| A. Session projection + unified source walker | `crates/shell/src/execution/agent_runtime/` | — |
| B. Status/toolbox consumer cutover | `crates/shell/src/execution/` | A |
| C. Canonical pathing + writer cutover | `crates/shell/src/execution/agent_runtime/`, `crates/shell/src/repl/` | A |
| D. Fixtures, helpers, docs, and compatibility cleanup | `crates/shell/tests/`, `docs/`, `llm-last-mile/` | B, C |

### Parallel lanes

- Lane A: Step A
- Lane B: Step B
- Lane C: Step C
- Lane D: Step D

### Execution order

1. launch Lane A first,
2. once Lane A lands, launch Lane B and Lane C in parallel worktrees,
3. run Lane D last after both contracts settle.

### Conflict flags

- Lanes B and C both depend on the final session-record API shape.
- Lanes A and C both touch `crates/shell/src/execution/agent_runtime/`, so they are not parallel.
- Lane D touches both fixture helpers and operator docs, so keep it last.

### Parallelization verdict

This plan has one foundation lane, two safe parallel implementation lanes, and one final
validation lane.

## Deferred Work

There is no `TODOS.md` in the repo root, so explicit deferrals stay here:

1. explicit CLI session selector such as `--orchestration-session-id`
   - fail-closed ambiguity is enough for this slice
2. removing flat compatibility reads entirely
   - do it after real upgrade confidence exists
3. any measured index or cache file
   - only after canonical scans prove too slow
4. broader session history or list-session productization
   - this slice only builds the storage and projection boundary those features need

## Definition of Done

This slice is done only when all of the following are true:

1. the store can load one orchestration session and list live sessions directly,
2. `agent status` consumes store-owned live session records,
3. `toolbox status|env` consumes store-owned live session resolution,
4. invalidation and invalidated-participant reads use the unified participant source walker,
5. `PLAN-05` tombstone invalidation and suppression semantics still pass after regrouping,
6. torn roots produced by normal runtime transitions degrade safely in reads,
7. canonical session-root layout is the write target by the end of the slice,
8. canonical-vs-flat precedence is resolved per object, not per root,
9. symlinked or non-regular canonical entries are ignored,
10. docs and fixtures stop describing flat handles or flat participant scans as the live authority.

## Completion Summary

- Step 0: scope accepted as-is after repo-state correction
- Architecture: store-owned session projection first, canonical pathing second
- Code shape: one grouping boundary, one participant walker, one precedence rule
- Tests: coverage diagrams produced, 33 concrete gaps or assertions identified
- Performance: no cache or index expansion justified
- NOT in scope: written
- What already exists: written
- Failure modes: 6 critical gaps remain until projection, precedence, and torn-write handling land
- Parallelization: 4 steps, 2 safe parallel implementation lanes
- Lake Score: complete option chosen for every in-slice decision

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Scope | Reframe the slice around session addressability, not directory shape | Mechanical | User outcome first | That is the real operator problem still left in the repo | Leading with layout as the problem statement |
| 2 | Architecture | Choose projection first, then layout migration | Mechanical | Pragmatic | Solves truth and ambiguity first, migrates disk locality second | Layout-first cutover |
| 3 | Compatibility | Merge by object, not by root | Mechanical | Explicit over clever | Parent precedence and participant precedence are different contracts | Root-level winner-take-all |
| 4 | Correctness | Move every enumerator and mutator onto one unified participant source walker | Mechanical | DRY | `PLAN-05` semantics cannot live on a different read path | Migrating only `status` and `toolbox` |
| 5 | Read contract | Model torn writes with warnings and incomplete-root handling | Mechanical | Systems over heroes | Parent and participant rewrites are already separate in the runtime | Pretending whole-session atomicity exists |
| 6 | Security | Reject symlinked or non-regular canonical entries | Mechanical | Boring by default | Nested directory traversal widens the local trust boundary | Trusting nested paths blindly |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
| --- | --- | --- | --- | --- | --- |
| CEO Review | `/plan-ceo-review` | Scope and strategy | 1 | CLEAR | Reframed the slice around operator session addressability, added the hybrid projection-first alternative, and demoted layout migration from the problem statement to an enabling step |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | SKIPPED | No separate Codex review run |
| Eng Review | `/plan-eng-review` | Architecture and tests (required) | 1 | CLEAR | Added unified participant source walking, torn-root tolerance, object-level precedence, symlink hardening, and the broader regression matrix needed to keep `PLAN-05` semantics intact |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | SKIPPED | No UI scope |

**CROSS-MODEL:** The review passes converged on the same correction. The plan only becomes sound when it is projection-first, consumer-first, and explicit about torn writes and compatibility precedence.

**UNRESOLVED:** 0 blocking design decisions remain inside slice `06`. The main deliberate deferral is explicit selector UX for ambiguous multi-session operator workflows.

**VERDICT:** CEO + ENG CLEARED. `PLAN-06` is ready to implement after `PLAN-05`.
