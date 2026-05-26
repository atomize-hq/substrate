# ADR-0045 — Orchestration Toolbox Internal MCP Identity and Trace Contract

## Status

- Status: Draft
- Queue state: Queued
- Original date (UTC): 2026-04-03
- Curated into `docs/adr/draft/`: 2026-05-26
- Owner(s): Spenser McConnell (Substrate)

## Curated From

- Planning ADR:
  - `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`

The project-management ADR remains the planning-rich source retained for compatibility while
`docs/project_management/**` is retired.

## Queued Direction

The toolbox remains queued as an internal MCP control-plane surface with explicit identity and
trace semantics, layered on top of the newer Agent Hub and tuple model.

The queued direction that still matters is:

- orchestrator-only toolbox access
- introspection-only v1 toolbox posture
- MCP protocol as the internal toolbox wire contract
- explicit control-plane identity and trace joinability

## Why Queued

This is active architectural input, but it is not landed and depends on the surrounding
orchestration/session work being finalized.

When implementation is ready, it should be restated against:

- `docs/adr/draft/ADR-0026-orchestration-toolbox-mcp.md`
- `docs/adr/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/adr/implemented/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- `docs/adr/implemented/ADR-0028-in-world-process-execution-tracing-parity.md`

## Draft Note

Keep the project-management ADR for detailed control-plane design, but use this curated draft as
the queued toolbox-contract placeholder.
