# ADR-0023 — In-World Substrate LLM Gateway (Front Door + Engines)

## Status

- Status: Historical
- Original date (UTC): 2026-02-03
- Curated into `docs/adr/historical/`: 2026-05-26
- Owner(s): Spenser McConnell (Substrate)

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`

This curated ADR is kept only as historical context. The project-management ADR remains as the
planning-rich source retained for compatibility while `docs/project_management/**` is retired.

## Historical Decision Snapshot

This draft proposed an in-world Substrate-owned LLM gateway front door so model egress would stay
inside the world boundary when world mode is enabled.

The historical shape matters because it established the original goals behind:

- keeping gateway egress inside the boundary
- avoiding policy bypass through host-local routing
- making world placement part of the operator-facing contract

## Why Historical

This proposal is no longer the current architectural truth.

Its runtime ownership assumptions were superseded by:

- `docs/adr/implemented/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/adr/implemented/ADR-0041-substrate-gateway-backend-adapter-contract.md`

Those successor ADRs preserve the boundary goals while replacing the older single-ADR framing with
an explicit Substrate-versus-`substrate-gateway` ownership split and adapter contract.

## Historical Note

Keep the original draft for archived gateway-capability planning context, not as a live runtime or
operator contract.
