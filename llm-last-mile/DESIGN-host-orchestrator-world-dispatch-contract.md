# Design: Host Orchestrator World Dispatch Contract

Status: draft design input. This document defines the first implementation-shaping contract for host-orchestrated world delegation after the landed host-session, shared-dispatch, and Linux-first world-member placement slices. It is not a public CLI design and it is not an implementation-ready SOW. It is the architectural design outline for the missing host-to-world control-plane seam.

## Why This Doc Exists

The repo already has:

1. durable host orchestration sessions,
2. explicit shared-world binding and world-generation truth,
3. retained world-member launch and follow-up seams on the runtime side,
4. a narrow public human control plane under `substrate agent start|turn|reattach|fork|stop`.

What the repo does not yet have is a frozen contract for how a host orchestration agent asks Substrate to create, steer, inspect, or stop world-side agent work.

That gap is real:

1. there is no landed orchestrator-to-world steering protocol,
2. there is no frozen request/response envelope for host agent dispatch into world,
3. the queued toolbox ADRs are still control-plane placeholders rather than live implementation truth,
4. the existing toolbox posture is intentionally introspection-only and must not be mistaken for a world-execution surface.

This design fills that exact gap.

## Relationship To Existing Decisions

This design must compose with the following existing repo truth:

1. [ADR-0047](../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md): the durable authority is the orchestration session, not any one attached host client.
2. [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](./29-shared-agent-dispatch-envelope-and-capability-override-contract.md): inventory-backed and persisted-attach-backed dispatch already share one internal contract family.
3. [11-in-world-member-dispatch-over-existing-host-world-transport.md](./11-in-world-member-dispatch-over-existing-host-world-transport.md): world-scoped member execution is a real runtime seam and should stay agent-native rather than being replaced by generic tools.
4. [ADR-0026](../docs/adr/draft/ADR-0026-orchestration-toolbox-mcp.md) and [ADR-0045](../docs/adr/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md): the future toolbox remains an internal control-plane surface and must not become a second execution plane.

## Problem Statement

How might Substrate let a host-scoped orchestration agent delegate and steer world-side agent work through a deterministic control-plane contract while preserving:

1. exact world/participant identity,
2. explicit lifecycle mode selection,
3. fail-closed policy boundaries,
4. durable host-session authority,
5. clean future support for asynchronous world-to-host re-engagement?

## Frozen Direction

The contract frozen by this design is:

1. world execution remains agent-native,
2. the host/orchestration agent never directly becomes the world execution surface,
3. the host/orchestration agent uses a narrow control-plane verb set to ask Substrate to allocate or steer world-side agent work,
4. every host-to-world dispatch request carries explicit `mode`,
5. mode defaults to `retained`,
6. `ephemeral` means no future routing obligation,
7. `retained` means the world worker becomes authoritative orchestration state,
8. runtime never guesses mode, backend, or target worker.

## Non-Goals

This design does not:

1. define the later human-facing direct-message surface for world workers,
2. turn the current toolbox into a general execution surface,
3. introduce fuzzy routing or backend heuristics,
4. replace the existing world-member runtime seam with generic tool execution,
5. define the durable obligation-ledger payload schema in full,
6. define the full policy schema in final implementation syntax.

## Current Gap: No Frozen Steering Contract Exists Yet

The user concern is correct: there is not yet a landed protocol/contract/tool surface for host orchestration agents sending messages to world agents or vice versa.

Current repo posture:

1. retained world-member execution exists as a runtime capability,
2. exact public human follow-up to world members exists through narrow `(orchestration_session_id, backend_id)` paths,
3. the queued toolbox ADRs still describe a control-plane direction, not landed steering verbs,
4. no current doc freezes the internal orchestrator-to-world request and response envelopes.

This design therefore introduces that missing seam explicitly.

## Conceptual Model

There are three distinct planes:

### 1. Host orchestration plane

The host orchestrator reasons, plans, and decides delegation.

It does not directly run world work.

### 2. Substrate control plane

Substrate accepts explicit host-orchestrator requests to:

1. allocate a world task,
2. allocate a retained world worker,
3. continue or inspect a retained world worker,
4. cancel or stop world-side work.

This plane is policy-gated and traceable.

### 3. World execution plane

Actual world-side work runs as a real world agent participant through the world-member runtime seam.

This plane is not a generic tool call.

## Control Verbs

The host orchestrator should interact with world work through a small explicit verb set:

1. `run_world_task`
2. `spawn_world_worker`
3. `continue_world_worker`
4. `inspect_world_worker`
5. `fork_world_worker`
6. `cancel_world_work`
7. `stop_world_worker`

Verb intent:

1. `run_world_task`
   - one-shot request,
   - requires `mode=ephemeral`,
   - no future continuation contract.
2. `spawn_world_worker`
   - allocate a retained world participant,
   - requires `mode=retained`,
   - future continuation is part of the contract.
