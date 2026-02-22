# agent-hub-concurrent-execution-output-routing — telemetry spec

Owner standard:

- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

- This spec is authoritative for new/changed telemetry (trace records) required by ADR-0017.

## Stability guarantees (explicit)

- Field stability: additive-only (new non-required fields permitted; no renames/removals).
- Backwards compatibility for consumers:
  - Consumers MUST tolerate unknown fields and unknown record types in `trace.jsonl`.
  - Existing span records MUST remain unchanged in meaning.

## Trace/log schema changes (authoritative)

### Structured agent event records

Requirement:

- Each structured agent event MUST be persisted as its own JSON record in canonical trace.

Canonical record:

- `event_type: "agent_event"`
- Required fields:
  - `event_type` (string; MUST be `"agent_event"`)
  - `ts` (string; RFC3339 UTC timestamp; MUST equal the envelope `ts`)
  - `session_id` (string; shell trace session id)
  - `component` (string; MUST be `"agent-hub"`)
  - Envelope required fields (top-level; schema: `agent-hub-event-envelope-schema-spec.md`):
    - `kind`
    - `agent_id`
    - `orchestration_session_id`
    - `run_id`
    - `data`

Optional envelope fields (top-level; emitted when known and applicable):

- `backend_id`, `thread_id`, `role`, `world_id`, `cmd_id`, `span_id`, `channel`, `project`

Redaction rule:

- `channel` MUST be safe-to-print (no secrets) per schema spec.

Consumer impact:

- Enables deterministic multi-agent joins and downstream routing without terminal scraping.

### PTY passthrough suppression summary records

When structured event lines are dropped during PTY passthrough, the shell MUST emit exactly one summary record.

Canonical record:

- `event_type: "warning"`
- Required fields:
  - `event_type` (string; must be `"warning"`)
  - `ts` (RFC3339 UTC string)
  - `session_id` (string)
  - `component` (string; MUST be `"shell"`)
  - `code` (string; MUST be `"pty_structured_event_drops"`)
  - `dropped_structured_event_lines` (int; number dropped)
  - `max_pty_buffered_lines` (int; effective configured cap)
- Optional fields:
  - `world_id` (string; when the passthrough is tied to a world session)
  - `cmd_id` (string; when the passthrough is tied to a command)
  - `span_id` (string; when the passthrough is tied to a traced span)

Redaction rule:

- No user-provided payload is emitted in this warning record.

Consumer impact:

- Downstream tools can report suppressed activity deterministically without heuristics.

### Clamp warning record (out-of-range `repl.max_pty_buffered_lines`)

If an out-of-range value is provided and clamped, the shell MUST emit a structured warning record (not PTY-injected).

Canonical record:

- `event_type: "warning"`
- Required fields:
  - `event_type` (string; must be `"warning"`)
  - `ts` (RFC3339 UTC string)
  - `session_id` (string)
  - `component` (string; MUST be `"shell"`)
  - `code` (string; MUST be `"config_value_clamped"`)
  - `key` (string; MUST be `"repl.max_pty_buffered_lines"`)
  - `provided` (int)
  - `effective` (int)
  - `min` (int; `0`)
  - `max` (int; `16384`)

## Metrics (if any)

- None in v1 (trace records are sufficient for correctness and auditing).

## Acceptance criteria (testable)

- Every structured agent event produces exactly one `event_type="agent_event"` trace record containing:
  - `session_id`
  - `component="agent-hub"`
  - the envelope fields at the record top level (no nested envelope object).
- Each PTY passthrough session that drops structured lines produces exactly one `code="pty_structured_event_drops"` warning record.
- A clamped config value produces exactly one `code="config_value_clamped"` warning record and does not change exit code.
