# Plan — World-First REPL With Persistent World PTY

This plan is anchored by:
- `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md`
- `docs/project_management/next/world-first-repl-persistent-pty/decision_register.md`

## Execution Phases (high-level)

1) REPL command routing
- Add `:host` prefix routing and ensure unprefixed commands are world-first.
- Enforce `:host` gating: when disabled, `:host ...` must error and must not execute on host or world.

2) Persistent world session
- Introduce a long-lived world-agent `/v1/stream` PTY session abstraction (initially Linux/macOS).
- Define and implement a deterministic command-boundary protocol to extract per-command exit status without terminating the session.
- Implement the marker framing + streaming parser (split-frame handling, prefix-gated candidate detection, seq+token validation).
- Implement per-command I/O modes:
  - Line mode: stdin redirected to `/dev/null` to avoid hangs for stdin-consuming commands.
  - PTY passthrough mode: raw terminal + stdin forwarding for TUIs/interactive programs (auto-PTY).
- Implement the per-line submission framing (brace-frame with marker invocation on the closing line) for both modes, so the shell parses the marker invocation before starting the user command.
- Implement marker parsing prefix-gate to avoid false protocol errors on binary output.
- Add per-command token validation to prevent early marker spoofing and desync.
- Preserve in-world cwd across snapshot-driven session restarts when possible.
- Implement restart-on-snapshot-hash-change (and workspace root changes), with explicit operator-visible messaging when a restart occurs and when cwd continuity cannot be preserved.
- Define `:pty` semantics: force PTY passthrough mode (in-world when world enabled; host PTY when `--no-world`).

3) Trace and diagnostics
- Ensure every REPL-entered command produces a trace span with correct `execution_origin`, exit code, and policy snapshot metadata.
- Provide high-signal failure modes when world is required but unavailable.
- Document v1 correlation limits for persistent sessions and ensure the design does not preclude `docs/BACKLOG.md` P0 “in-world process execution tracing parity”.

4) Validation
- Unit tests for routing and state invariants.
- Integration harness for multi-command REPL sessions (cwd/env persistence).
- Manual playbook per ADR.
- Add targeted tests for protocol robustness and security invariants:
  - Marker split across multiple `stdout` frames.
  - Binary output containing marker-like bytes: must not false-fatal unless prefix+validation matches.
  - Stdin-consuming commands in line mode (`cat`, `read`): must not hang the session (stdin redirected to `/dev/null`).
  - Auto-PTY commands (vim/python REPL): must receive stdin and function interactively in PTY passthrough mode.
  - `:host` disabled: must error and must not execute.
  - Snapshot drift restart: policy file/workspace root change triggers restart (cwd continuity preserved when possible).
