# PLAN-07: Shared-World Replacement Ordering, Rollback, and Atomic Metadata

Source file: [07-world-replacement-ordering-rollback-atomic-metadata.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/07-world-replacement-ordering-rollback-atomic-metadata.md)  
Branch: `feat/session-centric-state-store`  
Plan type: Linux backend hardening and proof-preservation slice, no UI scope  
Review posture: `/autoplan`-style scope tightening with `/plan-eng-review` structure and rigor  
Status: execution-ready as a repo-truth and regression-proof contract

## Objective

This is not a greenfield design doc. The Linux backend already contains the core replacement
mechanics this slice needs. The job here is to turn that existing behavior into one explicit,
regression-proof contract that later slices can consume without reinterpreting it.

`PLAN-07` freezes the backend contract that `PLAN-04`, `PLAN-05`, and `PLAN-06` depend on:

- replacement ordering must never leave an orchestration session without one recoverable active
  world,
- `session.json` writes must remain atomic for every shared-world transition,
- recovery must stay deterministic and fail closed on ambiguity,
- downstream proof consumers must continue to accept only `binding_state=Active`.

This is a narrow slice. It is also a hard one. Everything after slice `03` assumes the backend can
answer one question honestly: "what is the one reusable active world for this orchestration session
right now?" If restart can lie about that, the rest of the stack becomes a more sophisticated way
to persist corruption.

## Step 0: Scope Challenge

### 0A. Repo truth and why this slice exists

`PLAN-03` established explicit shared-world ownership. `PLAN-04`, `PLAN-05`, and `PLAN-06` all
consume that ownership.

What still needs to be locked down is the restart seam inside [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs) and [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs). That seam decides whether a restart produces:

- one durable active world,
- two ambiguous candidates,
- or zero valid worlds because the old one was moved aside before the new one was real.

That is the whole game.

### 0B. Existing code to reuse

