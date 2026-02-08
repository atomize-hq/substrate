# C1-spec — World-agent per-submission exec + explicit completion (server-side)

This slice implements persistent-session per-submission execution on world-agent:
`exec` → streamed `stdout` bytes → `command_complete`, with the locked DR-22 and DR-23 invariants enforced.

Authoritative specs (do not diverge):
- `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md`
- `docs/project_management/_archived/world-first-repl-persistent-pty/decision_register.md` (DR-07, DR-21, DR-22, DR-23)
- `docs/project_management/_archived/world-first-repl-persistent-pty/driver_loop_design.md`
- `docs/project_management/_archived/world-first-repl-persistent-pty/drain_design.md`

Depends on:
- `C0` (persistent-session bootstrap + ready).

Out of scope for C1:
- Host-side session client and interactive REPL behavior — C2/C3/C4.
- Non-interactive `-c/--command` behavior — C5.

## Contract (C1 deliverable)

Persistent-session execution:
- World-agent MUST accept `exec` only when idle (no pipelining); `exec` while a command is in-flight is a fatal protocol error.
- Completion is explicit:
  - World-agent MUST NOT infer completion from stdout bytes.
  - For each accepted `exec`, world-agent MUST emit exactly one `command_complete` with the corresponding `(seq, token_hex)`.

Evaluator execution model (v1, locked):
- Evaluator shell MUST be `/bin/bash --noprofile --norc`.
- v1 MUST use per-submission evaluator shells attached to the Session PTY (no single long-lived interactive bash interpreter).

Persistence scope (v1, locked):
- The trusted driver component MUST persist and re-apply exactly the ADR-guaranteed state across submissions:
  - physical cwd (`pwd -P` / `getcwd()` semantics),
  - and exported env mutations (`export`/`unset`).
- Other shell-local state is not guaranteed and MUST NOT be relied on by the protocol.

Command/control separation (DR-21, locked):
- Program text MUST NOT be sent over PTY stdin.
- Program text MUST be delivered to the trusted driver via a private command-control channel that is not inherited by untrusted user programs.

Control-plane handle privacy (DR-22, locked):
- Untrusted evaluator processes MUST NOT be able to access session infrastructure/control-plane handles/endpoints (including the `/v1/stream` WebSocket FD).
- Attempts to access inherited non-stdio FDs (for example via `/proc/self/fd` scanning where available and numeric redirections like `>&$FD` / `<&$FD`) MUST NOT allow:
  - spoofing `command_complete`,
  - reading tokens or future submissions,
  - or protocol desynchronization.

Output ordering / drain barrier (DR-23, locked):
- `command_complete` MUST NOT be emitted until all foreground Session PTY bytes for that command have been forwarded as `stdout` frames.
- v1 MUST implement a watermark drain barrier using Linux `ioctl(FIONREAD)` on the PTY master and MUST fail closed if unsupported (no timing heuristics; no quiescence/would-block drains).

Stdin/signal boundaries (PROTOCOL v1, locked):
- `stdin` frames MUST be dropped unless a command is running with `stdin_mode=passthrough`.
- `stdin` bytes arriving after command completion MUST be dropped (never misdelivered to the next command).
- `signal` frames MUST target the Session PTY foreground process group (not a PID; not session infrastructure).

## Acceptance criteria
- World-agent passes the per-exec protocol compliance tests defined in `docs/project_management/_archived/world-first-repl-persistent-pty/requirements_traceability.md`.
- World-agent fails closed (fatal `error`) on:
  - invalid `exec` payloads (base64/UTF-8/NUL policy per `PROTOCOL.md`),
  - `exec` while busy,
  - inability to satisfy DR-22 invariants at runtime,
  - inability to satisfy DR-23 ordering barrier for v1.

## Validation (C1-test scope)
Add world-agent tests that cover, at minimum:
- `exec` validation: base64 decode, UTF-8 validation, NUL rejection.
- No pipelining: `exec` while busy is fatal.
- Output ordering: watermark drain barrier prevents late foreground bytes after `command_complete`.
- Stdin gating: `stdin` is accepted only in passthrough mode and is dropped after completion.
- Signal targeting: `signal` targets the foreground process group and does not kill the session driver loop.
- Persistence: physical cwd and exported env mutations persist across successive `exec` submissions.
- DR-22 adversarial behavior: evaluator cannot access control-plane/session infrastructure handles and cannot spoof completion.

## Cross-platform notes
- Persistent-session behavior is implemented by world-agent (Linux). macOS hosts run through Lima (Linux guest); the same world-agent behavior is required.
