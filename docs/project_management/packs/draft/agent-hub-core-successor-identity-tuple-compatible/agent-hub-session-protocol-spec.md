# agent-hub-session-protocol-spec

This file is the single authoritative schema inventory for ADR-0044 session semantics inside this feature pack. It defines the backend capability descriptor, the hub-owned session-handle object, the lifecycle state machine, the host-orchestrator and world-member routing rules, the shared-world reuse boundary, and the exact machine-readable object shapes projected into `substrate agent list --json` and `substrate agent status --json`.

Authoritative inputs:
- `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/spec_manifest.md`
- `docs/project_management/_archived/next/agent_hub_core/decision_register.md`
- `crates/common/src/identity.rs`
- `crates/common/src/agent_events.rs`

## Authority boundary

- This file is authoritative for:
  - the canonical backend capability descriptor consumed by the hub
  - the canonical hub-owned session-handle schema
  - the lifecycle state names and legal transitions for agent sessions
  - the exact host-orchestrator eligibility rule
  - the exact world-scoped member routing, world reuse, and world restart invalidation rules
  - the exact machine-readable status objects reused by `substrate agent list --json` and `substrate agent status --json`
  - the exact structured event envelope reused between agent backends and the hub
- `contract.md` remains authoritative for operator-facing command behavior, exit codes, and human-readable render rules.
- `policy-spec.md` remains authoritative for ordered deny evaluation and fail-closed control-plane outcomes.
- `telemetry-spec.md` remains authoritative for trace placement, nested-record publication, and redaction rules after the hub projects protocol objects into canonical trace.

## Canonical tokens

- `backend_id` uses the derived adapter grammar `<kind>:<agent_id>`.
- `execution.scope` is exactly `host` or `world`.
- The pure-agent protocol token is exactly `uaa.agent.session`.
- Pure-agent identity uses:
  - `client=<agent_id>`
  - `router=agent_hub`
  - `protocol=uaa.agent.session`
  - `provider`: omitted
  - `auth_authority`: omitted
- Nested gateway-backed LLM activity stays on a separate correlated record with:
  - `router=substrate_gateway`
  - `provider`: required
  - `auth_authority`: required

## Canonical backend capability descriptor

The hub consumes one canonical capability descriptor per effective inventory item.

### `AgentBackendCapabilityDescriptorV1`

```json
{
  "agent_id": "codex",
  "backend_id": "cli:codex",
  "kind": "cli",
  "execution": {
    "scope": "world"
  },
  "protocol": "uaa.agent.session",
  "capabilities": {
    "session_start": true,
    "session_resume": true,
    "session_fork": true,
    "session_stop": true,
    "status_snapshot": true,
    "event_stream": true,
    "llm": true,
    "mcp_client": false
  }
}
```

Field rules:
- `agent_id` is the effective inventory id after workspace-over-global overlay resolution.
- `backend_id` is derived from `kind` plus `agent_id`. Backends do not publish an independent backend id field.
- `kind` is the effective inventory backend kind token.
- `execution.scope` is the effective runtime placement declared by inventory and is exactly `host` or `world`.
- `protocol` is exactly `uaa.agent.session`.
- `capabilities.session_start`, `session_resume`, `session_fork`, `session_stop`, `status_snapshot`, and `event_stream` are required booleans.
- `capabilities.llm` and `capabilities.mcp_client` are required booleans and feed the list-surface capability summary.
- Unknown top-level fields are invalid for `AgentBackendCapabilityDescriptorV1`.

Projection rule:
- `substrate agent list --json` projects each effective descriptor into the `agents[*]` shape defined in `contract.md`.
- The list surface is a projection. It does not widen, rename, or reinterpret the underlying descriptor fields defined here.

## Session-handle schema

The hub owns the session handle. Backends do not invent or persist an alternate handle format.

### `AgentSessionHandleV1`

