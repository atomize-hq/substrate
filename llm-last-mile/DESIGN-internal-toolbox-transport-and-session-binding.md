# Design: Internal Toolbox Transport And Session Binding

Status: draft design input. This document freezes the current landed internal toolbox transport boundary for host-orchestrator world dispatch after Slices `32` through `51`. It is not a public CLI contract, not an MCP wire-contract commitment, and not a final agent-visible tool UX. It defines the session-scoped transport, framing, identity ownership, and fail-closed binding rules that future agent-facing adapters must reuse.

## Why This Doc Exists

The earlier design stack froze:

1. host-to-world dispatch verbs,
2. retained-worker messaging,
3. steering policy,
4. world-worker lifecycle,
5. durable obligation, attach, and inbox behavior.

What those docs did not yet freeze is the now-landed transport truth underneath the internal caller surface:

1. the toolbox endpoint is already real,
2. the endpoint is already session-scoped,
3. the request contract is already typed,
4. the response path is already framed and event-capable,
5. the transport is still internal-only.

Without a dedicated transport/binding design:

1. later agent-facing tool-surface work will keep re-arguing whether the transport exists,
2. MCP placeholder ADR language can be mistaken for current runtime truth,
3. adapters may incorrectly treat runtime-owned identity fields as model-authored input,
4. future tool-surface work may reopen exact session/world binding semantics that are already landed.

This document closes that gap.

## Relationship To Existing Decisions

This design composes with:

1. [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md): the verb set, action/mode rules, and typed dispatch intent remain the semantic contract above this transport.
2. [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md): retained-worker follow-up semantics remain distinct from transport framing.
3. [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md): policy still gates which requests may traverse the transport.
4. [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md): lifecycle state stays separate from transport/event framing.
5. [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md) and [DESIGN-durable-orchestration-notification-inbox-contract.md](./DESIGN-durable-orchestration-notification-inbox-contract.md): durable review and inbox truth remain downstream of accepted control-plane actions rather than properties of the transport itself.
6. [SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md](./SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md): the first internal orchestrator-only caller surface is already landed.
7. [SPEC-47-internal-active-ephemeral-task-identity-and-inspect-cancel-widening.md](./SPEC-47-internal-active-ephemeral-task-identity-and-inspect-cancel-widening.md) and [SPEC-48](./SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md) through [SPEC-51](./SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md): later slices widened identity, obligation, and host-side continuation semantics without replacing this transport boundary.
8. [`docs/USAGE.md`](../docs/USAGE.md): toolbox `status` and `env` are already operator-visible introspection surfaces over this internal transport posture.
9. [ADR-0026](../docs/adr/draft/ADR-0026-orchestration-toolbox-mcp.md) and [ADR-0045](../docs/adr/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md): these remain queued placeholders and do not override the landed transport truth frozen here.

## Problem Statement

How might Substrate describe the already-landed internal toolbox transport so that:

1. current runtime truth is explicit,
2. future agent-facing tool work does not reinvent the internal wire,
3. session/world binding remains fail-closed,
4. runtime-owned identity fields are not pushed onto the model,
5. later adapters can expose tools without turning the transport into a public execution plane?

## Frozen Direction

This design freezes the following:

1. the internal toolbox transport already exists and is part of landed runtime truth,
2. the current internal wire contract is newline-delimited JSON over a session-scoped local endpoint, not MCP,
3. the endpoint is keyed by exact `orchestration_session_id`,
4. endpoint discovery/export is runtime-owned and only allowed when a live host-scoped orchestrator session exists,
5. request validation remains exact-identity and fail-closed:
   - `orchestration_session_id`
   - `caller_participant_id`
   - `target_backend_id`
   - `world_id`
   - `world_generation`
   - `target_participant_id` and `task_run_id` when the action requires them,
6. the transport is event-capable and not limited to one blocking result frame,
7. the transport remains internal-only and must not be mistaken for a public human CLI.

## Non-Goals

This design does not:

1. define the final agent-visible tool surface,
2. commit Substrate to MCP as the internal wire protocol,
3. define a public `substrate agent toolbox <verb>` execution CLI,
4. replace the existing `WorldDispatchRequestV1` / `WorldDispatchOutcomeV1` contract family,
5. define cross-host or federated transport semantics,
6. widen the current Unix-first transport floor into a full Windows/WSL caller design.

## Core Principle

The transport belongs to the runtime, not to the model.

The host AI may express intent, but the runtime owns:

1. endpoint discovery,
2. authoritative session identity,
3. authoritative caller identity,
4. authoritative world binding,
5. framing and terminality rules.

The model should never be expected to synthesize those from prose alone.

## Current Repo Floor

### 1. Session-scoped endpoint

