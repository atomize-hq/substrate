# Plan — World-First REPL With Persistent World PTY

This plan is anchored by:
- `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md`
- `docs/project_management/next/world-first-repl-persistent-pty/decision_register.md`

## Execution Phases (high-level)

1) REPL command routing
- Add `:host` prefix routing and ensure unprefixed commands are world-first.

2) Persistent world session
- Introduce a long-lived world PTY session abstraction for Linux/macOS.
- Define and implement a deterministic command-boundary protocol to extract per-command exit status without terminating the session.

3) Trace and diagnostics
- Ensure every REPL-entered command produces a trace span with correct `execution_origin`, exit code, and policy snapshot metadata.
- Provide high-signal failure modes when world is required but unavailable.

4) Validation
- Unit tests for routing and state invariants.
- Integration harness for multi-command REPL sessions (cwd/env persistence).
- Manual playbook per ADR.