| Sub-problem | Existing code | Plan |
| --- | --- | --- |
| Shared-owner replacement entrypoint | [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs) | Reuse |
| Pre-commit, rollback, and finalize state transitions | [SessionWorld::set_shared_binding_state(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs) | Reuse |
| Shared-world recovery from persisted metadata | [SessionWorld::recover_shared_active_from_root(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs) | Reuse |
| Atomic `session.json` write path | [SessionWorld::persist_metadata(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs) | Reuse |
| Downstream proof validation | [resolve_shared_world_binding(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs) | Reuse |
| PTY and non-PTY shared-world transport | [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs) and [crates/world-agent/src/pty.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs) | Reuse |
| Existing regression tests for success, rollback, and recovery | [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs) and [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs) | Extend |

### 0C. Minimum viable slice

Do this, and only this:

- preserve the current `Active -> Replacing -> Replaced` transaction shape,
- preserve rollback to `Active` on replacement-create failure,
- preserve atomic `session.json` writes for creation, replacement, rollback, and recovery repair,
- widen the regression net around finalize-warning, ambiguity, malformed proof, and temp-file
  cleanup behavior,
- keep world-agent proof consumers fail closed.

Do not:

- redesign runtime-state projection,
- move ownership authority into shell state,
- introduce a new registry, index, or cache file,
- change selected-orchestrator UX,
- expand cross-platform shared-world parity in this slice.

### 0D. Complexity check

This slice is intentionally smaller than `PLAN-04` through `PLAN-06`.

The honest production seam is four files:

1. [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)
2. [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)
3. [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
4. [crates/world-agent/src/pty.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs)

That is small enough to keep boring. Good. Boring is what you want in restart correctness.

### 0E. Search and built-in check

`[Layer 1]` wins here.

The repo already has the right primitives:

- explicit shared-owner proof types,
- explicit `Replacing` and `Replaced` states,
- one backend-local mutex for shared-owner replacement ordering,
- atomic rename-based metadata persistence,
- fail-closed proof validation at the agent boundary.

The correct move is to preserve and tighten that contract, not invent a second authority or a more
abstract restart engine.

### 0F. What already exists

- the backend-local shared-owner mutex already serializes replacement entry,
- replacement already has a pre-commit `Replacing` state instead of pretending create/finalize is
  one atomic in-memory action,
- recovery already distinguishes `Active`, `Replacing`, `Replaced`, and malformed metadata,
- proof validation already rejects mismatched ownership and non-Active state in
  `resolve_shared_world_binding(...)`,
- repo tests already cover the happy-path replacement and rollback shape.

That means the plan is not "invent the mechanism." It is "freeze the mechanism, tighten the
failure rules, and close the remaining proof gaps."

### 0G. NOT in scope

- runtime-state projection work already owned by [04-thread-world-binding-into-runtime-state.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/04-thread-world-binding-into-runtime-state.md)
- generation invalidation and member replacement semantics already owned by [05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md)
- session-centric registry reshaping already owned by [06-session-centric-state-store.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/06-session-centric-state-store.md)
- non-Linux shared-world replacement parity
- UI, CLI selector, or status-schema redesign

## Architecture Contract

### No-ambiguity rules

1. Only `binding_state=Active` is reusable downstream.
2. `Replacing` is an internal pre-commit state, not a public steady state.
3. The old world may move from `Active` to `Replacing` before the new world exists, but only if
   rollback to `Active` is still possible.
4. Replacement creation success must commit a new world with `world_generation = old + 1`.
5. Replacement creation failure must restore the old world to `Active` before returning failure.
6. Old-world finalization failure after replacement commit is a warning path, not a rollback path.
7. Recovery must prefer one committed `Active` replacement over an older `Replacing` predecessor.
8. Recovery must fail closed on multiple `Active` candidates or multiple unreconciled
   `Replacing` candidates.
9. Malformed, partial, cross-owned, or ownerless shared metadata is never adopted for
   shared-owner reuse.
10. Atomic metadata persistence applies to initial create, pre-commit, rollback, finalize, and
    recovery-driven repair writes.

### Replacement transaction diagram

```text
shared owner replacement for orchestration_session_id = S

current durable state
    │
    └── old world = Active(g)
            │
            ├── pre-commit metadata write
            ▼
        old world = Replacing(g)
            │
            ├── create replacement world root + metadata
            │       │
            │       ├── success
            │       ▼
            │   new world = Active(g+1)
            │       │
            │       ├── finalize old world metadata
            │       ▼
            │   old world = Replaced(g)
            │
            └── failure
                    │
                    ├── rollback old world metadata to Active(g)
                    ├── remove partial replacement root
                    ▼
                return failure with create/rollback/cleanup detail
```

Unsafe window allowed:

- old world temporarily marked `Replacing`, but still recoverable

Unsafe window forbidden:

- old world no longer reusable, new world not durably committed, and recovery has no valid active
  candidate

### Atomic metadata write contract

`SessionWorld::persist_metadata()` must keep this exact posture:

```text
serialize metadata
    │
    ├── write temp file inside world directory
    ├── sync temp file
    ├── rename temp -> session.json
    ├── best-effort sync containing directory
    └── remove temp file on failure paths
```

Why this matters:

- restart state transitions are metadata-driven,
- recovery trusts `session.json`,
- a torn write in `session.json` is not "just a doc corruption issue," it is an authority
  corruption issue.

### Recovery precedence

```text
recovery scan for one owner S
    │
    ├── Active worlds
    │     ├── 0 -> keep scanning
    │     ├── 1 -> candidate winner unless a same/newer Replacing makes state ambiguous
    │     └── >1 -> fail closed
    │
    └── Replacing worlds
          ├── 0 -> no recovery candidate
          ├── 1 with no Active -> repair back to Active and return it
          └── >1 -> fail closed
```

Hard rule:

- `Replaced` and `Abandoned` are never reusable

### Downstream proof boundary

```text
crates/world
    │  returns WorldHandle { id, shared_binding }
    ▼
crates/world-agent
    │  resolve_shared_world_binding(...)
    │
    ├── reject empty orchestration_session_id
    ├── reject empty world_id
    ├── reject world_id mismatch
    ├── reject orchestration_session_id mismatch
    └── reject binding_state != Active
    ▼
shell/runtime consumers
    │
    └── trust only authoritative Active proof
```

This slice does not let later layers infer or "heal" backend proof mistakes. The backend contract
must already be correct when it crosses this boundary.

## Concrete File Touch Plan

### 1. Replacement orchestration in `crates/world/src/lib.rs`

Primary seams:

- [replace_shared_owner_session_from_root(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)
- [replace_shared_owner_session_from_root_with_creator(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)
- [cleanup_partial_shared_world_root(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)
- [ensure_shared_owner_session_from_root(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)

Required behavior:

- resolve one active shared-owner world,
- enforce `expected_generation`,
- pre-commit the old world to `Replacing`,
- create the replacement world with `generation + 1`,
- on create failure, roll back the old world and clean the partial root,
- on finalize-old-world failure, keep the replacement active and log a warning.

Must not do:

- bypass the shared-owner mutex,
- silently ignore generation mismatch,
- delete the old world on failure,
- fall back to generic compatible reuse for explicit shared-owner replacement.

### 2. Metadata persistence and recovery in `crates/world/src/session.rs`

Primary seams:

- [persist_metadata(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)
- [set_shared_binding_state(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)
- [recover_shared_active_from_root(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)

Required behavior:

- permit only `Active -> Replacing -> Active|Replaced`,
- persist every transition atomically,
- repair a lone `Replacing` world back to `Active`,
- ignore malformed metadata without deleting it,
- fail closed on multiple `Active` or multiple unreconciled `Replacing` candidates.

Must not do:

- add new binding states for this slice,
- silently auto-choose between two active candidates,
- treat ownerless legacy metadata as shared-owner compatible.

### 3. Proof validation in `crates/world-agent`

Primary seams:

- [resolve_shared_world_binding(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- PTY startup path in [crates/world-agent/src/pty.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs)

Required behavior:

- use the same helper for PTY and non-PTY shared-world proof validation,
- reject non-active or malformed proof snapshots,
- keep proof validation local and explicit instead of letting shell callers paper over a bad
  binding.

### 4. Tests and docs

Primary test anchors:

- [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs)
- [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/tests/repl_persistent_session_bootstrap_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/repl_persistent_session_bootstrap_v1.rs)

This slice does not need a doc rewrite campaign. It does need the tests to prove the contract the
docs already claim.

## Execution Plan

### Ordered implementation sequence

Implement in this order. Each phase creates one invariant the next phase can safely consume.

1. **Freeze replacement ordering in `crates/world/src/lib.rs`**
   - keep `expected_generation` enforcement hard,
   - keep pre-commit `Active -> Replacing`,
   - keep rollback-to-`Active` on create failure,
   - preserve aggregated create/rollback/cleanup error reporting,
   - preserve finalize-old-world failure as warning-only after replacement commit.

2. **Freeze metadata authority in `crates/world/src/session.rs`**
   - allow only `Active -> Replacing -> Active|Replaced`,
   - keep `persist_metadata()` rename-based and temp-file-clean,
   - keep recovery deterministic across `Active`, `Replacing`, `Replaced`, and malformed inputs,
   - prove fail-closed ambiguity handling with targeted tests.

3. **Freeze the proof boundary in `crates/world-agent`**
   - keep `resolve_shared_world_binding(...)` as the single validation helper,
   - ensure PTY and non-PTY paths both route through it,
   - reject empty, mismatched, or non-Active proof snapshots before shell-visible success.

4. **Widen regression coverage before calling the slice done**
   - land the missing world/backend tests first,
   - then land proof-boundary tests,
   - then refresh the QA handoff artifact and plan closeout text.

### Phase-by-phase acceptance gates

| Phase | Acceptance gate |
| --- | --- |
| 1. Replacement ordering | no create-failure path can leave the owner without one recoverable `Active` world |
| 2. Metadata authority | every shared-world state write is atomic and recovery never silently chooses between ambiguous candidates |
| 3. Proof boundary | PTY and non-PTY consumers reject malformed or non-Active proof before reporting success |
| 4. Regression closeout | targeted tests cover success, rollback, ambiguity, malformed proof, and temp-file hygiene |

## Architecture Review

### Locked architecture decisions

1. **Keep metadata authority in the backend.**
   - `session.json` remains the recovery authority. Shell/runtime consumers read proof, they do not invent it.

2. **Keep `Replacing` as the only publicized pre-commit checkpoint.**
   - It is the durable rollback seam. Removing it would make failure recovery more magical, not simpler.

3. **Keep replacement commit asymmetric.**
   - Create failure rolls back. Old-world finalize failure after replacement commit does not. Once the new `Active` world exists durably, that becomes the truth.

4. **Keep fail-closed recovery on ambiguity.**
   - Two `Active` candidates or two unreconciled `Replacing` candidates are contract failures, not "pick the newer one" opportunities.

5. **Keep proof validation local and explicit.**
   - `resolve_shared_world_binding(...)` stays the one boundary. PTY and non-PTY codepaths must not fork their own interpretation logic.

### Architecture acceptance gates

1. **Restart gate**
   - `ReplaceExpectedGeneration` either returns one new `Active` world or returns failure with the old world still recoverable.

2. **Recovery gate**
   - recovery returns one authoritative world, repairs one lone `Replacing` world, or fails closed. Nothing in between.

3. **Boundary gate**
   - world-agent success paths never expose `Replacing`, `Replaced`, empty ids, or mismatched owner proof as reusable.

## Code Quality Review

### Implementation guardrails

1. one replacement transaction shape in `crates/world/src/lib.rs`, not separate success and failure semantics hidden across callsites,
2. one binding-state transition helper in `SessionWorld`, not ad hoc metadata mutation,
3. one proof-validation helper in `crates/world-agent`, not PTY-specific and service-specific drift,
4. no new binding states, cache files, registry files, or shell-side authority mirrors,
5. explicit aggregated errors on the bad restart path, because debugging restart corruption from partial logs is miserable.

### Minimal-diff rules

- keep the diff inside the four production files already identified unless a test helper extraction makes the test seams materially clearer,
- prefer extending existing tests in `crates/world` and `crates/world-agent` over creating a new harness,
- keep docs and plan updates bounded to contract wording, not a broader narrative rewrite.

## Error & Rescue Registry

| Failure point | What goes wrong | Required rescue |
| --- | --- | --- |
| replacement create fails after pre-commit | old world is stuck `Replacing` and no active world remains | roll old world back to `Active`, clean partial root, return one aggregated error |
| finalize-old-world write fails after replacement commit | operator assumes replacement failed and retries incorrectly | keep replacement authoritative, log warning, recover new `Active` world |
| multiple active shared worlds exist on disk | reuse becomes nondeterministic | fail closed with explicit ambiguity error |
| malformed or partial owner metadata is reused | a corrupted or cross-owned world becomes falsely authoritative | ignore malformed metadata and keep scanning |
| downstream proof accepts `Replacing` | shell/runtime consumes a pre-commit state as reusable | reject proof unless `binding_state=Active` |
| temp file survives or old bytes are partially replaced | restart state authority becomes vulnerable to torn metadata | keep rename-based atomicity and assert cleanup paths in tests |

## Test Review

One nice thing here: the repo already has real tests. This review is not inventing coverage from
scratch. It is checking the last few holes that would still hurt at 2am.

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/world/src/lib.rs
    │
    ├── [★★★ TESTED] replace success commits new Active and old Replaced
    ├── [★★★ TESTED] create failure rolls back old world and cleans partial root
    ├── [GAP]         generation conflict returns fail-closed error
    ├── [GAP]         finalize-old-world warning path leaves replacement authoritative
    └── [GAP]         aggregated create+rollback+cleanup error text is preserved

[+] crates/world/src/session.rs
    │
    ├── [★★★ TESTED] allowed binding-state transitions persist
    ├── [★★★ TESTED] lone Replacing world repairs back to Active
    ├── [★★★ TESTED] newer Active outranks older Replacing
    ├── [★★★ TESTED] ownerless, cross-owned, and partial metadata are rejected
    ├── [★★★ TESTED] atomic persist failure preserves prior bytes
    ├── [GAP]         multiple Active candidates fail closed
    └── [GAP]         happy-path atomic persist leaves no stray temp files

[+] crates/world-agent/src/service.rs
    │
    ├── [★★★ TESTED] mismatched orchestration_session_id is rejected
    ├── [GAP]         non-Active binding_state is rejected explicitly
    └── [GAP]         empty world_id / empty orchestration_session_id are rejected explicitly

─────────────────────────────────
COVERAGE: 8/15 paths tested (53%)
QUALITY:  ★★★: 8  ★★: 0  ★: 0
GAPS: 7 paths need regression tests
─────────────────────────────────
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] ReplaceExpectedGeneration success
    │
    ├── [★★★ TESTED] new generation becomes the only reusable Active world
    └── [GAP]         finalize warning path still returns success while old world finalize logs only

[+] ReplaceExpectedGeneration create failure
    │
    ├── [★★★ TESTED] old world returns to Active
    └── [★★★ TESTED] partial replacement root is deleted

[+] Crash or restart recovery
    │
    ├── [★★★ TESTED] lone Replacing world repairs back to Active
    ├── [★★★ TESTED] newer Active beats older Replacing
    └── [GAP]         multiple Active candidates produce hard failure, not silent winner selection

[+] Agent boundary proof validation
    │
    ├── [★★★ TESTED] owner mismatch is rejected
    ├── [GAP] [→E2E] PTY and non-PTY both reject non-Active proof snapshots
    └── [GAP]         empty proof fields are rejected before shell consumers see them

[+] Metadata durability
    │
    ├── [★★★ TESTED] write failure preserves prior metadata bytes
    └── [GAP]         successful writes leave no stray temp files behind

─────────────────────────────────
COVERAGE: 6/11 flows tested (55%)
GAPS: 5 flows need tests (1 should exercise the PTY/shared-world path)
─────────────────────────────────
```

### Required test additions by file

#### `crates/world/src/lib.rs`

Add regression coverage for:

- generation-mismatch failure on `ReplaceExpectedGeneration`,
- finalize-old-world failure after replacement commit still recovering the replacement,
- create failure that also captures rollback and cleanup failure text in one returned error.

#### `crates/world/src/session.rs`

Add regression coverage for:

- multiple `Active` shared-owner candidates fail closed,
- successful `persist_metadata()` leaves no `.<session.json>.*.tmp` files in the world directory.

#### `crates/world-agent/src/service.rs`

Add unit coverage for:

- `binding_state=Replacing` and `binding_state=Replaced` rejection,
- empty `world_id`,
- empty `orchestration_session_id`.

#### `crates/world-agent/tests/repl_persistent_session_bootstrap_v1.rs`

Add one integration-style proof that the PTY startup path rejects a non-Active shared-world proof
before the ready state becomes visible to shell consumers.

### Test commands

Run at minimum:

```bash
cargo test -p world replace_success_commits_new_active_and_finalizes_old_world -- --nocapture
cargo test -p world replace_failure_rolls_back_old_world_and_cleans_partial_root -- --nocapture
cargo test -p world -- --nocapture
cargo test -p world-agent resolve_shared_world_binding_rejects_mismatched_owner_proof -- --nocapture
cargo test -p world-agent -- --nocapture
```

Then run:

```bash
cargo test -p world-api shared_world_contract_round_trips_with_canonical_shape -- --nocapture
cargo test -p agent-api-types execute_response_shared_world_round_trip -- --nocapture
```

### QA artifact

Primary QA handoff artifact:
[spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260501-124044.md](/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260501-124044.md)

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
| --- | --- | --- | --- | --- | --- |
| replacement finalize warning path | replacement commits, old-world finalize write fails, operator misreads warning as rollback need | no | yes | partial today | yes until tested |
| ambiguous multi-active recovery | two active worlds exist for one owner and reuse becomes nondeterministic | no | yes | yes | yes until tested |
| non-Active proof rejection at PTY boundary | shell consumes a `Replacing` proof as reusable | no | yes | yes | yes until tested |
| empty proof fields | malformed proof crosses world-agent boundary and poisons runtime state | no | yes | yes | yes until tested |
| happy-path temp cleanup | temp metadata files accumulate and confuse debugging or future recovery tooling | no | partial | no | no, but still worth locking down |
| create+rollback+cleanup failure aggregation | root cause detail is lost during a bad restart window | no | partial | yes | no, but debugging gets worse |

Critical gap rule:

If a non-Active proof can cross the world-agent boundary, or if ambiguity between two active
shared worlds is not fail closed, this slice is not done.

## Performance Review

This is a correctness slice, not a throughput slice.

Still, four performance rules matter:

1. the shared-owner mutex is acceptable because replacement is a restart-boundary operation, not a
   per-command hot path,
2. atomic rename plus directory sync is the right cost to pay for authority writes,
3. recovery may scan world roots because correctness beats speculative indexing here,
4. adding a cache or index file would spend an innovation token on the wrong problem.

The performance footgun would be "optimize" away the safety barriers because restart is rare. Rare
code is exactly where silent corruption likes to hide.

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| A. Backend replacement and recovery hardening | `crates/world/src/` | — |
| B. Downstream proof validation expansion | `crates/world-agent/src/`, `crates/world-agent/tests/` | A |
| C. QA artifact and plan/docs closeout | `llm-last-mile/`, `~/.gstack/projects/` | A, B |

### Parallel lanes

- Lane A: step A
- Lane B: step B
- Lane C: step C

### Execution order

1. land Lane A first because it freezes the authority contract,
2. then run Lane B once the backend proof shape is stable,
3. finish with Lane C after tests and contract wording settle.

### Conflict flags

- `crates/world/src/lib.rs` and `crates/world/src/session.rs` are one coupled seam. Keep them in
  the same lane.
- Lane B depends on the exact backend proof contract from Lane A. Do not start PTY proof tests
  early.

### Parallelization verdict

This slice is mostly sequential. There is **one foundation lane**, **one late proof-validation
lane**, and **one closeout lane**.

## Deferred Work

There is no `TODOS.md` in the repo root, so explicit deferrals stay here:

1. runtime-state projection of authoritative world binding
   - owned by [PLAN-04](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-04.md)
2. participant invalidation and replacement-member semantics
   - owned by [PLAN-05](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-05.md)
3. session-centric runtime-store regrouping
   - owned by [PLAN-06](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-06.md)
4. non-Linux explicit shared-world replacement parity
   - still intentionally fail closed outside Linux
5. richer operator-facing restart diagnostics
   - useful later, not required for the backend authority contract itself

## Definition of Done

This slice is done only when all of the following are true:

1. replacement create failure never leaves the owner without one recoverable active world,
2. old-world rollback to `Active` is durable and recoverable,
3. successful replacement commits a new `Active` world with `generation + 1`,
4. partial replacement roots are cleaned on failure,
5. atomic metadata writes protect every shared-world state transition,
6. recovery chooses one authoritative result or fails closed on ambiguity,
7. ownerless, cross-owned, and partial metadata are never adopted for shared-owner reuse,
8. world-agent proof consumers accept only `binding_state=Active`,
9. regression tests cover success, rollback, cleanup, ambiguity, malformed proof, and metadata
   temp-file hygiene,
10. later slices can consume this backend contract without re-inventing authority in shell state.

## Completion Summary

- Step 0: scope accepted as-is after repo-truth correction
- Architecture Review: 2 issues found, both are proof gaps rather than structural redesign needs
- Code Quality Review: 1 issue found, happy-path temp-file hygiene lacks explicit regression proof
- Test Review: diagrams produced, 12 concrete gaps or assertions identified
- Performance Review: 0 issues found
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0, repo has no `TODOS.md`, deferrals captured here
- Failure modes: 4 critical gaps flagged
- Outside voice: skipped
- Parallelization: 3 lanes, 0 safe production-code parallel lanes before contract freeze
- Lake Score: complete option chosen for every in-slice decision

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Scope | Treat this as a repo-truth and regression-proof slice, not a greenfield redesign | Mechanical | Pragmatic | The core backend behavior already exists and should be tightened, not reimagined | Rewriting the restart architecture |
| 2 | Authority | Keep Linux `session.json` as the replacement authority seam | Mechanical | Explicit over clever | Recovery already trusts this file and later layers already depend on its truth | Moving authority into shell-local reconstruction |
| 3 | Safety | Preserve `Replacing` as the only allowed pre-commit state | Mechanical | Completeness | It gives rollback a durable checkpoint without advertising a reusable public state | Removing `Replacing` and hoping create/finalize is atomic enough |
| 4 | Proof boundary | Require world-agent to reject any non-Active proof | Mechanical | Systems over heroes | Later shell layers should not need to compensate for backend or transport mistakes | Letting shell consumers infer reusability from partial proof |
| 5 | Tests | Spend effort on finalize-warning, ambiguity, malformed proof, and temp-file hygiene | Mechanical | Boil the lake | These are the remaining holes most likely to bite during follow-on work | Stopping at the currently landed happy-path tests |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
| --- | --- | --- | --- | --- | --- |
| CEO Review | `/plan-ceo-review` | Scope and strategy | 0 | SKIPPED | Backend correctness slice, no separate CEO pass run |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | SKIPPED | No separate Codex review run |
| Eng Review | `/plan-eng-review` | Architecture and tests (required) | 1 | CLEAR | Reframed this as a repo-truth contract, preserved the current replacement state machine, and identified the remaining regression-proof gaps around finalize warning, ambiguity, malformed proof, and temp-file hygiene |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | SKIPPED | No UI scope |

**UNRESOLVED:** 0 blocking design decisions remain inside slice `07`. The remaining work is proof widening, not scope confusion.

**VERDICT:** ENG CLEARED. `PLAN-07` is ready to execute as the backend correctness guardrail that `PLAN-04`, `PLAN-05`, and `PLAN-06` consume.
