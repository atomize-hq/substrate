# C2-spec — Shell persistent session client core (protocol correctness; no REPL UX yet)

This slice introduces the host-side persistent session client used by the interactive REPL. It is responsible for protocol correctness and fail-closed behavior. It is not responsible for Reedline routing or rendering.

Authoritative specs (do not diverge):
- `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md` (wire protocol, authoritative)
- `docs/project_management/_archived/world-first-repl-persistent-pty/decision_register.md` (DR-08, DR-18, DR-19)

Depends on:
- `C1` (world-agent exec + command_complete).

Out of scope for C2:
- Interactive REPL routing/directives and lifecycle (`STATE_MACHINE.md`) — C3/C4.
- Non-interactive `-c/--command` and stdin pipe mode — C5.

## Contract (C2 deliverable)

Client protocol correctness (locked):
- Handshake:
  - Client MUST send `start_session` and wait for `ready`.
  - Client MUST validate `ready.protocol_version == 1` and fail closed otherwise.
- Execution:
  - Client MUST send `exec` only when no prior command is in-flight.
  - Client MUST validate `(seq, token_hex)` on the accepted `command_complete`; mismatches are fatal protocol errors (fail closed).
- Framing:
  - `stdout` frames are raw PTY bytes (stdout+stderr combined) and MUST be treated as a byte stream; the client MUST forward decoded bytes unchanged to the consumer.
  - Unknown server frame types are fatal protocol errors (fail closed).
- Exit semantics:
  - If `exit` arrives unexpectedly (no shutdown in progress), the client MUST treat it as fatal.
  - If the client initiated shutdown (`close`), a subsequent `exit` is treated as expected.

No fallbacks (locked):
- When world execution is enabled for the REPL and the persistent session cannot be started, the client MUST surface a high-signal error and MUST NOT fall back to a host execution path (DR-18).

## Acceptance criteria
- Client behavior matches `PROTOCOL.md` fail-closed rules for invalid `ready`, unknown frames, `error` frames, `exit` timing, and `(seq, token_hex)` mismatches.
- Client exposes a minimal interface that allows REPL wiring to:
  - start/stop the session,
  - submit one command at a time,
  - forward `stdout` bytes,
  - and receive a `command_complete` result (exit + physical cwd).

## Validation (C2-test scope)
Add tests that cover, at minimum:
- Version negotiation fail-closed behavior (`ready.protocol_version != 1`).
- No pipelining: attempting to `exec` concurrently is rejected by the client (and/or treated as fatal if the server violates the contract).
- `(seq, token_hex)` mismatch handling: fatal protocol error.
- Unknown server frame: fatal protocol error.
- Expected vs unexpected `exit` handling.