The canonical endpoint is derived from:

1. `SUBSTRATE_HOME/run/agent-toolbox/<orchestration_session_id>.sock`,
2. with a deterministic `/tmp/substrate-agent-toolbox/<orchestration_session_id>.sock` fallback when the preferred Unix path is too long.

This keeps endpoint identity session-local and deterministic.

### 2. Operator-visible introspection only

The current operator-visible toolbox surfaces are:

1. `substrate agent toolbox status`
2. `substrate agent toolbox env`

Those surfaces:

1. project posture and endpoint availability,
2. emit `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT` and `SUBSTRATE_AGENT_TOOLBOX_VERSION` only for a live allowed session,
3. do not provide a public human execution surface for `run_world_task`, `spawn_world_worker`, or later verbs.

### 3. Unix-first transport floor

The landed caller path is Unix-socket based.

The config model can name `tcp`, but the current runtime truth remains:

1. deterministic pre-runtime TCP loopback discovery is not frozen,
2. TCP is therefore not yet supported as the real caller path.

## Transport Framing

### Request shape

Each connection sends exactly one newline-delimited JSON request using the existing conceptual shape:

```text
WorldDispatchRequestV1
```

The transport does not define a second ad hoc payload family.

### Response shape

The server writes one or more newline-delimited JSON frames:

1. zero or more `event` frames,
2. exactly one terminal `result` frame.

Conceptual result frame families:

```text
{
  "version": 1,
  "frame_kind": "event" | "result",
  ...
}
```

Terminal success and failure remain explicit:

1. `result` + `ok=true` + typed `outcome`,
2. `result` + `ok=false` + explanation-ready `error`.

### Terminality rule

The stream is not complete until a terminal `result` frame is emitted.

If the owner disappears before a terminal frame is written, the transport must fail closed rather than silently treating the partial exchange as success.

## Event And Receipt Semantics

### `run_world_task`

`run_world_task` may emit a non-terminal event before the final outcome:

1. `frame_kind=event`
2. `event_kind=task_run_id_registered`
3. `action=run_world_task`
4. authoritative `task_run_id`

This freezes an important truth:

1. the internal toolbox is already receipt/event-capable,
2. the caller path cannot be modeled honestly as a single synchronous return value only.

### `spawn_world_worker`

`spawn_world_worker` remains receipt-oriented:

1. the runtime validates the upstream retained-worker `registered` event,
2. Substrate returns an authoritative retained-worker receipt,
3. the transport does not itself become the worker’s ongoing conversational execution channel.

### Later verbs

`continue_world_worker`, `inspect_world_worker`, `fork_world_worker`, `cancel_world_work`, and `stop_world_worker` all reuse the same transport family and terminal framing rules even when their outcome shapes differ.

## Identity Ownership

The transport accepts typed request fields, but future callers must distinguish between:

### Runtime-owned truth

1. `orchestration_session_id`
2. `caller_participant_id`
3. authoritative `world_id`
4. authoritative `world_generation`
5. endpoint path/version

### Caller intent

1. `action`
2. `mode`
3. `target_backend_id`
4. typed payload content
5. exact retained `target_participant_id` or `task_run_id` when the action requires them

### Hard rule

Future agent-facing adapters must not require the model to reconstruct runtime-owned truth from prompt text or stale cached values.

## Binding And Authorization Rules

This transport remains constrained by already-landed control-plane boundaries:

1. same orchestration session by default,
2. same authoritative world binding by default,
3. exact backend targeting,
4. policy-gated action/mode validity,
5. orchestrator-owned caller identity.

The transport is therefore not a generic local admin socket.

It is a session-private control-plane seam.

## Failure Posture

The transport must fail closed when:

1. no live orchestrator runtime owns the session,
2. the request cannot be decoded,
3. required dispatch fields are missing,
4. exact session or world binding does not match authoritative truth,
5. the action/mode/payload combination is invalid,
6. the owner closes before a terminal response.

Explanation-ready denials are part of the contract.

## Relationship To Future Agent-Facing Surfaces

Any future agent-visible surface:

1. native runtime tool bridge,
2. family-specific tool adapter,
3. later MCP wrapper,
4. smoke/debug harness,

must wrap this transport and its request/outcome contract rather than silently inventing a parallel control plane.

The adapter may simplify what the model sees, but it must preserve:

1. exact identity,
2. terminality,
3. receipt/event semantics,
4. fail-closed behavior.

## Summary

The transport frozen here is:

1. already landed,
2. session-scoped,
3. Unix-first,
4. newline-delimited JSON,
5. event-capable,
6. exact-identity and fail-closed,
7. internal-only.

That is the current truth future orchestrator-facing tool work must build on.
