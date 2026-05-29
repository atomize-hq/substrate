# Design: World Worker Lifecycle Model

Status: draft design input. This document defines the lifecycle model for world-side agent work after the dispatch-allocation and retained-worker messaging contracts have been frozen. It is not a public CLI contract. It freezes the runtime state distinctions and transition rules for `ephemeral` versus `retained` world work, including parent-session interaction, attention semantics, cancellation, stopping, invalidation, and worker-level fork lineage.

## Why This Doc Exists

The repo now has a frozen direction for:

1. how host orchestrators allocate and target world work,
2. how retained world workers exchange messages with the host orchestrator,
3. how host sessions remain the durable authority.

What is still missing is the runtime lifecycle model that keeps those decisions coherent.

Without a lifecycle design:

1. `ephemeral` and `retained` can collapse into labels over the same runtime,
2. `awaiting_attention` can get overloaded into worker state instead of host-session posture,
3. fork lineage can drift,
4. cancellation and stopping semantics can become ambiguous.

This document closes that gap.

## Relationship To Existing Decisions

This design composes with:

1. [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md): explicit mode and action selection.
2. [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md): conversational and operational message classes plus worker fork requests.
3. [ADR-0047](../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md): host-session durable authority and `awaiting_attention` posture.
4. [23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](./23-host-orchestrator-durable-session-and-parked-resumable-ownership.md): durable host posture and inbox-driven host attention semantics.
5. [11-in-world-member-dispatch-over-existing-host-world-transport.md](./11-in-world-member-dispatch-over-existing-host-world-transport.md): world-side agent execution remains agent-native.

## Problem Statement

How might Substrate model world-side work so that:

1. `ephemeral` truly means disposable one-shot execution,
2. `retained` truly means authoritative orchestration state with future routing obligations,
3. worker events can influence host-session posture without collapsing worker state into host state,
4. fork creates explicit sibling lineage instead of implicit duplication,
5. invalidation and world-generation drift stay fail-closed?

## Frozen Direction

This design freezes the following:

1. `ephemeral` and `retained` are different lifecycle classes, not just different launch labels,
2. worker lifecycle state is separate from host-session posture,
3. `awaiting_attention` remains host-session truth, not a world-worker lifecycle state,
4. world workers may influence host posture by emitting durable attention-driving events,
5. retained worker fork always creates a distinct child worker identity with explicit lineage,
6. world-generation mismatch invalidates routability rather than being silently repaired in place.

## Non-Goals

This design does not:

1. define the final storage schema for world-worker records,
2. define the final durable notification/inbox artifact shape,
3. define public user-facing commands for inspecting world-worker lifecycle directly,
4. replace exact identity routing with fuzzy worker selection,
5. redefine public host-session `fork`.

## Core Lifecycle Principle

The lifecycle boundary is:

1. `ephemeral` means "do work and disappear,"
2. `retained` means "become part of authoritative orchestration state."

If a worker is expected to:

1. ask follow-up questions,
2. survive past one result,
3. request approvals,
4. request fork,
5. be continued later,

then it is retained by nature.

## Two Lifecycle Families

There are two runtime lifecycle families:

1. ephemeral task lifecycle,
2. retained worker lifecycle.

They share identity, trace, policy, and world-binding fields, but they do not share the same transition rules.

## Ephemeral Task Lifecycle

### Canonical states

1. `allocating`
2. `running`
3. `completed`
4. `failed`
5. `cancelled`
6. `needs_retained_followup`
7. `invalidated`

### State meanings

1. `allocating`
   - the task has been accepted and runtime allocation is in progress.
2. `running`
   - world-side execution is actively in flight.
3. `completed`
   - the task produced its terminal result successfully.
4. `failed`
   - the task terminated unsuccessfully.
5. `cancelled`
   - the task was cancelled while active.
6. `needs_retained_followup`
   - the task cannot continue as one-shot work and explicitly recommends escalation to a retained worker.
7. `invalidated`
   - routing or world-binding authority no longer matches the active authoritative world posture.

### Allowed transitions

```text
allocating -> running
allocating -> failed
allocating -> invalidated

running -> completed
running -> failed
running -> cancelled
running -> needs_retained_followup
running -> invalidated
```

