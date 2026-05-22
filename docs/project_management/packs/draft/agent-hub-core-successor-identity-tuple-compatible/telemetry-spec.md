# agent-hub-core-successor-identity-tuple-compatible — telemetry spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

- This spec is authoritative for pure-agent and nested gateway-backed telemetry publication for ADR-0044.
- This spec owns:
  - the record-family split between pure-agent orchestration records and nested gateway-backed LLM records
  - top-level field placement for `client`, `router`, `protocol`, `backend_id`, `provider`, `auth_authority`, `world_id`, and `world_generation`
  - restart-alert publication for shared-world drift
  - redaction posture for nested request metadata
- This spec does not redefine the ADR-0017 event-envelope required keys or the ADR-0028 canonical correlation vocabulary.

Canonical references:
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md`
- `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- `docs/TRACE.md`

## Stability guarantees

- Publication is additive only.
- Existing ADR-0017 record families stay authoritative:
  - `event_type = "agent_event"`
  - `kind = registered|status|task_start|task_progress|task_end|pty_data|alert`
- Existing ADR-0028 correlation keys keep their current meanings.
- Consumers that ignore the additive tuple-compatible fields continue to parse canonical trace records.

## Record-family split

### Pure-agent orchestration record

A pure-agent orchestration record describes hub-owned agent control-plane or event-plane activity without nested LLM fulfillment.

Required top-level fields:

- ADR-0017 envelope and correlation keys:
  - `event_type = "agent_event"`
  - `ts`
  - `session_id`
  - `component = "agent-hub"`
  - `kind`
  - `orchestration_session_id`
  - `run_id`
  - `agent_id`
  - `backend_id`
  - `data`
- ADR-0044 tuple-compatible fields:
  - `client`
  - `router = "agent_hub"`
  - `protocol = "substrate.agent.session"`

Conditional top-level fields:

- `role`
- `thread_id`
- `cmd_id`
- `span_id`
- `channel`
- `world_id`
- `world_generation`

Omission rules:

- `provider` is omitted.
- `auth_authority` is omitted.
- `world_id` and `world_generation` are both omitted when the emitting record is host-scoped.
- `world_id` and `world_generation` are both present when the emitting record is world-scoped.

### Nested gateway-backed LLM record

A nested gateway-backed LLM record describes a distinct gateway fulfillment step triggered by an agent.

Required top-level fields:

- ADR-0017 envelope and correlation keys:
  - `event_type = "agent_event"`
  - `ts`
  - `session_id`
  - `component = "agent-hub"`
  - `kind`
  - `orchestration_session_id`
  - `run_id`
  - `agent_id`
  - `backend_id`
  - `data`
- nested-correlation fields:
  - `parent_run_id`
- ADR-0044 tuple-compatible fields:
  - `client`
  - `router = "substrate_gateway"`
  - `protocol`
  - `provider`
  - `auth_authority`

Conditional top-level fields:

- `thread_id`
- `cmd_id`
- `span_id`
- `channel`

Omission rules:

- `world_id` is omitted.
- `world_generation` is omitted.
- `role` is omitted because the nested record inherits agent identity from the parent pure-agent record and is not a second agent-role assignment surface.

## Correlation rules

- A nested record is correlated to its parent pure-agent record through this exact set:
  - shared `orchestration_session_id`
  - shared `agent_id`
  - shared `backend_id`
  - `parent_run_id` pointing to the parent pure-agent `run_id`
- `client` on the nested record stays equal to the parent agent runtime id.
- `router`, `provider`, `auth_authority`, and `protocol` on the nested record describe only the nested gateway-backed request.
- The nested record never mutates, widens, or retroactively annotates the parent pure-agent record.
- Status consumers use `parent_run_id` for correlation validation only. A nested row is emitted only when its `parent_run_id` matches the winning selected pure-agent `run_id` for that `(orchestration_session_id, agent_id)` pair. Nested rows tied to older historical pure-agent runs are ignored as stale history, and malformed selected-surface rows fail closed.

## `world_generation` publication path

This spec fixes the exact publication path for `world_generation` so status, alert, and trace surfaces agree.

### Steady-state pure-agent records

