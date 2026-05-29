# Design: Durable Orchestration Review and Inbox Projection

Status: draft design input. This document no longer treats notifications as the canonical durable artifact. In the greenfield architecture, the canonical durable artifact is the obligation ledger defined in [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md). This document freezes the review-facing projection of those obligations: inbox rendering, review lifecycle, attention semantics, and explicit resolution rules.

## Why This Doc Exists

The obligation-ledger design defines the durable source of truth.

What still needs a dedicated contract is the review-facing projection that answers:

1. what the host or operator sees,
2. how unresolved obligations become inbox items,
3. how `awaiting_attention` is derived,
4. how acknowledgement, resolution, and dismissal work.

Without a dedicated review/inbox projection:

1. the UI/CLI/read-side model will drift from the obligation ledger,
2. `awaiting_attention` will become ambiguous,
3. later host review paths will invent ad hoc resolution semantics.

## Relationship To Existing Decisions

This design composes with:

1. [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md): canonical durable artifact and shared substate model.
2. [ADR-0047](../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md): host-session durable authority and attention posture.
3. [23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](./23-host-orchestrator-durable-session-and-parked-resumable-ownership.md): deferred host review and exact-session `reattach`.
4. [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md): upstream worker event classes that may create obligations.
5. [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md): worker `attention_pending` remains distinct from host-session `awaiting_attention`.

## Problem Statement

How might Substrate project obligations into a durable inbox/review model so that:

1. host review works with no attached client,
2. `awaiting_attention` is authoritative and durable,
3. review lifecycle is explicit,
4. review state stays separate from attach-processing state,
5. no hidden prompt injection or implicit continuation occurs.

## Frozen Direction

This design freezes the following:

1. inbox items are review projections over obligations, not separate durable truth,
2. `awaiting_attention` is driven by unresolved attention-driving obligations,
3. acknowledgement, resolution, and dismissal mutate review state on the obligation,
4. review state and attach state are separate substates of the same obligation,
5. absence of an attached host client does not consume or clear obligations.

## Non-Goals

This design does not:

1. define the canonical durable artifact itself,
2. define router/daemon attach-processing behavior,
3. define the public inbox CLI or UI surface,
4. define live-stream transport encoding for active turns.

## Core Principle

Inbox rows are not prompts.

They are read-side views of obligations that summarize:

1. what happened,
2. who caused it,
3. whether host review is required,
4. whether the obligation is still unresolved.

The host may inspect those obligations through sanctioned control paths, but the inbox projection itself must not act as implicit prompt submission or hidden session resume.

## Canonical Projection Rule

The inbox is derived from the obligation ledger.

That means:

1. each inbox row corresponds to one obligation,
2. `obligation_id` is the canonical join key,
3. no inbox-only artifact may become more authoritative than the underlying obligation.

## Review Lifecycle

The review-facing lifecycle is the obligation `review_state`:

1. `unread`
2. `acked`
3. `resolved`
4. `dismissed`

### Allowed transitions

```text
unread -> acked
unread -> resolved
unread -> dismissed

acked -> resolved
acked -> dismissed
```

No transition back to `unread` is allowed.

## Obligation Kinds That Surface In Inbox Views

The review projection should support at minimum:

1. `follow_up_required`
2. `approval_required`
3. `blocked`
4. `task_completed`
5. `task_failed`
6. `runtime_alert`
7. `fork_request`
8. `fork_recommendation`
9. `escalation_recommended`
10. `result_available`

## Attention-Driving Rule

The host orchestration session enters or remains in `awaiting_attention` when at least one obligation exists with:

1. `attention_required=true`, and
2. `review_state` not in `resolved|dismissed`.

### Attention-driving kinds by default

1. `follow_up_required`
2. `approval_required`
3. `blocked`
4. `runtime_alert`
5. `fork_request`

### Usually non-attention-driving by default

1. `task_completed`
2. `task_failed`
3. `fork_recommendation`
4. `result_available`

## Producer Rule

Workers and runtime components create obligations, not inbox-only rows.

The inbox projection simply reflects those obligations for host review.

## Consumer Model

Consumers are sanctioned host-side review/control paths.

### Consumers may:

1. list reviewable obligations for an orchestration session,
2. acknowledge them,
3. resolve them by taking explicit action,
4. dismiss them under policy,
5. attach resolution metadata.

### Consumers may not:

1. auto-submit summaries as prompts to the host backend,
2. silently delete unresolved obligations,
3. treat trace-only history as equivalent to obligations,
4. resolve obligations without explicit causal action or allowed runtime policy.

## Host Reattach and Deferred Review

The absence of an attached host client must not:

1. drop obligations,
2. consume obligations,
3. invalidate the orchestration session,
4. force replay through synthetic bootstrap prompts.

Instead:

1. obligations remain durable,
2. the host session may move to `awaiting_attention`,
3. later `reattach` restores a live host client,
4. sanctioned host-side logic may inspect and work through the obligations.

## Mapping From Worker Events To Obligation Kinds

Required minimum mapping:

1. `follow_up_question` -> `follow_up_required`
2. `approval_request` -> `approval_required`
3. `blocked` -> `blocked`
4. `result` with durable review value -> `result_available`
5. `failure` -> `task_failed`
6. `fork_request` -> `fork_request`
7. `fork_recommendation` -> `fork_recommendation`
8. `needs_retained_followup` -> `escalation_recommended`

Not every streamed event must produce a durable obligation.

## Resolution Semantics

Examples:

1. answering a `follow_up_required` obligation through sanctioned host messaging should resolve that obligation,
2. explicitly granting or denying an `approval_required` obligation should resolve that obligation,
3. accepting a `fork_request` should create a causal control-plane action and then resolve that obligation,
4. dismissing a `fork_recommendation` should close review state without implying child-worker allocation.

## Interaction With Auto-Attach

Auto-attach is not part of the inbox projection.

It is a separate substate on the same obligation, defined by the auto-attach projection doc.

That means:

1. completing attach work does not itself resolve review state,
2. resolving review state may cancel future attach work if the obligation becomes non-actionable,
3. both flows share one canonical `obligation_id`.

## Summary

The review/inbox contract frozen here is:

1. the inbox is a projection over obligations,
2. `awaiting_attention` derives from unresolved attention-driving obligations,
3. review lifecycle is explicit and durable,
4. no inbox operation implicitly submits prompts or resumes host work.
