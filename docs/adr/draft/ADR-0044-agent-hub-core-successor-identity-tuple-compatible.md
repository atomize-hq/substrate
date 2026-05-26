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

## Why Queued

This is active architectural input, but it is not landed and still needs the queue of
orchestration/session/toolbox work around it.

When implementation is ready, it should be restated against:

- `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/adr/implemented/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- `docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md`
- `docs/adr/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`

## Draft Note

Keep the project-management ADR for detailed design reasoning, but treat this curated draft as the
queued Agent Hub successor anchor.
