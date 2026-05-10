# PLAN: Host Orchestrator Durable Session And Parked-Resumable Ownership

Source SOW: [llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md)  
ADR anchor: [ADR-0047 — Host Orchestrator Durable Session and Parked-Resumable Ownership](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)  
Adjacent landed slices: [llm-last-mile/PLAN-20.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-20.md), [llm-last-mile/PLAN-22.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-22.md)  
Branch: `feat/macos-lima-shared-owner-member-runtime-parity`  
Base branch: `main`  
Plan type: shell runtime-state and lifecycle hardening, no UI scope  
Review posture: unified execution plan, tightened to `/autoplan` completeness and `/plan-eng-review` rigor  
Status: execution-ready planning pass on 2026-05-10

## Objective

Make host orchestration durable across clean prompt-owner exits without changing the public verb contract.

This slice is done only when all of the following are true:

1. `substrate agent start|turn|reattach|stop` keep their current public grammar and exact-selector rules.
2. The Substrate-owned orchestration session, not the lifetime of one attached Codex process, is the durable authority.
3. Canonical runtime state persists explicit host posture: `active_attached`, `parked_resumable`, `awaiting_attention`, or `terminal`.
4. World-originated approvals, completion notices, follow-up messages, and runtime alerts survive detached-host periods in a session-local durable inbox.
5. `turn` and `reattach` resume valid parked host sessions cleanly and fail closed everywhere else.
6. Once a public prompt request emits `Accepted`, the bridge always emits `Completed` or `Failed`. Silent EOF is a bug.

## Executive Decision

The public control surface is already right. The missing work is lifecycle truth.

This plan does not introduce a new control plane, queue product, or routing model. It hardens the existing shell runtime so the persisted session record, participant record, and session-local inbox become the only authority for whether a host orchestration session is live, resumable, attention-needed, or terminal.

The implementation is one cohesive slice with six ordered workstreams:

1. freeze the persisted contract for session posture and host attachment truth,
2. add a durable inbox with authoritative pending counts,
3. rewrite clean-detach lifecycle handling so valid host sessions park instead of invalidate,
4. harden `turn`, `reattach`, and the public prompt bridge around that durable truth,
5. pin behavior with tests first,
6. close docs only after the behavior is proven.

## Locked Starting State

### What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| Public start/turn/reattach/stop verbs | [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:301) | Reuse. No new verbs. |
| Exact public follow-up routing | [`resolve_public_turn_target(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:711) | Reuse as the only public selector seam. |
| Public prompt envelope types | [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:327) | Reuse. Tighten terminal guarantees. |
| Canonical session-root live state | [`AGENT_ORCHESTRATION_GAP_MATRIX.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:78) | Reuse. Extend the existing session root. |
| Atomic JSON write precedent | [`write_atomic_json(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:912) | Reuse for inbox and session mutations. |
| Detached-world fail-closed behavior | [`run_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:354) | Reuse exactly. Do not loosen it. |
| Public control integration harness | [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:1) | Reuse and extend. |
| Existing reattach lineage proof | [`public_reattach_and_fork_preserve_exact_session_and_lineage_contracts()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:1083) | Reuse. Add parked-session semantics on top. |
| Existing clean-exit invalidation regression | [`start_host_orchestrator_runtime_invalidates_when_attached_control_exits()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:7335) | Replace with parked-resumable behavior. |

### Exact remaining gap

