# ADR-0017 — Agent Hub Concurrent Execution and Output Routing

## Status

- Status: Implemented
- Original date (UTC): 2026-01-25
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Substrate maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`

This curated ADR is the stable decision record. The project-management ADR remains as the
planning-rich historical source.

## Decision

Substrate must keep PTY byte streams and structured concurrent agent events as distinct output
classes so concurrent execution remains attributable and does not corrupt interactive PTY flows.

The stable decision is:

- PTY streams are forwarded as raw bytes
- structured agent events use a separate structured rendering path
- structured events must not be injected into PTY passthrough
- concurrent attribution must remain explicit and joinable
- buffering and dropped-event summaries must preserve operator clarity under pressure

## Stable Owned Surface

This ADR owns the stable output-routing and event-envelope behavior surfaced through:

- interactive REPL output behavior
- structured agent-event rendering and attribution
- trace correlation expectations for concurrent agent execution

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/execution/agent_events.rs`
- `crates/shell/src/repl/async_repl.rs`
- `crates/world-service/src/pty.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0016-world-first-repl-persistent-pty.md`
- `docs/adr/implemented/ADR-0028-in-world-process-execution-tracing-parity.md`
- `docs/adr/implemented/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/adr/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- `docs/adr/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`

## Historical Note

The original ADR contains pack-local rollout, smoke, and evidence references that remain useful
for historical context, but the stable output-routing contract now lives here.