```json
{
  "session_handle_id": "ash_001",
  "orchestration_session_id": "sess_001",
  "agent_id": "codex",
  "backend_id": "cli:codex",
  "role": "member",
  "protocol": "uaa.agent.session",
  "execution": {
    "scope": "world"
  },
  "state": "ready",
  "opened_at": "2026-04-24T18:30:00Z",
  "last_transition_at": "2026-04-24T18:30:00Z",
  "world_id": "world-17",
  "world_generation": 3,
  "parent_session_handle_id": null,
  "resumed_from_session_handle_id": null
}
```

Field rules:
- `session_handle_id` is hub-assigned and unique within the repository trace domain.
- `orchestration_session_id` groups the orchestrator session plus every member session opened under the same control-plane session.
- `agent_id`, `backend_id`, `protocol`, and `execution.scope` are copied from the canonical capability descriptor and never drift during the life of one handle.
- `role` is exactly `orchestrator` or `member`.
- `state` is one of the lifecycle state names defined below.
- `opened_at` is the first successful allocation time for this handle.
- `last_transition_at` is updated on every state transition.
- `world_id` and `world_generation` are both present or both absent.
- `world_id` and `world_generation` are required when `execution.scope=world`.
- `world_id` and `world_generation` are omitted when `execution.scope=host`.
- `parent_session_handle_id` is non-null only for a forked child handle.
- `resumed_from_session_handle_id` is non-null only when a new handle replaced an invalidated world-scoped handle after a hub-driven restart.

## Lifecycle state machine

### State names

- `allocating`
  - The hub accepted the request and is waiting for backend or world allocation to complete.
- `ready`
  - The handle is live and ready to accept work. No task is currently executing.
- `running`
  - The handle is actively executing or streaming work.
- `restarting`
  - The hub is replacing a world-scoped handle because the shared world restarted.
- `stopping`
  - The hub issued a stop and is waiting for terminal confirmation.
- `stopped`
  - Terminal state after a successful stop.
- `failed`
  - Terminal state after an execution or lifecycle failure that leaves the handle unusable.
- `invalidated`
  - Terminal state after world restart or orchestrator teardown made the handle unusable without an explicit stop.

### Legal transitions

| From | To | Trigger |
| --- | --- | --- |
| none | `allocating` | `start_session` or `fork_session` begins |
| `allocating` | `ready` | backend advertises a usable session |
| `allocating` | `failed` | backend or world allocation fails |
| `ready` | `running` | work dispatch begins |
| `running` | `ready` | work dispatch completes successfully |
| `ready` | `restarting` | shared world drift requires restart |
| `running` | `restarting` | shared world drift requires restart after the in-flight unit ends or is cancelled |
| `restarting` | `invalidated` | old world-scoped handle is retired |
| `restarting` | `failed` | restart attempt fails closed |
| `ready` | `stopping` | `stop_session` begins |
| `running` | `stopping` | `stop_session` begins while work is active |
| `stopping` | `stopped` | backend confirms stop |
| `ready` | `failed` | backend becomes unusable |
| `running` | `failed` | backend execution fails irrecoverably |

Non-negotiable transition rules:
- A world restart invalidates every live world-scoped member handle tied to the previous `world_id`.
- A replacement handle created after restart MUST use a new `session_handle_id`.
- The replacement handle MUST copy `orchestration_session_id`, `agent_id`, `backend_id`, `role`, and `protocol`.
- The replacement handle MUST publish `resumed_from_session_handle_id=<old session_handle_id>`.
- The replacement handle MUST publish the new `world_id` plus `world_generation = previous_world_generation + 1`.
- Host-scoped handles never transition through `restarting`.

## Host-orchestrator eligibility

The selected orchestrator is the effective inventory item named by `agents.hub.orchestrator_agent_id`.

An inventory item is orchestrator-eligible only when all of these checks pass:
1. the effective inventory contains the referenced `agent_id`
2. the effective inventory item is enabled
3. `execution.scope=host`
4. `protocol=uaa.agent.session`
5. `capabilities.session_start=true`
6. `capabilities.session_resume=true`
7. `capabilities.session_fork=true`
8. `capabilities.session_stop=true`
9. `capabilities.status_snapshot=true`
10. `capabilities.event_stream=true`
11. the derived `backend_id` is allowed by `agents.allowed_backends`

