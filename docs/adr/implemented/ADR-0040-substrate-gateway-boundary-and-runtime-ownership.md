# ADR-0040 — Substrate Gateway Boundary and Runtime Ownership

## Status

- Status: Implemented
- Original date (UTC): 2026-04-02
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Spenser McConnell (Substrate)

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`

This curated ADR is the stable decision record. The project-management ADR remains as the
planning-rich historical source.

## Decision

Substrate owns the trusted boundary around integrated gateway operation, while
`substrate-gateway` owns the in-world runtime behind that boundary.

Substrate owns:

- policy evaluation
- world placement
- lifecycle control
- host-to-world secret delivery
- operator UX
- canonical tracing

`substrate-gateway` owns:

- the in-world front door
- provider, planner, and executor internals
- normalized event generation inside the runtime

This split prevents gateway-local internals from silently becoming Substrate policy or operator
contract truth.

## Stable Owned Surface

The stable references for this ADR are:

- `docs/contracts/gateway/operator-contract.md`
- `docs/contracts/gateway/status-schema.md`
- `docs/contracts/gateway/runtime-parity.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/builtins/world_gateway.rs`
- `crates/world-service/src/gateway_runtime.rs`
- `crates/world-service/src/service.rs`
- `crates/shell/tests/world_gateway.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0027-llm-and-agent-config-policy-surface.md`
- `docs/adr/implemented/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- Historical predecessor kept for context:
  - `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`

## Historical Note

The original ADR contains pack-local planning context and external evidence links that remain
useful historically, but the stable ownership boundary lives here and in the gateway contract docs.
