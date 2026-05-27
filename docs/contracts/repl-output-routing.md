# REPL Output Routing Contract

This document is the durable contract reference for concurrent REPL output routing.

Related references:
- `docs/contracts/agent-event-envelope.md`
- `docs/TRACE.md`
- `docs/adr/implemented/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`

## Contract

Substrate REPL output has two distinct classes:

- PTY bytes
- structured agent events

The stable rules are:

- PTY bytes are forwarded as raw bytes and remain binary-safe.
- Structured agent events render through a structured printer path.
- Structured agent events must never be injected into PTY byte streams.
- Structured event rendering must not stall execution; bounded buffering plus drop is allowed
  during PTY passthrough.
- Every structured agent event must still persist as a canonical `agent_event` trace row.

## Identity Boundary

- `backend_id` is an adapter/backend identifier only, in `<kind>:<name>` form.
- `backend_id` must not be overloaded with provider, auth authority, router, client, or protocol
  meaning.
- Tuple semantics for `client`, `router`, `provider`, `auth_authority`, and `protocol` remain
  owned by the identity ADR chain rather than this output-routing contract.

## Idle REPL Behavior

When the line editor is active:

- out-of-band PTY bytes may render as raw bytes
- structured agent events may render through the structured printer
- neither output path may corrupt the prompt or input buffer

Current additive alert scope includes:

- `kind="alert"` with `data.code="world_restarted"`
- `kind="alert"` with `data.code="world_restart_required"`

## PTY Passthrough Behavior

During PTY passthrough:

- PTY bytes forward immediately as bytes.
- Structured agent events must not print live into the terminal stream.
- Structured event lines are buffered for deferred rendering.
- After passthrough ends, buffered lines print in order before the prompt returns.

## Suppression Summary

If buffered structured event lines overflow the configured cap:

- additional structured event lines are dropped
- exactly one structured warning record is emitted with:
  - `event_type="warning"`
  - `component="shell"`
  - `code="pty_structured_event_drops"`
- exactly one human-readable warning line is emitted through the normal warning channel

The suppression warning must not be injected into PTY bytes.

## Config Surface

This contract introduces no CLI flags and no environment overrides.

Effective precedence for `repl.max_pty_buffered_lines` is:

1. `<workspace_root>/.substrate/workspace.yaml`
2. `$SUBSTRATE_HOME/config.yaml`
3. built-in default

Owned key:

- `repl.max_pty_buffered_lines`
  - meaning: maximum number of structured event lines buffered during PTY passthrough
  - default: `2048`
  - bounds: `0..16384`

Invalid handling:

- invalid type/parse is a config-boundary error with exit `2`
- out-of-range values clamp to bounds and emit one structured warning record with:
  - `event_type="warning"`
  - `component="shell"`
  - `code="config_value_clamped"`

Warnings do not change command exit status.

## Platform Guarantee

- Linux: full support required
- macOS: full support required
- Windows: the same non-injection and structured-event separation rules apply anywhere PTY
  passthrough exists
