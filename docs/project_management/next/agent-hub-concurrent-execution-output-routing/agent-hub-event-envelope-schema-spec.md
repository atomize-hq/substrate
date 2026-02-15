# agent-hub-concurrent-execution-output-routing — agent hub event envelope schema spec

Owner standard:
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope
- This spec is authoritative for the structured agent event envelope used for:
  - host-side structured printing in the REPL, and
  - durable persistence of agent events to canonical trace.

## Format
- Format: JSON
- Canonical file name(s): N/A (event payloads are embedded in trace records)
- Canonical location(s):
  - `~/.substrate/trace.jsonl` (or `SHIM_TRACE_LOG`) as `event_type="agent_event"` records (see `telemetry-spec.md`)

## Compatibility policy (explicit)
- Forward compatibility: additive-only changes (new optional fields, new `kind` values, additive `data` fields).
- Backward compatibility: existing required fields MUST NOT be removed or renamed.
- Unknown fields handling: consumers MUST ignore unknown fields.
- Deprecation policy: deprecate by adding a new field and marking the old field as deprecated; do not reuse semantics.

## Schema (authoritative)

### Top-level envelope

All structured agent events are JSON objects with:

- `ts`
  - Type: string (RFC3339 UTC timestamp)
  - Required: yes
- `kind`
  - Type: string enum
  - Required: yes
  - Allowed values (v1):
    - `registered`
    - `status`
    - `task_start`
    - `task_progress`
    - `task_end`
    - `pty_data`
    - `alert`
- `data`
  - Type: object
  - Required: yes
  - Schema: depends on `kind` (see below)

Attribution + correlation fields:
- `agent_id`
  - Type: string
  - Required: yes
  - Meaning: actor/principal identifier (for agent-driven events, the agent inventory id).
- `backend_id`
  - Type: string
  - Required: no
  - Meaning: `<kind>:<name>` stable backend identifier (when known).
- `orchestration_session_id`
  - Type: string
  - Required: yes
  - Meaning: stable join key for multi-agent orchestration context.
- `run_id`
  - Type: string
  - Required: yes
  - Meaning: stable join key for a particular run within an orchestration session.
- `thread_id`
  - Type: string
  - Required: no
- `role`
  - Type: string
  - Required: no
  - Reserved values (v1): `orchestrator|member`
- `world_id`
  - Type: string
  - Required: no (but REQUIRED when the emitting backend executes inside a world boundary)
- `cmd_id`
  - Type: string
  - Required: no
- `span_id`
  - Type: string
  - Required: no

Routing hint:
- `channel`
  - Type: string
  - Required: no
  - Constraints:
    - Producer-declared only (MUST NOT be arbitrary user-provided freeform).
    - Capped (recommendation: <= 64 bytes when UTF-8 encoded).
    - MUST NOT contain secrets. Producers MUST drop unsafe values; emit a structured warning MAY occur.
    - MUST NOT affect policy gating decisions.

Legacy field (back-compat; not required by ADR-0017):
- `project`
  - Type: string
  - Required: no
  - Meaning: legacy grouping label; producers MAY omit.

### Per-kind `data` schema (v1)

#### `kind="registered" | "status" | "task_start" | "task_progress" | "task_end"`
- `data.message`
  - Type: string
  - Required: no
  - If present, it is safe to print and persist.

#### `kind="pty_data"`
- `data.stream`
  - Type: string enum
  - Required: yes
  - Allowed: `stdout|stderr`
- `data.chunk`
  - Type: string
  - Required: yes
  - Note: this is a structured representation for non-PTY “chunked” producer output; it MUST NOT be treated as a substitute for raw PTY bytes.

#### `kind="alert"`
- `data.code`
  - Type: string
  - Required: yes
  - Known codes (v1; schema-owned by ADR-0017):
    - `world_restarted`
    - `world_restart_required`
- `data.message`
  - Type: string
  - Required: yes (human-readable; safe to print/persist)

Additional fields for `data.code="world_restarted"`:
- `data.reason` (string; required; one of DR-0008 taxonomy strings)
- `data.on_drift` (string; required; `auto_restart`)
- `data.previous_world_id` (string; required)
- `data.new_world_id` (string; required)
- `data.previous_world_generation` (int; required)
- `data.new_world_generation` (int; required)

Additional fields for `data.code="world_restart_required"`:
- Schema is authoritative in `docs/project_management/next/agent_hub_core/decision_register.md` (DR-0009).

### Canonicalization rules (explicit)
- Producers MUST emit all required top-level fields.
- Producers MUST keep attribution/correlation fields at the top level (no nested attribution object).
- Consumers MUST treat the envelope as unordered JSON; no stable field ordering is required.

## Examples (authoritative)

### Minimal valid (status)
```json
{
  "ts": "2026-02-15T00:00:00Z",
  "kind": "status",
  "agent_id": "demo",
  "orchestration_session_id": "orch_123",
  "run_id": "run_123",
  "data": { "message": "starting" }
}
```

### PTY-style chunk (structured)
```json
{
  "ts": "2026-02-15T00:00:00Z",
  "kind": "pty_data",
  "agent_id": "burst-00",
  "orchestration_session_id": "orch_123",
  "run_id": "run_123",
  "data": { "stream": "stdout", "chunk": "hello\\n" }
}
```

### Alert: world restarted
```json
{
  "ts": "2026-02-15T00:00:00Z",
  "kind": "alert",
  "agent_id": "human",
  "role": "orchestrator",
  "orchestration_session_id": "orch_123",
  "run_id": "run_123",
  "world_id": "world_aaa",
  "data": {
    "code": "world_restarted",
    "reason": "policy_drift",
    "on_drift": "auto_restart",
    "previous_world_id": "world_aaa",
    "new_world_id": "world_bbb",
    "previous_world_generation": 0,
    "new_world_generation": 1,
    "message": "world restarted due to policy drift"
  }
}
```

### Invalid (missing required correlation fields)
```json
{
  "ts": "2026-02-15T00:00:00Z",
  "kind": "status",
  "agent_id": "demo",
  "data": { "message": "missing orchestration keys" }
}
```

## Error model (explicit)
- Producers MUST NOT emit envelopes missing required fields.
- Consumers MUST treat malformed payloads as non-fatal and MUST NOT crash; they MAY emit a structured warning.

## Security / redaction (explicit)
- Producers MUST NOT place secrets into:
  - `channel`, or
  - any other top-level attribution/correlation field.
- Any field that could contain secrets MUST live under `data` with an explicit redaction policy in a future schema revision; v1 disallows secrets entirely.

## Acceptance criteria (testable)
- Every emitted structured agent event contains the required top-level fields (`ts`, `kind`, `agent_id`, `orchestration_session_id`, `run_id`, `data`).
- `kind` values are in the allowed taxonomy (unknown kinds are rejected by producers and ignored by consumers).
- `channel` is producer-declared, bounded, and safe-to-print (unsafe values are dropped).

