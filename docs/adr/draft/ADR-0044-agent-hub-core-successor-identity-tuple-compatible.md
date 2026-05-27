# ADR-0044 — Agent Hub Core Successor

## Status

- Status: Draft
- Queue state: Queued
- Original date (UTC): 2026-04-03
- Curated into `docs/adr/draft/`: 2026-05-26
- Owner(s): Spenser McConnell (Substrate)

## Curated From

- Planning ADR:
  - `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`

The project-management ADR remains the planning-rich source retained for compatibility while
`docs/project_management/**` is retired.

## Queued Direction

The next Agent Hub contract should preserve backend-id safety while making orchestrator/member
semantics, session handles, and nested gateway-backed identity explicit.

The queued direction that still matters is:

- capability-driven agent backend semantics
- explicit host-scoped orchestrator and world-scoped member model
- identity that keeps `backend_id` separate from provider, auth authority, and protocol

## Folded Contract Detail

When this queued work lands, the successor command-surface intent is:

- the canonical runtime namespace is `substrate agent ...`
- the core read surfaces are:
  - `substrate agent list`
  - `substrate agent status`
  - `substrate agent doctor`
- `substrate agents validate` remains an inventory-validation compatibility leaf rather than a
  plural alias for the successor surfaces
- `backend_id` remains the adapter identifier in `<kind>:<agent_id>` form
- pure-agent rows use `router=agent_hub` and `protocol=substrate.agent.session`
- pure-agent rows omit `provider` and `auth_authority`
- nested gateway-backed rows stay separate from pure-agent rows and are the rows that publish
  `provider` and `auth_authority`
- the intended owner set remains `crates/shell`, `crates/common`, and the `transport-api-*`
  crates rather than a new `crates/agent-hub` crate

## Why Queued

This is active architectural input, but it is not landed and still needs the queue of
orchestration/session/toolbox work around it.

When implementation is ready, it should be restated against:

- `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/adr/implemented/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- `docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md`
- `docs/adr/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`

## Draft Note

This curated draft now carries the queued command-surface summary that previously lived only in the
pack contract file. Keep the project-management ADR as the planning-rich historical source.
