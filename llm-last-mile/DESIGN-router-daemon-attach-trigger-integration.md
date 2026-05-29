# Design: Router Daemon Attach-Trigger Integration

Status: draft design input. This document defines how the router/daemon layer watches durable orchestration obligations, evaluates auto-attach eligibility, coalesces session-scoped attach work, chooses continuity versus fresh attach, and records outcomes without bypassing host-session authority. It is not a public CLI design and it does not redefine the canonical durable artifact. The canonical durable truth remains the obligation ledger defined in [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md).

## Why This Doc Exists

The current design stack already freezes:

1. one canonical obligation ledger,
2. inbox/review as a projection over obligations,
3. attach-processing as a projection over obligations,
4. exact-session manual `reattach`,
5. the requirement that pending work may auto-trigger host attach.

What is still missing is the integration contract for the router/daemon that turns those decisions into one coherent execution model.

Without this document:

1. no component is clearly responsible for watching obligations,
2. attach eligibility and claim semantics may drift,
3. multiple obligations in one session may fan out multiple attach launches,
4. continuity versus fresh attach selection may be re-derived ad hoc,
5. loop prevention and failure recording will be underspecified.

This document closes that gap.

## Relationship To Existing Decisions

This design composes with:

1. [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md): canonical durable artifact and attach/review substates.
2. [DESIGN-auto-attach-trigger-and-work-queue-contract.md](./DESIGN-auto-attach-trigger-and-work-queue-contract.md): obligation eligibility and attach-state lifecycle.
3. [DESIGN-durable-orchestration-notification-inbox-contract.md](./DESIGN-durable-orchestration-notification-inbox-contract.md): review projection and `awaiting_attention` derivation.
4. [ADR-0047](../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md): exact-session durable authority and sanctioned `reattach`.
5. [29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md](./29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md): persisted host attach truth remains authoritative and attach-time overlays remain narrowing-only.
6. [31-lazy-host-attach-for-host-rooted-world-start.md](./31-lazy-host-attach-for-host-rooted-world-start.md): lazy attach must use real persisted launch truth and real pending context, not synthetic prompts.
7. [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md): auto-attach remains deny-by-default and policy-gated by obligation kind and exact boundaries.
8. [archive/LLM_AI_CAPABILITY_ENABLEMENT_PLANNING_ORDER.md](../archive/LLM_AI_CAPABILITY_ENABLEMENT_PLANNING_ORDER.md): the earlier router-daemon track wanted durable request-queue semantics; in the greenfield model, those semantics now live as attach-processing state inside obligations rather than in a second canonical queue ledger.

## Problem Statement

How might Substrate integrate a router/daemon with the obligation ledger so that:

1. eligible obligations can trigger host attach automatically,
2. one orchestration session produces at most one active attach launch at a time,
3. attach mode selection is deterministic and auditable,
4. obligations remain unresolved until the host explicitly works them,
5. loops, duplicate launches, and wrong-host claims fail closed?

## Frozen Direction

This design freezes the following:

1. a local router/daemon watches local obligations and local session truth,
2. the router evaluates eligibility from durable obligation state, not raw stream events,
3. attach launch is session-scoped even though eligibility is obligation-scoped,
4. the router must coalesce multiple eligible obligations for the same session into one attach episode,
5. continuity attach is preferred when a valid continuity selector exists,
6. fresh attach is the fallback only when continuity is unavailable but persisted attach truth remains valid,
7. attach success restores a host execution client; it does not resolve obligations automatically,
8. any future host-global inbox or remote ingress layer must feed local obligations first; the local router still acts on local obligations only.

## Non-Goals

This design does not:

1. define the public `reattach` CLI surface,
2. define the final on-disk storage watcher implementation,
3. define retry/backoff timing constants,
4. define multi-host federation protocols in full,
5. authorize direct worker continuation without restoring sanctioned host ownership.

## Core Principle

The router does not process prompts.

It processes attach-eligible obligations.

Its job is:

1. observe,
2. evaluate,
3. claim,
4. attach,
5. record.

It does not:

1. resolve obligations,
2. answer worker questions,
3. approve work,
4. continue world workers on behalf of the host.

## Router Ownership

