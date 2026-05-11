<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/feat-host-orchestrator-durable-session-autoplan-restore-20260511-131509.md -->
# PLAN: Finish The Durable Host-Orchestrator Session Model

Source SOW: [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)  
Primary landed precursor: [23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md)  
Corrective precursor: [24-fix-host-bootstrap-readiness-and-clean-detach-parking.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md)  
Caller-surface contract: [20-public-non-interactive-agent-caller-surface.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/20-public-non-interactive-agent-caller-surface.md)  
Packet index: [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)  
Supersedes: previous root `PLAN.md`, which was still scoped to the slice-24 bootstrap seam instead of the full durable-session truth record  
Branch: `feat/host-orchestrator-durable-session`  
Base branch: `main`  
Plan type: runtime correction and contract closeout, no UI scope, developer-facing CLI/runtime scope  
Status: unified implementation plan, 2026-05-11

## Objective

Finish the host-orchestrator durable session model so runtime behavior matches the truth record exactly, not approximately.

The architecture is already mostly present. The remaining problem is contract inconsistency at the public control surfaces. `start` and `turn` already behave like durable-session commands in important ways, but `reattach`, `stop`, and `status` still partially privilege live-owner transport over authoritative session truth. That leaves the model internally contradictory.

This plan closes that gap. It does not introduce a new lifecycle model. It makes the existing durable-session model authoritative everywhere.

## Acceptance Criteria

This plan is complete only when all of the following are true:

1. `substrate agent start --backend <backend_id> --prompt ...` creates or binds one durable Substrate-owned orchestration session and uses the user prompt as the true initial backend prompt.
2. That orchestration session remains open after the initial backend run exits cleanly. Clean detach is not terminal by itself.
3. `active_attached`, `parked_resumable`, and `awaiting_attention` are all treated as active durable-session states, not three different flavors of maybe-alive.
4. `substrate agent turn --session <id> --backend <backend_id> --prompt ...` resumes prompt-taking against that same exact durable session.
5. `substrate agent reattach --session <id>` reports success only when attached host ownership was durably re-established for that same exact durable session.
6. `substrate agent stop --session <id>` stops the durable session cleanly whether the session is attached or parked.
7. `substrate agent status --json` surfaces parked durable sessions as real active sessions instead of hiding them because no live owner process is attached.
8. Durable inbox items continue to land, persist, and drive `awaiting_attention` while no host client is attached.
9. Broken startup still fails closed as `runtime_start_failed`, and the public prompt bridge still guarantees `Accepted -> Completed|Failed`.

## Scope Lock

### In scope

1. Make `reattach` prove durable attached ownership before returning success.
2. Make `stop` session-centric so parked durable sessions stop cleanly without needing a live owner.
3. Make `status` session-centric so parked active sessions stay visible.
4. Prove that parked sessions can own durable inbox work and normalize to `awaiting_attention`.
5. Preserve landed `start` and `turn` semantics, including exact session targeting and fail-closed world-member follow-up.
6. Add regression coverage for the exact public CLI flows: `start`, `turn`, `reattach`, `stop`, `status`, inbox-driven posture changes.
7. Update docs that still imply durable truth depends on an attached live owner process.

### NOT in scope

1. New public verbs or UX for browsing inbox items.
2. A new timeout or stale-session lifecycle policy.
3. Reworking the durable inbox schema or artifact layout.
4. Broadening world-member detached recovery beyond the existing fail-closed contract.
5. Any rollback of slice-23 posture, participant, or inbox fields.
6. Platform expansion beyond keeping current Linux and macOS behavior compile-safe.

## Starting Point

