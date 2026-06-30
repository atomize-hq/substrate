# Agent Event Envelope Contract

This document is the durable contract reference for structured `agent_event` rows.

Related references:
- `docs/contracts/repl-output-routing.md`
- `docs/TRACE.md`
- `docs/adr/implemented/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/internals/agent_runtime/session_identity_and_continuity.md`

## Scope

This contract is authoritative for the structured agent event envelope used for:

- host-side structured REPL printing
- canonical `event_type="agent_event"` trace rows

Format:

- JSON object
- canonical location: `~/.substrate/trace.jsonl` or `SHIM_TRACE_LOG`

## Compatibility Policy

- additive-only for new optional fields, new `kind` values, and additive `data` fields
- existing required fields must not be removed or renamed
- consumers must ignore unknown fields

## Top-Level Envelope

All structured agent events are JSON objects with:

- `ts`
  - type: RFC3339 UTC string
  - required: yes
- `kind`
  - type: string enum
  - required: yes
  - allowed values in v1:
    - `registered`
    - `status`
    - `task_start`
    - `task_progress`
    - `task_end`
    - `pty_data`
    - `alert`
- `data`
  - type: object
  - required: yes

Required attribution and correlation fields:

- `agent_id`
- `orchestration_session_id`
- `run_id`

Optional correlation fields:

- `parent_run_id`
- `backend_id`
- `thread_id`
- `role`
- `participant_id`
- `parent_participant_id`
- `resumed_from_participant_id`
- `world_id`
- `world_generation`
- `cmd_id`
- `span_id`

Lineage and runtime-identity note:

- `participant_id`, `parent_participant_id`, `resumed_from_participant_id`, `world_id`, and
  `world_generation` are emitted here only as additive lineage/correlation fields.
- The canonical meaning and ownership boundaries for those ids live in
  `docs/internals/agent_runtime/session_identity_and_continuity.md`.

## Identity-Tuple-Compatible Metadata

Canonical envelope objects are optional and additive:

- `identity_tuple`
- `placement_posture`

When `identity_tuple` is present, canonical trace publication also projects these additive flat
fields for join/search convenience:

- `client`
- `router`
- `provider`
- `auth_authority`
- `protocol`

Boundary rules:

- pure agent/toolbox records may omit `provider` and `auth_authority`
- nested gateway-backed records may include `provider` and `auth_authority`
- `backend_id` remains adapter-only and must not be treated as semantic identity
- `placement_posture` remains the canonical execution-placement object; flat tuple fields do not
  replace it
- interpretation of these tuple fields remains owned by the identity ADR chain, not by this schema

## Routing Hint

Optional field:

- `channel`
  - producer-declared only
  - must not contain secrets
  - must not affect policy gating decisions

## Per-Kind Data Rules

For `registered`, `status`, `task_start`, `task_progress`, and `task_end`:

- `data.message` is optional and, when present, is safe to print and persist

For `pty_data`:

- `data.stream` is required and must be `stdout` or `stderr`
- `data.chunk` is required
- structured `pty_data` is not a substitute for raw PTY bytes

For `alert`:

- `data.code` is required
- `data.message` is required

Known v1 alert codes:

- `world_restarted`
- `world_restart_required`

Additional alert fields:

- `world_restarted` requires:
  - `data.reason`
  - `data.on_drift`
  - `data.previous_world_id`
  - `data.new_world_id`
  - `data.previous_world_generation`
  - `data.new_world_generation`
- `world_restart_required` requires:
  - `data.reason`
  - `data.required_action`
  - `data.on_drift`
  - `data.world_id`
  - `data.world_generation`

## Canonicalization Rules

- producers must emit all required top-level fields
- attribution and correlation fields stay at the top level
- consumers must treat the envelope as unordered JSON

## Omission Rules

- pure-agent records keep `client`, `router`, and `protocol`, and omit `provider` and
  `auth_authority`
- host-scoped pure-agent records omit `world_id` and `world_generation`
- nested gateway-backed records may add `provider` and `auth_authority`, but omit `world_id`
  and `world_generation`
