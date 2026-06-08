# Design: Host Orchestrator Tool Invocation Surface

Status: draft design input. This document defines the missing agent-facing tool invocation surface above the landed internal toolbox transport. It is not a public human CLI design, not an MCP-first commitment, and not a redesign of world dispatch runtime logic. It freezes the agent-facing boundary needed so a live host orchestrator can delegate world work, keep going, and later resume or inspect results without shelling back into `substrate` as a subprocess.

## Why This Doc Exists

The repo now already has:

1. a live internal toolbox transport,
2. a typed internal world-dispatch request contract,
3. landed verbs through Family 1 and Family 2,
4. durable host-session authority,
5. parked/resumable host ownership,
6. retained-worker lifecycle, obligation, and inbox semantics.

What it still does not have is the actual surface the host AI uses inside its own session:

1. a system prompt alone is not a callable tool interface,
2. a subprocess `substrate ...` CLI hop is not the intended in-session control path,
3. MCP may be a later adapter, but it is not required to describe the first honest agent-facing boundary.

Without this design:

1. the repo can overestimate how close it is to a real orchestrator experience,
2. transport truth and agent-visible tool truth can get conflated,
3. MCP placeholder language can pressure the architecture into a premature async/tooling commitment,
4. runtime-family support for `codex` and `claude_code` can drift into separate bespoke behaviors.

This document closes that gap.

## Relationship To Existing Decisions

This design composes with:

1. [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md): the tool surface must preserve the existing verb set and action/mode semantics.
2. [DESIGN-internal-toolbox-transport-and-session-binding.md](./DESIGN-internal-toolbox-transport-and-session-binding.md): the landed internal toolbox transport remains the source of truth below the tool surface.
3. [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md): retained-worker message semantics remain unchanged; the missing work is how the host agent invokes them.
4. [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md): parked/resumable retained work and active ephemeral identity already exist and should shape the tool model.
5. [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md), [DESIGN-router-daemon-attach-trigger-integration.md](./DESIGN-router-daemon-attach-trigger-integration.md), and [DESIGN-durable-orchestration-notification-inbox-contract.md](./DESIGN-durable-orchestration-notification-inbox-contract.md): the host may park and later re-engage through durable state rather than one long blocking tool call.
6. [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md): the current shell-owned orchestrator runtime families are `codex` and `claude_code`.
7. [ADR-0026](../docs/adr/draft/ADR-0026-orchestration-toolbox-mcp.md) and [ADR-0045](../docs/adr/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md): these remain queued placeholders and should not force an MCP-first v1 tool model.

## Problem Statement

How might Substrate expose the landed internal control plane to a live host orchestrator so that:

1. the host AI can actually call world-dispatch actions from inside its own session,
2. the host AI does not need to shell back into `substrate` as a subprocess,
3. the runtime can populate exact session/caller/world identity automatically,
4. long-lived or resumable work returns receipts and durable handles instead of forcing one blocking tool call,
5. both supported runtime families can share one semantic contract even if their integration mechanics differ?

## Frozen Direction

This design freezes the following:

1. the host orchestrator needs a first-class tool invocation surface inside the live session,
2. the first honest v1 is Substrate-native and adapter-based, not prompt-only and not subprocess-CLI-based,
3. the landed internal toolbox transport remains the single source of truth underneath that surface,
4. runtime-owned adapters populate authoritative session, caller, and world-binding fields instead of asking the model to invent them,
5. the tool model is receipt-oriented and resumable rather than “one tool call must block until all world work is finished,”
6. the agent-visible tool set should map onto the existing dispatch verbs instead of inventing a second vocabulary,
7. MCP may be added later as an adapter or export surface, but v1 must not depend on MCP-specific async/task support.

## Non-Goals

This design does not:

1. define a public human CLI for the internal verbs,
2. make the model open the toolbox socket directly,
3. replace the landed internal transport or dispatch contract,
4. require MCP Tasks or any other optional MCP extension in v1,
5. redesign parked/resumable orchestration state,
6. define a full plugin or marketplace UX for runtime families.

## Core Principle

The host orchestrator needs tools, not prose.

A prompt can describe:

1. what exists,
2. what actions are allowed,
3. when delegation is appropriate.

But a prompt alone cannot safely supply:

1. endpoint discovery,
2. exact runtime-owned identity,
3. framing and terminality,
4. event handling,
5. durable receipt management.

The missing piece is therefore not “better instructions.” It is a real tool boundary.

## Why Prompt-Only Is Insufficient

Even with perfect instructions, a prompt-only model would still lack:

1. a sanctioned way to send the request,
2. automatic insertion of authoritative runtime fields,
3. a uniform way to receive event-plus-result responses,
4. a safe default for parked/resumable follow-up.

Prompt text is documentation.

The orchestrator needs an invocation surface.

## Why Subprocess CLI Is Not The Primary Path

For debugging or smoke tests, a wrapper CLI can still be useful.

But it is the wrong primary in-session contract because:

