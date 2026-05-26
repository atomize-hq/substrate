# ADR-0022 — Forge as a Workflow Node

## Status

- Status: Draft
- Queue state: Queued
- Original date (UTC): 2026-02-03
- Curated into `docs/adr/draft/`: 2026-05-26
- Owner(s): Substrate maintainers

## Curated From

- Planning ADR:
  - `docs/project_management/adrs/draft/ADR-0022-forge-agent-loop-as-workflow-node.md`

The project-management ADR remains the planning-rich source retained for compatibility while
`docs/project_management/**` is retired.

## Queued Direction

Forge remains a queued composite node concept that should live inside a broader Substrate workflow
runtime rather than becoming the workflow engine itself.

The queued direction that still matters is:

- a bounded iterative agent-loop node
- nested spans and traceability under the core workflow runtime
- reuse of stable config/policy and tracing contracts instead of bespoke execution plumbing

## Why Queued

This is active architectural input, but it is not landed and depends on the workflow/runtime
cluster being settled first.

When implementation is ready, it should be restated against:

- `docs/adr/draft/ADR-0021-substrate-workflow-engine.md`
- `docs/adr/implemented/ADR-0028-in-world-process-execution-tracing-parity.md`
- `docs/adr/implemented/ADR-0027-llm-and-agent-config-policy-surface.md`

## Draft Note

Keep the project-management ADR for the original Forge framing, but treat this curated draft as a
queued derivative of the workflow-engine slice.
