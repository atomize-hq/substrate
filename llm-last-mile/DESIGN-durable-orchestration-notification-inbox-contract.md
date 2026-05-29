# Design: Durable Orchestration Notification and Inbox Contract

Status: draft design input. This document defines the durable notification and inbox contract that allows retained world workers and related runtime components to re-engage the host orchestration session without relying on an attached host client, synthetic prompt injection, or trace-only heuristics. It is not a public inbox UX design. It freezes the semantic contract for durable notification artifacts, attention-driving behavior, resolution rules, and the relationship between worker events and host-session posture.

## Why This Doc Exists

The repo already has:

1. durable host-session authority,
2. a narrow existing durable inbox seam,
3. explicit retained worker messaging and event classes,
4. a lifecycle model that distinguishes worker `attention_pending` from host-session `awaiting_attention`.

What is still missing is the stronger contract that ties those pieces together.

Without a frozen notification/inbox design:

1. world-to-host asynchronous re-engagement stays underspecified,
2. `awaiting_attention` risks becoming an informal UI concept rather than durable runtime truth,
3. worker events may drift into direct prompt injection or ad hoc side channels,
4. approval, blocked, fork-request, and follow-up semantics will diverge across implementation seams.

This document closes that gap.

## Relationship To Existing Decisions

This design composes with:

1. [ADR-0047](../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md): host-session durable authority and persisted attention posture.
2. [23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](./23-host-orchestrator-durable-session-and-parked-resumable-ownership.md): session-local durable inbox path and host attention semantics.
3. [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md): exact identity and dispatch/action join fields.
4. [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md): worker event classes and attention-required signaling.
5. [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md): worker `attention_pending`, host-session `awaiting_attention`, and lifecycle transition rules.

## Problem Statement

How might Substrate persist world-originated follow-up, approval, failure, progress, and fork-related events under the orchestration session so that:

1. no attached host client is required for delivery,
2. host-session `awaiting_attention` is durable and authoritative,
3. attention-driving events are resolved explicitly,
4. worker and host identities remain exact and auditable,
5. later reattach or host turns consume durable state through sanctioned control paths rather than hidden prompt injection?

## Frozen Direction

This design freezes the following:

1. durable notifications live under the orchestration session as authoritative state,
2. world workers and runtime components emit typed notifications, not raw prompt text for host injection,
3. `awaiting_attention` is driven by unresolved attention-driving notifications,
4. notification resolution is explicit and stateful,
5. durable notifications are separate from immediate streaming/rendering,
6. notification persistence is the authority for deferred host review when no client is attached.

## Non-Goals

This design does not:

1. define the public human inbox CLI or UI surface,
2. define all long-term compaction/retention mechanics in implementation detail,
3. turn notifications into a general message bus for arbitrary producer types,
4. allow notifications to bypass control-plane semantics,
5. define the full wire protocol used for live streaming of active turns.

## Core Principle

Notifications are not prompts.

They are durable orchestration artifacts that record:

1. something happened,
2. who caused it,
3. whether host review or response is required,
4. how that obligation is resolved.

The host agent may later inspect or consume those notifications through sanctioned control paths, but the notification itself must not act as an implicit prompt submission.

## Canonical Artifact Model

Conceptual shape:

```text
OrchestrationNotificationV1
- notification_id
- orchestration_session_id
- source_participant_id?
- source_backend_id?
- source_role?
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
- status
- created_at
- updated_at
- resolved_at?
- resolution_note?
```

## Required Field Semantics

1. `notification_id`
   - unique within the orchestration session.
2. `orchestration_session_id`
   - authoritative parent session.
3. `source_participant_id`
   - exact retained worker identity when the producer is a worker.
4. `source_backend_id`
   - exact backend identity when known.
5. `target_participant_id`
   - usually the current host orchestrator participant when one exists, but notifications remain valid even when no host client is attached.