1. the host AI session itself is the caller that needs the capability,
2. the runtime already owns the live orchestration session and toolbox endpoint,
3. shelling back into `substrate` creates re-entrancy and environment-coupling questions that are orthogonal to the real control-plane problem,
4. a subprocess hop still would not remove the need for runtime-owned identity/binding injection.

So the CLI may exist later for diagnostics, but it should not define the architecture.

## Three-Layer Model

The honest architecture is:

### 1. Internal transport layer

The already-landed session-scoped toolbox transport and `WorldDispatchRequestV1` / `WorldDispatchOutcomeV1` contract.

### 2. Runtime-owned adapter layer

Per-runtime-family adapter logic that:

1. exposes callable tools inside the host agent session,
2. validates arguments,
3. injects runtime-owned identity and binding fields,
4. translates tool calls into internal toolbox requests,
5. translates transport results back into agent-visible structured outcomes.

### 3. Agent-visible tool layer

The actual tools the host model sees and can iterate over naturally inside its session.

This keeps internal truth stable while letting runtime families differ in exposure mechanics.

## Canonical Tool Surface

The agent-visible tool vocabulary should stay aligned with the landed verb set:

1. `run_world_task`
2. `spawn_world_worker`
3. `continue_world_worker`
4. `inspect_world_worker`
5. `fork_world_worker`
6. `cancel_world_work`
7. `stop_world_worker`

### Tool-design rule

The agent-visible tool name may differ slightly by runtime-family UX, but the semantic contract should remain one-to-one with the underlying dispatch action.

That avoids:

1. a second vocabulary,
2. duplicated policy logic,
3. ambiguous traces between tool names and dispatch actions.

## Runtime-Owned Versus Agent-Provided Fields

### Runtime-owned by default

The adapter should inject or derive:

1. `request_id`
2. `idempotency_key`
3. `orchestration_session_id`
4. `caller_participant_id`
5. authoritative `world_id`
6. authoritative `world_generation`

### Agent-provided intent

The tool call should supply only what the host model is actually deciding:

1. `action` or tool name,
2. `mode` where relevant,
3. `target_backend_id`,
4. exact `target_participant_id` or `task_run_id` when continuing/inspecting/cancelling/stopping existing work,
5. typed payload content.

### Hard rule

The host model should never be responsible for reconstructing live session identity from prose or stale memory.

## Receipt-Oriented Async Model

The tool surface must match the orchestration story the repo already wants:

1. the host agent starts world work,
2. receives authoritative receipt identity,
3. moves on to other reasoning or parks,
4. later inspects, continues, cancels, or stops using durable handles.

### `run_world_task`

The important v1 behavior is:

1. the request may register `task_run_id` before terminal completion,
2. the host receives a durable active-task identity,
3. later `inspect_world_worker` or `cancel_world_work` may target that exact active work when policy allows.

The tool contract therefore must not assume one long blocking request is the only honest interaction pattern.

### `spawn_world_worker`

The important v1 behavior is:

1. the host receives authoritative retained-worker identity,
2. the worker becomes routable retained orchestration state,
3. future follow-up goes through the same tool family rather than through a hidden side channel.

### Parked/resumable consequence

The orchestration session may safely move into:

1. ongoing host reasoning,
2. detached but durable parked state,
3. later reattach/review/resume,

without the tool layer pretending that the original tool call remained “open” the whole time.

## Runtime-Family Adapter Requirements

The repo currently supports `codex` and `claude_code` as runtime-family truth for shell-owned orchestrators.

Any runtime-family adapter must support:

1. exposing a discoverable structured tool list to the live host agent,
2. accepting structured arguments and returning structured outcomes,
3. preserving receipt-oriented workflows instead of forcing one-shot synchronous completion,
4. keeping raw endpoint details hidden from the model by default,
5. preserving exact same-session and same-world-binding semantics.

### Allowed family differences

Family-specific differences may include:

1. how tools are declared to the model,
2. how results are rendered,
3. how tool help/schema metadata is surfaced.

### Forbidden family differences

Family-specific differences must not change:

1. dispatch verb meaning,
2. exact identity semantics,
3. fail-closed behavior,
4. which state becomes durable after a successful call.

## MCP As A Later Adapter

MCP remains a valid future adapter option, but the architecture frozen here is:

1. do not make MCP the internal source of truth,
2. do not require MCP-specific long-running task semantics for the first honest tool surface,
3. if MCP is added later, it should wrap the same semantic tool contract and underlying transport rather than replacing them.

That keeps the v1 design compatible with later MCP adoption without making current progress depend on uncertain client-side async semantics.

## Summary

The tool-invocation surface frozen here is:

1. agent-visible,
2. runtime-owned,
3. Substrate-native in v1,
4. receipt-oriented and resumable,
5. layered over the already-landed internal toolbox transport,
6. shared semantically across `codex` and `claude_code`,
7. compatible with a later MCP adapter, but not blocked on one.

That is the missing design layer between the landed control-plane runtime and a real host-orchestrator experience.