The v1 router/daemon should be a host-side control-plane component with responsibility for local orchestration-session attach recovery.

### Responsibilities

1. watch local obligations and session posture,
2. detect newly eligible attach obligations,
3. coalesce eligibility at the orchestration-session level,
4. claim one attach-processing obligation for the session,
5. execute sanctioned attach handling,
6. write attach-state outcomes back to the obligation ledger,
7. emit explanation-ready trace or event records for each decision.

### Non-responsibilities

1. no hidden prompt submission,
2. no host-review resolution,
3. no world-worker steering decisions,
4. no worker-fork acceptance on its own.

## Watch Scope

The router watches local durable state only.

### Inputs

1. local orchestration-session records,
2. local obligation ledger entries,
3. authoritative attached-participant truth,
4. authoritative persisted host attach contract truth,
5. authoritative continuity-selector truth when present.

### Future host-global layering rule

If a later host-global inbox or remote ingress layer exists under `SUBSTRATE_HOME`, it must first materialize local obligations before the router acts.

The router must not consume remote/global ingress records directly as if they were local obligations.

## Eligibility Evaluation

The router should evaluate attach eligibility for an obligation when any of the following change:

1. a new obligation is created,
2. an obligation review state changes,
3. an obligation attach state changes,
4. host attachment truth changes,
5. session posture changes,
6. continuity-selector truth changes.

### Required per-obligation checks

An obligation is eligible only when:

1. `review_state` is not `resolved|dismissed`,
2. `attach_state` is `not_requested|queued`,
3. `attention_required` or obligation-kind policy permits auto-attach,
4. the session is non-terminal,
5. no authoritative attached host participant exists,
6. exact boundary truth is valid,
7. the obligation is targeted at this host if `target_host_id` is present.

### Wrong-host rule

If `target_host_id` is present and does not match the local host identity, this router must not claim the obligation.

The safest default is:

1. leave it untouched if another host is expected to act, or
2. mark attach processing `dead_letter` with explanation-ready wrong-host reason if local materialization itself is invalid.

## Session-Level Coalescing

Attach launch is session-scoped.

That means multiple obligations in one orchestration session must not fan out multiple concurrent attach launches.

### Hard rule

At most one obligation may be `attach_state=claimed` for a given `orchestration_session_id` at a time.

### Coalescing behavior

1. multiple obligations may be eligible in the same session,
2. the router selects one as the claim leader for the next attach episode,
3. sibling eligible obligations remain unresolved and may remain `queued` or `not_requested` depending on implementation detail,
4. successful host attach for the session satisfies the session-scoped attach need once,
5. sibling attach-processing states should then converge to `cancelled` or remain `not_requested` with explicit causation pointing at the successful attach episode.

### Claim leader selection

When multiple obligations are eligible in the same session, the router should choose deterministically by:

1. highest attention/urgency class first,
2. then oldest `created_at`,
3. then stable `obligation_id` order.

This keeps attach behavior explanation-ready and avoids flaky selection.

## Claiming Contract

Claiming attach work means transitioning one obligation for the session from `queued` to `claimed`.

### Required claim checks

Before claiming, the router must re-check:

1. the session is still non-terminal,
2. no attached host participant now exists,
3. no sibling obligation in the same session is already `claimed`,
4. the obligation remains unresolved,
5. the obligation remains policy-eligible.

### Required claim metadata

On successful claim, the router should write:

1. `attach_state=claimed`
2. `attach_claim_owner=<router identity>`
3. increment `attach_attempt_count`
4. `attach_last_attempt_at=<timestamp>`

## Attach Mode Selection

The router must choose exactly one attach mode per attach episode.

### 1. Continuity attach

Use when:

1. a persisted continuity selector exists,
2. it still points to a valid backend-native session,
3. the selector is consistent with the authoritative persisted attach contract,
4. using continuity does not broaden persisted policy/capability truth.

### 2. Fresh attach

Use when:

1. no valid continuity selector exists, and
2. the persisted host attach contract is present and valid for launching a new host client.

### Hard rules

1. continuity is preferred over fresh attach when valid,
2. fresh attach must not be chosen just because continuity probing is inconvenient,
3. neither path may synthesize a bootstrap prompt,
4. neither path may invent a new durable attach contract.

