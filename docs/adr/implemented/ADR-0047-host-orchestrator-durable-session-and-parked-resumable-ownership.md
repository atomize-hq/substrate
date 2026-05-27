# ADR-0047 — Host Orchestrator Durable Session and Parked-Resumable Ownership

## Status

- Status: Implemented
- Original date (UTC): 2026-05-09
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Spenser McConnell (Substrate)

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md`

This curated ADR is the stable decision record. The project-management ADR remains as the
planning-rich historical source.

## Decision

The durable unit for host orchestration is the Substrate-owned orchestration session and its inbox
or task state, not the lifetime of any one attached backend client process.

The stable decision is:

- host orchestration sessions remain valid after clean client detachment
- parked and resumable posture is explicit persisted runtime truth
- follow-up, approval, and completion delivery must survive attached-client exit
- public prompt surfaces must terminate with explicit terminal envelopes after `Accepted`
- detached-world follow-up remains fail-closed until routed through a valid host owner path

## Stable Owned Surface

This ADR owns the durable host-orchestration posture surfaced through:

- `substrate agent start`
- `substrate agent turn`
- `substrate agent reattach`
- `substrate agent stop`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/execution/agents_cmd.rs`
- `crates/shell/src/execution/agent_runtime/control.rs`
- `crates/shell/src/execution/agent_runtime/session.rs`
- `crates/shell/src/execution/agent_runtime/state_store.rs`
- `crates/shell/src/repl/async_repl.rs`
- `crates/shell/tests/agent_public_control_surface_v1.rs`
- `crates/shell/tests/repl_world_first_routing_v1.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/adr/draft/ADR-0029-host-event-bus-and-router-daemon.md`
- `docs/adr/draft/ADR-0021-substrate-workflow-engine.md`

## Historical Note

The original ADR includes execution-plan framing and migration details. The stable host-session
durability contract now lives here.
