# Design: Durable Orchestration Obligation Ledger

Status: draft design input. This document defines the canonical durable artifact for deferred host review, attention posture, and later auto-attach processing. It replaces the split mental model where durable notifications are one authority and a separate attach queue is another. In the greenfield direction frozen here, the durable truth is one obligation ledger under the orchestration session, with inbox/review and auto-attach represented as projections or substates of the same artifact.

## Why This Doc Exists

The current design stack already froze:

1. durable host-session authority,
2. retained worker lifecycle and messaging semantics,
3. exact-identity host-to-world steering,
4. the requirement that auto-trigger attach from pending work is necessary.

What was still unstable was the durable artifact model behind those decisions.

The earlier draft direction split the problem into:

1. one durable notification/inbox artifact,
2. plus a second durable attach work-queue artifact.

That split is workable, but it is not the cleanest greenfield architecture because it creates two ledgers for one orchestration obligation.

This document closes that gap by freezing one canonical durable ledger instead.

## Relationship To Existing Decisions

This design composes with:

1. [ADR-0047](../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md): the orchestration session remains the durable authority root.
2. [23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](./23-host-orchestrator-durable-session-and-parked-resumable-ownership.md): deferred host review, durable parked ownership, and exact-session `reattach` remain canonical product intent.
3. [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md): exact session/backend/world identity and control-plane causation fields remain authoritative.
4. [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md): worker events such as `follow_up_question`, `approval_request`, `blocked`, `failure`, and `fork_request` are the upstream producers that may create obligations.
5. [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md): retained workers may enter `attention_pending`, but host-session `awaiting_attention` remains a separate authority layer.
6. [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md): whether an event kind may create an obligation, require host attention, or trigger auto-attach remains policy-gated and deny-by-default.

## Problem Statement

How might Substrate model durable deferred host work so that:

1. one artifact records what happened and what is still owed,
2. host review/inbox behavior is a projection, not a second source of truth,
3. auto-attach eligibility and processing are a projection, not a second source of truth,
4. `awaiting_attention` derives from authoritative unresolved obligations,
5. manual `reattach` and automatic attach share one causally coherent record.

## Frozen Direction

This design freezes the following:

1. the canonical durable artifact is an orchestration obligation, not a notification plus separate queue pair,
2. worker events and runtime alerts may create obligations,
3. inbox/review semantics are a read-side projection over obligations,
4. auto-attach semantics are an execution-side projection over obligations,
5. host-session posture derives from unresolved attention-driving obligations,
6. no projection may bypass host-session durable authority or synthesize hidden prompts,
7. exact identity and causation fields remain mandatory.

## Non-Goals

This design does not:

1. define the public inbox, queue, or CLI surface,
2. define final Rust type names or on-disk serialization syntax,
3. define router/daemon execution timing or backoff policy,
4. replace exact identity routing with fuzzy target selection,
5. authorize autonomous worker continuation that bypasses the host.

## Core Principle

An obligation is the durable answer to:

1. what happened,
2. who caused it,
3. whether host attention is required,
4. whether attach should be attempted when no host is attached,
5. whether the obligation has been acknowledged, processed, resolved, or dismissed.

Inbox rows and attach-processing state are therefore not separate durable truths.

They are different views of the same obligation.

## Canonical Artifact Model

Conceptual shape:

```text
OrchestrationObligationV1
- obligation_id
- orchestration_session_id
- source_participant_id?
- source_backend_id?
- source_role?
- origin_host_id?
- target_host_id?
- ingress_source_kind?
- ingress_source_id?
- ingress_received_at?
- target_participant_id?
- target_backend_id?
- world_id?
- world_generation?
- kind
- severity
- attention_required
- summary
- detail_ref?
- payload?
- causation_event_id?
- causation_message_id?
- causation_request_id?
- review_state
- attach_state
- attach_attempt_count
- attach_claim_owner?
- attach_last_attempt_at?
- attach_completion_reason?
- created_at
- updated_at
- resolved_at?
- resolution_note?
```

## Required Field Semantics

1. `obligation_id`
   - unique durable identifier within the orchestration session.
2. `orchestration_session_id`
   - authoritative parent session.
3. `source_participant_id`
   - exact retained worker or runtime producer identity when known.
4. `source_backend_id`
   - exact producer backend when known.
