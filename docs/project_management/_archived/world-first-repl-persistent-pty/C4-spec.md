# C4-spec — Interactive REPL byte-safe rendering + out-of-band stdout + structured output buffering

This slice implements the interactive REPL rendering invariants that require a byte-capable output path while Reedline is active, out-of-band stdout handling, and buffering of concurrent Substrate-managed structured output during PTY passthrough.

Authoritative specs (do not diverge):
- `docs/project_management/_archived/world-first-repl-persistent-pty/STATE_MACHINE.md` (host behavior, authoritative)
- `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md` (stdout frames are raw bytes, authoritative)
- `docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md` (structured output routing, authoritative)

Depends on:
- `C3` (interactive REPL lifecycle/routing wired to persistent session).

Out of scope for C4:
- Non-interactive `-c/--command` and stdin pipe mode — C5.

## Contract (C4 deliverable)

Byte-safe PTY rendering (locked):
- Session PTY `stdout` frames are raw bytes (stdout+stderr combined) and may be non-UTF8.
- While Reedline is active (waiting for user input), Substrate MUST render PTY bytes via a byte-capable output path that preserves the input buffer; PTY bytes MUST NOT be routed through string-only printers.

Out-of-band stdout while idle (locked):
- `stdout` frames MAY arrive while no `exec` is in-flight.
- The host MUST still forward/render these bytes (no suppression) and MUST NOT attribute them to a specific command in v1.

Structured host output separation (locked):
- Substrate-managed structured output MUST NOT be injected into PTY bytes.
- During PTY passthrough, structured host output SHOULD be buffered and rendered only after the foreground command completes.

## Acceptance criteria
- Interactive REPL renders raw PTY bytes without corrupting the Reedline input buffer.
- Out-of-band `stdout` bytes are rendered while idle and remain unattributed in v1.
- Concurrent structured host output does not corrupt TUIs during PTY passthrough (buffered then flushed after completion).

## Validation (C4-test scope)
Add tests that cover, at minimum:
- Out-of-band `stdout` rendering while idle (no input buffer corruption).
- PTY passthrough: raw stdin byte forwarding (including `0x03`) plus buffering behavior for structured host output.
- Regression coverage for “no structured output injected into PTY bytes”.