3. `continue_world_worker`
   - send a new instruction or prompt-bearing turn to an existing retained worker,
   - exact retained participant identity required.
4. `inspect_world_worker`
   - retrieve authoritative lifecycle/status/progress snapshot for a world worker or active ephemeral task.
5. `fork_world_worker`
   - ask Substrate to allocate a new world worker using an existing retained world worker as the source context,
   - valid only as an explicit control-plane action,
   - may be initiated either by the host orchestrator directly or by the orchestrator in response to a world-worker fork request.
6. `cancel_world_work`
   - stop active work in flight,
   - may target active ephemeral work or an active retained worker turn.
7. `stop_world_worker`
   - close a retained world participant as a durable lifecycle action.

## Canonical Request Envelope

The runtime should require a shared request shape conceptually equivalent to:

```text
WorldDispatchRequestV1
- request_id
- idempotency_key
- orchestration_session_id
- caller_participant_id
- caller_backend_id
- caller_role
- action
- mode
- target_backend_id
- target_participant_id?
- world_id
- world_generation
- payload
- capability_overrides
- requested_policy_narrowing
- created_at
```

### Required field semantics

1. `request_id`
   - unique request identifier for trace and outcome joinability.
2. `idempotency_key`
   - required for `run_world_task` and `spawn_world_worker` so retries do not allocate duplicate work.
3. `orchestration_session_id`
   - the durable authority root.
4. `caller_participant_id`
   - exact host orchestrator participant identity.
5. `action`
   - one of the frozen control verbs above.
6. `mode`
   - mandatory,
   - one of `ephemeral | retained`,
   - runtime must reject omitted mode.
7. `target_backend_id`
   - exact backend selection, never fuzzy.
8. `target_participant_id`
   - required for `continue_world_worker`, `inspect_world_worker` when aimed at a retained worker, and `stop_world_worker`.
9. `world_id` and `world_generation`
   - required for world-bound execution and steering,
   - must match the authoritative parent session binding.
10. `payload`
   - the task or steering content,
   - typed by action,
   - not overloaded into lifecycle semantics.
11. `capability_overrides`
   - narrowing-only.
12. `requested_policy_narrowing`
   - optional stricter request-time narrowing if the caller wants less than the baseline.

## Action and Mode Validity Matrix

The runtime should enforce the following:

1. `run_world_task`
   - valid only with `mode=ephemeral`.
2. `spawn_world_worker`
   - valid only with `mode=retained`.
3. `continue_world_worker`
   - valid only with `mode=retained` and exact `target_participant_id`.
4. `inspect_world_worker`
   - valid for:
     - active ephemeral work with exact runtime-owned task identity, or
     - retained workers with exact participant identity.
5. `fork_world_worker`
   - valid only with `mode=retained`,
   - requires exact source retained `target_participant_id`,
   - allocates a distinct new world worker identity rather than mutating the source worker in place.
6. `cancel_world_work`
   - valid for:
     - active ephemeral work, or
     - an active retained worker turn.
7. `stop_world_worker`
   - valid only for retained workers.

Anything outside that matrix fails closed.

## Host-To-World Steering Payload Shapes

This design freezes the need for typed steering payloads rather than one generic free-form blob.

Minimum payload families:

1. `TaskPayloadV1`
   - for `run_world_task`,
   - includes task text or structured task request,
   - no continuation expectations.
2. `WorkerSpawnPayloadV1`
   - for `spawn_world_worker`,
   - includes initial mission, constraints, and worker role intent.
3. `WorkerContinuePayloadV1`
   - for `continue_world_worker`,
   - includes follow-up instruction, prompt, or steering command.
4. `WorkerInspectPayloadV1`
   - for `inspect_world_worker`,
   - optional scope for how much status/progress detail to return.
5. `WorkerForkPayloadV1`
   - for `fork_world_worker`,
   - includes:
     - `fork_strategy`,
     - `fork_reason`,
     - source retained worker identity,
     - child task or mission,
     - context handoff policy,
     - whether the child should be `retained` or, if policy allows, `ephemeral`.
6. `WorkerCancelPayloadV1`
   - for `cancel_world_work`,
   - includes reason and optional graceful-vs-immediate intent.
7. `WorkerStopPayloadV1`
   - for `stop_world_worker`,
   - explicit durable closeout intent.

The important rule is not the exact Rust type names. The important rule is that lifecycle mode and steering action are explicit protocol fields, not hidden inside prompt wording.

## World-To-Host Outcome Contract

World-side work must return through an explicit outcome plane, not by synthesizing host prompts.

Conceptual outcome shape:

```text
WorldDispatchOutcomeV1
- request_id
- orchestration_session_id
- action
- mode
- state
- participant_id?
- task_run_id?
- summary
- detail_ref?
- escalation_hint?
- emitted_at
```

