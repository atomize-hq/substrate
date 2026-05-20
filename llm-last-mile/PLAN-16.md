# PLAN-16: Complete Backend-Id-Scoped Retained Runtime Selection And Multi-Member Coexistence

Source SOW: [16-explicit-backend-selection-by-backend-id.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/16-explicit-backend-selection-by-backend-id.md)  
Gap matrix anchor: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Adjacent slice: [17-turn-submit-reuse-for-selected-member.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/17-turn-submit-reuse-for-selected-member.md)  
Branch: `feat/session-centric-state-store`  
Base branch: `main`  
Plan type: Linux-first runtime-identity and retained-member cardinality plan, developer-facing shell/runtime behavior, cross-crate (`shell` + `world-agent` + `agent-api-types`), no UI scope  
Review posture: `/autoplan` scope discipline with `/plan-eng-review` depth, rewritten as one cohesive execution plan  
Status: execution-ready planning pass on 2026-05-05  
Outside voice: not used for this document generation

## Objective

Finish the already-landed explicit backend-targeting model so the runtime behaves like the selector contract already claims it should.

The shipped behavior is:

1. every explicit backend-targeted path resolves by exact derived `backend_id`
2. retained world-member lookup and reuse are keyed by `backend_id`, not singleton state or `agent_id` heuristics
3. multiple distinct world-scoped backends such as `cli:codex` and `cli:claude-code` can coexist as simultaneously retained live runtimes within one orchestration session and one authoritative world generation
4. duplicate retained live members for the same backend key still fail closed
5. follow-up submit validation remains exact about `participant_id`, `orchestrator_participant_id`, `world_id`, `world_generation`, and `backend_id`
6. no grammar, CLI, or product-surface redesign is smuggled into this slice

This is the whole point of the slice. Not new caller grammar. Not new submit transport. Not status-surface redesign. Just making backend identity the real ownership and reuse key all the way down.

## Plan Summary

The repo is already past the "can we target a named backend at all?" stage. The exact selector exists. The typed submit route exists. Backend identity already crosses the shell to world boundary.

What is still wrong is the ownership model behind those surfaces. The shell REPL still keeps one retained member slot. `live_member_for_generation(...)` still searches by `agent_id`. `world-agent` still rejects coexistence by enforcing "only one participant may be retained at a time." That means the current exact-backend path is real at the route edge and still partly fake underneath.

This plan fixes the identity model, not the product shape. It keeps the current targeted-turn REPL contract and typed world submit contract intact, then replaces both shell-side and world-side singleton assumptions with explicit backend-aware retained-member management. Distinct backend ids may coexist. Duplicate same-backend ownership still fails closed.

## Locked Starting State

### What already exists

The repo already has the pieces this plan must reuse:

- exact backend selection in [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
- REPL targeted-turn routing by exact backend id in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- typed world follow-up submit via `MemberTurnSubmitRequestV1` in [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- shell-to-world dispatch transport already carrying `backend_id` in [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- retained-world submit validation already checking exact `backend_id` in [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
- Linux-first targeted-turn tests already proving grammar rejection, exact routing, typed submit, and same-generation same-backend reuse in [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

### Exact remaining gap

The remaining gap is narrower than the SOW had to assume, but it is still real:

1. the shell still models retained world members as one `member_runtime: Option<AsyncReplAgentRuntime>` in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
2. shell-side live-member reuse still searches by `agent_id` plus `world_generation` in `live_member_for_generation(...)` rather than by the requested `backend_id`
3. `ensure_member_runtime_ready_for_descriptor(...)` still treats a different targeted backend as a replacement of the one global slot instead of a sibling retained runtime
4. `world-agent` still rejects coexistence in `register_member(...)` by failing whenever any retained member already exists
5. the gap matrix still truthfully describes explicit backend targeting as only partially productized because the identity model is still mixed
6. the already-landed submit path is correct only as long as there is at most one retained eligible world member

## Frozen Execution Contract

This section removes the wiggle room. If implementation wants to do something else, this plan is wrong and should be revised first.

### Non-negotiable invariants

1. `backend_id` is the canonical selector for every explicit backend-targeted path.
2. Exact backend selection must fail closed. No fallback to another backend. No fallback to singleton-member heuristics.
3. `execution.scope` remains part of the selector contract. Host and world cannot silently cross-match.
4. Retained world-member reuse is by backend identity inside one orchestration session and one authoritative world generation.
5. Distinct backend ids may coexist as retained live world members.
6. Duplicate retained live members for the same backend key in the same session and generation are an error.
7. `MemberTurnSubmitRequestV1` stays the submit contract. This slice does not invent a second submit route.
8. The current REPL targeted-turn grammar stays unchanged.
9. `substrate -c` stays wrap mode.
10. Linux-first world-member posture stays unchanged.

### Retained-member identity contract

The canonical retained-member key for this slice is:

```text
orchestration_session_id + world_generation + backend_id
```

That is the operator-visible and runtime-honest identity. `participant_id` is still the concrete retained instance identifier. It is not the reusable selection key.

Consequences:

1. shell-side readiness lookup must answer "do we already retain `cli:codex` for generation 7?"
2. world-side duplicate detection must answer "do we already retain one live runtime for `cli:codex` in this same session and generation?"
3. follow-up submit must still target the retained participant that owns that backend key

### Exact-selection contract

`validate_exact_backend_selection(...)` in [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) stays authoritative for explicit backend-targeted paths.

This plan does not replace it. It widens the rest of the runtime to match it.

The selector contract must distinguish these failures:

- backend missing from effective inventory
- backend present in inventory but wrong scope for the caller
- backend present but disallowed by policy
- backend present but not runtime-realizable
- backend duplicated across effective entries in the same scope

### Cardinality contract

This slice deliberately changes cardinality for world-scoped retained members.

Required behavior:

1. one orchestration session may retain more than one live world member at the same time
2. those retained live members may have different backend ids
3. the shell may reuse any retained member whose backend id exactly matches the requested backend and whose world generation is still authoritative
4. stale-generation members are still invalid and must not be silently reused
5. same-backend duplicates still fail closed

### Transport contract

The transport identity is already correct. Keep it that way.

`backend_id` must remain explicit in:

- member dispatch from shell to `world-agent`
- retained runtime state in `world-agent`
- world follow-up submit requests
- world follow-up submit validation
- emitted events and translated REPL output

### Product-boundary contract

This slice is not allowed to widen its surface area by convenience.

Rejected expansions:

- new caller grammar
- implicit default-backend routing
- new non-interactive agent caller syntax
- broader `substrate agent start|resume|fork|stop` productization
- toolbox or status redesign
- macOS parity

## Step 0: Scope Challenge

### 0A. What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| exact backend selection | [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) | Reuse. Do not invent a second explicit selector. |
| targeted-turn route already choosing host vs world by backend id | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse. This plan hardens what happens after route selection. |
| typed world follow-up submit contract | [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs) and [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs) | Reuse. No second submit transport. |
| retained submit validation against exact backend/world identity | [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs) | Reuse. Keep strict validation. |
| same-generation same-backend reuse proof | [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) | Extend. Add coexistence and duplicate-detection cases rather than replacing the harness. |
| shell and world invalidation on generation rollover | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse. Widen from one retained member to all retained members. |

### 0B. Minimum honest diff

The minimum honest implementation is:

1. harden every explicit backend-targeted selection path around `validate_exact_backend_selection(...)`
2. replace shell-side singleton retained-member state with a backend-aware collection
3. replace shell-side `agent_id` reuse lookup with backend-aware lookup
4. widen `world-agent` retained-member ownership from global singleton to multi-member coexistence
5. preserve strict duplicate detection for the same backend key
6. extend tests and the gap matrix so repo truth matches code truth

Anything smaller is fake progress.

Rejected shortcuts:

- keep one shell `Option` and silently stop/start whenever a second backend is targeted
- keep global singleton retention in `world-agent` and claim coexistence is "future work"
- keep `agent_id` as the reuse key because it is "close enough"
- add a backend aliasing layer
- reopen caller grammar or submit transport just because those files are already nearby

### 0C. Complexity check

This slice touches more than 8 files. That is still the minimal honest blast radius because the seam crosses selector hardening, shell retained-state ownership, world retained-state ownership, and integration proof.

Primary files expected to move:

1. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
2. [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
3. [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
4. [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
5. [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
6. [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
7. [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)
8. [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
9. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)

### 0D. Search and completeness check

Search-before-building result, in practical terms:

- **[Layer 1]** reuse `validate_exact_backend_selection(...)`
- **[Layer 1]** reuse `MemberTurnSubmitRequestV1` and `/v1/member_turn/stream`
- **[Layer 1]** reuse current policy allowlist and runtime-realizability checks
- **[Layer 1]** reuse current world-generation invalidation rules
- **[EUREKA]** this is not a transport problem anymore. It is an identity-and-cardinality problem.
- **[EUREKA]** `participant_id` is the retained instance handle, not the correct reuse selector. The public contract is `backend_id`, so reuse must align to `backend_id`.

Completeness rule for this plan:

- support simultaneous retained live members for distinct backend ids
- preserve fail-closed behavior for duplicate same-backend retention
- preserve strict submit validation against exact backend identity
- keep prior grammar and submit regressions green

### 0E. Distribution and runtime contract check

No new artifact type is introduced. This is not packaging work.

The ship surface is behavioral:

- exact backend selection semantics
- retained-member coexistence and reuse
- fail-closed duplicate detection
- truthful tests and docs

### 0F. NOT in scope

- new REPL caller grammar
- `substrate -c` redesign
- non-interactive agent caller surface
- public `substrate agent start|resume|fork|stop`
- macOS/Lima parity
- transcript or status-surface redesign
- broader selected-member submit/reuse product polish outside this backend-identity slice

## Architecture Review

### Locked architecture decisions

1. `validate_exact_backend_selection(...)` remains the canonical explicit selector.
2. Shell retained-member state becomes backend-aware, not participant-list guessing and not one global slot.
3. `world-agent` retained-member state allows coexistence of distinct backend ids.
4. Duplicate same-backend retention remains fail closed.
5. `MemberTurnSubmitRequestV1` remains the world submit contract.
6. World-generation invalidation still gates reuse.

### Architecture findings resolved in-plan

**Issue 1. The shell singleton is the main correctness bug now.**

The exact selector can already name `cli:codex` vs `cli:claude-code`. The shell cannot honor that honestly while it keeps one `member_runtime` slot. This plan replaces the slot with backend-aware retained-member state inside the REPL session.

**Issue 2. `agent_id` is the wrong reuse key for an operator-facing backend selector.**

The operator targets a backend id. The transport carries a backend id. The submit validator checks a backend id. Reuse by `agent_id` in the middle is an identity leak. This plan removes that mismatch.

**Issue 3. Shell and `world-agent` cardinality must widen together.**

If the shell can retain two backends but `world-agent` still rejects the second retained member, the feature is fake. If `world-agent` allows two but the shell keeps overwriting one slot, the feature is still fake. Both sides move in the same slice.

**Issue 4. Coexistence must not weaken duplicate detection.**

Allowing `cli:codex` and `cli:claude-code` together is correct. Allowing two live `cli:codex` retained members for the same session and generation is not. The new cardinality must be explicit about that difference.

**Issue 5. This slice does not need a new generic runtime registry abstraction.**

The runtime counts are small and the semantics are specific. A blunt, explicit backend-aware collection plus tight helper functions is better than an abstraction project.

### Architecture ASCII diagrams

### Exact backend selection to retained reuse

```text
operator targets backend_id
    |
    v
validate_exact_backend_selection(...)
    |
    +--> missing / wrong-scope / denied / unrealizable / duplicate
    |        |
    |        +--> fail closed with exact reason
    |
    v
descriptor { backend_id, scope, binary_path, ... }
    |
    v
scope == world ?
    |
    +--> no --> existing host path
    |
    v
retained lookup by:
orchestration_session_id + world_generation + backend_id
    |
    +--> retained exact match exists ----------> reuse that retained runtime
    |
    +--> none exists --------------------------> launch one retained runtime for that backend
    |
    +--> more than one exact match exists -----> fail closed
```

### Shell retained-member ownership model

```text
REPL session
    |
    +-- authoritative world binding
    |      world_id
    |      world_generation
    |
    +-- retained world members by backend_id
           "cli:codex"        -> AsyncReplAgentRuntime
           "cli:claude-code"  -> AsyncReplAgentRuntime
           ...
    |
    +-- world generation rollover
           |
           +--> invalidate every retained member for old generation
           +--> persist invalidated manifests
           +--> remove old backend entries from the active retained map
```

### `world-agent` retained-member ownership model

```text
MemberRuntimeManager
    |
    +-- active_members_by_participant_id
    |      participant_id -> retained runtime ownership
    |
    +-- exact backend query helper
    |      (orchestration_session_id, world_generation, backend_id)
    |             |
    |             +--> 0 matches  -> not retained
    |             +--> 1 match    -> exact retained runtime
    |             +--> 2+ matches -> fail closed
    |
    +-- active_turns_by_span_id
           submit span -> cancel handle + participant association
```

## Code Quality Review

### Findings resolved in-plan

**Issue 1. Keep the new shell state explicit.**

Use an obvious backend-aware retained-member collection. No generic registry framework. No trait hierarchy. The runtime already has enough moving parts.

**Issue 2. Centralize retained-member keying.**

Add one small helper seam for:

- exact retained lookup by backend id
- duplicate detection
- removal and invalidation on world-generation rollover

Do not copy this logic into targeted-turn reuse, startup preparation, and teardown separately.

**Issue 3. Keep selector logic in `validator.rs`, not in REPL routing code.**

If backend-selection error taxonomy starts drifting between `validator.rs` and `async_repl.rs`, the product becomes untestable. The REPL should consume exact selection, not reinterpret it.

**Issue 4. Update nearby diagrams or comments if they become stale.**

This repo already benefits from textual diagrams in planning docs. If touched code comments or nearby documentation imply a singleton retained-member model after this slice lands, they must be updated in the same change. Stale architecture comments are worse than none.

### Allowed code shape

1. Keep explicit backend-selection rules in [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs).
2. Keep shell retained-member orchestration in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).
3. Keep `world-agent` retained-member ownership logic in [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs).
4. Add only the smallest helper seams needed for backend-aware lookup and duplicate detection.
5. Do not add a new persistence layer, background reconciler, or secondary daemon.

## Test Review

### Test framework detection

This repo is Rust-first. The test surface for this plan is `cargo test` across shell unit tests, shell integration tests, and world-agent unit/integration tests.

Relevant existing suites already exist in:

- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/shell/src/execution/agent_runtime/validator.rs
    |
    ├── validate_exact_backend_selection(...)
    │   ├── [★★★ TESTED] Exact backend match returns descriptor
    │   ├── [★★★ TESTED] Duplicate exact backend entries fail closed
    │   ├── [GAP]         Wrong-scope error is distinct from "missing backend"
    │   ├── [GAP]         Policy deny stays exact when explicit backend is otherwise valid
    │   └── [GAP]         Runtime-unrealizable backend stays exact and does not fall back
    |
    └── validate_member_selection(...)
        └── [GAP] Legacy/non-targeted callers are audited so explicit backend paths do not regress to singleton heuristics

[+] crates/shell/src/repl/async_repl.rs
    |
    ├── retained world-member state
    │   ├── [★★★ TESTED] Same-generation same-backend reuse keeps one retained member
    │   ├── [GAP]         Same-generation different backend can coexist without tearing down the first retained member
    │   ├── [GAP]         Exact backend lookup reuses the correct retained runtime when two backends coexist
    │   ├── [GAP]         Duplicate retained runtime for the same backend key fails closed
    │   └── [GAP]         World-generation rollover invalidates every retained backend, not just one slot
    |
    ├── live_member_for_generation / exact retained lookup helper
    │   ├── [GAP]         Backend-aware lookup returns exact match for cli:codex
    │   ├── [GAP]         Backend-aware lookup returns exact match for cli:claude-code
    │   └── [GAP]         Same-agent-family / different-backend coexistence does not collide through agent_id
    |
    └── ensure_member_runtime_ready_for_descriptor(...)
        ├── [★★★ TESTED] Same-generation same-backend reuse does not relaunch
        ├── [GAP]         Different backend launches a second retained member instead of replacing the first
        └── [GAP]         Duplicate same-backend retained state is rejected before submit

[+] crates/world-agent/src/member_runtime.rs
    |
    ├── register_member(...)
    │   ├── [GAP]         Distinct backend ids can coexist for one session and generation
    │   ├── [GAP]         Same backend id duplicate fails closed
    │   └── [GAP]         Cleanup removes only the exact retained participant that exited
    |
    ├── exact retained-member lookup
    │   ├── [GAP]         Resolve cli:codex among simultaneous retained members
    │   ├── [GAP]         Resolve cli:claude-code among simultaneous retained members
    │   └── [GAP]         Multiple exact matches fail closed
    |
    └── submit_turn(...)
        ├── [★★★ TESTED] Backend mismatch is rejected
        ├── [★★★ TESTED] Missing surfaced session id is rejected
        ├── [★★★ TESTED] Concurrent submitted turn collision is rejected per participant
        └── [GAP]         Submit succeeds for one retained backend while another retained backend remains live and untouched

─────────────────────────────────
COVERAGE: some exact-routing and submit-identity paths are already tested, but coexistence coverage is still missing
QUALITY TARGET: all new coexistence and duplicate-detection paths reach at least ★★, all regressions and fail-closed paths reach ★★★
BIGGEST GAP: simultaneous retained distinct backends in one session/generation
─────────────────────────────────
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] Operator targets one backend twice
    |
    ├── [★★★ TESTED] "::cli:codex second" reuses retained cli:codex for same generation
    └── [GAP]         Same proof for cli:claude-code

[+] Operator targets two different backends in one live session
    |
    ├── [GAP] [→E2E] Launch cli:codex, then cli:claude-code, both remain retained
    ├── [GAP] [→E2E] Later target cli:codex again, exact codex runtime is reused
    └── [GAP] [→E2E] Later target cli:claude-code again, exact claude-code runtime is reused

[+] Operator error states
    |
    ├── [GAP]         Duplicate exact backend inventory entries fail with exact backend-specific error
    ├── [GAP]         Duplicate retained same-backend runtime fails closed instead of choosing one
    ├── [GAP]         Wrong-scope exact backend request remains exact and does not masquerade as missing
    └── [★★★ TESTED] Backend mismatch on submit is rejected

[+] Regression floor
    |
    ├── [★★★ TESTED] Exact targeted-turn grammar remains intact
    ├── [★★★ TESTED] Typed world submit route remains intact
    ├── [★★★ TESTED] Host non-active backend rejection remains intact
    └── [GAP]         Coexistence work must not regress the current same-generation single-backend reuse proof
```

### Required tests to add or extend

1. Extend unit tests in [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) to prove:
   - explicit wrong-scope errors are distinguishable from "backend missing"
   - policy-denied exact backend selection still reports the named backend
   - runtime-unrealizable exact backend selection does not fall back to a sibling backend
2. Extend [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) to prove:
   - `cli:codex` and `cli:claude-code` can both be retained in one orchestration session and one generation
   - targeting `cli:codex` after `cli:claude-code` reuses codex instead of replacing claude-code
   - targeting `cli:claude-code` after `cli:codex` reuses claude-code instead of replacing codex
   - duplicate same-backend retained state fails closed
3. Extend [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs) so the stub can model two retained backend ids at once and record exact submit requests separately.
4. Extend unit tests in [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs) to prove:
   - `register_member(...)` accepts distinct backend ids
   - duplicate same-backend retention fails closed
   - exact retained lookup fails closed when duplicates exist
5. Extend [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs) to prove one retained backend can service a submitted turn while another retained backend remains live and unaffected.
6. Keep the existing SOW 15 regression floor green. This slice is not allowed to break:
   - exact targeted-turn grammar
   - typed world submit route
   - host non-active backend rejection
   - same-generation same-backend reuse

### Regression rule for this slice

This slice changes retained-member ownership semantics. That makes these regressions mandatory:

1. same-generation same-backend reuse must keep passing
2. exact targeted-turn routing must keep passing
3. typed world submit validation must keep passing
4. host follow-up rejection for non-active backend must keep passing

No AskUserQuestion needed here. These are hard requirements.

## Failure Modes Registry

| Failure mode | Surface | Test coverage required | Handling required | User-visible outcome |
| --- | --- | --- | --- | --- |
| exact backend missing from inventory | selector | yes | fail closed | exact missing-backend error |
| exact backend exists only in wrong scope | selector | yes | fail closed | exact wrong-scope error |
| exact backend denied by policy | selector | yes | fail closed | exact deny naming backend id |
| exact backend unrealizable | selector | yes | fail closed | exact runtime-unavailable error |
| duplicate exact backend entries in effective inventory | selector | yes | fail closed | exact duplicate-inventory error |
| shell retained lookup finds none for targeted backend | shell reuse | yes | launch exact backend, not sibling backend | normal backend-specific launch |
| shell retained lookup finds duplicate same-backend live members | shell reuse | yes | fail closed | duplicate-retained-runtime error |
| world-agent retained registry already has distinct other backend | world registry | yes | allow coexistence | no operator-visible error |
| world-agent retained registry gets second same-backend retained member | world registry | yes | fail closed | duplicate-retained-runtime error |
| submit request backend id mismatches retained runtime | world submit | yes | fail closed | backend mismatch error |
| world-generation rollover leaves stale retained backend live | shell/world cleanup | yes | invalidate and remove stale retained entries | stale-generation recovery path |
| coexistence change accidentally tears down the wrong backend | shell/world teardown | yes | exact backend-scoped cleanup | operator keeps other backend live |

Critical gap rule for this plan:

no same-backend duplicate path is allowed to be both untested and auto-resolved. Duplicate same-backend ownership must always fail closed.

## Performance Review

This slice increases retained-member cardinality. That matters, but it is still human-paced control-plane work, not high-throughput backend traffic.

### Findings resolved in-plan

1. Retaining two live world members instead of one increases steady-state memory and process count. That is acceptable because the number of eligible retained backends is tiny and explicitly configured.
2. Duplicate detection can stay simple. A small scan across retained members for one session/generation is cheaper and safer than inventing a complex cache or secondary index too early.
3. World-generation rollover must clean up all stale retained members. If cleanup misses one, the problem is correctness first and resource leakage second.

### Performance posture

- no new N+1 style data-access concern exists
- no caching layer is needed
- no background polling loop is needed
- the cardinality increase is bounded by configured world-scoped backends and one authoritative world generation
- correctness wins over micro-optimizing lookup structures here

## DX Guardrails

This is developer-facing runtime behavior even though it is backend code.

Required operator experience:

1. exact backend failures must still name the backend id the operator requested
2. coexistence must be invisible when it works and explicit when it fails
3. duplicate same-backend retained runtime failures must say that the state is ambiguous, not just "submit failed"
4. world-generation rollover failures must remain explicit about stale retained state
5. submit errors must preserve backend identity in translated REPL output

Small details. Real product impact.

## Worktree Parallelization Strategy

This plan has real parallelization opportunities once the retained-member identity contract is frozen.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Freeze retained-member identity contract and exact error taxonomy | `crates/shell/src/execution/agent_runtime/`, `crates/shell/src/repl/`, docs | — |
| Shell retained-member collection + backend-aware reuse | `crates/shell/src/repl/` | Freeze retained-member identity contract and exact error taxonomy |
| `world-agent` coexistence + duplicate detection | `crates/world-agent/src/` | Freeze retained-member identity contract and exact error taxonomy |
| Integration tests + gap-matrix closeout | `crates/shell/tests/`, `crates/world-agent/tests/`, repo docs | Shell retained-member collection + backend-aware reuse, `world-agent` coexistence + duplicate detection |

### Parallel lanes

- Lane A: Shell retained-member collection + backend-aware reuse
  - sequential inside the lane because these steps share `crates/shell/src/repl/`
- Lane B: `world-agent` coexistence + duplicate detection
  - sequential inside the lane because these steps share `crates/world-agent/src/`
- Lane C: Integration tests + gap-matrix closeout
  - starts after A and B because the test stub and docs need the final retained-member contract

### Execution order

1. Freeze the retained-member identity contract and exact failure taxonomy.
2. Launch Lane A and Lane B in parallel worktrees.
3. Merge A and B.
4. Run Lane C for coexistence proof, duplicate-fail-closed proof, and gap-matrix truth updates.

### Conflict flags

- Lane A and Lane C both touch [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs). Keep final ownership with Lane C after shell behavior lands.
- Lane B and Lane C both touch [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs). Same rule.
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs) is the cardinality hotspot. Only one lane edits it at a time.

### Parallelization verdict

Three workstreams, two parallel implementation lanes, one final integration lane.

## Implementation Sequence

### Step 1. Freeze the retained-member identity contract

Files:

- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)

Deliver:

1. document the exact failure taxonomy for explicit backend-targeted selection
2. audit explicit backend-targeted callers so they all use `validate_exact_backend_selection(...)`
3. freeze the retained-member key as `orchestration_session_id + world_generation + backend_id`

Done means the selector and identity rules are explicit before cardinality changes land.

### Step 2. Replace shell-side singleton retained-member state

Files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Deliver:

1. replace `member_runtime: Option<AsyncReplAgentRuntime>` with a backend-aware retained-member collection
2. replace `live_member_for_generation(...)` with exact retained lookup by backend id
3. update `ensure_member_runtime_ready_for_descriptor(...)` so:
   - exact retained backend match is reused
   - missing exact retained backend is launched
   - duplicate same-backend retained entries fail closed
4. update world-generation rollover invalidation so every retained backend for the old generation is invalidated and removed

Done means the shell can honestly retain more than one live world member at once.

### Step 3. Widen `world-agent` retained-member ownership

Files:

- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)

Deliver:

1. remove the global "only one participant may be retained at a time" guard
2. add exact retained-member lookup by session/generation/backend id
3. allow distinct backend ids to coexist
4. reject duplicate same-backend retained members for the same session and generation
5. preserve current submit validation and submitted-turn collision behavior

Done means `world-agent` can retain `cli:codex` and `cli:claude-code` at the same time without weakening exact submit validation.

### Step 4. Tighten exact failure handling in adjacent explicit paths

Files:

- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Deliver:

1. preserve exact backend-specific errors for missing, wrong-scope, denied, unrealizable, and duplicate cases
2. ensure coexistence changes do not accidentally reintroduce legacy ambiguity-based selection for explicit backend paths
3. keep targeted submit and reuse paths keyed to exact backend identity end to end

Done means coexistence lands without softening the selector contract.

### Step 5. Close the coexistence proof and repo truth

Files:

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)

Deliver:

1. coexistence proof for `cli:codex` and `cli:claude-code`
2. exact reuse proof for each retained backend after both are live
3. duplicate same-backend fail-closed proof
4. unchanged current same-backend reuse proof
5. gap-matrix update from "REPL-first exact targeting with mixed identity model" to "backend-id-scoped retained coexistence landed on Linux-first runtime path"

Done means the docs tell the truth and the coexistence behavior is proven, not implied.

## Recommended Verification Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell validate_exact_backend_selection -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p world-agent member_runtime -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
```

## Definition of Done

1. Every explicit backend-targeted selector path still resolves by exact derived `backend_id`.
2. Wrong-scope, denied, unrealizable, duplicate, and missing backend failures remain exact and fail closed.
3. The shell can retain more than one live world-scoped member at once.
4. Retained shell reuse is by exact backend id, not by `agent_id`.
5. `world-agent` can retain distinct backend ids simultaneously in one session and generation.
6. `world-agent` still rejects duplicate same-backend retained members.
7. Exact targeted follow-up turns reuse the correct retained backend after coexistence is established.
8. World-generation rollover invalidates all stale retained backend entries.
9. Existing targeted-turn grammar and submit regressions remain green.
10. The gap matrix is updated to match shipped reality.

## Deferred Work

- non-REPL caller surface
- `substrate -c` redesign
- public `substrate agent start|resume|fork|stop`
- broader selected-member submit/reuse polish beyond this identity slice
- macOS/Lima parity
- status/toolbox work

## Completion Summary

- Step 0: Scope Challenge, scope accepted as-is
- Architecture Review: 5 issues found, all resolved in-plan
- Code Quality Review: 4 issues found, all resolved in-plan
- Test Review: diagram produced, coexistence and duplicate-detection gaps identified
- Performance Review: 3 issues found, all resolved in-plan
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 items proposed, deferred work stays inside this plan
- Failure modes: 0 acceptable silent duplicate-resolution gaps, 0 unresolved critical gaps after planned coverage lands
- Outside voice: skipped for this document generation
- Parallelization: 3 lanes, 2 parallel / 1 sequential integration lane
- Lake Score: 8/8 recommendations chose the complete option

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Step 0 | Treat this as an identity-and-cardinality slice, not a transport slice | Mechanical | Pragmatic | The submit route and exact selector already exist; the bug is the retained ownership model | reopening submit transport |
| 2 | Architecture | Keep `validate_exact_backend_selection(...)` as the canonical explicit selector | Mechanical | DRY | One selector contract is safer than duplicating backend-id logic in the REPL | bespoke REPL-only exact selection |
| 3 | Architecture | Key retained world-member reuse by `orchestration_session_id + world_generation + backend_id` | Mechanical | Explicit over clever | This matches the public selector and the authoritative world-binding model | reuse by `agent_id`, participant-id-only selection |
| 4 | Architecture | Replace shell singleton retained-member state with a backend-aware collection | Mechanical | Completeness | The shell cannot truthfully support exact backend coexistence with one global slot | stop-and-replace hidden singleton behavior |
| 5 | Architecture | Allow distinct backend ids to coexist in `world-agent` while rejecting duplicate same-backend retention | Mechanical | Systems over heroes | Coexistence is required, ambiguity is not | global singleton retention, permissive duplicate selection |
| 6 | Code Quality | Keep new helper seams small and local instead of introducing a generalized registry abstraction | Taste, resolved | Minimal diff | The runtime counts are tiny and the semantics are narrow | generic runtime registry framework |
| 7 | Test Review | Make coexistence proof and duplicate fail-closed proof mandatory | Mechanical | Completeness | This slice is not real until both behaviors are exercised end to end | relying on current single-backend reuse tests |
| 8 | Parallelization | Freeze identity rules first, then run shell and world cardinality work in parallel | Mechanical | Pragmatic | The shared contract is small but central, and it reduces merge churn across the two main lanes | parallel edits before key semantics are frozen |
