# Design: Retained World Worker Messaging and Steering Contract

Status: draft design input. This document defines the missing retained world-worker messaging and steering contract that sits above the dispatch-allocation contract and below the later durable orchestration obligation ledger plus its review/attach projections. It is not a public human UX design. It freezes how host orchestrators and retained world workers exchange turns, steering signals, progress, approvals, escalation requests, and fork requests.

## Why This Doc Exists

The dispatch-allocation design defines how world work is created and targeted. It does not fully define the ongoing message protocol between a host orchestrator and a retained world worker once that worker exists.

That gap matters because the architecture now assumes all of the following:

1. host orchestrators dispatch and steer world work through a control plane,
2. retained world workers may asynchronously re-engage the host,
3. `awaiting_attention` is driven by durable unresolved orchestration obligations,
4. world-worker fork may be initiated by the host or requested by the worker.

Without a frozen messaging contract, lifecycle, inbox, and policy docs will drift.

## Relationship To Existing Decisions

This design composes with:

1. [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md): dispatch allocation, exact targeting, and action vocabulary.
2. [ADR-0047](../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md): durable host-session authority and `awaiting_attention` posture.
3. [11-in-world-member-dispatch-over-existing-host-world-transport.md](./11-in-world-member-dispatch-over-existing-host-world-transport.md): retained world work stays agent-native through the world-member seam.
4. [ADR-0045](../docs/adr/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md): the future toolbox remains a control-plane surface and must not become a second execution plane.

## Problem Statement

How might Substrate define a deterministic retained-worker message contract so that:

1. host orchestrators can send conversational turns and operational steering to retained world workers,
2. retained world workers can reply, ask questions, report progress, request approvals, and request forks,
3. those interactions remain exact-identity, auditable, and fail-closed,
4. world-to-host asynchronous re-engagement feeds later inbox/attention semantics without synthetic prompt injection?

## Frozen Direction

This design freezes the following:

1. retained worker messaging is a first-class protocol, not a side effect of dispatch allocation,
2. messaging and lifecycle control are distinct classes even when they share identity and transport fields,
3. host-to-world retained turns require exact participant targeting,
4. world-to-host retained events are explicit typed messages, not ad hoc log parsing,
5. retained world-worker fork is permission-gated and explicit,
6. world workers may request fork, but they do not self-replicate silently.

## Non-Goals

This design does not:

1. define the later human direct-message surface for world workers,
2. define the final durable obligation artifact schema on disk,
3. define the full transport wire encoding or Rust type names,
4. redefine public host-session `fork`,
5. permit fuzzy routing or heuristic target selection.

## Distinct Message Planes

There are two retained-worker message planes:

### 1. Host -> retained worker

The host orchestrator sends:

1. conversational instructions,
2. follow-up turns,
3. approval responses,
4. operational steering,
5. explicit fork commands.

### 2. Retained worker -> host

The retained worker sends:

1. normal replies,
2. follow-up questions,
3. progress updates,
4. blocked or failed state,
5. approval requests,
6. attention-required events,
7. fork requests or fork recommendations,
8. result/completion events.

These must not be conflated into one generic untyped blob.

## Canonical Host-To-Worker Envelope

Conceptual shape:

```text
WorldWorkerMessageV1
- message_id
- orchestration_session_id
- source_participant_id
- target_participant_id
- source_backend_id
- target_backend_id
- world_id
- world_generation
- message_class
- causation_message_id?
- dispatch_request_id?
- thread_id
- expects_reply
- payload
- created_at
```

### Required semantics

1. `message_id`
   - unique within the orchestration session.
2. `source_participant_id`
   - exact host orchestrator identity.
3. `target_participant_id`
   - exact retained worker identity.
4. `message_class`
   - explicit typed class, never inferred from payload text.
5. `thread_id`
   - worker-local conversation thread or control thread identifier.
6. `expects_reply`
   - explicit signal for whether the sender expects a reply or only an acknowledgement.
7. `payload`
   - typed by message class.