- `world_generation` is a top-level integer field on a pure-agent orchestration record.
- `world_generation` is emitted only when `world_id` is emitted.
- `world_generation` starts at `0` for the first shared world allocated to an `orchestration_session_id`.
- `world_generation` increments by exactly `1` when the hub completes an automatic world restart for that orchestration session.

### `world_restarted` alert records

When the hub auto-restarts a shared world, it emits one `agent_event` record with:

- `kind = "alert"`
- top-level fields:
  - `orchestration_session_id`
  - `run_id`
  - `agent_id`
  - `backend_id`
  - `client`
  - `router = "agent_hub"`
  - `protocol = "substrate.agent.session"`
  - `role = "orchestrator"`
  - `world_id = <new_world_id>`
  - `world_generation = <new_world_generation>`
- `data` fields:
  - `code = "world_restarted"`
  - `reason`
  - `on_drift = "auto_restart"`
  - `previous_world_id`
  - `new_world_id`
  - `previous_world_generation`
  - `new_world_generation`
  - `message`

The top-level `world_id` and `world_generation` always identify the active replacement world. Previous values stay inside `data.previous_*`.

### `world_restart_required` alert records

When drift is detected under fail-closed posture, the hub emits one `agent_event` record with:

- `kind = "alert"`
- top-level fields:
  - `orchestration_session_id`
  - `run_id`
  - `agent_id`
  - `backend_id`
  - `client`
  - `router = "agent_hub"`
  - `protocol = "substrate.agent.session"`
  - `role = "orchestrator"`
- `world_id` and `world_generation`:
  - both are present when the invalidated world is still known at alert time
  - both are omitted when no concrete world allocation completed before the failure
- `data` fields:
  - `code = "world_restart_required"`
  - `reason`
  - `on_drift = "fail_closed"`
  - `message`

## Canonical trace placement

The shell persists both record families into `~/.substrate/trace.jsonl` as top-level additive metadata on `event_type = "agent_event"` records.

Trace rules:

- `backend_id` remains adapter/backend identity only.
- `client`, `router`, `protocol`, `provider`, and `auth_authority` remain top-level fields and never move under `data`.
- `world_id` and `world_generation` remain top-level fields and never move under `data`, except that historical values for restart alerts also appear under `data.previous_*` and `data.new_*`.
- `provider` and `auth_authority` appear only on nested gateway-backed LLM records.
- Pure-agent records never synthesize `provider` or `auth_authority`.

## Human-readable status and diagnostics alignment

- Human-readable `substrate agent status` renders nested gateway-backed rows as a separate section.
- The machine-readable trace split must match that operator-visible split exactly:
  - pure-agent status rows correspond to pure-agent orchestration records
  - nested status rows correspond to nested gateway-backed LLM records
- A consumer must never infer nested gateway fulfillment by checking for `provider` on a pure-agent record because that field is never published there.

## Redaction rules

- `client`, `router`, `provider`, and `auth_authority` publish normalized lowercase snake_case ids only.
- `protocol` publishes normalized lowercase dotted ids only.
- `auth_authority` identifies the authority class only.
- Telemetry never emits:
  - access tokens
  - API keys
  - session cookies
  - raw credential files
  - raw credential paths
  - gateway request bodies
  - provider-specific secret headers
  - secret-bearing URL query parameters
- `data` on nested gateway-backed records may contain safe operational metadata only:
  - stable request classification
  - retry count
  - safe denial reason
  - safe completion summary

## Acceptance criteria

- Pure-agent orchestration records and nested gateway-backed LLM records are always published as separate `agent_event` records.
- Pure-agent records always publish `client`, `router = "agent_hub"`, `protocol = "substrate.agent.session"`, and omit `provider` plus `auth_authority`.
- Nested gateway-backed records always publish `client`, `router = "substrate_gateway"`, `protocol`, `provider`, `auth_authority`, and omit `world_id` plus `world_generation`.
- `world_generation` is always top-level on steady-state world-scoped pure-agent records and on restart alerts that know the active world.
- Restart alerts always keep previous-generation values inside `data.previous_*` and active replacement values at the top level plus `data.new_*`.
- No canonical trace record emits secret-bearing nested request metadata.