### What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| Public root start surface | [`run_start(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | Reuse. No new verb. |
| Exact follow-up prompt surface | [`run_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | Reuse. Preserve exact `(session, backend)` targeting. |
| Current reattach surface | [`run_reattach(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | Keep the verb. Tighten success semantics. |
| Current stop surface | [`run_stop(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | Keep the verb. Replace live-owner-only assumption. |
| Current status surface | [`build_status_report(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | Reuse the report shape. Change parked-session projection truth. |
| Session discovery and strict linkage | [`load_session(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs), [`resolve_public_control_target(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse. Expand control-posture handling. |
| Detached continuity rules for bootstrap | [`classify_hidden_owner_helper_launch_readiness(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Keep landed bootstrap fix. Use the same truth model elsewhere. |
| Detached parking helpers | [`park_host_orchestrator_runtime(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs), [`build_parked_host_runtime_snapshots(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse. Do not invent a second parked-state writer. |
| Durable inbox persistence | [`persist_inbox_item(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs), [`persist_runtime_alert_for_dev_support(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_dev_support.rs) | Reuse. Prove it operationally for parked sessions. |
| Posture primitives | [`mark_active_attached(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs), [`mark_parked_resumable(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs), [`mark_awaiting_attention(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs) | Reuse exactly. |
| Public prompt bridge invariant | [`run_public_prompt_command(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs) | Preserve `Accepted -> Completed|Failed`. |

### Exact remaining gaps

| Gap | Current behavior | Why it is still wrong |
| --- | --- | --- |
| `reattach` is not fully landed | `run_reattach(...)` returns success immediately after helper launch | The truth record says success means attached ownership was actually restored, not that a helper briefly existed. |
| `stop` is still attached-owner-centric | `run_stop(...)` resolves only a live owner and uses private stop transport | Parked durable sessions should still stop cleanly. Requiring a live owner means the durable session is not actually authoritative. |
| `status` is still live-biased | `build_status_report(...)` primarily projects live participants and uses trace fallback for gaps | Parked sessions are valid active sessions and should be shown from authoritative session truth, not as best-effort leftovers. |
| Parked durable responsibility is under-proven | Inbox persistence exists, but not as a fully validated parked-session operational path | The truth record explicitly says parked orchestration authority is not fully proven while `reattach`, `stop`, and `status` still disagree with the model. |

### Minimum honest diff

The minimum honest implementation touches these modules:

1. [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
2. [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
3. [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
4. [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
5. targeted tests in [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) plus inline runtime/state-store tests
6. focused doc updates in [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md), [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md), and this root plan

Anything smaller leaves either `reattach`, `stop`, `status`, or parked responsibility ambiguous.

### Complexity, reuse, and completeness

This is a medium correction slice. It stays acceptable only if the implementation stays boring:

1. no new crate,
2. no new background daemon,
3. no new persisted authority model,
4. no new public CLI verb,
5. no second detached-session state machine.

Reuse rules:

1. **[Layer 1]** Reuse the landed detached continuity and parked snapshot helpers. Do not re-express the same truth in a new helper tree.
2. **[Layer 1]** Reuse existing durable inbox mutation flow. Do not create a side-channel counter or shadow queue.
3. **[Layer 1]** Reuse current public CLI verbs and exact-session resolution.
4. **[EUREKA]** The shared bug is not “reattach, stop, and status each need a one-off fix.” The shared bug is “public control paths still privilege live-owner transport over authoritative durable session truth.”

There is no root `TODOS.md` in this repo today. Deferred work stays explicit in `NOT in scope` and `Deferred follow-ups` instead of inventing a new backlog surface during this slice.

Shortcut versions are not acceptable:

1. fixing `reattach` without `stop` still leaves parked sessions half-real,
2. fixing `stop` without `status` still leaves operators blind,
3. fixing public control without inbox proof still leaves the parked authority story unproven,
4. fixing behavior without CLI regressions is just optimism with line numbers.

No new binary, package, or container artifact is introduced here. Distribution work is not part of this slice.

## Frozen Contract

If implementation wants to violate any rule in this section, stop and revise the plan first. Do not improvise around the contract.

### 1. Durable authority

The durable authority is:

1. the orchestration session record,
2. the authoritative participant linkage,
3. the durable inbox and pending-count state,
4. the persisted routing and lifecycle metadata.

The durable authority is not:

1. one current helper PID,
2. one current `codex exec` process,
3. one active private stop transport socket.

### 2. Active session postures

The session remains active and durable in all three of these postures:

1. `active_attached`,
2. `parked_resumable`,
3. `awaiting_attention`.

`terminal` is the only posture family that stops routability.

### 3. Reattach success semantics

`substrate agent reattach --session <id>` may only report success when all of the following are durably true for that exact orchestration session:

1. session state is still `Active`,
2. session posture is `active_attached`,
3. `attached_participant_id` points at the newly attached retained owner,
4. that participant reports `attached_client_present == true`,
5. that participant is authoritative-live for the session,
6. the resumed owner has not already fallen back to a detached posture before command completion.

If the helper launches and immediately detaches again, the command failed. “Helper spawned” is not a success condition.

### 4. Stop semantics

`substrate agent stop --session <id>` must stop the durable session model, not just the live-owner transport.

Rules:

1. attached sessions may still use private stop transport,
2. parked or `awaiting_attention` sessions must still stop cleanly through authoritative durable-state closeout,
3. stale linkage or already-terminal sessions still fail closed,
4. successful stop must drive the session to terminal persisted truth, not merely return transport success.

### 5. Status semantics

`substrate agent status --json` must surface active durable sessions from authoritative session truth.

That means:

1. a parked host session still appears as a real active session,
2. `awaiting_attention` remains visible with pending inbox truth,
3. live participant rows may enrich projection, but cannot be the only path to visibility,
4. torn or malformed roots may degrade to warnings, but valid parked sessions must not disappear.

### 6. Parked inbox semantics

While detached:

1. inbox items still persist under the authoritative session root,
2. `pending_inbox_count` stays authoritative,
3. zero pending items normalizes to `parked_resumable`,
4. pending items normalize to `awaiting_attention`,
5. later `turn`, `reattach`, or `stop` still target that same durable session.

### 7. Prompt-bridge invariant

The public prompt bridge invariant remains:

1. after `Accepted`, emit `Completed` or `Failed`,
2. parked continuity is runtime state, not a third terminal envelope,
3. silent EOF after `Accepted` is still a bug and must render explicit `Failed`.

## Architecture

### Architecture thesis

Public control should resolve through the session record first, then pick the transport or state transition that matches that session posture.

Today some codepaths still do the opposite. That is why parked sessions are “real” in one command and “gone” in another.

### Current broken control model

```text
CURRENT PUBLIC CONTROL MODEL
============================
durable session record exists
        |
        +--> turn
        |     -> mostly session-centric, exact backend targeting
        |
        +--> reattach
        |     -> spawn helper
        |     -> report success too early
        |
        +--> stop
        |     -> require reachable live owner
        |     -> parked session looks owner_unreachable
        |
        \--> status
              -> prefer live participant projection
              -> parked session can fade behind fallback logic
```

### Target durable control model

```text
TARGET PUBLIC CONTROL MODEL
===========================
authoritative session record
        |
        +--> posture = active_attached
        |     -> turn
        |     -> reattach rejects "already owned"
        |     -> stop uses private owner transport
        |     -> status shows active_attached
        |
        +--> posture = parked_resumable
        |     -> turn resumes exact session
        |     -> reattach restores attached ownership
        |     -> stop closes durable session directly
        |     -> status shows parked_resumable
        |
        +--> posture = awaiting_attention
        |     -> same exact session remains routable
        |     -> inbox work remains durable
        |     -> stop still closes durable session directly
        |     -> status shows awaiting_attention
        |
        \--> posture = terminal
              -> turn / reattach / stop fail closed or no-op by contract
```

### Durable session state machine

```text
HOST DURABLE SESSION STATE MACHINE
==================================
active_attached
    |
    +-- clean owner detach, no pending inbox --> parked_resumable
    |
    +-- clean owner detach, pending inbox > 0 --> awaiting_attention
    |
    +-- explicit stop --> terminal
    |
    \-- real runtime failure --> terminal

parked_resumable
    |
    +-- reattach succeeds --> active_attached
    +-- turn succeeds --> active_attached (then may park again later)
    +-- inbox item arrives --> awaiting_attention
    \-- explicit stop --> terminal

awaiting_attention
    |
    +-- inbox drained and still detached --> parked_resumable
    +-- reattach succeeds --> active_attached
    +-- turn succeeds --> active_attached
    \-- explicit stop --> terminal
```

### Dependency graph

```text
DURABLE CONTROL CLOSEOUT GRAPH
==============================
agents_cmd.rs
    |
    +--> reattach result semantics
    +--> stop resolution
    +--> status report assembly
    |
    +--> state_store.rs
    |     |
    |     +--> exact session resolution
    |     +--> posture classification
    |     +--> durable inbox truth
    |     \--> parked session projection helpers
    |
    +--> control.rs
    |     |
    |     +--> private stop for attached owners
    |     \--> prompt bridge terminal delivery
    |
    \--> async_repl.rs
          |
          +--> parked snapshot application
          +--> retained owner lifecycle
          \--> reattach stabilization / closeout helpers
```

### Production failure scenarios

| Codepath | Real failure | Planned handling |
| --- | --- | --- |
| `reattach` | helper spawns, command returns success, session is parked again one tick later | add a durable attached-success barrier before `reattach` returns success |
| `stop` | parked session reports `owner_unreachable` forever | split attached-stop from durable parked-stop |
| `status` | active parked session disappears because there is no live participant row | build a parked-session projection from the authoritative session record |
| inbox | durable inbox item lands but operator still sees parked instead of attention-needed | assert status projection and posture both follow authoritative `pending_inbox_count` |
| bridge | prompt accepted, helper disappears, caller sees silence | preserve explicit `Failed` after `Accepted` |
| world follow-up | parked-host fixes accidentally relax detached world-member fail-closed behavior | keep world-member detached follow-up unchanged and covered by regression tests |

## File And Symbol Change Map

| File | Symbols / seams | Required change |
| --- | --- | --- |
| [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | `run_reattach(...)`, `run_stop(...)`, `build_status_report(...)`, projection helpers | `reattach` must wait for durable attached truth. `stop` must support parked sessions. `status` must project parked sessions from session truth. |
| [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | `resolve_public_control_target(...)`, session loading/building, posture helpers, inbox helpers | Add one shared session-control posture contract used by `reattach`, `stop`, and `status`. Preserve exact linkage checks. |
| [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs) | private stop transport, prompt bridge helpers | Keep private stop for attached sessions. Preserve explicit terminal delivery after `Accepted`. |
| [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | retained owner lifecycle, parking helpers | Reuse parked snapshot flow. Add any owner-stabilization or durable closeout helper needed so `reattach` and parked `stop` are not transport illusions. |
| [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) | public CLI integration coverage | Add `start -> status -> turn -> reattach -> stop` durable-session regression flow. |
| Inline tests | `state_store.rs`, `control.rs`, `async_repl.rs` | Add posture, stop, status, inbox, and prompt-bridge regressions. |

## Implementation Plan

There are no unresolved design choices in this plan. The work is sequential until the control model is frozen, then parallelizable for tests and docs. Each workstream below specifies exactly what must change and what counts as done.

### Workstream 1: Freeze one session-control posture contract

Primary modules:

1. [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
2. [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Required changes:

1. Add one explicit helper that classifies session control posture from authoritative session state plus authoritative participant linkage.
2. Make that helper the shared truth for `reattach`, `stop`, and parked status projection.
3. Keep exact-session resolution strict.
4. Keep world-sensitive posture fail-closed where already required.

Done when:

1. the code has one obvious answer to “is this durable session attached, parked, attention-needed, or terminal?”,
2. public control paths stop re-deriving that truth ad hoc,
3. no helper falls back to PID liveness alone when the session record already says more.

### Workstream 2: Make `reattach` prove actual attachment

Primary modules:

1. [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
2. [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
3. [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Required changes:

1. Keep the hidden owner-helper resume path.
2. Add a stabilization barrier so `run_reattach(...)` returns success only after the resumed session is durably `active_attached`.
3. Fail if the helper launches and immediately parks again before command completion.
4. Preserve non-prompt-taking semantics and exact-session recovery.

Done when:

1. successful `reattach` means attached ownership is actually restored,
2. failed `reattach` never returns a false “active” JSON result,
3. the exact parked session remains the recovery target.

### Workstream 3: Make `stop` session-centric

Primary modules:

1. [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
2. [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
3. [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
4. [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Required changes:

1. Keep private stop transport for truly attached sessions.
2. Add a parked-session stop path that closes the durable session from authoritative runtime state without requiring a reachable live owner.
3. Make both paths converge on one terminal session-closeout helper so the persisted end state is identical.
4. Preserve strict stale-linkage failure instead of guessing.

Done when:

1. parked sessions stop cleanly,
2. attached sessions still stop cleanly,
3. successful stop ends in authoritative terminal persisted truth,
4. `owner_unreachable` is reserved for real failures, not valid parked sessions.

### Workstream 4: Make `status` reflect durable parked truth

Primary modules:

1. [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
2. [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Required changes:

1. Surface parked and attention-needed sessions from authoritative session records even when no live participant row exists.
2. Preserve live participant rows as enrichments, not the sole visibility path.
3. Preserve degraded warnings for torn roots.
4. Ensure parked-session status surfaces the real posture and session id instead of vanishing into trace fallback behavior.

Done when:

1. valid parked sessions are visible in `agent status --json`,
2. `awaiting_attention` is visible when pending inbox work exists,
3. status remains strict about malformed linkage while still readable for valid detached sessions.

### Workstream 5: Prove parked inbox responsibility operationally

Primary modules:

1. [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
2. [`agent_dev_support.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_dev_support.rs)
3. [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Required changes:

1. Add regression proof that a parked session can receive a durable inbox item while detached.
2. Assert `pending_inbox_count` increments and posture becomes `awaiting_attention`.
3. Assert `status` shows that same session as `awaiting_attention`.
4. Assert later `turn`, `reattach`, or `stop` still operate on that exact session.

Done when:

1. inbox work is not just stored, it is operationally part of the parked durable session model,
2. detached session truth remains internally consistent across store, status, and control surfaces.

### Workstream 6: Replace “looks right” with public CLI proof

Primary modules:

1. [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
2. inline tests in runtime/state-store modules

Required changes:

1. Add a real CLI regression that proves `start` yields a parked durable session.
2. Add `status` proof on that same session while parked.
3. Add `turn` proof on that same session.
4. Add `reattach` proof on that same session.
5. Add `stop` proof on that same session.
6. Add parked-inbox proof for `awaiting_attention`.
7. Keep broken bootstrap fail-closed and post-`Accepted` explicit failure coverage.

Done when:

1. the durable-session truth is proven from the actual public CLI surface,
2. synthetic fixtures remain support evidence, not the main proof.

### Workstream 7: Docs closeout

Likely docs:

1. [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)
2. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
3. this root [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)

Required changes:

1. Remove any remaining wording that implies parked means gone.
2. Clarify that `reattach` success means attached truth, not helper launch.
3. Clarify that `stop` is the closeout path for attached and parked durable sessions.
4. Update nearby ASCII diagrams if they become stale.

Done when:

1. repo docs match runtime truth,
2. no doc quietly preserves the attached-live-only mental model.

### Implementation order

1. Land Workstream 1 first. Everything else depends on one shared control-posture truth.
2. Land Workstreams 2, 3, and 4 in that order inside one sequential lane. They all touch the same control modules and are semantically coupled.
3. Land Workstream 5 once `status` and `stop` reflect the same durable truth.
4. Land Workstream 6 after behavior stabilizes, because the integration test should prove final semantics, not chase churn.
5. Land Workstream 7 after code behavior and tests are frozen.

## Engineering Guardrails

### Code quality guardrails

1. There must be one shared helper for session-control posture classification.
2. There must be one shared durable session closeout path for stop terminalization.
3. There must be one obvious status projection path for parked sessions.
4. Do not duplicate liveness-vs-authority predicates across `agents_cmd.rs`, `state_store.rs`, and `async_repl.rs`.
5. Reuse the existing session store and parked snapshot writers.
6. Reuse private stop transport for attached sessions instead of replacing it.
7. Extend existing CLI integration coverage before inventing a new durability suite.
8. Keep world-member detached follow-up logic unchanged unless a regression test proves a real bug there.

### Inline diagram candidates

If implementation adds non-obvious transition logic, add or update nearby ASCII comments in:

1. [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) for session-control posture classification,
2. [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) for attached-stop vs parked-stop dispatch,
3. [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) for retained-owner attach/detach lifecycle,
4. [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs) for prompt-bridge terminal delivery after `Accepted`.

### Performance guardrails

The hot path here is correctness under session-control resolution, not raw throughput.

Performance rules:

1. do not add repeated full-directory scans to `stop` or `status`,
2. prefer authoritative session record plus already-loaded participant set over extra trace fishing,
3. keep pending-inbox posture normalization O(1) by trusting the authoritative counter and invariant checks,
4. keep attached-stop on the existing private transport path,
5. avoid double-writing the same session terminalization through separate attached and parked codepaths.

## Test Strategy

### Test framework

This is a Rust workspace. Primary test surfaces already exist in:

1. inline module tests in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs), [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs), and [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
2. integration coverage in [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

### Code path coverage

```text
CODE PATH COVERAGE
==================
[+] state_store.rs
    |
    ├── [GAP] resolve_public_control_target() still treats stop as live-owner-only
    ├── [GAP] one shared session-control posture helper must feed reattach/stop/status
    ├── [★★★ TESTED, KEEP] pending inbox persistence updates authoritative pending count
    └── [★★ TESTED, KEEP] detached posture invariants already exist

[+] agents_cmd.rs
    |
    ├── [REGRESSION TEST REQUIRED] run_reattach() must not report success before durable attach
    ├── [REGRESSION TEST REQUIRED] run_stop() must stop parked durable sessions
    ├── [GAP] build_status_report() must surface parked sessions from authoritative session truth
    └── [★★ TESTED, KEEP] run_turn() preserves exact backend targeting

[+] async_repl.rs
    |
    ├── [★★ TESTED, KEEP] parked snapshot helpers already normalize parked vs attention posture
    ├── [GAP] retained-owner lifecycle needs a reattach stability proof point
    └── [GAP] any new parked-stop closeout helper must converge on one terminal session write path

[+] control.rs
    |
    ├── [★★ TESTED, KEEP] private stop transport exists for attached owners
    ├── [★★ TESTED, KEEP] Accepted -> Completed|Failed bridge shape exists
    └── [GAP] late-drop coverage must stay green while the durable control paths change
```

### User flow coverage

```text
USER FLOW COVERAGE
==================
[+] substrate agent start --backend <host> --prompt "hello" --json
    ├── [★★ TESTED, KEEP] clean bootstrap can land a parked durable session
    └── [GAP] parked session remains visible in status immediately after start

[+] substrate agent status --json
    ├── [GAP] parked_resumable session is shown as active durable truth
    └── [GAP] awaiting_attention session is shown when pending inbox work exists

[+] substrate agent turn --session <id> --backend <host> --prompt "next" --json
    ├── [★★ TESTED, KEEP] same parked session can resume prompt-taking
    └── [★★ TESTED, KEEP] detached world-member follow-up still fails closed

[+] substrate agent reattach --session <id> --json
    ├── [GAP] success means attached ownership actually restored
    └── [GAP] no immediate silent re-park after reported success

[+] substrate agent stop --session <id> --json
    ├── [GAP] attached stop still works
    └── [GAP] parked stop works without a live owner

[+] parked durable inbox responsibility
    ├── [GAP] detached runtime-alert/approval item lands durably
    ├── [GAP] posture changes to awaiting_attention
    └── [GAP] same exact session remains routable after inbox work appears

[+] post-Accepted bridge behavior
    └── [★★ TESTED, KEEP] helper drop after Accepted still yields explicit Failed
```

### Required test additions

1. Add a `reattach` regression proving the command only returns success once the session is durably `active_attached`.
2. Add a negative `reattach` regression proving immediate re-park or attach loss fails the command instead of returning false success.
3. Add a parked-stop regression proving `stop` works when no live owner process is attached.
4. Add an attached-stop regression to preserve current private transport behavior.
5. Add a status regression proving a parked durable session is visible in `agent status --json`.
6. Add a status regression proving a detached session with one pending inbox item appears as `awaiting_attention`.
7. Add an inbox-flow regression proving parked session -> inbox item -> attention posture -> reattach or turn still targets the same session.
8. Keep the broken-bootstrap and post-`Accepted` explicit-failure regressions green on the final merged tree.

### Test plan artifact to generate during implementation

When implementation starts, write a QA-facing artifact under `~/.gstack/projects/<slug>/` that covers:

1. `start`, `status`, `turn`, `reattach`, and `stop` on one exact durable session,
2. parked-empty versus awaiting-attention posture transitions,
3. attached-stop versus parked-stop behavior,
4. broken-bootstrap fail-closed behavior,
5. post-`Accepted` explicit-failure delivery,
6. detached-world fail-closed non-regression.

### Required validation commands

Run at minimum:

```bash
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell agent_runtime::control -- --nocapture
cargo test -p shell async_repl -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
```

Manual validation on the merged tree must prove the public contract, not just internal unit behavior:

```bash
# Flow A: parked-session visibility, inbox attention, turn, reattach, attached-stop
substrate agent start --backend <host_backend_id> --prompt "hello" --json
substrate agent status --json
<inject one durable inbox item onto the same session and capture persisted session truth>
substrate agent status --json
substrate agent turn --session <orchestration_session_id> --backend <host_backend_id> --prompt "next" --json
substrate agent reattach --session <orchestration_session_id> --json
substrate agent status --json
substrate agent stop --session <orchestration_session_id> --json
substrate agent status --json

# Flow B: parked-stop proof on a separate exact durable session
substrate agent start --backend <host_backend_id> --prompt "hello again" --json
substrate agent status --json
substrate agent stop --session <second_orchestration_session_id> --json
substrate agent status --json
```

Manual validation is complete only when all of the following are checked:

1. `start` leaves a non-terminal parked durable session when the clean bootstrap path is valid,
2. `status` shows that parked session before any follow-up owner is attached,
3. one detached inbox item moves the same session to `awaiting_attention`,
4. `turn` succeeds against that same exact session,
5. `reattach` succeeds only when attached ownership is truly restored,
6. `status` shows that same session as `awaiting_attention`,
7. persisted runtime truth after `reattach` is durably `active_attached` for that same exact session,
8. attached-session `stop` succeeds against that same exact session while still attached,
9. parked-session `stop` succeeds against a separate exact durable session with no attached owner,
10. broken bootstrap still fails as `runtime_start_failed`,
11. post-`Accepted` helper loss still renders explicit `Failed`,
12. detached-world follow-up still fails closed with reattach guidance.

## Failure Modes Registry

| Failure mode | Test required | Error handling required | User-visible outcome |
| --- | --- | --- | --- |
| `reattach` reports success while the session is still parked | yes | yes | explicit failure, not false active success |
| parked `stop` still returns `owner_unreachable` | yes | yes | parked durable session stops cleanly |
| parked session disappears from `status` | yes | yes | operator sees real parked session |
| detached inbox item lands but status stays `parked_resumable` | yes | yes | operator sees `awaiting_attention` |
| attached-stop and parked-stop produce different terminal persisted truth | yes | yes | one terminal session model, not two |
| post-`Accepted` helper drop goes silent | yes | yes | explicit `Failed` |
| detached world-member follow-up accidentally becomes permissive | yes | yes | explicit fail-closed rejection remains |

**Critical gaps this plan must close:** 5

1. false-success `reattach`,
2. live-owner-only `stop`,
3. parked-session invisibility in `status`,
4. unproven parked inbox operational ownership,
5. split control truth between session authority and transport liveness.

## Worktree Parallelization Strategy

This plan has limited real parallelism because the important work all converges on the same `crates/shell/src/execution/` and `crates/shell/src/execution/agent_runtime/` directories. The practical strategy is one code lane first, then split tests and docs once semantics are stable.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| 1. Freeze session-control posture contract | `crates/shell/src/execution/`, `crates/shell/src/execution/agent_runtime/` | — |
| 2. Reattach durability fix | `crates/shell/src/execution/`, `crates/shell/src/repl/`, `crates/shell/src/execution/agent_runtime/` | 1 |
| 3. Stop session-centric closeout | `crates/shell/src/execution/`, `crates/shell/src/execution/agent_runtime/`, `crates/shell/src/repl/` | 1, 2 |
| 4. Status plus inbox projection/proof | `crates/shell/src/execution/`, `crates/shell/src/execution/agent_runtime/` | 1, 3 |
| 5. Regression tests and manual fixtures | `crates/shell/tests/`, inline runtime test modules | 2, 3, 4 |
| 6. Docs closeout | `docs/`, `llm-last-mile/`, root plan docs | 2, 3, 4 |

### Parallel lanes

Lane A: 1 -> 2 -> 3 -> 4  
Sequential. Shared `execution/` and `agent_runtime/` ownership. Do not split this across workers unless you want merge-conflict theater.

Lane B: 5  
Starts after Lane A behavior stabilizes. Test harness work before that point is likely to chase moving semantics.

Lane C: 6  
Can run in parallel with Lane B after Lane A lands, because docs can key off the frozen merged behavior.

### Execution order

1. Launch Lane A first and keep it single-owner.
2. Once Lane A lands, launch Lane B and Lane C in parallel worktrees.
3. Merge Lane B first if docs need final command or output truth from the tests.

### Conflict flags

1. Steps 2, 3, and 4 all touch `crates/shell/src/execution/` and `crates/shell/src/execution/agent_runtime/`. Treat them as one lane.
2. If a test worker needs to modify helper code in `async_repl.rs` or `agents_cmd.rs`, pull that work back into Lane A instead of pretending it is independent.

## Deferred Follow-Ups

These were considered and are intentionally deferred:

1. Inbox browsing or approval UX, because this slice is about durable authority correctness, not operator productization.
2. Automatic stale-session timeout policy, because the truth record explicitly treats that as future deliberate lifecycle design.
3. World-member detached recovery broadening, because the current contract is intentionally fail-closed and that is not the bug here.
4. Broader parked-session telemetry and reporting polish, because correctness has to land before nicer visibility layers.

## Completion Checklist

The slice is not done until every item below is true on the merged tree:

1. one shared session-control posture helper governs `reattach`, `stop`, and parked-session `status`,
2. `reattach` success means durable `active_attached`, not “helper started,”
3. `stop` closes parked and attached sessions through one authoritative terminal closeout model,
4. `status --json` shows valid `parked_resumable` and `awaiting_attention` sessions from session truth,
5. a detached inbox item updates posture and stays tied to the same exact session,
6. the CLI regression wall proves `start -> status -> turn -> reattach -> stop` on one durable session,
7. broken bootstrap still fails closed,
8. post-`Accepted` helper loss still yields explicit `Failed`,
9. detached-world follow-up remains fail closed,
10. docs and nearby ASCII diagrams no longer describe an attached-live-only mental model.

## Completion Summary

- Step 0: Scope Challenge — complete, scope kept broad enough to finish the durable-session model instead of re-fixing only bootstrap
- Architecture — durable authority, control posture, and state-machine contracts frozen
- Implementation Plan — 7 workstreams with explicit dependencies and done criteria
- Engineering Guardrails — one posture helper, one stop closeout path, minimal-diff rules locked
- Test Strategy — coverage diagrams produced, 8 concrete gaps identified
- Failure modes — 5 critical gaps flagged
- NOT in scope — written
- What already exists — written
- Parallelization — 3 lanes, 1 real post-code parallel window
- Lake Score — the complete option was chosen throughout, because partial fixes would keep the durable-session model internally contradictory