## Host-To-Worker Message Classes

The host orchestrator should be allowed to send:

1. `instruction`
   - normal conversational task continuation or refinement.
2. `clarification_response`
   - answer to a worker follow-up question.
3. `approval_response`
   - explicit yes/no or constrained approval response.
4. `progress_ack`
   - acknowledgement that host received a progress update; usually optional.
5. `control_directive`
   - operational instruction such as pause, reduce scope, summarize, checkpoint, or prepare handoff.
6. `fork_command`
   - explicit request that Substrate allocate a child worker from the source retained worker context.

## Canonical Worker-To-Host Envelope

Conceptual shape:

```text
WorldWorkerEventV1
- event_id
- orchestration_session_id
- source_participant_id
- target_participant_id
- source_backend_id
- target_backend_id
- world_id
- world_generation
- event_class
- causation_message_id?
- dispatch_request_id?
- thread_id
- attention_required
- payload
- emitted_at
```

### Required semantics

1. `event_id`
   - unique event identifier.
2. `source_participant_id`
   - exact retained worker identity.
3. `target_participant_id`
   - exact host orchestrator identity.
4. `event_class`
   - explicit typed class, never inferred from free text.
5. `attention_required`
   - explicit signal for later durable obligation, review, and attach behavior.
6. `payload`
   - typed by event class.

## Worker-To-Host Event Classes

The retained worker should be allowed to emit:

1. `reply`
   - normal response to a host instruction.
2. `follow_up_question`
   - asks the host for clarification before continuing.
3. `progress_update`
   - informational progress, partial completion, or checkpoint status.
4. `approval_request`
   - requires an explicit host approval response.
5. `blocked`
   - the worker cannot continue without host input or external change.
6. `result`
   - successful unit of work or terminal completion for a retained mission phase.
7. `failure`
   - failure with explanation-ready reason.
8. `attention_required`
   - explicit signal that host review is needed.
9. `fork_request`
   - the worker requests that the host/orchestrator consider allocating a child worker.
10. `fork_recommendation`
   - non-binding recommendation that forking would be beneficial.
11. `control_ack`
   - acknowledgement of a host control directive.

## Fork Semantics

Forking needs special treatment because the repo already uses `fork` for host-session successor allocation.

### 1. Retained worker fork is not host-session fork

This design uses "worker fork" to mean:

1. source retained world worker remains in the same orchestration session,
2. a distinct child world worker is allocated under that same orchestration session,
3. the child receives a handoff summary or source-derived context,
4. exact lineage is persisted.

It does not mean:

1. allocate a new host orchestration session,
2. replace the source worker in place,
3. silently split one worker into two without durable lineage.

### 2. Two valid initiation paths

Worker fork may begin in either of these ways:

1. host-initiated:
   - the host sends `fork_command` to the source retained worker through Substrate control-plane handling,
   - Substrate allocates the child worker if policy permits.
2. worker-requested:
   - the retained worker emits `fork_request` or `fork_recommendation`,
   - the host reviews it,
   - the host then explicitly accepts or declines,
   - accepted requests lead to explicit `fork_world_worker` control-plane allocation.

### 3. Fork permission and autonomy

World workers must not silently fork themselves by default.

At minimum, the design requires separate policy or launch-time permission for:

1. whether a worker may emit `fork_request`,
2. whether a worker may emit only `fork_recommendation`,
3. whether a worker may execute limited auto-fork under a narrow rubric,
4. maximum fork depth and concurrency.

The safest default is:

1. workers may recommend or request,
2. the host/orchestrator decides,
3. Substrate enforces explicit policy.

### 4. Fork request payload requirements

Conceptual payload:

```text
ForkRequestPayloadV1
- fork_reason
- fork_goal
- recommended_backend_id?
- recommended_mode
- context_handoff_summary
- expected_benefit
- estimated_parallelism_value
- urgency
```

The payload should explain:

1. why a child worker helps,
2. what task the child would own,
3. whether the child should be retained or, if allowed, ephemeral,
4. what minimal context handoff is sufficient.

