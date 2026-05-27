# ADR-0028 — In-World Process Execution Tracing Parity

## Status

- Status: Implemented
- Original date (UTC): 2026-01-29
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Shell + World-Agent + World runtime

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

This curated ADR is the stable decision record. The project-management ADR remains as the
planning-rich historical source.

## Decision

Substrate tracing must preserve parity between host and in-world process execution so operators and
downstream tooling can reason about execution trees, exec/exit events, and correlation without
guessing which runtime path produced the record.

The stable decision is:

- in-world execution must emit canonical process execution telemetry
- exec/exit semantics must remain joinable across shell, shim, and world-service paths
- trace fields must preserve stable correlation vocabulary for downstream consumers
- parity gaps are correctness issues, not optional observability enhancements

## Stable Owned Surface

The stable references for this ADR are:

- `docs/internals/trace/schema.md`
- `docs/internals/trace/protocol.md`
- `docs/TRACE.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/trace`
- `crates/common/src/log_schema.rs`
- `crates/shim`
- `crates/world-service`

## Related ADRs

- `docs/adr/implemented/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- `docs/adr/implemented/ADR-0016-world-first-repl-persistent-pty.md`
- `docs/adr/draft/ADR-0029-host-event-bus-and-router-daemon.md`

## Historical Note

The original ADR captures the phased rollout and pack-local execution planning. The stable tracing
parity contract now lives here and in the trace internals docs.
