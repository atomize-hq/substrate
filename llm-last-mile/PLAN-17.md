# PLAN-17: Unify Selected-Member Follow-Up Submit And Reuse Semantics

Source SOW: [17-turn-submit-reuse-for-selected-member.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/17-turn-submit-reuse-for-selected-member.md)  
Gap matrix anchor: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Adjacent landed slices: [PLAN-15.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-15.md), [PLAN-16.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-16.md)  
Branch: `feat/session-centric-state-store`  
Base branch: `main`  
Plan type: Linux-first REPL-targeted follow-up submit/reuse hardening, developer-facing shell/runtime behavior, no new caller surface  
Review posture: `/autoplan` scope discipline with `/plan-eng-review` structure, rewritten as one cohesive execution plan  
Status: execution-ready planning pass on 2026-05-05  
Outside voice: not used for this document generation

## Objective

Treat the current Linux-first targeted-turn path as shipped, then finish the part that still reads like implementation scatter instead of a deliberate product contract.

This slice locks one explicit internal contract for selected-backend follow-up turns:

1. exact `backend_id` targeting stays the only targeted REPL grammar
2. host follow-up turns resume only the active orchestrator backend
3. Linux world follow-up turns reuse the exact retained selected member when it is still valid
4. relaunch happens only when retained world runtime state is missing or stale for the current world generation
5. operator-visible failures stay fail-closed, precise, and documented
6. no new non-REPL caller surface is introduced in this slice

## Plan Summary

The repo is no longer missing targeted-turn submit or backend-aware retained-member reuse. That work already landed in [PLAN-15.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-15.md) and [PLAN-16.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-16.md).

What is still missing is cohesion. The selected-member contract is spread across parser logic, route selection, readiness checks, submit functions, retained-member validation, and a few integration tests. The system works, but the contract still reads like code archaeology.

This slice does not add a new runtime capability. It makes the already-landed capability explicit, centralized, and regression-proof:

1. centralize targeted follow-up dispatch behind one internal helper seam in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
2. keep the host path shell-owned and session-resume based
3. keep the world path `world-agent`-owned and `MemberTurnSubmitRequestV1`-based
4. make reuse-vs-relaunch semantics deterministic and testable
5. tighten docs and operator-facing error truth so the repo says exactly what the runtime does

## Locked Starting State