### 5. Worker fork lineage

Every accepted worker fork must persist:

1. `forked_from_participant_id`,
2. `fork_request_event_id` or `fork_command_message_id`,
3. child `backend_id`,
4. child `world_id`,
5. child `world_generation`,
6. child mission or task handoff summary.

## Conversational Versus Operational Semantics

The protocol must distinguish conversational turns from operational steering.

### Conversational classes

1. `instruction`
2. `reply`
3. `follow_up_question`
4. `clarification_response`
5. `result`

These participate in worker-local conversational thread continuity.

### Operational classes

1. `control_directive`
2. `control_ack`
3. `approval_request`
4. `approval_response`
5. `progress_update`
6. `blocked`
7. `fork_request`
8. `fork_recommendation`
9. `fork_command`
10. `attention_required`

These affect control flow, not just chat continuity.

## Threading and Ordering Rules

The protocol must remain deterministic.

### Required rules

1. retained-worker messages are always anchored to exact `target_participant_id`,
2. each message or event belongs to a `thread_id`,
3. causation fields must link worker replies to specific host messages when applicable,
4. host-side ordering must be preserved per `(target_participant_id, thread_id)`,
5. the runtime must reject attempts to continue a retained worker without exact identity.

### v1 simplification

For v1, it is acceptable to treat each retained worker as having:

1. one primary conversational thread,
2. one control/event thread,

as long as the protocol leaves room for future refinement.

## Streaming and Delivery Expectations

The design should permit both immediate streamed responses and durable event persistence, but it must keep their meanings separate.

### Immediate channel

Used for:

1. in-band replies,
2. progress streaming,
3. quick acknowledgements,
4. active-turn observability.

### Durable channel

Used for:

1. unresolved follow-up questions,
2. approval requests,
3. blocked states,
4. attention-required events,
5. fork requests or recommendations,
6. result/failure obligations that must survive host detachment.

The durable obligation-ledger design will define the canonical on-disk representation later. This design only freezes which event classes need that durable path.

## Attention Semantics

A retained worker event may set `attention_required=true`.

That should mean:

1. the event is eligible to drive durable obligation state,
2. the host session may move to `awaiting_attention`,
3. the event is not equivalent to immediate prompt injection into the host agent.

Attention-driving classes should include at minimum:

1. `follow_up_question`
2. `approval_request`
3. `blocked`
4. `attention_required`
5. `fork_request`

`progress_update` should not drive attention by default.

## Failure and Escalation Behavior

Retained workers may:

1. report failure and remain resumable,
2. report blocked state and await host input,
3. request fork when decomposition would help,
4. request narrowing or replanning through normal typed events.

They may not:

1. silently rewrite their own lifecycle mode,
2. silently create sibling workers,
3. bypass host-orchestrator review when policy requires review.

## Exact Identity and Policy Expectations

Messaging must stay aligned with the dispatch contract:

1. exact `orchestration_session_id`,
2. exact `source_participant_id`,
3. exact `target_participant_id`,
4. exact `backend_id`,
5. exact authoritative `world_id` and `world_generation`.

No fuzzy routing by role name, prompt text, or "most recent live worker" is allowed.

Policy should be able to gate separately:

1. whether a worker may ask follow-up questions,
2. whether a worker may emit approval requests,
3. whether a worker may request fork,
4. whether a worker may auto-fork under any narrow allowed rubric,
5. whether the host may issue explicit fork commands,
6. whether the host may continue, cancel, or stop the retained worker.

## v1 Summary

The first shippable retained-worker messaging contract should therefore be:

1. explicit,
2. typed,
3. exact-identity,
4. split between conversational and operational classes,
5. durable-attention compatible,
6. fork-aware,
7. fail-closed by default.

## Follow-On Design Dependencies

This messaging contract should feed directly into:

1. the retained worker lifecycle design,
2. the durable obligation-ledger and review-projection designs,
3. the host-to-world steering policy matrix.