Failure rules:
- The hub never falls back to a different orchestrator.
- A world-scoped candidate is invalid even when every other capability check passes.
- `substrate agent doctor` and every control-plane entrypoint fail closed on any orchestrator-eligibility failure.

## Member dispatch model

- Every non-orchestrator agent session opened by the hub uses `role=member`.
- A member session inherits the parent `orchestration_session_id`.
- A host-scoped member omits `world_id` and `world_generation`.
- A world-scoped member requires `world_id` and `world_generation` from the shared-world contract below.
- `backend_id` never changes when a session moves between `ready`, `running`, `stopping`, or `failed`.
- Role assignment is not encoded into `backend_id`.

## Shared-world reuse and restart contract

- The v1 default is exactly one shared `world_id` per `orchestration_session_id` for all world-scoped member handles.
- Host-scoped orchestrator handles do not join that shared world and never publish `world_id` or `world_generation`.
- `world_generation` starts at `0` when the first shared world is allocated for an orchestration session.
- The shared world is reused while these inputs remain unchanged:
  - workspace root
  - effective policy snapshot
  - world-fs policy
  - network policy
  - requested execution scope
- Drift handling is controlled only by `agents.hub.world_restart.on_drift`.
- When `agents.hub.world_restart.on_drift=auto_restart`:
  - the hub MUST allocate a new shared world
  - the hub MUST increment `world_generation` by exactly `1`
  - the hub MUST invalidate every live world-scoped handle bound to the previous world
  - the hub MUST create replacement handles before more work is dispatched
- When `agents.hub.world_restart.on_drift=fail_closed`:
  - the hub MUST NOT restart implicitly
  - the hub MUST leave the prior world-scoped handles in `invalidated` or `failed`
  - the hub MUST require an explicit restart path before more world-scoped work begins

## Status exchange objects

### `AgentInventoryItemV1`

This is the exact object shape for each `substrate agent list --json` row.

```json
{
  "agent_id": "codex",
  "backend_id": "cli:codex",
  "kind": "cli",
  "execution": {
    "scope": "world"
  },
  "role": null,
  "capabilities_summary": {
    "llm": true,
    "mcp_client": false
  },
  "eligibility": {
    "state": "allowed",
    "reason": null
  },
  "protocol": "uaa.agent.session"
}
```

Field rules:
- `capabilities_summary.llm` is copied from `AgentBackendCapabilityDescriptorV1.capabilities.llm`.
- `capabilities_summary.mcp_client` is copied from `AgentBackendCapabilityDescriptorV1.capabilities.mcp_client`.
- `role` is `orchestrator` only for the selected orchestrator. Every other row uses `null`.
- `eligibility.state` is `allowed` or `denied`.
- `eligibility.reason` is non-null exactly when `eligibility.state=denied`.

### `AgentSessionStatusV1`

This is the exact object shape for each pure-agent entry in `substrate agent status --json`.

```json
{
  "orchestration_session_id": "sess_001",
  "agent_id": "codex",
  "backend_id": "cli:codex",
  "client": "codex",
  "router": "agent_hub",
  "protocol": "uaa.agent.session",
  "execution": {
    "scope": "world"
  },
  "role": "member",
  "last_event_at": "2026-04-24T18:30:00Z",
  "world_id": "world-17",
  "world_generation": 3
}
```

Field rules:
- `client` is exactly the emitting session `agent_id`.
- `router` is exactly `agent_hub`.
- `protocol` is exactly `uaa.agent.session`.
- `role` is `orchestrator` or `member`.
- `last_event_at` is the latest accepted structured event timestamp for that handle.
- `world_id` and `world_generation` are both required when `execution.scope=world`.
- `world_id` and `world_generation` are both omitted when `execution.scope=host`.
- `provider` and `auth_authority` never appear on `AgentSessionStatusV1`.

### `NestedLlmStatusRecordV1`

This is the exact object shape for each nested record in `substrate agent status --json`.

