# ADR-0021 — Substrate Workflow Engine

## Status

- Status: Draft
- Queue state: Queued
- Original date (UTC): 2026-02-03
- Curated into `docs/adr/draft/`: 2026-05-26
- Owner(s): Substrate maintainers

## Curated From

- Planning ADR:
  - `docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md`

The project-management ADR remains the planning-rich source retained for compatibility while
`docs/project_management/**` is retired.

## Queued Direction

Substrate needs a first-class workflow runtime that can execute DAG-shaped work under the existing
policy, trace, and isolation model rather than pushing multi-step orchestration outside the
product.

The queued direction that still matters is:

- a DAG workflow runtime with explicit node dependencies
- stable workflow-run and node-run tracing
- execution that continues to route tool/script work through existing policy and world boundaries
- extensibility through node executors rather than a one-off scheduler

## Why Queued

This is still active architectural input, but it is not landed and should not yet be treated as a
stable contract.

When implementation is ready, it should be restated against:

- `docs/adr/implemented/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- `docs/adr/implemented/ADR-0028-in-world-process-execution-tracing-parity.md`
- `docs/adr/draft/ADR-0029-host-event-bus-and-router-daemon.md`
- `docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md`

## Draft Note

Keep the project-management ADR for original planning detail, but treat this curated draft as the
queued workflow-runtime placeholder.
