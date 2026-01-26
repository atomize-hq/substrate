# Plan — Agent Hub Concurrent Execution Output Routing (Minimal)

This plan is anchored by:
- `docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md`

## Execution Phases (high-level)

1) Output classification (contract)
- Treat PTY output as raw bytes (binary-safe).
- Treat agent hub output as structured events (typed, attributable).

2) Interactive REPL rendering rules
- Do not inject structured events into PTY passthrough output.
- Buffer structured events during PTY passthrough and flush after passthrough ends.
- Render out-of-band PTY bytes while idle without corrupting Reedline input.

3) Validation
- Add tests that cover the rendering/routing contract and prevent regressions.
- Add a small manual playbook focused on “PTY passthrough + concurrent agent events”.