### Ephemeral invariants

1. no stable future-routable participant identity is promised,
2. no later `continue` contract exists,
3. no direct durable conversational obligation is created,
4. no worker-local `awaiting_attention` state exists,
5. any request for ongoing work must surface as `needs_retained_followup`,
6. successful completion or failure ends the routable lifecycle.

## Retained Worker Lifecycle

### Canonical states

1. `allocating`
2. `running`
3. `attention_pending`
4. `paused`
5. `parked`
6. `completed`
7. `failed`
8. `cancelled`
9. `stopped`
10. `invalidated`

### State meanings

1. `allocating`
   - the retained worker has been requested but is not yet fully routable.
2. `running`
   - the worker is active and routable for host-to-worker messaging.
3. `attention_pending`
   - the worker has emitted an unresolved event that requires host-side review or response before forward progress can continue.
4. `paused`
   - the worker is intentionally paused by control directive or runtime policy and is not currently executing.
5. `parked`
   - the worker remains valid and resumable, but no active turn is executing and it is waiting for later continuation.
6. `completed`
   - the worker finished its retained mission phase successfully and is terminally done.
7. `failed`
   - the worker terminated with failure and is not automatically resumable without explicit later policy.
8. `cancelled`
   - the worker or active worker turn was cancelled.
9. `stopped`
   - explicit durable closeout was requested through control plane.
10. `invalidated`
   - the worker lost authoritative routing validity because world binding, generation, or orchestration authority no longer matches.

### Allowed transitions

```text
allocating -> running
allocating -> failed
allocating -> invalidated

running -> attention_pending
running -> paused
running -> parked
running -> completed
running -> failed
running -> cancelled
running -> stopped
running -> invalidated

attention_pending -> running
attention_pending -> paused
attention_pending -> parked
attention_pending -> stopped
attention_pending -> invalidated

paused -> running
paused -> parked
paused -> stopped
paused -> invalidated

parked -> running
parked -> stopped
parked -> invalidated
```

### Retained invariants

1. a stable `participant_id` exists,
2. later `continue` targeting is part of the contract,
3. worker events may create durable host attention obligations,
4. exact backend, world binding, and lineage remain authoritative routing inputs,
5. the worker may request fork or recommend fork if policy and launch-time permissions allow it.

## Worker State Versus Host Posture

This design must keep worker lifecycle state distinct from host-session posture.

### Worker lifecycle answers

1. is the worker allocated?
2. is it running?
3. is it blocked pending host response?
4. is it parked and resumable?
5. is it terminal or invalidated?

### Host posture answers

1. is a host execution client attached?
2. does the orchestration session have unresolved notifications?
3. is the host session routable?

### Hard rule

`awaiting_attention` remains a host-session posture only.

Retained workers do not enter an `awaiting_attention` state. Instead:

1. the worker enters `attention_pending`,
2. it emits an attention-driving event,
3. the host session may then enter `awaiting_attention`.

That keeps the authority layers clean.

## Attention-Pending Semantics

Retained workers should move into `attention_pending` when they emit an unresolved event in one of these classes:

1. `follow_up_question`
2. `approval_request`
3. `blocked`
4. `attention_required`
5. `fork_request`

### Required behavior

1. the worker must remain exact-identity routable unless later invalidated or stopped,
2. no synthetic prompt may be injected into the host,
3. the event becomes eligible for durable notification/inbox persistence,
4. host response through a sanctioned message path may move the worker back to `running` or `parked`.

## Parked Semantics

`parked` is different from `paused` and `attention_pending`.

### `parked`

Means:

1. the worker is valid and resumable,
2. no unresolved blocking host response is required,
3. no active turn is executing,
4. later explicit continuation is allowed.

### `paused`

Means:

1. forward progress was intentionally suspended by control or policy,
2. the worker should not resume normal work until unpaused or redirected.

### `attention_pending`

Means:

1. host review or response is required before meaningful forward progress.

## Result and Completion Semantics

### Ephemeral

For `ephemeral`, `completed`, `failed`, `cancelled`, and `needs_retained_followup` are terminal lifecycle outcomes.

