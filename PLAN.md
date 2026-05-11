# PLAN: Fix Host Bootstrap Readiness And Clean-Detach Parking

Source SOW: [llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md)  
Prior corrective slice: [llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md)  
Durability ADR: [ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)  
Original failing path anchor: [llm-last-mile/PLAN-22.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-22.md)  
Branch: `feat/host-orchestrator-durable-session`  
Base branch: `main`  
Plan type: runtime correction slice, no UI scope  
Review posture: unified implementation plan, tightened to `/autoplan` completeness and `/plan-eng-review` execution rigor  
Status: execution-ready planning pass on 2026-05-10

## Objective

Fix the original host bootstrap smoke failure without backing out slice `23`.

This slice is complete only when all of the following are true:

1. `substrate agent start --backend <backend_id> --prompt ...` succeeds when bootstrap establishes a valid resumable host session and then the bootstrap control stream ends cleanly.
2. That clean bootstrap exit normalizes the session to `parked_resumable` or `awaiting_attention`, never `invalidated`, `failed`, or otherwise terminal, when the persisted continuity contract is satisfied.
3. `substrate agent turn --session <id> --backend <backend_id> --prompt ...` succeeds against the exact parked session created by that bootstrap path.
4. `substrate agent reattach --session <id>` succeeds against that same parked session and restores attached ownership without submitting a prompt.
5. Truly broken bootstrap still fails closed as `runtime_start_failed`.
6. The durable posture, participant attachment metadata, and session-local inbox from slice `23` remain authoritative.
7. The public prompt bridge still guarantees explicit `Completed` or `Failed` after `Accepted`.

## Plan Summary

This is not a new architecture slice. The durable session model already exists. The remaining bug is that bootstrap still trusts transient attached-process liveness more than persisted orchestration truth.

Today the runtime can model parked sessions and resume synthetic parked fixtures, but the real bootstrap path still has two bad assumptions:

1. readiness waits for attached-live ownership in [`hidden_owner_helper_launch_ready(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:924),
2. clean control-stream exit after continuity exists still invalidates in [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:7505).

The correction is one explicit contract, applied everywhere that matters:

1. define one shared notion of valid detached host continuity,
2. make bootstrap readiness accept that continuity,
3. make bootstrap teardown park or fail from that same contract,
4. prove the exact CLI bootstrap path with regression tests instead of synthetic-only fixtures.

## Locked Starting State

### What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| Public root start surface | [`run_start(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:301) | Reuse. No new verb. |
| Public follow-up and reattach surfaces | [`run_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:324), [`run_reattach(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:384) | Reuse. Tighten behavior only. |
| Hidden owner-helper readiness poll | [`wait_for_hidden_owner_helper_readiness(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:466) | Reuse the poll loop. Change the predicate. |
| Canonical bootstrap readiness predicate | [`hidden_owner_helper_launch_ready(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:924) | Rewrite its truth model. |
| Persisted contract and invariant enforcement from slice `23` | [`validate_runtime_contract(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2124) | Reuse. Do not invent a second contract. |
| Session posture primitives | [`mark_parked_resumable(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:178), [`mark_awaiting_attention(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:187) | Reuse. Drive them from the corrected classifier. |
| Existing explicit parking path | [`park_host_orchestrator_runtime(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:5012) | Reuse or share logic with bootstrap teardown. |
| Public prompt bridge | [`run_public_prompt_command(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1063) | Reuse. Keep terminal-delivery guarantee. |
| Detached-world fail-closed turn behavior | [`run_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:324) | Reuse exactly. |
| Public control integration harness | [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) | Reuse and extend with the real bootstrap regression. |
| Existing failing lifecycle regression | [`start_host_orchestrator_runtime_invalidates_when_attached_control_exits()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:7505) | Replace with the corrected expected behavior. |

### Exact remaining defect

1. `run_start(...)` launches the helper, then waits for readiness before prompt submission.
2. Readiness still requires authoritative live ownership and owner-process liveness.
3. A `codex exec` style backend can emit the session handle and internal session id, then end the bootstrap control stream cleanly.
4. The bootstrap teardown path still interprets that clean stream end as invalidation for host orchestrators.
5. The explicit parking path already exists, but bootstrap teardown does not consistently route through it.
6. Current tests prove parked-session semantics mostly from synthetic parked state, not from the original failing bootstrap path.

### Scope decision

This remains one correction slice. Do not split it into "readiness first" and "teardown later." One half without the other leaves the runtime internally contradictory.

## Scope Lock

### In scope

1. Reclassify hidden owner-helper startup readiness so valid parked host continuity counts as ready.
2. Route clean bootstrap control-stream exit through park-vs-fail classification instead of unconditional invalidation.
3. Align event-task and completion-task bootstrap teardown so race ordering cannot reintroduce invalidation.
4. Align read-side posture and status projection with the corrected startup contract.
5. Add real bootstrap-path regression tests for `start`, `turn`, and `reattach`.
6. Tighten docs and plan references that still imply attached-live ownership is required after bootstrap.

### NOT in scope

1. New public verbs or selector changes.
2. New inbox product UX such as listing or browsing inbox items.
3. Changing the durable inbox envelope again.
4. Detached-world recovery broadening.
5. Reworking compaction policy.
6. Backing out posture, attachment, or inbox fields from slice `23`.
7. Platform broadening beyond keeping current Linux and macOS behavior compile-safe.

## Step 0: Scope Challenge

### 0A. What already solves part of the problem

1. Persisted posture and participant resumability already exist. The repo does not need a second state model.
2. Exact `(session, backend)` routing already exists for `turn`.
3. Explicit `parked_resumable` and `awaiting_attention` normalization already exist in session state.
4. Public `reattach` already restores an owner loop for an orphaned authoritative session.
5. Detached-world follow-up already fails closed with reattach guidance.

What is missing is that bootstrap success is still judged with the old attached-live standard.

### 0B. Minimum honest diff

The minimum honest implementation touches these primary modules:

1. [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
2. [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
3. [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
4. [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
5. targeted tests in [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) and inline module tests
6. focused doc updates after behavior is proven

Anything smaller leaves readiness, teardown, or regression proof ambiguous.

### 0C. Complexity check

This is above "tiny patch" size, but it is not overbuilt if the implementation stays local and explicit:

1. no new crate,
2. no new public command surface,
3. no new background reconciler,
4. no second persisted authority,
5. no generic lifecycle framework.

### 0D. Search and reuse check

1. **[Layer 1]** Reuse the existing persisted contract from slice `23` instead of inventing new detached-session fields.
2. **[Layer 1]** Reuse the existing parking helpers and invariant checks.
3. **[Layer 1]** Reuse the existing public control harness and real CLI flows for regression coverage.
4. **[EUREKA]** The correction is not "make startup more tolerant." It is "judge startup from persisted continuity truth, not process liveness."

### 0E. TODOS cross-reference

There is no root `TODOS.md` today. Deferred work is captured in `NOT in scope` and `Deferred follow-ups` below instead of creating a new project convention inside this slice.

### 0F. Completeness check

Shortcut versions are not acceptable:

1. changing readiness without changing teardown is incomplete,
2. changing teardown without a shared continuity helper is race-prone,
3. fixing both without a real bootstrap regression test is hand-waving,
4. claiming success from synthetic parked fixtures alone is not enough.

## Frozen Runtime Contract

If implementation wants to violate any rule in this section, stop and revise the plan first.

### 1. Valid detached host continuity

Bootstrap may be treated as successfully established when all of the following are true:

1. the orchestration session remains non-terminal,
2. `active_session_handle_id` still points at the authoritative host participant,
3. `attached_participant_id == null`,
4. persisted posture is `parked_resumable` when `pending_inbox_count == 0`, or `awaiting_attention` when `pending_inbox_count > 0`,
5. the authoritative host participant remains `resume_eligible == true`,
6. the authoritative host participant has `attached_client_present == false`,
7. `uaa_session_id` exists when the backend resume path requires it,
8. no session or participant field marks the session invalidated, stopped, failed, or otherwise terminal.

This is the exact proof needed for clean bootstrap exit to count as success.

### 2. Clean bootstrap control-stream end

When continuity has already been established and the bootstrap control stream ends cleanly:

1. attachment diagnostics are released,
2. session `state` remains `Active`,
3. posture normalizes to `parked_resumable` or `awaiting_attention`,
4. the authoritative participant remains `resume_eligible == true`,
5. no `Invalidated`, `Failed`, or terminal transition occurs solely because the stream ended.

### 3. Broken startup still fails closed

Clean exit is still failure when any required resumability proof is missing, including:

1. no authoritative participant,
2. missing required `uaa_session_id`,
3. mismatched `active_session_handle_id`,
4. non-host-routable or terminal authoritative participant,
5. `resume_eligible == false`,
6. detached posture that disagrees with pending inbox truth,
7. already-terminal session lifecycle state.

### 4. Public bridge invariant

The public prompt bridge contract from slice `23` remains:

1. after `Accepted`, emit `Completed` or `Failed`,
2. parked continuity is runtime state, not an operator-visible third terminal envelope,
3. EOF or helper disappearance after `Accepted` is a bug and must render `Failed`.

### 5. `turn` and `reattach` semantics

This slice preserves:

1. `turn` as prompt-taking resume against exact `(orchestration_session_id, backend_id)`,
2. `reattach` as attached-owner recovery only,
3. detached-world follow-up as fail closed.

## Architecture Review

### Architecture thesis

Persist the truth you want bootstrap to trust.

The runtime already stores enough information to know that a host session is detached but resumable. The bug is that bootstrap still trusts transient attachment liveness more than that persisted truth.

### Current broken lifecycle

```text
CURRENT BOOTSTRAP MODEL
=======================
run_start()
    |
    +--> launch hidden owner-helper
    |
    +--> wait_for_hidden_owner_helper_readiness()
           |
           \--> hidden_owner_helper_launch_ready()
                  requires:
                  - session Active
                  - exact participant selected
                  - participant authoritative-live
                  - owner process alive
                  - internal session id present when required
    |
    +--> helper exits cleanly after continuity exists
           |
           \--> bootstrap teardown invalidates
                  because attached control ended
```

### Target lifecycle

```text
TARGET BOOTSTRAP MODEL
======================
run_start()
    |
    +--> launch hidden owner-helper
    |
    +--> wait_for_hidden_owner_helper_readiness()
           |
           \--> readiness = attached-live
                          OR valid detached continuity
    |
    +--> helper exits cleanly after continuity exists
           |
           \--> shared bootstrap teardown classifier
                  |
                  +--> continuity valid, no pending inbox
                  |       -> state Active
                  |       -> posture parked_resumable
                  |
                  +--> continuity valid, pending inbox > 0
                  |       -> state Active
                  |       -> posture awaiting_attention
                  |
                  \--> continuity invalid or incomplete
                          -> runtime_start_failed / invalidated
```

### Dependency graph

```text
BOOTSTRAP CORRECTION GRAPH
==========================
agents_cmd.rs
    |
    \--> control.rs
          |
          +--> state_store.rs
          |     |
          |     +--> readiness classification
          |     +--> continuity contract helper
          |     \--> posture invariant validation
          |
          \--> async_repl.rs
                |
                +--> bootstrap event-task teardown
                +--> bootstrap completion-task teardown
                \--> parking normalization
```

### Production failure scenarios

| Codepath | Real failure | Planned handling |
| --- | --- | --- |
| bootstrap readiness | valid detached continuity never counts as ready, `start` times out or fails | accept parked continuity when persisted invariants are satisfied |
| bootstrap event-task teardown | clean helper exit invalidates a valid session | route through shared park-vs-fail classification |
| bootstrap completion-task race | event task parks but completion path later fails the same session | use the same continuity decision in both paths |
| posture normalization | detached session with pending items still reports `parked_resumable` | normalize to `awaiting_attention` from authoritative pending count |
| bridge terminal delivery | `Accepted` is emitted, then stream dies and user sees silence | render explicit `Failed` |
| detached world protection | parked-host logic accidentally broadens world follow-up | preserve explicit fail-closed branch in `run_turn(...)` |

## File and Symbol Change Map

This section is binding. If implementation expands beyond this list, re-check scope before proceeding.

| File | Symbols / seams | Required change |
| --- | --- | --- |
| [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | `hidden_owner_helper_launch_ready(...)`, `validate_runtime_contract(...)`, posture/readiness helpers | Add one shared detached-continuity helper and use it for readiness and classification inputs. |
| [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs) | `wait_for_hidden_owner_helper_readiness(...)`, `run_public_prompt_command(...)` | Preserve poll loop, consume the corrected readiness predicate, keep `Accepted -> Completed|Failed`. |
| [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | bootstrap event-task teardown, bootstrap completion-task teardown, `park_host_orchestrator_runtime(...)` | Route clean bootstrap exit through shared park-vs-fail classification. Keep explicit failure for broken continuity. |
| [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | `run_start(...)`, `run_turn(...)`, `run_reattach(...)` | Align public flows with the corrected readiness/parking contract. No CLI surface changes. |
| [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) | start/turn/reattach integration coverage | Add the real bootstrap regression path. |
| Inline tests | `state_store.rs`, `control.rs`, `async_repl.rs` | Add contract, race, and bridge guard coverage. |

## Detailed Execution Plan

### Workstream 1: Freeze the shared continuity contract

Primary modules:

1. [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
2. [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)

Implement:

1. factor one shared helper that decides whether persisted host continuity is sufficient for bootstrap success,
2. update `hidden_owner_helper_launch_ready(...)` to accept either attached-live ownership or valid detached continuity,
3. keep the `require_internal_session_id` gate and apply it to the detached path too,
4. keep readiness false for terminal, mismatched, or non-resumable session state.

Exit criteria:

1. a cleanly detached but resumable host session passes readiness,
2. missing `uaa_session_id` still blocks readiness when required,
3. readiness no longer depends on `owner_process_is_alive(...)` once persisted continuity is valid.

### Workstream 2: Rewrite bootstrap teardown classification

Primary modules:

1. [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
2. any shared helpers added in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Implement:

1. replace unconditional host invalidation on clean bootstrap stream end with shared park-vs-fail classification,
2. route the completion-side startup-failure path through that same classification so event/completion ordering cannot diverge,
3. preserve invalidation for genuinely broken startup and for terminal lifecycle conditions,
4. reuse the existing parking semantics instead of inventing a second parking path.

Exit criteria:

1. `"shell-owned orchestrator control stream ended before completion observation"` is fatal only when continuity is not valid,
2. valid clean detach normalizes to `parked_resumable` or `awaiting_attention`,
3. event-task and completion-task teardown make the same decision for the same persisted state.

### Workstream 3: Align public command behavior and status projection

Primary modules:

1. [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
2. [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
3. read-side helpers in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Implement:

1. ensure `run_start(...)` can proceed when continuity is valid even if the helper is already detached,
2. keep `run_turn(...)` resuming exact parked host sessions only, no fuzzy recovery,
3. keep detached-world follow-up fail closed,
4. ensure read-side posture classification honors authoritative persisted posture and pending inbox count, not attachment liveness alone,
5. keep `runtime_start_failed` taxonomy aligned with the corrected bootstrap contract.

Exit criteria:

1. public `start` succeeds on the original manual failing path,
2. public `turn` works against the exact session created by that path,
3. public `reattach` works against the exact session created by that path,
4. broken bootstrap still emits explicit failure.

### Workstream 4: Replace synthetic-only proof with real regression coverage

Primary modules:

1. [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
2. [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
3. targeted unit tests in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
4. targeted tests in [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)

Implement:

1. replace the invalidation regression with a parked bootstrap regression,
2. add a real public `start` test where a fake host backend emits valid resumability metadata and then exits cleanly,
3. add a follow-up `turn` test against the parked session produced by that real bootstrap,
4. add a follow-up `reattach` test against that same parked session,
5. add a guard test for genuinely broken startup without resumability proof,
6. keep synthetic parked-session tests as support, not as the only proof.

Exit criteria:

1. the original manual smoke path is now proven in tests,
2. synthetic parked fixtures remain green,
3. the old invalidation assertion is gone.

### Workstream 5: Docs closeout

Likely docs:

1. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
2. [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)
3. this root [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)

Implement:

1. update docs that still imply bootstrap success requires a continuously attached live owner,
2. clarify the distinction between successful parked bootstrap, real startup failure, and post-`Accepted` terminal failure,
3. update any nearby ASCII diagrams that would become stale.

Exit criteria:

1. repo truth matches code truth,
2. no doc claims slice `24` is solved by synthetic parked fixtures alone.

## Code Quality Review

### DRY and explicitness rules

1. There must be one shared helper for "valid detached bootstrap continuity."
2. There must be one shared decision path for "park vs fail on clean bootstrap exit."
3. Do not copy the same posture predicate into `state_store.rs`, `control.rs`, and `async_repl.rs`.
4. Keep diagnostic booleans diagnostic. Do not quietly make them authoritative again.

### Minimal-diff rules

1. Reuse existing posture fields and invariant validation from slice `23`.
2. Reuse the existing parking path and adapt bootstrap teardown to call into it or share its logic.
3. Extend existing tests before inventing a new durability suite.

### Inline diagram candidates

If implementation adds non-obvious transition logic, add or update nearby ASCII comments in:

1. [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) for readiness classification,
2. [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) for bootstrap teardown flow,
3. [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs) for `Accepted -> Completed|Failed`.

## Test Review

### Test framework

This is a Rust workspace. Primary test surfaces already exist in:

1. inline module tests in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) and [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs),
2. REPL/runtime tests in [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs),
3. integration coverage in [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs).

### Code path coverage

```text
CODE PATH COVERAGE
==================
[+] state_store.rs
    |
    ├── [GAP] hidden_owner_helper_launch_ready() accepts detached resumable host continuity
    ├── [GAP] readiness rejects detached continuity when required internal session id is missing
    ├── [★★ TESTED, KEEP] parked_resumable / awaiting_attention invariant validation
    └── [GAP] read-side posture classification no longer depends on owner_process_is_alive()

[+] control.rs
    |
    ├── [GAP] wait_for_hidden_owner_helper_readiness() succeeds on valid detached continuity
    ├── [★★ TESTED, KEEP] runtime_start_failed classifier plumbing
    └── [GAP] Accepted always ends in Completed or Failed on late bootstrap failure

[+] async_repl.rs
    |
    ├── [REGRESSION TEST REQUIRED] clean host bootstrap exit parks instead of invalidates
    ├── [GAP] detached bootstrap with pending inbox becomes awaiting_attention
    ├── [GAP] broken bootstrap without resumability still fails closed
    └── [GAP] event/completion race normalizes consistently

[+] agents_cmd.rs
    |
    ├── [GAP] run_start() succeeds on real clean-exit bootstrap path
    ├── [GAP] run_turn() resumes the parked session created by that path
    ├── [★★ TESTED, KEEP] detached world follow-up stays fail closed
    └── [GAP] run_reattach() restores attached ownership for that same parked session
```

### User flow coverage

```text
USER FLOW COVERAGE
==================
[+] substrate agent start --backend <host> --prompt "hello" --json
    ├── [GAP] backend emits valid session handle, then control stream ends cleanly
    ├── [GAP] resulting session posture is parked_resumable when inbox is empty
    └── [GAP] resulting session posture is awaiting_attention when inbox has pending work

[+] substrate agent turn --session <id> --backend <host> --prompt "next" --json
    ├── [GAP] exact parked session resumes and accepts prompt
    └── [★★ TESTED, KEEP] detached world session still fails closed with reattach guidance

[+] substrate agent reattach --session <id> --json
    ├── [★★ TESTED, KEEP] exact-session lineage preservation
    └── [GAP] parked bootstrap session returns to attached ownership without prompt side effect

[+] Post-Accepted bridge behavior
    ├── [GAP] late helper drop emits explicit Failed
    └── [GAP] no silent terminal disappearance remains
```

### Required test additions

1. Replace `start_host_orchestrator_runtime_invalidates_when_attached_control_exits()` with a regression that asserts parking after continuity is established.
2. Add a unit test for `hidden_owner_helper_launch_ready(...)` that passes when the session is detached, non-terminal, resume-eligible, and internally resumable.
3. Add a unit test for `hidden_owner_helper_launch_ready(...)` that fails when the same detached session is missing required resume metadata.
4. Add an event/completion race test proving the same persisted session lands on the same outcome regardless of which task observes stream end first.
5. Extend `agent_public_control_surface_v1.rs` with a real `start -> parked -> turn` flow using a fake host backend that exits cleanly after publishing resumability metadata.
6. Extend `agent_public_control_surface_v1.rs` with `start -> parked -> reattach` for the same session.
7. Add a guard test proving bootstrap still fails closed when session handle or required `uaa_session_id` never becomes durable.
8. Add a control-path test proving post-`Accepted` EOF still renders `Failed`.

### Test plan artifact to generate during implementation

When implementation starts, write a QA-facing artifact under `~/.gstack/projects/<slug>/` that covers:

1. affected CLI flows: `start`, `turn`, `reattach`,
2. parked-empty versus attention-needed outcomes,
3. broken bootstrap versus valid parked continuity,
4. post-`Accepted` explicit failure delivery,
5. detached-world fail-closed regression protection.

### Required validation commands

Run at minimum:

```bash
cargo test -p shell async_repl -- --nocapture
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell agent_runtime::control -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
```

Manual validation must also rerun the real CLI flow:

```bash
substrate agent start --backend <host_backend_id> --prompt "hello" --json
substrate agent turn --session <orchestration_session_id> --backend <host_backend_id> --prompt "next" --json
substrate agent reattach --session <orchestration_session_id> --json
```

## Performance Review

The hot path here is not throughput. It is correctness under polling and teardown races.

Performance rules:

1. do not make readiness polling rescan more state than necessary on every loop,
2. do not add directory-wide scans to steady-state `start`, `turn`, or status reads,
3. keep pending-inbox truth O(1) by trusting the persisted counter plus invariant checks,
4. avoid duplicate full-session rewrites when only teardown classification changes one or two fields.

## Failure Modes Registry

| Failure mode | Test required | Error handling required | User-visible outcome |
| --- | --- | --- | --- |
| valid detached continuity never counts as ready | yes | yes | `start` succeeds instead of timing out or failing spuriously |
| clean helper exit invalidates a valid session | yes | yes | session parks instead of dying |
| pending inbox exists but posture stays parked_resumable | yes | yes | operator sees `awaiting_attention` |
| broken bootstrap gets parked anyway | yes | yes | explicit `runtime_start_failed` |
| post-`Accepted` stream drop goes silent | yes | yes | explicit `Failed` |
| detached world follow-up piggybacks on parked-host logic | yes | yes | explicit fail-closed rejection with reattach guidance |

**Critical gaps this plan must close:** 4

1. readiness false negative on valid detached continuity,
2. clean-exit invalidation on the bootstrap seam,
3. race divergence between event-task and completion-task teardown,
4. synthetic-only proof for a real CLI failure path.

## Worktree Parallelization Strategy

There is one real parallel window, and it opens only after the shared continuity contract is frozen.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| 1. Shared continuity and readiness contract | `crates/shell/src/execution/agent_runtime/` | — |
| 2. Bootstrap teardown rewrite | `crates/shell/src/repl/`, `crates/shell/src/execution/agent_runtime/` | 1 |
| 3. Public command and projection alignment | `crates/shell/src/execution/`, `crates/shell/src/execution/agent_runtime/` | 1 |
| 4. Regression and integration tests | `crates/shell/tests/`, inline test modules | 2, 3 |
| 5. Docs closeout | `docs/`, `llm-last-mile/`, root plan docs | 4 |

### Parallel lanes

Lane A: step 1 -> step 2  
Reason: teardown behavior depends directly on the frozen continuity contract.

Lane B: step 1 -> step 3  
Reason: once the shared helper and contract shape are fixed, public command behavior can be aligned mostly outside `async_repl.rs`.

Lane C: step 4 -> step 5  
Reason: validation and docs should happen on the merged tree, not on guessed intermediate behavior.

### Execution order

1. Freeze step 1 first. That is the contract every later change depends on.
2. Launch Lane A and Lane B in parallel worktrees only after step 1 is stable.
3. Merge A and B.
4. Run Lane C on the merged tree for regression proof and docs closeout.

### Conflict flags

1. `state_store.rs` is the contract hotspot. If both implementation lanes need the shared helper, assign helper ownership to one lane and keep the other lane caller-only.
2. `control.rs` is a secondary hotspot if readiness and prompt-bridge failure handling are mixed in the same patch. Split by function ownership, not fake file exclusivity.
3. `agent_public_control_surface_v1.rs` should be test-lane owned. Do not let both implementation lanes churn it at once.

## Implementation Checklist

1. Define one shared helper for valid detached bootstrap continuity.
2. Update `hidden_owner_helper_launch_ready(...)` to use that helper.
3. Update bootstrap event-task teardown to park when continuity is valid.
4. Update bootstrap completion-task teardown to make the same decision.
5. Keep invalidation only for genuinely broken startup.
6. Verify read-side posture classification respects pending inbox truth and detached resumability.
7. Extend public `start`, `turn`, and `reattach` tests around the real clean-exit bootstrap path.
8. Replace the old invalidation regression with a parked-session regression.
9. Update docs and diagrams after tests prove the corrected behavior.

## Deferred Follow-Ups

These are real ideas, not part of this PR:

1. Operator-facing inbox inspection commands.
2. Inbox retention windows and compaction policy.
3. Broader platform-parity work for the same durability model.

## Completion Summary

- Step 0: Scope Challenge, scope accepted as-is, with a strict no-new-control-plane rule.
- Architecture Review: one architectural direction, persisted detached continuity is authoritative once established.
- Code Quality Review: single continuity helper, single park-vs-fail decision path, no diagnostic booleans as authority.
- Test Review: coverage diagram produced, real bootstrap-path gaps identified, 4 critical.
- Performance Review: no throughput rewrite needed, but do not add polling or scan-heavy steady-state reads.
- NOT in scope: written.
- What already exists: written.
- Failure modes: 4 critical gaps flagged.
- Parallelization: 3 lanes total, 1 real parallel window after contract freeze.
- Lake Score: 9/9 recommendations choose the complete option over the shortcut.

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 0 | — | — |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | — | — |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 0 | — | Plan authored to eng-review structure, but no formal review run logged yet |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | skipped | No UI scope |

**VERDICT:** PLAN READY FOR IMPLEMENTATION REVIEW. If you want the full logged review chain later, run `/autoplan` or `/plan-eng-review` against this file.
