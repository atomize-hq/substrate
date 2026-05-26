# ADR-0029 — Host Event Bus and Router Daemon

## Status

- Status: Draft
- Queue state: Queued
- Original date (UTC): 2026-02-05
- Curated into `docs/adr/draft/`: 2026-05-26
- Owner(s): Spenser McConnell (Substrate); Shell maintainers

## Curated From

- Planning ADR:
  - `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

The project-management ADR remains the planning-rich source retained for compatibility while
`docs/project_management/**` is retired.

## Queued Direction

Substrate may need a host-scoped router service that consumes trace and event signals to drive
workflow triggers, requests, and cross-workspace routing.

The queued direction that still matters is:

- trace-driven or event-driven routing as a host-scoped service
- compatibility with durable host orchestration sessions
- reuse of existing trace and output attribution vocabulary

## Why Queued

This remains active input, but it is not landed and depends on the orchestration/session contract
and workflow direction being settled.

When implementation is ready, it should be restated against:

- `docs/adr/implemented/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- `docs/adr/implemented/ADR-0028-in-world-process-execution-tracing-parity.md`
- `docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md`
- `docs/adr/draft/ADR-0021-substrate-workflow-engine.md`

## Draft Note

Keep the project-management ADR for archived planning depth, but use this curated draft as the
queued router-service placeholder.