```json
{
  "parent": {
    "orchestration_session_id": "sess_001",
    "agent_id": "codex"
  },
  "backend_id": "cli:codex",
  "client": "codex",
  "router": "substrate_gateway",
  "provider": "openai",
  "auth_authority": "codex_subscription",
  "protocol": "openai.responses"
}
```

Field rules:
- `parent.orchestration_session_id` and `parent.agent_id` identify the pure-agent session that triggered the nested request.
- `backend_id` remains the parent agent backend id.
- `client` remains the parent agent id.
- `router` is exactly `substrate_gateway`.
- `provider` and `auth_authority` are required.
- `world_id` and `world_generation` never appear on `NestedLlmStatusRecordV1`.

### `AgentStatusResponseV1`

This is the exact top-level `substrate agent status --json` shape.

```json
{
  "disabled": false,
  "scope_filter": "any",
  "role_filter": null,
  "orchestrator_agent_id": "claude_code",
  "sessions": [],
  "nested_llm_records": []
}
```

Field rules:
- `scope_filter` is exactly `host`, `world`, or `any`.
- `role_filter` is `null`, `orchestrator`, or `member`.
- `sessions[*]` uses `AgentSessionStatusV1` exactly.
- `nested_llm_records[*]` uses `NestedLlmStatusRecordV1` exactly.
- `sessions` sort by `orchestration_session_id`, then `agent_id`, ascending byte order.
- `nested_llm_records` sort by `parent.orchestration_session_id`, then `parent.agent_id`, then nested request correlation id ascending byte order.

## Structured event exchange

Pure-agent session events exchanged between agent backends and the hub use the ADR-0017 `AgentEvent` envelope as implemented in `crates/common/src/agent_events.rs`.

Required envelope fields for pure-agent session events:
- `ts`
- `kind`
- `data`
- `agent_id`
- `orchestration_session_id`
- `run_id`

Required or conditional fields:
- `backend_id` is required for v1 agent-hub session events because the emitting backend is always known.
- `role` is required on orchestrator-emitted control-plane alerts and optional on member progress events.
- `world_id` is required when the emitting handle uses `execution.scope=world`.
- `identity_tuple` is required on pure-agent session events and MUST be:
  - `client=<agent_id>`
  - `router=agent_hub`
  - `protocol=uaa.agent.session`
  - `provider`: omitted
  - `auth_authority`: omitted
- `placement_posture` is optional in the exchange envelope and never changes the routing or eligibility rules defined in this file.

Event-kind rules:
- `registered`, `status`, `task_start`, `task_progress`, `task_end`, and `alert` are valid for pure-agent sessions.
- `pty_data` is valid only when the backend exposes a streamed task execution.
- Nested gateway-backed LLM events remain on the separate gateway event path and do not mutate the pure-agent event envelope.

## Ordering, retry, timeout, and idempotency

Ordering rules:
- The hub does not promise one global total order across all agents.
- For a single `session_handle_id`, events are processed in emission order.
- A `registered` event, if emitted, precedes every later event for the same handle.
- A terminal `task_end` or `alert` that closes an in-flight unit precedes any transition back to `ready`.

Retry rules:
- Capability discovery is retryable and side-effect free.
- `start_session` is retryable only while no terminal handle exists for the same `(orchestration_session_id, agent_id, role)` tuple.
- `stop_session` is retryable and returns the same terminal state when the handle is already `stopped`, `failed`, or `invalidated`.
- `fork_session` is intentionally non-idempotent. Each successful fork creates a new `session_handle_id`.

Idempotency rules:
- Repeating `start_session` for a live `(orchestration_session_id, agent_id, role)` tuple returns the existing live handle instead of creating a second live handle.
- Repeating `resume_session` for the same live `session_handle_id` returns that same handle.
- Repeating `stop_session` for a terminal handle returns the same terminal handle and does not create new events other than an at-most-once confirmation.

Timeout rules:
- Session allocation timeout is evaluated in `allocating`.
- Dispatch timeout is evaluated in `running`.
- A timeout transitions the handle to `failed` unless the world restart path first moved it to `restarting`.
