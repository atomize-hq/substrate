# ADR-0024 — CLI Backend Provider Engine (Subscription-First Cross-Routing)

## Status

- Status: Historical
- Original date (UTC): 2026-02-03
- Curated into `docs/adr/historical/`: 2026-05-26
- Owner(s): Spenser McConnell (Substrate)

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`

This curated ADR is kept only as historical context. The project-management ADR remains as the
planning-rich source retained for compatibility while `docs/project_management/**` is retired.

## Historical Decision Snapshot

This draft proposed treating subscription-authenticated CLIs as provider backends behind a
Substrate-local engine layer so cross-provider routing could work without forcing API keys.

The historical shape matters because it captured the product goal of:

- stable backend selection for CLI-backed fulfillment
- subscription-first usage
- controlled cross-routing inside the trusted boundary

## Why Historical

The product goal remains relevant, but the engine architecture did not stay authoritative.

Its stable successor is:

- `docs/adr/implemented/ADR-0041-substrate-gateway-backend-adapter-contract.md`

The current architecture keeps the stable backend-id and allowlisting posture while replacing the
older Substrate-local engine assumption with a gateway-owned adapter contract under the ownership
split from:

- `docs/adr/implemented/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`

## Historical Note

Keep the original draft for early CLI-backend strategy context, not as the current adapter or
runtime contract.
