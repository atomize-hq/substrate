# Contract — Agent Hub Concurrent Execution Output Routing

This document is the operator-facing contract summary for:
- `docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`

## Non-negotiable invariants

- **Two output classes.**
  - **PTY bytes** are raw bytes (binary-safe) and MUST be forwarded as bytes.
  - **Structured agent events** are typed JSON-like envelopes and MUST be rendered via a structured printer path.
- **No PTY injection.**
  - Structured agent events MUST NOT be injected into PTY byte streams during PTY passthrough (TUIs).
- **No execution backpressure from printing.**
  - Rendering structured events MUST NOT stall/slow execution (bounded buffering + drop is permitted in passthrough).
- **Durable observability.**
  - Every structured agent event MUST be persisted as its own canonical trace record (details: `telemetry-spec.md`).

## Output routing contract

### Output classes

1) **PTY byte stream**
- Source: world-agent streaming (`/v1/stream`) or host PTY passthrough.
- Handling: forwarded as raw bytes to the terminal (no UTF-8 assumptions).

2) **Structured agent events**
- Source: agent hub orchestration and/or internal REPL tasks.
- Handling: printed via the structured renderer (never mixed into PTY bytes).
- Envelope schema (authoritative): `agent-hub-event-envelope-schema-spec.md`.

### REPL mode: Idle (prompt active)

When the line editor is active, the shell MAY receive:
- out-of-band PTY bytes, and/or
- structured agent events.

Rules:
- Out-of-band PTY bytes MUST render as raw bytes and MUST NOT corrupt the prompt/input buffer.
- Structured agent events MUST render without corrupting the prompt/input buffer.

### REPL mode: PTY passthrough (TUI active)

During PTY passthrough:
- PTY bytes MUST be forwarded immediately as bytes.
- Structured agent events MUST NOT be printed live into the terminal stream.

Structured event handling during passthrough:
- The shell buffers up to `repl.max_pty_buffered_lines` structured event lines for deferred rendering.
- Once the cap is reached, additional structured event lines are dropped.
- After passthrough ends and before returning to the prompt:
  - buffered structured event lines are printed (order preserved), and
  - if any drops occurred, the shell emits exactly one suppression summary warning (see below).

### Overflow signaling (suppression summary)

If structured event lines were dropped during PTY passthrough, the shell MUST emit:
- One structured warning record (machine-readable; persisted to trace) with:
  - `event_type="warning"`
  - `component="shell"`
  - `code="pty_structured_event_drops"`
- One human-readable warning line via the normal warning channel.

The warning MUST NOT be injected into PTY bytes.

The warning payload schema and trace record shape are authoritative in `telemetry-spec.md`.

## Config surface

### Files and precedence (existing layering model)

This feature introduces no CLI flags or environment overrides for `repl.max_pty_buffered_lines`.
Effective value precedence is deterministic:
1) Workspace config: `<workspace_root>/.substrate/workspace.yaml` (highest precedence)
2) Global config: `$SUBSTRATE_HOME/config.yaml`
3) Built-in default (lowest precedence)

### Key: `repl.max_pty_buffered_lines`

Meaning:
- Maximum number of **structured event lines** buffered during PTY passthrough before dropping begins.
- This cap does not apply to PTY bytes.

Defaults and bounds:
- Default: `2048`
- Bounds: `min=0`, `max=16384`

Invalid handling (deterministic):
- Invalid type/parse: hard error at the config boundary (exit code `2`).
- Out-of-range integer: clamp to bounds and emit one structured warning record (no PTY injection; warning persisted to trace) with:
  - `event_type="warning"`
  - `component="shell"`
  - `code="config_value_clamped"`

## Exit codes

Taxonomy:
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

This feature adds no new exit codes beyond:
- `2` for invalid config type/parse at the config boundary for `repl.max_pty_buffered_lines`.

Warnings (including suppression summaries and clamp notices) MUST NOT change the command exit code.

## Platform guarantees

- Linux: full support required.
- macOS: full support required.
- Windows: the same non-injection and routing rules apply anywhere PTY passthrough exists; platforms without PTY passthrough still MUST preserve the structured-event path and prompt-safety invariants.