1. [`OrchestrationSessionRecord`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:24) does not yet model parked versus attached versus attention-needed truth.
2. [`AgentRuntimeSessionInternal`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:81) still leans on `control_owner_retained`, `event_stream_active`, and `completion_observer_retained` as near-authoritative liveness signals.
3. The REPL regression [`start_host_orchestrator_runtime_invalidates_when_attached_control_exits()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:7335) documents the current bug directly.
4. There is no first-class inbox under `sessions/<orchestration_session_id>/inbox/`.
5. The public prompt bridge can still degrade into late stream disappearance instead of a guaranteed terminal envelope after `Accepted`.

## Scope Lock

### In scope

1. Add explicit host posture fields to canonical session state.
2. Add additive host attachment and resume-eligibility fields to canonical participant state.
3. Introduce `sessions/<orchestration_session_id>/inbox/<item_id>.json` as a canonical durable inbox.
4. Change clean attached-host exit from invalidation to `parked_resumable` or `awaiting_attention` when the session remains valid.
5. Make `turn` and `reattach` resume parked host sessions without fuzzy routing.
6. Tighten the `Accepted -> Completed|Failed` runtime invariant.
7. Add or replace targeted tests and focused docs.

### NOT in scope

1. New public verbs, fuzzy selectors, default routing, or public member selectors.
2. Redesigning Linux world-member follow-up away from `MemberTurnSubmitRequestV1`.
3. Making detached-world follow-up self-sustaining without a valid host owner.
4. A new daemon, background reconciler, or second state authority.
5. Config-schema or policy-schema changes.
6. Windows/WSL parity expansion or a redesign of macOS/Lima beyond additive compile-safe compatibility.
7. Rich operator inbox product surfaces such as `substrate agent inbox list`.
8. Time-based inbox compaction policy. This slice defines compaction eligibility only.

### Chosen approach

| Approach | Summary | Effort | Risk | Decision |
| --- | --- | --- | --- | --- |
| A. Extend canonical session and participant state, add session-local inbox, rewrite lifecycle around parked ownership | Boring, explicit, minimal new surface | Medium | Low | **Accepted** |
| B. Keep legacy fields authoritative and infer posture heuristically | Smaller diff, wrong model, brittle | Small | High | Rejected |
| C. Add a second ownership registry or external queue | More moving parts, split authority | High | High | Rejected |
| D. Treat clean exit as terminal and rely on reattach-only recovery hacks | Preserves the current bug in prettier words | Small | High | Rejected |

## Step 0: Scope Challenge

### 0A. What already solves part of this problem

1. Exact selector and fail-closed routing already exist in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:711).
2. Private owner transport and prompt streaming already exist in [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1063).
3. Resume-helper launch already exists in [`run_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:324) and [`run_reattach(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:384).
4. Canonical session-root persistence already exists and already uses atomic-write helpers in the state store.

The repo does not need a new routing model. It needs the existing model to stop lying about ownership.

### 0B. Minimum honest diff

The minimum complete fix spans these primary modules:

1. [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
2. [`crates/shell/src/execution/agent_runtime/session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
3. [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
4. [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
5. [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
6. [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
7. targeted tests and focused docs listed later in this plan

Anything smaller leaves lifecycle truth, inbox durability, or operator-visible terminal delivery ambiguous.

### 0C. Complexity check

This slice crosses more than eight files. That is a smell. It is still the right scope because the bug is inherently cross-seam:

1. storage truth,
2. lifecycle transitions,
3. public control semantics,
4. regression coverage,
5. docs truth.

The plan stays engineered enough by holding these lines:

1. no new crate,
2. no new public API family,
3. no background daemon,
4. no second source of truth,
5. no speculative generic queue framework.

### 0D. Search and reuse check

1. **[Layer 1]** Reuse the exact session-root layout already documented in the gap matrix and earlier session-store slices.
2. **[Layer 1]** Reuse the existing atomic JSON write pattern in the state store for all inbox and session mutations.
3. **[Layer 1]** Reuse the current `start|turn|reattach|stop` surface and exact detached-world fail-closed guidance.
4. **[EUREKA]** The missing abstraction is not a new abstraction. The missing abstraction is that `state` and `posture` are different concepts and must both be persisted explicitly.

### 0E. TODOS cross-reference

There is no root `TODOS.md` today. This review therefore records deferments in `NOT in scope` and `Deferred follow-ups` below instead of inventing a new repo convention mid-slice.

### 0F. Completeness check

Shortcut versions are not acceptable:

1. adding posture fields without changing clean-exit lifecycle is incomplete,
2. changing clean-exit lifecycle without a durable inbox is incomplete,
3. adding an inbox without authoritative pending counts is incomplete,
4. changing behavior without replacing the invalidation regression and public control tests is incomplete.

Boil the lake inside this blast radius. The extra cost is explicit fields and more tests, not a quarter of work.

## Frozen Runtime Contract

If implementation wants to violate any rule in this section, revise the plan first.

### Public command contract

| Command | Allowed behavior | Forbidden behavior |
| --- | --- | --- |
| `substrate agent start --backend <backend_id> --prompt ...` | Host-only root prompt-taking entrypoint | New selector forms, world-only root start |
| `substrate agent turn --session <id> --backend <backend_id> --prompt ...` | Exact follow-up turn against an existing orchestration session | Fuzzy routing, latest-session fallback, prompt-less recovery |
| `substrate agent reattach --session <id>` | Restore attached owner control only | Submit a prompt, consume inbox implicitly |
| `substrate agent stop --session <id>` | Explicit terminal shutdown | Soft-detach aliasing |

### State versus posture

1. `state` remains the orchestration lifecycle machine.
2. `posture` becomes the attached/resumable/attention summary.
3. `state` and `posture` are related but not interchangeable.
4. Routing decisions that care about resumability or attention must consult persisted posture and participant resume truth, not attachment diagnostics alone.

### Session invariants

1. `attached_participant_id != null` if and only if `posture == active_attached`.
2. `attached_participant_id == null` is required for `parked_resumable`, `awaiting_attention`, and `terminal`.
3. `pending_inbox_count > 0` requires `posture == awaiting_attention` whenever the session is non-terminal and no host client is attached.
4. `pending_inbox_count == 0` plus at least one authoritative host participant with `resume_eligible == true` requires `posture == parked_resumable` whenever the session is non-terminal and detached.
5. `terminal` posture must align with non-routable lifecycle state such as `Invalidated`, `Stopped`, or `Failed`.
6. `active_session_handle_id` keeps its compatibility meaning as the authoritative orchestrator participant. It is not proof of current attachment.

### Participant invariants

1. `control_owner_retained`, `event_stream_active`, and `completion_observer_retained` remain diagnostics only.
2. `uaa_session_id` is a correlation identifier, not proof of attachment or resumability.
3. A host participant may be `resume_eligible == true` while `attached_client_present == false`.
4. Member-runtime participants may leave host-only attachment fields unset or at safe defaults.

### Durable inbox invariants

1. Every unresolved world-originated orchestration event is persisted as one file under `sessions/<session>/inbox/<item_id>.json`.
2. `state == pending` contributes to `pending_inbox_count`.
3. Resolving an item must update live pending counts immediately.
4. Resolving an item must not immediately delete the file artifact.
5. Missing attached host owner must never silently drop or consume a pending item.

### Prompt bridge invariant

Once a public prompt request emits `Accepted`, the bridge may terminate only with:

1. `Completed`, or
2. `Failed`.

Anything else is a runtime bug and must be rendered as `Failed`.

## Authoritative Runtime-State Shape

### Canonical filesystem layout

```text
~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/
├── session.json
├── participants/
│   └── <participant_id>.json
├── leases/
│   └── <participant_id>.lease
└── inbox/
    └── <item_id>.json
```

Compatibility snapshots may still exist elsewhere for read-side compatibility, but this session root remains authoritative.

### Required additive session fields

1. `posture: active_attached|parked_resumable|awaiting_attention|terminal`
2. `posture_changed_at: <timestamp>`
3. `attached_participant_id: <participant_id>|null`
4. `pending_inbox_count: <u64>`
5. `last_parked_at: <timestamp>|null`
6. `last_attention_at: <timestamp>|null`
7. `parked_reason: <string>|null`

### Required additive participant fields

1. `attached_client_present: <bool>`
2. `last_attached_at: <timestamp>|null`
3. `last_detached_at: <timestamp>|null`
4. `detach_reason: <string>|null`
5. `resume_eligible: <bool>`

### Durable inbox minimum schema

1. `schema_version`
2. `item_id`
3. `orchestration_session_id`
4. `kind: approval_required|completion_notice|follow_up_message|runtime_alert`
5. `state: pending|acknowledged|dismissed`
6. `created_at`
7. `resolved_at`
8. `correlation`
9. `payload_schema`
10. `payload`

The correlation envelope remains additive and nullable. It is a join surface, not an authority surface.

## Architecture Review

### Architecture thesis

Persist the truth you want to route on.

Right now the runtime knows too much about whether one control-owner task is still retained and not enough about whether the session itself is still valid. The fix is to make the session row authoritative for host posture, then make lifecycle code update that row intentionally.

### Current broken lifecycle

```text
CURRENT HOST OWNERSHIP MODEL
============================
host owner attached
    |
    | clean backend exit
    v
attachment diagnostics go false
    |
    +--> session often treated as invalidated
    +--> no durable parked posture
    +--> no canonical inbox for detached-host follow-up
    \--> public durability contract is weaker than the CLI suggests
```

### Target lifecycle

```text
TARGET HOST OWNERSHIP MODEL
===========================
active_attached
    |
    | clean backend exit, runtime still valid
    v
parked_resumable
    |
    +--> inbox empty
    |       -> stay parked_resumable
    |
    +--> inbox item arrives while detached
    |       -> awaiting_attention
    |
    +--> operator runs `agent turn`
    |       -> helper resumes, prompt submitted, posture returns active_attached
    |
    +--> operator runs `agent reattach`
    |       -> helper resumes, no prompt submitted, posture returns active_attached
    |
    \--> explicit stop / fatal invalidation / unrecoverable failure
            -> terminal
```

### Transition matrix

| Event | Precondition | State effect | Posture effect |
| --- | --- | --- | --- |
| successful host start | session established | lifecycle becomes active | `active_attached` |
| clean host exit, session still valid, no pending inbox | active host participant exists | lifecycle stays non-terminal | `parked_resumable` |
| clean host exit, session still valid, pending inbox exists | pending inbox count > 0 | lifecycle stays non-terminal | `awaiting_attention` |
| detached world event arrives | session non-terminal, no host attached | no lifecycle promotion | `awaiting_attention`, count increments |
| `turn` resumes parked host | exact `(session, backend)` target resolves and participant is resume-eligible | lifecycle remains active | `active_attached` |
| `reattach` resumes parked host | session non-terminal and recovery metadata intact | lifecycle remains active | `active_attached` |
| explicit `stop` | operator requests shutdown | lifecycle becomes stopping/stopped | `terminal` |
| fatal runtime failure | ownership cannot be recovered safely | lifecycle becomes failed/invalidated | `terminal` |

### Component dependency graph

```text
DURABLE SESSION OWNERSHIP GRAPH
===============================
agents_cmd.rs
    |
    +--> state_store.rs
    |     |
    |     +--> orchestration_session.rs
    |     +--> session.rs
    |     \--> inbox persistence helpers
    |
    +--> control.rs
    |     |
    |     +--> private prompt bridge
    |     \--> terminal envelope invariant
    |
    \--> async_repl.rs
          |
          +--> owner lifecycle transitions
          \--> parked vs invalidated behavior
```

The plan stays boring by keeping all ownership truth inside existing shell runtime modules.

### Production failure scenarios

| Codepath | Real failure | Planned handling |
| --- | --- | --- |
| attached host exits cleanly | valid session is marked invalid and follow-up dies | convert to `parked_resumable`, preserve resumability metadata |
| detached session receives approval request | work is lost because no client is attached | persist inbox item and increment pending count |
| prompt bridge emits `Accepted` then helper disappears | operator sees a hung or truncated turn | force explicit `Failed` path and test it |
| stale participant diagnostics say not live | session is still valid but routing rejects it | route on explicit posture and resume eligibility, not transient booleans |
| two recovery paths race | duplicate attached owners or bad successor lineage | serialize resume-sensitive state updates through the existing state-store authority |
| resolved inbox item is deleted immediately | no audit trail, no postmortem correlation | keep resolved artifacts until later compaction eligibility |

## Detailed Execution Plan

### Workstream 1: Freeze the persisted posture contract

Files:

1. [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
2. [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Implement:

1. add an explicit posture enum with `active_attached`, `parked_resumable`, `awaiting_attention`, and `terminal`,
2. add the session fields listed in `Authoritative Runtime-State Shape`,
3. centralize session-level invariant validation so impossible `(state, posture, attached_participant_id, pending_inbox_count)` combinations fail fast in one place,
4. add helpers that recompute posture from authoritative session + participant + inbox truth instead of command-specific heuristics.

Acceptance criteria:

1. session rows can represent every state in the transition matrix without ambiguity,
2. no caller can produce `active_attached` with `attached_participant_id == null`,
3. no caller can produce `parked_resumable` or `awaiting_attention` with `attached_participant_id != null`,
4. a non-terminal detached session with pending inbox items always normalizes to `awaiting_attention`.

Do not do:

1. do not create a second persisted manifest,
2. do not overload `active_session_handle_id`,
3. do not spread posture recomputation across `agents_cmd.rs`, `control.rs`, and `async_repl.rs`.

### Workstream 2: Add host attachment and resume truth to participant state

Files:

1. [`crates/shell/src/execution/agent_runtime/session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
2. any participant validation helpers read from [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Implement:

1. add `attached_client_present`, `last_attached_at`, `last_detached_at`, `detach_reason`, and `resume_eligible`,
2. keep `control_owner_retained`, `event_stream_active`, and `completion_observer_retained` as diagnostics only,
3. add host-only validation so parked host participants remain valid when detached and resume-eligible,
4. keep member-runtime participants safe by default when host-only fields do not apply.

Acceptance criteria:

1. clean host detachment preserves participant resumability instead of looking terminal,
2. a participant can be detached and still be explicitly recoverable,
3. code that currently infers liveness from diagnostic booleans is either removed or demoted behind new helpers.

Do not do:

1. do not reinterpret `uaa_session_id` as proof of current attachment,
2. do not require member-runtime participants to mimic host-only attachment semantics.

### Workstream 3: Add the canonical durable inbox

Files:

1. [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
2. supporting runtime plumbing as needed in [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs) or [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Implement:

1. add `sessions/<orchestration_session_id>/inbox/<item_id>.json`,
2. define one canonical envelope for `approval_required`, `completion_notice`, `follow_up_message`, and `runtime_alert`,
3. persist pending items atomically with authoritative `pending_inbox_count` updates,
4. support `pending`, `acknowledged`, and `dismissed`,
5. keep correlation metadata additive and nullable,
6. make resolved items non-pending immediately but still inspectable until later compaction.

Acceptance criteria:

1. detached host does not lose world-originated work,
2. live pending count is O(1) to read,
3. a resolved item disappears from pending posture calculations without deleting the audit artifact.

Do not do:

1. do not build a generic job system,
2. do not make steady-state reads rescan the whole inbox directory,
3. do not delete resolved items synchronously as part of normal command execution.

### Workstream 4: Rewrite clean-detach and resume semantics

Files:

1. [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
2. [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
3. [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
4. any read-side helpers in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Implement:

1. clean prompt-owner exit after successful session establishment transitions to `parked_resumable`, not automatic invalidation,
2. detached session plus pending inbox work transitions to `awaiting_attention`,
3. `turn` against a parked host session may restore a helper and submit the prompt,
4. `reattach` against a parked host session may restore the helper without submitting a prompt,
5. detached world follow-up stays fail closed and still instructs the operator to reattach first,
6. explicit `stop`, invalidation, and unrecoverable runtime failure still end in `terminal`.

Acceptance criteria:

1. the old clean-exit invalidation regression is replaced with parked-session behavior,
2. parked host `turn` and parked host `reattach` both stay inside the same orchestration session,
3. exact `(session, backend)` resolution remains the only public routing contract,
4. no valid recovery path depends on reviving diagnostic booleans alone.

Do not do:

1. do not make `reattach` a prompt shortcut,
2. do not add fuzzy backend recovery,
3. do not let detached-world routing piggyback on parked-host semantics.

### Workstream 5: Harden the terminal envelope contract

Files:

1. [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
2. integration tests in [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

Implement:

1. any path that already emitted `Accepted` must end in an explicit terminal envelope,
2. post-`Accepted` EOF becomes a hard failed path with a rendered `Failed` envelope,
3. parking or detachment may exist as runtime-state changes around the stream, but never as a missing terminal envelope.

Acceptance criteria:

1. operator-facing public commands never hang in a silent post-`Accepted` steady state,
2. transport disappearance after `Accepted` is rendered deterministically as failure,
3. tests prove both success and late-failure paths.

Do not do:

1. do not treat EOF as a soft operator-visible outcome,
2. do not let attachment-state transitions bypass terminal envelope emission.

### Workstream 6: Docs closeout after behavior is real

Likely docs:

1. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
2. [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)
3. [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md), only if it describes runtime ownership or session roots
4. this slice's SOW or ADR, only if implementation reveals wording drift

Acceptance criteria:

1. docs describe the same authority model as the code,
2. operator-facing docs explain parked host versus detached world clearly,
3. stale diagrams near touched code are updated in the same change.

Do not do:

1. do not lead the change with docs,
2. do not freeze wording for behavior that has not been proven by tests.

## Code Quality Review

### DRY and explicitness rules

1. Centralize posture recomputation in one place. Do not duplicate "detached plus pending count means awaiting attention" across command, lifecycle, and store code.
2. Centralize inbox counter maintenance. Do not let every caller increment and decrement pending counts ad hoc.
3. Prefer additive helpers inside existing modules over a new abstraction layer.
4. Keep attachment diagnostics as diagnostics. Do not silently promote them back into authority through helper call sites.

### Minimal-diff rules

1. Extend existing session and participant structs instead of introducing parallel persisted models.
2. Reuse the state store for path construction, validation, and atomic writes.
3. Extend existing test suites before creating a new giant one-off durability target.

### Inline diagram candidates

If implementation adds non-obvious transition logic, add or update nearby ASCII comments in:

1. [`orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs) for the `state` versus `posture` relationship,
2. [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) for owner-exit transition flow,
3. [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs) for the public prompt envelope lifecycle.

## Test Review

### Test framework

This is a Rust workspace. Primary test surfaces already exist in:

1. module tests in [`orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs), [`session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:595), [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1715), and [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1980),
2. integration coverage in [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs),
3. REPL/runtime coverage in [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).

### Code path coverage

```text
CODE PATH COVERAGE
==================
[+] orchestration_session.rs
    |
    ├── [GAP] posture enum + transition helpers
    ├── [GAP] illegal `(state, posture)` combination guards
    └── [GAP] `pending_inbox_count` and `attached_participant_id` invariants

[+] session.rs
    |
    ├── [GAP] participant attached/detached/resume-eligible fields
    ├── [GAP] host-only validation rules
    └── [GAP] member-safe defaults

[+] state_store.rs
    |
    ├── [GAP] inbox path creation + atomic persistence
    ├── [GAP] pending count recomputation/update rules
    ├── [GAP] parked/attention posture reads are authoritative
    └── [GAP] exact public turn routing against parked host sessions

[+] async_repl.rs
    |
    ├── [REGRESSION TEST REQUIRED] clean owner exit parks instead of invalidates
    ├── [GAP] detached + pending item transitions to awaiting_attention
    └── [GAP] terminal causes still invalidate/stop/fail correctly

[+] control.rs
    |
    ├── [GAP] `Accepted` always ends with `Completed` or `Failed`
    ├── [GAP] post-`Accepted` stream EOF renders `Failed`
    └── [GAP] parked ownership does not masquerade as silent transport loss

[+] agents_cmd.rs / public surface
    |
    ├── [GAP] `turn` resumes valid parked host session
    ├── [★★ TESTED, KEEP] detached world follow-up still fails closed
    └── [GAP] `reattach` restores ownership without submitting a prompt
```

### Operator flow coverage

```text
USER FLOW COVERAGE
==================
[+] Operator runs `substrate agent start --backend cli:codex --prompt "hello" --json`
    ├── [GAP] clean owner exit after successful establishment parks, not invalidates
    └── [GAP] follow-up recovery stays inside same orchestration session

[+] Operator runs `substrate agent turn --session <id> --backend cli:codex --prompt "next" --json`
    ├── [GAP] parked host resumes and accepts prompt
    ├── [★★ TESTED, KEEP] detached world follow-up rejected with reattach guidance
    └── [GAP] terminal host session rejected as terminal, not stale or ambiguous

[+] Operator runs `substrate agent reattach --session <id> --json`
    ├── [★★ TESTED, KEEP] lineage and exact-session preservation
    └── [GAP] no prompt submission side effect

[+] World-originated event arrives while host detached
    ├── [GAP] inbox item persisted
    ├── [GAP] pending count increments
    └── [GAP] posture becomes awaiting_attention

[+] Prompt bridge after `Accepted`
    ├── [GAP] helper transport EOF returns explicit `Failed`
    └── [GAP] no silent stream termination path remains
```

### Required test additions

1. Replace the current invalidation regression in [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:7335) with a parked-resumable regression.
2. Add unit tests in [`orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs) for legal posture/state combinations and impossible-combination rejection.
3. Add unit tests in [`session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:595) for host participant attachment/resume invariants.
4. Add unit tests in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1715) for inbox persistence, pending-count updates, and authoritative parked/attention reads.
5. Add control-path tests in [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1980) for `Accepted -> Failed` on bridge EOF.
6. Extend [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) to prove parked-host `turn`, parked-host `reattach`, detached-world non-regression, and explicit terminal-envelope behavior.
7. Keep world-first routing non-regression from `PLAN-22` green if any public follow-up behavior shares the same retained lineage assumptions.

### Test plan artifact requirement

Implementation should also write a human-readable eng-review test plan artifact under `~/.gstack/projects/<slug>/...` if the normal gstack flow is used, but the required content is already captured here:

1. parked host recovery,
2. detached-host inbox durability,
3. post-`Accepted` late-failure rendering,
4. detached-world fail-closed non-regression,
5. exact-session and exact-backend recovery.

### Required validation commands

Run at minimum:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell -- --nocapture
cargo test --workspace -- --nocapture
```

If docs or runtime-state schema wording changes surface broader coupling, also run:

```bash
substrate agent status --json
substrate agent doctor --json
```

## Performance Review

There is no obvious throughput problem here, but there are two real footguns:

1. recomputing inbox state by scanning the inbox directory on every public `turn` or `status` call,
2. turning every detached-host transition into a broad participant/session directory rescan.

Performance rules:

1. persist `pending_inbox_count` and keep it authoritative so read paths stay O(1) for posture summaries,
2. use directory scans for recovery and validation, not as the steady-state codepath for every command,
3. keep inbox writes one-file-per-item and atomic, but avoid unnecessary full-session rewrites when only an inbox item changes.

## Failure Modes Registry

| Failure mode | Test required | Error handling required | User-visible outcome |
| --- | --- | --- | --- |
| clean host exit invalidates live session | yes | yes | session stays resumable, not dead |
| detached host receives world approval | yes | yes | pending inbox item, awaiting-attention posture |
| post-`Accepted` bridge EOF | yes | yes | explicit `Failed`, never silent hang |
| parked host `turn` resumes wrong backend | yes | yes | exact backend failure, no fuzzy recovery |
| detached world follow-up slips through | yes | yes | explicit fail-closed rejection with reattach guidance |
| resolved inbox item deleted immediately | yes | yes | retained artifact remains inspectable |

**Critical gaps this plan must close:** 3

1. clean-exit invalidation,
2. no canonical durable inbox,
3. post-`Accepted` silent disappearance.

## Worktree Parallelization Strategy

This slice has one real parallel window, but only after the schema contract is frozen.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| 1. Posture and participant contract freeze | `crates/shell/src/execution/agent_runtime/` | — |
| 2. Durable inbox persistence | `crates/shell/src/execution/agent_runtime/` | 1 |
| 3. Lifecycle and public resume semantics | `crates/shell/src/repl/`, `crates/shell/src/execution/agents_cmd.rs`, `crates/shell/src/execution/agent_runtime/control.rs` | 1 |
| 4. Validation wall | `crates/shell/tests/`, unit-test modules in touched runtime files | 2, 3 |
| 5. Docs closeout | `docs/`, `AGENT_ORCHESTRATION_GAP_MATRIX.md`, `llm-last-mile/` | 4 |

### Parallel lanes

Lane A: step 1 -> step 2  
Reason: shared ownership of canonical persisted truth inside `agent_runtime/`.

Lane B: step 1 -> step 3  
Reason: after the posture contract is frozen, lifecycle and public control changes can proceed mostly in `async_repl.rs`, `agents_cmd.rs`, and `control.rs`.

Lane C: step 4 -> step 5  
Reason: closeout lane only after A and B merge. Tests and docs should validate merged behavior, not guess at it.

### Execution order

1. Land or freeze step 1 first. This is the shared contract that keeps everything else from drifting.
2. Launch Lane A and Lane B in parallel worktrees only after step 1 is stable.
3. Merge A and B.
4. Run Lane C on the merged tree for the validation wall and late docs closeout.

### Conflict flags

1. `crates/shell/src/execution/agent_runtime/session.rs` is a conflict hotspot. If Lane B needs to edit participant helpers directly, assign that file to Lane A or run sequentially.
2. `crates/shell/tests/agent_public_control_surface_v1.rs` is a closeout-only file. Do not let both parallel lanes edit it heavily at the same time.
3. If inbox persistence helpers end up living inside `control.rs` instead of the state store, the A/B split is no longer clean. In that case, collapse to sequential execution.

## Implementation Checklist

1. Add explicit posture fields to `OrchestrationSessionRecord`.
2. Add additive attachment and resume metadata to host participant records.
3. Add durable inbox path helpers and canonical inbox envelope persistence.
4. Update lifecycle code so clean owner exit parks rather than invalidates when the session remains valid.
5. Update public `turn` and `reattach` to operate against explicit parked ownership truth.
6. Pin the prompt-bridge terminal invariant after `Accepted`.
7. Replace the existing clean-exit invalidation regression with parked-resumable assertions.
8. Extend public control integration tests for parked-host resume and detached-host inbox semantics.
9. Update focused docs after the merged tree proves the behavior.

## Deferred Follow-Ups

These are real ideas, not part of this PR:

1. Numeric inbox retention duration and background compaction policy.
2. Broader operator-facing inbox inspection commands.
3. Windows/WSL and broader macOS parity improvements for the same durability model.

## Completion Summary

- Step 0: Scope Challenge, scope accepted as-is, with a strict no-new-control-plane rule.
- Architecture Review: one architectural direction, explicit session-owned durability over process-owned liveness.
- Code Quality Review: centralize posture and inbox logic, no second authority.
- Test Review: coverage diagram produced, concrete gaps identified across state, lifecycle, routing, and terminal delivery, 3 critical.
- Performance Review: no throughput rewrite needed, but avoid steady-state directory rescans.
- NOT in scope: written.
- What already exists: written.
- Failure modes: 3 critical gaps flagged.
- Parallelization: 3 lanes total, 1 real parallel window after contract freeze.
- Lake Score: 10/10 recommendations choose the complete option over the shortcut.