Terminal `state` values:

1. `completed`
2. `failed`
3. `cancelled`
4. `needs_retained_followup`

Rules:

1. `needs_retained_followup` is valid only for ephemeral work.
2. `needs_retained_followup` is a terminal escalation recommendation, not silent promotion.
3. retained workers may additionally emit non-terminal world-to-host obligations through the later durable obligation-ledger and review-projection designs.
4. successful `fork_world_worker` allocation must surface distinct child worker identity and lineage rather than reporting the source worker as if it had continued in place.

## Exact Identity Rules

This design must stay aligned with the repo’s existing fail-closed exact-selector posture.

### Required identity rules

1. the host orchestrator may steer only work inside its own `orchestration_session_id`,
2. retained worker continuation requires exact `participant_id`,
3. world binding must match exact authoritative `world_id` and `world_generation`,
4. backend targeting remains exact by `backend_id`,
5. no fuzzy selection by role label, prompt text, or "best matching live worker" is allowed.

### Why this is frozen

Without exact identity:

1. trace joinability becomes heuristic,
2. world-generation rollover becomes dangerous,
3. policy boundaries weaken,
4. later human direct-to-world surfaces become ambiguous.

## Error-On-Retained Rule

Mode selection must follow the already-agreed bias:

1. `retained` is the default,
2. `ephemeral` is allowed only when a fresh worker plus a minimal handoff summary is an acceptable retry path.

The runtime contract should therefore reject attempts to treat obviously conversational or resumable flows as untyped generic ephemeral work. The precise screening policy belongs in later caller design, but the runtime contract should already treat `retained` as the safe default.

## Policy Requirements

The exact final config keys can be designed later, but the control-plane contract must support separate policy gates for:

1. whether host orchestrators may dispatch world work at all,
2. allowed `backend_id`s for world dispatch,
3. allowed modes: `ephemeral`, `retained`,
4. allowed actions: `run`, `spawn`, `continue`, `inspect`, `cancel`, `stop`,
5. whether steering is restricted to the same `orchestration_session_id`,
6. whether steering is restricted to the same authoritative world binding,
7. whether request-time capability narrowing is allowed.

These gates belong to the control plane, not the execution plane.

## Trace and Audit Expectations

Every host-to-world dispatch request should be explanation-ready.

Minimum join fields:

1. `request_id`
2. `orchestration_session_id`
3. `caller_participant_id`
4. `target_backend_id`
5. `target_participant_id` when applicable
6. `world_id`
7. `world_generation`
8. `mode`
9. `action`

The runtime must be able to answer:

1. who requested this world work,
2. under which orchestration session,
3. in which world binding,
4. in which lifecycle mode,
5. with which exact target identity,
6. with what terminal outcome.

## Forking Semantics

This design introduces world-worker fork as a control-plane action, but it must stay distinct from the already-landed public host-session `fork`.

### 1. World-worker fork is not host-session successor fork

The repo already freezes public `substrate agent fork --session <orchestration_session_id>` as a host-session successor allocator. That meaning stays intact.

World-worker fork in this design is different:

1. it does not allocate a new host orchestration session,
2. it allocates a new world worker under the same durable orchestration session,
3. it preserves exact parent-session authority and authoritative world binding,
4. it records worker lineage explicitly.

### 2. World-worker fork requires explicit policy and action support

World-worker fork is not implicit continuation and it is not heuristic auto-splitting by runtime default.

Allowed paths:

1. the host orchestrator issues explicit `fork_world_worker`,
2. a retained world worker emits a fork request or fork recommendation message,
3. the host orchestrator accepts that request and issues explicit `fork_world_worker`.

Disallowed paths:

1. silent self-replication by a world worker without policy-granted permission,
2. runtime auto-forking based only on prompt text or execution heuristics,
3. in-place mutation of one retained worker into two without distinct lineage and identity.

### 3. Child worker identity and lineage

Every world-worker fork must persist:

1. `forked_from_participant_id`,
2. source `backend_id`,
3. source `world_id`,
4. source `world_generation`,
5. explicit `fork_reason`,
6. explicit child mission or task handoff summary.

The child worker is a new worker, not a resumed alias of the source.

## v1 Contract Summary

The first shippable host-orchestrator world-dispatch contract should therefore be:

1. internal only,
2. control-plane only,
3. exact-identity only,
4. explicit-mode only,
5. fail-closed by default,
6. agent-native on the world execution side.

## Follow-On Design Dependencies

This document intentionally leaves three follow-on designs to separate docs:

1. world participant lifecycle model for `ephemeral` versus `retained`,
2. durable orchestration obligation-ledger plus review/attach projection contracts for world-to-host re-engagement,
3. policy matrix for host-to-world steering.

Those follow-on docs should consume this dispatch contract rather than redefining it.