5. `origin_host_id`
   - optional stable host/machine identity for the host where the obligation originated.
   - not required in the purely local v1 posture, but the artifact must preserve room for it.
6. `target_host_id`
   - optional stable host/machine identity when an obligation is being materialized for a specific local host from a broader upstream system.
7. `ingress_source_kind`
   - optional classification for future non-local ingress such as `local_runtime`, `local_router`, `remote_webhook`, or `remote_sync`.
8. `ingress_source_id`
   - optional upstream ingress/request identifier for joinability when obligations are created from a future host-global inbox or remote federation layer.
9. `ingress_received_at`
   - optional local receipt timestamp for future remote or federated ingress.
10. `target_backend_id`
   - exact preferred host backend when known from persisted attach truth or session contract.
11. `world_id` and `world_generation`
   - authoritative world-bound context when world-side work caused the obligation.
12. `kind`
   - explicit obligation class, never inferred from free text.
13. `attention_required`
   - whether the obligation contributes to host-session `awaiting_attention` while unresolved.
14. `review_state`
   - durable host-review lifecycle.
15. `attach_state`
   - durable auto-attach processing lifecycle for the same obligation.
16. `attach_attempt_count`
   - number of attach-processing attempts started for this obligation.
17. `summary` / `payload`
   - explanation-ready content, but not an implicit prompt.

## Forward-Compatibility Identity Envelope

Substrate is local-first, but the local obligation model should stay open to a later host-global inbox, remote webhook ingress, and cross-machine coordination layers.

### Hard direction

The local obligation ledger remains authoritative for local session truth.

Future distributed layers must adapt into this ledger rather than replacing it.

### Fields we should preserve room for now

1. stable `obligation_id`
2. exact `orchestration_session_id`
3. exact source and target backend identity
4. authoritative `world_id` and `world_generation` when world-bound
5. causation fields:
   - `causation_event_id`
   - `causation_message_id`
   - `causation_request_id`
6. optional host/machine identity:
   - `origin_host_id`
   - `target_host_id`
7. optional ingress/source metadata:
   - `ingress_source_kind`
   - `ingress_source_id`
   - `ingress_received_at`

### Why freeze this now

These fields keep the local model joinable if a later global layer needs to:

1. deliver a request from outside the local machine,
2. synchronize or replay obligations across hosts,
3. correlate local action with upstream webhook, federation, or lease/lock coordination records.

## Future Host-Global Inbox Layering

The likely future distributed layer should live under `SUBSTRATE_HOME` as host-level state, but it must remain one rung above local orchestration-session truth.

### Recommended boundary

1. host-global inbox:
   - host-level ingress and synchronization layer,
   - may receive remote requests or webhook-originated work,
   - may coordinate cross-machine delivery, sync, or lock state.
2. local obligation ledger:
   - canonical local session truth,
   - drives local review, `awaiting_attention`, and attach processing.

### Recommended future path family

This design does not freeze a final schema yet, but it intentionally preserves room for a host-level namespace under:

1. `SUBSTRATE_HOME/host_inbox/`

Potential future contents may include:

1. ingress request logs,
2. durable receive cursor or sync state,
3. remote delivery and lease metadata,
4. host-global review or routing state that is not identical to any one orchestration session.

### Hard rule

The host-global inbox must not become the canonical owner of local orchestration-session state.

Instead:

1. remote or global ingress creates local trace events, local requests, and/or local obligations,
2. local policy evaluates them on the target host,
3. local obligations remain the authoritative record for session-local deferred work.

## Obligation Kinds

The minimum v1 obligation kinds should be:

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

These largely preserve the earlier notification taxonomy, but the artifact is now the obligation itself.

## Review Projection

The inbox/read-side model is a projection over obligations.

### Review state

Recommended durable review states:

1. `unread`
2. `acked`
3. `resolved`
4. `dismissed`

### State meanings

1. `unread`
   - persisted and not yet acknowledged by host-side review logic.
2. `acked`
   - seen, but the underlying obligation may still be outstanding.
3. `resolved`
   - the underlying obligation has been satisfied.
4. `dismissed`
   - explicitly closed without taking the implied action.

### Hard rule

The inbox is not the canonical artifact.

It is the review-facing projection of obligations filtered and rendered for host/operator consumption.

## Auto-Attach Projection

Auto-attach is also a projection over obligations.

### Attach state

Recommended durable attach states:

1. `not_requested`
2. `queued`
3. `claimed`
4. `completed`
5. `cancelled`
6. `dead_letter`

### State meanings

1. `not_requested`
   - no attach attempt is currently needed or permitted.
2. `queued`
   - the obligation is eligible for attach processing and awaits a processor.
3. `claimed`
   - one sanctioned processor is actively handling attach for this obligation.
4. `completed`
   - attach processing completed successfully for this obligation.
5. `cancelled`
   - attach processing became unnecessary or was superseded.
6. `dead_letter`
   - attach processing must stop under current truth or policy.

### Hard rule

Attach state belongs to the same obligation record.

There is no second canonical `work_queue` artifact in the greenfield model frozen here.

## Host Posture Derivation

`awaiting_attention` remains a host-session posture, not an obligation state and not a worker lifecycle state.

### Hard rule

The host orchestration session enters or remains in `awaiting_attention` when at least one unresolved obligation exists with:

1. `attention_required=true`, and
2. `review_state` not in `resolved|dismissed`.

Host posture is therefore derived from obligations, not from attach state.

## Trigger Eligibility

An obligation is eligible for auto-attach processing only when all of the following are true:

1. the parent orchestration session exists and is non-terminal,
2. there is no authoritative attached host participant,
3. the obligation remains unresolved,
4. policy allows auto-attach evaluation for the obligation kind,
5. exact boundary truth required for later attach is available or at least not disproven.

### Conservative v1 default

Eligible by default:

1. `follow_up_required`
2. `approval_required`
3. `blocked`
4. `fork_request`

Not eligible by default:

1. `task_completed`
2. `result_available`
3. `fork_recommendation`

Explicitly deferred unless policy opts in:

1. `task_failed`
2. `runtime_alert`
3. `escalation_recommended`

## Dedupe Model

The unified artifact collapses the previous queue-dedupe problem.

### Frozen direction

1. one durable obligation represents one causal orchestration obligation,
2. attach processing for that obligation lives inside the same artifact,
3. duplicate active attach requests for the same obligation are therefore impossible by design if `obligation_id` is stable.

If later implementations need a stronger causation key, it should be derived from:

1. `orchestration_session_id`
2. causation fields
3. `kind`
4. source identity

but the canonical durable join point remains `obligation_id`.

## Producer Rule

Worker events and runtime alerts do not create prompts.

They create obligations.

Examples:

1. `follow_up_question` -> `follow_up_required`
2. `approval_request` -> `approval_required`
3. `blocked` -> `blocked`
4. `result` with review value -> `result_available`
5. `failure` -> `task_failed`
6. `fork_request` -> `fork_request`
7. `fork_recommendation` -> `fork_recommendation`
8. `needs_retained_followup` -> `escalation_recommended`

## Consumer Rule

Host-side consumers may:

1. list obligations,
2. render inbox-style views,
3. acknowledge obligations,
4. resolve or dismiss obligations through explicit action,
5. process attach state through sanctioned router/daemon paths.

Consumers may not:

1. auto-submit obligation summaries as prompts,
2. treat attach-state completion as obligation resolution,
3. silently delete unresolved obligations,
4. bypass exact session identity or policy gates.

## Manual Reattach Interaction

Manual exact-session `reattach` remains canonical and valid.

### Required behavior

1. successful manual reattach may transition `attach_state` from `queued|claimed` to `cancelled` or `completed`,
2. successful manual reattach does not itself resolve the obligation,
3. later host action through sanctioned control paths resolves or dismisses the obligation explicitly.

## Why This Is Cleaner

This architecture removes dual-ledger drift.

It avoids needing separate answers to:

1. which artifact is authoritative,
2. how queue rows and inbox rows dedupe,
3. whether queue completion implies review completion,
4. how manual reattach supersedes queue work.

All of those become substate transitions on one canonical obligation.

## Deferred To Projection Docs

This document intentionally leaves details to follow-on docs:

1. inbox/review projection semantics and rendering,
2. auto-attach trigger and attach-processing semantics,
3. router/daemon ownership, claiming, and retry mechanics.

## Summary

The greenfield durable model frozen here is:

1. one canonical `OrchestrationObligationV1` ledger under the session,
2. review/inbox as a projection over obligations,
3. auto-attach as a projection over obligations,
4. host posture derived from unresolved attention-driving obligations,
5. no second canonical queue ledger.