### What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| Exact backend selection by canonical `backend_id` | [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) | Reuse as the only explicit selector. Do not duplicate selection logic in REPL routing. |
| Exact targeted-turn grammar and route-before-shell behavior | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Keep grammar unchanged. Harden behavior after parse, not the parse itself. |
| Host follow-up submit via UAA session resume | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse. No new transport. |
| Linux world follow-up submit via typed member-turn route | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs), [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs), [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs) | Reuse. Do not invent a second world submit path. |
| Backend-id-scoped retained-member coexistence and duplicate fail-closed behavior | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs), [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs) | Treat as landed baseline from PLAN-16, not work to redo. |
| Integration floor for grammar rejection, world submit reuse, and host mismatch rejection | [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) | Extend. Do not replace the harness. |
| Single-live-session status/toolbox preflight outside the REPL path | [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs), [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | Explicitly out of scope for this slice. Document the constraint, do not widen into status redesign. |

### Exact remaining gap

The remaining gap is narrower than the SOW had to assume:

1. the targeted-turn contract is still split inline across parse, route, readiness, and submit branches in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
2. host and world targeted-turn paths are behaviorally aligned, but they do not yet share one explicit orchestration seam that prevents drift in error mapping and operator-facing semantics
3. the positive host follow-up submit path is under-proved compared to the world path and the host rejection path
4. reuse-vs-relaunch semantics are correct in code, but not yet documented as one contract with one decision table
5. repo docs still need one fresh statement of what is shipped, what is Linux-only, and what is intentionally deferred

## Frozen Execution Contract

If implementation wants to do something else, revise this plan first. Do not freestyle past these rules.

### Non-negotiable invariants

1. `::<backend_id> <prompt>` remains the only targeted REPL follow-up grammar.
2. Plain REPL input remains shell execution, not implicit agent routing.
3. `validate_exact_backend_selection(...)` remains the canonical selector for explicit targeted-turn routing.
4. Host follow-up turns may target only the active orchestrator backend for the current REPL session.
5. Linux world follow-up turns must submit through `MemberTurnSubmitRequestV1` and `/v1/member_turn/stream`.
6. World-member reuse is by exact retained backend slot for the current orchestration session and current authoritative `world_generation`.
7. Relaunch is allowed only when the retained world runtime is missing, invalidated, or stale for the current `world_generation`.
8. Duplicate retained members for the same backend slot remain fail closed.
9. Non-Linux world follow-up turns remain explicit fail-closed behavior.
10. `substrate -c` remains shell wrap mode. No caller-surface widening in this slice.
11. `agent status` and toolbox ambiguity handling remain out of scope.

### Submit/reuse decision table

| Requested path | Preconditions | Required action | Explicitly disallowed fallback |
| --- | --- | --- | --- |
| Host targeted follow-up | exact host backend match, active orchestrator runtime retained, requested backend equals active orchestrator backend | call `submit_host_targeted_turn(...)` with session-resume extension | launching a second host runtime, retargeting to another host backend, shell fallback |
| Host targeted follow-up with wrong backend | exact host backend match, but requested backend is not the active orchestrator backend | fail closed with exact active-backend mismatch error | silent retarget, shell fallback, world-member launch |
| World targeted follow-up on Linux with valid retained member | exact world backend match, retained exact backend runtime authoritative-live for current generation | call `submit_world_targeted_turn(...)` against retained participant | relaunching a sibling member, switching backend slot, shell fallback |
| World targeted follow-up on Linux with missing or stale retained member | exact world backend match, but no authoritative-live retained runtime for current generation | call `ensure_member_runtime_ready_for_descriptor(...)`, then submit into the newly ready exact backend runtime | submit into stale participant, pick another retained member, shell fallback |
| World targeted follow-up on non-Linux | exact world backend match, but platform path does not support world-member reuse | fail closed with Linux-only error | pretending success, host submit, shell fallback |

### Failure taxonomy that must be preserved

| Failure class | Source of truth | Required behavior |
| --- | --- | --- |
| malformed targeted syntax | REPL parser in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | reject before shell fallback with exact accepted format |
| ambiguous exact backend entry | `validate_exact_backend_selection(...)` | reject with exact backend id and scope |
| wrong active host backend | host-targeted route gate in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | reject with expected and actual backend ids |
| no active orchestrator runtime | host-targeted dispatch seam | reject clearly, do not fabricate a new host runtime |
| world runtime unavailable after ready path | world-targeted dispatch seam | reject with named backend and platform/runtime context |
| retained identity drift | `validate_submit_turn_request(...)` in [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs) | reject with exact mismatched field |
| duplicate retained same-backend slot | retained-key validation in [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs) | reject fail closed, never choose one arbitrarily |
| concurrent submitted turn collision | submitted-turn slot reservation in [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs) | reject with active submitted-turn context, do not queue implicitly |

## Step 0: Scope Challenge

### 0A. What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| Exact selector contract | [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) | Reuse as-is. Do not move selector logic into the REPL helper. |
| Host-vs-world route classification | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Centralize the branch behind one helper seam. Do not change the operator grammar. |
| World readiness and retained-member validation | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs), [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs) | Reuse. Do not reopen retained-member identity or cardinality in this slice. |
| Host follow-up submit transport | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse session-resume submit. |
| World follow-up submit transport | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs), [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs) | Reuse typed submit. |
| Existing integration harness | [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs), [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs) | Extend. Keep one integration story. |

### 0B. Minimum honest diff

The minimum honest implementation is smaller than the original draft implied:

1. add one explicit targeted-follow-up dispatch seam in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
2. preserve `submit_host_targeted_turn(...)` and `submit_world_targeted_turn(...)` as separate transport implementations
3. extend shell integration tests to prove the positive host path, stale-world relaunch path, and explicit fail-closed cases
4. add selector-level regression tests only where the new dispatch seam depends on behavior that is not already pinned
5. update repo-truth docs so the shipped contract is stated plainly

Anything broader is scope creep. Anything smaller leaves the contract implicit.

### 0C. Complexity check

This plan does not trigger the overbuild smell if executed correctly.

Expected touched files:

1. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
2. [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
3. [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs) if new stub scripting is required
4. [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) for selector regression coverage only
5. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
6. [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md) if this packet index is used as current repo truth

Rejected expansions:

1. [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs) unless a testable operator-facing defect is discovered that cannot be pinned from the shell side
2. [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
3. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
4. non-REPL caller surfaces
5. macOS/Lima parity work
6. new public control-plane commands

### 0D. Search and completeness check

Search-before-building result, in practical terms:

- **[Layer 1]** reuse `validate_exact_backend_selection(...)`
- **[Layer 1]** reuse `ensure_member_runtime_ready_for_descriptor(...)`
- **[Layer 1]** reuse `submit_host_targeted_turn(...)`
- **[Layer 1]** reuse `submit_world_targeted_turn(...)`
- **[Layer 1]** reuse `MemberTurnSubmitRequestV1`
- **[EUREKA]** this is no longer a transport problem or a retained-key problem. It is a contract-centralization and regression-floor problem.
- **[EUREKA]** the cheapest complete version is not more runtime capability. It is one dispatch seam plus the missing regression proofs.

### 0E. Distribution and runtime contract check

No new artifact type is introduced.

Distribution is not the missing piece here. The missing deliverable is behavioral truth:

1. one obvious shell seam for targeted follow-up dispatch
2. one explicit operator contract for reuse versus relaunch
3. one test floor that proves the contract instead of implying it
4. one doc truth that says Linux-first and REPL-first without hedging

### 0F. NOT in scope

- `substrate -c` redesign
- non-interactive prompt submission surface
- default-agent routing for plain REPL input
- `substrate agent start|resume|fork|stop`
- status/toolbox ambiguity redesign
- macOS/Lima parity
- new world-agent transport families
- redoing PLAN-16 retained-member cardinality work

## Architecture Review

### Locked architecture decisions

1. Add one internal targeted-turn orchestration seam in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs). The name is flexible. The behavior is not.
2. That seam owns exact backend selection, route classification, readiness, and operator-visible error mapping.
3. The seam does not replace `submit_host_targeted_turn(...)` or `submit_world_targeted_turn(...)`. It chooses between them.
4. Host follow-up submit remains shell-owned and session-resume based.
5. World follow-up submit remains `world-agent`-owned and typed-request based.
6. Reuse-vs-relaunch logic for world members remains inside the existing readiness path, not in the submit function.

### Architecture findings resolved in-plan

**Issue 1. The contract is correct but scattered.**

Right now, the route branch, readiness branch, and submit branch each carry part of the targeted-turn story. That is fragile. The REPL should have one obvious path that says: parse exact backend, classify host vs world, ensure readiness if world, submit, surface exact failure.

**Issue 2. Host and world flows must look unified to the operator without faking a shared transport.**

One internal seam is correct. One generic transport abstraction is not. The operator should experience one selected-backend contract. The implementation should still preserve host-resume and world-submit as different mechanisms.

**Issue 3. Relaunch must stay a readiness concern, not a submit concern.**

If submit starts creating or replacing runtimes, the contract becomes impossible to reason about. World submit must target an exact retained participant. Readiness may create that participant only when the exact backend slot is missing or stale.

**Issue 4. Status/toolbox ambiguity is adjacent, not part of this slice.**

That constraint belongs in the plan because these files sit nearby and the temptation to widen scope will be high. This plan explicitly carries the boundary so implementation does not drift into control-plane redesign.

### Architecture ASCII diagrams

### End-to-end targeted follow-up dispatch

```text
operator enters exact targeted turn
    |
    v
parse "::<backend_id> <prompt>"
    |
    +--> malformed ----------------------------> fail closed before shell fallback
    |
    v
validate_exact_backend_selection(scope, backend_id)
    |
    +--> no match / ambiguous / unrealizable --> fail closed with exact reason
    |
    v
descriptor { backend_id, execution_scope, binary_path, backend_kind, ... }
    |
    +--> host scope
    |      |
    |      +--> active orchestrator runtime exists?
    |      |      |
    |      |      +--> no --------------------> fail closed
    |      |      |
    |      |      +--> yes, backend matches --> submit_host_targeted_turn(...)
    |      |      |
    |      |      +--> yes, backend mismatches -> fail closed
    |
    +--> world scope
           |
           +--> non-Linux --------------------> fail closed
           |
           +--> Linux
                  |
                  +--> ensure_member_runtime_ready_for_descriptor(...)
                  |      |
                  |      +--> exact backend retained and current ----> reuse
                  |      +--> missing/stale exact backend -----------> relaunch exact backend slot
                  |      +--> duplicate/invalid ---------------------> fail closed
                  |
                  +--> submit_world_targeted_turn(...)
```

### World reuse versus relaunch

```text
requested backend_id = "cli:codex"
current world_generation = 7
    |
    v
member_runtimes["cli:codex"] ?
    |
    +--> no ----------------------------------> prepare + launch retained cli:codex for generation 7
    |
    +--> yes
           |
           +--> authoritative-live, generation 7, exact participant matches live slot
           |      |
           |      +--> reuse retained participant
           |
           +--> stale generation / invalidated / mismatched live slot
                  |
                  +--> tear down old retained entry
                  +--> persist invalidation
                  +--> prepare + launch exact replacement
```

### Ownership split that must remain unchanged

```text
host targeted follow-up
    shell-owned runtime
    -> build session resume extension
    -> UAA run_control(...)

world targeted follow-up
    shell-owned exact backend selection + readiness
    -> typed MemberTurnSubmitRequestV1
    -> world-agent retained participant validation
    -> UAA run_control(...) inside world-agent
```

## Code Quality Review

### Findings resolved in-plan

**Issue 1. Inline branching is doing too much narrative work.**

The code should tell the story in one place. Right now the reader has to follow parse logic, backend validation, runtime matching, readiness, and submit functions in separate jumps. The implementation should make that story obvious.

**Issue 2. Error strings must not drift between route branches.**

A targeted host mismatch, a missing active host runtime, and a Linux-only world error are different operator problems. The dispatch seam must centralize where those are chosen so tests can pin them.

**Issue 3. This slice should avoid new abstractions that outlive the problem.**

This is a small blast-radius problem. A direct helper and a small route enum are enough. Anything broader is engineering theater.

**Issue 4. Docs are part of the quality bar here.**

The runtime contract already exists. If the docs keep describing an earlier gap state, the repo becomes misleading. That is a quality defect, not a cosmetic issue.

### Allowed code shape

1. Prefer a small explicit helper or enum over a new dispatcher module.
2. Keep targeted-turn orchestration in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) unless a tiny pure helper clearly improves readability in an already-related module.
3. Do not duplicate selector error taxonomy. Consume `validate_exact_backend_selection(...)` and preserve its exactness.
4. Do not duplicate retained-member identity validation. Leave `validate_submit_turn_request(...)` authoritative on the world side.
5. Do not introduce a generic runtime manager abstraction just to unify two call sites.