### Retained

For `retained`, `result` events may occur while the worker remains non-terminal.

That means:

1. partial results do not imply lifecycle completion,
2. only explicit terminal lifecycle transitions close the retained worker,
3. a retained worker may produce multiple result-bearing messages before entering `completed` or `stopped`.

## Cancellation and Stopping

These must remain distinct.

### Cancellation

Means:

1. stop currently active execution,
2. may leave retained lifecycle in `cancelled`,
3. primarily concerns active work in flight.

### Stopping

Means:

1. explicit durable closeout of a retained worker,
2. no further continuation should be allowed,
3. the worker enters `stopped`.

### Rules

1. `ephemeral` supports cancel while active; it does not support durable stop after terminal completion.
2. `retained` supports both cancel and stop.
3. stop is a control-plane lifecycle action, not merely a transport interruption.

## Invalidation

Invalidation is not ordinary failure.

### Causes of invalidation

1. authoritative `world_id` mismatch,
2. authoritative `world_generation` mismatch,
3. orchestration session no longer routable,
4. worker lineage no longer valid under current world replacement or authority truth.

### Required behavior

1. invalidated workers must not be silently reused,
2. invalidated workers must fail closed for continuation,
3. if future work is needed, a replacement retained worker must be allocated through explicit control-plane action.

## Fork Lifecycle

Fork is not a state. It is a transition-triggering control-plane event that allocates a new sibling worker.

### Source worker behavior

When a retained worker forks:

1. the source worker remains itself,
2. it does not become the child,
3. it may stay `running`, move to `parked`, or move to `attention_pending` depending on the fork policy and host decision,
4. it must preserve exact lineage identity.

### Child worker behavior

The child worker begins a new lifecycle:

```text
allocating -> running -> ...
```

### Fork invariants

1. child worker gets new `participant_id`,
2. child worker persists `forked_from_participant_id`,
3. child worker stays in the same `orchestration_session_id`,
4. child worker must inherit exact authoritative `world_id` and `world_generation` unless an explicit later design allows otherwise,
5. child worker mission must come from explicit handoff summary, not implicit shared mutable context.

## Parent Session Interaction Rules

The host orchestration session remains the durable authority root.

### What workers may influence

Retained workers may influence:

1. durable notification/inbox creation,
2. host-session `awaiting_attention`,
3. world-worker lineage graph under the same session,
4. later exact-target continuation options.

### What workers may not influence directly

Retained workers may not:

1. mutate host-session posture directly without going through durable event rules,
2. self-promote to host authority,
3. allocate sibling workers silently,
4. bypass exact identity or policy gates.

## Minimal Runtime Classification Rules

The runtime must keep the mode boundary honest.

### Ephemeral classification rule

`ephemeral` remains valid only while all are true:

1. no future continuation is promised,
2. no durable conversational obligation exists,
3. fresh retry is acceptable,
4. the task can terminate with one of:
   - `completed`
   - `failed`
   - `cancelled`
   - `needs_retained_followup`

### Retained classification rule

If the worker:

1. needs to ask questions,
2. can wait for approvals,
3. can later continue,
4. can request fork,
5. can produce multiple staged results,

then it is retained.

## v1 Recommended Simplifications

To keep v1 shippable without architectural debt:

1. prefer `retained` when ambiguous,
2. allow `ephemeral` only for truly disposable one-shot work,
3. support explicit `needs_retained_followup` instead of silent mode escalation,
4. support worker fork only under explicit policy and exact lineage recording,
5. keep one retained worker as one exact targetable identity at a time, even if it emits many progress/result events.

## v1 Summary

The first shippable world-worker lifecycle model should therefore be:

1. two-family: `ephemeral` and `retained`,
2. exact-identity and exact-world-binding,
3. host-posture separate from worker state,
4. fork-aware with sibling lineage,
5. fail-closed on invalidation,
6. explicit on cancellation, stopping, and escalation.

## Follow-On Design Dependencies

This lifecycle design should feed directly into:

1. the durable notification/inbox contract,
2. the host-to-world steering policy matrix,
3. later implementation planning for worker records and runtime state persistence.
