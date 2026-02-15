# OR0 Spec — Output routing contract + buffering during PTY passthrough

## Summary

This slice implements the ADR-0017 output routing contract:
- PTY output is forwarded as raw bytes (binary-safe).
- Structured agent events are rendered via a structured printer path.
- During PTY passthrough, structured agent events are buffered up to a deterministic cap and dropped beyond the cap without backpressuring execution.
- Structured agent events are persisted to canonical trace records.

## Definitions

- **PTY passthrough**: a mode where an interactive TUI is active and the terminal output must be treated as a raw byte stream.
- **Structured agent event**: a JSON-like envelope event (schema: `agent-hub-event-envelope-schema-spec.md`) emitted by agent hub orchestration and/or internal REPL tasks.
- **Suppressed structured line**: a structured event line that is not displayed live during PTY passthrough because the bounded buffer cap was exceeded.

## User-visible behavior

### Baseline (unchanged)
- PTY passthrough preserves TUI correctness (no host text injected into PTY bytes).
- Out-of-band structured events can be printed while the prompt is active.

### New / clarified behavior

1) **Two output classes are enforced**
- PTY bytes are forwarded as bytes.
- Structured agent events are rendered separately and are never mixed into PTY bytes.

2) **Structured events during PTY passthrough**
- While passthrough is active:
  - buffer up to `repl.max_pty_buffered_lines` structured event lines,
  - drop beyond the cap.
- After passthrough ends:
  - print buffered structured event lines in the order received,
  - if any drops occurred, emit exactly one suppression summary warning (not PTY-injected).

3) **Config knob**
- `repl.max_pty_buffered_lines` (default `2048`, bounds `0..=16384`) controls buffering.
- Invalid type/parse is a hard error (exit code `2`).
- Out-of-range values clamp with a structured warning and no exit code change.

4) **Trace persistence**
- Each structured agent event produces exactly one `event_type="agent_event"` record in `trace.jsonl`.
- Each passthrough session with drops produces exactly one `code="pty_structured_event_drops"` warning record in `trace.jsonl`.

## Output invariants (must hold)

- Structured agent events MUST NOT be injected into PTY bytes during passthrough.
- Printing structured events MUST NOT deadlock or block PTY byte forwarding.
- PTY byte forwarding MUST be binary-safe and MUST NOT assume UTF-8.

## Acceptance criteria

1) While a PTY passthrough command is active, demo structured events (`:demo-agent`) do not corrupt the TUI output.
2) If `repl.max_pty_buffered_lines=0`, structured event lines are not printed during passthrough and the suppression summary is emitted if any structured events occurred.
3) If the structured event volume exceeds the cap, the shell emits exactly one suppression summary warning after passthrough ends with `dropped_structured_event_lines > 0`.
4) `trace.jsonl` contains:
   - an `agent_event` record for each structured agent event, and
   - a `warning` record with `code="pty_structured_event_drops"` when suppression occurs.
5) Invalid config type/parse for `repl.max_pty_buffered_lines` fails at the config boundary with exit code `2`.

