# ADR-0026 — Orchestration Toolbox (Internal; MCP Protocol)

## Status

- Status: Draft
- Queue state: Queued
- Original date (UTC): 2026-02-09
- Curated into `docs/adr/draft/`: 2026-05-26
- Owner(s): Spenser McConnell (Substrate)

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`

The project-management ADR remains as the planning-rich source retained for compatibility while
`docs/project_management/**` is retired.

This curated ADR remains active architectural input, but it is not implemented and should not be
treated as implementation-ready.

## Queued Direction

This draft proposes an internal MCP toolbox that exposes orchestration-only tools to the
orchestrator agent without requiring bespoke SDK coupling.

The queued direction that still matters is:

- orchestrator-only tool access
- MCP as the internal tool protocol
- a dedicated control-plane toolbox instead of ad hoc integration points

## Why Queued

The core intent remains needed, but the current draft is not the form that should land.

It will require a rewrite when implementation is ready so it can align with:

- `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`
- `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`

Treat this curated draft as the queued placeholder for that future rewrite, not as current
operator/runtime truth and not as a closed historical-only artifact.

## Draft Note

Keep the project-management draft for planning-rich origin context, but resume from this curated
draft when the toolbox implementation slice is ready to be restated and landed.