## Attach Execution Boundary

The router does not perform business logic after attach.

Its attach execution boundary is:

1. restore or launch the sanctioned host execution client,
2. restore attached-host ownership truth for the session,
3. stop.

After that:

1. the host reviews obligations,
2. the host decides how to steer world work,
3. obligations resolve only through sanctioned host actions or allowed runtime policy.

## Success Semantics

Attach processing succeeds when:

1. a sanctioned host execution client is attached for the correct orchestration session,
2. authoritative attached-participant truth is restored,
3. the attach mode and target backend are explanation-ready.

### On success

The claimed obligation should transition to:

1. `attach_state=completed`
2. `attach_completion_reason=session_attach_restored`

Sibling eligible obligations in the same session should transition to:

1. `attach_state=cancelled`, or
2. remain `not_requested`

with explicit reason that the session-scoped attach need was already satisfied.

### Important rule

Successful attach must not auto-resolve any obligation review state.

## Failure Semantics

Attach processing fails closed.

### Dead-letter class

Move to `dead_letter` when:

1. persisted attach truth is missing or invalid,
2. required boundary truth is disproven,
3. the obligation is targeted at the wrong host,
4. policy denies the action,
5. both continuity and fresh attach are unavailable under current truth.

### Cancelled class

Move to `cancelled` when:

1. manual `reattach` already succeeded,
2. another obligation's attach episode already restored session attachment,
3. the obligation resolved or was dismissed before attach completed,
4. an attached host participant appeared through another sanctioned path.

## Loop Prevention

The router must not thrash.

### Hard loop-prevention rules

1. no second attach launch while the session already has an attached host participant,
2. no second attach launch while a sibling obligation in the same session is `claimed`,
3. no automatic retry from `completed`,
4. no automatic retry from `dead_letter` without explicit later state change,
5. no host attach launch purely because `awaiting_attention` remains true after a successful attach.

### Why the last rule matters

After successful attach, obligations may remain unresolved for some time while the host works them.

That must not itself re-trigger another attach episode.

## Manual Reattach Interaction

Manual exact-session `reattach` and router-owned auto-attach share the same authority model.

### Required behavior

1. if manual `reattach` succeeds before claim, eligible auto-attach processing must cancel,
2. if manual `reattach` succeeds during a claim, the router must converge the claimed obligation to `cancelled` or `completed` without duplicate launch,
3. manual `reattach` must not create a second attach episode,
4. automatic attach must not bypass exact-session selectors.

## Observability And Trace Requirements

Every router decision must be explanation-ready.

The router should emit enough structured data to answer:

1. which obligation was evaluated,
2. which session it belonged to,
3. why it was eligible or ineligible,
4. whether it lost a coalescing race to a sibling obligation,
5. which attach mode was selected,
6. whether attach succeeded, cancelled, or dead-lettered,
7. what exact host/backend/session identity was used.

### Minimum join fields

1. `obligation_id`
2. `orchestration_session_id`
3. `source_participant_id` when known
4. `target_backend_id`
5. `world_id`
6. `world_generation`
7. selected attach mode
8. router identity / claim owner

## Recommended Derived Event Families

The final trace schema belongs elsewhere, but the router contract should preserve room for derived events conceptually equivalent to:

1. `obligation_attach_evaluated`
2. `obligation_attach_claimed`
3. `obligation_attach_mode_selected`
4. `obligation_attach_succeeded`
5. `obligation_attach_cancelled`
6. `obligation_attach_dead_lettered`
7. `session_attach_coalesced`

## Interaction With Slice 31

This router contract sharpens the pending slice-31 rewrite.

Slice 31 should now assume:

1. born-unattached or detached host-rooted sessions may hold unresolved obligations,
2. the router may auto-trigger attach from those obligations,
3. the attach worker chooses continuity or fresh attach from persisted attach truth,
4. public world follow-up still remains fail-closed until host attachment is actually restored.

## v1 Summary

The first shippable router/daemon attach-trigger integration should therefore be:

1. local-obligation driven,
2. session-coalescing,
3. exact-identity and fail-closed,
4. continuity-first with fresh-attach fallback,
5. non-resolving with respect to obligation review state,
6. explanation-ready in trace and durable state.
