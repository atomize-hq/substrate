# ADR-0046 — Gateway Backend Selection Runtime Integration

## Status

- Status: Implemented
- Original date (UTC): 2026-04-21
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Spenser McConnell (Substrate)

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`

This curated ADR is the stable decision record. The project-management ADR remains as the
planning-rich historical source.

## Decision

The integrated gateway lifecycle must realize ADR-0041 through runtime selection, adapter binding,
capability gating, and auth handoff for the supported integrated backend set.

The stable decision is:

- `llm.routing.default_backend` remains the selected backend surface
- allowlisting and auth-read policy gates apply before runtime realization
- integrated lifecycle resolves one adapter binding for the selected backend
- missing binding, unsupported capabilities, or unavailable auth material fail closed
- runtime config and auth handoff are adapter-driven rather than one-off operator glue

The implemented scope can widen over time, but supported backends must continue to follow this
same realization contract.

## Stable Owned Surface

This ADR realizes the contract surfaces documented in:

- `docs/contracts/gateway/backend-adapter-selection.md`
- `docs/contracts/gateway/backend-adapter-protocol.md`
- `docs/contracts/gateway/backend-adapter-schema.md`
- `docs/contracts/gateway/operator-contract.md`
- `docs/contracts/gateway/status-schema.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/builtins/world_gateway.rs`
- `crates/world-service/src/gateway_runtime.rs`
- `crates/world-service/src/service.rs`
- `crates/shell/tests/world_gateway.rs`
- `crates/world-service/tests/gateway_runtime_parity.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0027-llm-and-agent-config-policy-surface.md`
- `docs/adr/implemented/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/adr/implemented/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/adr/implemented/ADR-0043-adr-0027-identity-tuple-policy-surface.md`

## Historical Note

The original ADR captures the planning and rollout framing for this implementation seam. Keep using
this curated ADR for the stable realization contract and the project-management ADR only for
historical execution context.
