# OR0 spec — Structured agent event envelope + canonical trace persistence foundation

This slice implements the “structured events are durable and joinable” foundation required by ADR-0017.

## Scope (in-scope; authoritative)

Implement and validate:
1) The structured agent event envelope (schema and invariants)
   - Authoritative schema: `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/agent-hub-event-envelope-schema-spec.md`
2) Canonical trace persistence for structured agent events
   - Authoritative record shapes: `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/telemetry-spec.md`

This slice is the minimal contract seam needed before any PTY passthrough buffering/routing changes.

## Out of scope (explicit)

- PTY passthrough buffering, drop behavior, and suppression summary emission (implemented in OR1).
- `repl.max_pty_buffered_lines` config key plumbing (implemented in OR1).
- Any changes to world-service PTY transport (`/v1/stream`).

## Behavior (authoritative)

### Envelope invariants

Structured agent events use the envelope fields and rules in:
- `agent-hub-event-envelope-schema-spec.md`

Non-negotiable:
- Producers emit all required envelope fields.
- Envelope correlation fields are top-level keys (not nested).
- `channel` is producer-declared and secrets-safe; unsafe values are dropped deterministically.
- `backend_id` remains adapter-only and MUST NOT be repurposed as provider/auth/protocol meaning.
- Optional tuple-compatible metadata (`client`, `router`, `provider`, `auth_authority`, `protocol`) remains top-level when present, but its semantics are delegated to ADR-0042, ADR-0044, and ADR-0045.

### Canonical trace persistence (agent events)

For every structured agent event emitted by the shell’s structured event path (including `:demo-agent`), append exactly one canonical trace record:
- `event_type="agent_event"`
- `component="agent-hub"`
- Record fields include the envelope fields at the record top level (no nested envelope object).

Canonical record requirements are owned by:
- `telemetry-spec.md`

### ID generation for shell-internal producers (`:demo-agent`)

For shell-internal structured event producers:
- `orchestration_session_id` is generated once per shell session (UUIDv7).
- `run_id` is generated once per `:demo-agent` invocation (UUIDv7).
- `agent_id="demo-agent"` and `role="member"` are emitted.

Clarification (Phase 8; non-negotiable):
- This “once per shell session” scoping is a `:demo-agent` implementation convenience only and MUST NOT be treated as a global invariant.
- Real Agent Hub orchestration MAY mint multiple `orchestration_session_id` values within a single `session_id`, and consumers MUST NOT assume or derive a 1:1 mapping (see ADR-0028 correlation vocabulary).

## Acceptance criteria (testable)

1) Schema correctness:
- A test suite exists and is green:
  - `cargo test -p substrate-common --test agent_hub_event_envelope_schema -- --nocapture` → exit `0`

2) Trace persistence:
- A test suite exists and is green:
  - `cargo test -p shell --test agent_hub_trace_persistence -- --nocapture` → exit `0`
- The tests assert:
  - one `event_type="agent_event"` record is appended per emitted event, and
  - required envelope fields exist at the trace record top level.
  - optional tuple-compatible metadata remains top-level when present, and `backend_id` is preserved as a distinct adapter id.