## Test Review

### Test framework detection

This repo is Rust-first and the relevant review surface is `cargo test`.

Primary suites for this slice:

1. [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
2. [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
3. selector tests in [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
4. existing retained-member validation tests in [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs), but only if a shell-level proof cannot pin the behavior adequately

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/shell/src/repl/async_repl.rs
    |
    ├── exact targeted-turn parse
    │   ├── [★★★ TESTED] malformed "::<backend_id>" rejects before shell fallback
    │   └── [★★★ TESTED] exact single-line grammar enters targeted route
    |
    ├── host targeted route
    │   ├── [★★★ TESTED] non-active host backend rejects fail-closed
    │   ├── [GAP]         active orchestrator backend follow-up turn resumes successfully
    │   └── [GAP]         missing active orchestrator runtime rejects without shell fallback
    |
    ├── world targeted route on Linux
    │   ├── [★★★ TESTED] exact retained member submit uses typed route without relaunch
    │   ├── [★★★ TESTED] same-generation world command reuses live member runtime
    │   ├── [GAP]         stale retained member relaunches exact backend slot before submit
    │   └── [GAP]         world-runtime-unavailable path names backend and fails closed
    |
    └── world targeted route on non-Linux
        └── [GAP]         Linux-only fail-closed contract is explicitly regression-tested

[+] crates/shell/src/execution/agent_runtime/validator.rs
    |
    ├── validate_exact_backend_selection(...)
    │   ├── [★★★ TESTED] exact backend match returns one descriptor
    │   ├── [★★★ TESTED] duplicate exact backend entries fail closed
    │   ├── [★★★ TESTED] exact world-vs-host scope mismatch is distinct behavior
    │   └── [GAP]         unrealizable exact backend stays exact and does not drift to another backend

[+] crates/world-agent/src/member_runtime.rs
    |
    ├── retained-key validation
    │   ├── [★★★ TESTED] identity drift rejects exact mismatched field
    │   ├── [★★★ TESTED] duplicate retained same-backend slot rejects
    │   └── [★★★ TESTED] concurrent submitted turn collision rejects
    |
    └── world submit path
        └── [★★★ TESTED] typed submit validates participant/world/backend identity

---------------------------------
COVERAGE TARGET
- all targeted host path outcomes at ★★★
- all selected-member relaunch and fail-closed outcomes at ★★★
- no new targeted-turn branch ships without a direct regression test
---------------------------------
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] Operator targets active host backend
    |
    └── [GAP] [->E2E] "::<active_backend_id> follow up" resumes the active orchestrator session

[+] Operator targets retained world backend
    |
    ├── [★★★ TESTED] exact retained backend reuses member and typed submit route
    └── [GAP] [->E2E] stale generation world member is replaced, then exact backend submit succeeds

[+] Operator mistargets host backend
    |
    └── [★★★ TESTED] fail closed, no shell fallback, no world-member launch

[+] Operator hits unsupported world follow-up path
    |
    └── [GAP] explicit Linux-only error remains pinned

[+] Operator-visible error states
    |
    ├── [★★★ TESTED] malformed syntax
    ├── [GAP]         no active orchestrator runtime
    ├── [GAP]         world-runtime-unavailable after readiness
    └── [★★★ TESTED] retained identity drift / concurrent submitted turn
```

### Required tests to add or extend

1. Add a positive host targeted-follow-up integration test in [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) that proves the active orchestrator backend can accept `::<backend_id> <prompt>` follow-up turns via session resume.
2. Add a regression test for `no active orchestrator runtime is available for targeted follow-up turns` so host-targeted dispatch never silently recreates or reroutes a host runtime.
3. Add a stale-generation world-member test that proves the selected backend relaunches only after generation drift, then submits against the replacement exact backend slot.
4. Add a non-Linux fail-closed regression that pins `world-targeted follow-up turns are supported on Linux only`.
5. Add selector-level tests if needed so exact unrealizable-backend behavior remains explicit and exact.

### QA-facing test artifact

During implementation, write a QA-facing artifact to:

```text
~/.gstack/projects/<slug>/<user>-feat-session-centric-state-store-eng-review-test-plan-<timestamp>.md
```

Required contents:

1. targeted host follow-up flow
2. targeted world reuse flow
3. stale-generation world relaunch flow
4. malformed syntax and wrong-backend fail-closed flows
5. Linux-only fail-closed flow for non-Linux world follow-up paths

This artifact is for `/qa` and `/qa-only`. Keep it user-journey oriented, not implementation-detail heavy.

### Regression rule for this slice

These tests are mandatory. No discussion:

1. malformed targeted syntax still rejects before shell fallback
2. host non-active backend rejection still rejects before shell fallback
3. exact world submit still uses `MemberTurnSubmitRequestV1`
4. same-generation retained world-member reuse still stays green
5. duplicate same-backend retained-member rejection still stays green

## Failure Modes Registry

| Failure mode | Test required | Error handling exists | User sees clear failure | Critical gap before this slice lands |
| --- | --- | --- | --- | --- |
| malformed `::` syntax falls through to shell execution | yes | yes | yes | yes |
| host follow-up targets non-active backend | yes | yes | yes | no |
| active host runtime missing on targeted submit | yes | yes | yes | yes |
| exact world backend missing live retained runtime | yes | yes via ready path | yes | no |
| stale retained world member is reused instead of replaced | yes | partial until regression pinned | no unless tested | yes |
| ready path fails and shell silently falls back | yes | should fail closed | must be yes | yes |
| world follow-up on non-Linux appears supported | yes | yes | yes | yes |
| selector drifts from exact match to best-effort match | yes | partial via selector tests | yes if surfaced | yes |
| retained identity drift accepted by world submit | already yes | yes | yes | no |
| concurrent submitted turn collision gets queued implicitly | already yes | yes | yes | no |

Critical gap rule for this plan:

No failure mode is allowed to be both untested and silent. The stale-world relaunch path and the missing-active-host-runtime path are the main current hazards.

## Performance Review

Performance is not the main risk here, but there are still rules.

### Findings resolved in-plan

1. targeted follow-up dispatch must stay in-memory on the hot path. Do not add session-store scans or trace replays to decide host-vs-world routing.
2. exact backend dispatch must use existing descriptor and retained-runtime maps, not new filesystem lookup.
3. world relaunch must stay conditional. Same-generation reuse must not do unnecessary teardown or relaunch work.
4. test scaffolding may get more stateful, but runtime code must not add per-turn O(n) scans across unrelated backends if an exact keyed lookup already exists.

### Performance posture

- no new N+1 style data-access concern exists
- no caching layer is needed
- no background polling loop is needed
- targeted follow-up remains human-paced control-plane traffic
- correctness wins over micro-optimizing lookup structures here

## DX Guardrails

This is developer-facing runtime behavior even though it is backend code.

Required operator experience:

1. exact backend failures must still name the backend id the operator requested
2. host and world follow-up turns must feel like one contract even though they use different transports
3. Linux-only failure must be explicit, not implied by a timeout or absence of output
4. stale-world recovery must preserve exact backend identity in translated REPL output
5. docs must say REPL-first and Linux-first plainly so operators do not infer a broader public surface

## Worktree Parallelization Strategy

This plan has limited but real parallelization opportunities once the dispatch contract is frozen.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Freeze targeted-follow-up contract and exact failure taxonomy | `crates/shell/src/repl/`, `crates/shell/src/execution/agent_runtime/`, repo docs | — |
| Dispatch seam extraction and shell-side error centralization | `crates/shell/src/repl/` | Freeze targeted-follow-up contract and exact failure taxonomy |
| Selector regression pinning | `crates/shell/src/execution/agent_runtime/` | Freeze targeted-follow-up contract and exact failure taxonomy |
| Integration tests, harness extensions, and repo-truth closeout | `crates/shell/tests/`, repo docs | Dispatch seam extraction and shell-side error centralization, Selector regression pinning |

### Parallel lanes

- Lane A: Dispatch seam extraction and shell-side error centralization
  - sequential inside the lane because these steps share `crates/shell/src/repl/`
- Lane B: Selector regression pinning
  - sequential inside the lane because these steps share `crates/shell/src/execution/agent_runtime/`
- Lane C: Integration tests, harness extensions, and repo-truth closeout
  - starts after A and B because the assertions and doc truth depend on the final contract

### Execution order

1. Freeze the targeted-follow-up contract and exact failure taxonomy.
2. Launch Lane A and Lane B in parallel worktrees.
3. Merge A and B.
4. Run Lane C for positive host proof, stale-world relaunch proof, Linux-only fail-closed proof, and doc closeout.

### Conflict flags

- Lane A and Lane C both depend on [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) behavior. Lane C should not edit that file unless a test proves the seam is still missing a contract edge.
- Lane C owns [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs). Keep test-file ownership there to avoid merge churn.
- Repo docs should move last. Updating the gap matrix before the tests land is how documentation drifts from reality again.

### Parallelization verdict

Three workstreams, two parallel implementation lanes, one final integration lane.

## Implementation Sequence

### Step 1. Freeze the targeted-follow-up contract

Files:

1. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
2. [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
3. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md) for language freeze only if needed during execution

Deliver:

1. freeze the exact failure taxonomy for targeted follow-up turns
2. document the route contract as host-active-only or world-exact-only, with no fallback
3. confirm the helper seam will own classification and error mapping, not selector semantics

Done means the contract is explicit before helper extraction begins.

### Step 2. Introduce one explicit targeted-follow-up dispatch seam

Files:

1. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Deliver:

1. extract the inline targeted-turn route from the REPL loop into one helper that owns:
   - exact backend selection
   - host-vs-world classification
   - host backend activity check
   - Linux-only world gate
   - world readiness call
   - final call into host or world submit
2. keep `submit_host_targeted_turn(...)` and `submit_world_targeted_turn(...)` intact as transport implementations
3. keep reuse-vs-relaunch inside `ensure_member_runtime_ready_for_descriptor(...)`

Done means one helper is the obvious place to read the selected-backend follow-up contract.

### Step 3. Freeze host targeted-follow-up semantics with positive proof

Files:

1. [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
2. [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs) only if harness support is needed

Deliver:

1. add a passing host targeted-follow-up test for the active orchestrator backend
2. add a missing-active-host-runtime fail-closed test if the runtime can be invalidated in the harness
3. assert no shell fallback and no world-member launch in both cases

Done means the host path has both positive and negative proof.

### Step 4. Freeze world selected-member relaunch semantics

Files:

1. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) only if helper extraction still leaves relaunch ambiguity
2. [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
3. [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs) if stale-generation scripting is needed

Deliver:

1. add a regression that simulates stale world generation and proves the exact backend slot is relaunched before submit
2. add a regression for explicit world-runtime-unavailable failure if the ready path cannot produce an authoritative-live selected member
3. keep the existing same-generation reuse proof unchanged

Done means same-generation exact backend reuse remains the default and relaunch happens only after missing or stale retained state.

### Step 5. Pin selector exactness where the dispatch seam depends on it

Files:

1. [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) if tests are missing

Deliver:

1. add tests for any exact-backend behaviors the new seam relies on but are not yet pinned
2. especially protect exact unrealizable-backend behavior if it is not already covered

Done means the dispatch seam depends on frozen selector behavior, not assumptions.

### Step 6. Update repo-truth docs and planning index

Files:

1. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
2. [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md) if it is used as the current slice index

Deliver:

1. describe targeted follow-up submit and reuse as landed, Linux-first, and REPL-first
2. describe remaining open work as non-REPL caller surface, status/toolbox productization, and macOS parity
3. remove any stale language that still implies selected-member follow-up submit is broadly unimplemented

Done means the repo says what the runtime actually does today.

## Recommended Verification Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell validate_exact_backend_selection -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p world-agent member_runtime -- --nocapture
```

## Definition of Done

1. `::<backend_id> <prompt>` remains the only targeted follow-up grammar.
2. exact backend selection still resolves through `validate_exact_backend_selection(...)`.
3. host targeted follow-up turns resume only the active orchestrator backend.
4. world targeted follow-up turns on Linux still submit through `MemberTurnSubmitRequestV1`.
5. same-generation exact world-member reuse remains green.
6. stale or missing retained world state causes exact-backend relaunch before submit, not sibling fallback.
7. missing-active-host-runtime and non-Linux world-targeting failures remain explicit and fail closed.
8. no new non-REPL caller surface is introduced.
9. repo docs say REPL-first and Linux-first plainly.
10. the integration suite proves the contract instead of implying it.

## Deferred Work

- non-REPL caller surface
- `substrate -c` redesign
- public `substrate agent start|resume|fork|stop`
- status/toolbox ambiguity handling
- macOS/Lima parity
- broader operator-surface productization beyond the exact targeted REPL path

## Completion Summary

- Step 0: Scope Challenge, scope accepted as-is
- Architecture Review: 4 issues found, all resolved in-plan
- Code Quality Review: 4 issues found, all resolved in-plan
- Test Review: diagram produced, 5 concrete regression gaps identified
- Performance Review: 4 issues found, all resolved in-plan
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 items proposed, deferred work stays inside this plan
- Failure modes: 4 critical gaps flagged until the new regression floor lands
- Outside voice: skipped for this document generation
- Parallelization: 3 lanes, 2 parallel / 1 sequential integration lane
- Lake Score: 8/8 recommendations chose the complete option

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Step 0 | Treat this as a contract-centralization and regression-floor slice, not a transport slice | Mechanical | Pragmatic | The submit transports and retained-member identity model already exist; the gap is cohesion and proof | reopening transport design |
| 2 | Architecture | Keep `validate_exact_backend_selection(...)` as the canonical explicit selector | Mechanical | DRY | One selector contract is safer than duplicating backend-id logic in the REPL | bespoke REPL-only exact selection |
| 3 | Architecture | Add one helper seam that chooses host vs world but does not abstract their transports together | Mechanical | Explicit over clever | Operators need one contract, not one fake implementation model | generic dispatcher framework |
| 4 | Architecture | Keep relaunch inside readiness, not submit | Mechanical | Systems over heroes | Submit should target retained identity, not mutate it | submit-triggered runtime creation |
| 5 | Code Quality | Keep changes inside existing shell and test modules | Mechanical | Minimal diff | The blast radius is small and already localized | new modules or services |
| 6 | Test Review | Make positive host-follow-up proof mandatory | Mechanical | Completeness | The host contract is not fully real until success is proven, not just rejection | inferring host behavior from negative tests |
| 7 | Test Review | Make stale-world relaunch proof mandatory | Mechanical | Completeness | Reuse-vs-relaunch is the main contract edge in this slice | relying on same-generation reuse tests only |
| 8 | Parallelization | Freeze contract first, then run shell seam work and selector pinning in parallel | Mechanical | Pragmatic | The shared contract is small but central, and it reduces merge churn | parallel edits before semantics are frozen |