6. `world_id` and `world_generation`
   - authoritative world binding context when world-side work caused the notification.
7. `kind`
   - explicit typed notification class.
8. `severity`
   - bounded operator significance, not arbitrary text.
9. `attention_required`
   - whether this item contributes to host-session attention posture until resolved.
10. `summary`
   - concise explanation-ready text.
11. `payload`
   - typed details associated with the notification class.
12. `status`
   - explicit lifecycle state for the notification itself.

## Notification Kinds

The contract should support at minimum:

1. `follow_up_required`
2. `approval_required`
3. `task_completed`
4. `task_failed`
5. `runtime_alert`
6. `blocked`
7. `fork_request`
8. `fork_recommendation`
9. `escalation_recommended`
10. `result_available`

### Kind semantics

1. `follow_up_required`
   - a retained worker asked a question or requires host instruction.
2. `approval_required`
   - explicit host approval is needed before forward progress.
3. `task_completed`
   - a mission phase or retained task completed.
4. `task_failed`
   - a retained mission phase or task failed.
5. `runtime_alert`
   - runtime-owned warning or exceptional condition that needs host awareness.
6. `blocked`
   - the worker cannot continue without host input or external change.
7. `fork_request`
   - a retained worker requests an explicit child-worker allocation.
8. `fork_recommendation`
   - a retained worker recommends forking but does not claim it is mandatory.
9. `escalation_recommended`
   - an `ephemeral` task terminated with `needs_retained_followup`.
10. `result_available`
   - durable result artifact exists and host review may be useful even if immediate attention is not required.

## Notification Status Lifecycle

Notification state is separate from worker lifecycle state.

### Canonical notification statuses

1. `unread`
2. `acked`
3. `resolved`
4. `dismissed`

### State meanings

1. `unread`
   - persisted and not yet acknowledged by host-side review logic.
2. `acked`
   - host or sanctioned runtime path has seen the notification, but required action may still be outstanding.
3. `resolved`
   - the underlying obligation has been satisfied.
4. `dismissed`
   - explicitly closed without taking the originally implied action.

### Allowed transitions

```text
unread -> acked
unread -> resolved
unread -> dismissed

acked -> resolved
acked -> dismissed
```

No transition back to `unread` is allowed.

## Attention-Driving Rule

The host orchestration session enters or remains in `awaiting_attention` when at least one unresolved attention-driving notification exists.

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

These may still be surfaced and retained, but they do not automatically require host attention unless policy or future product design says otherwise.

### Hard rule

`attention_required=true` plus unresolved status is what drives host posture, not whether a host client happened to be connected when the event arrived.

## Producer Classes

Notifications may be produced by:

1. retained world workers,
2. ephemeral task closeout adapters,
3. runtime control components,
4. world execution/runtime alert producers.

### Producer restrictions

1. producers must emit typed notifications, not generic chat messages,
2. producers must provide exact source identity when known,
3. producers must not mutate host posture directly; they only create notification artifacts,
4. any host posture update must be derived from authoritative unresolved notification state.

## Consumer Model

Consumers are sanctioned host-side control or review paths, not arbitrary readers writing hidden side effects.

### Consumers may:

1. list notifications for an orchestration session,
2. acknowledge notifications,
3. resolve notifications by taking explicit action,
4. dismiss notifications under policy,
5. attach resolution metadata.

### Consumers may not:

1. auto-submit notification summaries as prompts to the host backend,
2. silently delete unresolved notifications,
3. treat trace-only history as equivalent to canonical notification artifacts,
4. resolve notifications without explicit causal action or allowed operator/runtime policy.

## Host Reattach and Deferred Review Semantics

The absence of an attached host client must not:

1. drop notifications,
2. consume notifications,
3. invalidate the orchestration session,
4. force replay through synthetic bootstrap prompts.

Instead:

1. notifications accumulate durably,
2. the host session may move to `awaiting_attention`,
3. later `reattach` restores a live host client,
4. sanctioned host-side logic may inspect and work through the notifications.

This keeps the session durable while avoiding hidden prompt injection.

## Relationship To Worker Events

The messaging design defines worker events. This design defines which of those events require durable notification persistence.

### Required event -> notification mapping

1. `follow_up_question` -> `follow_up_required`
2. `approval_request` -> `approval_required`
3. `blocked` -> `blocked`
4. `failure` -> `task_failed` or `runtime_alert` depending on class
5. `result` -> `task_completed` or `result_available`
6. `fork_request` -> `fork_request`
7. `fork_recommendation` -> `fork_recommendation`
8. ephemeral `needs_retained_followup` terminal outcome -> `escalation_recommended`

### Important rule

Not every streamed event must become a durable notification.

For example:

1. frequent `progress_update` events may stream live,
2. only significant progress checkpoints should become durable artifacts when policy or product semantics require it.

## Fork Notification Semantics

Fork requests need durable handling because they often require host review while no host client is attached.

### `fork_request`

Means:

1. the worker believes a child worker is needed,
2. explicit host or sanctioned policy review is required,
3. unresolved state should usually drive `awaiting_attention`.

### `fork_recommendation`

Means:

1. the worker suggests parallel decomposition would help,
2. the host may ignore it without violating worker correctness,
3. it does not drive attention by default unless future policy says otherwise.

### Resolution behavior

1. accepting a `fork_request` should create a causal control-plane action and then resolve the notification,
2. declining it should dismiss or resolve it with an explicit note,
3. no worker may self-resolve its own fork request by silently allocating a child worker unless a later design explicitly allows tightly bounded auto-fork policy.

## Result and Failure Notifications

The contract should distinguish durable informational results from actionable failures.

### `task_completed`

Use when:

1. a retained worker mission phase completed,
2. host attention is not necessarily required,
3. durable record of completion matters.

### `task_failed`

Use when:

1. a retained worker failed in a way that may affect the orchestration plan,
2. host attention may or may not be required depending on policy,
3. a durable explanation-ready failure record is needed.

### `runtime_alert`

Use when:

1. the problem is with orchestration/runtime conditions rather than normal task failure,
2. host review is likely required,
3. failure to notice could lead to drift or invalid assumptions.

## Exact Identity Rules

Durable notifications must preserve exact identity context.

Required fields whenever known:

1. `orchestration_session_id`
2. `source_participant_id`
3. `source_backend_id`
4. `world_id`
5. `world_generation`
6. causal message, event, or request id

The runtime must be able to answer:

1. which worker emitted this,
2. under which world binding,
3. during which orchestration session,
4. in response to which prior message or request,
5. whether the notification is still unresolved.

## Retention and Compaction Principles

This design does not freeze final compaction algorithms, but it does freeze the retention principles:

1. unresolved notifications must never be compacted away,
2. resolution status must survive compaction,
3. informational resolved notifications may later be compacted to summaries,
4. compaction must not destroy causal traceability for still-relevant orchestration decisions.

## v1 Recommended Simplifications

To keep the first shipping implementation tight:

1. persist notifications only under the orchestration session root,
2. keep the status model to `unread | acked | resolved | dismissed`,
3. keep attention-driving rules explicit and small,
4. prefer exact mapping from worker event classes to notification kinds,
5. do not ship automatic host-side notification consumption or resume workflows.

## v1 Summary

The first shippable durable notification/inbox contract should therefore be:

1. typed,
2. exact-identity,
3. authoritative under the orchestration session,
4. separate from prompt submission,
5. explicit about unresolved versus resolved state,
6. the sole durable driver of host-session `awaiting_attention`.

## Follow-On Design Dependencies

This notification contract should feed directly into:

1. the host-to-world steering policy matrix,
2. later implementation planning for runtime state persistence,
3. any future human/operator inbox or review surface.
