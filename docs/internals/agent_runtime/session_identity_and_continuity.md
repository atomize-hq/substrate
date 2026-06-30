# Agent Runtime Session Identity and Continuity

This document is the canonical internal map for the identifiers that connect:

- durable host orchestration sessions
- runtime participant lineage
- shared-world binding proof
- backend-native resume handles
- canonical trace correlation

Use this page when deciding whether an id is:

- a public selector
- a live-runtime/session-store field
- a world-binding proof field
- a trace correlation field
- or a compatibility alias that should not spread further

## Rules of Thumb

- `orchestration_session_id` is the durable public session selector.
- `backend_id` is the exact backend selector and allowlist token.
- `participant_id` and `resumed_from_participant_id` are runtime lineage fields, not public control selectors.
- `world_id` and `world_generation` are authoritative world-binding proof fields, not public control selectors.
- `uaa_session_id` is a backend-native runtime handle. It is internal only.
- `active_session_handle_id` is compatibility storage for the active `participant_id`; public docs should not promote it.
- `shell_trace_session_id` and `latest_run_id` are correlation aids, not ownership or selector truth.

## Taxonomy

### `orchestration_session_id`

- Meaning: durable parent orchestration-session identity
- Scope: host-orchestrator session root
- Public: yes
- Authoritative for: public `substrate agent turn|reattach|fork|stop --session ...`
- Stored in:
  - `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/session.json`
  - `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/participants/<participant_id>.json`
- Also appears in: trace and agent-event correlation
- Not equivalent to:
  - shell trace `session_id`
  - `participant_id`
  - `uaa_session_id`

### `backend_id`

- Meaning: exact backend/adaptor identity in `<kind>:<agent_id>` form
- Scope: runtime selection, policy allowlisting, public follow-up targeting
- Public: yes
- Authoritative for: exact backend routing and policy gates
- Appears in:
  - public `substrate agent start|turn --backend ...`
  - session and participant records
  - trace/agent events when a concrete backend is involved
- Not equivalent to:
  - `provider`
  - `auth_authority`
  - `protocol`
  - semantic role meaning

### `participant_id`

- Meaning: canonical runtime lineage id for one participant row
- Scope: participant record, runtime lineage, status/debug surfaces, trace lineage
- Public: no, except as diagnostic output when surfaced
- Authoritative for: identifying one concrete participant row
- Stored in:
  - `participants/<participant_id>.json`
  - agent-event lineage fields
- Used for:
  - successor linkage
  - attached/live participant selection
  - member follow-up/debug correlation
- Not valid as a public session selector

### `resumed_from_participant_id`

- Meaning: lineage pointer from a successor participant back to the prior participant it resumed from
- Scope: runtime lineage and trace lineage
- Public: no, except as additive diagnostic output
- Authoritative for: successor/continuity lineage only
- Stored in:
  - participant records
  - `agent_event` additive lineage fields
- Notes:
  - this is the canonical lineage name
  - `resumed_from_session_handle_id` is compatibility-only aliasing on read

### `world_id` and `world_generation`

- Meaning:
  - `world_id`: identity of the current authoritative shared world
  - `world_generation`: replacement/version counter for that world slot
- Scope: shared-world binding proof
- Public: diagnostic/status only
- Authoritative for:
  - exact world-binding proof
  - member-dispatch validation
  - replacement/invalidation sequencing
- Stored in:
  - world backend metadata
  - session-root parent record when projected into shell-owned live state
  - participant/world-scoped runtime records where appropriate
- Join/use rule:
  - treat `world_id + world_generation` as one proof pair
- Not valid as public control selectors

### `uaa_session_id`

- Meaning: backend-native upstream runtime/session handle
- Scope: attach continuity and backend resume
- Public: no
- Authoritative for: resume against the backend-native runtime when continuity is supported
- Stored in:
  - participant internal state
  - host-attach continuity contract fields when surfaced internally
- Notes:
  - this is not the durable Substrate session id
  - it must not replace `orchestration_session_id` in operator or CLI language

### `active_session_handle_id`

- Meaning: compatibility storage alias for the active orchestrator `participant_id` on the parent session row
- Scope: session-root parent record
- Public: no
- Authoritative for: compatibility reads of “which participant is currently active”
- Notes:
  - the value is a `participant_id`
  - the canonical concept is “active participant”, not “session handle”
  - new docs should prefer `active_participant_id` language when describing the meaning

### `shell_trace_session_id`

- Meaning: shell trace session correlation id attached to the parent orchestration record
- Scope: shell/trace correlation
- Public: no
- Authoritative for: correlation back to the shell trace session only
- Notes:
  - this is not the orchestration-session id
  - it must not be used as ownership truth or as a public selector

### `latest_run_id`

- Meaning: stored pointer to the most recent run-scoped unit of work for a session or participant
- Scope: runtime session/participant state, status joins, and trace correlation
- Public: no
- Authoritative for: latest run correlation only
- Appears in:
  - participant/session internal state
  - status/debug projections that need to point at the latest emitted `run_id`
- Notes:
  - it is not a durable ownership id
  - it must not replace `orchestration_session_id` or `participant_id`
  - it is not an emitted `agent_event` top-level field; emitted rows carry `run_id`
  - it is join metadata that points at the currently/latest relevant emitted `run_id`

## Authority Boundaries

- Public control ownership:
  - `orchestration_session_id`
  - exact `backend_id`
- Runtime lineage ownership:
  - `participant_id`
  - `resumed_from_participant_id`
- Shared-world binding ownership:
  - `world_id`
  - `world_generation`
- Backend-native continuity ownership:
  - `uaa_session_id`
- Trace/session correlation ownership:
  - shell trace `session_id`
  - `shell_trace_session_id`
  - `run_id`
  - `latest_run_id`

## Compatibility Aliases to Contain

- `session_handle_id` is compatibility-only storage/reading around `participant_id`.
- `active_session_handle_id` is compatibility-only storage for the active `participant_id`.
- `resumed_from_session_handle_id` is compatibility-only aliasing on read for `resumed_from_participant_id`.

These names should not be expanded into new public contracts.

## Related Stable Docs

- `docs/USAGE.md` owns the public selector and command-surface contract.
- `docs/WORLD.md` owns shared-world proof semantics.
- `docs/TRACE.md` and `docs/contracts/agent-event-envelope.md` own emitted trace and event correlation fields.
- `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md` owns `backend_id` semantic boundaries.
- `docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md` owns the durable-session decision.
